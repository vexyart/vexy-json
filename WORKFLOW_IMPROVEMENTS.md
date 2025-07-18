# GitHub Actions Workflow Improvements

This document contains improvements to the GitHub Actions workflows that need to be applied manually due to GitHub App permission restrictions.

## Release Workflow Improvements

The following improvements should be applied to `.github/workflows/release.yml`:

### 1. Add Version Validation Step

After the "Update version numbers" step in the `create-release` job, add this validation step:

```yaml
      - name: Validate version consistency
        shell: bash
        run: |
          # Verify version was updated correctly
          EXPECTED_VERSION="${{ steps.get_version.outputs.version }}"
          ACTUAL_VERSION=$(./scripts/get-version.sh)
          
          if [[ "$ACTUAL_VERSION" != "$EXPECTED_VERSION" ]]; then
            echo "❌ Version mismatch: expected $EXPECTED_VERSION, got $ACTUAL_VERSION"
            exit 1
          fi
          
          echo "✅ Version validation successful: $ACTUAL_VERSION"
```

### 2. Add Rust Setup and Testing

After the validation step, add these steps:

```yaml
      - name: Setup Rust
        uses: dtolnay/rust-toolchain@stable
        
      - name: Setup Rust Cache
        uses: Swatinem/rust-cache@v2
        
      - name: Run release validation tests
        shell: bash
        run: |
          echo "🧪 Running release validation tests..."
          ./build.sh test
```

### 3. Fix Release Name Capitalization

Change the release name in the "Create Release" step from:
```yaml
release_name: VEXY_JSON v${{ steps.get_version.outputs.version }}
```

To:
```yaml
release_name: Vexy JSON v${{ steps.get_version.outputs.version }}
```

## Complete Updated create-release Job

Here's the complete updated `create-release` job section:

```yaml
  create-release:
    name: Create Release
    runs-on: ubuntu-latest
    outputs:
      upload_url: ${{ steps.create_release.outputs.upload_url }}
      version: ${{ steps.get_version.outputs.version }}
    steps:
      - uses: actions/checkout@v4

      - name: Get version
        id: get_version
        run: |
          if [ "${{ github.event_name }}" = "workflow_dispatch" ]; then
            VERSION="${{ github.event.inputs.version }}"
          else
            VERSION=${GITHUB_REF#refs/tags/v}
          fi
          echo "version=$VERSION" >> $GITHUB_OUTPUT

      - name: Update version numbers
        shell: bash
        run: |
          # Make scripts executable (skip on Windows)
          if [[ "${{ runner.os }}" != "Windows" ]]; then
            chmod +x scripts/get-version.sh scripts/update-versions.sh
          fi
          # Update all version numbers to match git tag
          bash ./scripts/update-versions.sh
          
      - name: Validate version consistency
        shell: bash
        run: |
          # Verify version was updated correctly
          EXPECTED_VERSION="${{ steps.get_version.outputs.version }}"
          ACTUAL_VERSION=$(./scripts/get-version.sh)
          
          if [[ "$ACTUAL_VERSION" != "$EXPECTED_VERSION" ]]; then
            echo "❌ Version mismatch: expected $EXPECTED_VERSION, got $ACTUAL_VERSION"
            exit 1
          fi
          
          echo "✅ Version validation successful: $ACTUAL_VERSION"
          
      - name: Setup Rust
        uses: dtolnay/rust-toolchain@stable
        
      - name: Setup Rust Cache
        uses: Swatinem/rust-cache@v2
        
      - name: Run release validation tests
        shell: bash
        run: |
          echo "🧪 Running release validation tests..."
          ./build.sh test

      - name: Create Release
        id: create_release
        uses: actions/create-release@v1
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
        with:
          tag_name: v${{ steps.get_version.outputs.version }}
          release_name: Vexy JSON v${{ steps.get_version.outputs.version }}
          draft: true
          prerelease: false
          body: |
            # Vexy JSON v${{ steps.get_version.outputs.version }}

            ## Highlights

            - SIMD-accelerated parsing for 2-3x performance improvement
            - Memory Pool V3 with 80% reduction in allocations
            - Parallel processing for large files
            - Streaming capability for gigabyte-scale files
            - Plugin system for extensibility
            - ML-based error recovery with actionable suggestions

            ## Installation

            ### macOS
            ```bash
            # Using Homebrew
            brew install vexy_json

            # Or download the installer
            # Download vexy_json-${{ steps.get_version.outputs.version }}-macos.dmg below
            ```

            ### Linux
            ```bash
            # Download and extract
            curl -L https://github.com/vexyart/vexy-json/releases/download/v${{ steps.get_version.outputs.version }}/vexy_json-${{ steps.get_version.outputs.version }}-linux-x86_64.tar.gz | tar xz
            sudo mv vexy_json /usr/local/bin/
            ```

            ### Windows
            ```powershell
            # Download vexy_json-${{ steps.get_version.outputs.version }}-windows-x86_64.zip below
            # Extract and add to PATH
            ```

            ### Cargo
            ```bash
            cargo install vexy_json-cli
            ```

            ## What's Changed

            See [CHANGELOG.md](https://github.com/vexyart/vexy-json/blob/v${{ steps.get_version.outputs.version }}/CHANGELOG.md) for details.

            ## Assets

            - **macOS**: `vexy_json-${{ steps.get_version.outputs.version }}-macos.dmg` - Installer with PKG
            - **macOS**: `vexy_json-${{ steps.get_version.outputs.version }}-macos.zip` - Standalone binary
            - **Linux**: `vexy_json-${{ steps.get_version.outputs.version }}-linux-x86_64.tar.gz` - x86_64 binary
            - **Linux**: `vexy_json-${{ steps.get_version.outputs.version }}-linux-aarch64.tar.gz` - ARM64 binary
            - **Windows**: `vexy_json-${{ steps.get_version.outputs.version }}-windows-x86_64.zip` - x86_64 binary
            - **Source**: `vexy_json-${{ steps.get_version.outputs.version }}.tar.gz` - Source code
```

## Benefits of These Improvements

1. **Version Validation**: Ensures that the version update process worked correctly
2. **Release Testing**: Runs comprehensive tests before creating the release
3. **Better Error Handling**: Fails early if version management has issues
4. **Consistent Naming**: Proper capitalization for release names

## How to Apply

1. Edit `.github/workflows/release.yml`
2. Find the `create-release` job
3. Apply the changes shown above
4. Commit and push the changes

These improvements will make the release process more reliable and catch potential issues early in the release pipeline.