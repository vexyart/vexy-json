---
this_file: .omc/plans/autopilot-impl.md
status: implementation-plan
source_spec: .omc/autopilot/spec.md
---

# Build, install, and publish workflow plan

## Goal

Deliver exactly three supported root commands: deterministic `build.sh`, user-owned `install.sh`, and fail-closed `publish.sh`; support truthful CLI `--repair` and lossless `--lms`; retain `crates/python` as the only Python binding; never publish during implementation or QA.

## Ownership and order

1. **Executor B — release, packaging, and workflow foundation**
   - Own only `build.sh`, `install.sh`, `publish.sh`, release helpers/tests, all Cargo manifests other than `crates/cli/Cargo.toml`, `crates/python` packaging/tests, and deletion of `release.sh`, `scripts/release.sh`, and `scripts/release.sh.backup`.
   - Replace the mixed legacy command flow with root-relative, quoted, `set -euo pipefail` scripts. Build logs to `build.log.txt`, never mutates tracked files, runs generic Cargo workspace commands with `--exclude vexy-json-python`, and gates every non-Python Rust member plus the canonical Maturin Python wheel/test/import/version path.
   - Implement install into `${VEXY_JSON_PREFIX:-$HOME/.local}` or `--prefix`, with a replaceable CLI and one prefix-owned Python venv; no `sudo`, system Python, shell-startup edits, Git mutation, or publishing.
   - Configure all publishable Cargo packages with crates.io metadata and `path` plus exact `version = "=X.Y.Z"` internal requirements; set `publish = false` for test-utils and Rust Python cdylib. Configure standalone hatch-vcs in `crates/python` while retaining Maturin as backend; make Cargo, Hatch, artifact metadata, and runtime versions equality-gated.
   - Implement dry-run preflight, build and package gates, explicit six-package publication ordering, and real-mode release state machine. The sole release mutation command is exactly `uvx gitnextver@1.0.1 --directory . --verbose`, once, after manifest allowlist validation and target rebuild. Verify peeled/direct local tag, HEAD, live origin branch, and remote tag all identify one commit before uploads. Use fake toolchains and temporary bare origins for every real-mode test.
   - Add release contract tests for dirty/untracked state, divergent refs, tag/version mismatch, exact allowed diff, cargo/Python dry-run commands, dependency waves/index delay, partial upload reporting, and absence of legacy scripts.

2. **Executor A — CLI feature and focused integration tests** *(starts after B establishes the install/build contract)*
   - Own only `crates/cli/src/main.rs`, `crates/cli/Cargo.toml`, and `crates/cli/tests/**`.
   - In `crates/cli/Cargo.toml`, own the CLI's crates.io metadata, synchronized package version, publication configuration, and exact `path` plus `version = "=X.Y.Z"` core dependency; coordinate the shared target version and release allowlist with Executor B without editing B-owned manifests.
   - Add explicit `--repair`: ordinary unexpected EOF remains an error; repair emits only safely closed syntax from received bytes, preserves JSON-only stdout, and emits byte offset, repair action, partial/lossy status, and no-missing-data warning on stderr. Reject incomplete/invalid UTF-8 and unsafe partial values.
   - Add `--lms`: execute fixed `lms ls --json --llm` without a shell; use a securely owned regular temp file for child stdout, forward child stderr, require success, parse after rewind, clean only its temp file, and report capture byte count on stderr. Reject conflicting positional/stdin/watch/NDJSON input modes; propagate missing command, child, I/O, parse, and broken-pipe failures.
   - Add deterministic fake-`lms` tests proving a 153,247-byte valid regular-file capture versus an unclosed 65,536-byte pipe prefix; cover repair truthfulness, capture semantic equality/sentinel, missing/nonzero/invalid child, cleanup, and downstream close through the installed binary where the release harness exposes it.
   - Require strict `cargo fmt --check` and Clippy with warnings denied for the changed CLI Rust surface; record the 249 pre-existing workspace rustfmt diffs and legacy workspace Clippy debt as explicit follow-up, rather than blocking this scoped change on unrelated baseline debt.

