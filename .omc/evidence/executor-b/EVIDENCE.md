---
this_file: .omc/evidence/executor-b/EVIDENCE.md
---

# Executor B evidence

## Workflow contracts

- Scenario: syntax, manifest policy, retired-script absence, root-relative execution from a path containing spaces, repeatable user-prefix installation, fail-closed dirty dry-run, no mutation in dry-run, and fake real-mode orchestration against a temporary bare origin.
- Invocation: `bash scripts/tests/workflow-contracts.sh`
- Binary observables: exit 0; two successful installs into the same prefix; six `cargo publish --dry-run` calls; no gitnextver in dry-run; dirty tree rejected before package gates; real fixture created exactly `v1.5.11` from `v1.5.10`; exact gitnextver command invoked once; all fake uploads occurred only after live local/remote ref equality.
- Captured artifact: `.omc/evidence/executor-b/workflow-contracts.log`

## Shell quality

- Scenario: all active shell entry points and the isolated contract harness.
- Invocation: `shellcheck build.sh install.sh publish.sh scripts/tests/workflow-contracts.sh`
- Binary observable: exit 0 and `SHELLCHECK_PASS`.
- Captured artifact: `.omc/evidence/executor-b/shellcheck.log`

## Plain build gate

- Scenario: real plain root build with the installed Rust/uv/Python toolchain.
- Invocation: `./build.sh`
- Binary observable: exit 1 at the first required `cargo fmt --all -- --check` boundary. No compilation, test, or package success is claimed. The log records pre-existing formatting drift across core/bench/test Rust files outside Executor B ownership and stable-rustfmt warnings for nightly-only `rustfmt.toml` keys.
- Captured artifacts: `build.log.txt` and `.omc/evidence/executor-b/build-first-run.log`.

## Live dry-run safety preflight

- Scenario: real checkout with concurrent tracked edits and untracked `.codegraph/`/`.omc/` state.
- Invocation: `./publish.sh --dry-run`
- Binary observable: nonzero exit during `read-only-preflight`, before build, gitnextver, or registry commands; exact message says a completely clean tree is required.
- Captured artifact: `.omc/evidence/executor-b/live-dry-run-preflight.log`.

## Static artifact checks

- Invocation: `bash -n build.sh install.sh publish.sh scripts/tests/workflow-contracts.sh`; `python3 scripts/release-manifests.py check`; `python3 -m py_compile scripts/release-manifests.py scripts/check-python-artifacts.py scripts/tests/fake-toolchain.py crates/python/tests/test_vexy_json.py`; `git diff --check -- <Executor-B-owned files>`.
- Binary observables: every invocation exited 0; synchronized manifest version reported `1.5.10`; the allowlist is exactly six public Rust packages while test-utils and Python cdylib are private.
- Captured artifact: workflow and shellcheck logs above; command output was also inspected directly in the executor session.
