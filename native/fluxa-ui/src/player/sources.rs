use crate::Viewport;

pub(crate) fn sources_modal(viewport: Viewport) -> bool {
    viewport.width >= 900.0
}
