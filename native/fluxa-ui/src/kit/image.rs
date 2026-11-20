use super::*;
use crate::components::ImageFit;
use crate::gfx::{Align2, Sense};

pub(crate) struct Image<'a> {
    url: Option<&'a str>,
    size: Vec2,
    fit: ImageFit,
    circle: bool,
    rounded: bool,
    priority: ArtworkPriority,
}

impl Image<'_> {}

pub(crate) fn fit_inside(rect: Rect, size: [u32; 2]) -> Rect {
    let aspect = size[0] as f32 / size[1].max(1) as f32;
    let scale = (rect.width() / aspect).min(rect.height());
    crate::flex::anchored(
        rect,
        Align2::CENTER_CENTER,
        Vec2::new(scale * aspect, scale),
        0.0,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contain_letterboxes_wide_images_inside_the_box() {
        let rect = Rect::from_min_max(Pos2::ZERO, Pos2::new(100.0, 100.0));
        let fitted = fit_inside(rect, [200, 100]);
        assert_eq!(fitted.width(), 100.0);
        assert_eq!(fitted.height(), 50.0);
        assert_eq!(fitted.center(), rect.center());
    }
}
