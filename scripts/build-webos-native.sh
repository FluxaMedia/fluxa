#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
sdk="${WEBOS_SDK:-$HOME/.cache/webos-toolchain/sdk/arm-webos-linux-gnueabi_sdk-buildroot}"
target=armv7-unknown-linux-gnueabi

[ -x "$sdk/bin/arm-linux-gcc" ] || {
  echo "webOS SDK not found at $sdk (set WEBOS_SDK; see github.com/openlgtv/buildroot-nc4 releases)" >&2
  exit 3
}

export PATH="$sdk/bin:$PATH"
export CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABI_LINKER=arm-linux-gcc
export CC_armv7_unknown_linux_gnueabi=arm-linux-gcc
export RUSTFLAGS="-C target-cpu=cortex-a9 ${RUSTFLAGS:-}"
export TMPDIR="${TMPDIR:-$HOME/.cache/claude-tmp}"
mkdir -p "$TMPDIR"

ffmpeg="${FFMPEG_OUT:-$HOME/.cache/webos-toolchain/ffmpeg/arm}/install"
features=()
if [ -d "$ffmpeg/lib" ]; then
  export FFMPEG_DIR="$ffmpeg"
  export WEBOS_SYSROOT="$sdk/arm-webos-linux-gnueabi/sysroot"
  export BINDGEN_EXTRA_CLANG_ARGS="--target=arm-linux-gnueabi --sysroot=$WEBOS_SYSROOT"
  features=(--features ffmpeg-static)
fi

cd "$root/native"
cargo build -p fluxa-webos --target "$target" --release "${features[@]}" "$@"
