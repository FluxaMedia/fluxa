#!/usr/bin/env bash
set -euo pipefail

PACKAGE="${1:-com.fluxa.app.tv}"
OUT_DIR="${2:-build/performance}"
TRACE_SECONDS="${TRACE_SECONDS:-20}"
mkdir -p "$OUT_DIR"

if ! command -v adb >/dev/null 2>&1; then
  echo "adb was not found in PATH" >&2
  exit 1
fi

if ! adb get-state >/dev/null 2>&1; then
  echo "No Android/TV device is connected" >&2
  exit 1
fi

STAMP="$(date +%Y%m%d-%H%M%S)"
REMOTE="/data/misc/perfetto-traces/fluxa-torrent-$STAMP.pb"
OUTPUT="$OUT_DIR/fluxa-torrent-$STAMP.pb"

adb shell am force-stop "$PACKAGE" >/dev/null
adb shell perfetto -o "$REMOTE" -t "${TRACE_SECONDS}s" -b 32mb \
  sched freq idle am wm gfx view binder_driver hal dalvik &
TRACE_PID=$!
sleep 1
adb shell monkey -p "$PACKAGE" -c android.intent.category.LAUNCHER 1 >/dev/null
wait "$TRACE_PID"
adb pull "$REMOTE" "$OUTPUT" >/dev/null
adb shell rm "$REMOTE" >/dev/null

echo "Perfetto trace: $OUTPUT"
