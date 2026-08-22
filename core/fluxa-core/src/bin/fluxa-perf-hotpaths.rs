use std::hint::black_box;
use std::time::Instant;

fn main() {
    let cases = [
        (
            "streamPlaybackInfo",
            r#"{"stream":{"url":"https://example.com/a.mp4"}}"#,
        ),
        ("normalizeContentType", r#"{"value":"series"}"#),
        ("stableFeedPart", r#"{"value":"Trending Now"}"#),
        (
            "playerTrackState",
            r#"{"streamsJson":"[]","profileJson":"{}"}"#,
        ),
    ];
    let iterations = std::env::var("FLUXA_PERF_ITERATIONS")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(100_000);
    let start = Instant::now();
    let mut bytes = 0usize;
    for index in 0..iterations {
        let (method, args) = cases[index % cases.len()];
        bytes += black_box(fluxa_core::ffi::core_invoke(method, args)).len();
    }
    let elapsed = start.elapsed();
    println!(
        "iterations={iterations} bytes={bytes} elapsed_us={} ns_per_call={}",
        elapsed.as_micros(),
        elapsed.as_nanos() / iterations as u128
    );
}
