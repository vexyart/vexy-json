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
    echo "  test         - Run comprehensive test suite"
    echo "  test-version - Test version management system"
    echo "  release      - Build in release mode (includes testing)"
    echo "  install      - Install CLI to /usr/local/bin (macOS)"
    echo "  wasm         - Build WebAssembly module"
    echo "  deliverables - Build distribution packages for all platforms"
    echo "  validate     - Validate project setup and dependencies"
    echo "  version      - Show current version information"
    echo "  help         - Show this help message"
    echo "  (none)       - Run all build steps (equivalent to 'all')"
    echo
}

# Function to generate llms.txt
build_llms() {
    echo -e "${BLUE}📝 Generating llms.txt...${NC}"
    
    # Check if llms command is available
    if ! command -v llms &> /dev/null; then
        echo -e "${YELLOW}⚠️  llms command not found, trying alternative method...${NC}"
        
        # Try using codetoprompt if available
        if command -v codetoprompt &> /dev/null; then
            echo -e "${BLUE}Using codetoprompt as alternative...${NC}"
            codetoprompt --compress --output "llms.txt" --respect-gitignore --cxml --exclude "*.svg,.specstory,*.md,*.txt,ref,testdata,*.lock,*.svg,*.css,*.txt" .
        else
            echo -e "${YELLOW}⚠️  Neither llms nor codetoprompt found, skipping llms.txt generation${NC}"
            echo -e "${YELLOW}    Install llms with: pip install llms${NC}"
            echo -e "${YELLOW}    Or codetoprompt with: cargo install codetoprompt${NC}"
            return 0
        fi
    else
        llms . "llms*.txt,*.d,*.json,*.html,*.svg,.specstory,ref,testdata,*.lock,*.svg,*.css,*.txt"
    fi
    
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

# Function to validate project setup
validate_project() {
    echo -e "${BLUE}🔍 Validating project setup...${NC}"
    
    # Check if we're in a git repository
    if ! git rev-parse --git-dir > /dev/null 2>&1; then
        echo -e "${RED}❌ Not in a git repository${NC}"
        return 1
    fi
    
    # Check if required scripts exist
    for script in "scripts/get-version.sh" "scripts/update-versions.sh"; do
        if [ ! -f "$script" ]; then
            echo -e "${RED}❌ Missing required script: $script${NC}"
            return 1
        fi
        
        # Make scripts executable
        chmod +x "$script"
    done
    
    # Check Rust installation
    if ! command -v cargo &> /dev/null; then
        echo -e "${RED}❌ Rust/Cargo not found${NC}"
        return 1
    fi
    
    echo -e "${GREEN}✅ Project validation completed${NC}"
    return 0
}

# Function to run comprehensive tests
run_tests() {
    echo -e "${BLUE}🧪 Running comprehensive test suite...${NC}"
    
    # Run workspace tests
    echo -e "${BLUE}Running workspace tests...${NC}"
    cargo test --workspace --all-features
    
    # Run doc tests
    echo -e "${BLUE}Running documentation tests...${NC}"
    cargo test --doc --workspace --all-features
    
    # Run example tests
    echo -e "${BLUE}Running example tests...${NC}"
    cargo test --examples
    
    # Build examples to ensure they compile
    echo -e "${BLUE}Building examples...${NC}"
    cargo build --examples
    
    echo -e "${GREEN}✅ All tests completed${NC}"
}

# Function to build in release mode
build_release() {
    echo -e "${BLUE}🚀 Building in release mode...${NC}"

    # Validate project first
    if ! validate_project; then
        echo -e "${RED}❌ Project validation failed${NC}"
        return 1
    fi

    # Get version
    VERSION=$(./scripts/get-version.sh 2>/dev/null || echo "dev")
    echo -e "${BLUE}Building version: ${VERSION}${NC}"

    # Update version numbers
    echo -e "${BLUE}📋 Updating version numbers...${NC}"
    ./scripts/update-versions.sh

    # Run comprehensive tests
    run_tests
    
    # Test version management system
    echo -e "${BLUE}🧪 Testing version management system...${NC}"
    ./scripts/test-version-system.sh

    # Build release
    echo -e "${BLUE}📦 Building release binaries...${NC}"
    cargo build --release

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

# Function to show version information
show_version() {
    echo -e "${BLUE}📊 Version Information${NC}"
    echo "=============================================="
    echo
    
    # Get version from script
    VERSION=$(./scripts/get-version.sh 2>/dev/null || echo "unknown")
    echo -e "${BLUE}Current version: ${GREEN}${VERSION}${NC}"
    
    # Show git information
    if git rev-parse --git-dir > /dev/null 2>&1; then
        echo -e "${BLUE}Git commit: ${GREEN}$(git rev-parse --short HEAD)${NC}"
        echo -e "${BLUE}Git branch: ${GREEN}$(git rev-parse --abbrev-ref HEAD)${NC}"
        
        # Show tags
        if git describe --tags > /dev/null 2>&1; then
            echo -e "${BLUE}Git describe: ${GREEN}$(git describe --tags)${NC}"
        fi
    fi
    
    # Show Rust version
    if command -v cargo &> /dev/null; then
        echo -e "${BLUE}Rust version: ${GREEN}$(rustc --version)${NC}"
    fi
    
    echo
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
test)
    if ! validate_project; then
        echo -e "${RED}❌ Project validation failed${NC}"
        exit 1
    fi
    run_tests
    ;;
test-version)
    if ! validate_project; then
        echo -e "${RED}❌ Project validation failed${NC}"
        exit 1
    fi
    echo -e "${BLUE}🧪 Running version management system tests...${NC}"
    ./scripts/test-version-system.sh
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
validate)
    validate_project
    ;;
version)
    show_version
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
