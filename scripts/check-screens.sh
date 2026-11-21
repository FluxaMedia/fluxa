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
  "phone-menu mobile settings/0 390 844 FLUXA_PREVIEW_MENU=navSidebarMode"
  "desktop-menu desktop settings/0 1280 800 FLUXA_PREVIEW_MENU=navSidebarMode"
  "tv-menu tv settings/0 1920 1080 FLUXA_PREVIEW_MENU=navSidebarMode"
  "desktop-watch desktop player 1280 720 FLUXA_PREVIEW_WATCH=panel"
  "phone-player-tracks mobile player/tracks 844 390"
  "tablet-gallery mobile gallery 768 1024"
  "tablet-sheet mobile gallery/sheet 768 1024"
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
  "wide-home desktop home 1893 675"
  "wide-library desktop library 1893 675"
  "wide-discover desktop discover 1893 675"
  "tablet-library mobile library 768 1024"
  "tablet-discover mobile discover 768 1024"
  "desktop-settings desktop settings/0 1280 800"
  "phone-detail mobile detail/movie/fixture-featured 390 844"
  "desktop-calendar desktop calendar 1280 800"
  "desktop-live desktop live 1280 800"
  "phone-live mobile live 390 844"
  "tv-live tv live 1920 1080"
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

run_scene() {
  local scene=$1 name form route width height extra
  read -r name form route width height extra <<<"$scene"
  local out="$work/$name.png" lint="$work/$name.lint" result="$work/$name.result"
  rm -f "$lint" "$result"
  rm -f "$out"
  local try
  for try in 1 2 3; do
    env ${extra:+"$extra"} FLUXA_LAYOUT_LINT="$lint" FLUXA_PREVIEW_FIXTURE=1 FLUXA_PREVIEW_FORM=$form FLUXA_PREVIEW_WAIT=${FLUXA_PREVIEW_WAIT:-6} \
      bash "$root/scripts/preview-desktop.sh" "$out" "$route" "$width" "$height" >/dev/null 2>&1 || true
    [ -s "$out" ] && break
  done
  if [ ! -s "$out" ]; then
    echo "FAIL $name: no screenshot" >"$result"
    return 0
  fi
  {
    if [ -s "$lint" ]; then
      echo "LINT $name: $(sort -u "$lint" | wc -l) layout violations ($lint)"
    fi
    if [ "$update" = "--update" ]; then
      cp "$out" "$baselines/$name.png"
      echo "updated $name"
    elif [ ! -f "$baselines/$name.png" ]; then
      echo "FAIL missing baseline $name (run with --update)"
    else
      local changed count total
      changed=$(compare -metric AE -fuzz 3% "$baselines/$name.png" "$out" "$work/$name-diff.png" 2>&1 || true)
      total=$((width * height))
      count=$(printf "%s" "$changed" | awk '{printf "%d", $1}')
      if [ "$count" -gt $((total / 500)) ]; then
        echo "FAIL $name: $changed pixels differ (diff: $work/$name-diff.png)"
      else
        echo "ok   $name"
      fi
    fi
  } >"$result"
}

jobs=${JOBS:-4}
names=()
for scene in "${scenes[@]}"; do
  read -r name _ <<<"$scene"
  [[ -z "${ONLY:-}" || ",$ONLY," == *",$name,"* ]] || continue
  names+=("$name")
  run_scene "$scene" &
  sleep "${STAGGER:-2}"
  while [ "$(jobs -rp | wc -l)" -ge "$jobs" ]; do wait -n || true; done
done
wait

failed=0
for name in "${names[@]}"; do
  cat "$work/$name.result"
  grep -qE "^(FAIL|LINT)" "$work/$name.result" && failed=1
done
exit $failed
