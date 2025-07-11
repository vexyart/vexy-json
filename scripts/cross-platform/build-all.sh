#!/bin/bash

# Cross-platform build script for Vexy JSON
# Builds binaries for all supported platforms using cross-compilation

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
BUILD_DIR="$PROJECT_ROOT/target/cross-platform-builds"
VERSION="${VERSION:-$(grep '^version' "$PROJECT_ROOT/Cargo.toml" | head -1 | cut -d'"' -f2)}"

# Supported targets
TARGETS=(
    "x86_64-unknown-linux-gnu"          # Linux x86_64
    "x86_64-unknown-linux-musl"         # Linux x86_64 (static)
    "aarch64-unknown-linux-gnu"         # Linux ARM64
    "x86_64-pc-windows-msvc"            # Windows x86_64
    "x86_64-apple-darwin"               # macOS Intel
    "aarch64-apple-darwin"              # macOS Apple Silicon
    "x86_64-unknown-freebsd"            # FreeBSD
    "wasm32-unknown-unknown"            # WebAssembly
)

# Utility functions
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

install_prerequisites() {
    log "Installing build prerequisites..."
    
    # Install cross compilation tool if not present
    if ! command -v cross &> /dev/null; then
        log "Installing cross..."
        cargo install cross --git https://github.com/cross-rs/cross
    fi
    
    # Install additional Rust targets
    for target in "${TARGETS[@]}"; do
        if [[ "$target" != "wasm32-unknown-unknown" ]]; then
            log "Adding target: $target"
            rustup target add "$target" || warning "Failed to add target $target"
        fi
    done
    
    # Install wasm-pack for WebAssembly builds
    if [[ " ${TARGETS[*]} " =~ " wasm32-unknown-unknown " ]]; then
        if ! command -v wasm-pack &> /dev/null; then
            log "Installing wasm-pack..."
            curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
        fi
    fi
    
    success "Prerequisites installed"
}

build_target() {
    local target="$1"
    local use_cross="${2:-auto}"
    
    log "Building for target: $target"
    
    # Determine if we should use cross or cargo
    local build_cmd="cargo"
    if [[ "$use_cross" == "yes" ]] || [[ "$use_cross" == "auto" && "$target" != *"$(uname -m)"* ]]; then
        if command -v cross &> /dev/null; then
            build_cmd="cross"
        else
            warning "cross not available, falling back to cargo"
        fi
    fi
    
    # Special handling for WebAssembly
    if [[ "$target" == "wasm32-unknown-unknown" ]]; then
        build_wasm
        return $?
    fi
    
    # Build the binary
    local output_dir="$PROJECT_ROOT/target/$target/release"
    mkdir -p "$output_dir"
    
    if $build_cmd build --release --bin vexy_json --target "$target"; then
        # Copy binary to build directory
        local binary_name="vexy_json"
        if [[ "$target" == *"windows"* ]]; then
            binary_name="vexy_json.exe"
        fi
        
        local output_name="vexy_json-$VERSION-$target"
        if [[ "$target" == *"windows"* ]]; then
            output_name="$output_name.exe"
        fi
        
        mkdir -p "$BUILD_DIR"
        cp "$output_dir/$binary_name" "$BUILD_DIR/$output_name"
        
        # Strip binary for size optimization (Unix only)
        if [[ "$target" != *"windows"* ]] && command -v strip &> /dev/null; then
            strip "$BUILD_DIR/$output_name" || warning "Failed to strip binary"
        fi
        
        success "Built $target -> $output_name"
        return 0
    else
        error "Failed to build for $target"
        return 1
    fi
}

build_wasm() {
    log "Building WebAssembly packages..."
    
    local wasm_dir="$PROJECT_ROOT/crates/wasm"
    if [[ ! -d "$wasm_dir" ]]; then
        error "WASM crate directory not found: $wasm_dir"
        return 1
    fi
    
    cd "$wasm_dir"
    
    # Build for web
    if wasm-pack build --target web --out-dir "$BUILD_DIR/wasm-web" --release; then
        success "Built WASM for web"
    else
        error "Failed to build WASM for web"
        return 1
    fi
    
    # Build for Node.js
    if wasm-pack build --target nodejs --out-dir "$BUILD_DIR/wasm-nodejs" --release; then
        success "Built WASM for Node.js"
    else
        error "Failed to build WASM for Node.js"
        return 1
    fi
    
    cd "$PROJECT_ROOT"
    
    # Create archives
    cd "$BUILD_DIR"
    tar -czf "vexy_json-$VERSION-wasm-web.tar.gz" wasm-web/
    tar -czf "vexy_json-$VERSION-wasm-nodejs.tar.gz" wasm-nodejs/
    cd "$PROJECT_ROOT"
    
    return 0
}

