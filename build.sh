#!/bin/bash
# Master build script for vexy_json project
# This script runs all build processes in the correct order

set -e

llms . "llms*.txt,*.d,*.json,*.html,*.css"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🚀 vexy_json Master Build Script${NC}"
echo "=============================================="
echo

# Make sure we're in the project root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# Get version from git tag
VERSION=$(./scripts/get-version.sh)
echo -e "${BLUE}Building version: ${VERSION}${NC}"
echo

# Update version numbers if we have a git tag
if git describe --exact-match --tags >/dev/null 2>&1; then
    echo -e "${BLUE}📋 Updating version numbers from git tag...${NC}"
    ./scripts/update-versions.sh
    echo
fi

# Check if required scripts exist
if [ ! -f "scripts/build.sh" ]; then
    echo -e "${RED}❌ Error: scripts/build.sh not found${NC}"
    exit 1
fi

if [ ! -f "scripts/build-wasm.sh" ]; then
    echo -e "${RED}❌ Error: scripts/build-wasm.sh not found${NC}"
    exit 1
fi

if [ ! -f "scripts/package-macos.sh" ]; then
    echo -e "${RED}❌ Error: scripts/package-macos.sh not found${NC}"
    exit 1
fi

# Step 1: Run main build script
echo -e "${BLUE}📋 Step 1: Running main build process...${NC}"
if ./scripts/build.sh; then
    echo -e "${GREEN}✅ Main build completed successfully${NC}"
else
    echo -e "${RED}❌ Main build failed${NC}"
    exit 1
fi

echo

# Step 2: Build WebAssembly module
echo -e "${BLUE}🕸️  Step 2: Building WebAssembly module...${NC}"
if ./scripts/build-wasm.sh release; then
    echo -e "${GREEN}✅ WebAssembly build completed successfully${NC}"
else
    echo -e "${RED}❌ WebAssembly build failed${NC}"
    exit 1
fi

echo

# Step 3: Package for macOS (only if on macOS)
if [[ "$OSTYPE" == "darwin"* ]]; then
    echo -e "${BLUE}📦 Step 3: Creating macOS package...${NC}"
    if ./scripts/package-macos.sh; then
        echo -e "${GREEN}✅ macOS packaging completed successfully${NC}"
    else
        echo -e "${RED}❌ macOS packaging failed${NC}"
        exit 1
    fi
else
    echo -e "${YELLOW}⚠️  Step 3: Skipping macOS packaging (not on macOS)${NC}"
fi

echo
echo -e "${GREEN}🎉 All build steps completed successfully!${NC}"
echo
echo -e "${BLUE}Build artifacts:${NC}"
echo "  • Rust library: target/release/libvexy_json.rlib"
echo "  • CLI binary: target/release/vexy_json"
echo "  • WebAssembly: docs/pkg/vexy_json_wasm_bg.wasm"
echo "  • Documentation: target/doc/vexy_json/index.html"

if [[ "$OSTYPE" == "darwin"* ]]; then
    echo "  • macOS installer: vexy_json-${VERSION}-macos.dmg"
fi

echo
echo -e "${BLUE}Next steps:${NC}"
echo "  1. Test the built artifacts"
echo "  2. Run integration tests"
echo "  3. Deploy to target environments"
