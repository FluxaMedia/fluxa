use core::ops::Range;

use unicode_bidi::BidiInfo;

use super::Glyph;

fn is_rtl_script(c: char) -> bool {
    matches!(c as u32,
        0x0590..=0x08FF | 0xFB1D..=0xFDFF | 0xFE70..=0xFEFF | 0x200F | 0x202B | 0x202E | 0x2067)
}

pub(crate) fn levels(text: &str) -> Option<Vec<u8>> {
    if !text.chars().any(is_rtl_script) {
        return None;
    }
    let info = BidiInfo::new(text, None);
    Some(info.levels.iter().map(|level| level.number()).collect())
}

pub(crate) fn reorder_row(glyphs: &mut Vec<Glyph>) {
    if glyphs.is_empty() {
        return;
    }

    let mut units: Vec<Range<usize>> = Vec::new();
    let mut start = 0;
    for index in 1..glyphs.len() {
        if glyphs[index].unit_start {
            units.push(start..index);
            start = index;
        }
    }
    units.push(start..glyphs.len());

    let origins: Vec<f32> = units
        .iter()
        .map(|range| base_glyph(&glyphs[range.clone()]).pos.x)
        .collect();
    let widths: Vec<f32> = units
        .iter()
        .enumerate()
        .map(|(i, range)| match origins.get(i + 1) {
            Some(next) => next - origins[i],
            None => (base_glyph(&glyphs[range.clone()]).max_x() - origins[i]).max(0.0),
        })
        .collect();
    let levels: Vec<u8> = units.iter().map(|range| glyphs[range.start].bidi_level).collect();

    let mut order: Vec<usize> = (0..units.len()).collect();
    let highest = levels.iter().copied().max().unwrap_or(0);
    let lowest_odd = levels.iter().copied().filter(|l| l % 2 == 1).min();
    if let Some(lowest_odd) = lowest_odd {
        for level in (lowest_odd..=highest).rev() {
            let mut i = 0;
            while i < order.len() {
                if levels[order[i]] >= level {
                    let mut j = i;
                    while j < order.len() && levels[order[j]] >= level {
                        j += 1;
                    }
                    order[i..j].reverse();
                    i = j;
                } else {
                    i += 1;
                }
            }
        }
    }

    let mut cursor = origins[0];
    let mut reordered = Vec::with_capacity(glyphs.len());
    for unit in order {
        let delta = cursor - origins[unit];
        for glyph in &glyphs[units[unit].clone()] {
            let mut glyph = *glyph;
            glyph.pos.x += delta;
            reordered.push(glyph);
        }
        cursor += widths[unit];
    }
    *glyphs = reordered;
}

fn base_glyph(unit: &[Glyph]) -> &Glyph {
    unit.iter()
        .find(|glyph| glyph.advance_width > 0.0)
        .unwrap_or(&unit[0])
}
