# Release Process

This document describes the automated release process for vexy_json.

## Overview

Releases are automatically triggered when a version tag is pushed to the repository. The tag must follow the format `v*.*.*` (e.g., `v1.2.0`).

## Prerequisites

Before creating a release, ensure:

1. **Version Updated**: Update the version in `Cargo.toml`
2. **Changelog Updated**: Add release notes to `CHANGELOG.md`
3. **Tests Pass**: Run `./build.sh` and ensure all tests pass
4. **Documentation Updated**: Update any relevant documentation

## GitHub Secrets Required

The following secrets must be configured in the repository settings:

- `CRATES_IO_TOKEN`: API token for publishing to crates.io
- `NPM_TOKEN`: API token for publishing to npm (optional)

## Creating a Release

1. **Update Version**:
   ```bash
   # Edit Cargo.toml and update the version field
   version = "1.2.0"
   ```

2. **Update Changelog**:
   ```bash
   # Add a new section to CHANGELOG.md
   ## [1.2.0] - 2025-01-XX
   - Feature: Added new functionality
   - Fix: Resolved issue with...
   ```

3. **Commit Changes**:
   ```bash
   git add Cargo.toml CHANGELOG.md
   git commit -m "chore: bump version to 1.2.0"
   git push
   ```

4. **Create and Push Tag**:
   ```bash
   git tag v1.2.0
   git push origin v1.2.0
   ```

## Automated Release Workflow

Once the tag is pushed, the GitHub Actions workflow will:

1. **Create GitHub Release**: Generate release notes from commits
2. **Build Binaries**: Compile for multiple platforms:
   - Linux (x86_64, aarch64) with musl for static linking
   - macOS (x86_64, aarch64)
   - Windows (x86_64, i686)
3. **Build macOS Package**: Create .dmg with .pkg installer
4. **Build WebAssembly**: Package WASM module and bindings
5. **Publish to crates.io**: Automatically publish the Rust crate
6. **Publish to npm**: Publish WASM package (if configured)
7. **Generate Checksums**: Create SHA256 checksums for all artifacts
8. **Update Documentation**: Deploy updated docs to GitHub Pages

## Release Assets

Each release includes:

- **Binary executables** for all supported platforms
- **macOS installer** (.dmg containing .pkg)
- **WebAssembly module** (tar.gz archive)
- **SHA256 checksums** for all files
- **Source code** archives (zip and tar.gz)

## Manual Release Steps

If automatic publishing fails:

### Publish to crates.io
```bash
cargo login <YOUR_API_TOKEN>
cargo publish
```

### Publish to npm
```bash
cd npm-pkg
npm login
npm publish --access public
```

## Rollback Process

If a release needs to be rolled back:

1. Delete the release from GitHub
2. Yank the version from crates.io: `cargo yank --version 1.2.0`
3. Unpublish from npm (within 72 hours): `npm unpublish @vexy_json/vexy_json@1.2.0`
4. Delete the git tag: `git push --delete origin v1.2.0`

## Troubleshooting

### Build Failures
- Check the GitHub Actions logs for specific errors
- Ensure all dependencies are properly specified
- Verify cross-compilation targets are correctly configured

### Publishing Failures
- Verify API tokens are correctly set in GitHub Secrets
- Check that the version doesn't already exist on the registry
- Ensure package metadata is complete and valid

### macOS Package Issues
- Verify the packaging script has executable permissions
- Check that the build completes successfully locally
- Ensure pkgbuild and productbuild tools are available

## Security Considerations

- Never commit API tokens to the repository
- Use GitHub Secrets for all sensitive credentials
- Consider signing binaries for production releases
- Enable 2FA on crates.io and npm accounts