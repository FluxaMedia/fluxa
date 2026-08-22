#![no_main]

use fluxa_core::fuzz_targets::parse_manifest;
use libfuzzer_sys::fuzz_target;

// Manifest JSON comes straight from whatever server an addon's transport URL
// points at — arbitrary, possibly adversarial, third-party-controlled input.
fuzz_target!(|data: &[u8]| {
    let Ok(body) = std::str::from_utf8(data) else { return };
    let _ = parse_manifest(body, "https://addon.example/manifest.json", "Unknown Addon");
});
