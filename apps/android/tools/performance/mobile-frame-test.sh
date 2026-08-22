#!/usr/bin/env bash
set -euo pipefail

PACKAGE="${1:-com.fluxa.app.mobile}"
OUT_DIR="${2:-build/performance}"
mkdir -p "$OUT_DIR"

if ! command -v adb >/dev/null 2>&1; then
  echo "adb was not found in PATH" >&2
  exit 1
fi
if ! adb get-state >/dev/null 2>&1; then
  echo "No Android device is connected" >&2
  exit 1
fi

read -r WIDTH HEIGHT < <(
  adb shell wm size | tr -d '\r' | sed -n 's/.* \([0-9][0-9]*\)x\([0-9][0-9]*\).*/\1 \2/p' | tail -1
)
if [[ -z "${WIDTH:-}" || -z "${HEIGHT:-}" ]]; then
  echo "Could not read the device display size" >&2
  exit 1
fi

x_mid=$((WIDTH / 2))
y_top=$((HEIGHT * 28 / 100))
y_bottom=$((HEIGHT * 82 / 100))
x_left=$((WIDTH * 18 / 100))
x_right=$((WIDTH * 84 / 100))
y_hero=$((HEIGHT * 42 / 100))
y_row=$((HEIGHT * 68 / 100))

echo "Starting $PACKAGE and resetting frame statistics..."
adb shell am force-stop "$PACKAGE" >/dev/null
adb shell monkey -p "$PACKAGE" -c android.intent.category.LAUNCHER 1 >/dev/null
sleep 6
adb shell dumpsys gfxinfo "$PACKAGE" reset >/dev/null

for _ in 1 2 3; do
  adb shell input swipe "$x_right" "$y_hero" "$x_left" "$y_hero" 180
  sleep 0.18
done
for _ in 1 2 3 4 5 6; do
  adb shell input swipe "$x_mid" "$y_bottom" "$x_mid" "$y_top" 220
  sleep 0.18
done
for _ in 1 2 3; do
  adb shell input swipe "$x_right" "$y_row" "$x_left" "$y_row" 180
  sleep 0.18
done
for _ in 1 2 3 4; do
  adb shell input swipe "$x_mid" "$y_top" "$x_mid" "$y_bottom" 220
  sleep 0.18
done

# Include the 18-second Home billboard update.
sleep 19
for _ in 1 2 3 4; do
  adb shell input swipe "$x_mid" "$y_bottom" "$x_mid" "$y_top" 220
  sleep 0.16
done

STAMP="$(date +%Y%m%d-%H%M%S)"
OUTPUT="$OUT_DIR/mobile-gfxinfo-$STAMP.txt"
adb shell dumpsys gfxinfo "$PACKAGE" framestats > "$OUTPUT"

echo
echo "Frame summary"
grep -E "Total frames rendered|Janky frames|50th percentile|90th percentile|95th percentile|99th percentile" "$OUTPUT" || true
echo
echo "Full report: $OUTPUT"
