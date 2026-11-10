use super::*;
use crate::Space;
use crate::dp::Dp;

pub(crate) fn poster_card(
    painter: &Painter,
    rect: Rect,
    card: &HomeCard,
    metrics: UiMetrics,
    assets: &mut impl HomeAssets,
    motion_active: bool,
) {
    if !painter.clip_rect().intersects(rect) {
        if near_screen(painter, rect) {
            crate::card_logos::want(card);
            let target = artwork_target_size(rect.size(), painter.ctx().pixels_per_point());
            assets.prefetch_for(card.poster_art(), target, ArtworkPriority::Prefetch);
        }
        return;
    }
    crate::card_logos::want(card);
    crate::motion::press_scale(painter, rect, || {
        poster_card_body(painter, rect, card, metrics, assets, motion_active, true)
    });
}

fn expanded_bounds(height: f32) -> (Vec2, Vec2) {
    let wide = height * 16.0 / 9.0;
    (
        Vec2::new(wide, height),
        Vec2::new(wide * 0.4, height * 0.26),
    )
}

pub(crate) fn prefetch_expanded(
    context: &crate::gfx::Context,
    card: &HomeCard,
    height: f32,
    priority: ArtworkPriority,
    assets: &mut impl HomeAssets,
) {
    let (backdrop, logo) = expanded_bounds(height);
    let ppp = context.pixels_per_point();
    crate::card_logos::want(card);
    assets.prefetch_for(
        card.backdrop_url.as_deref().or(card.artwork_url.as_deref()),
        artwork_target_size(backdrop, ppp),
        priority,
    );
    assets.prefetch_for(
        crate::card_logos::logo_of(card).as_deref(),
        artwork_target_size(logo, ppp),
        priority,
    );
}

pub(crate) fn expanded_poster_card(
    painter: &Painter,
    rect: Rect,
    card: &HomeCard,
    progress: f32,
    trailer: Option<crate::gfx::TextureId>,
    metrics: UiMetrics,
    assets: &mut impl HomeAssets,
) {
    crate::motion::press_scale(painter, rect, || {
        poster_card_body(painter, rect, card, metrics, assets, false, false);
        let radius = metrics.card_radius;
        let alpha = (255.0 * progress) as u8;
        let url = card.backdrop_url.as_deref().or(card.artwork_url.as_deref());
        let (backdrop_bounds, logo_bounds) = expanded_bounds(rect.height());
        let image = rect;
        rounded_artwork(
            painter,
            image,
            radius,
            url,
            artwork_target_size(backdrop_bounds, painter.ctx().pixels_per_point()),
            ArtworkPriority::Visible,
            Color32::from_white_alpha(alpha),
            assets,
        );
        if let Some(texture) = trailer {
            let fade = crate::motion::appear(
                painter.ctx(),
                Id::new(("fluxa-poster-trailer-fade", card.id.as_deref())),
                crate::motion::slow() * 2.0,
                crate::motion::Ease::Linear,
            );
            rounded_image(
                painter,
                texture,
                image,
                radius,
                cover_uv([1920, 1080], image),
                Color32::from_white_alpha((f32::from(alpha) * fade) as u8),
            );
        }
        let scrim = crate::flex::anchored(
            rect,
            Align2::CENTER_BOTTOM,
            Vec2::new(
                rect.width(),
                rect.height() * metrics.surface_card_logo_scrim_share,
            ),
            0.0,
        );
        paint_vertical_gradient(
            painter,
            scrim,
            Color32::TRANSPARENT,
            Color32::from_black_alpha((180.0 * progress) as u8),
        );
        let pad = Dp::D14.px();
        let logo_url = crate::card_logos::logo_of(card);
        let logo = title_logo(
            painter.ctx().pixels_per_point(),
            logo_url.as_deref(),
            logo_bounds,
            ArtworkPriority::Visible,
            assets,
        );
        match logo {
            Some((texture, size)) => texture_image(
                painter,
                texture,
                crate::flex::anchored(rect, Align2::LEFT_BOTTOM, size, pad),
                full_uv(),
                Color32::from_white_alpha(alpha),
            ),
            None if failed_dom_image(
                painter,
                assets,
                crate::flex::anchored(rect, Align2::LEFT_BOTTOM, logo_bounds, pad),
                0.0,
                logo_url.as_deref(),
                artwork_target_size(logo_bounds, painter.ctx().pixels_per_point()),
                ImageFit::Contain,
                [0.0, 1.0],
            ) => {}
            None if logo_pending(
                card,
                logo_url.as_deref(),
                artwork_target_size(logo_bounds, painter.ctx().pixels_per_point()),
                assets,
            ) => {}
            None => {
                let size = metrics.screen_card_title_size + 4.0;
                paint_elided_text(
                    painter,
                    crate::flex::anchored(rect, Align2::LEFT_BOTTOM, Vec2::splat(size), pad).min,
                    &card.title,
                    FontId::proportional(size),
                    rect.width() * 0.6,
                    metrics.ink(alpha),
                );
            }
        }
    });
}

