#!/bin/bash
# Update version numbers across the project based on git tag

set -e

# Check if version is provided by release script
if [ -n "$RELEASE_VERSION" ]; then
    VERSION="$RELEASE_VERSION"
else
    # Get the version from git tag
    VERSION=$(./scripts/get-version.sh)
fi

echo "Updating project to version: $VERSION"

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

update_file() {
    local file=$1
    local pattern=$2
    local replacement=$3
    
    if [ -f "$file" ]; then
        if grep -q "$pattern" "$file"; then
            sed -i.bak "$replacement" "$file"
            rm -f "${file}.bak"
            echo -e "${GREEN}✓${NC} Updated $file"
        else
            echo -e "${YELLOW}⚠${NC} Pattern not found in $file"
        fi
    else
        echo -e "${YELLOW}⚠${NC} File not found: $file"
    fi
}

# Update Cargo.toml files - only update package version, not dependency versions
echo "Updating Cargo.toml files..."
for toml in Cargo.toml crates/*/Cargo.toml bindings/*/Cargo.toml; do
    if [ -f "$toml" ]; then
        # Only update the version in the [package] section, not in dependencies
        # This matches version at the start of a line (package version)
        awk -v ver="$VERSION" '
            /^\[package\]/ { in_package=1 }
            /^\[/ && !/^\[package\]/ { in_package=0 }
            in_package && /^version = / { sub(/version = ".*"/, "version = \"" ver "\"") }
            { print }
        ' "$toml" > "$toml.tmp" && mv "$toml.tmp" "$toml"
        echo -e "${GREEN}✓${NC} Updated $toml"
    fi
done

# Update workspace dependencies
echo "Updating workspace dependencies..."
update_file "Cargo.toml" 'vexy_json-core = { version = ".*"' "s/vexy_json-core = { version = \".*\"/vexy_json-core = { version = \"$VERSION\"/"
update_file "Cargo.toml" 'vexy_json = { version = ".*"' "s/vexy_json = { version = \".*\"/vexy_json = { version = \"$VERSION\"/"

# Update Python bindings
echo "Updating Python bindings..."
update_file "bindings/python/pyproject.toml" '^version = ".*"' "s/^version = \".*\"/version = \"$VERSION\"/"
update_file "crates/python/src/lib.rs" '__version__ = ".*"' "s/__version__ = \".*\"/__version__ = \"$VERSION\"/"

# Update package.json files
echo "Updating package.json files..."
for pkg in crates/wasm/pkg/package.json docs/pkg/package.json; do
    if [ -f "$pkg" ]; then
        # Use a different approach for JSON
        if command -v jq &> /dev/null; then
            jq ".version = \"$VERSION\"" "$pkg" > "$pkg.tmp" && mv "$pkg.tmp" "$pkg"
            echo -e "${GREEN}✓${NC} Updated $pkg"
        else
            update_file "$pkg" '"version": ".*"' "s/\"version\": \".*\"/\"version\": \"$VERSION\"/"
        fi
    fi
done

# Update Homebrew formula (only the version, not the URL)
echo "Updating Homebrew formula..."
if [ -f "Formula/vexy_json.rb" ]; then
    # Only update if this looks like a release version (not -dev)
    if [[ ! "$VERSION" =~ -dev$ ]]; then
        update_file "Formula/vexy_json.rb" 'version ".*"' "s/version \".*\"/version \"$VERSION\"/"
        # Note: The URL in the formula should be updated during release
    else
        echo -e "${YELLOW}⚠${NC} Skipping Homebrew formula update for dev version"
    fi
fi

# Create version file for build scripts
echo "$VERSION" > .version

echo
echo "Version update complete: $VERSION"
echo
echo "Files with version $VERSION:"
grep -l "version = \"$VERSION\"" Cargo.toml crates/*/Cargo.toml 2>/dev/null | head -5
echo "..."