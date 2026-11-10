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

pub(crate) fn image(url: Option<&str>, size: Vec2) -> Image<'_> {
    Image {
        url,
        size,
        fit: ImageFit::Cover,
        circle: false,
        rounded: false,
        priority: ArtworkPriority::Visible,
    }
}

impl Image<'_> {
    pub fn contain(mut self) -> Self {
        self.fit = ImageFit::Contain;
        self
    }

    pub fn circle(mut self) -> Self {
        self.circle = true;
        self
    }

    pub fn rounded(mut self) -> Self {
        self.rounded = true;
        self
    }

    pub fn priority(mut self, priority: ArtworkPriority) -> Self {
        self.priority = priority;
        self
    }

    pub fn show(self, ui: &mut Ui, assets: &mut impl HomeAssets) -> Response {
        let tokens = ui.tokens();
        let (rect, response) = ui.allocate_exact_size(self.size, Sense::hover());
        let radius = if self.circle {
            rect.width().min(rect.height()) * 0.5
        } else if self.rounded {
            tokens.card_radius
        } else {
            0.0
        };
        let target = artwork_target_size(self.size, ui.ctx().pixels_per_point());
        let texture = assets.texture_for(self.url, target, self.priority);
        let Some(texture) = texture else {
            ui.painter()
                .rect_filled(rect, radius, tokens.surface_raised);
            return response;
        };
        let (dest, uv) = match (self.fit, assets.texture_size(self.url)) {
            (ImageFit::Cover, Some(size)) => (rect, cover_uv(size, rect)),
            (ImageFit::Contain, Some(size)) => (fit_inside(rect, size), full_uv()),
            _ => (rect, full_uv()),
        };
        crate::components::rounded_image(ui.painter(), texture, dest, radius, uv, Color32::WHITE);
        response
    }
}

fn fit_inside(rect: Rect, size: [u32; 2]) -> Rect {
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
