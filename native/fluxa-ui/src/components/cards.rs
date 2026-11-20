use super::*;

pub(crate) fn textless_logo(
    painter: &Painter,
    rect: Rect,
    url: &str,
    assets: &mut impl HomeAssets,
) {
    let metrics = crate::metrics::active();
    let scrim = crate::flex::anchored(
        rect,
        Align2::CENTER_BOTTOM,
        Vec2::new(
            rect.width(),
            rect.height() * metrics.surface_card_logo_scrim_share,
        ),
        0.0,
    );
    crate::paint_vertical_gradient(
        &painter.with_clip_rect(rect.intersect(painter.clip_rect())),
        scrim,
        Color32::TRANSPARENT,
        Color32::from_rgba_unmultiplied(0, 0, 0, metrics.card_logo_scrim),
    );
    let bounds = Vec2::new(
        rect.width() * metrics.surface_card_textless_logo_width_share,
        rect.height() * metrics.surface_card_textless_logo_height_share,
    );
    let Some((texture, logo_size)) = title_logo(
        painter.ctx().pixels_per_point(),
        Some(url),
        bounds,
        ArtworkPriority::Visible,
        assets,
    ) else {
        return;
    };
    let band = crate::flex::anchored(
        rect,
        Align2::CENTER_BOTTOM,
        Vec2::new(
            rect.width(),
            rect.height() * metrics.surface_card_textless_logo_center * 2.0,
        ),
        0.0,
    );
    texture_image(
        painter,
        texture,
        crate::flex::anchored(band, Align2::CENTER_CENTER, logo_size, 0.0),
        Rect::from_min_max(crate::gfx::Pos2::ZERO, crate::gfx::Pos2::new(1.0, 1.0)),
        Color32::WHITE,
    );
}

pub(crate) fn landscape_logo(
    painter: &Painter,
    rect: Rect,
    card: &HomeCard,
    assets: &mut impl HomeAssets,
) {
    let metrics = crate::metrics::active();
    let bounds = Vec2::new(
        rect.width() * metrics.surface_card_landscape_logo_width_share,
        rect.height() * metrics.surface_card_landscape_logo_height_share,
    );
    let logo_url = crate::card_logos::logo_of(card);
    let Some((texture, logo_size)) = title_logo(
        painter.ctx().pixels_per_point(),
        logo_url.as_deref(),
        bounds,
        ArtworkPriority::Visible,
        assets,
    ) else {
        failed_dom_image(
            painter,
            assets,
            crate::flex::anchored(rect, Align2::CENTER_CENTER, bounds, 0.0),
            0.0,
            logo_url.as_deref(),
            artwork_target_size(bounds, painter.ctx().pixels_per_point()),
            ImageFit::Contain,
            [0.5, 0.5],
        );
        return;
    };
    texture_image(
        painter,
        texture,
        crate::flex::anchored(rect, Align2::CENTER_CENTER, logo_size, 0.0),
        Rect::from_min_max(crate::gfx::Pos2::ZERO, crate::gfx::Pos2::new(1.0, 1.0)),
        Color32::WHITE,
    );
}
