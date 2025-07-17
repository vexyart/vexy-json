# Vexy JSON Release Guide

This guide covers the complete release process for Vexy JSON, including local development, testing, and automated CI/CD workflows.

## Quick Start

### Local Development

```bash
# Set up development environment
./scripts/dev-workflow.sh setup

# Quick development checks
./scripts/dev-workflow.sh check

# Format code
./scripts/dev-workflow.sh format

# Show project status
./scripts/dev-workflow.sh status
```

### Building and Testing

```bash
# Comprehensive build with all tests
./build.sh

# Quick test only
./build.sh test

# Test version management system
./build.sh test-version

# Validate project setup
./build.sh validate

# Show version information
./build.sh version
```

### Creating a Release

```bash
# Create a new release (interactive)
./scripts/release.sh --version 2.0.8

# Preview release without making changes
./scripts/release.sh --version 2.0.8 --dry-run

# Prepare release locally only
./scripts/release.sh --version 2.0.8 --local-only
```

## Detailed Release Process

### 1. Pre-Release Preparation

#### Development Environment Setup
```bash
# Install development tools
./scripts/dev-workflow.sh setup

# Set up git hooks for pre-commit checks
# (automatically done by setup command)
```

#### Code Quality Checks
```bash
# Format code and fix common issues
./scripts/dev-workflow.sh format

# Run comprehensive checks
./scripts/dev-workflow.sh check

# Run pre-commit validation
./scripts/dev-workflow.sh pre-commit
```

### 2. Version Management

#### Understanding Version Detection
Vexy JSON uses git tags as the single source of truth for versioning:

```bash
# Get current version (from git tags or Cargo.toml)
./scripts/get-version.sh

# Show detailed version information
./build.sh version
```

#### Version Formats
- **Release versions**: `v2.0.8` (creates version `2.0.8`)
- **Development versions**: `2.0.8-dev` (between releases)
- **Pre-release versions**: `v2.0.8-beta` (creates version `2.0.8-beta`)

### 3. Testing and Validation

#### Comprehensive Testing
```bash
# Run all tests including validation
./build.sh

# Run only tests (no build)
./build.sh test

# Test version management system
./build.sh test-version

# Validate project setup
./build.sh validate
```

#### Test Categories
- **Unit Tests**: `cargo test --workspace`
- **Integration Tests**: Examples and real-world scenarios
- **Documentation Tests**: `cargo test --doc`
- **Format/Lint Tests**: `cargo fmt --check` and `cargo clippy`
- **Version System Tests**: Git tag-based versioning validation

### 4. Release Creation

#### Automated Release (Recommended)
```bash
# Create and push release tag
git tag v2.0.8
git push origin v2.0.8

# GitHub Actions will automatically:
# 1. Update all version numbers
# 2. Run comprehensive tests
# 3. Build multiplatform binaries
# 4. Create GitHub release
# 5. Publish to crates.io
# 6. Publish WASM to NPM
# 7. Update Homebrew formula
```

#### Manual Release Process
```bash
# Complete release with all artifacts
./scripts/release.sh --version 2.0.8

# Preview what would be done
./scripts/release.sh --version 2.0.8 --dry-run

# Prepare locally (for testing)
./scripts/release.sh --version 2.0.8 --local-only
```

### 5. CI/CD Workflows

#### GitHub Actions Workflows

1. **CI Workflow** (`.github/workflows/ci.yml`)
   - Triggered on push/PR to main
   - Runs tests on multiple platforms
   - Validates code format and linting
   - Builds examples and documentation

2. **Release Workflow** (`.github/workflows/release.yml`)
   - Triggered on git tag push (`v*`)
   - Updates version numbers automatically
   - Builds multiplatform binaries
   - Creates GitHub release
   - Publishes to package registries

3. **Additional Workflows**
   - WASM builds
   - Security audits
   - Documentation deployment
   - Benchmark tracking

#### Monitoring Releases
```bash
# Check GitHub Actions status
# Visit: https://github.com/vexyart/vexy-json/actions

# Using GitHub CLI
gh workflow list
gh run list --workflow=release.yml
```

## Build System Reference

### Available Commands

#### Build Commands
- `./build.sh` - Complete build with all tests
- `./build.sh debug` - Debug build with basic tests
- `./build.sh release` - Release build with comprehensive testing
- `./build.sh clean` - Clean all build artifacts

