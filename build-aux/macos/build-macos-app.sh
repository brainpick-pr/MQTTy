#!/bin/bash
# Build MQTTy as a macOS .app bundle
# Requires: Homebrew with gtk4, libadwaita, gtksourceview5, paho-mqtt-c, etc.

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
BUILD_DIR="$PROJECT_ROOT/build-macos"
APP_NAME="MQTTy"
APP_BUNDLE="$BUILD_DIR/$APP_NAME.app"
VERSION=$(grep '^version' "$PROJECT_ROOT/Cargo.toml" | head -1 | sed 's/.*"\(.*\)"/\1/')

echo "======================================"
echo "Building MQTTy v$VERSION for macOS"
echo "======================================"

# Check for required dependencies
echo "Checking dependencies..."

check_brew_package() {
    if ! brew list "$1" &>/dev/null; then
        echo "Missing: $1"
        echo "Install with: brew install $1"
        return 1
    fi
    return 0
}

MISSING_DEPS=0
for pkg in gtk4 libadwaita gtksourceview5 libpaho-mqtt librsvg meson ninja pkg-config gawk; do
    if ! check_brew_package "$pkg"; then
        MISSING_DEPS=1
    fi
done

if [ $MISSING_DEPS -eq 1 ]; then
    echo ""
    echo "Please install missing dependencies and try again."
    echo "You can install all at once with:"
    echo "  brew install gtk4 libadwaita gtksourceview5 libpaho-mqtt librsvg meson ninja pkg-config gawk"
    exit 1
fi

echo "All dependencies found!"

# Generate .icns if it doesn't exist
ICNS_FILE="$PROJECT_ROOT/data/icons/macos/MQTTy.icns"
if [ ! -f "$ICNS_FILE" ]; then
    echo "Generating macOS icon..."
    "$SCRIPT_DIR/generate-icns.sh"
fi

# Clean and create build directory
rm -rf "$BUILD_DIR"
mkdir -p "$BUILD_DIR"

# Build with Meson
echo "Configuring with Meson..."
cd "$PROJECT_ROOT"

# Set up environment for Homebrew GTK
export PKG_CONFIG_PATH="$(brew --prefix)/lib/pkgconfig:$(brew --prefix)/share/pkgconfig:$PKG_CONFIG_PATH"
export LDFLAGS="-L$(brew --prefix)/lib"
export CPPFLAGS="-I$(brew --prefix)/include"
export PATH="$(brew --prefix)/bin:$PATH"

meson setup "$BUILD_DIR/meson" \
    --prefix="$APP_BUNDLE/Contents/Resources" \
    --bindir="../MacOS" \
    -Dprofile=default

echo "Building..."
meson compile -C "$BUILD_DIR/meson"

# Create app bundle structure
echo "Creating app bundle..."
mkdir -p "$APP_BUNDLE/Contents/MacOS"
mkdir -p "$APP_BUNDLE/Contents/Resources"
mkdir -p "$APP_BUNDLE/Contents/Resources/share/icons/hicolor/scalable/apps"
mkdir -p "$APP_BUNDLE/Contents/Resources/share/glib-2.0/schemas"
mkdir -p "$APP_BUNDLE/Contents/Resources/share/locale"

# Copy Info.plist
cp "$SCRIPT_DIR/Info.plist" "$APP_BUNDLE/Contents/"

# Update version in Info.plist
sed -i '' "s/0\.1\.4/$VERSION/g" "$APP_BUNDLE/Contents/Info.plist"

# Copy icon
cp "$ICNS_FILE" "$APP_BUNDLE/Contents/Resources/"

# Copy binary
if [ -f "$BUILD_DIR/meson/src/release/MQTTy" ]; then
    cp "$BUILD_DIR/meson/src/release/MQTTy" "$APP_BUNDLE/Contents/MacOS/"
elif [ -f "$BUILD_DIR/meson/src/MQTTy" ]; then
    cp "$BUILD_DIR/meson/src/MQTTy" "$APP_BUNDLE/Contents/MacOS/"
else
    echo "Error: Could not find compiled binary"
    find "$BUILD_DIR" -name "MQTTy" -type f
    exit 1
fi

# Copy resources if meson install was successful
if [ -d "$BUILD_DIR/meson/data" ]; then
    # Copy GResource file
    find "$BUILD_DIR/meson/data" -name "*.gresource" -exec cp {} "$APP_BUNDLE/Contents/Resources/" \;
fi

# Copy SVG icon for GTK
cp "$PROJECT_ROOT/data/icons/io.github.otaxhu.MQTTy.svg" \
   "$APP_BUNDLE/Contents/Resources/share/icons/hicolor/scalable/apps/"

# Compile and copy GSettings schema
if [ -f "$PROJECT_ROOT/data/io.github.otaxhu.MQTTy.gschema.xml.in" ]; then
    SCHEMA_FILE="$APP_BUNDLE/Contents/Resources/share/glib-2.0/schemas/io.github.otaxhu.MQTTy.gschema.xml"
    sed "s/@app-id@/io.github.otaxhu.MQTTy/g" "$PROJECT_ROOT/data/io.github.otaxhu.MQTTy.gschema.xml.in" > "$SCHEMA_FILE"
    glib-compile-schemas "$APP_BUNDLE/Contents/Resources/share/glib-2.0/schemas/"
fi

# Create launcher script that sets up GTK environment
cat > "$APP_BUNDLE/Contents/MacOS/MQTTy-launcher" << 'LAUNCHER'
#!/bin/bash
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RESOURCES_DIR="$SCRIPT_DIR/../Resources"

# Set up environment for GTK on macOS
export XDG_DATA_DIRS="$RESOURCES_DIR/share:$(brew --prefix)/share:/usr/local/share:/usr/share"
export GSETTINGS_SCHEMA_DIR="$RESOURCES_DIR/share/glib-2.0/schemas"
export GTK_PATH="$(brew --prefix)/lib/gtk-4.0"
export GDK_PIXBUF_MODULE_FILE="$(brew --prefix)/lib/gdk-pixbuf-2.0/2.10.0/loaders.cache"
export GTK_IM_MODULE_FILE="$(brew --prefix)/lib/gtk-4.0/4.0.0/immodules.cache"

# Run the actual binary
exec "$SCRIPT_DIR/MQTTy-bin" "$@"
LAUNCHER

chmod +x "$APP_BUNDLE/Contents/MacOS/MQTTy-launcher"

# Rename binary and update Info.plist to use launcher
mv "$APP_BUNDLE/Contents/MacOS/MQTTy" "$APP_BUNDLE/Contents/MacOS/MQTTy-bin"
# Update CFBundleExecutable to use the launcher script (use plutil for safety)
plutil -replace CFBundleExecutable -string "MQTTy-launcher" "$APP_BUNDLE/Contents/Info.plist"

# Set executable permissions
chmod +x "$APP_BUNDLE/Contents/MacOS/MQTTy-bin"

echo ""
echo "======================================"
echo "Build complete!"
echo "======================================"
echo "App bundle: $APP_BUNDLE"
echo ""
echo "To run: open \"$APP_BUNDLE\""
echo ""
echo "To create a DMG for distribution:"
echo "  hdiutil create -volname MQTTy -srcfolder \"$APP_BUNDLE\" -ov -format UDZO \"$BUILD_DIR/MQTTy-$VERSION.dmg\""
