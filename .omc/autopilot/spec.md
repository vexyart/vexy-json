---
this_file: .omc/autopilot/spec.md
status: accepted-for-planning
---

# Build, install, and publish workflow specification

## Problem

Vexy JSON needs three trustworthy root commands:

- plain `./build.sh` builds and tests the complete Rust workspace and the canonical Python package at `crates/python`;
- plain `./install.sh` installs the `vexy-json` CLI and Python binding into user-writable locations, without `sudo`, and reports the environment it changed;
- plain `./publish.sh` creates one patch release and actually publishes the supported Rust crates to crates.io and `crates/python` to PyPI, while `./publish.sh --dry-run` exercises all non-upload package gates.

Release creation is consequential because the required versioning tool stages, commits, tags, and pushes unconditionally. Build and install must therefore be separated completely from release mutation. Registry publication is also non-atomic: a failure after one crate upload cannot be rolled back safely.

Current repository evidence makes a rewrite of the command surface necessary:

- The workspace includes `crates/python` and already excludes stale `bindings/python` (`Cargo.toml:3-12`).
- `build.sh` currently updates tracked version files during a release build (`build.sh:149-182`) and contains a macOS-only `sudo` installer with inconsistent `vexy-json`/`vexy_json` paths (`build.sh:185-212`).
- The legacy release implementation stages everything and creates its own tag (`scripts/release.sh:446-481`) but deliberately does not publish crates (`scripts/release.sh:583-618`).
- Every internal Cargo dependency is path-only, which is unsuitable for crates.io packages; for example the root uses path-only core and serde dependencies (`Cargo.toml:27-33`).
- `crates/python/pyproject.toml` declares a Maturin backend and a dynamic version (`crates/python/pyproject.toml:1-29`) but has no functional hatch-vcs version provider.
- All eight workspace packages currently report version `1.5.10`; the newest local tag is `v1.5.10`, so the next patch prediction in the current snapshot is `1.5.11`. The script must calculate this live and never hard-code it.
- The exact live command `lms ls --json --llm | jq` fails at column/byte 65,536. The same `lms` command redirected to a regular file produces valid 153,247-byte JSON. This is an upstream flush/pipe failure: a downstream parser can make the received prefix syntactically valid, but it cannot reconstruct bytes that `lms` never wrote.
- The verified repository baseline is not globally formatting/lint-clean: full-workspace `rustfmt --check` currently reports 249 pre-existing diffs, and warning-denied workspace Clippy has known legacy debt. This task must not reformat the workspace or claim that debt is resolved.
- On macOS, generic Cargo workspace build/test/doc/example commands cannot link the `vexy-json-python` package with PyO3's `extension-module` feature correctly outside Maturin. That package must be excluded from only those generic Cargo invocations and then built and tested mandatorily as the canonical Maturin wheel; this is a toolchain boundary, not a skipped Python gate.

## Constraints

### Command and scope boundaries

1. The supported build/install/release surface is exactly root `build.sh`, `install.sh`, and `publish.sh`.
2. Remove root `release.sh`, `scripts/release.sh`, and `scripts/release.sh.backup`. Remove supported-documentation instructions that invoke any `release.sh`. Do not implement `publish.sh` as a wrapper around legacy release code.
3. `crates/python` is the only Python binding that may be built, installed, tested, versioned, or published. `bindings/python` remains excluded from the workspace and every active command. Do not silently delete archival content from `bindings/python`; remove or label supported documentation that directs users there.
4. Keep scope to shell orchestration, necessary manifest/package fixes, focused tests, and source-of-truth documentation. Do not add GitHub Release, npm, Homebrew, DMG, analytics, retry frameworks, or CI redesign.
5. No live registry publication is permitted during implementation, QA, or review. Test real-mode orchestration with temporary repositories and fake executables; execute the real command only when a human intentionally runs plain `./publish.sh` with real credentials.
6. The installed CLI must address the observed `lms` failure truthfully in two distinct modes: explicit lossy EOF repair for piped prefixes, and a lossless `--lms` capture path that gives the child a regular-file stdout. JSON remains on stdout; repair/capture diagnostics remain on stderr so downstream `jq` is not contaminated.
7. Quality enforcement is baseline-aware: apply strict rustfmt and warning-denied Clippy to the changed `vexy-json-cli` package, but do not make full-workspace formatting or warning cleanup an acceptance condition. Generic Cargo build, tests, doc-tests, and examples remain mandatory for the workspace with only `vexy-json-python` excluded; the excluded package has its own mandatory Maturin wheel build/install/test gate. Record workspace normalization as explicit follow-up work.

