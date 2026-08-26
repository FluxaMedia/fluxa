#!/usr/bin/env bash
set -euo pipefail

ffmpeg_version="${FLUXA_FFMPEG_VERSION:-7.1.1}"
project_dir="$(cd "$(dirname "$0")/.." && pwd)"
vendor_dir="$project_dir/FluxaPlayerKit/Vendor"
work_dir="$project_dir/Generated/ffmpeg"
source_dir="$work_dir/ffmpeg-$ffmpeg_version"
headers_dir="$work_dir/include"
xcframework="$vendor_dir/CFFmpeg.xcframework"

if [[ "${FLUXA_SKIP_FFMPEG_BUILD:-0}" == "1" ]]; then
    exit 0
fi

if [[ -d "$xcframework" && "${FLUXA_FORCE_FFMPEG_BUILD:-0}" != "1" ]]; then
    exit 0
fi

ios_min="${IPHONEOS_DEPLOYMENT_TARGET:-17.0}"
tvos_min="${TVOS_DEPLOYMENT_TARGET:-17.0}"
macos_min="${MACOSX_DEPLOYMENT_TARGET:-13.0}"

components=(
    --disable-everything
    --enable-protocol=file
    --enable-protocol=http
    --enable-protocol=tcp
    --enable-protocol=pipe
    --enable-demuxer=matroska
    --enable-demuxer=mov
    --enable-demuxer=mpegts
    --enable-demuxer=avi
    --enable-demuxer=asf
    --enable-demuxer=flv
    --enable-demuxer=hls
    --enable-demuxer=mp3
    --enable-demuxer=flac
    --enable-demuxer=wav
    --enable-demuxer=ogg
    --enable-demuxer=aac
    --enable-demuxer=ac3
    --enable-demuxer=dts
    --enable-demuxer=srt
    --enable-demuxer=ass
    --enable-demuxer=webvtt
    --enable-decoder=aac
    --enable-decoder=aac_latm
    --enable-decoder=ac3
    --enable-decoder=eac3
    --enable-decoder=dca
    --enable-decoder=truehd
    --enable-decoder=mlp
    --enable-decoder=mp3
    --enable-decoder=flac
    --enable-decoder=alac
    --enable-decoder=opus
    --enable-decoder=vorbis
    --enable-decoder=pcm_s16le
    --enable-decoder=pcm_s24le
    --enable-decoder=pcm_s32le
    --enable-decoder=pcm_f32le
    --enable-decoder=h264
    --enable-decoder=hevc
    --enable-decoder=vp9
    --enable-decoder=mpeg4
    --enable-decoder=subrip
    --enable-decoder=ass
    --enable-decoder=webvtt
    --enable-decoder=dvd_subtitle
    --enable-decoder=hdmv_pgs_subtitle
    --enable-parser=h264
    --enable-parser=hevc
    --enable-parser=vp9
    --enable-parser=aac
    --enable-parser=aac_latm
    --enable-parser=ac3
    --enable-parser=dca
    --enable-parser=mpegaudio
    --enable-parser=flac
    --enable-parser=opus
    --enable-bsf=h264_mp4toannexb
    --enable-bsf=hevc_mp4toannexb
    --enable-bsf=extract_extradata
    --enable-bsf=aac_adtstoasc
    --enable-muxer=mp4
    --enable-muxer=mov
    --enable-swresample
)

base_flags=(
    --disable-programs
    --disable-doc
    --disable-debug
    --disable-shared
    --enable-static
    --enable-pic
    --disable-gpl
    --disable-nonfree
    --disable-autodetect
    --disable-avdevice
    --disable-avfilter
    --disable-postproc
    --enable-videotoolbox
    --enable-audiotoolbox
    --enable-cross-compile
    --target-os=darwin
)

slice_configure() {
    local arch="$1" sdk="$2" triple="$3" min_flag="$4"
    local sdk_path cflags
    sdk_path="$(xcrun --sdk "$sdk" --show-sdk-path)"
    cflags="-arch $arch -target $triple -isysroot $sdk_path $min_flag"

    echo "Configuring FFmpeg for $arch/$sdk ($triple)"
    "$source_dir/configure" \
        --prefix="$PWD/install" \
        --arch="$arch" \
        --cc="$(xcrun --sdk "$sdk" --find clang)" \
        --ar="$(xcrun --sdk "$sdk" --find ar)" \
        --ranlib="$(xcrun --sdk "$sdk" --find ranlib)" \
        --extra-cflags="$cflags" \
        --extra-ldflags="-arch $arch -target $triple -isysroot $sdk_path $min_flag" \
        "${base_flags[@]}" \
        "${components[@]}"
    echo "Finished configuring FFmpeg for $arch/$sdk"
}

