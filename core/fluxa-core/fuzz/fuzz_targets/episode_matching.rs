#![no_main]

use fluxa_core::fuzz_targets::{contains_compact_episode, contains_spaced_episode, parse_episode_locator};
use libfuzzer_sys::fuzz_target;

// Targets the exact bug class found by hand earlier: byte-vs-char-boundary
// panics in the SxxExx / "Season X Episode Y" scanners, which run on
// addon-provided stream titles and filenames.
fuzz_target!(|data: &[u8]| {
    let Ok(text) = std::str::from_utf8(data) else { return };
    let _ = parse_episode_locator(text);
    let _ = contains_compact_episode(text, 1, 2);
    let _ = contains_spaced_episode(text, 1, 2);
});
