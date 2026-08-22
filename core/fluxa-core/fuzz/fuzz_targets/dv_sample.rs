#![no_main]

use fluxa_core::fuzz_targets::fuzz_process_sample;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    fuzz_process_sample(data);
});
