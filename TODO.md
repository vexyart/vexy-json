# this_file: docs/internal/TODO.md

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

## COMPLETED (Previous work)
- [x] Remove ZZSON reference from PLAN.md
- [x] Verify no other documentation contains old naming
- [x] Update WORK.md with current status
- [x] Run `cargo fix --workspace` for automatic fixes
- [x] Fix unused imports in `trace_parse.rs`
- [x] Review dead code warnings and decide: implement or remove
- [x] Add `#[allow(dead_code)]` for code that will be implemented later
- [x] Re-run build to verify warning reduction
- [x] Target: Reduce warnings to < 10 (achieved 0 warnings!)
- [x] Run each failing test individually to diagnose issues
- [x] Fix `basic_parsing::test_implicit_arrays` - Fixed parser to handle space-separated values with comments
- [x] Fix `comment_handling::test_multi_line_comments` - Fixed implicit array creation for comment-separated values

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