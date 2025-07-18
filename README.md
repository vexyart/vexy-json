# Vexy JSON Documentation & Web Tool

This directory contains the documentation website and interactive web tool for Vexy JSON.

## Recent Updates

### Version 1.2.4 - Critical WebAssembly Fix

Fixed a major bug where WebAssembly bindings returned JavaScript Maps instead of plain objects for parsed JSON. Objects like `{a:1}` now correctly return `{"a":1}` instead of empty objects. See [Troubleshooting](troubleshooting.md) for details.

## Development Quick Start

### Building and Testing
```bash
# Complete build with all tests
./build.sh

# Quick development checks
./scripts/dev-workflow.sh check

# Create a new release
./scripts/release.sh --version 2.0.8
```

### Available Commands
- `./build.sh` - Complete build and test
- `./build.sh test` - Run comprehensive tests
- `./build.sh version` - Show version information
- `./scripts/dev-workflow.sh setup` - Set up development environment
- `./scripts/release.sh --version X.Y.Z` - Create a release

For complete development and release documentation, see [RELEASE_GUIDE.md](RELEASE_GUIDE.md).

## Structure

- **Jekyll Site**: The main documentation is built with Jekyll using the `just-the-docs` theme
- **Web Tool**: Interactive JSON parser tool at `/tool.html`
- **WASM Package**: Pre-built WebAssembly module in `/pkg/`
- **Debug Tools**: Various test pages for debugging WebAssembly issues

## Hosting Configuration

### GitHub Pages

The site is automatically deployed to GitHub Pages via the `.github/workflows/pages.yml` workflow:

1. **Build Process**: 
   - Builds WASM module using `wasm-pack`
   - Builds Jekyll site with proper asset inclusion
   - Deploys to GitHub Pages

2. **MIME Type Handling**:
   - `_headers`: Netlify-style headers (for potential future migration)
   - `.htaccess`: Apache-style configuration for WASM files
   - Jekyll includes both files for maximum compatibility

3. **Asset Management**:
   - WASM files are included via Jekyll's `include` directive
   - Proper caching headers set for static assets
   - CORS enabled for WebAssembly files

### Local Development

To run locally:

```bash
# Install dependencies
bundle install

# Serve Jekyll site
bundle exec jekyll serve

# Or serve with drafts and live reload
bundle exec jekyll serve --drafts --livereload
```

## Web Tool Features

The interactive tool (`/tool.html`) provides:

- **Real-time parsing** with debounced input
- **Syntax highlighting** for JSON input
- **Error highlighting** with position indicators
- **Example library** showcasing Vexy JSON features
- **Download functionality** for parsed results
- **Share URLs** for collaboration
- **Performance metrics** display

## Browser Compatibility

- **Modern Browsers**: Chrome 57+, Firefox 52+, Safari 11+, Edge 16+
- **WebAssembly**: Required for parser functionality
- **Fallback**: Graceful degradation when WASM unavailable

## Security

- **Content Security Policy**: Configured for WASM execution
- **CORS Headers**: Properly configured for cross-origin requests
- **HTTPS**: Required for some WASM features (served via GitHub Pages)
