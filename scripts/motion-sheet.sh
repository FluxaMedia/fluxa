#!/usr/bin/env bash
set -euo pipefail

if [ $# -lt 1 ]; then
  echo "usage: $0 out.png [seconds] [fps] [adb input command...]" >&2
  exit 1
fi
out=$1
seconds=${2:-2}
fps=${3:-15}
shift $(( $# < 3 ? $# : 3 ))

work=${TMPDIR:-$HOME/.cache/claude-tmp}/motion-$$
mkdir -p "$work"
trap 'rm -rf "$work"; adb shell rm -f /sdcard/fluxa-motion.mp4' EXIT

adb shell screenrecord --time-limit "$seconds" --bit-rate 12000000 /sdcard/fluxa-motion.mp4 &
recorder=$!
sleep 0.6
if [ $# -gt 0 ]; then
  adb shell input "$@"
fi
wait $recorder
adb pull /sdcard/fluxa-motion.mp4 "$work/motion.mp4" >/dev/null

columns=$(( seconds * fps < 8 ? seconds * fps : 8 ))
ffmpeg -loglevel error -y -i "$work/motion.mp4" \
  -vf "fps=$fps,scale=200:-1,tile=${columns}x$(( (seconds * fps + columns - 1) / columns ))" \
  -frames:v 1 "$out"
