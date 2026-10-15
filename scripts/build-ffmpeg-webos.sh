#!/usr/bin/env bash
set -euo pipefail

sdk="${WEBOS_SDK:-$HOME/.cache/webos-toolchain/sdk/arm-webos-linux-gnueabi_sdk-buildroot}"
src="${FFMPEG_SRC:-$HOME/.cache/webos-toolchain/ffmpeg/src/ffmpeg-9.0.2}"
out="${FFMPEG_OUT:-$HOME/.cache/webos-toolchain/ffmpeg/arm}"
sysroot="$sdk/arm-webos-linux-gnueabi/sysroot"

[ -x "$src/configure" ] || { echo "FFmpeg source not found at $src" >&2; exit 3; }
export PATH="$sdk/bin:$PATH"
export TMPDIR="${TMPDIR:-$HOME/.cache/claude-tmp}"
mkdir -p "$TMPDIR" "$out/build"

demuxers=matroska,mov,mpegts,mpegps,avi,flv,ogg,wav,flac,mp3,aac,ac3,eac3,dts,hls,dash,webm_dash_manifest,concat,srt,webvtt,ass,subviewer
parsers=h264,hevc,vp9,av1,aac,aac_latm,ac3,opus,mpegaudio,dca,vorbis,flac
protocols=file,http,https,tcp,tls,crypto,hls,data,pipe

cd "$out/build"
"$src/configure" \
  --prefix="$out/install" \
  --enable-cross-compile --arch=arm --cpu=cortex-a9 --target-os=linux \
  --cross-prefix=arm-linux- --sysroot="$sysroot" \
  --disable-everything --disable-programs --disable-doc --disable-debug \
  --disable-avdevice --disable-avfilter --disable-swscale --disable-swresample \
  --enable-static --disable-shared --enable-pic \
  --enable-network --enable-openssl --enable-version3 \
  --disable-bzlib --disable-lzma --disable-iconv --disable-xlib --disable-sdl2 \
  --extra-cflags="-I$sysroot/usr/include -O2" --extra-ldflags="-L$sysroot/usr/lib" \
  --enable-demuxer="$demuxers" --enable-parser="$parsers" --enable-protocol="$protocols"
make -j"$(nproc)"
make install
