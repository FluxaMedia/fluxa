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

// Handles fit in f64 (JS has no u64).
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
