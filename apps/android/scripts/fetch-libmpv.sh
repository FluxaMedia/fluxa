#!/usr/bin/env bash
set -euo pipefail

repo="KhooLy/mpv"
tag="${1:-latest}"
asset="libmpv-android.zip"
dest="$(cd "$(dirname "$0")/.." && pwd)/app/libmpv"

latest_tag() {
    curl -fsSL --max-time 5 -o /dev/null -w '%{url_effective}' "https://github.com/$repo/releases/latest" 2>/dev/null | sed -n 's#.*/tag/##p'
}

stamp="$dest/.tag"
installed="$(cat "$stamp" 2>/dev/null || true)"
if [[ -d "$dest/jni" && "${FLUXA_FORCE_LIBMPV_FETCH:-0}" != "1" ]]; then
    [[ "$installed" == "local" ]] && exit 0
    want="$tag"
    [[ "$tag" == "latest" ]] && want="$(latest_tag || true)"
    [[ -z "$want" || "$installed" == "$want" ]] && exit 0
fi

if [[ -n "${FLUXA_LIBMPV_ANDROID_DIR:-}" ]]; then
    rm -rf "$dest"
    mkdir -p "$dest"
    cp -r "$FLUXA_LIBMPV_ANDROID_DIR"/. "$dest"/
    echo local > "$stamp"
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
rm -rf "$dest"
mkdir -p "$dest"
unzip -q "$tmp/$asset" -d "$dest"
test -f "$dest/jni/arm64-v8a/libmpv.so"
resolved="$tag"
[[ "$tag" == "latest" ]] && resolved="$(latest_tag || true)"
echo "$resolved" > "$stamp"