create_universal_macos() {
    log "Creating universal macOS binary..."
    
    local intel_binary="$BUILD_DIR/vexy_json-$VERSION-x86_64-apple-darwin"
    local arm_binary="$BUILD_DIR/vexy_json-$VERSION-aarch64-apple-darwin"
    local universal_binary="$BUILD_DIR/vexy_json-$VERSION-universal-apple-darwin"
    
    if [[ -f "$intel_binary" && -f "$arm_binary" ]]; then
        if command -v lipo &> /dev/null; then
            lipo -create -output "$universal_binary" "$intel_binary" "$arm_binary"
            success "Created universal macOS binary"
        else
            warning "lipo not available, skipping universal binary creation"
        fi
    else
        warning "Both Intel and ARM64 macOS binaries not found, skipping universal binary"
    fi
}

create_archives() {
    log "Creating release archives..."
    
    cd "$BUILD_DIR"
    
    # Create individual archives for each binary
    for file in vexy_json-$VERSION-*; do
        if [[ -f "$file" && "$file" != *.tar.gz && "$file" != *.zip ]]; then
            local archive_name="${file}.tar.gz"
            tar -czf "$archive_name" "$file"
            success "Created archive: $archive_name"
        fi
    done
    
    # Create a comprehensive archive with all binaries
    tar -czf "vexy_json-$VERSION-all-platforms.tar.gz" vexy_json-$VERSION-*
    success "Created comprehensive archive: vexy_json-$VERSION-all-platforms.tar.gz"
    
    cd "$PROJECT_ROOT"
}

generate_checksums() {
    log "Generating checksums..."
    
    cd "$BUILD_DIR"
    
    # Generate SHA256 checksums
    if command -v sha256sum &> /dev/null; then
        sha256sum vexy_json-$VERSION-* > checksums.sha256
    elif command -v shasum &> /dev/null; then
        shasum -a 256 vexy_json-$VERSION-* > checksums.sha256
    else
        warning "No SHA256 utility found, skipping checksum generation"
        cd "$PROJECT_ROOT"
        return
    fi
    
    success "Generated checksums.sha256"
    cd "$PROJECT_ROOT"
}

print_summary() {
    echo
    echo -e "${GREEN}🎉 Cross-platform build completed!${NC}"
    echo
    echo -e "${BLUE}Build artifacts in: $BUILD_DIR${NC}"
    echo
    
    if [[ -d "$BUILD_DIR" ]]; then
        echo -e "${BLUE}Generated files:${NC}"
        ls -la "$BUILD_DIR" | grep -E "(vexy_json-|checksums)" | while read -r line; do
            echo "  $line"
        done
    fi
    
    echo
    echo -e "${BLUE}Next steps:${NC}"
    echo "  1. Test the binaries on their respective platforms"
    echo "  2. Upload to GitHub releases"
    echo "  3. Update package managers (Homebrew, etc.)"
}

main() {
    echo -e "${BLUE}
╔══════════════════════════════════════╗
║       VEXY_JSON Cross-Platform Build     ║
║              v$VERSION                 ║
╚══════════════════════════════════════╝
${NC}"
    
    # Parse command line arguments
    local targets_to_build=("${TARGETS[@]}")
    local force_cross="auto"
    
    while [[ $# -gt 0 ]]; do
        case $1 in
            --target)
                targets_to_build=("$2")
                shift 2
                ;;
            --targets)
                IFS=',' read -ra targets_to_build <<< "$2"
                shift 2
                ;;
            --force-cross)
                force_cross="yes"
                shift
                ;;
            --no-cross)
                force_cross="no"
                shift
                ;;
            -h|--help)
                echo "Usage: $0 [OPTIONS]"
                echo "Options:"
                echo "  --target TARGET       Build only specified target"
                echo "  --targets TARGET,..   Build only specified targets (comma-separated)"
                echo "  --force-cross         Always use cross for compilation"
                echo "  --no-cross           Never use cross, only cargo"
                echo "  -h, --help           Show this help"
                echo
                echo "Supported targets:"
                printf '  %s\n' "${TARGETS[@]}"
                exit 0
                ;;
            *)
                error "Unknown option: $1"
                exit 1
                ;;
        esac
    done
    
    log "Building Vexy JSON v$VERSION for ${#targets_to_build[@]} targets"
    
    # Create build directory
    mkdir -p "$BUILD_DIR"
    
    # Install prerequisites
    install_prerequisites
    
    # Build for each target
    local failed_targets=()
    for target in "${targets_to_build[@]}"; do
        if ! build_target "$target" "$force_cross"; then
            failed_targets+=("$target")
        fi
    done
    
    # Create universal macOS binary if both architectures were built
    if [[ " ${targets_to_build[*]} " =~ " x86_64-apple-darwin " ]] && [[ " ${targets_to_build[*]} " =~ " aarch64-apple-darwin " ]]; then
        create_universal_macos
    fi
    
    # Create archives and checksums
    create_archives
    generate_checksums
    
    # Print summary
    print_summary
    
    # Report any failures
    if [[ ${#failed_targets[@]} -gt 0 ]]; then
        echo
        error "Failed to build for the following targets:"
        printf '  %s\n' "${failed_targets[@]}"
        exit 1
    fi
    
    success "All targets built successfully!"
}

# Handle Ctrl+C gracefully
trap 'echo -e "\n${RED}Build interrupted by user${NC}"; exit 1' INT

# Run main function
main "$@"