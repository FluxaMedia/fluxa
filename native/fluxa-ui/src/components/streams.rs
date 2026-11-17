use super::*;

struct StreamRowLayout {
    addon: Option<std::sync::Arc<crate::gfx::Galley>>,
    name: std::sync::Arc<crate::gfx::Galley>,
    detail: Option<std::sync::Arc<crate::gfx::Galley>>,
    badges: Vec<(std::sync::Arc<crate::gfx::Galley>, usize, Rect)>,
    badges_height: f32,
    height: f32,
}

pub(crate) fn badge_chip_layout(
    painter: &crate::gfx::Painter,
    badge: &crate::SourceBadge,
    font_size: f32,
) -> (std::sync::Arc<crate::gfx::Galley>, Vec2) {
    let galley = painter.layout_no_wrap(
        badge.label.clone(),
        FontId::proportional(font_size),
        Color32::PLACEHOLDER,
    );
    let icon = if badge.image.is_some() {
        galley.size().y + 4.0
    } else {
        0.0
    };
    let size = galley.size() + Vec2::new(font_size * 1.4 + icon, font_size * 0.7);
    (galley, size)
}

fn readable_on(fill: Color32) -> Color32 {
    let luminance =
        0.299 * f32::from(fill.r()) + 0.587 * f32::from(fill.g()) + 0.114 * f32::from(fill.b());
    if luminance > 150.0 {
        Color32::BLACK
    } else {
        text_primary()
    }
}


pub(crate) fn paint_badge_chip_rounded(
    painter: &crate::gfx::Painter,
    chip: Rect,
    galley: std::sync::Arc<crate::gfx::Galley>,
    badge: &crate::SourceBadge,
    assets: &mut impl HomeAssets,
    rounding: f32,
) {
    let text = if badge.outline {
        let color = badge
            .border
            .or(badge.fill)
            .unwrap_or(crate::stylesheet::color("dim"));
        painter.rect_stroke(
            chip,
            rounding,
            crate::gfx::Stroke::new(stroke::hairline(), color),
            crate::gfx::StrokeKind::Inside,
        );
        badge.text.or(badge.fill).unwrap_or(text_primary())
    } else {
        let fill = badge.fill.unwrap_or(crate::stylesheet::color("track"));
        painter.rect_filled(chip, rounding, fill);
        if let Some(border) = badge.border {
            painter.rect_stroke(
                chip,
                rounding,
                crate::gfx::Stroke::new(stroke::hairline(), border),
                crate::gfx::StrokeKind::Inside,
            );
        }
        badge.text.unwrap_or(match badge.fill {
            Some(fill) => readable_on(fill),
            None => crate::stylesheet::color("text"),
        })
    };
    let mut left = chip.left() + (chip.width() - galley.size().x) * 0.5;
    let side = galley.size().y;
    let icon = crate::flex::anchored(chip, Align2::LEFT_CENTER, Vec2::splat(side), 0.0)
        .translate(Vec2::new(chip.height() * 0.35, 0.0));
    let badge_texture = badge
        .image
        .as_deref()
        .and_then(|url| assets.texture(Some(url)));
    if badge_texture.is_none() {
        crate::components::failed_dom_image(
            painter,
            assets,
            icon,
            0.0,
            badge.image.as_deref(),
            [u32::MAX; 2],
            crate::ImageFit::Contain,
            [0.5, 0.5],
        );
    }
    if let Some(texture) = badge_texture {
        painter.image(
            texture,
            icon,
            Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
            text_primary(),
        );
    }
    if badge.image.is_some() {
        left = chip.right() - chip.height() * 0.35 - galley.size().x;
    }
    painter.galley(
        Pos2::new(left, chip.center().y - galley.size().y * 0.5),
        galley,
        text,
    );
}

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum StreamAction {
    Download,
    Copy,
}



