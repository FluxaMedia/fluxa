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
            rect.height() * crate::vars::px("surface-card-logo-scrim-share"),
        ),
        0.0,
    );
    crate::paint_vertical_gradient(
        &painter.with_clip_rect(rect.intersect(painter.clip_rect())),
        scrim,
        Color32::TRANSPARENT,
        Color32::from_rgba_unmultiplied(0, 0, 0, crate::vars::byte("card-logo-scrim")),
    );
    let bounds = Vec2::new(
        rect.width() * crate::vars::px("surface-card-textless-logo-width-share"),
        rect.height() * crate::vars::px("surface-card-textless-logo-height-share"),
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
            rect.height() * crate::vars::px("surface-card-textless-logo-center") * 2.0,
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
        rect.width() * crate::vars::px("surface-card-landscape-logo-width-share"),
        rect.height() * crate::vars::px("surface-card-landscape-logo-height-share"),
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
