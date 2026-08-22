# Platform integration guide

fluxa-core ships as a compiled native library. Each platform links against it differently.

## Android

**How it links:** Two paths, both real.

**Primary path — JNI (`bindings/jni.rs`):** ~157 hand-written `extern "system"` functions called directly from `FluxaCoreNative.kt`. This covers most of the crate's surface area, including the headless engine, stream selection, DV policy, calendar, watchlist, and more.

**Secondary path — UniFFI:** A small subset of functions in `bindings/uniffi.rs` marked `#[uniffi::export]` generate Kotlin bindings (`FluxaCoreUniFfi.kt`). This covers headless engine lifecycle, `coreCapabilities`, and version.

### Setup

1. Build `.so` files for each ABI (see [building.md](building.md)).
2. Place them in `src/main/jniLibs/<abi>/libfluxa_core.so` in the Android project.
3. `System.loadLibrary("fluxa_core")` in your `Application` class or via the generated UniFFI loader.
4. Call `FluxaCoreNative.createHeadlessEngine(initialJson)` to get a handle, then dispatch actions and complete effects through `FluxaCoreNative.*` methods.

### Adding a new capability for Android

Add a new `string_fn!` (or equivalent) entry to `src/bindings/jni.rs`. If the same capability also needs to reach desktop or Swift, add a route to `src/ffi.rs::route_*` too. There is no shared registration — you wire each platform separately.

---

## Desktop (Linux / macOS / Windows)

**How it links:** Plain Rust path dependency in `apps/desktop` (a Tauri app):

```toml
fluxa_core = { path = "../../../core/fluxa-core" }
```

No FFI marshaling — it calls Rust functions directly.

**Two call sites:**

1. `FluxaCore::*` methods (in `src/core_api.rs`) for the 8 things desktop calls without going through the dispatcher: headless engine lifecycle, `stream_playback_info_json`, `torrent_runtime_info_json`, `player_buffer_targets_json`, `offline_download_plan_json`.

2. `fluxa_core::ffi::core_invoke(method, args_json)` for everything else — the full ~115-method dispatcher.

### Adding a new capability for desktop

If desktop needs it via `core_invoke`: add a route arm to the appropriate `route_*` function in `src/ffi.rs`.

If desktop needs a direct `FluxaCore` method (unusual — only do this if `core_invoke` is genuinely not suitable): add it to `src/core_api.rs` and confirm there's a real call site in `apps/desktop/src-tauri/src/` before adding.

---

## iOS / tvOS

**How it links:** Via UniFFI-generated Swift bindings. `src/bindings/uniffi.rs::core_invoke` is the intended entry point — Swift calls it instead of binding each helper individually.

Build the crate with `--features uniffi-bindings`, generate Swift source with `uniffi-bindgen`, and link the resulting `.xcframework` into the Xcode project.

---

## webOS

**How it links:** Via `wasm-bindgen` exports in `src/bindings/wasm.rs`.

Build with `--no-default-features --features wasm` using `wasm-pack` or `cargo build --target wasm32-unknown-unknown`. The WASM module exposes the same `core_invoke` entry point.

---

## Wire contract

The JSON field names and nesting in anything that crosses `dispatch` / `completeEffect` / `core_invoke` are read by platform code in the consuming repos. Do not rename wire fields without coordinating with the consumer. Internal Rust refactors are safe as long as `#[serde(rename_all = "camelCase")]` output is unchanged.
