use super::*;
use crate::dp::Dp;
use crate::flex::{self, Align, Insets};
use crate::{IconSize, Space};

pub(crate) fn toast(
    context: &crate::gfx::Context,
    viewport: Viewport,
    top_center: Pos2,
    text: &str,
    level: Option<f32>,
    opacity: f32,
) {
    let metrics = UiMetrics::for_viewport(viewport);
    let (size, pad, bar) = (
        metrics.toast_text_size,
        metrics.toast_padding,
        metrics.toast_bar_width,
    );
    let painter = context.layer_painter(crate::gfx::LayerId::new(
        crate::gfx::Order::Tooltip,
        Id::new("fluxa-player-toast"),
    ));
    let font = FontId::proportional(size);
    let galley =
        painter.layout_no_wrap(text.to_owned(), font, metrics.ink((255.0 * opacity) as u8));
    let text_width = galley.size().x;
    let gap = Space::S14.px(&metrics);
    let mut cells = vec![flex::slot(text_width).id(0)];
    if level.is_some() {
        cells.push(flex::slot(bar).id(1));
    }
    let cells = flex::row(cells)
        .gap(gap)
        .align(Align::Stretch)
        .padding(Insets::xy(pad, 0.0));
    let width = cells.outer_size().x;
    let height = size * metrics.control_toast_height_ratio;
    let rect = flex::anchored(
        Rect::from_x_y_ranges(
            (top_center.x - width)..=(top_center.x + width),
            top_center.y..=(top_center.y + height),
        ),
        Align2::CENTER_TOP,
        Vec2::new(width, height),
        0.0,
    );
    painter.rect_filled(
        rect,
        metrics.dialog_radius,
        metrics.surface.gamma_multiply(245.0 / 255.0 * opacity),
    );
    painter.rect_stroke(
        rect,
        metrics.dialog_radius,
        crate::gfx::Stroke::new(stroke::hairline(), metrics.ink((20.0 * opacity) as u8)),
        crate::gfx::StrokeKind::Inside,
    );
    let placed = cells.solve(rect);
    let label = placed.find(0).unwrap_or(rect);
    let meter = placed.find(1);
    let text_rect = flex::anchored(label, Align2::LEFT_CENTER, galley.size(), 0.0);
    painter.galley(text_rect.min, galley, metrics.text_primary);
    if let (Some(level), Some(meter)) = (level, meter) {
        let track = flex::anchored(
            meter,
            Align2::CENTER_CENTER,
            Vec2::new(bar, Dp::D4.px()),
            0.0,
        );
        painter.rect_filled(track, radius::BAR, metrics.ink((60.0 * opacity) as u8));
        let filled = Rect::from_min_size(
            track.min,
            Vec2::new(bar * level.clamp(0.0, 1.0), track.height()),
        );
        painter.rect_filled(filled, radius::BAR, metrics.ink((255.0 * opacity) as u8));
    }
}

pub(crate) fn calendar_release_row(
    ui: &mut Ui,
    card: &HomeCard,
    width: f32,
    metrics: UiMetrics,
    assets: &mut impl HomeAssets,
) -> Response {
    let height = metrics.screen_control_height.max(Dp::D64.px());
    let has_art = card.artwork_url.is_some();
    let shown = crate::kit::view()
        .row()
        .max_width(width)
        .height(height)
        .clickable()
        .class("media-calendar-release")
        .show(ui, |ui| {
            if has_art {
                let (thumb, _) = ui.allocate_exact_size(
                    Vec2::new(height * metrics.control_release_thumb_ratio, height),
                    Sense::hover(),
                );
                artwork_image(
                    ui.painter(),
                    thumb,
                    card.artwork_url.as_deref(),
                    [96, 128],
                    ArtworkPriority::Visible,
                    Color32::WHITE,
                    assets,
                );
            }
            let padding = if has_art {
                crate::flex::Space::S12
            } else {
                crate::flex::Space::S10
            };
            crate::kit::view()
                .column()
                .padding(padding)
                .gap(crate::flex::Space::S4)
                .show(ui, |ui| {
                    ui.label(
                        RichText::new(&card.title)
                            .font(crate::fonts::medium(metrics.screen_card_title_size))
                            .color(metrics.text_primary),
                    );
                    ui.label(
                        RichText::new(&card.subtitle)
                            .font(crate::fonts::regular(metrics.screen_card_subtitle_size))
                            .color(metrics.text_secondary),
                    );
                });
        });
    shown.response
}

/// Shared image path for UI components: request the appropriately sized
/// cached texture, calculate cover-crop UVs from its decoded dimensions, and
/// paint it consistently. Screens should compose this instead of creating an
/// independent image/loading path.
pub(crate) fn artwork_image(
    painter: &Painter,
    rect: Rect,
    url: Option<&str>,
    target_size: [u32; 2],
    priority: ArtworkPriority,
    tint: Color32,
    assets: &mut impl HomeAssets,
) -> bool {
    rounded_artwork(painter, rect, 0.0, url, target_size, priority, tint, assets)
}

