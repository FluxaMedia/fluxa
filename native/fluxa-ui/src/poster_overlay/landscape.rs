use super::*;

pub(crate) fn paint_landscape(painter: &Painter, rect: Rect, card: &HomeCard) {
    let Some(overlays) = current(painter.ctx()) else {
        return;
    };
    let painter = painter.with_clip_rect(rect.intersect(painter.clip_rect()));
    let facts = &card.overlay;
    let inset = (rect.height() * 0.06).max(4.0);
    let size = (rect.height() * 0.09).clamp(8.0, 13.0) * overlays.scale;
    let graded = painter
        .ctx()
        .data(|data| data.get_temp::<Arc<Enrichment>>(enrichment_id()))
        .and_then(|enrichment| enrichment.graded(facts).and_then(|graded| graded.score));
    let rating = graded.filter(|_| overlays.mdblist_score).or(facts.rating);
    if let (true, Some(value)) = (overlays.rating.is_some(), rating) {
        let (source, name) = if rating == facts.rating {
            ("IMDb", "imdb")
        } else {
            ("★", "mdblist")
        };
        let logo = rating_logo(painter.ctx(), name);
        let text = match logo {
            Some(_) => format!("{value:.1}"),
            None => format!("{source} {value:.1}"),
        };
        let galley = painter.layout_no_wrap(text, FontId::proportional(size), Color32::WHITE);
        let logo_width = logo.map_or(0.0, |(_, aspect)| aspect.x * galley.size().y + size * 0.35);
        let (fill, _) = score_colors(value);
        let badge = Rect::from_min_size(
            rect.left_top() + Vec2::splat(inset),
            galley.size() + Vec2::new(size * 0.9 + logo_width, size * 0.5),
        );
        painter.rect_filled(badge, 3.0, Color32::from_black_alpha(190));
        painter.rect_stroke(badge, 3.0, Stroke::new(1.0, fill), egui::StrokeKind::Inside);
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
            Color32::WHITE,
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
    let rating = overlays.rating.and(
        painter
            .ctx()
            .data(|data| data.get_temp::<Arc<Enrichment>>(enrichment_id()))
            .and_then(|enrichment| enrichment.graded(facts).and_then(|graded| graded.score))
            .filter(|_| overlays.mdblist_score)
            .or(facts.rating),
    );
    let text = [
        (!facts.caption.is_empty()).then(|| facts.caption.clone()),
        rating.map(|value| format!("★ {value:.1}")),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>()
    .join(" · ");
    if text.is_empty() {
        return;
    }
    let band = Rect::from_min_max(Pos2::new(rect.left(), rect.bottom() - size * 2.4), rect.max);
    crate::paint_vertical_gradient(
        &painter,
        Rect::from_min_max(Pos2::new(rect.left(), band.top() - size * 2.0), rect.max),
        Color32::TRANSPARENT,
        Color32::from_black_alpha(200),
    );
    let inset = size * 0.8;
    let galley = painter.layout_no_wrap(text, FontId::proportional(size), Color32::WHITE);
    painter.galley(
        Pos2::new(band.left() + inset, band.center().y - galley.size().y * 0.5),
        galley,
        Color32::WHITE,
    );
}

pub(super) fn watched_mark(painter: &Painter, rect: Rect) {
    painter.circle_filled(
        rect.center(),
        rect.width() * 0.5,
        Color32::from_black_alpha(190),
    );
    let at = |x: f32, y: f32| rect.min + rect.size() * Vec2::new(x, y);
    painter.line(
        vec![at(0.28, 0.52), at(0.44, 0.67), at(0.73, 0.36)],
        Stroke::new(rect.width() * 0.11, Color32::WHITE),
    );
}

pub(super) fn saved_mark(painter: &Painter, rect: Rect) {
    painter.circle_filled(
        rect.center(),
        rect.width() * 0.5,
        Color32::from_black_alpha(190),
    );
    let at = |x: f32, y: f32| rect.min + rect.size() * Vec2::new(x, y);
    painter.add(Shape::convex_polygon(
        vec![
            at(0.34, 0.26),
            at(0.66, 0.26),
            at(0.66, 0.74),
            at(0.5, 0.62),
        ],
        Color32::WHITE,
        Stroke::NONE,
    ));
    painter.add(Shape::convex_polygon(
        vec![at(0.34, 0.26), at(0.5, 0.62), at(0.34, 0.74)],
        Color32::WHITE,
        Stroke::NONE,
    ));
}
