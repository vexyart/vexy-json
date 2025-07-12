---
nav_title: Distribution Builds
nav_order: 4
---

# vexy_json Distribution Build Scripts

This directory contains robust, maintainable scripts for building vexy_json CLI deliverables for all major platforms:

- **macOS**: Universal binary, .pkg installer, and .dmg disk image
- **Windows**: .exe in a .zip archive
- **Linux**: Static binary in .tar.gz, plus .deb and .rpm packages if possible

## Prerequisites

- Rust toolchain (with `cargo`, `cargo-zigbuild`, `cross`, `cargo-deb`, `cargo-rpm`, `cargo-bundle`, `cargo-wix`)
- macOS: `create-dmg`, `pkgbuild`, `productbuild`
- Windows: `zip`, `x86_64-pc-windows-gnu` toolchain
- Linux: `dpkg`, `rpm`, `tar`, `gzip`

## Usage

From the project root:

```bash
./scripts/dist/build_all.sh [--release] [--version <semver>] [--skip-macos] [--skip-windows] [--skip-linux]
```

- `--release`: Build in release mode (optimized)
- `--version <semver>`: Override version (default: from Cargo.toml)
- `--skip-macos`, `--skip-windows`, `--skip-linux`: Skip building for a platform

All output is placed in the `dist/` directory.

## What Gets Built

- **macOS**: Universal binary, .pkg installer, .dmg disk image
- **Windows**: .exe in a .zip archive
- **Linux**: Static binary in .tar.gz, .deb, and .rpm (if tools available)

## Robustness & Maintenance

- The script is failsafe (`set -euo pipefail`)
- All steps are logged
- Platform builds can be skipped individually
- Version is auto-detected from Cargo.toml unless overridden
- All intermediate files are cleaned up

## CI/CD Integration

The GitHub Actions workflow for releases should call this script for all builds. The workflow should then upload the resulting artifacts to the GitHub release.

## Extending

- To add new platforms or packaging formats, add new sections to `build_all.sh`
- Keep all platform-specific logic in this script for maintainability
- Document any new dependencies in this README

## Support

For issues, see the main vexy_json repository or open an issue.