### Safety and repeatability

1. Use `set -euo pipefail`, resolve the repository root from each script's own location, quote paths, and stop on the first failed gate.
2. `build.sh` and `install.sh` must never invoke `gitnextver`, `git add`, `git commit`, `git tag`, `git push`, a registry upload, or a version-update helper. They must not change tracked source files.
3. Re-running build or install must be safe. Build artifacts may be replaced. Install must upgrade/replace the same user-owned CLI and virtual environment rather than creating an unbounded series of environments.
4. Build logging must preserve the failing command's exit status and write `build.log.txt`; the caller must be able to inspect all warnings/errors after `./build.sh`.
5. Do not use `sudo`, system Python, or system site-packages. Never print, trace, or place registry tokens in command arguments.
6. A real release requires a completely clean worktree, including untracked files, before any scripted source mutation. The script must not delete, stash, commit, or ignore unexpected files to make the tree clean. The current untracked `.codegraph/` state would correctly block a real release until the repository owner resolves it.

### Version invariants

1. The release tag is the public version source. Configure hatch-vcs in `crates/python` and query it independently; the Maturin-built Python artifact and synchronized Cargo manifests must equal that tag-derived result.
2. Before invoking gitnextver in real mode, calculate the next patch from canonical SemVer tags, then synchronize every workspace Cargo `[package].version` to that predicted version. Synchronize every internal registry dependency requirement to exact `=X.Y.Z` while retaining its local `path`.
3. The synchronized Cargo files must be the only tracked changes created by release preparation. Assert an explicit path allowlist before gitnextver; never let its unconditional `git add` absorb unrelated changes.
4. Do not manually commit, tag, or push the synchronized version changes. The one authorized release mutation command is exactly, once:

   ```bash
   uvx gitnextver@1.0.1 --directory . --verbose
   ```

   It takes only `--directory` and `--verbose`; do not invent version, dry-run, or push flags. It must never run in build, install, `publish.sh --dry-run`, or tests against the live checkout.
5. The new local tag must equal `v${predicted_version}`. The local tag target, local `HEAD`, the live `origin` branch ref, and the live remote tag target must all identify the same commit before the first registry upload. For an annotated tag, compare its peeled `^{}` commit; for a lightweight tag, compare the direct ref. Do not rely only on a success banner or a potentially stale local remote-tracking ref.
6. After tagging, run the standalone hatch-vcs query `uvx --with hatch-vcs hatchling version` in `crates/python`. The result, final Python wheel `METADATA` version, `vexy_json.__version__`, the created tag without `v`, and every synchronized Cargo package version must all be equal.

### Python backend compatibility gate

Keep Maturin as the build backend for the PyO3 extension. Hatch-vcs cannot inject a version into the Maturin backend, whose dynamic version comes from Cargo, so the truthful integration is a dual-source equality gate rather than a fake backend hook:

```toml
[tool.hatch.version]
source = "vcs"

[tool.hatch.version.raw-options]
root = "../.."
```

From `crates/python`, `uvx --with hatch-vcs hatchling version` resolves the repository-root Git version. Release preparation synchronizes Cargo to the predicted patch; gitnextver creates the tag; the standalone hatch-vcs query proves the tag-derived version; Maturin builds from the synchronized Cargo version; artifact inspection proves both paths converge on the same exact version. The Hatch table is therefore an executable release assertion, not a claim that Hatch builds the extension.

