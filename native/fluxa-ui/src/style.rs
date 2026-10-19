use super::*;
use std::sync::atomic::{AtomicU8, AtomicU32, Ordering};

static SHADES: [AtomicU8; 5] = [
    AtomicU8::new(90),
    AtomicU8::new(150),
    AtomicU8::new(180),
    AtomicU8::new(215),
    AtomicU8::new(240),
];
static PANEL_ALPHA: AtomicU8 = AtomicU8::new(242);

static CONTROL_SIZES: [AtomicU32; 4] = [
    AtomicU32::new(0.44f32.to_bits()),
    AtomicU32::new(4.0f32.to_bits()),
    AtomicU32::new(6.0f32.to_bits()),
    AtomicU32::new(5.0f32.to_bits()),
];

pub(crate) fn set_control_sizes(sizes: [f32; 4]) {
    for (cell, value) in CONTROL_SIZES.iter().zip(sizes) {
        cell.store(value.to_bits(), Ordering::Relaxed);
    }
}

fn control_size(index: usize) -> f32 {
    f32::from_bits(CONTROL_SIZES[index].load(Ordering::Relaxed))
}

pub(crate) fn glyph_ratio() -> f32 {
    control_size(0)
}

pub(crate) fn slider_track() -> f32 {
    control_size(1)
}

pub(crate) fn slider_knob() -> f32 {
    control_size(2)
}

pub(crate) fn slider_mark() -> f32 {
    control_size(3)
}

pub(crate) fn set_tones(tones: [u8; 10], strokes: [f32; 3]) {
    for (cell, value) in tone::VALUES.iter().zip(tones) {
        cell.store(value, Ordering::Relaxed);
    }
    for (cell, value) in stroke::VALUES.iter().zip(strokes) {
        cell.store(value.to_bits(), Ordering::Relaxed);
    }
}

pub(crate) fn set_shades(shades: [u8; 5], panel: u8) {
    for (cell, value) in SHADES.iter().zip(shades) {
        cell.store(value, Ordering::Relaxed);
    }
    PANEL_ALPHA.store(panel, Ordering::Relaxed);
}

pub(crate) mod radius {
    pub const BAR: f32 = 2.0;

    pub fn control() -> f32 {
        crate::control_radius()
    }

    pub fn chip() -> f32 {
        crate::control_radius() * 0.45
    }
}

pub(crate) mod tone {
    use std::sync::atomic::Ordering;

    pub(super) static VALUES: [std::sync::atomic::AtomicU8; 10] = [
        std::sync::atomic::AtomicU8::new(10),
        std::sync::atomic::AtomicU8::new(20),
        std::sync::atomic::AtomicU8::new(30),
        std::sync::atomic::AtomicU8::new(50),
        std::sync::atomic::AtomicU8::new(90),
        std::sync::atomic::AtomicU8::new(130),
        std::sync::atomic::AtomicU8::new(170),
        std::sync::atomic::AtomicU8::new(210),
        std::sync::atomic::AtomicU8::new(235),
        std::sync::atomic::AtomicU8::new(255),
    ];

    pub fn faint() -> u8 {
        VALUES[0].load(Ordering::Relaxed)
    }

    pub fn soft() -> u8 {
        VALUES[1].load(Ordering::Relaxed)
    }

    pub fn raised() -> u8 {
        VALUES[2].load(Ordering::Relaxed)
    }

    pub fn strong() -> u8 {
        VALUES[3].load(Ordering::Relaxed)
    }

    pub fn dim() -> u8 {
        VALUES[4].load(Ordering::Relaxed)
    }

    pub fn muted() -> u8 {
        VALUES[5].load(Ordering::Relaxed)
    }

    pub fn subtle() -> u8 {
        VALUES[6].load(Ordering::Relaxed)
    }

    pub fn body() -> u8 {
        VALUES[7].load(Ordering::Relaxed)
    }

    pub fn text() -> u8 {
        VALUES[8].load(Ordering::Relaxed)
    }

    pub fn full() -> u8 {
        VALUES[9].load(Ordering::Relaxed)
    }
}

pub(crate) mod stroke {
    use std::sync::atomic::{AtomicU32, Ordering};

    pub(super) static VALUES: [AtomicU32; 3] = [
        AtomicU32::new(1.0f32.to_bits()),
        AtomicU32::new(1.6f32.to_bits()),
        AtomicU32::new(2.0f32.to_bits()),
    ];

    pub fn hairline() -> f32 {
        f32::from_bits(VALUES[0].load(Ordering::Relaxed))
    }

    pub fn icon() -> f32 {
        f32::from_bits(VALUES[1].load(Ordering::Relaxed))
    }

    pub fn emphasis() -> f32 {
        f32::from_bits(VALUES[2].load(Ordering::Relaxed))
    }
}

pub(crate) mod size {
    pub const BUTTON_TEXT: f32 = 13.0;
}

#[derive(Clone, Copy)]
pub(crate) enum Shade {
    Faint,
    Veil,
    Scrim,
    Strong,
    Opaque,
}

pub(crate) fn shade(level: Shade) -> Color32 {
    Color32::from_black_alpha(SHADES[level as usize].load(Ordering::Relaxed))
}

pub(crate) fn badge_fill() -> Color32 {
    shade(Shade::Scrim)
}

pub(crate) fn overlay_panel() -> Color32 {
    let [r, g, b, _] = surface_color().to_array();
    Color32::from_rgba_unmultiplied(r, g, b, PANEL_ALPHA.load(Ordering::Relaxed))
}
