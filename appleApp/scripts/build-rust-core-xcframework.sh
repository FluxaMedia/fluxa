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

apple_sdk_for_rust_target() {
    case "$1" in
        aarch64-apple-ios) echo "iphoneos" ;;
        aarch64-apple-ios-sim | x86_64-apple-ios) echo "iphonesimulator" ;;
        aarch64-apple-tvos) echo "appletvos" ;;
        aarch64-apple-tvos-sim) echo "appletvsimulator" ;;
        *) echo "" ;;
    esac
}

apple_clang_triple_for_rust_target() {
    case "$1" in
        aarch64-apple-ios) echo "arm64-apple-ios" ;;
        aarch64-apple-ios-sim) echo "arm64-apple-ios-simulator" ;;
        x86_64-apple-ios) echo "x86_64-apple-ios-simulator" ;;
        aarch64-apple-tvos) echo "arm64-apple-tvos" ;;
        aarch64-apple-tvos-sim) echo "arm64-apple-tvos-simulator" ;;
        *) echo "" ;;
    esac
}

build_rust_core() {
    local rust_target=""
    local prev_was_target_flag=0
    for arg in "$@"; do
        if [[ "$prev_was_target_flag" == "1" ]]; then
            rust_target="$arg"
            break
        fi
        [[ "$arg" == "--target" ]] && prev_was_target_flag=1 || prev_was_target_flag=0
    done

    local bindgen_env=()
    local deployment_env=()
    if [[ -n "$rust_target" ]]; then
        local sdk clang_triple sdk_path
        sdk="$(apple_sdk_for_rust_target "$rust_target")"
        clang_triple="$(apple_clang_triple_for_rust_target "$rust_target")"
        if [[ -n "$sdk" && -n "$clang_triple" ]]; then
            sdk_path="$(xcrun --sdk "$sdk" --show-sdk-path)"
            bindgen_env=(
                "LIBCLANG_PATH=$(xcode-select -p)/Toolchains/XcodeDefault.xctoolchain/usr/lib"
                "BINDGEN_EXTRA_CLANG_ARGS=--target=$clang_triple --sysroot=$sdk_path -isysroot $sdk_path"
            )
        fi
        case "$sdk" in
            iphoneos | iphonesimulator)
                deployment_env=("IPHONEOS_DEPLOYMENT_TARGET=${IPHONEOS_DEPLOYMENT_TARGET:-18.5}")
                ;;
            appletvos | appletvsimulator)
                deployment_env=("TVOS_DEPLOYMENT_TARGET=${TVOS_DEPLOYMENT_TARGET:-18.5}")
                ;;
        esac
    fi

    local cargo_cmd=(cargo build --no-default-features --features ios "$@")
    [[ "$profile" == "Release" ]] && cargo_cmd+=(--release)

    local env_args=()
    [[ ${#bindgen_env[@]} -gt 0 ]] && env_args+=("${bindgen_env[@]}")
    [[ ${#deployment_env[@]} -gt 0 ]] && env_args+=("${deployment_env[@]}")
    if [[ ${#env_args[@]} -gt 0 ]]; then
        env "${env_args[@]}" "${cargo_cmd[@]}"
    else
        "${cargo_cmd[@]}"
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
