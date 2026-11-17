use super::*;

pub(crate) struct StickyAction {
    pub node: u64,
    pub icon: Option<crate::gfx::TextureId>,
    pub active: bool,
    pub hint: String,
}

pub(crate) struct StickyBar<'a> {
    pub back_node: u64,
    pub back_icon: Option<crate::gfx::TextureId>,
    pub logo: Option<(crate::gfx::TextureId, Vec2)>,
    pub title: &'a str,
    pub actions: Vec<StickyAction>,
}

pub(crate) struct StickyOutput {
    pub clicked: Option<u64>,
    pub rects: Vec<(u64, Rect)>,
}

