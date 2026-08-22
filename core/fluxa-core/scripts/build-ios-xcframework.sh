#!/usr/bin/env bash
set -euo pipefail

if [[ "$(uname)" != "Darwin" ]]; then
    echo "error: this script requires macOS with Xcode command line tools installed" >&2
    exit 1
fi

if ! command -v xcodebuild >/dev/null 2>&1; then
    echo "error: xcodebuild not found — install Xcode" >&2
    exit 1
fi

cd "$(dirname "$0")/.."

TARGETS=(aarch64-apple-ios aarch64-apple-ios-sim)

for target in "${TARGETS[@]}"; do
    rustup target add "$target"
done

export LIBCLANG_PATH="$(xcode-select -p)/Toolchains/XcodeDefault.xctoolchain/usr/lib"

apple_sdk_for_target() {
    case "$1" in
        aarch64-apple-ios) echo "iphoneos" ;;
        aarch64-apple-ios-sim) echo "iphonesimulator" ;;
        *) echo "error: no SDK mapping for target $1" >&2; exit 1 ;;
    esac
}

apple_clang_triple_for_target() {
    case "$1" in
        aarch64-apple-ios) echo "arm64-apple-ios" ;;
        aarch64-apple-ios-sim) echo "arm64-apple-ios-simulator" ;;
        *) echo "error: no clang triple mapping for target $1" >&2; exit 1 ;;
    esac
}

for target in "${TARGETS[@]}"; do
    sdk="$(apple_sdk_for_target "$target")"
    sdk_path="$(xcrun --sdk "$sdk" --show-sdk-path)"
    clang_triple="$(apple_clang_triple_for_target "$target")"
    BINDGEN_EXTRA_CLANG_ARGS="--target=$clang_triple --sysroot=$sdk_path -isysroot $sdk_path" \
        cargo build --release --target "$target" --no-default-features --features ios --lib
done

BINDINGS_DIR="target/uniffi-swift"
rm -rf "$BINDINGS_DIR"
mkdir -p "$BINDINGS_DIR"

cargo run --features uniffi-cli --bin uniffi-bindgen generate \
    --library "target/aarch64-apple-ios/release/libfluxa_core.a" \
    --language swift \
    --config uniffi.toml \
    --out-dir "$BINDINGS_DIR"

HEADERS_DIR="$BINDINGS_DIR/Headers"
mkdir -p "$HEADERS_DIR"
mv "$BINDINGS_DIR"/*.h "$HEADERS_DIR"/
mv "$BINDINGS_DIR"/*.modulemap "$HEADERS_DIR/module.modulemap"

XCFRAMEWORK_OUT="build/ios/FluxaCore.xcframework"
rm -rf "build/ios"
mkdir -p "build/ios"

xcodebuild -create-xcframework \
    -library "target/aarch64-apple-ios/release/libfluxa_core.a" -headers "$HEADERS_DIR" \
    -library "target/aarch64-apple-ios-sim/release/libfluxa_core.a" -headers "$HEADERS_DIR" \
    -output "$XCFRAMEWORK_OUT"

echo "xcframework: $XCFRAMEWORK_OUT"
echo "Swift bindings: $BINDINGS_DIR"/*.swift
