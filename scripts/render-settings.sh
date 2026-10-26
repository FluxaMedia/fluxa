#!/usr/bin/env bash
set -euo pipefail

root=$(cd "$(dirname "$0")/.." && pwd)
form=${1:-desktop}
out=${2:-${TMPDIR:-$HOME/.cache/claude-tmp}/settings-$form}
case $form in
  mobile) size="390 844" ;;
  tv) size="1920 1080" ;;
  *) size="1280 800" ;;
esac
count=$(grep -c '^        title: "' "$root/native/fluxa-ui/src/settings/sections.rs" || true)
mkdir -p "$out"
cargo build --manifest-path "$root/native/Cargo.toml" -p fluxa-desktop --offline -q

render() {
  FLUXA_PREVIEW_FIXTURE=1 FLUXA_PREVIEW_FORM=$form \
    bash "$root/scripts/preview-desktop.sh" "$out/$(printf %02d "$1").png" "settings/$1" $size >/dev/null 2>&1
}
export -f render
export root form out size
seq 0 $((count - 1)) | xargs -P "${JOBS:-4}" -I{} bash -c 'render {}'
if command -v montage >/dev/null; then
  montage "$out"/*.png -tile 5x -geometry 640x400+6+6 -background '#111' "$out/all.png"
fi
echo "$out"
