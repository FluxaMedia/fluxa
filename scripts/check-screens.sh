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
  "phone-menu mobile gallery/menu 390 844"
  "desktop-menu desktop gallery/menu 1280 800"
  "tv-menu tv gallery/menu 1920 1080"
  "desktop-watch desktop gallery/watch 1280 800"
  "desktop-watch-room desktop gallery/watch-room 1280 800"
  "phone-watch-room mobile gallery/watch-room 390 844"
  "phone-player-tracks mobile player/tracks 844 390"
  "desktop-player-chapters desktop player/chapters 1280 720"
  "desktop-player-sync desktop player/sync-open 1280 720"
  "tv-player-speed tv player/speed 1920 1080"
  "desktop-player-trailer desktop player/trailer 1280 720"
  "desktop-home desktop home 1280 800"
  "phone-home mobile home 390 844"
  "tv-home tv home 1920 1080"
  "desktop-library desktop library 1280 800"
  "phone-discover mobile discover 390 844"
  "desktop-discover desktop discover 1280 800"
  "desktop-settings desktop settings/0 1280 800"
  "phone-detail mobile detail/movie/fixture-featured 390 844"
  "desktop-calendar desktop calendar 1280 800"
  "desktop-calendar-day desktop calendar/12 1280 800"
  "phone-calendar-day mobile calendar/12 390 844"
  "desktop-library-downloads desktop library/downloads 1280 800"
  "phone-library-downloads mobile library/downloads 390 844"
  "desktop-settings-content-hub desktop settings/18/hub 1280 800"
  "desktop-settings-addons desktop settings/18 1280 800"
  "desktop-settings-plugins desktop settings/19 1280 800"
  "desktop-settings-collections desktop settings/17 1280 800"
  "desktop-settings-badges desktop settings/13 1280 800"
  "phone-settings-hub mobile settings/hub 390 844"
  "phone-settings-account mobile settings/0 390 844"
  "phone-settings-addons mobile settings/18 390 844"
  "desktop-player-sources desktop player/sources 1280 720"
  "phone-player-sources mobile player/sources 390 844"
  "desktop-player-loading desktop player/loading 1280 720"
  "desktop-player-recommendations desktop player/recommendations 1280 720"
  "desktop-player-next desktop player/next 1280 720"
)

cargo build --manifest-path "$root/native/Cargo.toml" -p fluxa-desktop --offline -q
export FLUXA_PREVIEW_NO_BUILD=1

failed=0
for scene in "${scenes[@]}"; do
  read -r name form route width height <<<"$scene"
  out="$work/$name.png"
  lint="$work/$name.lint"
  rm -f "$lint"
  FLUXA_LAYOUT_LINT="$lint" FLUXA_PREVIEW_FIXTURE=1 FLUXA_PREVIEW_FORM=$form FLUXA_PREVIEW_WAIT=${FLUXA_PREVIEW_WAIT:-9} \
    bash "$root/scripts/preview-desktop.sh" "$out" "$route" "$width" "$height" >/dev/null 2>&1
  if [ -s "$lint" ]; then
    echo "LINT $name: $(sort -u "$lint" | wc -l) layout violations ($lint)"
    failed=1
  fi
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
