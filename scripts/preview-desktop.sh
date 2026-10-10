#!/usr/bin/env bash
set -euo pipefail

if [ $# -lt 2 ]; then
  echo "usage: $0 out.png route[/settings-page] [width] [height]" >&2
  exit 1
fi
out=$(realpath -m "$1")
width=${3:-1280}
height=${4:-800}
root=$(cd "$(dirname "$0")/.." && pwd)
data=${FLUXA_DATA_DIR:-${TMPDIR:-$HOME/.cache/claude-tmp}/fluxa-preview}
mkdir -p "$data"

cargo build --manifest-path "$root/native/Cargo.toml" -p fluxa-desktop --offline -q

env -u WAYLAND_DISPLAY FLUXA_PREVIEW="$2" FLUXA_DATA_DIR="$data" \
  xvfb-run -a -s "-screen 0 ${width}x${height}x24" bash -c "
    '$root/native/target/debug/fluxa-desktop' >/dev/null 2>&1 &
    app=\$!
    sleep \${FLUXA_PREVIEW_WAIT:-7}
    import -window root '$out'
    kill \$app
  "
