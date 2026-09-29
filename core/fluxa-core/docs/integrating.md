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

1. `Engine` (`dispatch`, `complete`) for state and effects.

2. `fluxa_core::ffi::core_invoke(method, args_json)` for plan calls that have no engine action yet.

Stream and torrent planning for the streaming engine is called directly as `fluxa_core::player::stream_policy::*`.

### Adding a new capability for desktop

Prefer an engine action and effect. If the logic is not wired yet, add a route arm to the `routes` folder of the matching domain (or `src/services/<name>/routes.rs`), register it in `src/ffi/methods.rs`, and add the name to `tests/wire/core_invoke_methods.txt`.

---

## webOS

**How it links:** Via `wasm-bindgen` exports in `src/bindings/wasm.rs`.

Build with `--no-default-features --features wasm` using `wasm-pack` or `cargo build --target wasm32-unknown-unknown`. The WASM module exposes the same `core_invoke` entry point.

---

## Wire contract

The JSON field names and nesting in anything that crosses `dispatch` / `completeEffect` / `core_invoke` are read by platform code in the consuming repos. Do not rename wire fields without coordinating with the consumer. Internal Rust refactors are safe as long as `#[serde(rename_all = "camelCase")]` output is unchanged.
