#!/bin/bash
# this_file: scripts/build-deliverables.sh
# Build deliverables for all platforms according to issue 620
# Creates dist/{macos,windows,linux} with proper packaging

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
VERSION=$(grep '^version' "$PROJECT_ROOT/Cargo.toml" | head -1 | cut -d'"' -f2)
DIST_DIR="$PROJECT_ROOT/dist"
BINARY_NAME="vexy-json"

# Function to print messages
log() {
    echo -e "${BLUE}[$(date +'%Y-%m-%d %H:%M:%S')]${NC} $1"
}

success() {
    echo -e "${GREEN}✅ $1${NC}"
}

error() {
    echo -e "${RED}❌ $1${NC}" >&2
}

# Clean and create dist directories
prepare_dist() {
    log "Preparing dist directories..."
    rm -rf "$DIST_DIR"
    mkdir -p "$DIST_DIR"/{macos,windows,linux}
    success "Created dist directories"
}

# Build for macOS and create DMG
build_macos() {
    log "Building macOS deliverables..."
    
    local MACOS_DIR="$DIST_DIR/macos"
    
    # Build native binary for current macOS architecture
    cargo build --release -p vexy-json-cli --bin "$BINARY_NAME"
    
    # Copy binary
    cp "target/release/$BINARY_NAME" "$MACOS_DIR/"
    
    # Use existing package-macos.sh script to create DMG
    if [[ -x "$SCRIPT_DIR/package-macos.sh" ]]; then
        log "Creating macOS DMG package..."
        cd "$PROJECT_ROOT"
        "$SCRIPT_DIR/package-macos.sh"
        
        # Move DMG to dist/macos
        if [[ -f "$MACOS_DIR/${BINARY_NAME}-${VERSION}-macos.dmg" ]]; then
            success "Created macOS DMG: $MACOS_DIR/${BINARY_NAME}-${VERSION}-macos.dmg"
        fi
    else
        error "package-macos.sh script not found"
    fi
    
    # Also create a simple tarball of the binary
    cd "$MACOS_DIR"
    tar -czf "${BINARY_NAME}-${VERSION}-macos.tar.gz" "$BINARY_NAME"
    success "Created macOS tarball: ${BINARY_NAME}-${VERSION}-macos.tar.gz"
    cd "$PROJECT_ROOT"
}

# Build for Windows and create ZIP
build_windows() {
    log "Building Windows deliverables..."
    
    local WINDOWS_DIR="$DIST_DIR/windows"
    
    # Check if we can cross-compile to Windows
    if command -v cross &> /dev/null; then
        log "Cross-compiling for Windows..."
        cross build --release -p vexy-json-cli --bin "$BINARY_NAME" --target x86_64-pc-windows-msvc
        cp "target/x86_64-pc-windows-msvc/release/${BINARY_NAME}.exe" "$WINDOWS_DIR/"
    else
        # Check if we have the Windows target installed
        if rustup target list --installed | grep -q "x86_64-pc-windows-gnu"; then
            log "Building for Windows using cargo..."
            cargo build --release -p vexy-json-cli --bin "$BINARY_NAME" --target x86_64-pc-windows-gnu
            cp "target/x86_64-pc-windows-gnu/release/${BINARY_NAME}.exe" "$WINDOWS_DIR/"
        else
            error "Cannot build for Windows. Install cross or add Windows target."
            return 1
        fi
    fi
    
    # Create ZIP
    cd "$WINDOWS_DIR"
    zip "${BINARY_NAME}-${VERSION}-windows.zip" "${BINARY_NAME}.exe"
    success "Created Windows ZIP: ${BINARY_NAME}-${VERSION}-windows.zip"
    cd "$PROJECT_ROOT"
}

# Build for Linux and create TGZ
build_linux() {
    log "Building Linux deliverables..."
    
    local LINUX_DIR="$DIST_DIR/linux"
    
    # Build static binary using musl if possible
    if rustup target list --installed | grep -q "x86_64-unknown-linux-musl"; then
        log "Building static Linux binary with musl..."
        cargo build --release -p vexy-json-cli --bin "$BINARY_NAME" --target x86_64-unknown-linux-musl
        cp "target/x86_64-unknown-linux-musl/release/$BINARY_NAME" "$LINUX_DIR/"
    else
        log "Building Linux binary..."
        cargo build --release -p vexy-json-cli --bin "$BINARY_NAME"
        cp "target/release/$BINARY_NAME" "$LINUX_DIR/"
    fi
    
    # Strip the binary
    if command -v strip &> /dev/null; then
        strip "$LINUX_DIR/$BINARY_NAME"
    fi
    
    # Create TGZ
    cd "$LINUX_DIR"
    tar -czf "${BINARY_NAME}-${VERSION}-linux.tar.gz" "$BINARY_NAME"
    success "Created Linux TGZ: ${BINARY_NAME}-${VERSION}-linux.tar.gz"
    cd "$PROJECT_ROOT"
}

