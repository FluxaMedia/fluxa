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

pub(crate) fn paint_badge_chip(
    painter: &crate::gfx::Painter,
    chip: Rect,
    galley: std::sync::Arc<crate::gfx::Galley>,
    badge: &crate::SourceBadge,
    assets: &mut impl HomeAssets,
) {
    paint_badge_chip_rounded(painter, chip, galley, badge, assets, chip.height() * 0.5);
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

fn stream_actions_width(source: &crate::PlayerSource, metrics: UiMetrics) -> f32 {
    if source.actions {
        metrics.streams_stream_action_size * 2.0 + metrics.streams_stream_action_gap + 12.0
    } else {
        0.0
    }
}

fn stream_row_layout(
    painter: &crate::gfx::Painter,
    source: &crate::PlayerSource,
    show_addon: bool,
    width: f32,
    metrics: UiMetrics,
) -> StreamRowLayout {
    let text_width = (width - metrics.streams_stream_row_pad * 2.0).max(1.0);
    let addon = (show_addon && !source.name.starts_with(&source.addon)).then(|| {
        painter.layout_no_wrap(
            source.addon.clone(),
            crate::fonts::regular(metrics.screen_card_subtitle_size + 1.0),
            metrics.text_muted,
        )
    });
    let addon_width = addon.as_ref().map_or(0.0, |galley| galley.size().x + 12.0)
        + stream_actions_width(source, metrics);
    let mut name_job = crate::emoji::job(
        &source.name,
        FontId::proportional(metrics.screen_card_title_size + 3.0),
        metrics.text_primary,
        text_width - addon_width,
    );
    name_job.wrap.max_rows = 3;
    let name = painter.layout_job(name_job);
    let detail = (!source.detail.is_empty()).then(|| {
        let font_size = metrics.screen_card_subtitle_size + 2.0;
        let mut job = crate::emoji::job(
            &source.detail,
            crate::fonts::regular(font_size),
            metrics.text_secondary,
            text_width,
        );
        for section in &mut job.sections {
            section.format.line_height = Some(font_size * 1.3);
        }
        painter.layout_job(job)
    });
    let detail_height = detail.as_ref().map_or(0.0, |galley| galley.size().y + 8.0);
    let badge_size = metrics.screen_card_subtitle_size;
    let badge_gap = 6.0;
    let chips: Vec<_> = source
        .badges
        .iter()
        .map(|badge| badge_chip_layout(painter, badge, badge_size))
        .collect();
    let flow = crate::flex::row(
        chips
            .iter()
            .enumerate()
            .map(|(index, (_, size))| crate::flex::leaf(*size).id(index)),
    )
    .wrap()
    .gap(badge_gap)
    .line_gap(badge_gap)
    .align(crate::flex::Align::Start)
    .solve(crate::gfx::rect_at(Pos2::ZERO, Vec2::new(text_width, 1.0e6)));
    let mut badges_height = 0.0;
    let badges = chips
        .into_iter()
        .enumerate()
        .map(|(index, (galley, size))| {
            let slot = flow
                .find(index)
                .unwrap_or(crate::gfx::rect_at(Pos2::ZERO, size));
            badges_height = f32::max(badges_height, slot.bottom());
            (galley, index, slot)
        })
        .collect::<Vec<_>>();
    if badges_height > 0.0 {
        badges_height += 12.0;
    }
    let height =
        metrics.streams_stream_row_pad * 2.0 + name.size().y + detail_height + badges_height;
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
) -> (Response, Option<StreamAction>) {
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
        crate::gfx::Stroke::new(stroke::hairline(), metrics.border),
        crate::gfx::StrokeKind::Inside,
    );
    let mut origin = rect.min + Vec2::splat(metrics.streams_stream_row_pad);
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
        let pos = crate::gfx::Pos2::new(
            rect.right()
                - metrics.streams_stream_row_pad
                - stream_actions_width(source, metrics)
                - addon.size().x,
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
    let mut action = None;
    if source.actions {
        let buttons = [
            (StreamAction::Copy, "Copy"),
            (StreamAction::Download, "Downloads"),
        ];
        let size = metrics.streams_stream_action_size;
        let top = rect.top() + metrics.streams_stream_row_pad - 4.0;
        let placed = crate::flex::row(
            buttons
                .iter()
                .enumerate()
                .rev()
                .map(|(index, _)| crate::flex::leaf(Vec2::splat(size)).id(index).shrink(0.0)),
        )
        .gap(metrics.streams_stream_action_gap)
        .justify(crate::flex::Justify::End)
        .solve(Rect::from_min_max(
            Pos2::new(rect.left(), top),
            Pos2::new(rect.right() - metrics.streams_stream_row_pad, top + size),
        ));
        for (index, (kind, icon)) in buttons.into_iter().enumerate() {
            let slot = placed.find(index).unwrap_or(Rect::NOTHING);
            let clicked = ui
                .scope_builder(crate::gfx::UiBuilder::new().max_rect(slot), |ui| {
                    icon_button(
                        ui,
                        assets.icon(icon),
                        size,
                        metrics.text_secondary,
                        true,
                        false,
                        true,
                    )
                })
                .inner
                .clicked();
            if clicked {
                action = Some(kind);
            }
        }
    }
    (response, action)
}
