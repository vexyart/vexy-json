#!/bin/bash
# this_file: release.sh
# Wrapper for release script with automatic version increment

# Function to get the next version based on increment type
get_next_version() {
    local increment_type="${1:-patch}"  # Default to patch
    
    # Get the latest tag that looks like a version
    local latest_tag=$(git tag -l "v*" | grep -E '^v[0-9]+\.[0-9]+\.[0-9]+$' | sort -V | tail -n1)
    
    if [ -z "$latest_tag" ]; then
        echo "1.0.0"
        return
    fi
    
    # Remove the 'v' prefix
    local version=${latest_tag#v}
    
    # Split version into components
    local major=$(echo "$version" | cut -d. -f1)
    local minor=$(echo "$version" | cut -d. -f2)
    local patch=$(echo "$version" | cut -d. -f3)
    
    # Increment based on type
    case "$increment_type" in
        major)
            major=$((major + 1))
            minor=0
            patch=0
            ;;
        minor)
            minor=$((minor + 1))
            patch=0
            ;;
        patch|*)
            patch=$((patch + 1))
            ;;
    esac
    
    echo "${major}.${minor}.${patch}"
}

# Special handling for help flag
if [[ "$1" == "--help" ]] || [[ "$1" == "-h" ]]; then
    echo "Usage: $0 [VERSION|--major|--minor|--patch] [--dry-run] [--skip-tests]"
    echo
    echo "VERSION can be:"
    echo "  - A specific version number (e.g., 1.2.3)"
    echo "  - --major  : Increment major version (1.0.13 -> 2.0.0)"
    echo "  - --minor  : Increment minor version (1.0.13 -> 1.1.0)"
    echo "  - --patch  : Increment patch version (1.0.13 -> 1.0.14) [default]"
    echo "  - (empty)  : Same as --patch"
    echo
    echo "Current latest version: $(git tag -l "v*" | grep -E '^v[0-9]+\.[0-9]+\.[0-9]+$' | sort -V | tail -n1)"
    echo
    exit 0
fi

# Determine version increment type
INCREMENT_TYPE="patch"
if [[ "$1" == "--major" ]]; then
    INCREMENT_TYPE="major"
    shift
elif [[ "$1" == "--minor" ]]; then
    INCREMENT_TYPE="minor"
    shift
elif [[ "$1" == "--patch" ]]; then
    INCREMENT_TYPE="patch"
    shift
fi

# Check if version was provided
if [ $# -eq 0 ] || [[ "$1" == --* ]]; then
    # No version provided or first arg is a flag
    VERSION=$(get_next_version "$INCREMENT_TYPE")
    echo "No version specified. Auto-incrementing $INCREMENT_TYPE version to: $VERSION"
    echo
    
    # Call the actual release script with the auto-generated version
    if [ $# -eq 0 ]; then
        exec ./scripts/release.sh "$VERSION"
    else
        # Insert version before any flags
        exec ./scripts/release.sh "$VERSION" "$@"
    fi
else
    # Version was provided, forward all arguments as-is
    exec ./scripts/release.sh "$@"
fi