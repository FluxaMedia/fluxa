//! Sheet tiling and WebVTT preview output (Jellyfin-style xywh cues).

use image::RgbImage;
use image::codecs::jpeg::JpegEncoder;
use image::imageops::FilterType;

#[derive(Debug, Clone)]
pub struct SheetSpec {
    pub tile_w: u32,
    pub tile_h: u32,
    pub columns: usize,
}

impl Default for SheetSpec {
    fn default() -> Self {
        Self {
            tile_w: 160,
            tile_h: 90,
            columns: 10,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Sheet {
    pub index: usize,
    pub jpeg: Vec<u8>,
    pub count: usize,
}

pub fn tile(frames: &[(f64, Vec<u8>)], spec: &SheetSpec) -> Vec<Sheet> {
    let mut sheets = Vec::new();
    if frames.is_empty() {
        return sheets;
    }
    let per_sheet = spec.columns * spec.columns;
    for (index, group) in frames.chunks(per_sheet).enumerate() {
        let rows = group.len().div_ceil(spec.columns);
        let mut canvas = RgbImage::new(
            (spec.columns as u32) * spec.tile_w,
            (rows as u32) * spec.tile_h,
        );
        for (cell, (_, jpeg)) in group.iter().enumerate() {
            let Ok(decoded) = image::load_from_memory(jpeg) else {
                continue;
            };
            let thumb = decoded
                .resize_exact(spec.tile_w, spec.tile_h, FilterType::Triangle)
                .to_rgb8();
            let x = (cell % spec.columns) as u32 * spec.tile_w;
            let y = (cell / spec.columns) as u32 * spec.tile_h;
            for (px, py, pixel) in thumb.enumerate_pixels() {
                canvas.put_pixel(x + px, y + py, *pixel);
            }
        }
        let mut jpeg = Vec::new();
        if JpegEncoder::new_with_quality(&mut jpeg, 80)
            .encode_image(&canvas)
            .is_ok()
        {
            sheets.push(Sheet {
                index: index + 1,
                jpeg,
                count: group.len(),
            });
        }
    }
    sheets
}

fn stamp(t: f64) -> String {
    let total = (t.max(0.0) * 1000.0).round() as u64;
    let ms = total % 1000;
    let total = total / 1000;
    let s = total % 60;
    let m = (total / 60) % 60;
    let h = total / 3600;
    format!("{h:02}:{m:02}:{s:02}.{ms:03}")
}

/// WebVTT where each cue maps a time span to a tile rectangle. `sheet_url`
/// may contain a `{sheet}` placeholder that callers substitute with the
/// hosted sheet file name.
pub fn vtt(times: &[f64], spec: &SheetSpec, sheet_url: &str) -> String {
    let mut out = String::from("WEBVTT\n\n");
    let per_sheet = spec.columns * spec.columns;
    for (cell, &t) in times.iter().enumerate() {
        let end = times.get(cell + 1).copied().unwrap_or(t + 5.0);
        let sheet_no = cell / per_sheet + 1;
        let x = (cell % spec.columns) * spec.tile_w as usize;
        let y = (cell / spec.columns) % spec.columns * spec.tile_h as usize;
        let url = sheet_url.replace("{sheet}", &sheet_no.to_string());
        out.push_str(&format!(
            "{} --> {}\n{}#xywh={},{},{},{}\n\n",
            stamp(t),
            stamp(end),
            url,
            x,
            y,
            spec.tile_w,
            spec.tile_h
        ));
    }
    out
}
