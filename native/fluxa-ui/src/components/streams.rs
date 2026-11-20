use super::*;

struct StreamRowLayout {
    addon: Option<std::sync::Arc<crate::gfx::Galley>>,
    name: std::sync::Arc<crate::gfx::Galley>,
    detail: Option<std::sync::Arc<crate::gfx::Galley>>,
    badges: Vec<(std::sync::Arc<crate::gfx::Galley>, usize, Rect)>,
    badges_height: f32,
    height: f32,
}

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum StreamAction {
    Download,
    Copy,
}