pub(super) fn near_screen(painter: &Painter, rect: Rect) -> bool {
    let screen = painter.ctx().content_rect();
    screen.expand2(screen.size()).intersects(rect)
}

#[allow(clippy::too_many_arguments)]
pub(super) fn loading_fill(painter: &Painter) -> Color32 {
    let time = painter.ctx().input(|input| input.time()) as f32;
    painter
        .ctx()
        .request_repaint_after(crate::motion::HALF_FRAME_RATE);
    let wave = (time * std::f32::consts::TAU / 1.4).sin() * 0.5 + 0.5;
    ink(tone::faint() + (wave * f32::from(tone::faint())) as u8)
}

pub(crate) fn loading_rect(painter: &Painter, rect: Rect, radius: f32) {
    painter.rect_filled(rect, radius, loading_fill(painter));
}

pub(crate) fn spinner(painter: &Painter, center: Pos2, radius: f32) {
    let time = painter.ctx().input(|input| input.time()) as f32;
    painter.ctx().request_repaint_after(crate::motion::FRAME);
    let start = time * std::f32::consts::TAU * 0.9;
    let points: Vec<Pos2> = (0..=24)
        .map(|step| {
            let angle = start + step as f32 / 24.0 * std::f32::consts::PI * 1.5;
            center + Vec2::angled(angle) * radius
        })
        .collect();
    painter.add(crate::gfx::Shape::line(
        points,
        crate::gfx::Stroke::new(radius * 0.25, text_primary()),
    ));
}

pub(crate) fn loading_circle(painter: &Painter, center: Pos2, radius: f32) {
    painter.circle_filled(center, radius, loading_fill(painter));
}

pub(crate) fn rounded_artwork(
    painter: &Painter,
    rect: Rect,
    radius: f32,
    url: Option<&str>,
    target_size: [u32; 2],
    priority: ArtworkPriority,
    tint: Color32,
    assets: &mut impl HomeAssets,
) -> bool {
    if !painter.clip_rect().intersects(rect) {
        if near_screen(painter, rect) {
            assets.prefetch_for(url, target_size, ArtworkPriority::Prefetch);
        }
        return false;
    }
    let Some(texture) = assets.texture_for(url, target_size, priority) else {
        if url.is_some() {
            if assets.artwork_failed(url, target_size) {
                if let Some(url) = url.filter(|_| assets.dom_images()) {
                    dom_image(painter, rect, radius, url);
                    return true;
                }
                painter.rect_filled(rect, radius, crate::stylesheet::color("well"));
            } else {
                loading_rect(painter, rect, radius);
            }
        }
        return false;
    };
    let uv = assets
        .texture_size(url)
        .map(|size| cover_uv(size, rect))
        .unwrap_or_else(full_uv);
    if radius > 0.0 {
        rounded_image(painter, texture, rect, radius, uv, tint);
    } else {
        texture_image(painter, texture, rect, uv, tint);
    }
    true
}

pub(crate) fn season_tile(
    ui: &mut Ui,
    assets: &mut impl HomeAssets,
    poster: Option<&str>,
    label: &str,
    selected: bool,
    size: Vec2,
    label_height: f32,
    metrics: UiMetrics,
) -> Response {
    let (rect, response) =
        ui.allocate_exact_size(Vec2::new(size.x, size.y + label_height), Sense::click());
    let painter = ui.painter();
    let stacked = flex::column([
        flex::leaf(size).id(0),
        flex::leaf(Vec2::new(size.x, label_height)).id(1),
    ])
    .align(crate::flex::Align::Start)
    .solve(rect);
    let image = stacked.find(0).unwrap_or(rect);
    let caption = stacked.find(1).unwrap_or(rect);
    painter.rect_filled(image, radius::control(), metrics.ink(tone::faint()));
    let tint = if selected {
        metrics.text_primary
    } else {
        metrics.ink(tone::subtle())
    };
    rounded_artwork(
        painter,
        image,
        Dp::D8.px(),
        poster,
        artwork_target_size(image.size(), painter.ctx().pixels_per_point()),
        ArtworkPriority::Visible,
        tint,
        assets,
    );
    if selected {
        painter.rect_stroke(
            image,
            radius::control(),
            crate::gfx::Stroke::new(stroke::emphasis(), accent()),
            crate::gfx::StrokeKind::Inside,
        );
    }
    let font = FontId::proportional(metrics.text.meta);
    draw_text(
        painter,
        flex::anchored(
            caption,
            Align2::CENTER_TOP,
            Vec2::new(size.x, 0.0),
            Space::S6.px(&metrics),
        )
        .center_top(),
        Align2::CENTER_TOP,
        label,
        font,
        if selected {
            metrics.text_primary
        } else {
            metrics.text_muted
        },
        Some(size.x),
    );
    response
}

