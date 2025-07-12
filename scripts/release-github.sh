#!/bin/bash
# GitHub-integrated release script for Vexy JSON

set -e

# Default values
VERSION=""
DRY_RUN=false
SKIP_TESTS=false

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

print_usage() {
    echo "Usage: $0 --version VERSION [OPTIONS]"
    echo
    echo "Options:"
    echo "  --version VERSION    Version to release (e.g., 2.0.0)"
    echo "  --dry-run           Run without making actual changes"
    echo "  --skip-tests        Skip running tests"
    echo "  --help              Show this help message"
    echo
    echo "Example:"
    echo "  $0 --version 2.0.0"
}

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
    exit 1
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --version)
            VERSION="$2"
            shift 2
            ;;
        --dry-run)
            DRY_RUN=true
            shift
            ;;
        --skip-tests)
            SKIP_TESTS=true
            shift
            ;;
        --help)
            print_usage
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            print_usage
            exit 1
            ;;
    esac
done

# Validate version
if [ -z "$VERSION" ]; then
    log_error "Version is required"
    print_usage
    exit 1
fi

if ! [[ "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    log_error "Invalid version format. Expected: X.Y.Z"
    exit 1
fi

echo "=== Vexy JSON GitHub Release v$VERSION ==="
echo
if [ "$DRY_RUN" = true ]; then
    log_warn "Running in dry-run mode - no changes will be made"
fi
echo

# 1. Check prerequisites
log_info "Checking prerequisites..."

# Check if gh CLI is installed
if ! command -v gh &> /dev/null; then
    log_error "GitHub CLI (gh) is not installed. Install it from: https://cli.github.com/"
fi

# Check if authenticated
if ! gh auth status &>/dev/null; then
    log_error "Not authenticated with GitHub. Run: gh auth login"
fi

# Check git status
if [ -n "$(git status --porcelain)" ]; then
    log_warn "Working directory has uncommitted changes"
    git status --short
    echo
    read -p "Continue anyway? (y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi

log_success "Prerequisites checked"
echo

# 2. Run pre-release checks
log_info "Running pre-release checks..."
if [ -f "scripts/pre-release-check.sh" ]; then
    ./scripts/pre-release-check.sh || {
        log_error "Pre-release checks failed"
    }
else
    log_warn "Pre-release check script not found"
fi
echo

# 3. Run tests (unless skipped)
if [ "$SKIP_TESTS" = false ]; then
    log_info "Running tests..."
    if [ "$DRY_RUN" = false ]; then
        cargo test --all-features || log_error "Tests failed"
        cargo check --all-features || log_error "Cargo check failed"
    else
        log_info "[DRY RUN] Would run: cargo test --all-features"
        log_info "[DRY RUN] Would run: cargo check --all-features"
    fi
    log_success "Tests passed"
else
    log_warn "Skipping tests"
fi
echo

# 4. Update version in release.sh if needed
log_info "Checking release.sh version..."
if grep -q "VERSION=\"$VERSION\"" release.sh; then
    log_success "release.sh already has correct version"
else
    if [ "$DRY_RUN" = false ]; then
        sed -i.bak "s/VERSION=\"[^\"]*\"/VERSION=\"$VERSION\"/" release.sh
        rm release.sh.bak
        log_success "Updated release.sh to version $VERSION"
    else
        log_info "[DRY RUN] Would update release.sh to version $VERSION"
    fi
fi
echo

# 5. Create git tag
log_info "Creating git tag v$VERSION..."
if git rev-parse "v$VERSION" >/dev/null 2>&1; then
    log_warn "Tag v$VERSION already exists"
else
    if [ "$DRY_RUN" = false ]; then
        git tag -a "v$VERSION" -m "Release v$VERSION"
        log_success "Created tag v$VERSION"
    else
        log_info "[DRY RUN] Would create tag v$VERSION"
    fi
fi
echo

# 6. Push tag to trigger GitHub Actions
log_info "Pushing tag to GitHub..."
if [ "$DRY_RUN" = false ]; then
    git push origin "v$VERSION" || log_error "Failed to push tag"
    log_success "Pushed tag v$VERSION to GitHub"
else
    log_info "[DRY RUN] Would push tag v$VERSION to GitHub"
fi
echo

# 7. Monitor GitHub Actions
if [ "$DRY_RUN" = false ]; then
    log_info "GitHub Actions release workflow triggered!"
    echo
    echo "You can monitor the release progress at:"
    echo "https://github.com/vexyart/vexy-json/actions"
    echo
    echo "Or watch it here:"
    
    # Wait a moment for the workflow to start
    sleep 5
    
    # Get the workflow run
    RUN_ID=$(gh run list --workflow=release.yml --limit 1 --json databaseId --jq '.[0].databaseId')
    
    if [ -n "$RUN_ID" ]; then
        echo "Workflow run: https://github.com/vexyart/vexy-json/actions/runs/$RUN_ID"
        echo
        echo "Watching workflow progress..."
        gh run watch "$RUN_ID"
    else
        log_warn "Could not find workflow run. Check manually at GitHub Actions."
    fi
else
    log_info "[DRY RUN] Would trigger GitHub Actions release workflow"
fi

echo
echo "=== Release Summary ==="
echo
if [ "$DRY_RUN" = false ]; then
    log_success "Release v$VERSION initiated successfully!"
    echo
    echo "GitHub Actions will now:"
    echo "  • Build binaries for all platforms (macOS, Linux, Windows)"
    echo "  • Create macOS installer (.dmg with .pkg)"
    echo "  • Build and package WASM modules"
    echo "  • Create GitHub release with all artifacts"
    echo "  • Publish to crates.io"
    echo "  • Publish to npm"
    echo "  • Update Homebrew formula"
    echo
    echo "The release will be created as a draft. Once all artifacts are uploaded,"
    echo "it will be automatically published."
else
    log_info "Dry run completed. No changes were made."
    echo
    echo "To perform the actual release, run:"
    echo "  $0 --version $VERSION"
fi