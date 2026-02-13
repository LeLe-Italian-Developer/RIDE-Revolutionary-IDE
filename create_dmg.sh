#!/bin/bash
set -e

# Configuration
APP_NAME="RIDE"
SOURCE_DIR="../VSCode-darwin-arm64"
APP_BUNDLE="${SOURCE_DIR}/${APP_NAME}.app"
DMG_NAME="RIDE-1.110.0-darwin-arm64.dmg"
STAGING_DIR="build/dmg-staging"

echo "Checking for app bundle at $APP_BUNDLE..."
if [ ! -d "$APP_BUNDLE" ]; then
    echo "Error: App bundle not found at $APP_BUNDLE"
    # Handle 'RIDE - Revolutionary IDE.app' case
    if [ -d "${SOURCE_DIR}/RIDE - Revolutionary IDE.app" ]; then
        echo "Found 'RIDE - Revolutionary IDE.app', renaming to 'RIDE.app'..."
        mv "${SOURCE_DIR}/RIDE - Revolutionary IDE.app" "${SOURCE_DIR}/RIDE.app"
    elif [ -d "${SOURCE_DIR}/ride.app" ]; then
        echo "Found ride.app, renaming to RIDE.app..."
        mv "${SOURCE_DIR}/ride.app" "${SOURCE_DIR}/RIDE.app"
    else
        ls -l "$SOURCE_DIR"
        exit 1
    fi
fi

echo "Cleaning up..."
rm -rf "$STAGING_DIR" "$DMG_NAME"
mkdir -p "$STAGING_DIR"

echo "Copying app to staging..."
cp -R "$APP_BUNDLE" "$STAGING_DIR/"

echo "Creating Applications link..."
ln -s /Applications "$STAGING_DIR/Applications"

echo "Creating DMG..."
# -volname sets the mounted disk name
# -srcfolder sets the source content
# -ov overwrites existing file
# -format UDBZ is zlib compressed (smaller)
hdiutil create -volname "${APP_NAME}" -srcfolder "$STAGING_DIR" -ov -format UDBZ "$DMG_NAME"

echo "Done: $DMG_NAME"
