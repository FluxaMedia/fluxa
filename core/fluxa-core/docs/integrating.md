# Platform integration guide

fluxa-core ships as a compiled native library. Each platform links against it differently.

## Android, iOS, tvOS

**How it links:** Plain Rust dependency of the platform renderer crates (`native/fluxa-android-renderer`, `native/fluxa-apple-renderer`). The Kotlin and Swift hosts only talk to the renderer, never to fluxa-core directly.

---

## Desktop (Linux / macOS / Windows)

**How it links:** Plain Rust path dependency from the native workspace (`native/fluxa-app`, `native/fluxa-effects`, `native/fluxa-host`):

```toml
fluxa_core = { path = "../../core/fluxa-core" }
```

No FFI marshaling — it calls Rust functions directly.

**Two call sites:**

1. `FluxaCore::*` methods (in `src/core_api.rs`) for the 8 things desktop calls without going through the dispatcher: headless engine lifecycle, `stream_playback_info_json`, `torrent_runtime_info_json`, `player_buffer_targets_json`, `offline_download_plan_json`.

2. `fluxa_core::ffi::core_invoke(method, args_json)` for everything else — the full ~115-method dispatcher.

### Adding a new capability for desktop

If desktop needs it via `core_invoke`: add a route arm to the appropriate `route_*` function in `src/ffi.rs`.

If desktop needs a direct `FluxaCore` method (unusual — only do this if `core_invoke` is genuinely not suitable): add it to `src/core_api.rs` and confirm there's a real call site in `native/` before adding.

---

## webOS

**How it links:** Via `wasm-bindgen` exports in `src/bindings/wasm.rs`.

Build with `--no-default-features --features wasm` using `wasm-pack` or `cargo build --target wasm32-unknown-unknown`. The WASM module exposes the same `core_invoke` entry point.

---

## Wire contract

The JSON field names and nesting in anything that crosses `dispatch` / `completeEffect` / `core_invoke` are read by platform code in the consuming repos. Do not rename wire fields without coordinating with the consumer. Internal Rust refactors are safe as long as `#[serde(rename_all = "camelCase")]` output is unchanged.
