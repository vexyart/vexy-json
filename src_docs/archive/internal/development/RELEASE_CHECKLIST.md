# Vexy JSON Release Checklist

This checklist guides the release process for Vexy JSON. Follow these steps to ensure a smooth release.

## Pre-Release Verification

### 1. Code Quality
- [ ] All tests pass: `./build.sh`
- [ ] No critical bugs or issues
- [ ] Documentation is up to date
- [ ] CHANGELOG.md reflects all changes

### 2. Version Verification
- [ ] Version numbers are consistent across all files
- [ ] Run `./scripts/get-version.sh` to verify current version
- [ ] Ensure version follows semantic versioning

### 3. Build Verification
- [ ] Release build completes: `cargo build --release`
- [ ] All examples compile: `cargo build --examples`
- [ ] Benchmarks run: `cargo bench`
- [ ] Cross-platform builds work (if applicable)

## Release Process

### 1. Final Preparation
- [ ] Ensure working directory is clean: `git status`
- [ ] All changes are committed
- [ ] On the correct branch (usually `main`)

### 2. Execute Release
```bash
# Run the release script with the new version
./release.sh <version>

# Example:
./release.sh 2.0.0
```

### 3. Release Script Actions
The release script will automatically:
- Update version numbers across all files
- Create a git tag with 'v' prefix
- Build release artifacts in `dist/`
- Commit all changes
- Push commits and tags to GitHub

### 4. Post-Release Verification
- [ ] Check GitHub for the new tag
- [ ] Verify release artifacts in `dist/` directory
- [ ] Test installation from release artifacts
- [ ] Update any package registries (crates.io, npm, etc.)

## Platform-Specific Releases

### Crates.io (Rust)
```bash
cd crates/core && cargo publish
cd ../serde && cargo publish
cd ../cli && cargo publish
```

### NPM (WebAssembly)
```bash
cd crates/wasm
wasm-pack build --release
cd pkg && npm publish
```

### Homebrew (macOS)
- [ ] Update Formula/vexy_json.rb with new version and SHA256
- [ ] Test installation: `brew install --build-from-source ./Formula/vexy_json.rb`
- [ ] Submit PR to homebrew-core (if applicable)

## Communication

### 1. Release Notes
- [ ] Create GitHub release with changelog
- [ ] Highlight breaking changes
- [ ] Thank contributors

### 2. Announcements
- [ ] Update project README with new version
- [ ] Post to relevant forums/communities
- [ ] Update documentation site

## Rollback Plan

If issues are discovered post-release:
1. Document the issue
2. Decide on fix urgency
3. If critical:
   - Prepare patch release (x.y.z+1)
   - Follow expedited release process
4. If non-critical:
   - Schedule for next regular release
   - Document in known issues

## Notes

- Always test the release process with `--dry-run` first
- Keep release commits atomic and focused
- Tag releases consistently with 'v' prefix (e.g., v2.0.0)
- Maintain backward compatibility when possible