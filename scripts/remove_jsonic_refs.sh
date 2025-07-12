#!/bin/bash
# this_file: scripts/remove_jsonic_refs.sh

# Script to systematically remove jsonic references from the codebase

echo "Removing jsonic references from codebase..."

# Function to replace jsonic references in a file
replace_in_file() {
    local file="$1"
    local temp_file="${file}.tmp"
    
    # Skip binary files and certain directories
    if [[ -f "$file" ]] && file "$file" | grep -q "text"; then
        # Replace various forms of jsonic
        sed -E \
            -e 's/[Jj]sonic/Vexy JSON/g' \
            -e 's/JSONIC/VEXY_JSON/g' \
            -e 's/jsonic-/vexy-json-/g' \
            -e 's/\.jsonic/\.vexy_json/g' \
            -e 's/"jsonic"/"vexy_json"/g' \
            -e "s/'jsonic'/'vexy_json'/g" \
            -e 's/`jsonic`/`vexy_json`/g' \
            -e 's/Jsonic Tool/Vexy JSON Tool/g' \
            -e 's/Jsonic Specific/Vexy JSON Specific/g' \
            -e 's/JavaScript library `vexy_json`/reference JavaScript implementation/g' \
            -e 's/A Rust port of the reference JavaScript implementation/A forgiving JSON parser inspired by JavaScript relaxed JSON parsers/g' \
            "$file" > "$temp_file"
        
        # Only replace if changes were made
        if ! cmp -s "$file" "$temp_file"; then
            mv "$temp_file" "$file"
            echo "Updated: $file"
        else
            rm "$temp_file"
        fi
    fi
}

# Find all text files and replace jsonic references
export -f replace_in_file
find . -type f \
    -not -path "./target/*" \
    -not -path "./dist/*" \
    -not -path "./ref/*" \
    -not -path "./.git/*" \
    -not -path "./node_modules/*" \
    -not -name "*.dmg" \
    -not -name "*.pkg" \
    -not -name "*.wasm" \
    -not -name "remove_jsonic_refs.sh" \
    -exec bash -c 'replace_in_file "$0"' {} \;

echo "jsonic reference removal complete!"