Explicitly reject switching the native build to `hatchling-pyo3-plugin` or `hatch-maturin-build` in this iteration: current official research found these alternatives too immature for the release-critical path. Also reject copying a static Python version or claiming hatch-vcs directly controls Maturin. Completion requires the standalone hatch-vcs result, Cargo version, wheel metadata, and runtime version equality proof.

## Options

### Option A: extend the existing build and release scripts

This preserves familiar filenames and some old packaging behavior, but it retains a mixed build/install/release lifecycle, stale `bindings/python` paths, interactive/eval-based release logic, and multiple tag authorities. It also leaves `release.sh` as a competing command. Rejected.

### Option B: three root orchestrators with explicit package policy

Rewrite `build.sh`, add `install.sh` and `publish.sh`, delete the release.sh workflow, and let each root script own one lifecycle. Keep commands visible and linear; use at most one small private helper for deterministic Cargo manifest synchronization if direct shell editing would be unsafe. This duplicates a little validation but makes mutation authority obvious and testable. Selected.

### Option C: a general release framework/configuration layer

A shared command framework could model stages and resume state, but it adds abstractions before the basic release works and obscures the exact destructive boundary. Rejected for this utility.

## Decision

### 1. `build.sh`: deterministic build and test

Plain `./build.sh` is the only complete build gate. It must:

1. validate required tools and the canonical workspace layout;
2. record resolved `rustc`, `cargo`, `uv`, Python, and native-extension build-tool versions in `build.log.txt`;
3. run `cargo fmt -p vexy-json-cli -- --check` and warning-denied Clippy for `vexy-json-cli` with its relevant targets/features; then build, test, doc-test, and compile examples for the Cargo workspace with `--exclude vexy-json-python` on the applicable generic workspace commands (the project rule remains: users run `./build.sh`, not those Cargo commands directly);
4. build canonical `crates/python` separately using Maturin through `uv build --no-sources`; this mandatory wheel path replaces only the invalid generic Cargo link path for the PyO3 extension and does not weaken Python coverage;
5. install the built Python artifact into a fresh temporary virtual environment, run the canonical Python test suite with real collection, and smoke-test import, parsing, and `__version__`;
6. verify the Git tracked/untracked status is identical to its entry baseline, apart from ignored build outputs, and write a clear success/failure summary to `build.log.txt`.

The existing canonical Python tests currently include stale imports (`crates/python/tests/test_vexy_json.py:4-13`). Fix or replace those tests so the gate exercises the actual exported API; do not exclude Python to obtain a green build.

### 2. `install.sh`: user-owned CLI plus isolated Python environment

Plain `./install.sh` installs from the current checkout without changing it:

1. Default the install prefix to a documented user-writable location (recommended `${VEXY_JSON_PREFIX:-$HOME/.local}`) and allow an explicit `--prefix PATH` override.
2. Install `crates/cli` with Cargo under that prefix, using the actual binary name `vexy-json`; repeated runs replace the same installation.
3. Create/reuse one isolated virtual environment under the prefix (recommended `$prefix/share/vexy-json/venv`) and install the canonical `crates/python` artifact into it with uv. Never install from `bindings/python`.
4. Verify `$prefix/bin/vexy-json --version`, Python import, a representative forgiving parse, and the installed Python version.
5. Print resolved tool versions, prefix, CLI path, virtual-environment path, Python executable, and the exact PATH/activation guidance. Do not modify shell startup files automatically.

### 3. Installed CLI behavior for truncated `lms` output

Provide both truthful workflows because they solve different problems:

1. **Explicit lossy repair:**

   ```bash
   lms ls --json --llm | vexy-json --repair --compact | jq
   ```

   When EOF arrives inside an open JSON structure, `--repair` may close only the structures/tokens that can be repaired from the received prefix and emit syntactically valid compact JSON. If repair succeeds, stdout contains JSON and stderr must prominently state that input ended unexpectedly, report the received byte offset/repair actions, label the result partial/lossy, and say that missing upstream bytes were not recovered. Without explicit `--repair`, unexpected EOF remains an error. Never invent array items, object members, string contents, or a claim of completeness.

