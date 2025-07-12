#!/bin/bash
# this_file: scripts/remove_jsonic_refs_targeted.sh

# Script to carefully remove jsonic references while preserving important context

echo "Removing jsonic references with targeted replacements..."

# Replace in Python bindings description
if [ -f "bindings/python/pyproject.toml" ]; then
    echo "Updating Python bindings description..."
    sed -i.bak 's/Rust port of the JavaScript library jsonic/Forgiving JSON parser with relaxed syntax support/g' bindings/python/pyproject.toml
    sed -i.bak 's/"jsonic"//g' bindings/python/pyproject.toml  # Remove from keywords
    rm bindings/python/pyproject.toml.bak
fi

# Update main library documentation
if [ -f "src/lib.rs" ]; then
    echo "Already updated src/lib.rs"
fi

# Update CLAUDE.md
if [ -f "CLAUDE.md" ]; then
    echo "Updating CLAUDE.md..."
    sed -i.bak 's/`vexy_json` is a Rust port of the JavaScript library `jsonic`/`vexy_json` is a forgiving JSON parser/g' CLAUDE.md
    sed -i.bak 's/jsonic/the reference implementation/g' CLAUDE.md
    rm CLAUDE.md.bak
fi

# Update documentation files in docs/
echo "Updating documentation files..."
find docs -name "*.md" -type f | while read -r file; do
    # Replace jsonic tool references with vexy-json tool
    sed -i.bak 's/jsonic-tool/vexy-json-tool/g' "$file"
    sed -i.bak 's/Jsonic Tool/Vexy JSON Tool/g' "$file"
    sed -i.bak 's/Jsonic Specific/Vexy JSON Specific/g' "$file"
    # Replace generic jsonic references
    sed -i.bak 's/\[jsonic\]/[the reference implementation]/g' "$file"
    sed -i.bak 's/port of jsonic/forgiving JSON parser/g' "$file"
    rm "${file}.bak"
done

# Update test files - only update descriptive text, not test names or compatibility notes
echo "Updating test file descriptions..."
find tests -name "*.rs" -type f | while read -r file; do
    # Replace "ported from jsonic" with "based on reference implementation"
    sed -i.bak 's/ported from jsonic/based on reference implementation tests from/g' "$file"
    # Keep compatibility notes as they are important for understanding behavior
    rm "${file}.bak"
done

# Update benchmark file
if [ -f "benches/comprehensive_comparison.rs" ]; then
    echo "Updating benchmark file..."
    # Replace jsonic variable names with ref_impl
    sed -i.bak 's/jsonic_time/ref_impl_time/g' benches/comprehensive_comparison.rs
    sed -i.bak 's/jsonic_success/ref_impl_success/g' benches/comprehensive_comparison.rs
    sed -i.bak 's/jsonic_error/ref_impl_error/g' benches/comprehensive_comparison.rs
    sed -i.bak 's/run_jsonic_benchmark/run_ref_impl_benchmark/g' benches/comprehensive_comparison.rs
    sed -i.bak 's/Jsonic/Reference Implementation/g' benches/comprehensive_comparison.rs
    rm benches/comprehensive_comparison.rs.bak
fi

# Update real world scenarios test
if [ -f "tests/real_world_scenarios.rs" ]; then
    echo "Updating real world scenarios test..."
    sed -i.bak 's/test_json_to_jsonic_migration/test_json_to_vexy_json_migration/g' tests/real_world_scenarios.rs
    sed -i.bak 's/jsonic_version/vexy_json_version/g' tests/real_world_scenarios.rs
    sed -i.bak 's/"jsonic"/"vexy_json"/g' tests/real_world_scenarios.rs
    rm tests/real_world_scenarios.rs.bak
fi

# Rename jsonic-tool.js to vexy-json-tool.js
if [ -f "docs/assets/js/jsonic-tool.js" ]; then
    echo "Renaming jsonic-tool.js..."
    mv "docs/assets/js/jsonic-tool.js" "docs/assets/js/vexy-json-tool.js"
fi

# Update HTML files that reference the tool
find docs -name "*.html" -type f | while read -r file; do
    sed -i.bak 's/jsonic-tool\.js/vexy-json-tool.js/g' "$file"
    rm "${file}.bak"
done

echo "Targeted jsonic reference removal complete!"
echo "Note: Some references in test comments were preserved as they provide important context about test origins and compatibility."