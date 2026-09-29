# Building

## Features

| Feature | What it enables |
|---|---|
| `js-engine` (default) | QuickJS for the YouTube trailer cipher |
| `wasm` | `wasm-bindgen` exports for webOS |
| `fuzzing` | Enables fuzz targets |

Workspace crates:

| Crate | Purpose |
|---|---|
| `fluxa_core` | Domain logic, engine, string router |
| `fluxa_media` | Matroska demux, fMP4/WebM mux |
| `fluxa_plugin_runtime` | QuickJS plugin sandbox and its bridges |
| `fluxa_streaming_engine` | Torrent and HTTP proxy |

## Common commands

```bash
cargo build
cargo test --workspace --lib
cargo check --no-default-features --features wasm
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

## Panic policy

The release profile keeps `panic = "unwind"`. `ffi::core_invoke` uses `catch_unwind` so a panic in domain logic returns a safe null/error instead of aborting the host process. Switching to `panic = "abort"` would silently defeat this.

## fluxa-streaming-engine

The companion crate at `fluxa-streaming-engine/` builds independently:

```bash
cd fluxa-streaming-engine
cargo build                          # native features (tokio, axum, librqbit)
cargo build --bin torrent_serve      # local torrent HTTP proxy
cargo build --bin companion_server   # fluxa-web's local companion process
```

It calls `fluxa_core::player::stream_policy` for stream playback and torrent runtime planning.
