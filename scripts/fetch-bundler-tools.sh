#!/usr/bin/env bash
# Puts the tools Tauri's bundler downloads into its tools directory, with retries, so that a
# GitHub Releases hiccup does not fail the bundle: the bundler downloads them once, with no retry,
# and GitHub answered 504 to it three times in a row on main (runs 42464179086, 42468592945).
#
#   bash scripts/fetch-bundler-tools.sh <tools-dir> <Linux|Windows>
#
# Names, URLs and the WiX checksum are those of tauri-bundler 2.10.4, the bundler of
# @tauri-apps/cli 2.12.4 (bundle/linux/appimage/linuxdeploy.rs, bundle/windows/msi/mod.rs). If a
# later CLI expects other files, it does not find these and downloads its own, as without this script.
set -euo pipefail

dir=$1
os=$2
mkdir -p "$dir"

fetch() {
  local url=$1 out=$2
  [ -s "$out" ] && return
  curl --fail --location --silent --show-error --retry 6 --retry-all-errors --retry-delay 10 \
    --max-time 300 --output "$out.part" "$url"
  mv "$out.part" "$out"
}

case "$os" in
  Linux)
    releases=https://github.com/tauri-apps/binary-releases/releases/download
    fetch "$releases/apprun-old/AppRun-x86_64" "$dir/AppRun-x86_64"
    fetch "$releases/linuxdeploy-07333c6/linuxdeploy-x86_64.AppImage" "$dir/linuxdeploy-07333c6-x86_64.AppImage"
    # Optional for the bundler, which falls back to linuxdeploy's built-in copy without it.
    fetch https://github.com/linuxdeploy/linuxdeploy-plugin-appimage/releases/download/continuous/linuxdeploy-plugin-appimage-x86_64.AppImage \
      "$dir/linuxdeploy-plugin-appimage.AppImage"
    chmod +x "$dir"/*.AppImage "$dir/AppRun-x86_64"
    ;;
  Windows)
    wix="$dir/WixTools314"
    [ -e "$wix/candle.exe" ] && exit 0
    zip="$dir/wix314-binaries.zip"
    fetch https://github.com/wixtoolset/wix3/releases/download/wix3141rtm/wix314-binaries.zip "$zip"
    echo "6ac824e1642d6f7277d0ed7ea09411a508f6116ba6fae0aa5f2c7daa2ff43d31  $zip" | sha256sum --check --status
    mkdir -p "$wix"
    powershell -NoProfile -Command "Expand-Archive -Force -Path '$zip' -DestinationPath '$wix'"
    rm "$zip"
    ;;
  *)
    echo "unsupported OS: $os" >&2
    exit 1
    ;;
esac
