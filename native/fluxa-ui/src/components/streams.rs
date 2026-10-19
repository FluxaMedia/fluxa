use super::*;

struct StreamRowLayout {
    addon: Option<std::sync::Arc<egui::Galley>>,
    name: std::sync::Arc<egui::Galley>,
    detail: Option<std::sync::Arc<egui::Galley>>,
    badges: Vec<(std::sync::Arc<egui::Galley>, usize, Rect)>,
    badges_height: f32,
    height: f32,
}

pub(crate) fn badge_chip_layout(
    painter: &egui::Painter,
    badge: &crate::SourceBadge,
    font_size: f32,
) -> (std::sync::Arc<egui::Galley>, Vec2) {
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

pub(crate) fn paint_badge_chip(
    painter: &egui::Painter,
    chip: Rect,
    galley: std::sync::Arc<egui::Galley>,
    badge: &crate::SourceBadge,
    assets: &mut impl HomeAssets,
) {
    let rounding = chip.height() * 0.5;
    let text = if badge.outline {
        let color = badge.border.or(badge.fill).unwrap_or(ink(tone::dim()));
        painter.rect_stroke(
            chip,
            rounding,
            egui::Stroke::new(stroke::hairline(), color),
            egui::StrokeKind::Inside,
        );
        badge.text.or(badge.fill).unwrap_or(text_primary())
    } else {
        let fill = badge.fill.unwrap_or(ink(tone::soft()));
        painter.rect_filled(chip, rounding, fill);
        if let Some(border) = badge.border {
            painter.rect_stroke(
                chip,
                rounding,
                egui::Stroke::new(stroke::hairline(), border),
                egui::StrokeKind::Inside,
            );
        }
        badge.text.unwrap_or(match badge.fill {
            Some(fill) => readable_on(fill),
            None => ink(tone::text()),
        })
    };
    let mut left = chip.left() + (chip.width() - galley.size().x) * 0.5;
    if let Some(texture) = badge
        .image
        .as_deref()
        .and_then(|url| assets.texture(Some(url)))
    {
        let side = galley.size().y;
        let icon = Rect::from_min_size(
            Pos2::new(
                chip.left() + chip.height() * 0.35,
                chip.center().y - side * 0.5,
            ),
            Vec2::splat(side),
        );
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

const STREAM_ROW_PAD: f32 = 20.0;

fn stream_row_layout(
    painter: &egui::Painter,
    source: &crate::PlayerSource,
    show_addon: bool,
    width: f32,
    metrics: UiMetrics,
) -> StreamRowLayout {
    let text_width = (width - STREAM_ROW_PAD * 2.0).max(1.0);
    let addon = (show_addon && !source.name.starts_with(&source.addon)).then(|| {
        painter.layout_no_wrap(
            source.addon.clone(),
            crate::fonts::regular(metrics.screen_card_subtitle_size + 1.0),
            metrics.text_muted,
        )
    });
    let addon_width = addon.as_ref().map_or(0.0, |galley| galley.size().x + 12.0);
    let mut name_job = crate::emoji::job(
        &source.name,
        FontId::proportional(metrics.screen_card_title_size + 3.0),
        metrics.text_primary,
        text_width - addon_width,
    );
    name_job.wrap.max_rows = 3;
    let name = painter.layout_job(name_job);
    let detail = (!source.detail.is_empty()).then(|| {
        painter.layout_job(crate::emoji::job(
            &source.detail,
            crate::fonts::regular(metrics.screen_card_subtitle_size + 2.0),
            metrics.text_secondary,
            text_width,
        ))
    });
    let detail_height = detail.as_ref().map_or(0.0, |galley| galley.size().y + 8.0);
    let badge_size = metrics.screen_card_subtitle_size;
    let badge_gap = 6.0;
    let mut badges = Vec::with_capacity(source.badges.len());
    let (mut x, mut y) = (0.0, 0.0);
    let mut badges_height = 0.0;
    for (index, badge) in source.badges.iter().enumerate() {
        let (galley, size) = badge_chip_layout(painter, badge, badge_size);
        if x > 0.0 && x + size.x > text_width {
            x = 0.0;
            y += size.y + badge_gap;
        }
        badges.push((galley, index, Rect::from_min_size(Pos2::new(x, y), size)));
        x += size.x + badge_gap;
        badges_height = y + size.y;
    }
    if badges_height > 0.0 {
        badges_height += 12.0;
    }
    let height = STREAM_ROW_PAD * 2.0 + name.size().y + detail_height + badges_height;
    StreamRowLayout {
        addon,
        name,
        detail,
        badges,
        badges_height,
        height,
    }
}

pub(crate) fn stream_row(
    ui: &mut Ui,
    source: &crate::PlayerSource,
    show_addon: bool,
    width: f32,
    metrics: UiMetrics,
    assets: &mut impl HomeAssets,
) -> Response {
    let painter = ui.painter().clone();
    let StreamRowLayout {
        addon,
        name,
        detail,
        badges,
        badges_height,
        height,
    } = stream_row_layout(&painter, source, show_addon, width, metrics);
    let (rect, response) = ui.allocate_exact_size(Vec2::new(width, height), Sense::click());
    let fill = if response.is_pointer_button_down_on() {
        metrics.surface_raised
    } else {
        metrics.surface
    };
    painter.rect_filled(rect, metrics.card_radius, fill);
    painter.rect_stroke(
        rect,
        metrics.card_radius,
        egui::Stroke::new(stroke::hairline(), metrics.border),
        egui::StrokeKind::Inside,
    );
    let mut origin = rect.min + Vec2::splat(STREAM_ROW_PAD);
    let chips_origin = if source.badges_top {
        let top = origin;
        origin.y += badges_height;
        top
    } else {
        origin
    };
    let name_height = name.size().y;
    crate::emoji::paint(&painter, origin, name, assets);
    if let Some(addon) = addon {
        let pos = egui::Pos2::new(
            rect.right() - STREAM_ROW_PAD - addon.size().x,
            origin.y + 2.0,
        );
        painter.galley(pos, addon, metrics.text_primary);
    }
    let mut below = name_height + 8.0;
    if let Some(detail) = detail {
        let detail_height = detail.size().y;
        crate::emoji::paint(&painter, origin + Vec2::new(0.0, below), detail, assets);
        below += detail_height + 8.0;
    }
    let chips_y = if source.badges_top { 0.0 } else { below + 4.0 };
    for (galley, index, chip) in badges {
        let chip = chip.translate(chips_origin.to_vec2() + Vec2::new(0.0, chips_y));
        paint_badge_chip(&painter, chip, galley, &source.badges[index], assets);
    }
    response
}
