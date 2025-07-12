#!/bin/bash

# macOS Installer Build Script for Vexy JSON
# Creates a professional .dmg installer with .pkg that installs CLI to /usr/local/bin

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
VERSION="${VERSION:-$(grep '^version' "$PROJECT_ROOT/Cargo.toml" | head -1 | cut -d'"' -f2)}"
BUILD_DIR="$PROJECT_ROOT/target/macos-installer"
APP_NAME="vexy_json"
BUNDLE_ID="com.twardoch.vexy_json"
DMG_NAME="vexy_json-$VERSION-macos.dmg"

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

check_prerequisites() {
    log "Checking prerequisites..."
    
    # Check if we're on macOS
    if [[ "$OSTYPE" != "darwin"* ]]; then
        error "This script must be run on macOS"
        exit 1
    fi
    
    # Check for required tools
    local tools=("cargo" "rustup" "pkgbuild" "productbuild" "lipo")
    for tool in "${tools[@]}"; do
        if ! command -v "$tool" &> /dev/null; then
            error "Required tool '$tool' not found in PATH"
            exit 1
        fi
    done
    
    # Check for create-dmg
    if ! command -v create-dmg &> /dev/null; then
        warning "create-dmg not found, attempting to install..."
        if command -v brew &> /dev/null; then
            brew install create-dmg
        elif command -v npm &> /dev/null; then
            npm install -g create-dmg
        else
            error "Please install create-dmg: brew install create-dmg"
            exit 1
        fi
    fi
    
    # Install required Rust targets
    rustup target add x86_64-apple-darwin
    rustup target add aarch64-apple-darwin
    
    success "Prerequisites check passed"
}

build_universal_binary() {
    log "Building universal binary..."
    
    # Build for Intel
    log "Building for Intel (x86_64)..."
    cargo build --release --bin vexy_json --target x86_64-apple-darwin
    
    # Build for Apple Silicon
    log "Building for Apple Silicon (aarch64)..."
    cargo build --release --bin vexy_json --target aarch64-apple-darwin
    
    # Create universal binary
    log "Creating universal binary..."
    mkdir -p "$PROJECT_ROOT/target/release"
    lipo -create -output "$PROJECT_ROOT/target/release/vexy_json" \
        "$PROJECT_ROOT/target/x86_64-apple-darwin/release/vexy_json" \
        "$PROJECT_ROOT/target/aarch64-apple-darwin/release/vexy_json"
    
    # Verify the universal binary
    if lipo -info "$PROJECT_ROOT/target/release/vexy_json" | grep -q "x86_64 arm64"; then
        success "Universal binary created successfully"
    else
        error "Failed to create universal binary"
        exit 1
    fi
}

create_installer_structure() {
    log "Creating installer structure..."
    
    # Clean and create build directory
    rm -rf "$BUILD_DIR"
    mkdir -p "$BUILD_DIR"
    
    # Create package root structure
    local pkg_root="$BUILD_DIR/pkg-root"
    mkdir -p "$pkg_root/usr/local/bin"
    
    # Copy the universal binary
    cp "$PROJECT_ROOT/target/release/vexy_json" "$pkg_root/usr/local/bin/"
    chmod +x "$pkg_root/usr/local/bin/vexy_json"
    
    # Create scripts directory for pre/post install scripts
    mkdir -p "$BUILD_DIR/scripts"
    
    # Create postinstall script
    cat > "$BUILD_DIR/scripts/postinstall" << 'EOF'
#!/bin/bash

# Post-installation script for Vexy JSON

# Add /usr/local/bin to PATH if not already present
for shell_profile in "$HOME/.bashrc" "$HOME/.bash_profile" "$HOME/.zshrc" "$HOME/.profile"; do
    if [[ -f "$shell_profile" ]] && ! grep -q "/usr/local/bin" "$shell_profile"; then
        echo 'export PATH="/usr/local/bin:$PATH"' >> "$shell_profile"
    fi
done

# Verify installation
if command -v vexy_json &> /dev/null; then
    echo "Vexy JSON installed successfully!"
    echo "Version: $(vexy_json --version 2>/dev/null || echo 'Unknown')"
    echo "You may need to restart your terminal or run 'source ~/.bashrc' (or similar) to use vexy_json."
else
    echo "Installation completed, but vexy_json may not be in your PATH."
    echo "Try restarting your terminal or adding /usr/local/bin to your PATH."
fi

exit 0
EOF
    
    chmod +x "$BUILD_DIR/scripts/postinstall"
    
    success "Installer structure created"
}

