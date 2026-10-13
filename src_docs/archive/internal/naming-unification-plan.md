# Naming Unification Plan for Vexy JSON

## Current Naming Conventions

Based on analysis of the codebase, here are the current naming patterns:

1. **Project Name**: "Vexy JSON" (with space)
2. **Rust Crate Names**: `vexy-json-*` (hyphenated)
3. **Rust Module Names**: `vexy_json_*` (underscored)
4. **Import Paths**: `vexy_json` (underscored)
5. **Type Names**: `VexyJson*` (PascalCase)
6. **Binaries**: `vexy_json` (underscored)
7. **Web Assets**: Mixed (`vexy-json-tool.js`, `vexy_json-tool`)
8. **URLs**: Mixed patterns

## Recommended Naming Standards

### 1. Human-Readable Contexts
- **Project Name**: "Vexy JSON" (with space)
- **Documentation Headers**: "Vexy JSON"
- **GitHub Repo**: `vexy-json` (hyphenated)
- **URLs**: `vexy-json` (hyphenated)

### 2. Rust/Cargo Contexts
- **Crate Names**: `vexy-json-*` (hyphenated) - Required by Cargo
- **Binary Name**: `vexy_json` (underscored) - For CLI consistency
- **Module Names**: `vexy_json_*` (underscored) - Rust convention
- **Import Paths**: `vexy_json` (underscored) - Rust convention
- **Type Names**: `VexyJson*` (PascalCase) - Rust convention

### 3. Language Bindings
- **Python Package**: `vexy-json` (hyphenated) - PyPI convention
- **Python Module**: `vexy_json` (underscored) - Python import convention
- **NPM Package**: `@vexy-json/vexy-json` (hyphenated) - NPM convention
- **C/C++ Headers**: `vexy_json.h` (underscored)
- **C/C++ Types**: `VexyJson*` (PascalCase)

### 4. Web Assets
- **JavaScript Files**: `vexy-json-*.js` (hyphenated)
- **HTML IDs**: `vexy-json-*` (hyphenated)
- **CSS Classes**: `vexy-json-*` (hyphenated)
- **Tool URLs**: `/vexy-json-tool/` (hyphenated)

## Specific Changes Needed

### High Priority
1. **Standardize Web Tool URLs**:
   - Change: `/vexy_json-tool/` → `/vexy-json-tool/`
   - Redirect old URLs for compatibility

2. **Unify JavaScript Asset Names**:
   - Rename inconsistent files to use `vexy-json-*` pattern

3. **Fix Mixed URL References**:
   - Update all GitHub URLs to use consistent pattern
   - Use `vexy-json` in URLs, not `vexy_json`

### Medium Priority
1. **Documentation Consistency**:
   - Ensure "Vexy JSON" (with space) in all prose
   - Use backticks for code references: `vexy_json`

2. **Update Package Metadata**:
   - Ensure all package.json, Cargo.toml files use correct naming

### Low Priority
1. **Internal Variable Names**:
   - Keep existing internal naming unless refactoring
   - Follow language conventions when adding new code

## Implementation Steps

1. **Create Naming Lint Script**:
   - Script to check for naming violations
   - Run in CI to prevent regressions

2. **Update Documentation**:
   - Batch update all markdown files
   - Update HTML/web assets

3. **Add Redirects**:
   - Set up URL redirects for changed paths
   - Maintain backward compatibility

4. **Update Package Metadata**:
   - Cargo.toml files
   - package.json files
   - pyproject.toml files

5. **Test All Changes**:
   - Verify imports still work
   - Check all URLs resolve
   - Test package installations

## Summary

The key principle is to use:
- "Vexy JSON" (with space) for human-readable contexts
- `vexy-json` (hyphenated) for URLs and package names
- `vexy_json` (underscored) for code imports and binaries
- `VexyJson` (PascalCase) for type names

This maintains consistency while respecting the conventions of each ecosystem.