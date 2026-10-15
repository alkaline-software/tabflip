#!/bin/sh
# Draw the app icon (scripts/icon/draw.swift) and pack the .icns.
#
#   scripts/make-icon.sh
set -eu
cd "$(dirname "$0")/.."
dir=crates/tabflip-app/assets/icon
mkdir -p "$dir"
swift scripts/icon/draw.swift "$dir"
iconutil -c icns "$dir/icon.iconset" -o "$dir/icon.icns"
rm -rf "$dir/icon.iconset"
ls "$dir"
