# Vexy JSON Release Process

This document describes the complete release process for Vexy JSON v2.0.0 and future versions.

## Overview

The Vexy JSON release process is fully automated using GitHub Actions. When you push a version tag (e.g., `v2.0.0`), the following happens automatically:

1. **CI/CD Pipeline** runs all tests on multiple platforms
2. **Release Workflow** creates binaries for all platforms
3. **Installers** are built (macOS DMG with PKG)
4. **WASM modules** are compiled and packaged
5. **GitHub Release** is created with all artifacts
6. **Publishing** to crates.io and npm
7. **Documentation** is updated on GitHub Pages

## Prerequisites

Before releasing, ensure you have:

- [ ] GitHub CLI (`gh`) installed and authenticated
- [ ] Rust toolchain installed
- [ ] Write access to the repository
- [ ] API tokens configured (see below)

## Required Secrets

Configure these secrets in your GitHub repository settings:

- `CARGO_REGISTRY_TOKEN` - For publishing to crates.io
- `NPM_TOKEN` - For publishing to npm
- `HOMEBREW_GITHUB_TOKEN` - For updating Homebrew formula (optional)

## Release Steps

### 1. Pre-Release Checklist

Run the pre-release check script:

```bash
./scripts/pre-release-check.sh
```

This validates:
- Version numbers are consistent
- Documentation is updated
- GitHub Actions workflows exist
- Code builds successfully
- Working directory is clean

### 2. Quick Release (Recommended)

For a standard release, use the GitHub release script:

```bash
./scripts/release-github.sh --version 2.0.0
```

This script will:
- Run pre-release checks
- Execute tests
- Create and push the git tag
- Monitor the GitHub Actions workflow

### 3. Manual Release

If you prefer manual control:

```bash
# 1. Run tests
cargo test --all-features

# 2. Create tag
git tag -a v2.0.0 -m "Release v2.0.0"

# 3. Push tag
git push origin v2.0.0

# 4. Monitor GitHub Actions
gh run watch
```

### 4. Alternative: Trigger via GitHub UI

You can also trigger a release from the GitHub Actions tab:

1. Go to Actions → Release workflow
2. Click "Run workflow"
3. Enter the version (e.g., "2.0.0")
4. Click "Run workflow"

## Release Artifacts

The automated release creates:

### Binaries
- **macOS**: Universal binary (x86_64 + ARM64)
  - `vexy_json-2.0.0-macos.zip` - Standalone binary
  - `vexy_json-2.0.0-macos.dmg` - Installer with PKG
- **Linux**: 
  - `vexy_json-2.0.0-linux-x86_64.tar.gz` - x86_64 binary
  - `vexy_json-2.0.0-linux-aarch64.tar.gz` - ARM64 binary
- **Windows**:
  - `vexy_json-2.0.0-windows-x86_64.zip` - x86_64 binary

### WASM Package
- `vexy_json-wasm-2.0.0.tar.gz` - WebAssembly module with TypeScript bindings

### Source
- Source code archives (automatically created by GitHub)

## Platform-Specific Details

### macOS Installer

The macOS installer includes:
- Universal binary supporting Intel and Apple Silicon
- PKG installer that places `vexy_json` in `/usr/local/bin`
- Code-signed DMG (requires Apple Developer certificate)
- Automatic PATH configuration

### Linux Packages

Future releases will include:
- `.deb` packages for Debian/Ubuntu
- `.rpm` packages for Fedora/RHEL
- AppImage for universal Linux support

### Windows Installer

Future releases will include:
- MSI installer with PATH configuration
- Chocolatey package

## Post-Release

After the release is published:

1. **Verify Installation Methods**:
   ```bash
   # Homebrew (macOS)
   brew update && brew install vexy_json
   
   # Cargo
   cargo install vexy_json-cli
   
   # npm (WASM)
   npm install vexy_json-wasm
   ```

2. **Update Documentation**:
   - The docs site auto-updates via GitHub Pages
   - Verify at: https://twardoch.github.io/vexy_json/

3. **Announce Release**:
   - GitHub Discussions
   - Twitter/Social Media
   - Rust Forums
   - Reddit (r/rust)

## Troubleshooting

### Release Workflow Fails

1. Check GitHub Actions logs
2. Common issues:
   - Missing secrets (CARGO_REGISTRY_TOKEN, etc.)
   - Version already published
   - Test failures on specific platforms

### Tag Already Exists

```bash
# Delete local tag
git tag -d v2.0.0

# Delete remote tag
git push origin :refs/tags/v2.0.0

# Recreate tag
git tag -a v2.0.0 -m "Release v2.0.0"
git push origin v2.0.0
```

### Partial Release

If some artifacts fail:
1. Fix the issue
2. Re-run failed jobs in GitHub Actions
3. The release will update automatically

## Version Numbering

Vexy JSON follows Semantic Versioning:

- **Major** (X.0.0): Breaking API changes
- **Minor** (0.X.0): New features, backward compatible
- **Patch** (0.0.X): Bug fixes

## Release Frequency

- **Major releases**: Annually or as needed
- **Minor releases**: Quarterly
- **Patch releases**: As needed for critical fixes

## Security Releases

For security fixes:
1. Follow responsible disclosure
2. Prepare fix in private
3. Release with security advisory
4. Backport to supported versions

## Appendix: Local Testing

To test the release process locally:

```bash
# Dry run of release script
./scripts/release-github.sh --version 2.0.0 --dry-run

# Test build scripts
./build.sh --all

# Test packaging
./scripts/package-macos.sh 2.0.0
```

## Support

For release issues:
- Open an issue on GitHub
- Contact maintainers
- Check GitHub Actions documentation