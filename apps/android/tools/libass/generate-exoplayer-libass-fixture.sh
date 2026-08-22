#!/usr/bin/env bash
set -euo pipefail

out_dir="${1:-/tmp/fluxa-libass-fixture}"
mkdir -p "$out_dir"

ass_file="$out_dir/libass-exoplayer-probe.ass"
video_file="$out_dir/libass-exoplayer-video.mp4"
mkv_file="$out_dir/libass-exoplayer-probe.mkv"

cat > "$ass_file" <<'ASS'
[Script Info]
ScriptType: v4.00+
PlayResX: 640
PlayResY: 360
WrapStyle: 0

[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Probe,sans-serif,48,&H0000FF00,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,3,0,5,20,20,20,1

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:01.00,0:00:03.00,Probe,,0,0,0,,{\pos(320,180)}EXOPLAYER LIBASS PROBE
ASS

ffmpeg -y \
  -f lavfi -i "color=c=black:s=640x360:d=4:r=30" \
  -f lavfi -i "sine=frequency=1000:duration=4" \
  -c:v libx264 -pix_fmt yuv420p \
  -c:a aac -shortest "$video_file"

ffmpeg -y \
  -i "$video_file" \
  -i "$ass_file" \
  -map 0:v:0 -map 0:a:0 -map 1:0 \
  -c:v copy -c:a copy -c:s ass \
  -metadata:s:s:0 language=eng \
  -metadata:s:s:0 title="Fluxa Libass Probe" \
  "$mkv_file"

printf '%s\n' "$mkv_file"
