# Building

## Features

| Feature | What it enables |
|---|---|
| `native` (default) | Full native surface: Dolby Vision RPU and the JS plugin engine |
| `full-api` | Complete domain/helper API surface used by `core_invoke`, WASM, and the Rust renderers |
| `desktop` | Named alias for the full desktop API surface |
| `streaming-shared` | Minimal `FluxaCore` stream policy facade used by `fluxa-streaming-engine` |
| `ios` | Full API and JS plugin engine for the Apple renderer |
| `wasm` | `wasm-bindgen` exports for webOS |
| `fuzzing` | Enables fuzz targets |

## Common commands

```bash
# default build (native features)
cargo build

# run the test suite (~190 tests, fast)
cargo test --lib

# check the webOS/WASM path compiles
cargo check --no-default-features --features wasm

# check the narrow surface used by fluxa-streaming-engine
cargo check --no-default-features --features streaming-shared

# release build (LTO + strip)
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

Non-native consumers intentionally compile partial API surfaces: desktop uses direct Rust calls plus `core_invoke`, WASM exposes a small JS bridge, and `fluxa-streaming-engine` only needs stream policy helpers. These builds suppress dead-code noise from API functions that only the full native surface reaches.

The default `native` build keeps normal dead-code checking because it compiles the full API surface.

## Panic policy

The release profile keeps `panic = "unwind"`. `ffi.rs::core_invoke` uses `catch_unwind` so a panic in domain logic returns a safe null/error instead of aborting the host process. Switching to `panic = "abort"` would silently defeat this.

## fluxa-streaming-engine

The companion crate at `fluxa-streaming-engine/` builds independently:

```bash
cd fluxa-streaming-engine
cargo build                          # native features (tokio, axum, librqbit)
cargo build --bin torrent_serve      # local torrent HTTP proxy
cargo build --bin companion_server   # fluxa-web's local companion process
```

Its dependency on `fluxa_core` enables only `streaming-shared`, so streaming builds do not compile the full Android/desktop helper surface just to call stream playback and torrent runtime planning.
