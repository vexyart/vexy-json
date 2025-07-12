# this_file: docs/internal/WORK.md

# Work Progress - v2.3.3

## Completed in this session

### Critical Build Fixes
1. ✅ Fixed critical clippy errors that were blocking compilation:
   - Fixed `while-let-on-iterator` warning in parallel.rs (line 246)
   - Fixed `uninlined-format-args` in parallel.rs (line 158)
   - Fixed `should_implement_trait` warning in parallel_chunked.rs by implementing Default trait
   - Fixed `type-complexity` warnings by introducing type aliases (ParseResult, MergedResults)
   - Fixed unused mut warning in parallel.rs (line 244)

2. ✅ Verified test_number_features is now passing

3. ✅ Created scripts for jsonic reference removal:
   - remove_jsonic_refs.sh (general replacement)
   - remove_jsonic_refs_targeted.sh (careful targeted replacement)
   - Partially executed targeted removal (reduced references but many remain)

### Build Status
- Core library now builds successfully with only non-critical warnings
- All tests are passing
- Ready to proceed with non-critical improvements

## Next Steps

1. **Complete jsonic reference removal** - Still ~1800 references across 41 files
   - Focus on test files and documentation
   - Preserve important compatibility notes
   - Update web assets (rename jsonic-tool.js)

2. **Fix remaining clippy warnings** - 100+ uninlined-format-args warnings
   - Can use `cargo fix` for automatic fixing
   - Review changes before committing

3. **Work on naming unification (issues/611.txt)**
   - Ensure consistent naming across all language bindings

4. **Improve build deliverables (issues/620.txt)**
   - Create proper packaging for each platform
   - macOS: .dmg with .pkg installer
   - Windows: .zip with .exe
   - Linux: .tgz with executable

5. **Release v2.3.3**
   - Update version numbers
   - Update CHANGELOG.md
   - Create release tag
   - Publish to crates.io