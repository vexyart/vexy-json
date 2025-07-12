# this_file: TODO.md

## Phase 1: Complete jsonic References Removal (IMMEDIATE)

- [ ] Execute remaining removal scripts to complete final 382 references from 31 files (79% done)
- [ ] Clean test files: Update comments and variable names in test files
- [ ] Update documentation: Remove "jsonic" from HTML, markdown, and tool descriptions
- [ ] Clean JavaScript assets: Update vexy-json-tool.js references
- [ ] Update build scripts: Clean remaining scattered references
- [ ] Verify completeness: Re-run grep to ensure no "jsonic" references remain

## Phase 2: Build and Deliverables (MEDIUM PRIORITY)

- [ ] Read issues/620.txt to understand build deliverables requirements
- [ ] Fix macOS packaging: Correct binary path in packaging script (vexy-json not vexy_json)
- [ ] Test build deliverables: Run build-deliverables.sh and test on all platforms
- [ ] Run full release script: Execute `./scripts/release.sh 1.0.6` to verify complete build

## Phase 3: Clippy Warnings Cleanup

- [ ] Fix clippy::uninlined-format-args warnings (100+ occurrences)
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