#!/usr/bin/env bash
# Downloads the gpu-next-patched libmpv build for the current platform from
# https://github.com/KhooLy/mpv releases and unpacks it into native/fluxa-mpv/lib/.
#
# Usage: ./native/fluxa-mpv/fetch-libmpv.sh [tag]
# Defaults to the latest release if no tag is given.

set -euo pipefail

REPO="KhooLy/mpv"
TAG="${1:-latest}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
LIB_DIR="$SCRIPT_DIR/lib"

case "$(uname -s)" in
    Linux*)  ASSET="libmpv-linux-x86_64.zip" ;;
    Darwin*) ASSET="libmpv-macos-universal.zip" ;;
    MINGW*|MSYS*|CYGWIN*) ASSET="libmpv-windows-x86_64.zip" ;;
    *) echo "unrecognized platform $(uname -s)" >&2; exit 1 ;;
esac

if [[ "$TAG" == "latest" ]]; then
    URL="https://github.com/$REPO/releases/latest/download/$ASSET"
else
    URL="https://github.com/$REPO/releases/download/$TAG/$ASSET"
fi

mkdir -p "$LIB_DIR"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

echo "Downloading $URL"
curl -fL "$URL" -o "$TMP/$ASSET"
unzip -o "$TMP/$ASSET" -d "$LIB_DIR"

if [[ "$(uname -s)" == "Linux" && -f "$LIB_DIR/libpulse.so.0" ]]; then
    # libpulse.so.0 from the Ubuntu build has a private, versioned dependency.
    # ldd on libmpv alone can pass on CI while the downloaded bundle fails to
    # load on another distro because that transitive library was omitted.
    PULSE_COMMON="$(readelf -d "$LIB_DIR/libpulse.so.0" | sed -n 's/.*Shared library: \[\(libpulsecommon-[^]]*\)\].*/\1/p' | head -n 1)"
    if [[ -n "$PULSE_COMMON" && ! -f "$LIB_DIR/$PULSE_COMMON" ]]; then
        for dir in \
            /usr/lib/x86_64-linux-gnu/pulseaudio \
            /usr/lib/pulseaudio \
            '/usr/lib/Fluxa Desktop/lib'; do
            if [[ -f "$dir/$PULSE_COMMON" ]]; then
                cp -L "$dir/$PULSE_COMMON" "$LIB_DIR/$PULSE_COMMON"
                break
            fi
        done
        if [[ ! -f "$LIB_DIR/$PULSE_COMMON" ]]; then
            echo "Missing bundled $PULSE_COMMON required by libpulse.so.0" >&2
            exit 1
        fi
    fi
fi

if [[ "$(uname -s)" == "Darwin" ]]; then
    test -f "$LIB_DIR/libmpv.dylib"
    test -f "$LIB_DIR/libMoltenVK.dylib"
    compgen -G "$LIB_DIR/libplacebo*.dylib" >/dev/null
    otool -L "$LIB_DIR/libmpv.dylib" | grep -F libplacebo
    for dylib in "$LIB_DIR"/*.dylib; do
        arches="$(lipo -archs "$dylib")"
        echo "$dylib: $arches"
        echo "$arches" | grep -qw arm64
        echo "$arches" | grep -qw x86_64
        for arch in arm64 x86_64; do
            if otool -arch "$arch" -L "$dylib" | tail -n +3 | grep -Eq '^[[:space:]]+(/opt/homebrew/|/usr/local/)'; then
                echo "non-portable macOS dependency in $dylib ($arch)" >&2
                exit 1
            fi
        done
    done
fi

echo "libmpv installed into $LIB_DIR"
