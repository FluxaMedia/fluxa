#![no_main]

use fluxa_core::fuzz_targets::percent_decode_component;
use libfuzzer_sys::fuzz_target;

// Targets the exact bug class found by hand earlier: a '%' next to a
// multi-byte UTF-8 character panicking on a mid-character slice bound.
fuzz_target!(|data: &[u8]| {
    let Ok(text) = std::str::from_utf8(data) else { return };
    let _ = percent_decode_component(text);
});
