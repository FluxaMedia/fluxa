#!/usr/bin/env bash
set -euo pipefail

PACKAGE="${1:-com.fluxa.app.tv}"
OUT_DIR="${2:-build/performance}"
ROUNDS="${ROUNDS:-30}"
mkdir -p "$OUT_DIR"

if ! command -v adb >/dev/null 2>&1; then
  echo "adb was not found in PATH" >&2
  exit 1
fi

if ! adb get-state >/dev/null 2>&1; then
  echo "No Android/TV device is connected" >&2
  exit 1
fi

echo "Starting $PACKAGE and resetting frame statistics..."
adb shell am force-stop "$PACKAGE" >/dev/null
adb shell monkey -p "$PACKAGE" -c android.intent.category.LAUNCHER 1 >/dev/null
sleep 6
adb shell dumpsys gfxinfo "$PACKAGE" reset >/dev/null

# Exercise enough time to include the 18-second Home billboard rotation.
for ((i = 0; i < ROUNDS; i++)); do
  adb shell input keyevent KEYCODE_DPAD_RIGHT
  sleep 0.12
done
for ((i = 0; i < 5; i++)); do
  adb shell input keyevent KEYCODE_DPAD_DOWN
  sleep 0.18
done
for ((i = 0; i < ROUNDS; i++)); do
  adb shell input keyevent KEYCODE_DPAD_LEFT
  sleep 0.12
done
sleep 19
for ((i = 0; i < ROUNDS; i++)); do
  adb shell input keyevent KEYCODE_DPAD_RIGHT
  sleep 0.12
done

STAMP="$(date +%Y%m%d-%H%M%S)"
OUTPUT="$OUT_DIR/gfxinfo-$STAMP.txt"
adb shell dumpsys gfxinfo "$PACKAGE" framestats > "$OUTPUT"

echo
echo "Frame summary"
grep -E "Total frames rendered|Janky frames|50th percentile|90th percentile|95th percentile|99th percentile" "$OUTPUT" || true
echo
echo "Full report: $OUTPUT"
