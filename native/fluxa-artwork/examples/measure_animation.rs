use std::{env, fs, time::Instant};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args().skip(1);
    let width: u32 = args
        .next()
        .ok_or("usage: measure_animation <w> <h> <files..>")?
        .parse()?;
    let height: u32 = args.next().ok_or("missing height")?.parse()?;
    let target = fluxa_artwork::animation_target_size([width, height]);
    for path in args {
        let bytes = fs::read(&path)?;
        let started = Instant::now();
        let atlas = match fluxa_artwork::decode_animation_atlas(&bytes, target) {
            Ok(Some(atlas)) => atlas,
            Ok(None) => {
                println!("{path}: not animated");
                continue;
            }
            Err(error) => {
                println!("{path}: {error}");
                continue;
            }
        };
        let elapsed = started.elapsed();
        let frames = atlas.frames.len();
        let size = atlas.frames[0].image_size;
        let atlas_bytes = atlas.image.width() as u64 * atlas.image.height() as u64 * 4;
        let frame_bytes = size[0] as u64 * size[1] as u64 * 4;
        let total_ms: u128 = atlas.frames.iter().map(|f| f.duration.as_millis()).sum();
        println!(
            "{}\tsrc={}KB\tframes={frames}\tframe={}x{}\tatlas={}x{}\tgpu={:.1}MB\tring2={:.2}MB\tloop={total_ms}ms\tdecode={}ms ({:.1}ms/frame)",
            path.rsplit('/').next().unwrap(),
            bytes.len() / 1024,
            size[0],
            size[1],
            atlas.image.width(),
            atlas.image.height(),
            atlas_bytes as f64 / 1048576.0,
            (frame_bytes * 2) as f64 / 1048576.0,
            elapsed.as_millis(),
            elapsed.as_secs_f64() * 1000.0 / frames as f64,
        );
    }
    Ok(())
}
