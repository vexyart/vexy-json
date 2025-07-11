#!/bin/bash
# Pre-release checklist for Vexy JSON v2.0.0

set -e

echo "=== Vexy JSON v2.0.0 Pre-Release Checklist ==="
echo

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

check_pass() {
    echo -e "${GREEN}✓${NC} $1"
}

check_fail() {
    echo -e "${RED}✗${NC} $1"
    exit 1
}

check_warn() {
    echo -e "${YELLOW}⚠${NC} $1"
}

# 1. Check version numbers
echo "1. Checking version numbers..."
VERSION="2.0.0"

# Check Cargo.toml files
if grep -q "version = \"$VERSION\"" Cargo.toml; then
    check_pass "Root Cargo.toml version is $VERSION"
else
    check_fail "Root Cargo.toml version is not $VERSION"
fi

for crate in core cli wasm serde test-utils c-api python; do
    if grep -q "version = \"$VERSION\"" "crates/$crate/Cargo.toml"; then
        check_pass "crates/$crate/Cargo.toml version is $VERSION"
    else
        check_fail "crates/$crate/Cargo.toml version is not $VERSION"
    fi
done

echo

# 2. Check GitHub Actions workflows
echo "2. Checking GitHub Actions workflows..."
for workflow in ci release fuzz docs; do
    if [ -f ".github/workflows/$workflow.yml" ]; then
        check_pass "GitHub workflow $workflow.yml exists"
    else
        check_fail "GitHub workflow $workflow.yml is missing"
    fi
done

echo

# 3. Check build scripts
echo "3. Checking build scripts..."
for script in build.sh release.sh scripts/build-wasm.sh scripts/package-macos.sh; do
    if [ -f "$script" ]; then
        if [ -x "$script" ]; then
            check_pass "$script exists and is executable"
        else
            check_warn "$script exists but is not executable - run: chmod +x $script"
        fi
    else
        check_fail "$script is missing"
    fi
done

echo

# 4. Check documentation
echo "4. Checking documentation..."
if [ -d "docs" ]; then
    check_pass "Documentation directory exists"
    
    for doc in index.md api.md usage.md release-notes.md migration-guide.md; do
        if [ -f "docs/$doc" ]; then
            if grep -q "2.0.0" "docs/$doc"; then
                check_pass "docs/$doc contains v2.0.0 references"
            else
                check_warn "docs/$doc may not be updated for v2.0.0"
            fi
        else
            check_fail "docs/$doc is missing"
        fi
    done
else
    check_fail "Documentation directory is missing"
fi

echo

# 5. Run basic build test
echo "5. Running basic build test..."
if cargo check --all-features &>/dev/null; then
    check_pass "Cargo check passes"
else
    check_fail "Cargo check failed"
fi

echo

# 6. Check for uncommitted changes
echo "6. Checking git status..."
if [ -z "$(git status --porcelain)" ]; then
    check_pass "Working directory is clean"
else
    check_warn "There are uncommitted changes:"
    git status --short
fi

echo

# 7. Check README
echo "7. Checking README..."
if grep -q "Vexy JSON v2.0.0" README.md; then
    check_pass "README.md contains v2.0.0"
else
    check_fail "README.md is not updated for v2.0.0"
fi

echo

# 8. Summary
echo "=== Pre-Release Summary ==="
echo
echo "If all checks passed, you're ready to release v2.0.0!"
echo
echo "Next steps:"
echo "1. Commit any remaining changes"
echo "2. Run: ./release.sh --version 2.0.0"
echo "3. Or push a tag: git tag v2.0.0 && git push origin v2.0.0"
echo
echo "The GitHub Actions will automatically:"
echo "- Build binaries for all platforms"
echo "- Create macOS installer (.dmg with .pkg)"
echo "- Build WASM modules"
echo "- Create GitHub release with all artifacts"
echo "- Publish to crates.io and npm"