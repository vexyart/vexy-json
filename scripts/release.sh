#!/bin/bash

# Vexy JSON Release Script
# This script automates the complete release process for Vexy JSON
# Usage: ./release.sh VERSION [--dry-run] [--skip-tests]
# Example: ./release.sh 2.0.8

set -euo pipefail

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")"/.. && pwd)"

echo "Running release script from: $(pwd)"

# Error handler
error_handler() {
    local line_no=$1
    local error_code=$2
    error "Error occurred in script at line $line_no with exit code $error_code"
    error "Release process failed. Please check the logs and fix any issues."

    # If we created a tag but failed later, inform the user
    if git rev-parse "v$VERSION" >/dev/null 2>&1; then
        warning "Git tag v$VERSION was created but the release did not complete."
        warning "You may need to delete the tag with: git tag -d v$VERSION"
    fi

    exit $error_code
}

trap 'error_handler ${LINENO} $?' ERR

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
VERSION="" # Will be set from command line
DRY_RUN=false
SKIP_TESTS=false
BUILD_DIR="$PROJECT_ROOT/dist"

# Define utility functions first
log() {
    echo -e "${BLUE}[$(date +'%Y-%m-%d %H:%M:%S')]${NC} $1"
}

success() {
    echo -e "${GREEN}✅ $1${NC}"
}

error() {
    echo -e "${RED}❌ $1${NC}" >&2
}

warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

info() {
    echo -e "${CYAN}ℹ️  $1${NC}"
}

# Check for help flag first
if [[ "${1:-}" == "-h" ]] || [[ "${1:-}" == "--help" ]]; then
    echo "Usage: $0 VERSION [--dry-run] [--skip-tests]"
    echo "  VERSION       Semantic version (e.g., 2.0.8)"
    echo "  --dry-run     Show what would be done without executing"
    echo "  --skip-tests  Skip running tests"
    echo "Example: $0 2.0.8"
    exit 0
fi

# Check if version was provided as first argument
if [[ $# -eq 0 ]]; then
    error "Version number required"
    echo "Usage: $0 VERSION [--dry-run] [--skip-tests]"
    echo "Example: $0 2.0.8"
    exit 1
fi

# Get version from first argument
VERSION="$1"
shift

# Parse remaining command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
    --dry-run)
        DRY_RUN=true
        shift
        ;;
    --skip-tests)
        SKIP_TESTS=true
        shift
        ;;
    *)
        echo "Unknown option $1"
        echo "Usage: $0 VERSION [--dry-run] [--skip-tests]"
        exit 1
        ;;
    esac
done

# Remove 'v' prefix if provided
VERSION="${VERSION#v}"

