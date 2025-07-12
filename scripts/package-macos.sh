#!/bin/bash
# this_file: scripts/package-macos.sh
# Package vexy_json for macOS as a .pkg inside a .dmg

set -e

# Configuration
BINARY_NAME="vexy-json"
VERSION=$(grep '^version' Cargo.toml | head -1 | cut -d'"' -f2)
BUNDLE_ID="com.twardoch.vexy-json"
INSTALL_LOCATION="/usr/local/bin"
BUILD_DIR="target/macos-package"
PKG_NAME="${BINARY_NAME}-${VERSION}.pkg"
DMG_NAME="${BINARY_NAME}-${VERSION}-macos.dmg"

echo "Building vexy_json v${VERSION} for macOS..."

# Clean and create build directory
rm -rf "${BUILD_DIR}"
mkdir -p "${BUILD_DIR}/root${INSTALL_LOCATION}"
mkdir -p "${BUILD_DIR}/scripts"
mkdir -p "${BUILD_DIR}/dmg"

# Build release binary
echo "Building release binary..."
cargo build --release

# Copy binary to package root
cp "target/release/${BINARY_NAME}" "${BUILD_DIR}/root${INSTALL_LOCATION}/"

# Create postinstall script to set permissions
cat > "${BUILD_DIR}/scripts/postinstall" << 'EOF'
#!/bin/bash
chmod 755 /usr/local/bin/vexy-json
exit 0
EOF
chmod +x "${BUILD_DIR}/scripts/postinstall"

# Build the package
echo "Creating installer package..."
pkgbuild \
    --root "${BUILD_DIR}/root" \
    --identifier "${BUNDLE_ID}" \
    --version "${VERSION}" \
    --scripts "${BUILD_DIR}/scripts" \
    --install-location "/" \
    "${BUILD_DIR}/${PKG_NAME}"

# Create a simple distribution XML for productbuild
cat > "${BUILD_DIR}/distribution.xml" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<installer-gui-script minSpecVersion="1">
    <title>vexy-json ${VERSION}</title>
    <organization>com.twardoch</organization>
    <domains enable_anywhere="true"/>
    <installation-check script="pm_install_check();"/>
    <script>
    function pm_install_check() {
        if(system.compareVersions(system.version.ProductVersion,'10.10') &lt; 0) {
            my.result.title = 'Failure';
            my.result.message = 'You need at least macOS 10.10 to install vexy-json.';
            my.result.type = 'Fatal';
            return false;
        }
        return true;
    }
    </script>
    <choices-outline>
        <line choice="default">
            <line choice="${BUNDLE_ID}"/>
        </line>
    </choices-outline>
    <choice id="default"/>
    <choice id="${BUNDLE_ID}" visible="false">
        <pkg-ref id="${BUNDLE_ID}"/>
    </choice>
    <pkg-ref id="${BUNDLE_ID}" version="${VERSION}" onConclusion="none">${PKG_NAME}</pkg-ref>
</installer-gui-script>
EOF

# Build final package with productbuild
productbuild \
    --distribution "${BUILD_DIR}/distribution.xml" \
    --package-path "${BUILD_DIR}" \
    "${BUILD_DIR}/dmg/${PKG_NAME}"

# Create README for DMG
cat > "${BUILD_DIR}/dmg/README.txt" << EOF
vexy-json ${VERSION} for macOS
========================

A forgiving JSON parser - Rust port of jsonic

Installation:
1. Double-click on ${PKG_NAME} to install
2. The 'vexy-json' command will be installed to /usr/local/bin
3. You may need to restart your terminal after installation

Usage:
  echo '{"foo": "bar",}' | vexy-json

For more information, visit:
https://github.com/vexyart/vexy-json

EOF

# Create the DMG
echo "Creating DMG..."
hdiutil create -volname "vexy-json ${VERSION}" \
    -srcfolder "${BUILD_DIR}/dmg" \
    -ov -format UDZO \
    "${DMG_NAME}"

# Cleanup
rm -rf "${BUILD_DIR}"

echo "✅ Successfully created ${DMG_NAME}"
echo "   Package contains ${PKG_NAME} installer"
echo "   Will install vexy-json to ${INSTALL_LOCATION}"