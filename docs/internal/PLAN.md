# this_file: docs/internal/TODO.md

Now /report and mark completed items as done in <!-- Imported from: PLAN.md -->
# this_file: docs/internal/PLAN.md

# Vexy JSON Improvement Plan - v2.3.2 the reference implementation Removal & Build Fixes

## Executive Summary

Following the successful project renaming to Vexy JSON, this plan addresses critical remaining issues:

### New Critical Issues Found (v2.3.2)
1. **the reference implementation references removal** - Found 50 files containing "the reference implementation" references that need cleanup
2. **Test failure** - test_number_features failing due to number format parsing issues
3. **Build warnings** - 3 unused variable warnings in examples/recursive_parser.rs
4. **Build status** - Build succeeds but with warnings and 1 test failure

### Completed (v2.3.0)
1. ✅ **C API naming fixed** - Resolved struct name mismatches
2. ✅ **Critical compilation errors fixed** - Added missing struct fields and enum variants
3. ✅ **README.md updated** - Removed migration tool references

### Current Status (v2.3.1)
1. **Naming consistency** - Minor inconsistencies found in Python bindings
2. **Compilation warnings** - 24 warnings (reduced from 30)
3. **Test failures** - 8 failing tests remain
4. **Build successful** - Core and CLI build without errors
5. **Documentation** - Mostly consistent, one ZZSON reference remains

## Post-Migration Findings

### Naming Analysis Results
1. **Old Naming References**: Only 2 files contain "zzson" - both in documentation (PLAN.md and issue 610.txt)
2. **Python Bindings**: Test file previously used `VexyJSONParser` but was fixed to `VexyJsonParser`
3. **Naming Conventions**: Generally consistent across languages:
   - Rust: `vexy_json-*` (crate names), `VexyJson*` (types)
   - C/C++: `VexyJson*` (types)
   - Python: `vexy_json` (package), `VexyJson*` (classes)
   - JavaScript: `VexyJson*` (classes)
   - Documentation: "Vexy JSON" (with space)

## Priority Groups

### Group 0: IMMEDIATE - Critical Fixes

#### 0.1 Remove the reference implementation References (50 files)
- **High Priority**: Remove all "the reference implementation" references from codebase
- **Files affected**: 50 files including tests, documentation, and code
- **Impact**: Legacy naming that confuses project identity
- **Categories to clean**:
  - Test files: `the reference implementation_*.rs`, `supported_the reference implementation.rs`
  - Documentation: HTML files, markdown files, tool descriptions
  - Code references: Comments, variable names, function names
  - Configuration: pyproject.toml, Cargo.toml references

#### 0.2 Fix Test Failure (1 failure)
- **test_number_features** - Number format parsing for octal (0o77), binary (0b1010), underscore separators (1_000_000)
- **Root cause**: Parser doesn't support these number formats, or incorrectly identifies them as floats. The tests are failing because they expect `Number::Integer` but receive `Number::Float`.
- **Fix needed**: Implement support for these number formats, ensuring they are correctly parsed as integers when applicable. This involves modifying the number parsing logic in `crates/core/src/parser/number.rs` to handle binary, octal, hexadecimal, and underscore separators.

#### 0.3 Fix Build Warnings (3 warnings)
- **examples/recursive_parser.rs**: 3 unused variable warnings
- **Simple fix**: Prefix variables with underscore or use the results
- **Impact**: Clean build output

### Group 1: HIGH Priority - Clean Up Warnings

#### 1.1 Dead Code Cleanup (24 warnings)
- **Unused methods**: `analyze_custom_error`, `analyze_context_error`, `analyze_invalid_utf8`
- **Unused fields**: `confidence`, `patterns`, `learned_patterns`, `lookahead_size`, etc.
- **Unused variants**: `StateChange`, `InsertString`, `ReplaceRange`, etc.
- **Decision needed**: Either implement these features or remove the dead code

#### 1.2 Import Cleanup
- Fix unused imports in `trace_parse.rs`
- Run `cargo fix` to automatically clean up simple warnings
- Target: Reduce warnings from 24 to under 10 (achieved 0 warnings!)

### Group 2: MEDIUM Priority - Post-Release Improvements

#### 2.1 Architecture Improvements
- Complete the pattern-based error recovery system (currently stubbed)
- Implement the ML-based pattern recognition
- Finish the streaming parser implementation
- Optimize memory pool usage

#### 2.2 Performance Enhancements
- Remove dead code to reduce binary size
- Optimize hot paths identified by warnings
- Implement SIMD optimizations where applicable

#### 2.3 Testing Infrastructure
- Add integration tests for all language bindings
- Create property-based tests for edge cases
- Set up continuous fuzzing

### Group 3: LOW Priority - Future Enhancements

#### 3.1 Plugin System
- Design and implement a plugin architecture
- Create example plugins
- Document plugin development

#### 3.2 Advanced Features
- Incremental parsing for live editing
- Schema validation integration
- Advanced error recovery strategies
- JSON path query support

## Implementation Plan

### Phase 1: the reference implementation References Removal (Immediate - 2-3 hours)
1. **Rename test files**: `the reference implementation_*.rs` → `vexy_json_*.rs` or `compat_*.rs`
2. **Update documentation**: Remove "the reference implementation" from HTML, markdown, and tool descriptions
3. **Clean code references**: Replace "the reference implementation" with "vexy_json" in comments and variable names
4. **Update configurations**: Clean pyproject.toml and Cargo.toml references
5. **Verify completeness**: Re-run grep to ensure no "the reference implementation" references remain

### Phase 2: Build Fixes (30 minutes)
1. **Fix unused variables**: Prefix with underscore in examples/recursive_parser.rs
2. **Fix test failure**: Investigate and fix test_number_features number format parsing
   - **Action**: Modify `crates/core/src/parser/number.rs` to correctly parse binary (0b), octal (0o), hexadecimal (0x), and numbers with underscore separators. Ensure these are represented as `Number::Integer` where appropriate.
3. **Verify build**: Run `./build.sh` to confirm clean build

### Phase 3: Final Verification (30 minutes)
1. Run full test suite to ensure no regressions
2. Check build output for warnings
3. Verify all the reference implementation references are removed

### Phase 4: Release Preparation (1 day)
1. Run full test suite on all platforms.
2. Update version to 2.3.1 in all Cargo.toml files.
3. Update CHANGELOG.md with all fixes.
4. Create git tag v2.3.1.
5. Publish to crates.io.

## Success Metrics

- ✅ Zero references to ZZSON in code
- ✅ Successful build of core and CLI
- ⬜ Reduced warnings to < 10 (currently 24)
- ⬜ All 8 failing tests fixed
- ⬜ Clean documentation with no migration artifacts

## Current State Summary

The Vexy JSON project has successfully completed its renaming from ZZSON. The codebase is:
- **Functionally correct** - Builds and runs
- **Mostly consistent** - Naming follows language conventions
- **Nearly release-ready** - Only cleanup tasks remain

## Next Steps

1. Remove the ZZSON reference from line 8 of this file
2. Run `cargo fix` to clean up simple warnings
3. Investigate and fix the 8 failing tests
4. Release version 2.3.1 as a "post-migration cleanup" release

The project is in good shape with only minor housekeeping tasks remaining.