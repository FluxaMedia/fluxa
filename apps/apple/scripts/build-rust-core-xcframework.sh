#!/usr/bin/env bash
set -euo pipefail

if [[ "${FLUXA_SKIP_RUST_CORE_BUILD:-0}" == "1" ]]; then
    exit 0
fi

project_dir="$(cd "$(dirname "$0")/.." && pwd)"
native_dir="$(cd "$project_dir/../../native" && pwd)"
output_dir="$project_dir/Generated"
profile="release"
[[ "${CONFIGURATION:-Debug}" == "Release" ]] || profile="debug"

sdk_for() {
    case "$1" in
        aarch64-apple-ios) echo iphoneos ;;
        aarch64-apple-ios-sim | x86_64-apple-ios) echo iphonesimulator ;;
        aarch64-apple-tvos) echo appletvos ;;
        aarch64-apple-tvos-sim) echo appletvsimulator ;;
    esac
}

build() {
    local target="$1" sdk_path
    sdk_path="$(xcrun --sdk "$(sdk_for "$target")" --show-sdk-path)"
    local cmd=(cargo build -p fluxa-apple-renderer --target "$target")
    if [[ "$target" == *tvos* ]]; then
        cmd=(cargo "+${FLUXA_TVOS_TOOLCHAIN:-nightly}" build -Zbuild-std=std,panic_abort -p fluxa-apple-renderer --target "$target")
    fi
    [[ "$profile" == "release" ]] && cmd+=(--release)
    (cd "$native_dir" && env \
        "IPHONEOS_DEPLOYMENT_TARGET=${IPHONEOS_DEPLOYMENT_TARGET:-18.5}" \
        "TVOS_DEPLOYMENT_TARGET=${TVOS_DEPLOYMENT_TARGET:-17.0}" \
        "SDKROOT=$sdk_path" \
        "${cmd[@]}")
}

targets=(aarch64-apple-ios aarch64-apple-ios-sim x86_64-apple-ios aarch64-apple-tvos aarch64-apple-tvos-sim)
for target in "${targets[@]}"; do
    build "$target"
done

lib() { echo "$native_dir/target/$1/$profile/libfluxa_apple_renderer.a"; }
mkdir -p "$output_dir"
lipo -create "$(lib aarch64-apple-ios-sim)" "$(lib x86_64-apple-ios)" -output "$output_dir/libfluxa_apple_renderer-ios-simulator.a"
cp "$(lib aarch64-apple-ios)" "$output_dir/libfluxa_apple_renderer-ios.a"
cp "$(lib aarch64-apple-tvos)" "$output_dir/libfluxa_apple_renderer-tvos.a"
cp "$(lib aarch64-apple-tvos-sim)" "$output_dir/libfluxa_apple_renderer-tvos-simulator.a"
