use super::*;
use crate::{Shade, shade};
use crate::{badge_fill, components};

pub(crate) fn paint_landscape(painter: &Painter, rect: Rect, card: &HomeCard) {
    let Some(overlays) = current(painter.ctx()) else {
        return;
    };
    let painter = painter.with_clip_rect(rect.intersect(painter.clip_rect()));
    let facts = &card.overlay;
    let inset = (rect.height() * 0.06).max(4.0);
    let size = (rect.height() * 0.09).clamp(8.0, 13.0) * overlays.scale;
    let enrichment = painter
        .ctx()
        .data(|data| data.get_temp::<Arc<Enrichment>>(enrichment_id()));
    let graded = enrichment
        .as_deref()
        .and_then(|enrichment| enrichment.graded(facts));
    let (rating, rating_logo_name) = meta::resolve_rating(&overlays, facts, graded);
    if let (true, Some(value)) = (overlays.rating.is_some(), rating) {
        let logo = rating_logo(painter.ctx(), rating_logo_name);
        let text = match (logo, rating_logo_name) {
            (None, "imdb") => format!("IMDb {value:.1}"),
            _ => format!("{value:.1}"),
        };
        let galley = painter.layout_no_wrap(text, FontId::proportional(size), Color32::WHITE);
        let logo_width = logo.map_or(0.0, |(_, aspect)| aspect.x * galley.size().y + size * 0.35);
        let (fill, _) = score_colors(value);
        let badge = Rect::from_min_size(
            rect.left_top() + Vec2::splat(inset),
            galley.size() + Vec2::new(size * 0.9 + logo_width, size * 0.5),
        );
        components::overlay_chip(&painter, badge, Some(fill));
        let height = galley.size().y;
        draw_logo(
            &painter,
            logo,
            Pos2::new(badge.left() + size * 0.45, badge.center().y),
            height,
        );
        painter.galley(
            badge.center() - galley.size() * 0.5 + Vec2::new(logo_width * 0.5, 0.0),
            galley,
            text_primary(),
        );
    }
    if overlays.quality {
        let quality = card
            .id
            .as_deref()
            .zip(
                painter
                    .ctx()
                    .data(|data| data.get_temp::<Arc<PersonalIndex>>(personal_id())),
            )
            .and_then(|(id, index)| index.0.get(id).map(|personal| personal.quality))
            .unwrap_or_default();
        quality_badges(
            &painter,
            rect.right() - inset,
            rect.top() + inset,
            size,
            quality,
        );
    }
}

pub(crate) fn paint_landscape_band(painter: &Painter, rect: Rect, card: &HomeCard) {
    let Some(overlays) = current(painter.ctx()) else {
        return;
    };
    let painter = painter.with_clip_rect(rect.intersect(painter.clip_rect()));
    let facts = &card.overlay;
    let size = (rect.height() * 0.08).clamp(8.0, 13.0) * overlays.scale;
    let enrichment = painter
        .ctx()
        .data(|data| data.get_temp::<Arc<Enrichment>>(enrichment_id()));
    let graded = enrichment
        .as_deref()
        .and_then(|enrichment| enrichment.graded(facts));
    let rating = overlays
        .rating
        .and(meta::resolve_rating(&overlays, facts, graded).0);
    let line = meta::meta(facts, rating, &overlays, graded);
    if line.prefix.is_empty() && line.score.is_none() {
        return;
    }
    let band = Rect::from_min_max(Pos2::new(rect.left(), rect.bottom() - size * 2.4), rect.max);
    crate::paint_vertical_gradient(
        &painter,
        Rect::from_min_max(Pos2::new(rect.left(), band.top() - size * 2.0), rect.max),
        Color32::TRANSPARENT,
        shade(Shade::Strong),
    );
    meta::meta_row(
        &painter,
        band.left() + size * 0.8,
        band.center().y,
        size,
        &line,
        text_primary(),
    );
}

pub(super) fn watched_mark(painter: &Painter, rect: Rect) {
    painter.circle_filled(rect.center(), rect.width() * 0.5, badge_fill());
    let at = |x: f32, y: f32| rect.min + rect.size() * Vec2::new(x, y);
    painter.line(
        vec![at(0.28, 0.52), at(0.44, 0.67), at(0.73, 0.36)],
        Stroke::new(rect.width() * 0.11, text_primary()),
    );
}

pub(super) fn saved_mark(painter: &Painter, rect: Rect) {
    painter.circle_filled(rect.center(), rect.width() * 0.5, badge_fill());
    let at = |x: f32, y: f32| rect.min + rect.size() * Vec2::new(x, y);
    painter.add(Shape::convex_polygon(
        vec![
            at(0.34, 0.26),
            at(0.66, 0.26),
            at(0.66, 0.74),
            at(0.5, 0.62),
        ],
        text_primary(),
        Stroke::NONE,
    ));
    painter.add(Shape::convex_polygon(
        vec![at(0.34, 0.26), at(0.5, 0.62), at(0.34, 0.74)],
        text_primary(),
        Stroke::NONE,
    ));
}