# Generate checksums for all deliverables
generate_checksums() {
    log "Generating checksums..."
    
    for platform in macos windows linux; do
        if [[ -d "$DIST_DIR/$platform" ]]; then
            cd "$DIST_DIR/$platform"
            
            # Generate SHA256 checksums
            if command -v sha256sum &> /dev/null; then
                sha256sum * > checksums.sha256
            elif command -v shasum &> /dev/null; then
                shasum -a 256 * > checksums.sha256
            fi
            
            success "Generated checksums for $platform"
        fi
    done
    
    cd "$PROJECT_ROOT"
}

# Create a README for dist directory
create_dist_readme() {
    cat > "$DIST_DIR/README.md" << EOF
# Vexy JSON v${VERSION} - Distribution Files

This directory contains pre-built binaries and installers for Vexy JSON.

## Directory Structure

- \`macos/\` - macOS builds
  - \`${BINARY_NAME}-${VERSION}-macos.dmg\` - DMG installer that installs to /usr/local/bin
  - \`${BINARY_NAME}-${VERSION}-macos.tar.gz\` - Standalone binary tarball
  - \`${BINARY_NAME}\` - Raw binary
  
- \`windows/\` - Windows builds  
  - \`${BINARY_NAME}-${VERSION}-windows.zip\` - ZIP containing the executable
  - \`${BINARY_NAME}.exe\` - Raw executable
  
- \`linux/\` - Linux builds
  - \`${BINARY_NAME}-${VERSION}-linux.tar.gz\` - Standalone binary tarball
  - \`${BINARY_NAME}\` - Raw binary (statically linked if built with musl)

## Installation

### macOS
1. Download the .dmg file
2. Open it and run the installer
3. The \`vexy-json\` command will be available in your terminal

### Windows
1. Download the .zip file
2. Extract it to a directory in your PATH
3. Run \`vexy-json.exe\` from the command prompt

### Linux
1. Download the .tar.gz file
2. Extract it: \`tar -xzf vexy_json-${VERSION}-linux.tar.gz\`
3. Move the binary to a directory in your PATH: \`sudo mv vexy-json /usr/local/bin/\`
4. Make it executable: \`chmod +x /usr/local/bin/vexy-json\`

## Verification

Each platform directory contains a \`checksums.sha256\` file. Verify your download:

\`\`\`bash
# macOS/Linux
shasum -a 256 -c checksums.sha256

# Or if sha256sum is available
sha256sum -c checksums.sha256
\`\`\`

## Usage

\`\`\`bash
# Parse JSON from stdin
echo '{"key": "value"}' | vexy-json

# Parse JSON file
vexy-json < data.json

# Pretty print JSON
echo '{"compact":true}' | vexy-json
\`\`\`

For more information: https://github.com/vexyart/vexy-json
EOF
    
    success "Created dist/README.md"
}

# Main build process
main() {
    echo -e "${BLUE}
╔══════════════════════════════════════╗
║     VEXY JSON Build Deliverables     ║
║              v${VERSION}             ║
╚══════════════════════════════════════╝
${NC}"
    
    # Check if we're on macOS
    if [[ "$(uname)" != "Darwin" ]]; then
        echo -e "${YELLOW}⚠️  Warning: This script works best on macOS.${NC}"
        echo -e "${YELLOW}   Windows and Linux builds may require cross-compilation tools.${NC}"
    fi
    
    # Prepare dist directory
    prepare_dist
    
    # Build for each platform
    echo
    build_macos
    
    echo
    build_windows || echo -e "${YELLOW}⚠️  Windows build failed or skipped${NC}"
    
    echo
    build_linux || echo -e "${YELLOW}⚠️  Linux build failed or skipped${NC}"
    
    # Generate checksums
    echo
    generate_checksums
    
    # Create README
    create_dist_readme
    
    # Summary
    echo
    echo -e "${GREEN}🎉 Build deliverables completed!${NC}"
    echo
    echo -e "${BLUE}Distribution files created in: $DIST_DIR${NC}"
    echo
    
    # List created files
    for platform in macos windows linux; do
        if [[ -d "$DIST_DIR/$platform" ]]; then
            echo -e "${BLUE}$platform:${NC}"
            ls -la "$DIST_DIR/$platform" | grep -v "^total" | grep -v "^d"
            echo
        fi
    done
    
    echo -e "${BLUE}Next steps:${NC}"
    echo "  1. Test the binaries on their respective platforms"
    echo "  2. Upload to GitHub releases"
    echo "  3. Update the release notes"
}

# Run main
main "$@"