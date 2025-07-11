# Homebrew Formula for vexy_json

This directory contains the Homebrew formula for installing vexy_json on macOS.

## Installation

To install vexy_json using this formula:

```bash
# Add this tap (once the formula is in a tap repository)
brew tap twardoch/vexy_json

# Install vexy_json
brew install vexy_json
```

Or install directly from the formula file:

```bash
brew install ./Formula/vexy_json.rb
```

## Testing the Formula

To test the formula locally:

```bash
brew install --build-from-source ./Formula/vexy_json.rb
brew test vexy_json
brew audit --strict vexy_json
```

## Updating the Formula

When releasing a new version:

1. Update the `url` to point to the new release tag
2. Update the SHA256 checksum:
   ```bash
   curl -sL https://github.com/twardoch/vexy_json/archive/refs/tags/vX.Y.Z.tar.gz | shasum -a 256
   ```
3. Test the formula thoroughly
4. Submit to Homebrew or update your tap

## Formula Details

- **Dependencies**: Only requires Rust for building (no runtime dependencies)
- **Build**: Uses cargo to build from source
- **Tests**: Includes comprehensive tests for JSON parsing, forgiving features, and error repair