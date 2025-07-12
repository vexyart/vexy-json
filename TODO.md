# this_file: docs/internal/TODO.md

Now /report and mark completed items as done in @PLAN.md and @TODO.md. Then run `./build.sh` and then check the `./build_logs`. If needed read the `./llms.txt` code snapshot. Then /work on items from @TODO.md consulting on @PLAN.md. Then review reflect refine revise, and then continue to /work on @PLAN.md and @TODO.md until every single item and issue has been fixed. Iterate iterate iterate! Do not stop, do not ask for confirmation. Work! When you're finishing one task or item, say "Wait, but..." and go on to the next task/item. It's CRUCIAL that we get to a solution that BUILDS everything correctly!

## CRITICAL BUILD FIXES (IMMEDIATE - Must fix for release)

### 1. Clippy Linter Errors (Blocking compilation)

- [ ] Fix clippy::uninlined-format-args errors in build.rs files:
  - [ ] crates/core/build.rs:8 - Change `println!("cargo:rustc-env=VEXY_JSON_VERSION={}", version)` to `println!("cargo:rustc-env=VEXY_JSON_VERSION={version}")`
  - [ ] crates/cli/build.rs:8 - Apply same format string fix
  - [ ] crates/wasm/build.rs:8 - Apply same format string fix

- [ ] Fix clippy::needless-borrows-for-generic-args errors:
  - [ ] crates/core/build.rs:18 - Change `&["describe", "--exact-match", "--tags"]` to `["describe", "--exact-match", "--tags"]`
  - [ ] crates/core/build.rs:30 - Change `&["describe", "--tags", "--always"]` to `["describe", "--tags", "--always"]`
  - [ ] crates/cli/build.rs:18,30 - Apply same array reference fixes
  - [ ] crates/wasm/build.rs:18,30 - Apply same array reference fixes

- [ ] Fix clippy::unnecessary-map-or errors:
  - [ ] crates/core/build.rs:36 - Change `map_or(false, |c| c.is_numeric())` to `is_some_and(|c| c.is_numeric())`
  - [ ] crates/cli/build.rs:36 - Apply same map_or fix
  - [ ] crates/wasm/build.rs:36 - Apply same map_or fix

### 2. Test Failures (1 failure)

- [ ] Fix property test failure in tests/property_tests.rs:
  - [ ] test_simple_structures failing with: `assertion failed: (left == right) left: 2, right: 3`
  - [ ] Minimal failing input: keys = ["A", "_"], values = [""]
  - [ ] Issue: Mismatch between keys.len() (2) and values.len() (1) but test expects them to be equal
  - [ ] Fix: Update test logic to handle mismatched key/value counts or adjust the property generation

### 3. Rustfmt Formatting Issues

- [ ] Fix formatting in benches/benchmark.rs:1 - Remove extra blank line after file comment
- [ ] Fix formatting in benches/comparison.rs:21 - Reformat bench_with_input call to multi-line
- [ ] Fix formatting in benches/lexer_microbenchmarks.rs:5 - Fix whitespace
- [ ] Fix formatting in benches/lexer_microbenchmarks.rs:15-39 - Reformat bench_with_input calls
- [ ] Run `cargo fmt --all` to automatically fix all formatting issues

### 4. Compilation Warnings (Non-blocking but should fix)

- [ ] Fix unused variable warnings:
  - [ ] crates/c-api/build.rs:13 - Prefix `cbindgen` with underscore: `_cbindgen`
  - [ ] crates/wasm/src/lib.rs:126 - Prefix `indent_size` with underscore: `_indent_size`
  - [ ] crates/core/examples/advanced_repair.rs:95 - Prefix `strategies` with underscore: `_strategies`

- [ ] Fix useless_ptr_null_checks warning:
  - [ ] crates/core/src/optimization/memory_pool.rs:381 - Remove unnecessary null check for `ptr.as_ptr()`

## Build Verification Steps

- [ ] Run `cargo clippy --all-targets --all-features -- -D warnings` to ensure no clippy errors
- [ ] Run `cargo fmt --all -- --check` to verify formatting
- [ ] Run `cargo test` to ensure all tests pass
- [ ] Run full release script: `./scripts/release.sh 1.0.3` to verify complete build

## Unify naming

- [ ] Work on `issues/611.txt`

## Phase 1: jsonic References Removal (IMMEDIATE)

- [ ] Rename test files: `jsonic_*.rs` → `vexy_json_*.rs` or `compat_*.rs`
- [ ] Update documentation: Remove "jsonic" from HTML, markdown files
- [ ] Clean code references: Replace "jsonic" with "vexy_json" in comments/variables
- [ ] Update configurations: Clean pyproject.toml and Cargo.toml references
- [ ] Verify completeness: Re-run grep to ensure no "jsonic" references remain

## Phase 2: Build Fixes (IMMEDIATE)

- [ ] Fix unused variables: Prefix with underscore in examples/recursive_parser.rs
- [ ] Fix test failure: Investigate and fix test_number_features number format parsing
- [ ] Verify build: Run `./build.sh` to confirm clean build

## Phase 3: Final Verification

- [ ] Run full test suite to ensure no regressions
- [ ] Check build output for warnings
- [ ] Verify all jsonic references are removed

## Phase 4: Release Preparation

- [ ] Run full test suite on all platforms
- [ ] Update version to 2.3.1 in all Cargo.toml files
- [ ] Update CHANGELOG.md with all fixes
- [ ] Create git tag v2.3.1
- [ ] Publish to crates.io

## Future Development (Post v2.3.1)

### Architecture Improvements

- [ ] Complete the pattern-based error recovery system (currently stubbed)
- [ ] Implement the ML-based pattern recognition
- [ ] Finish the streaming parser implementation
- [ ] Optimize memory pool usage

### Performance Enhancements

- [ ] Remove dead code to reduce binary size
- [ ] Optimize hot paths identified by warnings
- [ ] Implement SIMD optimizations where applicable

### Testing Infrastructure

- [ ] Add integration tests for all language bindings
- [ ] Create property-based tests for edge cases
- [ ] Set up continuous fuzzing

### Plugin System

- [ ] Design and implement a plugin architecture
- [ ] Create example plugins
- [ ] Document plugin development

### Advanced Features

- [ ] Incremental parsing for live editing
- [ ] Schema validation integration
- [ ] Advanced error recovery strategies
- [ ] JSON path query support
