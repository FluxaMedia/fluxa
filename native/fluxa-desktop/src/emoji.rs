use fontdb::{Database, Family, Query};
use image::{DynamicImage, RgbaImage, imageops};
use rustybuzz::ttf_parser::RasterImageFormat;
use rustybuzz::{Face, UnicodeBuffer};
use std::sync::OnceLock;

struct EmojiFont {
    data: Vec<u8>,
    index: u32,
}

static FONT: OnceLock<Option<EmojiFont>> = OnceLock::new();

fn load() -> Option<EmojiFont> {
    let mut db = Database::new();
    db.load_system_fonts();
    let id = db.query(&Query {
        families: &[
            Family::Name("Noto Color Emoji"),
            Family::Name("Apple Color Emoji"),
        ],
        ..Query::default()
    })?;
    db.with_face_data(id, |data, index| EmojiFont {
        data: data.to_vec(),
        index,
    })
}

pub fn warm_up() {
    std::thread::spawn(|| {
        FONT.get_or_init(load);
    });
}

pub fn rasterize(cluster: &str, size: u32) -> Option<(u32, u32, Vec<u8>)> {
    let font = FONT.get_or_init(load).as_ref()?;
    let face = Face::from_slice(&font.data, font.index)?;
    let mut buffer = UnicodeBuffer::new();
    buffer.push_str(cluster);
    buffer.guess_segment_properties();
    let shaped = rustybuzz::shape(&face, &[], buffer);
    let [info] = shaped.glyph_infos() else {
        return None;
    };
    let glyph = rustybuzz::ttf_parser::GlyphId(u16::try_from(info.glyph_id).ok()?);
    if glyph.0 == 0 {
        return None;
    }
    let raster = face.glyph_raster_image(glyph, size as u16)?;
    if raster.format != RasterImageFormat::PNG {
        return None;
    }
    let decoded = image::load_from_memory_with_format(raster.data, image::ImageFormat::Png)
        .ok()?
        .into_rgba8();
    let side = decoded.width().max(decoded.height());
    let scale = size as f32 / side as f32;
    let width = ((decoded.width() as f32 * scale).round() as u32).max(1);
    let height = ((decoded.height() as f32 * scale).round() as u32).max(1);
    let scaled = DynamicImage::ImageRgba8(decoded)
        .resize_exact(width, height, imageops::FilterType::Lanczos3)
        .into_rgba8();
    let mut canvas = RgbaImage::new(size, size);
    imageops::overlay(
        &mut canvas,
        &scaled,
        ((size - width.min(size)) / 2) as i64,
        ((size - height.min(size)) / 2) as i64,
    );
    Some((size, size, canvas.into_raw()))
}

#[cfg(test)]
mod tests {
    use super::rasterize;

    fn bounds(rgba: &[u8], size: u32) -> (u32, u32) {
        let mut max_x = 0;
        let mut min_x = size;
        let mut max_y = 0;
        let mut min_y = size;
        for (i, px) in rgba.chunks(4).enumerate() {
            if px[3] > 8 {
                let (x, y) = (i as u32 % size, i as u32 / size);
                min_x = min_x.min(x);
                max_x = max_x.max(x);
                min_y = min_y.min(y);
                max_y = max_y.max(y);
            }
        }
        (max_x - min_x + 1, max_y - min_y + 1)
    }

    #[test]
    fn flags_stay_wide_and_clusters_resolve() {
        let Some((w, h, flag)) = rasterize("🇹🇷", 96) else {
            return;
        };
        assert_eq!((w, h), (96, 96));
        let (bw, bh) = bounds(&flag, 96);
        assert!(bw as f32 / bh as f32 > 1.2, "{bw}x{bh}");
        for cluster in ["👍🏽", "👨‍👩‍👧", "1️⃣", "😀"] {
            assert!(rasterize(cluster, 96).is_some(), "{cluster}");
        }
    }
}