3. **Executor C — documentation only** *(starts after A and B contract tests pass)*
   - Own only active source documentation: root quick-start/release/version/build docs, Python README, getting-started/troubleshooting, and project status/dependency records named in the specification.
   - State the three commands, canonical `crates/python`, user-owned destinations, one gitnextver authority, credentials, dry-run/non-atomic recovery behavior, package order, Hatch/Maturin equality gate, and `--repair` versus `--lms` data-loss distinction. Remove active legacy `release.sh`/`bindings/python` instructions; retain historical references only as superseded context. Rebuild generated docs from source.

## Dependencies

- B first: command boundaries, manifests, Python version gate, and test harness determine A's installed-binary and release test surface.
- A follows B's install contract; A must not alter B-owned scripts/manifests apart from its own CLI Cargo manifest.
- B's Python package gate must run exactly `cd crates/python && uvx --with hatch-vcs hatchling version` and require its tag-derived result to equal Cargo, built wheel/sdist metadata, and runtime `vexy_json.__version__`; it must also assert that `crates/python/pyproject.toml` retains Maturin as its build backend.
- C follows passing A+B contracts and changes no executable artifacts.
- All touched source/Markdown files include a correct project-relative `this_file` record.

## Verification gates

1. Static: `bash -n build.sh install.sh publish.sh`; run scripts outside repo root and from a path with spaces; scan active scripts/docs for forbidden legacy paths and forbidden build/install mutation commands.
2. Build/install: capture Git tracked/untracked baseline; run `./build.sh` twice and inspect `build.log.txt`; require generic Cargo build, tests, documentation tests, and example compilation with `--workspace --exclude vexy-json-python` for all non-Python Rust members, plus the mandatory canonical Maturin Python wheel build/test. Require strict formatting/Clippy-with-warnings-denied for the changed CLI Rust surface. Record 249 pre-existing workspace rustfmt diffs and legacy workspace Clippy debt as follow-up baseline work. Run `./install.sh --prefix "$test_prefix"` twice and validate installed CLI/Python import/forgiving parse/version without baseline drift.
3. CLI: unconditionally execute deterministic fake-`lms` repair and capture scenarios through installed `vexy-json`, not `cargo run`; require regular-file capture at 153,247 bytes and pipe truncation at 65,536 bytes, capture semantic equality/sentinel, and parseable JSON-only stdout. Require repair stderr to name unexpected EOF, byte 65,536, repair actions, partial/lossy output, and missing bytes not recovered; require capture stderr to name the regular-file source and 153,247-byte count. Also require the live installed-LMS baseline in this acceptance run: direct regular-file `lms ls --json --llm` output is valid JSON of exactly 153,247 bytes; piped output is exactly 65,536 bytes; and direct `jq` fails at that 65,536-byte boundary. Preserve the same diagnostics and no-semantic-equality claim for repaired output.
4. Dry run: in a clean temporary clone with command shims, run only `./publish.sh --dry-run`; require six `cargo publish --dry-run` gates, `uv build --no-sources`, artifact checks, and `uv publish --dry-run`; prove no gitnextver, ref/source mutation, non-dry registry command, or live credential requirement.
5. Real-mode orchestration: use only an isolated bare origin plus fake Cargo/uv registries. Assert exact ordering and one gitnextver invocation, `v1.5.10 -> v1.5.11` fixture version equality, ref equality before first fake upload, and fail-closed behavior for every listed failure boundary. **Never run plain `./publish.sh` against crates.io or PyPI.**
6. Final: run plain `./build.sh`, inspect `build.log.txt`, rerun the mandatory installed-binary fake-`lms` repair/capture suite with its required stderr assertions, run only real-tool `./publish.sh --dry-run`, rerun fake real-mode tests, then obtain fresh architect review before completion.
