#!/bin/bash
set -euo pipefail

if [[ "$(uname -s)" != "Darwin" ]]; then
  echo "package-dmg: macOS only" >&2
  exit 1
fi

ROOT="$(cd "$(dirname "$0")/../../../.." && pwd)"
cd "$ROOT"

VERSION="$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)"
if [[ -z "$VERSION" ]]; then
  echo "package-dmg: could not read version from Cargo.toml" >&2
  exit 1
fi

cargo build --release --no-default-features

APP="dist/Key Desk.app"
rm -rf "$APP"
mkdir -p "$APP/Contents/MacOS"
cp target/release/key-desk "$APP/Contents/MacOS/key-desk"
chmod +x "$APP/Contents/MacOS/key-desk"

cat > "$APP/Contents/Info.plist" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleName</key>
  <string>Key Desk</string>
  <key>CFBundleDisplayName</key>
  <string>Key Desk</string>
  <key>CFBundleIdentifier</key>
  <string>app.keydesk</string>
  <key>CFBundleExecutable</key>
  <string>key-desk</string>
  <key>CFBundlePackageType</key>
  <string>APPL</string>
  <key>CFBundleVersion</key>
  <string>${VERSION}</string>
  <key>CFBundleShortVersionString</key>
  <string>${VERSION}</string>
  <key>LSMinimumSystemVersion</key>
  <string>12.0</string>
  <key>NSHighResolutionCapable</key>
  <true/>
</dict>
</plist>
EOF

STAGE="$(mktemp -d)"
trap 'rm -rf "$STAGE"' EXIT
cp -R "$APP" "$STAGE/"
ln -s /Applications "$STAGE/Applications"

DMG="dist/key-desk-${VERSION}.dmg"
rm -f "$DMG"
hdiutil create -volname "Key Desk" -srcfolder "$STAGE" -ov -format UDZO "$DMG"

echo "package-dmg: $ROOT/$DMG"
