use image::AnimationDecoder;
use std::{collections::BTreeMap, env, fs, io::Cursor, path::Path};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args_os().skip(1);
    let path = args.next().ok_or(
        "usage: cargo run -p fluxa-artwork --example inspect_animated_webp -- <file.webp>",
    )?;
    let ivf_dir = args.next();
    let bytes = fs::read(path)?;
    let Some(animation) = fluxa_artwork::webp_animation::parse(&bytes)? else {
        println!("valid non-animated WebP");
        return Ok(());
    };
    let vp8 = animation
        .frames
        .iter()
        .filter(|frame| frame.vp8.is_some())
        .count();
    let vp8l = animation
        .frames
        .iter()
        .filter(|frame| frame.vp8l.is_some())
        .count();
    let alpha = animation
        .frames
        .iter()
        .filter(|frame| frame.alpha.is_some())
        .count();
    println!(
        "{}x{}, {} frames (VP8: {vp8}, VP8L: {vp8l}, ALPH: {alpha}), loop={:?}",
        animation.canvas_width,
        animation.canvas_height,
        animation.frames.len(),
        animation.loop_count,
    );
    let mut timestamp_ms = 0u64;
    let mut vp8_groups = BTreeMap::<(u32, u32), Vec<(u64, Vec<u8>)>>::new();
    for (index, frame) in animation.frames.iter().enumerate() {
        let decoded_alpha = frame
            .alpha
            .as_ref()
            .map(|alpha| {
                fluxa_artwork::webp_animation::decode_alpha(frame.width, frame.height, alpha)
            })
            .transpose()?;
        let alpha_format = frame.alpha.as_ref().map_or_else(
            || "none".to_owned(),
            |alpha| {
                format!(
                    "method={},filter={},preprocess={}",
                    alpha.compression_method, alpha.filter, alpha.preprocessing
                )
            },
        );
        println!(
            "#{index:04}: {}x{} at {},{} for {}ms blend={} dispose={} vp8={}B alpha={}B ({alpha_format}) decoded-alpha={}B vp8l={}B",
            frame.width,
            frame.height,
            frame.x,
            frame.y,
            frame.duration_ms,
            if frame.no_blend { "replace" } else { "over" },
            frame.dispose_to_background,
            frame.vp8.as_ref().map_or(0, Vec::len),
            frame.alpha.as_ref().map_or(0, |alpha| alpha.data.len()),
            decoded_alpha.as_ref().map_or(0, Vec::len),
            frame.vp8l.as_ref().map_or(0, Vec::len),
        );
        if let Some(payload) = &frame.vp8 {
            vp8_groups
                .entry((frame.width, frame.height))
                .or_default()
                .push((timestamp_ms, payload.clone()));
        }
        timestamp_ms = timestamp_ms.saturating_add(u64::from(frame.duration_ms));
    }
    if let Some(directory) = ivf_dir {
        let directory = Path::new(&directory);
        fs::create_dir_all(directory)?;
        for ((width, height), frames) in vp8_groups {
            let width = u16::try_from(width)?;
            let height = u16::try_from(height)?;
            let mut ivf =
                Vec::with_capacity(32 + frames.iter().map(|(_, p)| p.len() + 12).sum::<usize>());
            ivf.extend_from_slice(b"DKIF");
            ivf.extend_from_slice(&0u16.to_le_bytes());
            ivf.extend_from_slice(&32u16.to_le_bytes());
            ivf.extend_from_slice(b"VP80");
            ivf.extend_from_slice(&width.to_le_bytes());
            ivf.extend_from_slice(&height.to_le_bytes());
            ivf.extend_from_slice(&1000u32.to_le_bytes());
            ivf.extend_from_slice(&1u32.to_le_bytes());
            ivf.extend_from_slice(&(frames.len() as u32).to_le_bytes());
            ivf.extend_from_slice(&0u32.to_le_bytes());
            for (timestamp, payload) in frames {
                ivf.extend_from_slice(&(payload.len() as u32).to_le_bytes());
                ivf.extend_from_slice(&timestamp.to_le_bytes());
                ivf.extend_from_slice(&payload);
            }
            let output = directory.join(format!("vp8-{width}x{height}.ivf"));
            fs::write(&output, ivf)?;
            println!("wrote {}", output.display());
        }
    }
    if let Some(frame) = animation.frames.first()
        && frame.x == 0
        && frame.y == 0
        && frame.width == animation.canvas_width
        && frame.height == animation.canvas_height
        && let Some(alpha) = frame.alpha.as_ref()
    {
        let extracted =
            fluxa_artwork::webp_animation::decode_alpha(frame.width, frame.height, alpha)?;
        let cpu_frame = image::codecs::webp::WebPDecoder::new(Cursor::new(bytes))?
            .into_frames()
            .next()
            .ok_or("WebP decoder returned no frame")??
            .into_buffer();
        let mismatches = extracted
            .iter()
            .zip(cpu_frame.pixels().map(|pixel| pixel[3]))
            .filter(|(decoded, reference)| **decoded != *reference)
            .count();
        println!(
            "first full-frame ALPH vs CPU WebP decode: {mismatches} mismatches / {} samples",
            extracted.len()
        );
    }
    Ok(())
}
