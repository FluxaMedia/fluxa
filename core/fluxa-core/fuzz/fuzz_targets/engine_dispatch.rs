#![no_main]

use fluxa_core::fuzz_targets::{
    create_headless_engine, destroy_headless_engine, headless_engine_complete_effect_json,
    headless_engine_dispatch_json,
};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let split = data.iter().position(|&b| b == b'\n').unwrap_or(data.len());
    let (action_bytes, rest) = data.split_at(split);
    let effect_bytes = rest.strip_prefix(b"\n").unwrap_or(rest);

    let handle = create_headless_engine("{}");

    if let Ok(action_json) = std::str::from_utf8(action_bytes) {
        let _ = headless_engine_dispatch_json(handle, action_json);
    }
    if let Ok(effect_json) = std::str::from_utf8(effect_bytes) {
        let _ = headless_engine_complete_effect_json(handle, effect_json);
    }

    destroy_headless_engine(handle);
});
