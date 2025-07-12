# macOS Packaging Guide

This guide explains how to package vexy_json for macOS distribution as a `.dmg` containing a `.pkg` installer.

## Prerequisites

- macOS development environment
- Xcode Command Line Tools installed
- Rust toolchain installed
- Valid code signing certificate (optional, for signed packages)

## Building the Package

Run the packaging script from the project root:

```bash
./scripts/package-macos.sh
```

This script will:
1. Build the release binary using `cargo build --release`
2. Create a `.pkg` installer that installs vexy_json to `/usr/local/bin`
3. Wrap the `.pkg` in a `.dmg` for easy distribution

## Output

The script produces:
- `vexy_json-{VERSION}-macos.dmg` - The distributable disk image
- Contains the `.pkg` installer and a README

## Installation

Users can install vexy_json by:
1. Opening the `.dmg` file
2. Double-clicking the `.pkg` installer
3. Following the installation wizard
4. The `vexy_json` command will be available in their terminal

## Code Signing (Optional)

To sign the package for distribution outside the App Store:

```bash
# Sign the package
productsign --sign "Developer ID Installer: Your Name (TEAMID)" \
    unsigned.pkg signed.pkg

# Sign the DMG
codesign --sign "Developer ID Application: Your Name (TEAMID)" \
    --timestamp vexy_json-*.dmg
```

## Notarization (Recommended)

For macOS 10.15+ distribution, notarize the DMG:

```bash
# Submit for notarization
xcrun altool --notarize-app \
    --primary-bundle-id "com.twardoch.vexy_json" \
    --username "your-apple-id@example.com" \
    --password "@keychain:AC_PASSWORD" \
    --file vexy_json-*.dmg

# Staple the notarization ticket
xcrun stapler staple vexy_json-*.dmg
```

## Automation

This packaging process is automated in the GitHub Actions release workflow. See `.github/workflows/release.yml` for the CI/CD implementation.