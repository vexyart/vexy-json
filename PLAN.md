# this_file: docs/internal/PLAN.md

# Vexy JSON Improvement Plan - v1.5.2 Post-Release Fixes

## Executive Summary

Following the v1.5.2 release, critical issues were identified during the build and release process that need immediate attention. This plan addresses compilation failures, test failures, and build system issues.

### Completed (v2.3.3)

1. ✅ **Critical clippy errors fixed** - All blocking compilation errors resolved
2. ✅ **Test failures fixed** - test_number_features now passing
3. ✅ **Build warnings fixed** - Unused variable warnings resolved
4. ✅ **Build status** - Core library builds successfully
5. ✅ **Build deliverables script** - Created build-deliverables.sh for all platforms
6. ✅ **Applied clippy fixes** - Reduced warnings using cargo clippy --fix
7. ✅ **Naming unification plan** - Created detailed standards in docs/naming-unification-plan.md

### Completed (v2.3.2)

1. ✅ **Build script improvements** - Rewrote `./build.sh` with modular commands
2. ✅ **Critical clippy errors fixed** - Fixed all blocking compilation errors
3. ✅ **Test failures fixed** - Fixed property test failure (duplicate keys)
4. ✅ **Compilation warnings fixed** - Fixed unused variables and null check warnings
5. ✅ **Rustfmt applied** - Formatted entire codebase

### Completed (v2.3.0)

1. ✅ **C API naming fixed** - Resolved struct name mismatches
2. ✅ **Critical compilation errors fixed** - Added missing struct fields and enum variants
3. ✅ **README.md updated** - Removed migration tool references

### Current Issues Discovered (v1.5.2 Release)

1. **Build Failures** - ❌ CRITICAL - ./build.sh fails with 143 clippy errors preventing compilation
2. **Test Failures** - ❌ CRITICAL - 20 tests failed but release continued anyway
3. **Fuzzing Issues** - ⚠️ WARNING - Fuzz tests require nightly compiler but using stable
4. **Code Formatting** - ⚠️ WARNING - rustfmt check failed due to deprecated option
5. **Release Process** - ⚠️ WARNING - Release succeeded despite build/test failures

## Critical Issues Analysis

### 1. Build Failures (143 Clippy Errors)
- **93 errors**: `format!` strings can use inline variables
- **7 errors**: Identical if-else blocks
- **4 errors**: Unnecessary let bindings before return
- **3 errors**: Iterating on map values incorrectly
- **3 errors**: Manual prefix stripping
- **Various**: Type complexity, Default trait implementations needed

### 2. Test Failures (20 Failed Tests)
Failed test modules:
- `error::recovery_v2::tests::test_bracket_matching`
- `lazy::tests` (multiple failures)
- `lexer::debug_lexer::tests`
- `optimization::memory_pool_v2::tests`
- `parser::iterative::tests` (multiple failures)
- `streaming::` tests (multiple failures)
- `plugin::plugins::datetime::tests`

### 3. Build System Issues
- Fuzzing requires nightly Rust but build uses stable
- rustfmt has deprecated `fn_args_layout` option
- Release script ignores test failures

## Priority Groups

### Group 0: IMMEDIATE - Critical Build & Test Fixes

#### 0.1 Fix Clippy Errors Blocking Compilation (143 errors)

- [ ] **CRITICAL**: Fix format string errors - use inline variables `{var}` instead of `{}`, var
- [ ] **CRITICAL**: Remove identical if-else blocks (7 occurrences)
- [ ] **CRITICAL**: Fix unnecessary let bindings before return (4 occurrences)
- [ ] **CRITICAL**: Fix map iterator usage (use `.values()` instead of `.iter()`)
- [ ] **CRITICAL**: Implement Default trait for required types
- [ ] **Action**: Run `cargo clippy --fix` where safe, manually fix remaining

#### 0.2 Fix Failing Tests (20 test failures)

- [ ] **CRITICAL**: Fix bracket matching test in error recovery v2
- [ ] **CRITICAL**: Fix lazy parser tests (array, object parsing)
- [ ] **CRITICAL**: Fix iterative parser tests
- [ ] **CRITICAL**: Fix streaming/NDJSON parser tests
- [ ] **CRITICAL**: Fix memory pool allocation tests
- [ ] **Action**: Debug each failing test and fix root causes

#### 0.3 Fix Build System Issues

- [ ] **HIGH**: Update rustfmt.toml - change `fn_args_layout` to `fn_params_layout`
- [ ] **HIGH**: Make fuzzing optional or add nightly toolchain detection
- [ ] **HIGH**: Fix release script to fail on test failures
- [ ] **Action**: Update configuration files and scripts

### Group 1: HIGH Priority - Clean Up Remaining Warnings