# Validate version format
if ! [[ "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9]+)?$ ]]; then
    error "Invalid version format: $VERSION"
    echo "Expected format: X.Y.Z or X.Y.Z-suffix"
    exit 1
fi

info "Preparing release for version $VERSION"

run_cmd() {
    local cmd="$1"
    local desc="${2:-$cmd}"

    log "Running: $desc"

    if [ "$DRY_RUN" = true ]; then
        echo -e "${YELLOW}[DRY RUN]${NC} Would execute: $cmd"
        return 0
    fi

    if eval "$cmd"; then
        success "$desc completed"
        return 0
    else
        error "$desc failed"
        return 1
    fi
}

check_prerequisites() {
    log "Checking prerequisites..."

    # Check if we're in the right directory
    if [[ ! -f "$PROJECT_ROOT/Cargo.toml" ]]; then
        error "Not in Vexy JSON project root (no Cargo.toml found)"
        exit 1
    fi

    # Check if we're in a git repository
    if ! git rev-parse --git-dir >/dev/null 2>&1; then
        error "Not in a git repository"
        exit 1
    fi

    # Check for required tools
    local tools=("cargo" "git")
    local optional_tools=("wasm-pack" "npm" "create-dmg" "gh")

    for tool in "${tools[@]}"; do
        if ! command -v "$tool" &>/dev/null; then
            error "Required tool '$tool' not found in PATH"
            exit 1
        fi
    done

    # Check optional tools
    for tool in "${optional_tools[@]}"; do
        if ! command -v "$tool" &>/dev/null; then
            warning "Optional tool '$tool' not found. Some features may be skipped."
        fi
    done

    # Check if we're on the main branch
    local branch=$(git branch --show-current)
    if [[ "$branch" != "main" ]]; then
        warning "Not on main branch (currently on: $branch)"
        if [ "$DRY_RUN" = false ]; then
            read -p "Continue anyway? (y/N): " -n 1 -r
            echo
            if [[ ! $REPLY =~ ^[Yy]$ ]]; then
                exit 1
            fi
        fi
    fi

    # Check for uncommitted changes
    if [[ -n $(git status --porcelain) ]]; then
        warning "Working directory has uncommitted changes"
        git status --short
        info "These changes will be committed as part of the release"
    fi

    success "Prerequisites check passed"
}

update_version() {
    log "Updating version to $VERSION..."

    # Create the git tag first - this becomes the source of truth
    local tag="v$VERSION"

    # Check if tag already exists
    if git rev-parse "$tag" >/dev/null 2>&1; then
        error "Git tag $tag already exists"
        exit 1
    fi

    # Update version files using the script (which will now use our tag)
    if [ -f "./scripts/update-versions.sh" ]; then
        # Temporarily set the version in environment for the script
        export RELEASE_VERSION="$VERSION"
        run_cmd "./scripts/update-versions.sh" "Update all version numbers to $VERSION"
        unset RELEASE_VERSION
    else
        # Fallback to manual updates
        # Update root Cargo.toml
        run_cmd "sed -i.bak 's/^version = .*/version = \"$VERSION\"/' Cargo.toml" "Update root Cargo.toml version"

        # Update all crate Cargo.toml files
        local crates=("crates/core" "crates/cli" "crates/wasm" "crates/serde" "crates/test-utils" "crates/c-api" "bindings/python")
        for crate in "${crates[@]}"; do
            if [[ -f "$crate/Cargo.toml" ]]; then
                run_cmd "sed -i.bak 's/^version = .*/version = \"$VERSION\"/' $crate/Cargo.toml" "Update $crate version"
            fi
        done

        # Update package.json files
        if [[ -f "package.json" ]]; then
            run_cmd "sed -i.bak 's/\"version\": \"[^\"]*\"/\"version\": \"$VERSION\"/' package.json" "Update package.json version"
        fi

        if [[ -f "docs/pkg/package.json" ]]; then
            run_cmd "sed -i.bak 's/\"version\": \"[^\"]*\"/\"version\": \"$VERSION\"/' docs/pkg/package.json" "Update WASM package.json version"
        fi

        # Clean up backup files
        if [ "$DRY_RUN" = false ]; then
            find . -name "*.bak" -delete
        fi
    fi

    success "Version updated to $VERSION"
}

run_tests() {
    if [ "$SKIP_TESTS" = true ]; then
        warning "Skipping tests (--skip-tests flag provided)"
        return 0
    fi

    log "Running comprehensive test suite..."

    # Cargo tests
    run_cmd "cargo test --all-features --workspace --exclude vexy-json-python" "Run all Rust tests"

    # Cargo clippy
    run_cmd "cargo clippy --all-features --workspace --exclude vexy-json-python -- -D warnings -A missing_docs" "Run clippy linter"

    # Cargo fmt check
    run_cmd "cargo fmt --all -- --check" "Check code formatting"

    # Run fuzzing tests (quick run) - requires nightly toolchain
    if [[ -d "fuzz" ]]; then
        # Check if nightly toolchain is available
        if rustc --version | grep -q "nightly"; then
            log "Running fuzz tests (quick run)..."
            cd fuzz
            run_cmd "cargo fuzz list | head -3 | xargs -I {} timeout 30s cargo fuzz run {} || true" "Quick fuzz testing"
            cd "$PROJECT_ROOT"
        else
            warning "Skipping fuzz tests (requires nightly Rust toolchain)"
        fi
    fi

    # Build examples
    run_cmd "cargo build --examples --release" "Build all examples"

    success "All tests passed"
}

build_rust_artifacts() {
    log "Building Rust artifacts..."

    # Create build directory
    run_cmd "mkdir -p \"$BUILD_DIR\"" "Create build directory"

    # Build release binary
    run_cmd "cargo build --release -p vexy-json-cli --bin vexy-json" "Build release CLI binary"

    # Build library
    run_cmd "cargo build --release --lib" "Build release library"

    # Generate documentation
    run_cmd "cargo doc --no-deps --all-features" "Generate documentation"

    # Copy artifacts
    if [ "$DRY_RUN" = false ]; then
        if [[ -f "target/release/vexy-json" ]]; then
            cp "target/release/vexy-json" "$BUILD_DIR/vexy-json-$VERSION-$(uname -m)-$(uname -s | tr '[:upper:]' '[:lower:]')"
        else
            warning "Release binary not found at target/release/vexy-json"
        fi
    fi

    success "Rust artifacts built"
}

build_wasm() {
    if ! command -v wasm-pack &>/dev/null; then
        warning "wasm-pack not found, skipping WebAssembly build"
        return 0
    fi

    log "Building WebAssembly module..."

    if [[ ! -d "$PROJECT_ROOT/crates/wasm" ]]; then
        warning "WASM crate not found at crates/wasm, skipping"
        return 0
    fi

    cd "$PROJECT_ROOT/crates/wasm"

    # Build WASM with wasm-pack
    run_cmd "wasm-pack build --target web --out-dir ../../docs/pkg --release" "Build WASM for web"
    run_cmd "wasm-pack build --target nodejs --out-dir ../../docs/pkg/nodejs --release" "Build WASM for Node.js"

    cd "$PROJECT_ROOT"

    # Update package version in generated package.json
    if [[ -f "docs/pkg/package.json" && "$DRY_RUN" = false ]]; then
        sed -i.bak "s/\"version\": \"[^\"]*\"/\"version\": \"$VERSION\"/" docs/pkg/package.json
        rm -f docs/pkg/package.json.bak
    fi

    success "WebAssembly module built"
}

build_macos_installer() {
    if [[ "$OSTYPE" != "darwin"* ]]; then
        warning "Skipping macOS installer (not on macOS)"
        return 0
    fi

    log "Building macOS installer..."

    local app_name="vexy-json"
    local installer_dir="$BUILD_DIR/macos-installer"
    local dmg_name="vexy-json-$VERSION-macos.dmg"

    run_cmd "mkdir -p \"$installer_dir/pkg-root/usr/local/bin\"" "Create installer structure"

    # Copy binary
    if [ "$DRY_RUN" = false ]; then
        cp "target/release/vexy-json" "$installer_dir/pkg-root/usr/local/bin/"
    fi

    # Create package
    run_cmd "pkgbuild --root \"$installer_dir/pkg-root\" --identifier \"com.twardoch.vexy-json\" --version \"$VERSION\" --install-location \"/\" \"$installer_dir/$app_name.pkg\"" "Create pkg installer"

    # Create DMG
    local dmg_temp_dir="$installer_dir/dmg-temp"
    run_cmd "mkdir -p \"$dmg_temp_dir\"" "Create DMG temp directory"

    if [ "$DRY_RUN" = false ]; then
        cp "$installer_dir/$app_name.pkg" "$dmg_temp_dir/"

        # Create a simple README for the DMG
        cat >"$dmg_temp_dir/README.txt" <<EOF
VEXY_JSON v$VERSION

This package will install the vexy-json command-line tool to /usr/local/bin.

After installation, you can use vexy-json from the command line:
  echo '{"key": "value"}' | vexy-json

For more information, visit: https://github.com/vexyart/vexy-json
EOF
    fi

    # Create DMG
    run_cmd "create-dmg --volname \"Vexy JSON $VERSION\" --window-pos 200 120 --window-size 600 400 --icon-size 100 --app-drop-link 425 120 \"$BUILD_DIR/$dmg_name\" \"$dmg_temp_dir\"" "Create DMG installer"

    success "macOS installer created: $dmg_name"
}

build_linux_packages() {
    log "Building Linux packages..."

    # Build static binary for Linux
    if [[ "$OSTYPE" == "linux-gnu"* ]]; then
        run_cmd "cargo build --release -p vexy-json-cli --target x86_64-unknown-linux-musl --bin vexy-json" "Build static Linux binary"

        if [ "$DRY_RUN" = false ]; then
            cp "target/x86_64-unknown-linux-musl/release/vexy-json" "$BUILD_DIR/vexy-json-$VERSION-x86_64-linux-musl"
        fi
    else
        warning "Skipping Linux builds (not on Linux)"
    fi

    success "Linux packages prepared"
}

create_release_archive() {
    log "Creating release archives..."

    local archive_dir="$BUILD_DIR/vexy-json-$VERSION"
    run_cmd "mkdir -p \"$archive_dir\"" "Create archive directory"

    if [ "$DRY_RUN" = false ]; then
        # Copy documentation
        for file in README.md LICENSE* CHANGELOG.md; do
            if [[ -f "$file" ]]; then
                cp "$file" "$archive_dir/" || warning "Failed to copy $file"
            fi
        done

        # Copy built artifacts
        if [[ -f "target/release/vexy-json" ]]; then
            cp "target/release/vexy-json" "$archive_dir/" || warning "Failed to copy binary"
        else
            warning "No release binary found to include in archive"
        fi

        # Create source archive
        git archive --format=tar.gz --prefix="vexy-json-$VERSION-src/" HEAD >"$BUILD_DIR/vexy-json-$VERSION-src.tar.gz" || {
            warning "Failed to create source archive"
        }

        # Create binary archive if we have files
        if [[ -d "$archive_dir" ]] && [[ -n $(ls -A "$archive_dir") ]]; then
            cd "$BUILD_DIR"
            tar -czf "vexy-json-$VERSION-$(uname -m)-$(uname -s | tr '[:upper:]' '[:lower:]').tar.gz" "vexy-json-$VERSION" || {
                warning "Failed to create binary archive"
            }
            cd "$PROJECT_ROOT"
        else
            warning "No files to archive"
        fi
    fi

    success "Release archives created"
}

commit_and_tag() {
    log "Committing changes and creating git tag..."

    local tag="v$VERSION"

    # Add all changes
    run_cmd "git add -A" "Stage all changes for release"

    # Commit changes
    local commit_msg="Release v$VERSION\n\nThis commit updates all version numbers and prepares the release."

    if [ "$DRY_RUN" = false ]; then
        if git diff --cached --quiet; then
            info "No changes to commit"
        else
            git commit -m "$commit_msg" || {
                error "Failed to commit changes"
                exit 1
            }
            success "Changes committed for v$VERSION"
        fi
    else
        echo -e "${YELLOW}[DRY RUN]${NC} Would commit with message: $commit_msg"
    fi

    # Create annotated tag
    run_cmd "git tag -a \"$tag\" -m \"Release VEXY_JSON v$VERSION\n\nSee CHANGELOG.md for detailed release notes.\"" "Create release tag"

    success "Git tag $tag created"

    # Verify tag was created
    if ! git rev-parse "$tag" >/dev/null 2>&1; then
        error "Failed to create git tag $tag"
        exit 1
    fi
}

run_github_release() {
    log "Preparing GitHub release..."

    if ! command -v gh &>/dev/null; then
        warning "GitHub CLI not found, skipping automated release creation"
        info "Manually create release at: https://github.com/vexyart/vexy-json/releases/new?tag=v$VERSION"
        return 0
    fi

    # Check if gh is authenticated
    if ! gh auth status &>/dev/null; then
        warning "GitHub CLI not authenticated, skipping automated release"
        info "Run 'gh auth login' then manually create release"
        return 0
    fi

    # Create release notes
    local release_notes="$BUILD_DIR/release-notes.md"
    if [ "$DRY_RUN" = false ]; then
        cat >"$release_notes" <<'EOF'
# Vexy JSON v2.0.0 - Major Performance & Architecture Release

🚀 This release represents a major architectural and performance milestone for VEXY_JSON, featuring comprehensive improvements in parsing speed, memory efficiency, and extensibility.

## ✅ Major Features

### ⚡ Performance & Optimization
- **SIMD-Accelerated Parsing** - 2-3x performance improvement for large files
- **Memory Pool V3** - 80% reduction in allocations with typed arenas
- **Parallel Processing** - Intelligent chunked processing for gigabyte-sized JSON files
- **Zero-copy** parsing paths for simple values

### 🏗️ Architecture & Extensibility
- **Streaming Parser V2** - Event-driven API for processing massive files
- **Plugin System** - Extensible architecture with ParserPlugin trait
- **Modular Architecture** - Clean separation with JsonLexer traits
- **AST Builder & Visitor** - Comprehensive AST manipulation capabilities

### 🛡️ Quality & Reliability
- **Error Recovery V2** - ML-based pattern recognition with actionable suggestions
- **Comprehensive Fuzzing** - 4 specialized targets with extensive coverage
- **Enhanced Error Messages** - Context-aware suggestions and recovery strategies
- **Type-Safe Error Handling** - Comprehensive error taxonomy with structured codes

## 📊 Performance Improvements

- **2-3x faster** string scanning with SIMD optimization
- **80% reduction** in allocations for typical workloads
- **Parallel processing** for files > 1MB with intelligent boundary detection
- **String interning** for common JSON keys
- **Streaming capability** for minimal memory usage on large files

## 🔄 Migration from v1.x

- Core parsing API remains compatible
- New streaming and parallel APIs are additive
- Plugin system is entirely new (opt-in)
- Performance improvements are automatic
- Error types have been restructured (but improved)

## 📦 Installation

```bash
cargo install vexy-json --version 2.0.0
```

Or download pre-built binaries from the assets below.

---

**Full Changelog**: https://github.com/vexyart/vexy-json/compare/v1.5.27...v2.0.0
EOF
    fi

    # Collect assets
    local assets=()
    if [[ -f "$BUILD_DIR/vexy-json-$VERSION-macos.dmg" ]]; then
        assets+=("$BUILD_DIR/vexy-json-$VERSION-macos.dmg")
    fi

    # Find all tar.gz files
    while IFS= read -r -d '' file; do
        assets+=("$file")
    done < <(find "$BUILD_DIR" -name "*.tar.gz" -print0)

    # Create release
    local gh_cmd="gh release create 'v$VERSION' --title 'Vexy JSON v$VERSION' --notes-file '$release_notes'"

    # Add assets
    for asset in "${assets[@]}"; do
        if [[ -f "$asset" ]]; then
            gh_cmd="$gh_cmd '$asset'"
        fi
    done

    run_cmd "$gh_cmd" "Create GitHub release"

    success "GitHub release created"
}

publish_crates() {
    log "Publishing to crates.io..."

    warning "Crates.io publishing requires manual intervention"
    info "Run the following commands to publish:"
    info "  cargo publish -p vexy-json-test-utils"
    info "  cargo publish -p vexy-json-core"
    info "  cargo publish -p vexy-json-serde"
    info "  cargo publish -p vexy-json-cli"
    info "  cargo publish -p vexy-json-wasm"
    info "  cargo publish -p vexy-json-c-api"
    info "  cargo publish -p vexy-json"

    if [ "$DRY_RUN" = false ]; then
        # read -p "Publish to crates.io now? (y/N): " -n 1 -r
        REPLY="N" # FIXME TODO
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            # Publish in dependency order
            run_cmd "cargo publish -p vexy-json-test-utils" "Publish vexy-json-test-utils"
            sleep 10 # Wait for crates.io to process
            run_cmd "cargo publish -p vexy-json-core" "Publish vexy-json-core"
            sleep 10
            run_cmd "cargo publish -p vexy-json-serde" "Publish vexy-json-serde"
            sleep 10
            run_cmd "cargo publish -p vexy-json-cli" "Publish vexy-json-cli"
            sleep 10
            run_cmd "cargo publish -p vexy-json-wasm" "Publish vexy-json-wasm"
            sleep 10
            run_cmd "cargo publish -p vexy-json-c-api" "Publish vexy-json-c-api"
            sleep 10
            run_cmd "cargo publish -p vexy-json" "Publish main vexy-json crate"

            success "All crates published to crates.io"
        fi
    fi
}

push_to_remote() {
    log "Pushing to remote repository..."

    local tag="v$VERSION"

    # Get current branch
    local branch=$(git branch --show-current)

    # Check if we have a remote named 'origin'
    if ! git remote | grep -q '^origin'; then
        error "No 'origin' remote found. Please add a remote repository."
        exit 1
    fi

    # Push commits
    run_cmd "git push origin $branch" "Push commits to origin/$branch"

    # Push tag
    run_cmd "git push origin $tag" "Push tag $tag to origin"

    # Verify tag was pushed
    if [ "$DRY_RUN" = false ]; then
        if ! git ls-remote --tags origin | grep -q "refs/tags/$tag"; then
            warning "Tag may not have been pushed successfully. Retrying..."
            git push origin $tag || {
                error "Failed to push tag to remote"
                exit 1
            }
        fi
    fi

    success "Changes and tag pushed to remote repository"
}

cleanup() {
    log "Cleaning up..."

    # Remove build artifacts if requested
    if [ "$DRY_RUN" = false ]; then
        read -p "Remove build directory $BUILD_DIR? (y/N): " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            rm -rf "$BUILD_DIR"
            success "Build directory cleaned"
        fi
    fi
}

main() {
    echo -e "${PURPLE}=== VEXY JSON RELEASE AUTOMATION ===${NC}"

    echo -e "${CYAN}Vexy JSON v$VERSION Release Automation Script${NC}"
    echo -e "${CYAN}=========================================${NC}"
    echo

    if [ "$DRY_RUN" = true ]; then
        warning "DRY RUN MODE - No changes will be made"
        echo
    fi

    # Show release plan
    echo -e "${BLUE}Release Plan:${NC}"
    echo "  1. Check prerequisites and validate environment"
    echo "  2. Update version numbers across all files"
    echo "  3. Run comprehensive test suite"
    echo "  4. Build release artifacts (Rust, WASM, installers)"
    echo "  5. Create release archives in dist/"
    echo "  6. Commit changes and create git tag v$VERSION"
    echo "  7. Push changes and tag to remote repository"
    echo "  8. Create GitHub release (if gh CLI available)"
    echo "  9. Publish to crates.io (interactive)"
    echo " 10. Cleanup temporary files"
    echo

    echo

    # Track which steps completed
    local steps_completed=()

    # Execute release steps
    check_prerequisites && steps_completed+=("prerequisites")
    update_version && steps_completed+=("version_update")
    run_tests && steps_completed+=("tests")
    build_rust_artifacts && steps_completed+=("rust_build")
    build_wasm && steps_completed+=("wasm_build")
    build_macos_installer && steps_completed+=("macos_installer")
    build_linux_packages && steps_completed+=("linux_packages")
    create_release_archive && steps_completed+=("archives")
    commit_and_tag && steps_completed+=("git_tag")
    push_to_remote && steps_completed+=("git_push")
    run_github_release && steps_completed+=("github_release")
    publish_crates && steps_completed+=("crates_publish")
    cleanup && steps_completed+=("cleanup")

    echo
    echo -e "${GREEN}🎉 Vexy JSON v$VERSION release completed successfully!${NC}"
    echo
    echo -e "${BLUE}Completed steps:${NC}"
    for step in "${steps_completed[@]}"; do
        echo "  ✓ $step"
    done
    echo
    echo -e "${BLUE}Release artifacts created in: $BUILD_DIR${NC}"
    echo -e "${BLUE}Git tag created and pushed: v$VERSION${NC}"
    echo -e "${BLUE}Next steps:${NC}"
    echo "  1. Verify GitHub release: https://github.com/vexyart/vexy-json/releases"
    echo "  2. Update documentation websites"
    echo "  3. Announce the release"
    echo
}

# Handle Ctrl+C gracefully
interrupt_handler() {
    echo -e "\n${RED}Release interrupted by user${NC}"

    # If we created a tag but didn't push it, inform the user
    if [ -n "${VERSION:-}" ] && git rev-parse "v$VERSION" >/dev/null 2>&1; then
        if ! git ls-remote --tags origin 2>/dev/null | grep -q "refs/tags/v$VERSION"; then
            warning "Local tag v$VERSION was created but not pushed."
            warning "You can delete it with: git tag -d v$VERSION"
        fi
    fi

    exit 1
}

trap interrupt_handler INT

# Run main function
main "$@"
