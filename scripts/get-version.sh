#!/bin/bash
# Get version from git tag or fallback to Cargo.toml

# Default fallback version
FALLBACK_VERSION="2.0.0"

# Function to extract version from Cargo.toml
get_cargo_version() {
    if [ -f "Cargo.toml" ]; then
        grep -E '^version = ".*"' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/'
    else
        echo "$FALLBACK_VERSION"
    fi
}

# Function to get version from git
get_git_version() {
    # Check if we're in a git repository
    if ! git rev-parse --git-dir > /dev/null 2>&1; then
        return 1
    fi
    
    # Try to get the exact tag for the current commit
    TAG=$(git describe --exact-match --tags 2>/dev/null)
    
    if [ $? -eq 0 ]; then
        # Remove 'v' prefix if present
        VERSION=${TAG#v}
        echo "$VERSION"
        return 0
    fi
    
    # If no exact tag, try to get the most recent tag with commit info
    TAG=$(git describe --tags --always 2>/dev/null)
    
    if [ $? -eq 0 ] && [ "$TAG" != "" ]; then
        # Check if this looks like a version tag
        if [[ "$TAG" =~ ^v?[0-9]+\.[0-9]+\.[0-9]+ ]]; then
            # Remove 'v' prefix and any commit suffix
            VERSION=$(echo "$TAG" | sed 's/^v//' | sed 's/-.*//')
            # If we have commits since the tag, append -dev
            if [[ "$TAG" =~ -[0-9]+-g[0-9a-f]+ ]]; then
                VERSION="${VERSION}-dev"
            fi
            echo "$VERSION"
            return 0
        fi
    fi
    
    return 1
}

# Main logic
if VERSION=$(get_git_version); then
    echo "$VERSION"
else
    # Fallback to Cargo.toml version
    get_cargo_version
fi