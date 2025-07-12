# this_file: TODO.md

## CRITICAL: v1.5.2 Release Issues - Immediate Fixes Required

### Phase 1: Fix Critical Build Errors (IMMEDIATE)
- [ ] Fix rustfmt.toml - change `fn_args_layout` to `fn_params_layout`
- [ ] Run `cargo clippy --fix --allow-dirty --allow-staged` to apply automatic fixes
- [ ] Fix format string errors - replace `format!("{}", var)` with `format!("{var}")` (93 occurrences)
- [ ] Fix identical if-else blocks (7 occurrences)
- [ ] Fix unnecessary let bindings before return (4 occurrences)
- [ ] Fix map iterator usage - use `.values()` instead of `.iter().map(|(_, v)| v)`
- [ ] Implement Default trait for: TypedArena, StreamingParser, SmallVec, ScopedMemoryPoolV3, PerformanceMonitor, MLPatternRecognizer, MemoryPoolV3, ErrorRecoveryEngineV2
- [ ] Fix type complexity warnings - extract type aliases
- [ ] Fix remaining manual implementations

### Phase 2: Fix Failing Tests (HIGH PRIORITY)
- [ ] Fix `error::recovery_v2::tests::test_bracket_matching` - bracket type detection
- [ ] Fix `lazy::tests::test_lazy_array` - UnexpectedChar error
- [ ] Fix `lazy::tests::test_lazy_parser_small_object` - Expected string key EOF error
- [ ] Fix `lazy::tests::test_lazy_parser_with_threshold` - value parsing
- [ ] Fix `lexer::debug_lexer::tests::test_debug_lexer_error_logging` - error detection
- [ ] Fix `lexer::fast_lexer::tests::test_fast_lexer_stats` - token count mismatch
- [ ] Fix `optimization::memory_pool_v2::tests::test_scoped_pool` - allocation tracking
- [ ] Fix `parser::iterative::tests` - array/object parsing state machine
- [ ] Fix `parallel_chunked::tests::test_chunked_ndjson` - empty values
- [ ] Fix `parser::optimized_v2::tests` - memory stats tracking
- [ ] Fix `plugin::plugins::datetime::tests::test_custom_format` - object type error
- [ ] Fix `streaming::event_parser::tests` - incomplete JSON handling
- [ ] Fix `streaming::ndjson::tests` - line counting and parsing

### Phase 3: Fix Build System (HIGH PRIORITY)
- [ ] Update build.sh - temporarily remove `-D warnings` from RUSTFLAGS
- [ ] Make fuzzing conditional - check for nightly toolchain before running fuzz tests
- [ ] Update release.sh - add `set -e` and proper test failure checking
- [ ] Test full build process after fixes

### Phase 4: Prepare Clean Release (v1.5.3)
- [ ] Run full test suite and ensure all pass
- [ ] Run clippy with warnings only (not deny)
- [ ] Update version to 1.5.3 in all Cargo.toml files
- [ ] Update CHANGELOG.md with all fixes
- [ ] Run release script with proper validation
- [ ] Publish to crates.io

## Future Tasks (Post-v1.5.3)

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