2. **Lossless dedicated capture:**

   ```bash
   vexy-json --lms --compact | jq
   ```

   `--lms` executes the fixed argv `lms ls --json --llm` without a shell. Give the child a securely created regular temporary file as stdout, preserve/forward its stderr, wait for successful exit, rewind/read the file, parse it, and emit the requested Vexy JSON formatting to stdout. Report capture source and byte count on stderr, clean up the temporary file, and propagate missing-command, spawn, I/O, nonzero-child, parse, and downstream broken-pipe failures accurately. `--lms` conflicts with positional input files and stdin/watch/NDJSON modes; ordinary formatting flags such as `--compact` remain available.

The capture mode is the preferred answer when the complete `lms` dataset matters. Repair mode exists for already-truncated streams and guarantees syntax, not data recovery.

### 4. Rust publication policy

Publication is an explicit allowlist, never `cargo publish --workspace`:

| Wave | Cargo package | Reason |
|---|---|---|
| 1 | `vexy-json-core` | no internal dependency |
| 2 | `vexy-json-serde` | depends on core; root's default feature needs it |
| 3 | `vexy-json-cli` | depends on core |
| 3 | `vexy-json-wasm` | depends on core |
| 3 | `vexy-json-c-api` | depends on core |
| 4 | root `vexy-json` | depends on core and optional/default serde |

Wave 3 is logically parallel but publish sequentially for deterministic logs and recovery. Mark `vexy-json-test-utils` and `vexy-json-python` with `publish = false`: test utilities are internal, and the Python cdylib is delivered through PyPI. Add complete crates.io metadata to every allowlisted package. Each allowlisted internal path dependency must also include exact `version = "=X.Y.Z"` so the packaged crate resolves from crates.io.

### 5. `publish.sh --dry-run`: packageability without release mutation

The dry run must perform useful work, not merely echo commands:

1. run read-only repository/branch/tag/tool preflight and reject a dirty tree;
2. run `./build.sh`;
3. run `cargo publish --dry-run` for every Rust allowlisted package in dependency order;
4. run `uv build --no-sources` for `crates/python`, inspect wheel/sdist contents and version metadata, install the wheel into a fresh environment, and run Python smoke/tests;
5. run `uv publish --dry-run` against the explicitly selected Python artifacts;
6. print the predicted next patch and the exact real-mode plan.

These gates may create ignored local build artifacts such as Cargo `target/` and Python `dist/`; "non-mutating" here means no tracked-source, Git-history/ref, remote-ref, or registry mutation. The dry run must never synchronize versions, invoke gitnextver, invoke `cargo publish` without `--dry-run`, invoke `uv publish` without `--dry-run`, tag, commit, or push. Upload credentials are not required for dry-run mode.

Because dependent crates at an unpublished target version may not yet be resolvable from crates.io during Cargo package verification, dry-run validates the current coherent manifest graph. Real mode must re-run the relevant `cargo publish --dry-run` immediately before each actual package upload after its prerequisite wave is visible. Do not bypass verification with `--no-verify` merely to hide registry propagation.

### 6. Plain `publish.sh`: real release state machine

Real mode is linear and fail-closed:

