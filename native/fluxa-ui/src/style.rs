use super::*;

pub(crate) mod radius {
    pub const BAR: f32 = 2.0;
    pub const CHIP: f32 = 4.0;
    pub const CONTROL: f32 = 10.0;
}

pub(crate) mod stroke {
    pub const HAIRLINE: f32 = 1.0;
    pub const ICON: f32 = 1.6;
    pub const EMPHASIS: f32 = 2.0;
}

pub(crate) mod size {
    pub const BUTTON_HEIGHT: f32 = 44.0;
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
    Color32::from_black_alpha(match level {
        Shade::Faint => 90,
        Shade::Veil => 150,
        Shade::Scrim => 180,
        Shade::Strong => 215,
        Shade::Opaque => 240,
    })
}

pub(crate) fn badge_fill() -> Color32 {
    shade(Shade::Scrim)
}

pub(crate) fn overlay_panel() -> Color32 {
    let [r, g, b, _] = surface_color().to_array();
    Color32::from_rgba_unmultiplied(r, g, b, 242)
}
