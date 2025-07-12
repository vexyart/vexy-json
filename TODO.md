# this_file: TODO.md

## Completed Tasks (v2.3.4)

- [x] Successfully migrated documentation from Jekyll to MkDocs Material
- [x] Fixed missing comprehensive_comparison benchmark reference in Cargo.toml
- [x] Complete jsonic References Removal - Removed all code references (only docs remain)
- [x] Build and Deliverables - Fixed binary naming and packaging scripts
- [x] Test build deliverables - Successfully built for macOS and Linux
- [x] Fix clippy::uninlined-format-args warnings - Applied cargo clippy --fix

## Phase 3: Remaining Clippy Warnings Cleanup
- [ ] Fix clippy::for-kv-map warnings in iterator usage
- [ ] Fix clippy::should_implement_trait warnings for type conversions
- [ ] Apply other minor clippy fixes and suggestions

## Phase 4: Naming Unification

- [ ] Standardize Web Tool URLs: `/vexy_json-tool/` → `/vexy-json-tool/`
- [ ] Unify JavaScript asset names to use `vexy-json-*` pattern
- [ ] Fix mixed URL references in documentation
- [ ] Ensure "Vexy JSON" (with space) in all prose documentation
- [ ] Use backticks for code references: `vexy_json`
- [ ] Update all package metadata for consistent naming
- [ ] Create naming lint script to check violations
- [ ] Add URL redirects for backward compatibility

## Phase 5: Final Verification and Release

- [ ] Run full test suite on all platforms
- [ ] Check build output for warnings
- [ ] Verify all jsonic references are removed
- [ ] Update version in all Cargo.toml files
- [ ] Update CHANGELOG.md with all fixes
- [ ] Create git tag
- [ ] Publish to crates.io

## Future Development (Post-Release)

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