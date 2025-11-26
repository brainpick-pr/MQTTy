#!/bin/bash
# Generate macOS .icns file from SVG icon
# Requires: rsvg-convert (from librsvg) or ImageMagick

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
ICON_SVG="$PROJECT_ROOT/data/icons/io.github.otaxhu.MQTTy.svg"
OUTPUT_DIR="$PROJECT_ROOT/data/icons/macos"
ICONSET_DIR="$OUTPUT_DIR/MQTTy.iconset"
ICNS_FILE="$OUTPUT_DIR/MQTTy.icns"

# Create output directories
mkdir -p "$OUTPUT_DIR"
mkdir -p "$ICONSET_DIR"

echo "Generating macOS icon set from $ICON_SVG..."

# Check for rsvg-convert or convert (ImageMagick)
if command -v rsvg-convert &> /dev/null; then
    CONVERT_CMD="rsvg-convert"
elif command -v convert &> /dev/null; then
    CONVERT_CMD="convert"
else
    echo "Error: Neither rsvg-convert nor ImageMagick convert found."
    echo "Install with: brew install librsvg  OR  brew install imagemagick"
    exit 1
fi

# Generate all required icon sizes for macOS
# Standard sizes: 16, 32, 128, 256, 512 (with @2x variants)
SIZES=(16 32 128 256 512)

for size in "${SIZES[@]}"; do
    echo "Generating ${size}x${size}..."
    if [ "$CONVERT_CMD" = "rsvg-convert" ]; then
        rsvg-convert -w "$size" -h "$size" "$ICON_SVG" -o "$ICONSET_DIR/icon_${size}x${size}.png"
    else
        convert -background none -resize "${size}x${size}" "$ICON_SVG" "$ICONSET_DIR/icon_${size}x${size}.png"
    fi

    # Generate @2x variant (except for 512 which becomes 1024)
    double=$((size * 2))
    echo "Generating ${size}x${size}@2x (${double}x${double})..."
    if [ "$CONVERT_CMD" = "rsvg-convert" ]; then
        rsvg-convert -w "$double" -h "$double" "$ICON_SVG" -o "$ICONSET_DIR/icon_${size}x${size}@2x.png"
    else
        convert -background none -resize "${double}x${double}" "$ICON_SVG" "$ICONSET_DIR/icon_${size}x${size}@2x.png"
    fi
done

# Convert iconset to icns using iconutil
echo "Converting iconset to .icns..."
iconutil -c icns "$ICONSET_DIR" -o "$ICNS_FILE"

# Clean up iconset directory
rm -rf "$ICONSET_DIR"

echo "Successfully created $ICNS_FILE"
