#!/bin/sh
# Build target/debug/tabflip into an app bundle and open it, quitting a
# dev copy that is already running.
#
#   scripts/dev-app.sh
#
# tabflip needs Accessibility, and macOS keeps that grant by who signed
# the app. A build signed by nobody (ad-hoc) is a new app after every
# build and has to be allowed again each time, so the bundle is signed
# with the first identity found of:
#
#   $TABFLIP_SIGN_IDENTITY
#   a "Developer ID Application" certificate (a release signer's machine)
#   an "Apple Development" certificate (free with any Apple ID, from Xcode)
#   "tabflip dev", made once by scripts/make-dev-cert.sh
#
# The bundle sits in a folder named ".noindex", so Spotlight and Launchpad
# do not list a second tabflip beside an installed one.
set -eu
cd "$(dirname "$0")/.."
export PATH="$HOME/.cargo/bin:$PATH"
cargo build -p tabflip-app
bin=target/debug/tabflip
app=target/debug/dev.noindex/tabflip.app

mkdir -p "$app/Contents/MacOS" "$app/Contents/Resources"
rm -f "$app/Contents/MacOS/tabflip"
cp "$bin" "$app/Contents/MacOS/tabflip"
cp -f crates/tabflip-app/assets/icon/icon.icns "$app/Contents/Resources/tabflip.icns"
cat > "$app/Contents/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleName</key><string>tabflip dev</string>
  <key>CFBundleDisplayName</key><string>tabflip (dev)</string>
  <key>CFBundleIdentifier</key><string>com.alkalinesoftware.tabflip.dev</string>
  <key>CFBundleVersion</key><string>0</string>
  <key>CFBundleShortVersionString</key><string>dev</string>
  <key>CFBundleExecutable</key><string>tabflip</string>
  <key>CFBundleIconFile</key><string>tabflip</string>
  <key>CFBundlePackageType</key><string>APPL</string>
  <key>LSMinimumSystemVersion</key><string>14.0</string>
  <key>LSUIElement</key><true/>
  <key>NSHighResolutionCapable</key><true/>
  <key>CFBundleDevelopmentRegion</key><string>en</string>
  <key>CFBundleLocalizations</key><array><string>en</string><string>zh-Hans</string><string>zh-Hant</string><string>ja</string><string>ko</string><string>fr</string><string>de</string><string>es</string></array>
</dict>
</plist>
PLIST

identities=$(security find-identity -p codesigning 2>/dev/null || true)
pick() { printf '%s\n' "$identities" | sed -n "s/.*\"\($1[^\"]*\)\".*/\1/p" | head -1; }
identity="${TABFLIP_SIGN_IDENTITY:-}"
[ -n "$identity" ] || identity=$(pick "Developer ID Application")
[ -n "$identity" ] || identity=$(pick "Apple Development")
[ -n "$identity" ] || identity=$(pick "tabflip dev")
if [ -n "$identity" ]; then
  codesign --force --sign "$identity" "$app" >/dev/null 2>&1 || { echo "dev-app: signing as \"$identity\" failed" >&2; exit 1; }
  echo "dev-app: signed as \"$identity\""
else
  codesign --force --sign - "$app" >/dev/null 2>&1
  echo "dev-app: no signing identity, so Accessibility must be allowed again after each build." >&2
  echo "dev-app: scripts/make-dev-cert.sh makes one that lasts." >&2
fi

# Any running copy quits, the dev one and an installed one, since two
# would both take the shortcut (SIGTERM puts the system's ⌘Tab back), and
# this one opens. Its log is ~/Library/Logs/tabflip-dev.log.
pkill -TERM -f "tabflip.app/Contents/MacOS/tabflip" 2>/dev/null && sleep 0.5 || true
open "$app"
echo "dev-app: opened $app; in Accessibility it is \"tabflip dev\"" >&2