create_package() {
    log "Creating .pkg installer..."
    
    local pkg_file="$BUILD_DIR/$APP_NAME.pkg"
    
    # Build the package
    pkgbuild \
        --root "$BUILD_DIR/pkg-root" \
        --identifier "$BUNDLE_ID" \
        --version "$VERSION" \
        --install-location "/" \
        --scripts "$BUILD_DIR/scripts" \
        "$pkg_file"
    
    if [[ -f "$pkg_file" ]]; then
        success "Package created: $pkg_file"
    else
        error "Failed to create package"
        exit 1
    fi
    
    # Get package size for display
    local pkg_size=$(du -h "$pkg_file" | cut -f1)
    log "Package size: $pkg_size"
}

create_dmg() {
    log "Creating DMG installer..."
    
    local dmg_temp_dir="$BUILD_DIR/dmg-temp"
    local final_dmg="$PROJECT_ROOT/$DMG_NAME"
    
    # Clean up any existing DMG
    rm -f "$final_dmg"
    
    # Create DMG temporary directory
    rm -rf "$dmg_temp_dir"
    mkdir -p "$dmg_temp_dir"
    
    # Copy package to DMG temp directory
    cp "$BUILD_DIR/$APP_NAME.pkg" "$dmg_temp_dir/"
    
    # Create README for the DMG
    cat > "$dmg_temp_dir/README.txt" << EOF
VEXY_JSON v$VERSION - High-Performance JSON Parser

This installer will install the vexy_json command-line tool to /usr/local/bin.

Installation Instructions:
1. Double-click on vexy_json.pkg to run the installer
2. Follow the installation prompts
3. Restart your terminal or run 'source ~/.bashrc' to update your PATH

After installation, you can use vexy_json from the command line:
  echo '{"key": "value"}' | vexy_json
  vexy_json --help

Features:
• SIMD-accelerated parsing (2-3x faster)
• Memory pool optimization (80% less allocation)
• Parallel processing for large files
• Streaming API for gigabyte-sized files
• Plugin system for extensibility
• Enhanced error recovery with suggestions

For more information:
  Website: https://github.com/vexyart/vexy-json
  Documentation: https://twardoch.github.io/vexy_json/

License: MIT OR Apache-2.0
EOF
    
    # Create License file
    if [[ -f "$PROJECT_ROOT/LICENSE" ]]; then
        cp "$PROJECT_ROOT/LICENSE" "$dmg_temp_dir/"
    elif [[ -f "$PROJECT_ROOT/LICENSE-MIT" ]]; then
        cp "$PROJECT_ROOT/LICENSE-MIT" "$dmg_temp_dir/LICENSE"
    fi
    
    # Create the DMG with create-dmg
    create-dmg \
        --volname "Vexy JSON v$VERSION" \
        --volicon "$dmg_temp_dir" \
        --window-pos 200 120 \
        --window-size 800 600 \
        --icon-size 100 \
        --icon "$APP_NAME.pkg" 200 190 \
        --hide-extension "$APP_NAME.pkg" \
        --app-drop-link 600 185 \
        --background-color "#f0f0f0" \
        "$final_dmg" \
        "$dmg_temp_dir"
    
    if [[ -f "$final_dmg" ]]; then
        success "DMG created: $final_dmg"
        
        # Get DMG size
        local dmg_size=$(du -h "$final_dmg" | cut -f1)
        log "DMG size: $dmg_size"
        
        # Verify DMG can be mounted
        if hdiutil attach "$final_dmg" -readonly -nobrowse -mountpoint "/tmp/vexy_json-verify-$$"; then
            log "DMG verification: mountable ✓"
            hdiutil detach "/tmp/vexy_json-verify-$$" || true
        else
            warning "DMG verification failed - may not be mountable"
        fi
    else
        error "Failed to create DMG"
        exit 1
    fi
}