1. **Read-only preflight:** require the expected release branch (`main` unless documented otherwise), configured upstream `origin/main`, local `HEAD` equal to the live remote branch, a clean worktree including untracked files, required tools, supported package metadata, no existing predicted local/remote tag, and nonempty publish credentials. Require `CARGO_REGISTRY_TOKEN` and `UV_PUBLISH_TOKEN` for the minimal deterministic contract; never echo them or enable shell tracing.
2. **Pre-mutation gates:** run the same build and non-upload package gates as dry-run against the current coherent versions. Abort before changing versions if any gate fails.
3. **Prepare target version:** calculate the next patch from live canonical tags. Synchronize all workspace Cargo package versions and exact internal dependency requirements. Do not write a Python static version. Assert target equality and that the Git diff contains only the exact manifest allowlist.
4. **Target build gate:** run `./build.sh` again with the synchronized Cargo versions. Abort before gitnextver on failure, leaving a clearly reported, reviewable manifest diff and making no attempt to clean it automatically.
5. **Create release:** invoke exactly `uvx gitnextver@1.0.1 --directory . --verbose`. Invoke it once. It exclusively owns staging, the release commit, tag creation, and pushing.
6. **Verify release refs:** require a clean post-command tree; require exactly the predicted local tag on `HEAD`; compare `HEAD`, the peeled local tag commit, live `origin` branch hash, and live remote tag commit. Any mismatch blocks all uploads.
7. **Publish Rust:** for each wave, run `cargo publish --dry-run` and then actual `cargo publish` for the same explicit package. Wait for/check registry visibility before publishing a dependent wave; do not use blind fixed sleeps.
8. **Publish Python:** rebuild `crates/python` from the verified release tag with `uv build --no-sources`; inspect artifact versions and imports; run `uv publish --dry-run`; then run actual `uv publish` only for the inspected wheel/sdist paths.
9. **Report:** list release commit/tag and every successfully published `name@version`. Exit nonzero if any intended artifact is not confirmed.

### 7. Failure and recovery policy

- Before gitnextver: fail with no Git or registry side effects. If version synchronization already changed manifests, preserve and report the allowlisted diff; never discard it automatically.
- After gitnextver but before upload: preserve the verified release commit/tag and report which ref check failed. Never delete or rewrite a local/remote tag automatically.
- During crates.io/PyPI publication: publication is irreversible and may have succeeded despite a client timeout. Stop on the first failure; print confirmed completed artifacts, the ambiguous artifact, remaining artifacts, and read-only registry-verification guidance. Do not blindly retry, yank, unpublish, retag, or restart the entire sequence.
- A later recovery is a deliberate operator action after checking registry truth. Do not add opaque local resume state in the first implementation; the Git tag and registries are authoritative.

## Edge cases

- Dirty tracked or untracked files, including generated but unignored `.codegraph/`, block dry-run and real publish without cleanup.
- Detached HEAD, wrong branch, missing `origin`, missing upstream, local/remote branch divergence, shallow/missing tags, malformed tags, and an existing predicted tag all fail before source mutation.
- gitnextver returns success but the branch or tag push is rejected: independent live remote-ref equality still blocks upload.
- gitnextver creates a different tag than predicted, more than one new release tag, or a tag not pointing at `HEAD`: block upload.
- An annotated tag's object hash differs from its commit hash: compare the peeled commit; support a lightweight tag only by comparing its direct commit.
- Cargo manifests agree on package versions but an internal dependency requirement is stale: fail version synchronization validation.
- A package is accidentally publishable outside the allowlist: require `publish = false` for test-utils and the Rust Python cdylib and never use workspace-wide publish.
- crates.io has not indexed a newly uploaded prerequisite: poll/query with a bounded deadline, then stop with partial-state guidance; do not publish dependents prematurely.
- PyPI already contains one identical artifact after an interrupted upload: verify exact filename/content before using uv's documented idempotent retry behavior; conflicting content fails closed.
- Python builds successfully but wheel `METADATA`, sdist name, Cargo version, or runtime `__version__` disagree: block PyPI upload.
- Missing compilers/Python headers, unsupported Python, unwritable prefix, spaces in paths, or an active unrelated virtual environment: quote paths, use the script-owned interpreter explicitly, and fail with remediation.
- Piped input stops at exactly 65,536 bytes while inside a JSON string or UTF-8 code point: repair must not fabricate the missing suffix. Invalid/incomplete UTF-8 needs an explicit error rather than lossy replacement disguised as source data.
- EOF repair receives a prefix with no structurally safe partial value: fail rather than emit misleading JSON. All repair diagnostics go to stderr and name the result as partial.
- `--lms` sees a missing executable, child nonzero exit, a zero/oversized capture, invalid full JSON, temp-file failure, or interrupted child: return nonzero, report the exact boundary, and clean up only the script-owned temporary file.
- `jq` or another downstream consumer closes early: handle broken pipe without a panic or a false success message; never confuse a downstream close with complete processing.

