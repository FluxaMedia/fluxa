use super::*;
use std::sync::atomic::{AtomicU8, Ordering};

static SHADES: [AtomicU8; 5] = [
    AtomicU8::new(90),
    AtomicU8::new(150),
    AtomicU8::new(180),
    AtomicU8::new(215),
    AtomicU8::new(240),
];
static PANEL_ALPHA: AtomicU8 = AtomicU8::new(242);

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
    pub const FAINT: u8 = 10;
    pub const SOFT: u8 = 20;
    pub const RAISED: u8 = 30;
    pub const STRONG: u8 = 50;
    pub const DIM: u8 = 90;
    pub const MUTED: u8 = 130;
    pub const SUBTLE: u8 = 170;
    pub const BODY: u8 = 210;
    pub const TEXT: u8 = 235;
    pub const FULL: u8 = 255;
}

pub(crate) mod stroke {
    pub const HAIRLINE: f32 = 1.0;
    pub const ICON: f32 = 1.6;
    pub const EMPHASIS: f32 = 2.0;
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
