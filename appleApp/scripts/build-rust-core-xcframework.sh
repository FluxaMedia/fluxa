#!/usr/bin/env bash
set -euo pipefail

project_dir="$(cd "$(dirname "$0")/../.." && pwd)"
rust_dir="$(cd "$project_dir/../fluxa-core" && pwd)"
output_dir="$project_dir/appleApp/Generated"
headers_dir="$output_dir/FluxaRustCoreFFI"
profile="${CONFIGURATION:-Debug}"

if [[ "$profile" == "Release" ]]; then
    cargo_profile="release"
else
    cargo_profile="debug"
fi

build_rust_core() {
    if [[ "$profile" == "Release" ]]; then
        cargo build --no-default-features --features ios "$@" --release
    else
        cargo build --no-default-features --features ios "$@"
    fi
}

build_streaming_engine() {
    local ios_deployment_target="${IPHONEOS_DEPLOYMENT_TARGET:-18.5}"

    if [[ "$profile" == "Release" ]]; then
        IPHONEOS_DEPLOYMENT_TARGET="$ios_deployment_target" cargo build -p fluxa_streaming_engine --no-default-features --features apple "$@" --release
    else
        IPHONEOS_DEPLOYMENT_TARGET="$ios_deployment_target" cargo build -p fluxa_streaming_engine --no-default-features --features apple "$@"
    fi
}

should_build_streaming_engine() {
    [[ "${FLUXA_BUILD_STREAMING_ENGINE:-0}" == "1" ]] || [[ "${PLATFORM_NAME:-}" == "iphoneos" ]] || [[ "${PLATFORM_NAME:-}" == "iphonesimulator" ]]
}

targets=(
    aarch64-apple-ios
    aarch64-apple-ios-sim
    x86_64-apple-ios
    aarch64-apple-tvos
    aarch64-apple-tvos-sim
)

installed_targets="$(rustup target list --installed)"
for target in "${targets[@]}"; do
    if ! grep -qx "$target" <<< "$installed_targets"; then
        echo "Missing Rust target: $target"
        echo "Install it with: rustup target add $target"
        exit 1
    fi
done

mkdir -p "$output_dir" "$headers_dir"
rm -rf "$output_dir/FluxaRustCore.xcframework"

pushd "$rust_dir" >/dev/null
build_rust_core
cargo run --no-default-features --features uniffi-cli --bin uniffi-bindgen generate \
    --library "target/$cargo_profile/libfluxa_core.dylib" \
    --language swift \
    --config uniffi.toml \
    --out-dir "$output_dir"

for target in "${targets[@]}"; do
    build_rust_core --target "$target"
done

if should_build_streaming_engine; then
    for target in aarch64-apple-ios aarch64-apple-ios-sim x86_64-apple-ios; do
        build_streaming_engine --target "$target"
    done
fi

lipo -create \
    "target/aarch64-apple-ios-sim/$cargo_profile/libfluxa_core.a" \
    "target/x86_64-apple-ios/$cargo_profile/libfluxa_core.a" \
    -output "$output_dir/libfluxa_core-ios-simulator.a"
cp "target/aarch64-apple-ios/$cargo_profile/libfluxa_core.a" "$output_dir/libfluxa_core-ios.a"
cp "target/aarch64-apple-tvos/$cargo_profile/libfluxa_core.a" "$output_dir/libfluxa_core-tvos.a"
cp "target/aarch64-apple-tvos-sim/$cargo_profile/libfluxa_core.a" "$output_dir/libfluxa_core-tvos-simulator.a"
if should_build_streaming_engine; then
    lipo -create \
        "target/aarch64-apple-ios-sim/$cargo_profile/libfluxa_streaming_engine.a" \
        "target/x86_64-apple-ios/$cargo_profile/libfluxa_streaming_engine.a" \
        -output "$output_dir/libfluxa_streaming_engine-ios-simulator.a"
    cp "target/aarch64-apple-ios/$cargo_profile/libfluxa_streaming_engine.a" "$output_dir/libfluxa_streaming_engine-ios.a"
fi
cp "$output_dir/FluxaRustCoreFFI.h" "$headers_dir/FluxaRustCoreFFI.h"
cp "$output_dir/FluxaRustCoreFFI.modulemap" "$headers_dir/module.modulemap"

xcodebuild -create-xcframework \
    -library "target/aarch64-apple-ios/$cargo_profile/libfluxa_core.a" -headers "$headers_dir" \
    -library "$output_dir/libfluxa_core-ios-simulator.a" -headers "$headers_dir" \
    -library "target/aarch64-apple-tvos/$cargo_profile/libfluxa_core.a" -headers "$headers_dir" \
    -library "target/aarch64-apple-tvos-sim/$cargo_profile/libfluxa_core.a" -headers "$headers_dir" \
    -output "$output_dir/FluxaRustCore.xcframework"
popd >/dev/null
