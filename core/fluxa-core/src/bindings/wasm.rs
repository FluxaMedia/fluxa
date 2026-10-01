use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn core_invoke(method: &str, args_json: &str) -> String {
    crate::ffi::core_invoke(method, args_json)
}

#[wasm_bindgen]
pub fn fluxa_core_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}
