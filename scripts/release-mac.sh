#!/bin/zsh
# The macOS release build: a Developer ID signature with the hardened
# runtime, a disk image with the app and an Applications link, and (with
# --notarize) notarization and a stapled ticket. Run on the machine of
# whoever holds the Developer ID (WORKFLOW.md); CI runs it with
# --identity - for its ad-hoc drafts.
#
#   scripts/release-mac.sh [--target <triple>] [--no-build] [--identity <name>|-]
#                          [--notarize] [--out <path.dmg>]
#
#   --identity     default: $TABFLIP_SIGN_IDENTITY, else the first
#                  "Developer ID Application" in the keychain
#   --identity -   ad-hoc signature
#   --notarize     xcrun notarytool with the keychain profile
#                  $TABFLIP_NOTARY_PROFILE (default notarytool-profile),
#                  then staple and validate
set -euo pipefail
cd "$(dirname "$0")/.."
export PATH="$HOME/.cargo/bin:$PATH"

IDENTITY="${TABFLIP_SIGN_IDENTITY:-$(security find-identity -v -p codesigning 2>/dev/null | sed -n 's/.*"\(Developer ID Application: [^"]*\)".*/\1/p' | head -1)}"
NOTARY_PROFILE="${TABFLIP_NOTARY_PROFILE:-notarytool-profile}"
TARGET=""; BUILD=1; NOTARIZE=0; OUT=""
while [[ $# -gt 0 ]]; do
  case "$1" in
    --target) TARGET="$2"; shift 2 ;;
    --no-build) BUILD=0; shift ;;
    --identity) IDENTITY="$2"; shift 2 ;;
    --notarize) NOTARIZE=1; shift ;;
    --out) OUT="$2"; shift 2 ;;
    *) echo "unknown option $1" >&2; exit 2 ;;
  esac
done
[[ -n "$IDENTITY" ]] || { echo "release-mac: no Developer ID Application certificate in the keychain; pass --identity - for an ad-hoc build" >&2; exit 1; }

case "${TARGET:-$(uname -m)}" in
  aarch64*|arm64) ARCH=arm64 ;;
  x86_64*) ARCH=x64 ;;
  *) ARCH="$(uname -m)" ;;
esac
OUT="${OUT:-dist/tabflip-mac-$ARCH.dmg}"
VERSION=$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)
TARGET_FLAG=(); [[ -n "$TARGET" ]] && TARGET_FLAG=(--target "$TARGET")

if (( BUILD )); then
  cargo build --release --locked -p tabflip-app "${TARGET_FLAG[@]}"
fi
rm -rf dist/tabflip.app
cargo packager --release -p tabflip-app "${TARGET_FLAG[@]}" --formats app
APP=dist/tabflip.app
[[ -d "$APP" ]] || { echo "packager left no $APP" >&2; exit 1; }

# The hardened runtime and a secure timestamp are what notarization
# requires. tabflip asks for no entitlements: Accessibility is granted by
# the person, not declared.
if [[ "$IDENTITY" == "-" ]]; then
  codesign --force --deep --sign - "$APP"
else
  codesign --force --deep --sign "$IDENTITY" --options runtime --timestamp "$APP"
fi
codesign --verify --deep --strict --verbose=2 "$APP"

STAGE=dist/dmg-staging
rm -rf "$STAGE" "$OUT"
mkdir -p "$STAGE" "$(dirname "$OUT")"
cp -R "$APP" "$STAGE/tabflip.app"
ln -s /Applications "$STAGE/Applications"
# hdiutil fails now and then with "Resource busy" on a machine that has
# just built, so it is tried a few times.
made=""
for attempt in 1 2 3 4 5; do
  if hdiutil create -volname "tabflip v$VERSION" -srcfolder "$STAGE" -ov -format UDZO -imagekey zlib-level=9 "$OUT"; then
    made=1
    break
  fi
  echo "release-mac: hdiutil create failed (attempt $attempt), trying again" >&2
  sleep $((attempt * 3))
done
[ -n "$made" ] || { echo "release-mac: could not create the disk image" >&2; exit 1; }
# Left in dist/, the bundle is a second tabflip for Launchpad and Spotlight.
rm -rf "$STAGE" "$APP" dist/.cargo-packager
if [[ "$IDENTITY" != "-" ]]; then
  codesign --force --sign "$IDENTITY" "$OUT"
fi

if (( NOTARIZE )); then
  [[ "$IDENTITY" != "-" ]] || { echo "an ad-hoc signature cannot be notarized" >&2; exit 1; }
  xcrun notarytool submit "$OUT" --keychain-profile "$NOTARY_PROFILE" --wait
  xcrun stapler staple "$OUT"
  xcrun stapler validate "$OUT"
fi

HOW=$([[ "$IDENTITY" == "-" ]] && echo "ad-hoc" || echo "signed")
(( NOTARIZE )) && HOW="$HOW, notarized"
echo "built $OUT ($VERSION, $ARCH, $HOW)"
