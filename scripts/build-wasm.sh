#!/bin/bash
# this_file: build-wasm.sh

# WebAssembly Build Script for vexy_json
# Automated build script using wasm-pack with configurable dev/release modes
# Outputs to docs/pkg/ directory for web integration

set -e

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
OUTPUT_DIR="$PROJECT_ROOT/docs/pkg"
BUILD_MODE="${1:-dev}" # dev or release (using dev with optimized release profile)

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🔧 vexy_json WebAssembly Build Script${NC}"
echo "=================================================="
echo -e "Build mode: ${YELLOW}$BUILD_MODE${NC}"
echo -e "Output directory: ${YELLOW}$OUTPUT_DIR${NC}"

# Get version from git if available
if [ -f "$PROJECT_ROOT/scripts/get-version.sh" ]; then
    VERSION=$("$PROJECT_ROOT/scripts/get-version.sh")
    echo -e "Version: ${YELLOW}$VERSION${NC}"
fi
echo

# Check if wasm-pack is installed
if ! command -v wasm-pack &>/dev/null; then
    echo -e "${RED}❌ Error: wasm-pack is not installed${NC}"
    echo "Please install wasm-pack:"
    echo "  curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh"
    exit 1
fi

# Check if the wasm feature dependencies are available
echo -e "${BLUE}🔍 Checking WebAssembly dependencies...${NC}"
if ! grep -q 'wasm-bindgen' "$PROJECT_ROOT/crates/wasm/Cargo.toml"; then
    echo -e "${RED}❌ Error: WebAssembly dependencies not found in crates/wasm/Cargo.toml${NC}"
    echo "Please ensure the 'wasm' feature and dependencies are configured."
    exit 1
fi

# Create output directory if it doesn't exist
mkdir -p "$OUTPUT_DIR"

# Navigate to wasm crate directory
cd "$PROJECT_ROOT/crates/wasm"

# Set build arguments based on mode
if [ "$BUILD_MODE" = "release" ]; then
    WASM_PACK_ARGS="--target web --out-dir $OUTPUT_DIR --release"
    echo -e "${GREEN}🚀 Building WebAssembly module (release mode with size optimizations)...${NC}"
else
    # Debug mode is the default for wasm-pack (no flag needed)
    WASM_PACK_ARGS="--target web --out-dir $OUTPUT_DIR --dev"
    echo -e "${YELLOW}🔨 Building WebAssembly module (development mode)...${NC}"
fi

# Build the WebAssembly module
echo "Running: wasm-pack build $WASM_PACK_ARGS"
if wasm-pack build $WASM_PACK_ARGS; then
    echo -e "${GREEN}✅ WebAssembly build completed successfully!${NC}"
    
    # Update package.json version if we have a VERSION
    if [ -n "$VERSION" ] && [ -f "$OUTPUT_DIR/package.json" ]; then
        echo -e "${BLUE}📋 Updating package.json version to $VERSION...${NC}"
        if command -v jq &>/dev/null; then
            jq ".version = \"$VERSION\"" "$OUTPUT_DIR/package.json" > "$OUTPUT_DIR/package.json.tmp" && mv "$OUTPUT_DIR/package.json.tmp" "$OUTPUT_DIR/package.json"
        else
            sed -i.bak "s/\"version\": \"[^\"]*\"/\"version\": \"$VERSION\"/" "$OUTPUT_DIR/package.json"
            rm -f "$OUTPUT_DIR/package.json.bak"
        fi
        echo -e "${GREEN}✅ Updated package.json version${NC}"
    fi
else
    echo -e "${RED}❌ WebAssembly build failed${NC}"
    exit 1
fi

# Additional optimization with wasm-opt if available
if [ -f "$OUTPUT_DIR/vexy_json_wasm_bg.wasm" ] && command -v wasm-opt &>/dev/null; then
    echo -e "${BLUE}🔧 Optimizing WASM bundle with wasm-opt...${NC}"
    ORIGINAL_SIZE=$(stat -f%z "$OUTPUT_DIR/vexy_json_wasm_bg.wasm" 2>/dev/null || stat -c%s "$OUTPUT_DIR/vexy_json_wasm_bg.wasm" 2>/dev/null)
    wasm-opt -Oz "$OUTPUT_DIR/vexy_json_wasm_bg.wasm" -o "$OUTPUT_DIR/vexy_json_wasm_bg.wasm.opt"
    if [ -f "$OUTPUT_DIR/vexy_json_wasm_bg.wasm.opt" ]; then
        mv "$OUTPUT_DIR/vexy_json_wasm_bg.wasm.opt" "$OUTPUT_DIR/vexy_json_wasm_bg.wasm"
        OPTIMIZED_SIZE=$(stat -f%z "$OUTPUT_DIR/vexy_json_wasm_bg.wasm" 2>/dev/null || stat -c%s "$OUTPUT_DIR/vexy_json_wasm_bg.wasm" 2>/dev/null)
        REDUCTION=$((ORIGINAL_SIZE - OPTIMIZED_SIZE))
        echo -e "${GREEN}✅ Additional optimization saved ${YELLOW}$REDUCTION bytes${NC}"
    fi
fi

# Report bundle size
if [ -f "$OUTPUT_DIR/vexy_json_wasm_bg.wasm" ]; then
    WASM_SIZE=$(du -h "$OUTPUT_DIR/vexy_json_wasm_bg.wasm" | cut -f1)
    echo -e "${GREEN}📦 Final WASM bundle size: ${YELLOW}$WASM_SIZE${NC}"

    # Size warnings
    WASM_SIZE_BYTES=$(stat -f%z "$OUTPUT_DIR/vexy_json_wasm_bg.wasm" 2>/dev/null || stat -c%s "$OUTPUT_DIR/vexy_json_wasm_bg.wasm" 2>/dev/null)
    if [ "$WASM_SIZE_BYTES" -gt 1048576 ]; then # 1MB
        echo -e "${YELLOW}⚠️  Warning: WASM bundle is larger than 1MB${NC}"
        echo "   Consider optimizing for web deployment"
    elif [ "$WASM_SIZE_BYTES" -lt 512000 ]; then # 500KB
        echo -e "${GREEN}✅ Excellent! Bundle size is under 500KB${NC}"
    fi
fi

# List generated files
echo
echo -e "${BLUE}📁 Generated files in $OUTPUT_DIR:${NC}"
ls -la "$OUTPUT_DIR/" | grep -E '\.(wasm|js|ts|json)$' || echo "No WebAssembly files found"

echo
echo -e "${GREEN}🎉 WebAssembly build process completed!${NC}"
echo
echo -e "${BLUE}Next steps:${NC}"
echo "1. Test the WASM module with a simple HTML page"
echo "2. Integrate into the web interface (docs/tool.html)"
echo "3. Add error handling and user feedback"
echo "4. Test with various JSON inputs"
echo
echo -e "${BLUE}Example usage in HTML:${NC}"
echo "  <script type=\"module\">"
echo "    import init, { parse_json } from './pkg/vexy_json.js';"
echo "    await init();"
echo "    const result = parse_json('{\"test\": true}');"
echo "  </script>"