pub(crate) fn episode_artwork(
    painter: &Painter,
    assets: &mut impl HomeAssets,
    rect: Rect,
    episode: &DetailEpisode,
    hide_still: bool,
    hovered: bool,
    selected: bool,
    metrics: UiMetrics,
) {
    painter.rect_filled(rect, radius::control(), metrics.ink(tone::faint()));
    if hide_still
        || !rounded_artwork(
            painter,
            rect,
            Dp::D10.px(),
            episode.thumbnail.as_deref(),
            artwork_target_size(rect.size(), painter.ctx().pixels_per_point()),
            ArtworkPriority::Visible,
            Color32::WHITE,
            assets,
        )
    {
        draw_text(
            painter,
            rect.center(),
            Align2::CENTER_CENTER,
            format!("E{}", episode.number),
            FontId::proportional(metrics.text.title),
            metrics.text_muted,
            None,
        );
    }
    if hovered {
        painter.rect_filled(rect, radius::control(), shade(Shade::Faint));
        if let Some(icon) = assets.icon("PlayFilled") {
            painter.image(
                icon,
                flex::anchored(rect, Align2::CENTER_CENTER, Vec2::splat(Dp::D32.px()), 0.0),
                Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                metrics.text_primary,
            );
        }
    }
    if episode.watched {
        let badge = flex::anchored(
            rect,
            Align2::RIGHT_TOP,
            Vec2::splat(Dp::D10.px() * 2.0),
            Space::S6.px(&metrics),
        );
        painter.circle_filled(badge.center(), Dp::D10.px(), metrics.text_primary);
        if let Some(icon) = assets.icon("Check") {
            painter.image(
                icon,
                flex::anchored(badge, Align2::CENTER_CENTER, Vec2::splat(Dp::D14.px()), 0.0),
                Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                background(),
            );
        }
    } else if episode.progress > 0.0 {
        let bar = flex::anchored(
            rect,
            Align2::LEFT_BOTTOM,
            Vec2::new(rect.width() * episode.progress, Dp::D3.px()),
            0.0,
        );
        painter.rect_filled(bar, 0.0, accent());
    }
    if selected {
        painter.rect_stroke(
            rect,
            radius::control(),
            crate::gfx::Stroke::new(stroke::emphasis(), accent()),
            crate::gfx::StrokeKind::Inside,
        );
    }
}

pub(crate) fn trailer_card(
    painter: &Painter,
    assets: &mut impl HomeAssets,
    rect: Rect,
    trailer: &DetailTrailer,
    hovered: bool,
    metrics: UiMetrics,
) {
    painter.rect_filled(rect, radius::control(), metrics.ink(tone::faint()));
    rounded_artwork(
        painter,
        rect,
        Dp::D10.px(),
        trailer.thumbnail.as_deref(),
        artwork_target_size(rect.size(), painter.ctx().pixels_per_point()),
        ArtworkPriority::Visible,
        Color32::WHITE,
        assets,
    );
    painter.rect_filled(
        rect,
        radius::control(),
        shade(if hovered { Shade::Faint } else { Shade::Veil }),
    );
    let center = rect.center();
    painter.circle_filled(center, Dp::D20.px(), shade(Shade::Veil));
    if let Some(icon) = assets.icon("PlayFilled") {
        painter.image(
            icon,
            flex::anchored(
                rect,
                Align2::CENTER_CENTER,
                Vec2::splat(IconSize::Md.px(&metrics)),
                0.0,
            ),
            Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
            metrics.text_primary,
        );
    }
    draw_text(
        painter,
        rect.left_bottom() + Vec2::new(0.0, Space::S10.px(&metrics)),
        Align2::LEFT_TOP,
        &trailer.title,
        FontId::proportional(Dp::D14.px()),
        text_primary(),
        Some(rect.width()),
    );
}

pub(crate) fn episode_text(
    painter: &Painter,
    origin: Pos2,
    width: f32,
    number: i64,
    title: &str,
    overview: Option<(&str, usize)>,
) {
    let title = if title.is_empty() {
        number.to_string()
    } else {
        format!("{number}. {title}")
    };
    let metrics = crate::metrics::active();
    let font = FontId::proportional(Dp::D14.px());
    draw_text(
        painter,
        origin,
        Align2::LEFT_TOP,
        &title,
        font,
        text_primary(),
        Some(width),
    );
    if let Some((overview, rows)) = overview {
        let color = crate::stylesheet::color("glyph");
        let galley = wrapped_text(painter, overview, Dp::D12.px(), color, width, rows);
        painter.galley(origin + Vec2::new(0.0, Dp::D20.px()), galley, color);
    }
}
