use std::sync::Arc;

use egui::text::{LayoutJob, TextFormat};
use egui::{Color32, FontId, Galley, Painter, Pos2, Rect, Vec2};

use super::{ArtworkPriority, HomeAssets, full_uv};

const TWEMOJI: &str = "https://cdn.jsdelivr.net/gh/jdecked/twemoji@15.1.0/assets/72x72";

fn is_base(c: char) -> bool {
    matches!(c as u32,
        0x1F000..=0x1FAFF | 0x2600..=0x27BF | 0x2300..=0x23FF | 0x2B00..=0x2BFF
        | 0x2190..=0x21FF | 0x3030 | 0x303D | 0x3297 | 0x3299 | 0x00A9 | 0x00AE | 0x203C | 0x2049
        | 0x2122 | 0x2139)
}

fn is_joiner(c: char) -> bool {
    matches!(c as u32, 0xFE0F | 0x200D | 0x20E3 | 0x1F3FB..=0x1F3FF | 0xE0020..=0xE007F)
}

fn is_flag(c: char) -> bool {
    matches!(c as u32, 0x1F1E6..=0x1F1FF)
}

pub(crate) fn job(text: &str, font: FontId, color: Color32, width: f32) -> LayoutJob {
    let mut job = LayoutJob::default();
    job.wrap.max_width = width;
    let hidden = FontId::new(0.01, font.family.clone());
    let mut run = String::new();
    let mut run_kind = 0;
    let mut open_flag = false;
    for c in text.chars() {
        let kind = if is_joiner(c) || (is_flag(c) && open_flag) {
            open_flag = false;
            2
        } else if is_base(c) {
            open_flag = is_flag(c);
            1
        } else {
            open_flag = false;
            0
        };
        if kind != run_kind && !run.is_empty() {
            push(&mut job, &run, run_kind, &font, &hidden, color);
            run.clear();
        }
        run_kind = kind;
        run.push(c);
    }
    push(&mut job, &run, run_kind, &font, &hidden, color);
    job
}

fn push(job: &mut LayoutJob, run: &str, kind: u8, font: &FontId, hidden: &FontId, color: Color32) {
    if run.is_empty() {
        return;
    }
    let format = match kind {
        0 => TextFormat::simple(font.clone(), color),
        1 => TextFormat::simple(font.clone(), Color32::TRANSPARENT),
        _ => TextFormat::simple(hidden.clone(), Color32::TRANSPARENT),
    };
    job.append(run, 0.0, format);
}

pub(crate) fn paint(
    painter: &Painter,
    pos: Pos2,
    galley: Arc<Galley>,
    assets: &mut impl HomeAssets,
) {
    let mut clusters = Vec::new();
    for placed in &galley.rows {
        let glyphs = &placed.row.glyphs;
        let mut i = 0;
        while i < glyphs.len() {
            let first = &glyphs[i];
            if !is_base(first.chr) {
                i += 1;
                continue;
            }
            let mut chars = vec![first.chr];
            let mut j = i + 1;
            if is_flag(first.chr) {
                if glyphs.get(j).is_some_and(|glyph| is_flag(glyph.chr)) {
                    chars.push(glyphs[j].chr);
                    j += 1;
                }
            } else {
                while let Some(glyph) = glyphs.get(j) {
                    let after_zwj = chars.last() == Some(&'\u{200D}');
                    if is_joiner(glyph.chr) || (after_zwj && is_base(glyph.chr)) {
                        chars.push(glyph.chr);
                        j += 1;
                    } else {
                        break;
                    }
                }
            }
            let size = first.font_height;
            let left = pos.x + placed.pos.x + first.pos.x;
            let top = pos.y + placed.pos.y + first.pos.y - first.font_ascent;
            clusters.push((chars, Rect::from_min_size(Pos2::new(left, top), Vec2::splat(size))));
            i = j;
        }
    }
    painter.galley(pos, galley, Color32::WHITE);
    let native = assets.has_native_emoji();
    for (chars, rect) in clusters {
        let texture = if native {
            assets.emoji(&chars.iter().collect::<String>())
        } else {
            let url = format!("{TWEMOJI}/{}.png", code(&chars));
            assets.texture_for(Some(&url), [72, 72], ArtworkPriority::Visible)
        };
        if let Some(texture) = texture {
            painter.image(texture, rect.shrink(rect.width() * 0.06), full_uv(), Color32::WHITE);
        }
    }
}

fn code(chars: &[char]) -> String {
    let keep_fe0f = chars.contains(&'\u{200D}');
    chars
        .iter()
        .filter(|c| keep_fe0f || **c != '\u{FE0F}')
        .map(|c| format!("{:x}", *c as u32))
        .collect::<Vec<_>>()
        .join("-")
}
