use super::*;

#[derive(Clone, Copy)]
pub(super) struct Frame<'a> {
    pub context: &'a crate::gfx::Context,
    pub viewport: Viewport,
    pub settings: &'a SettingsModel,
    pub language: &'a str,
    pub painter: &'a crate::gfx::Painter,
    pub scroll_y: f32,
    pub section_top: f32,
    pub active_section: usize,
    pub searching: bool,
}
