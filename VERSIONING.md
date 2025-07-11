# Git Tag-Based Versioning for Vexy JSON

This document describes how Vexy JSON implements automatic versioning based on git tags.

## Overview

Vexy JSON uses git tags as the single source of truth for version numbers. When you create a git tag like `v2.0.7`, all components automatically inherit that version during build and release.

## How It Works

### 1. Version Detection Script

The `scripts/get-version.sh` script determines the current version by:
- First checking for an exact git tag on the current commit
- Falling back to the most recent tag with `-dev` suffix if not on a tagged commit
- Using Cargo.toml version as a last resort

```bash
# Get current version
./scripts/get-version.sh
# Output: 2.0.7 (if on tag v2.0.7)
# Output: 2.0.7-dev (if commits after tag v2.0.7)
```

### 2. Version Update Script

The `scripts/update-versions.sh` script updates all version references:
- All Cargo.toml files
- Python package configuration
- JavaScript/WASM package.json files
- Homebrew formula (for releases only)

```bash
# Update all versions to match git tag
./scripts/update-versions.sh
```

### 3. Build-Time Version Injection

Each Rust crate has a `build.rs` that:
- Detects the version from git at compile time
- Sets `VEXY_JSON_VERSION` environment variable
- Falls back to `CARGO_PKG_VERSION` if git is unavailable

This allows the CLI and libraries to display the correct version:
```rust
// In code
env!("VEXY_JSON_VERSION", env!("CARGO_PKG_VERSION"))
```

### 4. Automated Updates

The build and release scripts automatically update versions:

#### During Development
```bash
# Build script detects and uses git version
./build.sh
# Output: Building version: 2.0.7-dev
```

#### During Release
```bash
# Tag a release
git tag v2.0.7
git push origin v2.0.7

# Or use release script
./scripts/release-github.sh --version 2.0.7
```

### 5. GitHub Actions Integration

The release workflow automatically:
- Detects version from git tag
- Updates all version files before building
- Ensures all artifacts have consistent versions

## Version Locations

Versions are dynamically updated in:

### Rust Crates
- `/Cargo.toml` - Workspace version
- `/crates/*/Cargo.toml` - Individual crate versions
- Build-time injection via `build.rs`

### Python Bindings
- `/bindings/python/pyproject.toml`
- `/crates/python/src/lib.rs` - `__version__` attribute

### JavaScript/WASM
- `/crates/wasm/pkg/package.json` - Updated after build
- `/docs/pkg/package.json` - For web distribution

### Other Files
- `/Formula/vexy_json.rb` - Homebrew formula (releases only)
- CLI `--version` output
- API version info methods

## Workflow Examples

### Creating a New Release

1. **Tag the release:**
   ```bash
   git tag v2.0.7
   git push origin v2.0.7
   ```

2. **GitHub Actions automatically:**
   - Updates all version numbers to 2.0.7
   - Builds all artifacts with version 2.0.7
   - Creates release with properly versioned files

### Local Development

1. **After creating a tag locally:**
   ```bash
   git tag v2.0.8-beta
   ./build.sh
   ```
   All builds will use version 2.0.8-beta

2. **Between releases:**
   ```bash
   # Currently at 5 commits after v2.0.7
   ./scripts/get-version.sh
   # Output: 2.0.7-dev
   ```

### Manual Version Update

If needed, you can manually update versions:
```bash
# This reads from git and updates all files
./scripts/update-versions.sh
```

## Benefits

1. **Single Source of Truth**: Git tags define versions
2. **Automatic Propagation**: No manual version updates needed
3. **Consistent Versions**: All components share the same version
4. **Development Versions**: Automatic `-dev` suffix between releases
5. **CI/CD Integration**: Works seamlessly with GitHub Actions

## Troubleshooting

### Version Not Updating

1. Check if you're on a tagged commit:
   ```bash
   git describe --tags
   ```

2. Manually run version update:
   ```bash
   ./scripts/update-versions.sh
   ```

### Build Shows Wrong Version

1. Clean build artifacts:
   ```bash
   cargo clean
   ```

2. Ensure git repository is accessible during build

### CI/CD Issues

The GitHub Actions workflows handle version updates automatically. If issues occur:
1. Check that scripts are executable
2. Verify git tag format (should be `vX.Y.Z`)
3. Ensure all secrets are configured

## Best Practices

1. **Always tag releases** with semantic version format: `vMAJOR.MINOR.PATCH`
2. **Don't manually edit** version numbers in files
3. **Use release script** for consistent release process
4. **Test locally** with `./scripts/get-version.sh` before pushing tags

## Implementation Details

The versioning system consists of:

- **Shell Scripts**: Version detection and update logic
- **Build Scripts**: Rust `build.rs` files for compile-time injection
- **CI/CD Integration**: GitHub Actions workflows with version handling
- **Fallback Logic**: Graceful degradation when git isn't available

This approach ensures that version management is automated, consistent, and reliable across all components of the Vexy JSON project.