use egui::Color32;

pub(crate) const SURFACE: Color32 = Color32::from_rgb(19, 19, 19);
pub(crate) const SURFACE_RAISED: Color32 = Color32::from_rgb(28, 28, 28);
pub(crate) const BORDER: Color32 = Color32::from_rgba_premultiplied(20, 20, 20, 20);
pub(crate) const DANGER: Color32 = Color32::from_rgb(170, 48, 48);
pub(crate) const RADIUS_CARD: f32 = 14.0;
pub(crate) const RADIUS_PANEL: f32 = 14.0;

pub(crate) fn border() -> egui::Stroke {
    egui::Stroke::new(1.0, BORDER)
}
