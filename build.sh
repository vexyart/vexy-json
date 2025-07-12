#!/bin/bash
# Master build script for vexy_json project
# Usage: ./build.sh [command]
# Commands:
#   llms     - Generate llms.txt file
#   clean    - Clean all build artifacts
#   debug    - Build in debug mode
#   release  - Build in release mode
#   install  - Install CLI to /usr/local/bin (macOS)
#   wasm     - Build WebAssembly module
#   (none)   - Run all build steps

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
NC='\033[0m' # No Color

# Make sure we're in the project root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# Function to print usage
usage() {
    echo -e "${BLUE}🚀 Vexy JSON Build Script${NC}"
    echo "=============================================="
    echo
    echo "Usage: $0 [command]"
    echo
    echo "Commands:"
    echo "  llms         - Generate llms.txt file for AI context"
    echo "  clean        - Clean all build artifacts"
    echo "  debug        - Build in debug mode"
    echo "  release      - Build in release mode"
    echo "  install      - Install CLI to /usr/local/bin (macOS)"
    echo "  wasm         - Build WebAssembly module"
    echo "  deliverables - Build distribution packages for all platforms"
    echo "  help         - Show this help message"
    echo "  (none)       - Run all build steps (equivalent to 'all')"
    echo
}

# Function to generate llms.txt
build_llms() {
    echo -e "${BLUE}📝 Generating llms.txt...${NC}"
    llms . "llms*.txt,*.d,*.json,*.html,*.svg,.specstory,ref,testdata,*.lock,*.svg,*.css,*.txt"
    echo -e "${GREEN}✅ llms.txt generated successfully${NC}"
}

# Function to clean build artifacts
build_clean() {
    echo -e "${BLUE}🧹 Cleaning build artifacts...${NC}"
    cargo clean
    rm -rf docs/pkg
    rm -rf dist
    rm -f build.log.txt
    rm -f llms.txt
    echo -e "${GREEN}✅ Clean completed${NC}"
}

# Function to build in debug mode
build_debug() {
    echo -e "${BLUE}🔨 Building in debug mode...${NC}"
    cargo build
    cargo test
    echo -e "${GREEN}✅ Debug build completed${NC}"
}

# Function to build in release mode
build_release() {
    echo -e "${BLUE}🚀 Building in release mode...${NC}"

    # Get version
    VERSION=$(./scripts/get-version.sh 2>/dev/null || echo "dev")
    echo -e "${BLUE}Building version: ${VERSION}${NC}"

    # Update version numbers
    echo -e "${BLUE}📋 Updating version numbers...${NC}"
    ./scripts/update-versions.sh

    # Build release
    echo -e "${BLUE}📦 Building release binaries...${NC}"
    cargo build --release

    # Run tests
    echo -e "${BLUE}🧪 Running tests...${NC}"
    cargo test --release

    # Build documentation
    echo -e "${BLUE}📚 Building documentation...${NC}"
    cargo doc --no-deps

    echo -e "${GREEN}✅ Release build completed${NC}"
}

# Function to install CLI
build_install() {
    if [[ "$OSTYPE" != "darwin"* ]]; then
        echo -e "${RED}❌ Install command is currently only supported on macOS${NC}"
        exit 1
    fi

    echo -e "${BLUE}📥 Installing Vexy JSON CLI...${NC}"

    # Build release if not already built
    if [ ! -f "target/release/vexy-json" ]; then
        echo -e "${YELLOW}⚠️  Release binary not found, building...${NC}"
        build_release
    fi

    # Copy to /usr/local/bin
    echo -e "${BLUE}Installing to /usr/local/bin...${NC}"
    sudo cp target/release/vexy_json /usr/local/bin/
    sudo chmod +x /usr/local/bin/vexy_json

    # Verify installation
    if command -v vexy_json &>/dev/null; then
        echo -e "${GREEN}✅ Vexy JSON CLI installed successfully${NC}"
        echo -e "${BLUE}Version: $(vexy_json --version)${NC}"
    else
        echo -e "${RED}❌ Installation verification failed${NC}"
        exit 1
    fi
}

# Function to build WASM
build_wasm() {
    echo -e "${BLUE}🕸️  Building WebAssembly module...${NC}"

    if [ ! -f "scripts/build-wasm.sh" ]; then
        echo -e "${RED}❌ Error: scripts/build-wasm.sh not found${NC}"
        exit 1
    fi

    ./scripts/build-wasm.sh release
    echo -e "${GREEN}✅ WebAssembly build completed${NC}"
}

# Function to run all build steps
build_all() {
    echo -e "${PURPLE}🚀 Running all build steps...${NC}"
    echo "=============================================="
    echo

    # Generate llms.txt
    build_llms
    echo

    # Build release
    build_release
    echo

    # Build WASM
    build_wasm
    echo

    # Package for macOS (only if on macOS)
    if [[ "$OSTYPE" == "darwin"* ]]; then
        echo -e "${BLUE}📦 Creating macOS package...${NC}"
        if [ -f "scripts/package-macos.sh" ]; then
            ./scripts/package-macos.sh
            echo -e "${GREEN}✅ macOS packaging completed${NC}"
        else
            echo -e "${YELLOW}⚠️  macOS packaging script not found${NC}"
        fi
    fi

    echo
    echo -e "${GREEN}🎉 All build steps completed successfully!${NC}"
    echo
    echo -e "${BLUE}Build artifacts:${NC}"
    echo "  • Rust library: target/release/libvexy_json.rlib"
    echo "  • CLI binary: target/release/vexy_json"
    echo "  • WebAssembly: docs/pkg/vexy_json_wasm_bg.wasm"
    echo "  • Documentation: target/doc/vexy_json/index.html"

    VERSION=$(./scripts/get-version.sh 2>/dev/null || echo "dev")
    if [[ "$OSTYPE" == "darwin"* ]] && [ -f "vexy_json-${VERSION}-macos.dmg" ]; then
        echo "  • macOS installer: vexy_json-${VERSION}-macos.dmg"
    fi
}

# Main script logic
case "${1:-all}" in
llms)
    build_llms
    ;;
clean)
    build_clean
    ;;
debug)
    build_debug
    ;;
release)
    build_release
    ;;
install)
    build_install
    ;;
wasm)
    build_wasm
    ;;
deliverables)
    "$SCRIPT_DIR/scripts/build-deliverables.sh"
    ;;
help | --help | -h)
    usage
    ;;
all | "")
    build_all
    ;;
*)
    echo -e "${RED}❌ Unknown command: $1${NC}"
    echo
    usage
    exit 1
    ;;
esac
