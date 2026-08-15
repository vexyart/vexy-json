# this_file: docs/internal/WORK.md

# Work Progress

## Release publication recovery

- Diagnosed the interrupted v1.5.11 release as an oversized root crate: its
  implicit Cargo file set produced a 30.2 MiB upload containing generated docs,
  old installers, release archives, and local development artifacts.
- Confirmed the five dependency crates and release refs, then published the
  missing Python 1.5.11 artifacts without replaying completed uploads.
- Restricted the root crate package surface and updated the public README.
- Confirmed `build.sh` builds and tests both language surfaces, and updated
  `install.sh` to install the Rust CLI plus the Python wheel via
  `uv pip install --system`.

## Latest completed work

### Streaming parser emergency repair — completed

- Root cause: `StreamingParser` returned `EndOfInput` forever, so seven tests
  that collected parser events could run without end.
- The repair makes EOF one-shot and the iterator finite. Regression drains are
  bounded; newline handling, empty-array closure, and malformed fixtures were
  corrected.
- `build.sh` now handles `INT`, `TERM`, and failures truthfully, with workflow
  contracts reflecting the checked build path.
- Stale Python wheel tests and documentation now match established forgiving
  semantics.

### Verification evidence

- Workflow contracts passed.
- `./build.sh` passed the Rust workspace and packaged Python test suite:
  36 passed, 2 skipped.
- Final `./build.sh` run exited 0 in 14.22 s with 429,047,808-byte maximum RSS
  and 0 swaps.

### Known remaining warnings

- Unstable rustfmt configuration noise remains.
- `extract_string_content` remains unused.

## Remaining roadmap context

The streaming parser roadmap item remains open. The emergency termination repair
does not make the parser constant-memory: `input_buffer` retains all input, and
front-removal from `Vec` plus queued events prevent scalable constant-memory
streaming. See `PLAN.md` and `TODO.md` for the remaining work.

## Historical status snapshot (July 13, 2025)

### Code quality improvements — completed

- Added `#[allow(dead_code)]` attributes to suppress warnings in benchmarks.
- Refactored parser function types for better clarity.
- Simplified string formatting in stack overflow tests.
- Enhanced `newline_as_comma.rs` test cases.

### Version management — completed

- Updated all crates to version 1.5.6.
- Synchronized version numbers across the workspace.
- Updated package.json files for WASM builds.

### Documentation updates — completed

- Enhanced CHANGELOG.md with detailed recent changes.
- Updated MkDocs documentation builds.
- Cleaned up internal planning documents.
