use std::sync::{Arc, OnceLock};

use ar_reshaper::{ArabicReshaper, ReshaperConfig};
use unicode_bidi::{BidiInfo, Level};

use super::Glyph;

const FILLER: char = '\u{200B}';

#[derive(Clone, Copy)]
pub(crate) struct Resolved {
    pub chr: char,
    pub level: u8,
}

pub(crate) struct Analysis {
    by_byte: Vec<Resolved>,
}

impl Analysis {
    pub(crate) fn at(&self, byte: usize) -> Resolved {
        self.by_byte[byte]
    }
}

fn is_rtl_script(c: char) -> bool {
    matches!(c as u32,
        0x0590..=0x08FF | 0xFB1D..=0xFDFF | 0xFE70..=0xFEFF | 0x200F | 0x202B | 0x202E | 0x2067)
}

pub(crate) fn analyze(text: &str) -> Option<Arc<Analysis>> {
    if !text.chars().any(is_rtl_script) {
        return None;
    }
    let info = BidiInfo::new(text, None);
    let mut by_byte = vec![Resolved { chr: '\0', level: 0 }; text.len() + 1];
    for para in &info.paragraphs {
        let range = para.range.clone();
        let original: Vec<(usize, char)> = text[range.clone()]
            .char_indices()
            .map(|(i, c)| (range.start + i, c))
            .collect();
        let shaped = shape(&original.iter().map(|(_, c)| *c).collect::<String>());
        for (n, (byte, c)) in original.iter().enumerate() {
            let level = info.levels[*byte];
            let mut chr = shaped.as_ref().map_or(*c, |s| s[n]);
            if level.is_rtl() {
                chr = mirror(chr);
            }
            by_byte[*byte] = Resolved { chr, level: level.number() };
        }
    }
    Some(Arc::new(Analysis { by_byte }))
}

fn reshaper() -> &'static ArabicReshaper {
    static RESHAPER: OnceLock<ArabicReshaper> = OnceLock::new();
    RESHAPER.get_or_init(|| {
        ArabicReshaper::new(ReshaperConfig {
            delete_harakat: false,
            support_ligatures: false,
            ..ReshaperConfig::default()
        })
    })
}

fn shape(paragraph: &str) -> Option<Vec<char>> {
    if !paragraph.chars().any(|c| matches!(c as u32, 0x0600..=0x06FF | 0x0750..=0x077F | 0x08A0..=0x08FF)) {
        return None;
    }
    let original: Vec<char> = paragraph.chars().collect();
    let mut shaped: Vec<char> = reshaper().reshape(paragraph).chars().collect();
    if shaped.len() != original.len() {
        return None;
    }
    for i in 0..shaped.len().saturating_sub(1) {
        let lam = shaped[i];
        let ligature = match shaped[i + 1] {
            '\u{FE82}' => ['\u{FEF5}', '\u{FEF6}'],
            '\u{FE84}' => ['\u{FEF7}', '\u{FEF8}'],
            '\u{FE88}' => ['\u{FEF9}', '\u{FEFA}'],
            '\u{FE8E}' => ['\u{FEFB}', '\u{FEFC}'],
            _ => continue,
        };
        let form = match lam {
            '\u{FEDF}' => 0,
            '\u{FEE0}' => 1,
            _ => continue,
        };
        shaped[i] = ligature[form];
        shaped[i + 1] = FILLER;
    }
    Some(shaped)
}

fn mirror(c: char) -> char {
    match c {
        '(' => ')',
        ')' => '(',
        '[' => ']',
        ']' => '[',
        '{' => '}',
        '}' => '{',
        '<' => '>',
        '>' => '<',
        '«' => '»',
        '»' => '«',
        '‹' => '›',
        '›' => '‹',
        _ => c,
    }
}

pub(crate) fn reorder_row(glyphs: &mut [Glyph]) {
    if glyphs.iter().all(|g| g.bidi_level == 0) {
        return;
    }
    let levels: Vec<Level> = glyphs
        .iter()
        .map(|g| Level::new(g.bidi_level).unwrap_or(Level::ltr()))
        .collect();
    let order = BidiInfo::reorder_visual(&levels);
    let start = glyphs.iter().map(|g| g.pos.x).fold(f32::INFINITY, f32::min);
    let logical: Vec<Glyph> = glyphs.to_vec();
    let mut x = start;
    for (slot, &source) in order.iter().enumerate() {
        let mut glyph = logical[source];
        glyph.pos.x = x;
        x += glyph.advance_width;
        glyphs[slot] = glyph;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::text::font::UvRect;
    use emath::pos2;

    fn glyph(chr: char, level: u8, x: f32) -> Glyph {
        Glyph {
            chr,
            pos: pos2(x, 0.0),
            advance_width: 10.0,
            line_height: 12.0,
            font_ascent: 10.0,
            font_height: 12.0,
            font_face_ascent: 10.0,
            font_face_height: 12.0,
            uv_rect: UvRect::default(),
            section_index: 0,
            first_vertex: 0,
            bidi_level: level,
        }
    }

    #[test]
    fn latin_text_skips_analysis() {
        assert!(analyze("Continue watching 2024").is_none());
    }

    #[test]
    fn lam_alef_collapses_into_one_glyph() {
        let text = "لا";
        let analysis = analyze(text).unwrap();
        assert_eq!(analysis.at(0).chr, '\u{FEFB}');
        assert_eq!(analysis.at(2).chr, FILLER);
    }

    #[test]
    fn joined_letters_use_contextual_forms() {
        let analysis = analyze("بب").unwrap();
        assert_eq!(analysis.at(0).chr, '\u{FE91}');
        assert_eq!(analysis.at(2).chr, '\u{FE90}');
    }

    #[test]
    fn digits_stay_left_to_right_inside_rtl_run() {
        let text = "שלום 123";
        let analysis = analyze(text).unwrap();
        let digit = text.find('1').unwrap();
        assert_eq!(analysis.at(digit).level, 2);
        assert_eq!(analysis.at(0).level, 1);
    }

    #[test]
    fn rtl_row_is_laid_out_right_to_left() {
        let mut glyphs = vec![
            glyph('a', 1, 0.0),
            glyph('b', 1, 10.0),
            glyph('c', 1, 20.0),
        ];
        reorder_row(&mut glyphs);
        let order: Vec<char> = glyphs.iter().map(|g| g.chr).collect();
        assert_eq!(order, vec!['c', 'b', 'a']);
        assert_eq!(glyphs[0].pos.x, 0.0);
        assert_eq!(glyphs[2].pos.x, 20.0);
    }

    #[test]
    fn mirrored_brackets_in_rtl() {
        let analysis = analyze("(א)").unwrap();
        assert_eq!(analysis.at(0).chr, ')');
    }

}