## Test strategy and acceptance criteria

### Static and contract tests

- `bash -n build.sh install.sh publish.sh` passes.
- Shell tests run from outside the repository root and from a path containing spaces.
- Source scans prove `build.sh` and `install.sh` contain no gitnextver, version-update, publish, push, or sudo path; all three active scripts and docs contain no `bindings/python` execution path.
- Root `release.sh`, `scripts/release.sh`, and `scripts/release.sh.backup` are absent, and supported docs contain no `release.sh` instructions.
- Cargo metadata proves the six-package allowlist, `publish = false` for test-utils/Python, one synchronized package version, and exact internal dependency requirements.

### Build acceptance

1. Capture `git status --porcelain=v1 --untracked-files=all` and tracked diff.
2. Run plain `./build.sh` only.
3. Inspect `build.log.txt` directly; require strict CLI-package rustfmt/Clippy, generic Cargo workspace build/test/doc-test/example results excluding only `vexy-json-python`, and the separate mandatory Maturin Python wheel build, wheel install, pytest collection, and import/parse/version smoke. The log must identify the PyO3/Maturin boundary rather than describe Python as skipped. It must also label the 249 workspace rustfmt diffs and legacy workspace Clippy debt as the verified baseline/follow-up, not claim global cleanliness; any new CLI-package formatting or warning failure remains blocking.
4. Assert the Git status/diff matches the captured baseline and no `bindings/python` output was used.
5. Run `./build.sh` a second time and require the same success contract.

### Install acceptance

- In a temporary user-owned prefix, run plain `./install.sh --prefix "$test_prefix"` twice.
- Require no sudo/system-site mutation, executable `$test_prefix/bin/vexy-json`, a working isolated Python interpreter, import/parse/version smoke, and accurate PATH/activation instructions.
- PATH shims fail the test if install invokes gitnextver, Git mutation, or registry publication.

### Installed CLI and `lms` acceptance

- Run all UX checks through the installed `$test_prefix/bin/vexy-json`, not only `cargo run`.
- Preserve the reported live baseline as acceptance evidence: `lms ls --json --llm` redirected directly to a regular file is valid JSON of exactly 153,247 bytes; the same stdout sent through a pipe truncates at exactly 65,536 bytes and direct `jq` fails at column 65,536.
- Require `lms ls --json --llm | $test_prefix/bin/vexy-json --repair --compact | jq` to produce syntactically valid partial JSON. Capture stderr and assert it names unexpected EOF, byte 65,536, repair actions, partial/lossy output, and the fact that missing bytes were not recovered. The test must not assert semantic equality with the full baseline.
- Require `$test_prefix/bin/vexy-json --lms --compact | jq` to succeed, report a 153,247-byte regular-file capture on stderr, and produce JSON semantically equal to the direct-redirection baseline, including a sentinel from the end of the full dataset.
- Add a deterministic fake `lms` earlier in PATH for CI: it emits a valid 153,247-byte dataset only when stdout is a regular file and emits an unclosed 65,536-byte prefix when stdout is a FIFO/pipe. Test repair truthfulness, capture completeness, missing executable, nonzero child, invalid output, temp cleanup, and downstream early close without depending solely on mutable live `lms` data.

### Dry-run acceptance

- In a clean temporary repository, run `./publish.sh --dry-run` with logging shims around `git`, `cargo`, `uv`, and `uvx`.
- Require all six `cargo publish --dry-run` gates, canonical `uv build --no-sources`, Python artifact/import tests, and `uv publish --dry-run`.
- Require zero git commit/tag/push, zero version-file mutation, zero gitnextver invocation, and zero non-dry registry command. Compare refs, tracked files, and remote sentinel state before/after.
- Repeat with dirty tree, wrong branch, divergent remote, existing tag, stale Cargo requirement, Python version mismatch, and absent tool; each fails before a forbidden action with a specific message.

