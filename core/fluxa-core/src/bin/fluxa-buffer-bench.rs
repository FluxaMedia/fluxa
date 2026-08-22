use std::hint::black_box;
use std::time::Instant;

fn main() {
    let chunk_size = std::env::var("FLUXA_BUFFER_SIZE")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(65_536);
    let total = std::env::var("FLUXA_BUFFER_BYTES")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(64 * 1024 * 1024);
    let source = (0..total).map(|index| index as u8).collect::<Vec<_>>();
    let start = Instant::now();
    let mut checksum = 0u8;
    for chunk in source.chunks(chunk_size) {
        for byte in black_box(chunk) {
            checksum = checksum.wrapping_add(*byte);
        }
    }
    let elapsed = start.elapsed();
    println!(
        "buffer_bytes={chunk_size} total_bytes={total} elapsed_us={} checksum={checksum}",
        elapsed.as_micros()
    );
}