pub(crate) fn expanded_info_panel(
    context: &crate::gfx::Context,
    node: u64,
    rect: Rect,
    clip: Rect,
    meta: &str,
    synopsis: &str,
    progress: f32,
    metrics: UiMetrics,
) {
    let pad = 4.0;
    let body = preview_body_size(metrics);
    let alpha = (255.0 * progress) as u8;
    let mut job = crate::gfx::text::LayoutJob::single_section(
        synopsis.to_owned(),
        crate::gfx::TextFormat::simple(crate::fonts::regular(body), Color32::WHITE),
    );
    job.wrap.max_width = rect.width() - pad * 2.0;
    job.wrap.max_rows = 2;
    let galley = context.fonts_mut(|fonts| fonts.layout_job(job));
    let meta_height = if meta.is_empty() {
        0.0
    } else {
        body * metrics.surface_card_line_height + Space::S6.px(&metrics)
    };
    let top_gap = metrics.focus_ring_expand + metrics.focus_ring_width + Space::S8.px(&metrics);
    let full = top_gap + meta_height + galley.size().y;
    let origin = rect.left_bottom();
    crate::gfx::Area::new(Id::new(("fluxa-expanded-info", node)))
        .order(crate::gfx::Order::Tooltip)
        .fixed_pos(origin)
        .interactable(false)
        .constrain(false)
        .show(context, |ui| {
            let panel = Rect::from_min_size(origin, Vec2::new(rect.width(), full * progress));
            let painter = ui.painter().with_clip_rect(clip.intersect(panel));
            let (meta_id, synopsis_id) = (1, 2);
            let copy = crate::flex::column([
                crate::flex::leaf(Vec2::new(rect.width() - pad * 2.0, meta_height)).id(meta_id),
                crate::flex::leaf(galley.size()).id(synopsis_id),
            ])
            .padding(crate::flex::Insets {
                left: pad,
                top: top_gap,
                right: 0.0,
                bottom: 0.0,
            })
            .solve(Rect::from_min_size(origin, Vec2::new(rect.width(), full)));
            if !meta.is_empty() {
                paint_elided_text(
                    &painter,
                    copy.find(meta_id).unwrap_or(panel).min,
                    meta,
                    crate::fonts::regular(body),
                    rect.width() - pad * 2.0,
                    metrics.ink(alpha),
                );
            }
            painter.galley_with_override_text_color(
                copy.find(synopsis_id).unwrap_or(panel).min,
                galley,
                metrics.ink(((150.0 * progress) as u8).max(1)),
            );
        });
}

pub(crate) fn preview_is_compact(card: &HomeCard) -> bool {
    card.row_kind == crate::HomeRowKind::Continue
}

fn preview_image_height(width: f32, compact: bool) -> f32 {
    if compact {
        width * 9.0 / 16.0 * 0.8
    } else {
        width * 9.0 / 16.0
    }
}

fn preview_body_size(metrics: UiMetrics) -> f32 {
    (metrics.screen_card_subtitle_size + 3.0).max(15.0)
}

fn logo_pending(
    card: &HomeCard,
    url: Option<&str>,
    target: [u32; 2],
    assets: &impl HomeAssets,
) -> bool {
    match url {
        Some(url) => !assets.artwork_failed(Some(url), target),
        None => crate::card_logos::awaiting(card),
    }
}

pub(crate) fn hover_preview_reset(context: &crate::gfx::Context, order: crate::gfx::Order) {
    crate::motion::reset(context, Id::new(("fluxa-hover-preview-appear", order)));
}