### Real-mode orchestration acceptance without live publication

- Use an isolated bare `origin`, temporary clone, deterministic tags, and fake registry commands. Never point QA at crates.io or PyPI.
- Assert ordering: read-only preflight -> package gates -> Cargo synchronization -> target build -> exact single gitnextver command -> local/branch/remote-tag equality -> Rust waves -> final Python artifact checks -> PyPI upload.
- Assert the synchronized target for a `v1.5.10` fixture is exactly `1.5.11`, the new tag is exactly `v1.5.11`, and Cargo, wheel metadata, and runtime versions all equal `1.5.11`.
- From `crates/python`, assert `uvx --with hatch-vcs hatchling version` is exactly `1.5.11` on that tag and equals Cargo, wheel `METADATA`, sdist, and runtime `__version__`; prove Maturin remains the active backend.
- Simulate a gitnextver push rejection, wrong tag, annotated tag, lightweight tag, missing credential, registry-index delay, Cargo upload timeout, and PyPI partial upload. No scenario may upload before ref equality or automatically delete/rewrite/yank/unpublish anything.

### Final verification gate

After implementation, run plain `./build.sh`, inspect `build.log.txt`, run only `./publish.sh --dry-run` against real package tooling, run real-mode tests with fakes, and obtain fresh Architect approval. Do not run a live plain `./publish.sh` as QA evidence.

## Documentation and file metadata

Update the active source-of-truth documentation:

- `README.md`: the three-command quick start and environment destinations;
- `RELEASE_GUIDE.md` and `docs-src/dev/release-process.md`: one release authority, credentials, exact gitnextver boundary, dry-run semantics, package order, and partial-failure recovery;
- `docs-src/dev/build-process.md` and `VERSIONING.md`: build/install separation and tag/Cargo/hatch-vcs equality;
- `crates/python/README.md`: canonical local install/build/test and tag-derived version behavior;
- `docs-src/user/getting-started.md` and the active troubleshooting guide: installed `--repair` versus lossless `--lms` UX, stderr repair warnings, and the explicit no-recovery-of-missing-bytes guarantee;
- `CHANGELOG.md`, `WORK.md`, `TODO.md`, `docs/internal/PLAN.md`, and `DEPENDENCIES.md` (create if absent): completed workflow change, remaining verified work, the purpose/status of uv, Maturin, and hatch-vcs, and an explicit follow-up for the 249 pre-existing rustfmt diffs plus warning-denied workspace Clippy debt.

Do not rewrite historical changelog entries merely because they mention an old release mechanism. Remove active instructions and label retained historical material as superseded where ambiguity remains. Generated documentation output is rebuilt from source, not hand-edited.

Every new or touched source file must carry its project-relative `this_file` record near the top. Shell files place `# this_file: path` immediately after the shebang. TOML/Python/Rust/test helpers use the appropriate leading comment. Touched Markdown uses YAML frontmatter with `this_file: path`. Verify these paths after file moves or deletions.

## External facts to retain during implementation

- Cargo recommends `cargo publish --dry-run`; path-only dependencies are not publishable and need registry versions alongside local paths.
- uv recommends `uv build --no-sources` for publishable packaging and supports `uv publish --dry-run`; PyPI credentials may be supplied as `UV_PUBLISH_TOKEN`.
- crates.io credentials may be supplied as `CARGO_REGISTRY_TOKEN`.
- gitnextver is pinned to `1.0.1`, accepts only the specified directory/verbose options here, and unconditionally stages, commits, tags, and pushes. Its output is never sufficient evidence of remote success.
- Maturin remains the Python build backend; standalone hatch-vcs/Hatchling calculates the tag-derived comparison version via `uvx --with hatch-vcs hatchling version`. The build is releasable only when both version paths agree.
- The observed `lms` pipe loses everything after byte 65,536 while regular-file redirection yields valid 153,247-byte JSON. Parsing can repair syntax in the received prefix, never reconstruct unwritten bytes; `--lms` avoids the upstream failure by capturing to a regular file.
