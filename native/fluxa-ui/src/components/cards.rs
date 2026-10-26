use super::*;

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
            let target = artwork_target_size(rect.size(), painter.ctx().pixels_per_point());
            assets.prefetch_for(card.poster_art(), target, ArtworkPriority::Prefetch);
        }
        return;
    }
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
    context: &egui::Context,
    card: &HomeCard,
    height: f32,
    priority: ArtworkPriority,
    assets: &mut impl HomeAssets,
) {
    let (backdrop, logo) = expanded_bounds(height);
    let ppp = context.pixels_per_point();
    assets.prefetch_for(
        card.backdrop_url.as_deref().or(card.artwork_url.as_deref()),
        artwork_target_size(backdrop, ppp),
        priority,
    );
    assets.prefetch_for(
        card.logo_url.as_deref(),
        artwork_target_size(logo, ppp),
        priority,
    );
}

pub(crate) fn expanded_poster_card(
    painter: &Painter,
    rect: Rect,
    card: &HomeCard,
    progress: f32,
    trailer: Option<egui::TextureId>,
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
        let scrim = Rect::from_min_max(
            Pos2::new(rect.left(), rect.top() + rect.height() * 0.5),
            rect.max,
        );
        paint_vertical_gradient(
            painter,
            scrim,
            Color32::TRANSPARENT,
            Color32::from_black_alpha((180.0 * progress) as u8),
        );
        let pad = 14.0;
        let logo = title_logo(
            painter.ctx().pixels_per_point(),
            card.logo_url.as_deref(),
            logo_bounds,
            ArtworkPriority::Visible,
            assets,
        );
        match logo {
            Some((texture, size)) => texture_image(
                painter,
                texture,
                Rect::from_min_size(
                    Pos2::new(rect.left() + pad, rect.bottom() - pad - size.y),
                    size,
                ),
                full_uv(),
                Color32::from_white_alpha(alpha),
            ),
            None if failed_dom_image(
                painter,
                assets,
                Rect::from_min_size(
                    Pos2::new(rect.left() + pad, rect.bottom() - pad - logo_bounds.y),
                    logo_bounds,
                ),
                0.0,
                card.logo_url.as_deref(),
                artwork_target_size(logo_bounds, painter.ctx().pixels_per_point()),
                ImageFit::Contain,
                [0.0, 1.0],
            ) => {}
            None => {
                let size = metrics.screen_card_title_size + 4.0;
                paint_elided_text(
                    painter,
                    Pos2::new(rect.left() + pad, rect.bottom() - pad - size),
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
    context: &egui::Context,
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
    let mut job = egui::text::LayoutJob::single_section(
        synopsis.to_owned(),
        egui::TextFormat::simple(crate::fonts::regular(body), Color32::WHITE),
    );
    job.wrap.max_width = rect.width() - pad * 2.0;
    job.wrap.max_rows = 2;
    let galley = context.fonts_mut(|fonts| fonts.layout_job(job));
    let meta_height = if meta.is_empty() {
        0.0
    } else {
        body * 1.25 + 6.0
    };
    let top_gap = metrics.focus_ring_expand + metrics.focus_ring_width + 8.0;
    let full = top_gap + meta_height + galley.size().y;
    let origin = rect.left_bottom();
    egui::Area::new(Id::new(("fluxa-expanded-info", node)))
        .order(egui::Order::Tooltip)
        .fixed_pos(origin)
        .interactable(false)
        .constrain(false)
        .show(context, |ui| {
            let panel = Rect::from_min_size(origin, Vec2::new(rect.width(), full * progress));
            let painter = ui.painter().with_clip_rect(clip.intersect(panel));
            let left = rect.left() + pad;
            let mut y = panel.top() + top_gap;
            if !meta.is_empty() {
                paint_elided_text(
                    &painter,
                    Pos2::new(left, y),
                    meta,
                    crate::fonts::regular(body),
                    rect.width() - pad * 2.0,
                    metrics.ink(alpha),
                );
                y += meta_height;
            }
            painter.galley_with_override_text_color(
                Pos2::new(left, y),
                galley,
                metrics.ink(((150.0 * progress) as u8).max(1)),
            );
        });
}

const PREVIEW_PAD: f32 = 18.0;
const PREVIEW_BUTTON_HEIGHT: f32 = 44.0;

fn preview_body_size(metrics: UiMetrics) -> f32 {
    (metrics.screen_card_subtitle_size + 3.0).max(15.0)
}

pub(crate) fn hover_preview_reset(context: &egui::Context, order: egui::Order) {
    crate::motion::reset(context, Id::new(("fluxa-hover-preview-appear", order)));
}

pub(crate) fn hover_preview_rect(
    context: &egui::Context,
    anchor: Rect,
    screen: Rect,
    synopsis: &str,
    metrics: UiMetrics,
) -> (Rect, std::sync::Arc<egui::Galley>) {
    let width = (anchor.width() * 2.4).clamp(400.0, 500.0);
    let margin = 12.0;
    let body = preview_body_size(metrics);
    let image_height = width * 9.0 / 16.0;
    let fixed = image_height + body + 12.0 + 16.0 + PREVIEW_BUTTON_HEIGHT + PREVIEW_PAD * 2.0;
    let line = body * 1.25;
    let rows = (((screen.height() - margin * 2.0 - fixed) / line).floor() as usize).max(1);
    let mut job = egui::text::LayoutJob::single_section(
        synopsis.to_owned(),
        egui::TextFormat::simple(crate::fonts::regular(body), metrics.ink(tone::subtle())),
    );
    job.wrap.max_width = width - PREVIEW_PAD * 2.0;
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
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn hover_preview(
    context: &egui::Context,
    rect: Rect,
    card: &HomeCard,
    trailer: Option<egui::TextureId>,
    meta: &str,
    synopsis: std::sync::Arc<egui::Galley>,
    play_label: &str,
    progress: Option<f32>,
    order: egui::Order,
    metrics: UiMetrics,
    assets: &mut impl HomeAssets,
) -> (Response, Option<PreviewAction>) {
    let saved = card
        .id
        .as_deref()
        .and_then(|id| crate::poster_overlay::personal_for(context, id))
        .is_some_and(|personal| personal.saved);
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
        egui::LayerId::new(order, Id::new("fluxa-hover-preview")),
        egui::emath::TSTransform::new(rect.center().to_vec2() * (1.0 - scale), scale),
    );
    egui::Area::new(Id::new("fluxa-hover-preview"))
        .order(order)
        .fixed_pos(rect.min)
        .constrain(false)
        .show(context, |ui| {
            ui.set_opacity(appear);
            let response = ui.allocate_rect(rect, Sense::click());
            let radius = metrics.card_radius;
            let corner = egui::CornerRadius::from(radius);
            let painter = ui.painter().clone();
            painter.rect_filled(rect.expand(1.0), radius + 1.0, shade(Shade::Veil));
            painter.rect_filled(rect, radius, metrics.surface);

            let image_height = rect.width() * 9.0 / 16.0;
            let image =
                Rect::from_min_size(rect.min, Vec2::new(rect.width(), image_height + radius));
            let url = card.backdrop_url.as_deref().or(card.artwork_url.as_deref());
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
            let info =
                Rect::from_min_max(Pos2::new(rect.left(), rect.top() + image_height), rect.max);
            painter.rect_filled(
                info,
                egui::CornerRadius {
                    nw: 0,
                    ne: 0,
                    sw: corner.sw,
                    se: corner.se,
                },
                metrics.surface,
            );
            let scrim = Rect::from_min_max(
                Pos2::new(rect.left(), rect.top() + image_height * 0.4),
                Pos2::new(rect.right(), info.top()),
            );
            paint_vertical_gradient(&painter, scrim, Color32::TRANSPARENT, metrics.surface);
            painter.rect_stroke(
                rect,
                radius,
                egui::Stroke::new(stroke::hairline(), metrics.border),
                egui::StrokeKind::Inside,
            );

            let left = rect.left() + PREVIEW_PAD;
            let text_width = rect.width() - PREVIEW_PAD * 2.0;
            let preview_logo = Vec2::new(rect.width() * 0.5, image_height * 0.32);
            let logo = title_logo(
                context.pixels_per_point(),
                card.logo_url.as_deref(),
                preview_logo,
                ArtworkPriority::Visible,
                assets,
            );
            match logo {
                Some((texture, size)) => texture_image(
                    &painter,
                    texture,
                    Rect::from_min_size(Pos2::new(left, info.top() - 12.0 - size.y), size),
                    full_uv(),
                    Color32::WHITE,
                ),
                None if failed_dom_image(
                    &painter,
                    assets,
                    Rect::from_min_size(
                        Pos2::new(left, info.top() - 12.0 - preview_logo.y),
                        preview_logo,
                    ),
                    0.0,
                    card.logo_url.as_deref(),
                    artwork_target_size(preview_logo, context.pixels_per_point()),
                    ImageFit::Contain,
                    [0.0, 1.0],
                ) => {}
                None => paint_elided_text(
                    &painter,
                    Pos2::new(left, info.top() - 14.0 - title_size),
                    &card.title,
                    crate::fonts::medium(title_size),
                    text_width,
                    metrics.text_primary,
                ),
            }

            let mut y = info.top() + 2.0;
            let rating = card.overlay.rating.map(|rating| format!("{rating:.1}/10"));
            let meta_line = [rating.as_deref().unwrap_or(""), meta]
                .iter()
                .filter(|part| !part.is_empty())
                .copied()
                .collect::<Vec<_>>()
                .join("  ·  ");
            if !meta_line.is_empty() {
                paint_elided_text(
                    &painter,
                    Pos2::new(left, y),
                    &meta_line,
                    crate::fonts::regular(body_size),
                    text_width,
                    metrics.ink(tone::body()),
                );
            }
            y += body_size + 12.0;
            painter.galley(Pos2::new(left, y), synopsis, metrics.text_primary);

            let gap = 10.0;
            let row_y = rect.bottom() - PREVIEW_PAD - PREVIEW_BUTTON_HEIGHT;
            let icon_rect = Rect::from_min_size(
                Pos2::new(rect.right() - PREVIEW_PAD - PREVIEW_BUTTON_HEIGHT, row_y),
                Vec2::splat(PREVIEW_BUTTON_HEIGHT),
            );
            let play_rect = Rect::from_min_max(
                Pos2::new(left, row_y),
                Pos2::new(icon_rect.left() - gap, row_y + PREVIEW_BUTTON_HEIGHT),
            );
            let mut action = None;
            let play = ui
                .scope_builder(egui::UiBuilder::new().max_rect(play_rect), |ui| {
                    play_button(
                        ui,
                        assets,
                        play_label,
                        Some(play_rect.width()),
                        PREVIEW_BUTTON_HEIGHT,
                        button_size,
                        progress,
                    )
                })
                .inner;
            if play.clicked() {
                action = Some(PreviewAction::Play);
            }
            let watchlist = ui.interact(
                icon_rect,
                Id::new(("fluxa-hover-preview-pill", "watchlist")),
                Sense::click(),
            );
            let fill = if watchlist.hovered() { 46 } else { 28 };
            painter.rect_filled(icon_rect, PREVIEW_BUTTON_HEIGHT * 0.5, metrics.ink(fill));
            if let Some(icon) = assets.icon(if saved { "Check" } else { "Plus" }) {
                painter.image(
                    icon,
                    Rect::from_center_size(icon_rect.center(), Vec2::splat(20.0)),
                    full_uv(),
                    Color32::WHITE,
                );
            }
            if watchlist.clicked() {
                action = Some(PreviewAction::Watchlist);
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
        egui::Stroke::new(stroke::hairline(), metrics.border),
        egui::StrokeKind::Inside,
    );
    let inset = metrics.control_gap;
    let thumb = Rect::from_min_size(
        rect.min + Vec2::splat(inset),
        Vec2::new(
            (rect.height() - inset * 2.0) * 2.0 / 3.0,
            rect.height() - inset * 2.0,
        ),
    );
    poster_card_body(painter, thumb, card, metrics, assets, false, false);
    let text_x = thumb.right() + inset * 1.5;
    let text_width = rect.right() - text_x - inset;
    let title_size = metrics.screen_card_title_size + 2.0;
    let title_y = rect.center().y - title_size - metrics.control_gap * 0.5;
    paint_elided_text(
        painter,
        egui::Pos2::new(text_x, title_y),
        &card.title,
        crate::fonts::medium(title_size),
        text_width,
        metrics.text_primary,
    );
    paint_elided_text(
        painter,
        egui::Pos2::new(text_x, title_y + title_size + 4.0),
        &card.subtitle,
        crate::fonts::regular(metrics.screen_card_subtitle_size),
        text_width,
        metrics.ink(tone::subtle()),
    );
    if card.progress > 0.0 {
        let bar = Rect::from_min_size(
            egui::Pos2::new(text_x, thumb.bottom() - metrics.card_progress_height),
            Vec2::new(text_width.min(220.0), metrics.card_progress_height),
        );
        painter.rect_filled(bar, radius::BAR, metrics.ink(tone::raised()));
        painter.rect_filled(
            Rect::from_min_size(
                bar.min,
                Vec2::new(bar.width() * card.progress.clamp(0.0, 1.0), bar.height()),
            ),
            2.0,
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
            ink(tone::text()),
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
            ink(tone::text()),
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
            egui::Pos2::new(
                tile.min.x + crop.min.x * tile.width(),
                tile.min.y + crop.min.y * tile.height(),
            ),
            egui::Pos2::new(
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
    let label_y = rect.bottom() + metrics.control_gap;
    paint_elided_text(
        painter,
        egui::Pos2::new(rect.left(), label_y),
        &card.title,
        title_font,
        rect.width(),
        metrics.text_primary,
    );
    paint_elided_text(
        painter,
        egui::Pos2::new(rect.left(), label_y + metrics.screen_card_title_size + 2.0),
        &card.subtitle,
        subtitle_font,
        rect.width(),
        metrics.ink(tone::subtle()),
    );
}

fn textless_logo(painter: &Painter, rect: Rect, url: &str, assets: &mut impl HomeAssets) {
    let scrim = Rect::from_min_max(
        egui::Pos2::new(rect.left(), rect.bottom() - rect.height() * 0.5),
        rect.max,
    );
    crate::paint_vertical_gradient(
        &painter.with_clip_rect(rect.intersect(painter.clip_rect())),
        scrim,
        Color32::TRANSPARENT,
        Color32::from_rgba_unmultiplied(0, 0, 0, 190),
    );
    let bounds = Vec2::new(rect.width() * 0.76, rect.height() * 0.2);
    let Some((texture, logo_size)) = title_logo(
        painter.ctx().pixels_per_point(),
        Some(url),
        bounds,
        ArtworkPriority::Visible,
        assets,
    ) else {
        return;
    };
    let center = egui::Pos2::new(rect.center().x, rect.bottom() - rect.height() * 0.27);
    texture_image(
        painter,
        texture,
        Rect::from_center_size(center, logo_size),
        Rect::from_min_max(egui::Pos2::ZERO, egui::Pos2::new(1.0, 1.0)),
        Color32::WHITE,
    );
}

fn landscape_logo(painter: &Painter, rect: Rect, card: &HomeCard, assets: &mut impl HomeAssets) {
    let bounds = Vec2::new(rect.width() * 0.5, rect.height() * 0.3);
    let Some((texture, logo_size)) = title_logo(
        painter.ctx().pixels_per_point(),
        card.logo_url.as_deref(),
        bounds,
        ArtworkPriority::Visible,
        assets,
    ) else {
        failed_dom_image(
            painter,
            assets,
            Rect::from_center_size(rect.center(), bounds),
            0.0,
            card.logo_url.as_deref(),
            artwork_target_size(bounds, painter.ctx().pixels_per_point()),
            ImageFit::Contain,
            [0.5, 0.5],
        );
        return;
    };
    let center = egui::Pos2::new(rect.center().x, rect.top() + rect.height() * 0.5);
    texture_image(
        painter,
        texture,
        Rect::from_center_size(center, logo_size),
        Rect::from_min_max(egui::Pos2::ZERO, egui::Pos2::new(1.0, 1.0)),
        Color32::WHITE,
    );
}

/// Lay out poster labels once with egui's native single-line ellipsis. The
/// previous path measured each string repeatedly to find a fitting prefix,
/// which multiplied text shaping work across every visible result per frame.
pub(super) fn paint_elided_text(
    painter: &Painter,
    position: egui::Pos2,
    text: &str,
    font: FontId,
    max_width: f32,
    color: Color32,
) {
    // egui caches font shaping internally, but building a LayoutJob and
    // requesting a new galley for every visible card on every frame still
    // costs enough to hitch while a page of posters enters the viewport.
    // Keep the already-laid-out galley in egui's per-context temp store.
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    text.hash(&mut hasher);
    font.hash(&mut hasher);
    max_width.to_bits().hash(&mut hasher);
    color.to_array().hash(&mut hasher);
    painter.ctx().pixels_per_point().to_bits().hash(&mut hasher);
    let galley_id = Id::new("fluxa-poster-label-galley").with(hasher.finish());
    let galley = painter
        .ctx()
        .data(|data| data.get_temp::<std::sync::Arc<egui::Galley>>(galley_id));
    let galley = galley.unwrap_or_else(|| {
        let mut job = egui::text::LayoutJob::simple_singleline(text.to_owned(), font, color);
        job.wrap = egui::text::TextWrapping::truncate_at_width(max_width);
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
    painter.ctx().request_repaint();
}

pub(crate) fn continue_card(
    painter: &Painter,
    rect: Rect,
    card: &HomeCard,
    metrics: UiMetrics,
    assets: &mut impl HomeAssets,
) {
    crate::motion::press_scale(painter, rect, || {
        continue_card_body(painter, rect, card, metrics, assets)
    });
}

fn continue_card_body(
    painter: &Painter,
    rect: Rect,
    card: &HomeCard,
    metrics: UiMetrics,
    assets: &mut impl HomeAssets,
) {
    painter.rect_filled(rect, metrics.card_radius, metrics.surface);

    artwork_image(
        painter,
        rect,
        card.artwork_url.as_deref(),
        artwork_target_size(
            Vec2::new(rect.width(), rect.height()),
            painter.ctx().pixels_per_point(),
        ),
        ArtworkPriority::Visible,
        ink(tone::text()),
        assets,
    );

    let scrim_top = rect.top() + rect.height() * 0.2;
    let scrim_mid = rect.top() + rect.height() * 0.5;
    paint_vertical_gradient(
        painter,
        Rect::from_min_max(
            egui::Pos2::new(rect.left(), scrim_top),
            egui::Pos2::new(rect.right(), scrim_mid),
        ),
        Color32::TRANSPARENT,
        shade(Shade::Scrim),
    );
    paint_vertical_gradient(
        painter,
        Rect::from_min_max(egui::Pos2::new(rect.left(), scrim_mid), rect.right_bottom()),
        shade(Shade::Scrim),
        shade(Shade::Opaque),
    );
    let title_font = crate::fonts::medium(metrics.screen_card_title_size);
    let subtitle_font = crate::fonts::regular(metrics.screen_card_subtitle_size);
    let content_width = (rect.width() - metrics.card_content_padding * 2.0).max(1.0);
    let bar_top = rect.bottom() - metrics.card_content_padding - metrics.card_progress_height;
    let has_subtitle = !card.subtitle.is_empty();
    let subtitle_height = if has_subtitle {
        metrics.screen_card_subtitle_size * 1.25
    } else {
        0.0
    };
    let title_height = metrics.screen_card_title_size * 1.25;
    let subtitle_top = bar_top - metrics.control_gap * 0.5 - subtitle_height;
    let title_top = if has_subtitle {
        subtitle_top - metrics.control_gap * 0.35 - title_height
    } else {
        subtitle_top - title_height
    };
    draw_text(
        painter,
        egui::Pos2::new(rect.left() + metrics.card_content_padding, title_top),
        Align2::LEFT_TOP,
        &card.title,
        title_font,
        metrics.text_primary,
        Some(content_width),
    );
    if has_subtitle {
        draw_text(
            painter,
            egui::Pos2::new(rect.left() + metrics.card_content_padding, subtitle_top),
            Align2::LEFT_TOP,
            &card.subtitle,
            subtitle_font.clone(),
            metrics.ink(tone::body()),
            Some(content_width),
        );
    }
    let badge_pad = metrics.screen_card_subtitle_size * 0.45;
    let badge_height = metrics.screen_card_subtitle_size + badge_pad * 1.4;
    let inset = (rect.height() * 0.06).max(4.0);
    for (text, right) in [(&card.episodes, false), (&card.detail, true)] {
        if text.is_empty() {
            continue;
        }
        let galley = painter.layout_no_wrap(text.clone(), subtitle_font.clone(), Color32::WHITE);
        let size = Vec2::new(galley.size().x + badge_pad * 2.0, badge_height);
        let origin = if right {
            egui::Pos2::new(rect.right() - inset - size.x, rect.top() + inset)
        } else {
            egui::Pos2::new(rect.left() + inset, rect.top() + inset)
        };
        let badge = Rect::from_min_size(origin, size);
        overlay_chip(painter, badge, None);
        painter.galley(
            badge.center() - galley.size() * 0.5,
            galley,
            metrics.text_primary,
        );
    }
    let bar_origin = egui::Pos2::new(rect.left() + metrics.card_content_padding, bar_top);
    painter.rect_filled(
        Rect::from_min_size(
            bar_origin,
            Vec2::new(content_width, metrics.card_progress_height),
        ),
        2.0,
        metrics.ink(tone::strong()),
    );
    painter.rect_filled(
        Rect::from_min_size(
            bar_origin,
            Vec2::new(
                content_width * card.progress.clamp(0.0, 1.0),
                metrics.card_progress_height,
            ),
        ),
        2.0,
        metrics.accent,
    );
}
