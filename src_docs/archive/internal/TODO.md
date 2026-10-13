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
<!-- End of import from: PLAN.md --> and <!-- Circular import detected: TODO.md --> Then run `./build.sh` and then check the `./build_logs`. If needed read the <!-- Import failed: llms.txt - Only .md files are supported --> code snapshot. Then /work on items from <!-- Circular import detected: TODO.md --> consulting on <!-- Import failed: PLAN.md. - Only .md files are supported --> Then review reflect refine revise, and then continue to /work on <!-- Imported from: PLAN.md -->
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
<!-- End of import from: PLAN.md --> and <!-- Circular import detected: TODO.md --> until every single item and issue has been fixed. Iterate iterate iterate! Do not stop, do not ask for confirmation. Work! When you're finishing one task or item, say "Wait, but..." and go on to the next task/item. It’s CRUCIAL that we get to a solution that BUILDS everything correctly!

## Unify Naming Conventions

This section outlines a detailed plan to unify the naming conventions across the Vexy JSON project, ensuring consistency in how the project name is represented in different contexts (code, documentation, configuration, etc.).

### Naming Strategy Summary:
- **`Vexy JSON` (Title Case with space):** Primary human-readable project name. Use in documentation titles, user-facing messages, and general descriptive text.
- **`vexy_json` (snake_case):** Rust crate names, Python package names, internal code references (variables, functions), and file/directory names where snake_case is conventional.
- **`VexyJson` (PascalCase):** Rust and C/C++ type names (structs, enums, classes).
- **`vexy-json` (kebab-case):** URLs, repository names, and CLI commands.
- **`VEXY_JSON` (All Caps with underscore):** Reserved for constants or placeholders (e.g., `%%VEXY_JSON_VERSION%%`).

### Implementation Steps:

- [ ] **Review and Update `README.md`:**
    - Ensure the main title is "Vexy JSON".
    - Verify all descriptive text uses "Vexy JSON".
    - Confirm code examples use `vexy_json` for imports and calls.

- [ ] **Review and Update `AGENTS.md` and `CLAUDE.md`:**
    - Ensure project overview sections use `vexy_json` for the Rust library name and "Vexy JSON" for the overall project name in descriptive text.
    - Verify consistency in crate names (`vexy_json-core`, `vexy_json-cli`, etc.).

- [ ] **Review and Update `PLAN.md`:**
    - Ensure all references to the project name in descriptive text use "Vexy JSON".
    - Confirm consistency in naming conventions for Rust, C/C++, Python, and JavaScript as per the strategy.

- [ ] **Review and Update Rust Code (`.rs` files):**
    - **`vexy_json` (snake_case):**
        - Verify `use vexy_json::...` and `use vexy_json_core::...` statements.
        - Ensure function calls like `vexy_json::parse` are consistent.
        - Check `Cargo.toml` files within `crates/` for `name = "vexy_json-..."` and `dependencies.vexy_json-core` etc.
        - **Action**: If any Rust code uses `VexyJson` or `VEXYJSON` where `vexy_json` (snake_case) is expected for crate/module names or function calls, change it.
    - **`VexyJson` (PascalCase):**
        - Verify struct and enum names (e.g., `VexyJsonParserOptions`, `VexyJsonParseResult`).
        - **Action**: If any Rust code uses `vexy_json` or `VEXY_JSON` where `VexyJson` (PascalCase) is expected for type names, change it.

- [ ] **Review and Update Python Bindings (`bindings/python/`):**
    - **`vexy_json` (snake_case):**
        - Verify `import vexy_json` and usage like `vexy_json.parse()`.
        - Check `bindings/python/src/vexy_json/__init__.py` for package name and module-level documentation.
        - Check `bindings/python/README.md` for installation instructions (`pip install vexy_json`) and code examples.
        - **Action**: Ensure all Python code and documentation consistently use `vexy_json` (snake_case) for the package and its functions.
    - **`VexyJson` (PascalCase):**
        - Verify class names like `VexyJsonParser` (if present, based on `WORK.md` fix).
        - **Action**: If any Python code uses `vexy_json` or `VEXY_JSON` where `VexyJson` (PascalCase) is expected for class names, change it.

- [ ] **Review and Update C/C++ Bindings (`crates/c-api/`):**
    - **`vexy_json` (snake_case):**
        - Verify C function names (e.g., `vexy_json_version`, `vexy_json_parse`).
        - Check `crates/c-api/include/vexy_json.h` and `vexy_json.hpp` for function and namespace names.
        - **Action**: Ensure consistency with `vexy_json` (snake_case) for C API functions and C++ namespace.
    - **`VexyJson` (PascalCase):**
        - Verify struct names (e.g., `VexyJsonParserOptions`, `VexyJsonParseResult`).
        - **Action**: Ensure consistency with `VexyJson` (PascalCase) for C/C++ types.

- [ ] **Review and Update JavaScript/WASM (`crates/wasm/`, `docs/assets/js/`):**
    - **`vexy_json` (snake_case):**
        - Verify imports like `vexy_json_wasm.js`.
        - Check `docs/assets/js/tool.js` for `trackingId: 'vexy_json-web-tool'` and console logs.
        - Check `docs/assets/js/examples.js` for `name: "vexy_json"` and `description: 'Showcase of all vexy_json forgiving features together'`.
        - **Action**: Ensure consistency with `vexy_json` (snake_case) for module names and internal JavaScript references.
    - **`VEXY_JSON` (All Caps with underscore):**
        - Verify usage of `%%VEXY_JSON_VERSION%%` as a placeholder.
        - **Action**: Ensure `VEXY_JSON` is only used for such placeholders.

- [ ] **Review and Update Configuration Files (`Cargo.toml`, `pyproject.toml`, `oss-fuzz/project.yaml`, `scripts/package.json`):**
    - **`vexy_json` (snake_case):**
        - Verify `name` fields in `Cargo.toml` and `scripts/package.json`.
        - Verify dependency names.
        - **Action**: Ensure `vexy_json` (snake_case) is used for package/crate names.
    - **`vexy-json` (kebab-case):**
        - Verify `repository` and `homepage` URLs in `Cargo.toml` and `oss-fuzz/project.yaml`.
        - Verify references in `oss-fuzz/README.md` and `Formula/README.md`.
        - **Action**: Ensure `vexy-json` (kebab-case) is used for URLs and repository names.

- [ ] **Review and Update Shell Scripts (`.sh` files):**
    - **`vexy_json` (snake_case):**
        - Verify `cargo build --bin vexy_json` and similar commands.
        - Verify file paths like `target/release/vexy_json`.
        - **Action**: Ensure `vexy_json` (snake_case) is used for binary names and related file paths.
    - **`VEXY_JSON` (All Caps with underscore):**
        - Verify usage in generated `README.txt` (e.g., `VEXY_JSON v$VERSION`).
        - **Action**: Confirm this usage is acceptable for generated output.

- [ ] **Review and Update Homebrew Formula (`Formula/vexy_json.rb`):**
    - **`VexyJson` (PascalCase):**
        - Verify class name `class VexyJson < Formula`.
        - **Action**: Ensure this remains `VexyJson`.
    - **`vexy_json` (snake_case):**
        - Verify `homepage`, `url`, `bin/vexy_json` references.
        - **Action**: Ensure consistency with `vexy_json` (snake_case) for binary and URL components.

- [ ] **Final Verification:**
    - After making changes, re-run `rg -C 3 "vexy" > grep.txt` and review the output to ensure all changes are applied correctly and no new inconsistencies are introduced.
    - Run `./build.sh` to confirm the project still builds and tests pass (addressing the number parsing issue separately).