#### 1.1 Clippy Warnings Cleanup (100+ warnings)

- [ ] **clippy::uninlined-format-args**: 100+ occurrences throughout codebase
- [ ] **clippy::for-kv-map**: Several warnings in iterator usage
- [ ] **clippy::should_implement_trait**: Type conversion warnings
- [ ] **Other minor clippy suggestions**: Various style improvements

#### 1.2 Naming Unification Implementation

- [ ] **High Priority**: Standardize Web Tool URLs: `/vexy_json-tool/` → `/vexy-json-tool/`
- [ ] **High Priority**: Unify JavaScript asset names to use `vexy-json-*` pattern
- [ ] **High Priority**: Fix mixed URL references in documentation
- [ ] **Medium Priority**: Ensure "Vexy JSON" (with space) in all prose documentation
- [ ] **Medium Priority**: Use backticks for code references: `vexy_json`
- [ ] **Medium Priority**: Update all package metadata for consistent naming

### Group 2: MEDIUM Priority - Post-Release Improvements

#### 2.1 Architecture Improvements

- [ ] Complete the pattern-based error recovery system (currently stubbed)
- [ ] Implement the ML-based pattern recognition
- [ ] Finish the streaming parser implementation
- [ ] Optimize memory pool usage

#### 2.2 Performance Enhancements

- [ ] Remove dead code to reduce binary size
- [ ] Optimize hot paths identified by warnings
- [ ] Implement SIMD optimizations where applicable

#### 2.3 Testing Infrastructure

- [ ] Add integration tests for all language bindings
- [ ] Create property-based tests for edge cases
- [ ] Set up continuous fuzzing

### Group 3: LOW Priority - Future Enhancements

#### 3.1 Plugin System

- [ ] Design and implement a plugin architecture
- [ ] Create example plugins
- [ ] Document plugin development

#### 3.2 Advanced Features

- [ ] Incremental parsing for live editing
- [ ] Schema validation integration
- [ ] Advanced error recovery strategies
- [ ] JSON path query support

## Implementation Plan

### Phase 1: Fix Critical Build Errors (Immediate)

1. **Fix rustfmt configuration**: Update rustfmt.toml to use `fn_params_layout`
2. **Run cargo clippy --fix**: Apply automatic fixes where safe
3. **Fix format strings manually**: Replace `format!("{}", var)` with `format!("{var}")`
4. **Fix identical if-else blocks**: Refactor or remove duplicate code
5. **Fix map iterations**: Use `.values()` instead of `.iter().map(|(_, v)| v)`
6. **Implement Default traits**: Add Default implementations for required types

### Phase 2: Fix Failing Tests (High Priority)

1. **Debug bracket matching test**: Fix error recovery v2 bracket detection
2. **Fix lazy parser**: Resolve EOF and parsing issues in lazy module
3. **Fix iterative parser**: Address array/object parsing state machine
4. **Fix streaming tests**: Resolve NDJSON and event parser issues
5. **Fix memory pool tests**: Ensure proper allocation tracking
6. **Run test suite**: Verify all tests pass

### Phase 3: Fix Build System (High Priority)

1. **Update build.sh**: Remove strict clippy deny warnings for now
2. **Fix fuzzing**: Make fuzz tests conditional on nightly toolchain
3. **Update release.sh**: Add test failure check that stops release
4. **Test build process**: Run full build and verify success

### Phase 4: Prepare Clean Release (v1.5.3)

1. **Run full test suite**: Ensure all tests pass
2. **Run clippy with warnings**: Check remaining non-critical issues
3. **Update version**: Bump to 1.5.3 in all Cargo.toml files
4. **Update CHANGELOG.md**: Document all fixes
5. **Create release**: Run release script with proper checks
6. **Publish to crates.io**: Complete the release process

## Success Metrics

- [ ] ❌ Build completes without errors (currently 143 clippy errors)
- [ ] ❌ All tests pass (currently 20 failures)
- [ ] ⬜ Fuzzing works or is properly disabled
- [ ] ⬜ Release script validates test success
- [ ] ⬜ Clean release v1.5.3 published

## Current State Summary

The v1.5.2 release exposed critical issues:

- **Build System**: Too strict clippy settings prevent compilation
- **Test Suite**: 20 tests failing but ignored by release process
- **Quality Control**: Release proceeded despite failures
- **Configuration**: Outdated rustfmt options and fuzzing requirements

## Immediate Next Steps

1. Fix rustfmt.toml configuration
2. Apply cargo clippy --fix for automatic fixes
3. Manually fix remaining clippy errors
4. Debug and fix all 20 failing tests
5. Update build and release scripts
6. Prepare and release v1.5.3 with all fixes

This is a critical situation that needs immediate attention before any further development.
