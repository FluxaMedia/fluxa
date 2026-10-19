#!/usr/bin/env bash
set -euo pipefail

root=$(cd "$(dirname "$0")/.." && pwd)
baselines="$root/native/fixtures/screens"
work="${TMPDIR:-$HOME/.cache/claude-tmp}/fluxa-screens"
update=${1:-}
mkdir -p "$baselines" "$work"

scenes=(
  "phone-gallery mobile gallery 390 1100"
  "desktop-gallery desktop gallery 1280 900"
  "phone-sheet mobile gallery/sheet 390 844"
  "desktop-drawer desktop gallery/drawer 1280 800"
  "tv-focus tv gallery/focus 1920 1080"
  "phone-player-tracks mobile player/tracks 844 390"
  "desktop-player-chapters desktop player/chapters 1280 720"
  "tv-player-speed tv player/speed 1920 1080"
)

failed=0
for scene in "${scenes[@]}"; do
  read -r name form route width height <<<"$scene"
  out="$work/$name.png"
  FLUXA_PREVIEW_FORM=$form FLUXA_PREVIEW_WAIT=${FLUXA_PREVIEW_WAIT:-9} \
    bash "$root/scripts/preview-desktop.sh" "$out" "$route" "$width" "$height" >/dev/null 2>&1
  if [ "$update" = "--update" ]; then
    cp "$out" "$baselines/$name.png"
    echo "updated $name"
    continue
  fi
  if [ ! -f "$baselines/$name.png" ]; then
    echo "missing baseline $name (run with --update)"
    failed=1
    continue
  fi
  changed=$(compare -metric AE -fuzz 3% "$baselines/$name.png" "$out" "$work/$name-diff.png" 2>&1 || true)
  total=$((width * height))
  count=$(printf "%s" "$changed" | awk '{printf "%d", $1}')
  if [ "$count" -gt $((total / 500)) ]; then
    echo "FAIL $name: $changed pixels differ (diff: $work/$name-diff.png)"
    failed=1
  else
    echo "ok   $name"
  fi
done
exit $failed