#### Test Commands
- `./build.sh test` - Run comprehensive test suite
- `./build.sh test-version` - Test version management system
- `./build.sh validate` - Validate project setup

#### Utility Commands
- `./build.sh version` - Show version information
- `./build.sh help` - Show usage information
- `./build.sh wasm` - Build WebAssembly module
- `./build.sh install` - Install CLI locally (macOS)

### Development Workflow Commands

#### Setup and Maintenance
- `./scripts/dev-workflow.sh setup` - Set up development environment
- `./scripts/dev-workflow.sh status` - Show project status
- `./scripts/dev-workflow.sh clean-tags` - Remove test tags

#### Code Quality
- `./scripts/dev-workflow.sh check` - Quick development checks
- `./scripts/dev-workflow.sh format` - Format code
- `./scripts/dev-workflow.sh pre-commit` - Pre-commit validation

#### Version Management
- `./scripts/dev-workflow.sh bump-version` - Create test version
- `./scripts/get-version.sh` - Get current version
- `./scripts/update-versions.sh` - Update all version files

## Version Management Details

### How It Works

1. **Version Detection**: `./scripts/get-version.sh`
   - Checks for exact git tag on current commit
   - Falls back to most recent tag with `-dev` suffix
   - Uses Cargo.toml version as last resort

2. **Version Propagation**: `./scripts/update-versions.sh`
   - Updates all Cargo.toml files
   - Updates Python package configurations
   - Updates JavaScript/WASM packages
   - Updates Homebrew formula (releases only)

3. **Build-time Injection**: Each crate's `build.rs`
   - Detects version from git at compile time
   - Sets environment variables for runtime use
   - Fallback to Cargo.toml if git unavailable

### Version Validation

The system includes comprehensive validation:

```bash
# Test version detection and updating
./scripts/test-version-system.sh

# Validate version consistency
./build.sh test-version
```

## Release Artifacts

### Binary Distributions
- **macOS**: Universal binary (x86_64 + ARM64) with installer
- **Linux**: Static binaries for x86_64 and ARM64
- **Windows**: x86_64 executable with installer

### Package Registries
- **Rust**: crates.io (multiple crates)
- **JavaScript**: NPM (WASM package)
- **macOS**: Homebrew formula
- **Python**: PyPI (future)

### Documentation
- **API Documentation**: Generated with rustdoc
- **User Guide**: Comprehensive documentation site
- **Examples**: Runnable code examples

## Troubleshooting

### Common Issues

#### Version Not Updating
```bash
# Check git status
git status

# Manually update versions
./scripts/update-versions.sh

# Verify version detection
./scripts/get-version.sh
```

#### Build Failures
```bash
# Clean and rebuild
./build.sh clean
./build.sh

# Check project validation
./build.sh validate

# Run specific tests
./build.sh test
```

#### CI/CD Issues
```bash
# Check GitHub Actions logs
gh run list --workflow=release.yml
gh run view <run-id>

# Validate locally first
./scripts/release.sh --version X.Y.Z --dry-run
```

### Development Environment Issues
```bash
# Reset development environment
./scripts/dev-workflow.sh setup

# Check project status
./scripts/dev-workflow.sh status

# Clean up test artifacts
./scripts/dev-workflow.sh clean-tags
```

## Best Practices

### Version Management
1. Always use semantic versioning (X.Y.Z)
2. Tag releases with `v` prefix (`v2.0.8`)
3. Don't manually edit version numbers
4. Test locally before pushing tags

### Development Workflow
1. Use pre-commit hooks for code quality
2. Run comprehensive tests before releases
3. Validate version management system regularly
4. Keep development environment up to date

### Release Process
1. Always run dry-run before actual release
2. Monitor CI/CD pipeline after tag push
3. Verify artifacts are correctly built
4. Test installations from various sources

## Resources

### Documentation
- [VERSIONING.md](VERSIONING.md) - Detailed version management
- [CHANGELOG.md](CHANGELOG.md) - Release history
- [README.md](README.md) - Project overview

### Scripts
- `./build.sh` - Main build system
- `./scripts/release.sh` - Release automation
- `./scripts/dev-workflow.sh` - Development helpers
- `./scripts/test-version-system.sh` - Version system tests

### GitHub
- [Actions](https://github.com/vexyart/vexy-json/actions) - CI/CD status
- [Releases](https://github.com/vexyart/vexy-json/releases) - Release history
- [Issues](https://github.com/vexyart/vexy-json/issues) - Bug reports and features