use super::*;
use crate::flex::{self, Node, Placed};
use crate::gfx::{Align2, Color32, FontId, Painter};

pub(crate) enum Part {
    Text {
        text: String,
        font: FontId,
        color: Color32,
    },
    Icon {
        name: &'static str,
        size: f32,
        tint: Color32,
    },
    Avatar {
        radius: f32,
        name: String,
        url: Option<String>,
    },
}

impl Part {
    pub fn text(text: impl Into<String>, font: FontId, color: Color32) -> Self {
        Self::Text {
            text: text.into(),
            font,
            color,
        }
    }

    pub fn icon(name: &'static str, size: f32, tint: Color32) -> Self {
        Self::Icon { name, size, tint }
    }

    pub fn avatar(radius: f32, name: impl Into<String>, url: Option<&str>) -> Self {
        Self::Avatar {
            radius,
            name: name.into(),
            url: url.map(ToOwned::to_owned),
        }
    }

    fn size(&self, painter: &Painter) -> Vec2 {
        match self {
            Self::Text { text, font, .. } => flex::text_size(painter, text, font),
            Self::Icon { size, .. } => Vec2::splat(*size),
            Self::Avatar { radius, .. } => Vec2::splat(radius * 2.0),
        }
    }

    fn paint(
        &self,
        painter: &Painter,
        rect: Rect,
        assets: &mut impl HomeAssets,
        metrics: UiMetrics,
    ) {
        match self {
            Self::Text { text, font, color } => {
                flex::paint_text(painter, rect, text, font, *color, Align2::CENTER_CENTER)
            }
            Self::Icon { name, tint, .. } => {
                if let Some(texture) = assets.icon(name) {
                    painter.image(texture, rect, full_uv(), *tint);
                }
            }
            Self::Avatar {
                radius, name, url, ..
            } => crate::components::avatar(
                painter,
                assets,
                rect.center(),
                *radius,
                name,
                url.as_deref(),
                metrics,
            ),
        }
    }
}

#[derive(Default)]
pub(crate) struct Scene {
    parts: Vec<Part>,
}

impl Scene {
    pub fn add(&mut self, painter: &Painter, part: Part) -> Node {
        let node = flex::leaf(part.size(painter)).id(self.parts.len());
        self.parts.push(part);
        node
    }

    pub fn paint(
        &self,
        placed: &Placed,
        painter: &Painter,
        assets: &mut impl HomeAssets,
        metrics: UiMetrics,
    ) {
        for (id, rect) in placed.tagged() {
            if let Some(part) = self.parts.get(id) {
                part.paint(painter, rect, assets, metrics);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parts_report_the_size_of_their_content() {
        let context = crate::gfx::Context::default();
        context.set_fonts(crate::fonts::definitions());
        let mut sizes = Vec::new();
        context
            .run_ui(crate::gfx::RawInput::default(), |root| {
                let painter = root.painter().clone();
                let font = FontId::proportional(14.0);
                sizes.push(Part::icon("Plus", 20.0, Color32::WHITE).size(&painter));
                sizes.push(Part::avatar(15.0, "A", None).size(&painter));
                let short = Part::text("a", font.clone(), Color32::WHITE).size(&painter);
                let long = Part::text("a much longer label", font, Color32::WHITE).size(&painter);
                sizes.push(short);
                sizes.push(long);
            })
            .drop_without_applying_deltas();
        assert_eq!(sizes[0], Vec2::splat(20.0));
        assert_eq!(sizes[1], Vec2::splat(30.0));
        assert!(sizes[3].x > sizes[2].x);
        assert!((sizes[2].y - sizes[3].y).abs() < 0.5);
    }
}
