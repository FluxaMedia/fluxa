# Building

## Features

| Feature | What it enables |
|---|---|
| `native` (default) | Full Android/native surface: JNI bindings, Dolby Vision RPU, UniFFI Kotlin bindings |
| `full-api` | Complete domain/helper API surface used by JNI, `core_invoke`, UniFFI, WASM, and desktop |
| `desktop` | Named alias for the full desktop/Tauri-compatible API surface |
| `streaming-shared` | Minimal `FluxaCore` stream policy facade used by `fluxa-streaming-engine` |
| `uniffi-bindings` | UniFFI runtime support (pulled in by `native`) |
| `uniffi-cli` | Adds the `uniffi-bindgen` binary for generating Kotlin/Swift source |
| `wasm` | `wasm-bindgen` exports for webOS |
| `fuzzing` | Enables fuzz targets |

## Common commands

```bash
# default build (native features — what Android uses)
cargo build

# run the test suite (~190 tests, fast)
cargo test --lib

# check the webOS/WASM path compiles
cargo check --no-default-features --features wasm

# check the narrow surface used by fluxa-streaming-engine
cargo check --no-default-features --features streaming-shared

# generate UniFFI Kotlin bindings
cargo run --bin uniffi-bindgen --features uniffi-cli -- generate \
    --library target/debug/libfluxa_core.so \
    --language kotlin \
    --out-dir <output-dir>

# release build (LTO + strip, used for Android .so)
cargo build --release
```

## Android cross-compilation

Install targets first:

```bash
rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android
```

Then build for each ABI, pointing to the Android NDK toolchain:

```bash
cargo build --release --target aarch64-linux-android
cargo build --release --target armv7-linux-androideabi
cargo build --release --target x86_64-linux-android
```

The Android project (`apps/android`) picks up the resulting `.so` files from `target/<abi>/release/libfluxa_core.so`.

## Partial API builds

Non-native consumers intentionally compile partial API surfaces: desktop uses direct Rust/Tauri calls plus `core_invoke`, WASM exposes a small JS bridge, and `fluxa-streaming-engine` only needs stream policy helpers. These builds suppress dead-code noise from API functions that are only reachable through Android/JNI.

The default `native` build keeps normal dead-code checking because it compiles the exhaustive Android JNI surface.

## Panic policy

The release profile keeps `panic = "unwind"`. The JNI boundary in `bindings/jni.rs` and `ffi.rs::core_invoke` both use `catch_unwind` so a panic in domain logic returns a safe null/error instead of aborting the host process. Switching to `panic = "abort"` would silently defeat this.

## fluxa-streaming-engine

The companion crate at `fluxa-streaming-engine/` builds independently:

```bash
cd fluxa-streaming-engine
cargo build                          # native features (tokio, axum, librqbit, jni)
cargo build --bin torrent_serve      # local torrent HTTP proxy
cargo build --bin companion_server   # fluxa-web's local companion process
```

Its dependency on `fluxa_core` enables only `streaming-shared`, so streaming builds do not compile the full Android/desktop helper surface just to call stream playback and torrent runtime planning.
