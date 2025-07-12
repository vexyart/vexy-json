# this_file: docs/internal/TODO.md

Now /report and mark completed items as done in @PLAN.md and @TODO.md. Then run `./build.sh` and then check the `./build_logs`. If needed read the @llms.txt code snapshot. Then /work on items from @TODO.md consulting on @PLAN.md. Then review reflect refine revise, and then continue to /work on @PLAN.md and @TODO.md until every single item and issue has been fixed. Iterate iterate iterate! Do not stop, do not ask for confirmation. Work! When you're finishing one task or item, say "Wait, but..." and go on to the next task/item. It’s CRUCIAL that we get to a solution that BUILDS everything correctly!

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
