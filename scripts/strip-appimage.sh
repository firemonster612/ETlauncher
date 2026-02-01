#!/bin/bash
# Strips the AppImage binary after building with NO_STRIP=1
# This is needed on non-LTS distros where linuxdeploy's strip binary is outdated

set -e

APPIMAGE_DIR="src-tauri/target/release/bundle/appimage"
APPIMAGE=$(find "$APPIMAGE_DIR" -maxdepth 1 -name "*.AppImage" ! -name "*_stripped.AppImage" | head -1)

if [ -z "$APPIMAGE" ]; then
    exit 1
fi

echo "Stripping AppImage..."

# Clean up any previous extraction
rm -rf "$APPIMAGE_DIR/squashfs-root"

# Extract
cd "$APPIMAGE_DIR"
APPIMAGE_BASENAME=$(basename "$APPIMAGE")
"./$APPIMAGE_BASENAME" --appimage-extract > /dev/null 2>&1

# Strip the binary
BINARY="squashfs-root/usr/bin/etlauncher"
if [ -f "$BINARY" ]; then
    strip "$BINARY"
else
    exit 1
fi

# Repack
STRIPPED_NAME="${APPIMAGE_BASENAME%.AppImage}_stripped.AppImage"

# Download appimagetool if not available
if ! command -v appimagetool &> /dev/null; then
    if [ ! -f /tmp/appimagetool ]; then
        wget -q https://github.com/AppImage/appimagetool/releases/download/continuous/appimagetool-x86_64.AppImage -O /tmp/appimagetool
        chmod +x /tmp/appimagetool
    fi
    APPIMAGETOOL=/tmp/appimagetool
else
    APPIMAGETOOL=appimagetool
fi

ARCH=x86_64 "$APPIMAGETOOL" squashfs-root "$STRIPPED_NAME" > /dev/null 2>&1

# Clean up
rm -rf squashfs-root

echo "Stripped AppImage: $(pwd)/$STRIPPED_NAME"