pub(crate) fn hover_preview_rect(
    context: &crate::gfx::Context,
    anchor: Rect,
    screen: Rect,
    synopsis: &str,
    compact: bool,
    metrics: UiMetrics,
) -> (Rect, std::sync::Arc<crate::gfx::Galley>) {
    let width = if compact {
        metrics.card_popover_min.of(anchor.width())
    } else {
        metrics.card_popover_max.of(anchor.width())
    };
    let margin = 12.0;
    let body = preview_body_size(metrics);
    let image_height = preview_image_height(width, compact);
    let fixed = image_height
        + body
        + 12.0
        + 16.0
        + metrics.cards_preview_button_height
        + metrics.cards_preview_pad * 2.0;
    let line = body * 1.25;
    let rows = (((screen.height() - margin * 2.0 - fixed) / line).floor() as usize).max(1);
    let mut job = crate::gfx::text::LayoutJob::single_section(
        synopsis.to_owned(),
        crate::gfx::TextFormat::simple(crate::fonts::regular(body), metrics.ink(tone::subtle())),
    );
    job.wrap.max_width = width - metrics.cards_preview_pad * 2.0;
    job.wrap.max_rows = rows;
    let galley = context.fonts_mut(|fonts| fonts.layout_job(job));
    let height = fixed + galley.size().y;
    let left = screen.left() + margin;
    let top = screen.top() + margin;
    let x =
        (anchor.center().x - width * 0.5).clamp(left, (screen.right() - margin - width).max(left));
    let y =
        (anchor.center().y - height * 0.5).clamp(top, (screen.bottom() - margin - height).max(top));
    (
        Rect::from_min_size(Pos2::new(x, y), Vec2::new(width, height)),
        galley,
    )
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum PreviewAction {
    Play,
    Watchlist,
    Details,
    Watched,
    ClearProgress,
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn hover_preview(
    context: &crate::gfx::Context,
    rect: Rect,
    card: &HomeCard,
    trailer: Option<crate::gfx::TextureId>,
    meta: &str,
    synopsis: std::sync::Arc<crate::gfx::Galley>,
    play_label: &str,
    progress: Option<f32>,
    quick_language: Option<&str>,
    order: crate::gfx::Order,
    metrics: UiMetrics,
    assets: &mut impl HomeAssets,
) -> (Response, Option<PreviewAction>) {
    let personal = card
        .id
        .as_deref()
        .and_then(|id| crate::poster_overlay::personal_for(context, id));
    let saved = personal.as_ref().is_some_and(|personal| personal.saved);
    let watched = personal.as_ref().is_some_and(|personal| personal.watched);
    let has_progress = progress.is_some()
        || card.progress > 0.0
        || personal
            .as_ref()
            .is_some_and(|personal| personal.progress > 0.0);
    let title_size = (metrics.screen_card_title_size + 7.0).max(22.0);
    let body_size = preview_body_size(metrics);
    let button_size = body_size + 1.0;
    let appear = crate::motion::appear(
        context,
        Id::new(("fluxa-hover-preview-appear", order)),
        crate::motion::normal(),
        crate::motion::Ease::Out,
    );
    let scale = 0.92 + 0.08 * appear;
    context.set_transform_layer(
        crate::gfx::LayerId::new(order, Id::new("fluxa-hover-preview")),
        crate::gfx::emath::TSTransform::new(rect.center().to_vec2() * (1.0 - scale), scale),
    );
    crate::gfx::Area::new(Id::new("fluxa-hover-preview"))
        .order(order)
        .fixed_pos(rect.min)
        .constrain(false)
        .show(context, |ui| {
            ui.set_opacity(appear);
            let response = ui.allocate_rect(rect, Sense::click());
            let radius = metrics.card_radius;
            let corner = crate::gfx::CornerRadius::from(radius);
            let painter = ui.painter().clone();
            painter.rect_filled(rect.expand(1.0), radius + 1.0, shade(Shade::Veil));
            painter.rect_filled(rect, radius, metrics.surface);

            let image_height = preview_image_height(rect.width(), preview_is_compact(card));
            let image =
                Rect::from_min_size(rect.min, Vec2::new(rect.width(), image_height + radius));
            let episode_still = preview_is_compact(card)
                .then(|| {
                    card.raw
                        .get("lastEpisodeThumbnail")
                        .and_then(serde_json::Value::as_str)
                        .filter(|url| !url.is_empty())
                })
                .flatten();
            let url = episode_still
                .or(card.backdrop_url.as_deref())
                .or(card.artwork_url.as_deref());
            rounded_artwork(
                &painter,
                image,
                radius,
                url,
                artwork_target_size(image.size(), context.pixels_per_point()),
                ArtworkPriority::Visible,
                Color32::WHITE,
                assets,
            );
            if let Some(texture) = trailer {
                let fade = crate::motion::appear(
                    context,
                    Id::new(("fluxa-hover-trailer-fade", card.id.as_deref())),
                    crate::motion::slow() * 2.0,
                    crate::motion::Ease::Linear,
                );
                rounded_image(
                    &painter,
                    texture,
                    image,
                    radius,
                    cover_uv([1920, 1080], image),
                    Color32::from_white_alpha((255.0 * fade) as u8),
                );
            }
            let (image_id, info_id) = (1, 2);
            let split = crate::flex::column([
                crate::flex::leaf(Vec2::new(rect.width(), image_height)).id(image_id),
                crate::flex::fill().id(info_id),
            ])
            .solve(rect);
            let image_slot = split.find(image_id).unwrap_or(rect);
            let info = split.find(info_id).unwrap_or(rect);
            painter.rect_filled(
                info,
                crate::gfx::CornerRadius {
                    nw: 0,
                    ne: 0,
                    sw: corner.sw,
                    se: corner.se,
                },
                metrics.surface,
            );
            let scrim = Rect::from_min_max(
                Pos2::new(
                    rect.left(),
                    rect.top() + image_height * metrics.surface_card_scrim_share,
                ),
                Pos2::new(rect.right(), info.top() + stroke::hairline()),
            );
            paint_vertical_gradient(&painter, scrim, Color32::TRANSPARENT, metrics.surface);
            painter.rect_stroke(
                rect,
                radius,
                crate::gfx::Stroke::new(stroke::hairline(), metrics.border),
                crate::gfx::StrokeKind::Inside,
            );

            let text_width = rect.width() - metrics.cards_preview_pad * 2.0;
            let preview_logo = Vec2::new(
                rect.width() * metrics.surface_card_preview_logo_width_share,
                image_height * metrics.surface_card_preview_logo_height_share,
            );
            let logo_inset = crate::flex::Insets {
                left: metrics.cards_preview_pad,
                top: 0.0,
                right: 0.0,
                bottom: Space::S12.px(&metrics),
            };
            crate::card_logos::want(card);
            let logo_url = crate::card_logos::logo_of(card);
            let logo = title_logo(
                context.pixels_per_point(),
                logo_url.as_deref(),
                preview_logo,
                ArtworkPriority::Visible,
                assets,
            );
            match logo {
                Some((texture, size)) => texture_image(
                    &painter,
                    texture,
                    pinned(image_slot, Align2::LEFT_BOTTOM, size, logo_inset),
                    full_uv(),
                    Color32::WHITE,
                ),
                None if failed_dom_image(
                    &painter,
                    assets,
                    pinned(image_slot, Align2::LEFT_BOTTOM, preview_logo, logo_inset),
                    0.0,
                    logo_url.as_deref(),
                    artwork_target_size(preview_logo, context.pixels_per_point()),
                    ImageFit::Contain,
                    [0.0, 1.0],
                ) => {}
                None if logo_pending(
                    card,
                    logo_url.as_deref(),
                    artwork_target_size(preview_logo, context.pixels_per_point()),
                    assets,
                ) => {}
                None => paint_elided_text(
                    &painter,
                    pinned(
                        image_slot,
                        Align2::LEFT_BOTTOM,
                        Vec2::splat(title_size),
                        crate::flex::Insets {
                            bottom: Space::S14.px(&metrics),
                            ..logo_inset
                        },
                    )
                    .min,
                    &card.title,
                    crate::fonts::medium(title_size),
                    text_width,
                    metrics.text_primary,
                ),
            }

            let rating = card.overlay.rating.map(|rating| format!("{rating:.1}/10"));
            let meta_line = [rating.as_deref().unwrap_or(""), meta]
                .iter()
                .filter(|part| !part.is_empty())
                .copied()
                .collect::<Vec<_>>()
                .join("  ·  ");
            let (meta_id, synopsis_id) = (1, 2);
            let copy = crate::flex::column([
                crate::flex::leaf(Vec2::new(text_width, body_size)).id(meta_id),
                crate::flex::leaf(synopsis.size()).id(synopsis_id),
            ])
            .gap(Space::S12.px(&metrics))
            .padding(crate::flex::Insets {
                left: metrics.cards_preview_pad,
                top: Space::S2.px(&metrics),
                right: 0.0,
                bottom: 0.0,
            })
            .solve(info);
            if !meta_line.is_empty() {
                paint_elided_text(
                    &painter,
                    copy.find(meta_id).unwrap_or(info).min,
                    &meta_line,
                    crate::fonts::regular(body_size),
                    text_width,
                    metrics.ink(tone::body()),
                );
            }
            painter.galley(
                copy.find(synopsis_id).unwrap_or(info).min,
                synopsis,
                metrics.text_primary,
            );

            let mut icons = vec![(
                PreviewAction::Watchlist,
                if saved { "Check" } else { "Plus" },
                if saved {
                    "menu.remove_watchlist"
                } else {
                    "menu.add_watchlist"
                },
            )];
            if quick_language.is_some() {
                icons.push((PreviewAction::Details, "Info", "menu.details"));
                if card.item_type.as_deref() == Some("movie") {
                    icons.push(if watched {
                        (PreviewAction::Watched, "EyeOff", "menu.mark_unwatched")
                    } else {
                        (PreviewAction::Watched, "CircleCheck", "menu.mark_watched")
                    });
                }
                if has_progress {
                    icons.push((
                        PreviewAction::ClearProgress,
                        "Close",
                        "menu.remove_continue",
                    ));
                }
            }
            let mut action = None;
            let button = metrics.cards_preview_button_height;
            let play_id = 0;
            let footer = crate::flex::anchored(
                rect,
                Align2::CENTER_BOTTOM,
                Vec2::new(text_width, button),
                metrics.cards_preview_pad,
            );
            let controls = crate::flex::row(
                std::iter::once(crate::flex::fill().height(button).id(play_id)).chain(
                    (1..=icons.len())
                        .rev()
                        .map(|index| crate::flex::leaf(Vec2::splat(button)).id(index)),
                ),
            )
            .gap(Space::S10.px(&metrics))
            .solve(footer);
            let play_rect = controls.find(play_id).unwrap_or(footer);
            for (index, (kind, icon_name, label_key)) in icons.into_iter().enumerate() {
                let icon_rect = controls.find(index + 1).unwrap_or(footer);
                let mut pill = ui.interact(
                    icon_rect,
                    Id::new(("fluxa-hover-preview-pill", label_key)),
                    Sense::click(),
                );
                if let Some(language) = quick_language {
                    pill = pill.on_hover_text(crate::localized(label_key, language));
                }
                let fill = if pill.hovered() { 46 } else { 28 };
                painter.rect_filled(icon_rect, button * 0.5, metrics.ink(fill));
                if let Some(icon) = assets.icon(icon_name) {
                    painter.image(
                        icon,
                        crate::flex::anchored(
                            icon_rect,
                            Align2::CENTER_CENTER,
                            Vec2::splat(Dp::D20.px()),
                            0.0,
                        ),
                        full_uv(),
                        Color32::WHITE,
                    );
                }
                if pill.clicked() {
                    action = Some(kind);
                }
            }
            let play = ui
                .scope_builder(crate::gfx::UiBuilder::new().max_rect(play_rect), |ui| {
                    play_button(
                        ui,
                        assets,
                        play_label,
                        Some(play_rect.width()),
                        metrics.cards_preview_button_height,
                        button_size,
                        progress,
                    )
                })
                .inner;
            if play.clicked() {
                action = Some(PreviewAction::Play);
            }
            (response, action)
        })
        .inner
}

pub(crate) fn library_row(
    painter: &Painter,
    rect: Rect,
    card: &HomeCard,
    metrics: UiMetrics,
    assets: &mut impl HomeAssets,
) {
    painter.rect_filled(rect, metrics.card_radius, metrics.surface);
    painter.rect_stroke(
        rect,
        metrics.card_radius,
        crate::gfx::Stroke::new(stroke::hairline(), metrics.border),
        crate::gfx::StrokeKind::Inside,
    );
    let inset = metrics.control_gap;
    let thumb_height = rect.height() - inset * 2.0;
    let (thumb_id, text_id) = (1, 2);
    let placed = crate::flex::row([
        crate::flex::leaf(Vec2::new(thumb_height * 2.0 / 3.0, thumb_height)).id(thumb_id),
        crate::flex::fill().id(text_id),
    ])
    .gap(inset * metrics.surface_card_row_gap_share)
    .padding(crate::flex::Insets::all(inset))
    .solve(rect);
    let thumb = placed.find(thumb_id).unwrap_or(rect);
    let text_area = placed.find(text_id).unwrap_or(rect);
    poster_card_body(painter, thumb, card, metrics, assets, false, false);
    let text_x = text_area.left();
    let text_width = text_area.width();
    let title_size = metrics.screen_card_title_size + Space::S2.px(&metrics);
    let title_y = rect.center().y - title_size - metrics.control_gap * 0.5;
    paint_elided_text(
        painter,
        crate::gfx::Pos2::new(text_x, title_y),
        &card.title,
        crate::fonts::medium(title_size),
        text_width,
        metrics.text_primary,
    );
    paint_elided_text(
        painter,
        crate::gfx::Pos2::new(text_x, title_y + title_size + Space::S4.px(&metrics)),
        &card.subtitle,
        crate::fonts::regular(metrics.screen_card_subtitle_size),
        text_width,
        metrics.ink(tone::subtle()),
    );
    if card.progress > 0.0 {
        let bar = Rect::from_min_size(
            crate::gfx::Pos2::new(text_x, thumb.bottom() - metrics.card_progress_height),
            Vec2::new(
                text_width.min(metrics.surface_card_row_bar_max),
                metrics.card_progress_height,
            ),
        );
        painter.rect_filled(bar, radius::BAR, metrics.ink(tone::raised()));
        painter.rect_filled(
            Rect::from_min_size(
                bar.min,
                Vec2::new(bar.width() * card.progress.clamp(0.0, 1.0), bar.height()),
            ),
            Dp::D2.px(),
            metrics.accent,
        );
    }
}

fn poster_card_body(
    painter: &Painter,
    rect: Rect,
    card: &HomeCard,
    metrics: UiMetrics,
    assets: &mut impl HomeAssets,
    motion_active: bool,
    labels: bool,
) {
    painter.rect_filled(rect, metrics.card_radius, metrics.surface);

    let landscape = card.is_landscape();
    let artwork = card.poster_art();
    // Show the ordinary poster as soon as it is ready. Animated artwork can
    // take substantially longer to prepare, so don't let its decode leave a
    // blank tile or occupy the first artwork requests for an entire row.
    let target = artwork_target_size(
        Vec2::new(rect.width(), rect.height()),
        painter.ctx().pixels_per_point(),
    );
    let custom = crate::poster_overlay::custom_url(painter.ctx(), card);
    let custom_ready = custom.as_deref().is_some_and(|url| {
        assets
            .texture_for(Some(url), target, ArtworkPriority::Visible)
            .is_some()
    });
    let has_static_artwork = if custom_ready {
        rounded_artwork(
            painter,
            rect,
            metrics.card_radius,
            custom.as_deref(),
            target,
            ArtworkPriority::Visible,
            crate::stylesheet::color("text"),
            assets,
        )
    } else {
        rounded_artwork(
            painter,
            rect,
            metrics.card_radius,
            artwork,
            target,
            ArtworkPriority::Visible,
            crate::stylesheet::color("text"),
            assets,
        )
    };
    let motion_wanted = motion_active && (has_static_artwork || card.artwork_url.is_none());
    let dom_motion = if motion_wanted
        && assets.dom_images()
        && let Some(url) = card.motion_url.as_deref()
    {
        dom_image(painter, rect, metrics.card_radius, url);
        true
    } else {
        false
    };
    let motion_texture = if motion_wanted
        && !dom_motion
        && let Some(url) = card.motion_url.as_deref()
    {
        assets.animated_texture_for(
            Some(url),
            fluxa_artwork::animation_target_size(artwork_target_size(
                Vec2::new(rect.width(), rect.height()),
                painter.ctx().pixels_per_point(),
            )),
            ArtworkPriority::Visible,
        )
    } else {
        None
    };
    if let Some(animated) = motion_texture {
        // Animated artwork is a replacement for the static tile, not a
        // transparent overlay on top of it. Clear first so stale poster pixels
        // can never show through transparent animation frames.
        let crop = crate::cover_uv(animated.image_size, rect);
        let tile = animated.uv;
        let uv = Rect::from_min_max(
            crate::gfx::Pos2::new(
                tile.min.x + crop.min.x * tile.width(),
                tile.min.y + crop.min.y * tile.height(),
            ),
            crate::gfx::Pos2::new(
                tile.min.x + crop.max.x * tile.width(),
                tile.min.y + crop.max.y * tile.height(),
            ),
        );
        texture_image(painter, animated.texture, rect, uv, Color32::WHITE);
    }
    if landscape {
        if crate::poster_overlay::custom_url(painter.ctx(), card).is_none() {
            landscape_logo(painter, rect, card, assets);
        }
        crate::poster_overlay::paint_landscape_band(painter, rect, card);
    } else {
        if let Some(logo) =
            crate::poster_overlay::textless_art(painter.ctx(), card).and_then(|art| art.logo)
        {
            textless_logo(painter, rect, &logo, assets);
        }
        let tones = assets.artwork_tones(card.artwork_url.as_deref());
        crate::poster_overlay::paint(painter, rect, card, metrics.card_radius, tones);
    }

    if !labels || card.hide_title && card.row_kind == crate::HomeRowKind::Collection {
        return;
    }
    let title_font = crate::fonts::medium(metrics.screen_card_title_size);
    let subtitle_font = crate::fonts::regular(metrics.screen_card_subtitle_size);
    let (title_id, subtitle_id) = (1, 2);
    let labels = crate::flex::column([
        crate::flex::leaf(Vec2::new(rect.width(), metrics.screen_card_title_size)).id(title_id),
        crate::flex::leaf(Vec2::new(rect.width(), metrics.screen_card_subtitle_size))
            .id(subtitle_id),
    ])
    .gap(Space::S2.px(&metrics))
    .padding(crate::flex::Insets {
        left: 0.0,
        top: metrics.control_gap,
        right: 0.0,
        bottom: 0.0,
    });
    let below = Rect::from_min_size(rect.left_bottom(), labels.outer_size());
    let placed = labels.solve(below);
    paint_elided_text(
        painter,
        placed.find(title_id).unwrap_or(below).min,
        &card.title,
        title_font,
        rect.width(),
        metrics.text_primary,
    );
    paint_elided_text(
        painter,
        placed.find(subtitle_id).unwrap_or(below).min,
        &card.subtitle,
        subtitle_font,
        rect.width(),
        metrics.ink(tone::subtle()),
    );
}

fn textless_logo(painter: &Painter, rect: Rect, url: &str, assets: &mut impl HomeAssets) {
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

fn landscape_logo(painter: &Painter, rect: Rect, card: &HomeCard, assets: &mut impl HomeAssets) {
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

pub(super) fn pinned(bounds: Rect, align: Align2, size: Vec2, insets: crate::flex::Insets) -> Rect {
    crate::flex::stack(align, [crate::flex::leaf(size).id(0)])
        .padding(insets)
        .solve(bounds)
        .find(0)
        .unwrap_or(Rect::from_min_size(bounds.min, size))
}

/// Lay out poster labels once with the backend's native single-line ellipsis. The
/// previous path measured each string repeatedly to find a fitting prefix,
/// which multiplied text shaping work across every visible result per frame.
pub(super) fn paint_elided_text(
    painter: &Painter,
    position: crate::gfx::Pos2,
    text: &str,
    font: FontId,
    max_width: f32,
    color: Color32,
) {
    // the backend caches font shaping internally, but building a LayoutJob and
    // requesting a new galley for every visible card on every frame still
    // costs enough to hitch while a page of posters enters the viewport.
    // Keep the already-laid-out galley in the backend's per-context temp store.
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    text.hash(&mut hasher);
    font.hash(&mut hasher);
    max_width.to_bits().hash(&mut hasher);
    color.to_array().hash(&mut hasher);
    painter.ctx().pixels_per_point().to_bits().hash(&mut hasher);
    let galley_id = Id::new("fluxa-poster-label-galley").with(hasher.finish());
    let galley = painter
        .ctx()
        .data(|data| data.get_temp::<std::sync::Arc<crate::gfx::Galley>>(galley_id));
    let galley = galley.unwrap_or_else(|| {
        let mut job = crate::gfx::text::LayoutJob::simple_singleline(text.to_owned(), font, color);
        job.wrap = crate::gfx::text::TextWrapping::truncate_at_width(max_width);
        let galley = painter.layout_job(job);
        painter
            .ctx()
            .data_mut(|data| data.insert_temp(galley_id, galley.clone()));
        galley
    });
    painter.galley(position, galley, color);
}

pub(crate) fn skeleton_card(painter: &Painter, rect: Rect, metrics: UiMetrics) {
    let time = painter.ctx().input(|input| input.time) as f32;
    let pulse = 0.5 - 0.5 * (time * std::f32::consts::TAU / 1.6).cos();
    let low = f32::from(tone::raised());
    let high = f32::from(tone::strong());
    painter.rect_filled(
        rect,
        metrics.card_radius,
        ink((low + (high - low) * pulse) as u8),
    );
    painter
        .ctx()
        .request_repaint_after(crate::motion::HALF_FRAME_RATE);
}

pub(crate) fn continue_card(
    painter: &Painter,
    rect: Rect,
    artwork_size: Vec2,
    card: &HomeCard,
    metrics: UiMetrics,
    assets: &mut impl HomeAssets,
) {
    crate::motion::press_scale(painter, rect, || {
        continue_card_body(painter, rect, artwork_size, card, metrics, assets)
    });
}

fn continue_card_body(
    painter: &Painter,
    rect: Rect,
    artwork_size: Vec2,
    card: &HomeCard,
    metrics: UiMetrics,
    assets: &mut impl HomeAssets,
) {
    painter.rect_filled(rect, metrics.card_radius, metrics.surface);

    artwork_image(
        painter,
        rect,
        card.artwork_url.as_deref(),
        artwork_target_size(artwork_size, painter.ctx().pixels_per_point()),
        ArtworkPriority::Visible,
        crate::stylesheet::color("text"),
        assets,
    );

    let scrim_top = rect.top() + rect.height() * metrics.surface_card_continue_scrim_top;
    let scrim_mid = rect.top() + rect.height() * metrics.surface_card_continue_scrim_mid;
    paint_vertical_gradient(
        painter,
        Rect::from_min_max(
            crate::gfx::Pos2::new(rect.left(), scrim_top),
            crate::gfx::Pos2::new(rect.right(), scrim_mid),
        ),
        Color32::TRANSPARENT,
        shade(Shade::Scrim),
    );
    paint_vertical_gradient(
        painter,
        Rect::from_min_max(
            crate::gfx::Pos2::new(rect.left(), scrim_mid),
            rect.right_bottom(),
        ),
        shade(Shade::Scrim),
        shade(Shade::Opaque),
    );
    let title_font = crate::fonts::medium(metrics.screen_card_title_size);
    let subtitle_font = crate::fonts::regular(metrics.screen_card_subtitle_size);
    let content_width = (rect.width() - metrics.card_content_padding * 2.0).max(1.0);
    let has_subtitle = !card.subtitle.is_empty();
    let subtitle_height = if has_subtitle {
        metrics.screen_card_subtitle_size * metrics.surface_card_line_height
    } else {
        0.0
    };
    let title_height = metrics.screen_card_title_size * metrics.surface_card_line_height;
    let (title_id, subtitle_id, bar_id) = (1, 2, 3);
    let mut stacked = vec![crate::flex::leaf(Vec2::new(content_width, title_height)).id(title_id)];
    if has_subtitle {
        stacked.push(crate::flex::leaf(Vec2::new(
            0.0,
            metrics.control_gap * metrics.surface_card_title_gap_share,
        )));
        stacked.push(crate::flex::leaf(Vec2::new(content_width, subtitle_height)).id(subtitle_id));
    }
    stacked.push(crate::flex::leaf(Vec2::new(
        0.0,
        metrics.control_gap * metrics.surface_card_bar_gap_share,
    )));
    stacked
        .push(crate::flex::leaf(Vec2::new(content_width, metrics.card_progress_height)).id(bar_id));
    let placed = crate::flex::column(stacked)
        .justify(crate::flex::Justify::End)
        .padding(crate::flex::Insets::all(metrics.card_content_padding))
        .solve(rect);
    let rect_of = |id| placed.find(id).unwrap_or(rect);
    draw_text(
        painter,
        rect_of(title_id).min,
        Align2::LEFT_TOP,
        &card.title,
        title_font,
        metrics.text_primary,
        Some(content_width),
    );
    if has_subtitle {
        draw_text(
            painter,
            rect_of(subtitle_id).min,
            Align2::LEFT_TOP,
            &card.subtitle,
            subtitle_font.clone(),
            metrics.ink(tone::body()),
            Some(content_width),
        );
    }
    let badge_pad = metrics.screen_card_subtitle_size * metrics.surface_card_badge_pad_share;
    let badge_height =
        metrics.screen_card_subtitle_size + badge_pad * metrics.surface_card_badge_height_share;
    let inset = (rect.height() * metrics.surface_card_badge_inset_share).max(Dp::D4.px());
    for (text, right) in [(&card.episodes, false), (&card.detail, true)] {
        if text.is_empty() {
            continue;
        }
        let galley = painter.layout_no_wrap(text.clone(), subtitle_font.clone(), Color32::WHITE);
        let size = Vec2::new(galley.size().x + badge_pad * 2.0, badge_height);
        let badge = crate::flex::anchored(
            rect,
            if right {
                Align2::RIGHT_TOP
            } else {
                Align2::LEFT_TOP
            },
            size,
            inset,
        );
        overlay_chip(painter, badge, None);
        painter.galley(
            badge.center() - galley.size() * 0.5,
            galley,
            metrics.text_primary,
        );
    }
    let bar = rect_of(bar_id);
    painter.rect_filled(bar, Dp::D2.px(), metrics.ink(tone::strong()));
    painter.rect_filled(
        Rect::from_min_size(
            bar.min,
            Vec2::new(
                content_width * card.progress.clamp(0.0, 1.0),
                metrics.card_progress_height,
            ),
        ),
        Dp::D2.px(),
        metrics.accent,
    );
}