build_slice() {
    local name="$1" arch="$2" sdk="$3" triple="$4" min_flag="$5"
    local slice_dir="$work_dir/build/$name"

    if [[ -f "$slice_dir/install/lib/libavformat.a" && "${FLUXA_FORCE_FFMPEG_BUILD:-0}" != "1" ]]; then
        return
    fi

    rm -rf "$slice_dir"
    mkdir -p "$slice_dir"
    pushd "$slice_dir" >/dev/null
    slice_configure "$arch" "$sdk" "$triple" "$min_flag"
    # FFmpeg has a large C translation-unit fan-out; keeping the default
    # deliberately modest avoids runner memory pressure. Override this for a
    # larger self-hosted builder with FLUXA_FFMPEG_JOBS.
    make -j"${FLUXA_FFMPEG_JOBS:-2}"
    # The full install target also installs docs, examples and pkg-config
    # metadata that are not part of the XCFramework. Those targets can fail
    # independently after the libraries were built; keep the artifact limited
    # to the libraries and public headers consumed by FluxaPlayerKit.
    echo "Installing FFmpeg libraries for $name"
    make install-libs
    echo "Installing FFmpeg headers for $name"
    make install-headers
    echo "Finished FFmpeg install for $name"
    popd >/dev/null

    merge_slice "$name"
}

merge_slice() {
    local name="$1"
    local lib_dir="$work_dir/build/$name/install/lib"
    local output="$work_dir/build/$name/libCFFmpeg.a"
    echo "Merging FFmpeg libraries for $name"
    if ! libtool -static -no_warning_for_no_symbols -o "$output" \
        "$lib_dir/libavformat.a" \
        "$lib_dir/libavcodec.a" \
        "$lib_dir/libswresample.a" \
        "$lib_dir/libavutil.a"; then
        echo "libtool failed while merging $name" >&2
        return 1
    fi
    echo "Finished FFmpeg merge for $name"
}

fetch_source() {
    if [[ -d "$source_dir" ]]; then
        return
    fi
    mkdir -p "$work_dir"
    local tarball="$work_dir/ffmpeg-$ffmpeg_version.tar.xz"
    curl -fsSL -o "$tarball" "https://ffmpeg.org/releases/ffmpeg-$ffmpeg_version.tar.xz"
    tar -xf "$tarball" -C "$work_dir"
}

stage_headers() {
    local reference="$work_dir/build/ios-device/install/include"
    rm -rf "$headers_dir"
    mkdir -p "$headers_dir"
    cp -R "$reference/"* "$headers_dir/"
    cat > "$headers_dir/module.modulemap" <<'MODULEMAP'
module CFFmpeg {
    header "libavformat/avformat.h"
    header "libavcodec/avcodec.h"
    header "libavutil/avutil.h"
    header "libavutil/imgutils.h"
    header "libavutil/opt.h"
    header "libavutil/channel_layout.h"
    header "libswresample/swresample.h"
    export *
}
MODULEMAP
}

fetch_source

build_slice ios-device arm64 iphoneos arm64-apple-ios "-mios-version-min=$ios_min"
build_slice ios-sim-arm64 arm64 iphonesimulator arm64-apple-ios-simulator "-mios-simulator-version-min=$ios_min"
build_slice ios-sim-x86_64 x86_64 iphonesimulator x86_64-apple-ios-simulator "-mios-simulator-version-min=$ios_min"
build_slice tvos-device arm64 appletvos arm64-apple-tvos "-mtvos-version-min=$tvos_min"
build_slice tvos-sim arm64 appletvsimulator arm64-apple-tvos-simulator "-mtvos-simulator-version-min=$tvos_min"
build_slice macos-arm64 arm64 macosx arm64-apple-macos "-mmacosx-version-min=$macos_min"
build_slice macos-x86_64 x86_64 macosx x86_64-apple-macos "-mmacosx-version-min=$macos_min"

stage_headers

lipo -create \
    "$work_dir/build/ios-sim-arm64/libCFFmpeg.a" \
    "$work_dir/build/ios-sim-x86_64/libCFFmpeg.a" \
    -output "$work_dir/build/libCFFmpeg-ios-simulator.a"
lipo -create \
    "$work_dir/build/macos-arm64/libCFFmpeg.a" \
    "$work_dir/build/macos-x86_64/libCFFmpeg.a" \
    -output "$work_dir/build/libCFFmpeg-macos.a"

mkdir -p "$vendor_dir"
rm -rf "$xcframework"
xcodebuild -create-xcframework \
    -library "$work_dir/build/ios-device/libCFFmpeg.a" -headers "$headers_dir" \
    -library "$work_dir/build/libCFFmpeg-ios-simulator.a" -headers "$headers_dir" \
    -library "$work_dir/build/tvos-device/libCFFmpeg.a" -headers "$headers_dir" \
    -library "$work_dir/build/tvos-sim/libCFFmpeg.a" -headers "$headers_dir" \
    -library "$work_dir/build/libCFFmpeg-macos.a" -headers "$headers_dir" \
    -output "$xcframework" >/dev/null

echo "CFFmpeg.xcframework -> $xcframework"
