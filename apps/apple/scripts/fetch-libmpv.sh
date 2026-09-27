#!/usr/bin/env bash
set -euo pipefail

repo="KhooLy/mpv"
tag="${1:-latest}"
asset="libmpv-apple.xcframework.zip"
vendor_dir="$(cd "$(dirname "$0")/.." && pwd)/Vendor"

if [[ -d "$vendor_dir/Libmpv.xcframework" && "${FLUXA_FORCE_LIBMPV_FETCH:-0}" != "1" ]]; then
    exit 0
fi

if [[ "$tag" == "latest" ]]; then
    url="https://github.com/$repo/releases/latest/download/$asset"
else
    url="https://github.com/$repo/releases/download/$tag/$asset"
fi

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
curl -fL "$url" -o "$tmp/$asset"
rm -rf "$vendor_dir/Libmpv.xcframework"
mkdir -p "$vendor_dir"
unzip -q "$tmp/$asset" -d "$vendor_dir"
test -d "$vendor_dir/Libmpv.xcframework"