create_zip_alternative() {
    log "Creating ZIP alternative..."
    
    local zip_dir="$BUILD_DIR/zip-package"
    local zip_file="$PROJECT_ROOT/vexy_json-$VERSION-macos.zip"
    
    mkdir -p "$zip_dir"
    
    # Copy binary
    cp "$PROJECT_ROOT/target/release/vexy_json" "$zip_dir/"
    
    # Create installation script
    cat > "$zip_dir/install.sh" << 'EOF'
#!/bin/bash

# Simple installation script for Vexy JSON

set -e

echo "Installing Vexy JSON to /usr/local/bin..."

# Check if we have write permissions
if [[ ! -w "/usr/local/bin" ]]; then
    echo "Note: You may be prompted for your password to install to /usr/local/bin"
    sudo cp vexy_json /usr/local/bin/
    sudo chmod +x /usr/local/bin/vexy_json
else
    cp vexy_json /usr/local/bin/
    chmod +x /usr/local/bin/vexy_json
fi

echo "Vexy JSON installed successfully!"
echo "Try: vexy_json --help"
EOF
    
    chmod +x "$zip_dir/install.sh"
    
    # Create README
    cat > "$zip_dir/README.txt" << EOF
VEXY_JSON v$VERSION - Simple ZIP Installation

This is a simple ZIP package containing the vexy_json binary.

Installation:
1. Run: ./install.sh
   OR
2. Manually copy 'vexy_json' to a directory in your PATH

Usage:
  echo '{"key": "value"}' | vexy_json
  vexy_json --help

For the full installer experience, download the .dmg file instead.
EOF
    
    # Create ZIP
    cd "$zip_dir"
    zip -r "$zip_file" .
    cd "$PROJECT_ROOT"
    
    if [[ -f "$zip_file" ]]; then
        success "ZIP package created: $zip_file"
    fi
}

verify_installation() {
    log "Verifying installation components..."
    
    # Check if binary works
    if "$PROJECT_ROOT/target/release/vexy_json" --version &> /dev/null; then
        success "Binary verification: working ✓"
    else
        error "Binary verification failed"
        exit 1
    fi
    
    # Check package contents
    if pkgutil --payload-files "$BUILD_DIR/$APP_NAME.pkg" | grep -q "usr/local/bin/vexy_json"; then
        success "Package verification: contains binary ✓"
    else
        error "Package verification failed"
        exit 1
    fi
}

print_summary() {
    echo
    echo -e "${GREEN}🎉 macOS installer build completed!${NC}"
    echo
    echo -e "${BLUE}Generated files:${NC}"
    echo "  📦 DMG Installer: $DMG_NAME"
    if [[ -f "$PROJECT_ROOT/vexy_json-$VERSION-macos.zip" ]]; then
        echo "  📁 ZIP Package: vexy_json-$VERSION-macos.zip"
    fi
    echo "  🔧 PKG Installer: $BUILD_DIR/$APP_NAME.pkg"
    echo "  🔨 Universal Binary: $PROJECT_ROOT/target/release/vexy_json"
    echo
    
    echo -e "${BLUE}Installation instructions for users:${NC}"
    echo "  1. Download and open $DMG_NAME"
    echo "  2. Double-click vexy_json.pkg to install"
    echo "  3. Follow the installer prompts"
    echo "  4. Restart terminal or run 'source ~/.bashrc'"
    echo
    
    echo -e "${BLUE}Binary details:${NC}"
    lipo -info "$PROJECT_ROOT/target/release/vexy_json" | sed 's/^/  /'
    echo
    
    echo -e "${BLUE}Next steps:${NC}"
    echo "  1. Test the installer on a clean macOS system"
    echo "  2. Upload to GitHub releases"
    echo "  3. Update Homebrew formula"
    echo "  4. Test on both Intel and Apple Silicon Macs"
}

main() {
    echo -e "${BLUE}
╔══════════════════════════════════════╗
║      VEXY_JSON macOS Installer Build     ║
║              v$VERSION                 ║
╚══════════════════════════════════════╝
${NC}"
    
    # Parse command line arguments
    local skip_dmg=false
    local skip_zip=false
    
    while [[ $# -gt 0 ]]; do
        case $1 in
            --skip-dmg)
                skip_dmg=true
                shift
                ;;
            --skip-zip)
                skip_zip=true
                shift
                ;;
            -h|--help)
                echo "Usage: $0 [OPTIONS]"
                echo "Options:"
                echo "  --skip-dmg           Skip DMG creation"
                echo "  --skip-zip           Skip ZIP package creation"
                echo "  -h, --help          Show this help"
                exit 0
                ;;
            *)
                error "Unknown option: $1"
                exit 1
                ;;
        esac
    done
    
    cd "$PROJECT_ROOT"
    
    # Execute build steps
    check_prerequisites
    build_universal_binary
    create_installer_structure
    create_package
    
    if [[ "$skip_dmg" != true ]]; then
        create_dmg
    fi
    
    if [[ "$skip_zip" != true ]]; then
        create_zip_alternative
    fi
    
    verify_installation
    print_summary
    
    success "macOS installer build completed successfully!"
}

# Handle Ctrl+C gracefully
trap 'echo -e "\n${RED}Build interrupted by user${NC}"; exit 1' INT

# Run main function
main "$@"