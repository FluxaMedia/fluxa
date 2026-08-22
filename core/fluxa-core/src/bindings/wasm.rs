use wasm_bindgen::prelude::*;

use crate::FluxaCore;

#[wasm_bindgen]
pub fn core_invoke(method: &str, args_json: &str) -> String {
    crate::ffi::core_invoke(method, args_json)
}

#[wasm_bindgen]
pub fn fluxa_core_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

// Mirrors the engine_init/engine_dispatch/engine_complete_effect/engine_snapshot
// Tauri commands in apps/desktop/src-tauri/src/lib.rs so the JS engine.ts glue
// is identical between desktop and web. Handles fit in f64 (JS has no u64).
#[wasm_bindgen]
pub fn engine_init(initial_json: &str) -> f64 {
    FluxaCore::create_headless_engine(initial_json) as f64
}

#[wasm_bindgen]
pub fn engine_dispatch(handle: f64, action_json: &str) -> Option<String> {
    FluxaCore::headless_engine_dispatch_json(handle as u64, action_json)
}

#[wasm_bindgen]
pub fn engine_complete_effect(handle: f64, result_json: &str) -> Option<String> {
    FluxaCore::headless_engine_complete_effect_json(handle as u64, result_json)
}

#[wasm_bindgen]
pub fn engine_snapshot(handle: f64) -> Option<String> {
    FluxaCore::headless_engine_snapshot_json(handle as u64)
}

/// Remuxes a complete in-memory MKV file into WebM (bitstream copy, no
/// re-encode) for MediaSource Extensions. For anything but small files,
/// prefer `IncrementalMkvRemuxer` below — this whole-buffer variant needs
/// the entire source downloaded first.
#[wasm_bindgen]
pub fn remux_mkv_to_webm(mkv_bytes: &[u8]) -> Result<Vec<u8>, JsValue> {
    crate::media_demux::remux_mkv_to_webm(mkv_bytes).map_err(|e| JsValue::from_str(&e))
}

/// Streaming MKV -> WebM remuxer: feed it chunks as they arrive (e.g. from a
/// `fetch()` `ReadableStream`) via `push`, and append whatever bytes come
/// back to a `SourceBuffer` right away — playback can start before the
/// whole source has downloaded. Call `finish()` once at end-of-stream to
/// flush the final Cluster.
#[wasm_bindgen]
pub struct IncrementalMkvRemuxer {
    inner: crate::media_demux::IncrementalRemuxSession,
}

#[wasm_bindgen]
impl IncrementalMkvRemuxer {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self { inner: crate::media_demux::IncrementalRemuxSession::new() }
    }

    pub fn push(&mut self, chunk: &[u8]) -> Vec<u8> {
        self.inner.push(chunk)
    }

    pub fn finish(&mut self) -> Vec<u8> {
        self.inner.finish()
    }
}

impl Default for IncrementalMkvRemuxer {
    fn default() -> Self {
        Self::new()
    }
}
