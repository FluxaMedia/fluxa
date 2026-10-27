//! Reusable Fluxa UI components.
//!
//! Components own visual paint and typography. Screens own composition and
//! navigation only. This keeps desktop, Android, and TV on the same card and
//! control implementation while allowing each host to provide a different
//! viewport and input adapter.

use egui::{Align2, Color32, FontId, Id, Painter, Pos2, Rect, Response, RichText, Sense, Ui, Vec2};
use std::hash::{Hash, Hasher};

mod cards;
mod controls;
mod detail_modal;
mod downloads;
mod dropdown;
mod filmstrip;
mod input;
mod keycap;
mod menu;
mod mini_player;
mod overlay;
mod recent_searches;
mod seats;
mod slider;
mod streams;
mod subtitle_preview;
mod ticket;

pub(crate) use cards::*;
pub(crate) use controls::*;
pub use detail_modal::*;
pub use downloads::*;
pub(crate) use dropdown::*;
pub(crate) use filmstrip::*;
pub use input::*;
pub(crate) use keycap::*;
pub use menu::*;
pub use mini_player::*;
pub(crate) use overlay::*;
pub use recent_searches::*;
pub(crate) use seats::*;
pub(crate) use slider::*;
pub(crate) use streams::*;
pub use subtitle_preview::SubtitleLook;
pub(crate) use subtitle_preview::subtitle_sample;
pub(crate) use ticket::*;

use super::{
    ArtworkPriority, DetailEpisode, DetailTrailer, HomeAssets, HomeCard, HomeLayout, Shade,
    UiMetrics, Viewport, accent, artwork_target_size, background, badge_fill, button_height,
    cover_uv, danger_color, draw_text, full_uv, ink, paint_vertical_gradient, radius, shade, size,
    stroke, text_primary, text_width, tone, truncate_to_width,
};

pub(super) fn toast(
    context: &egui::Context,
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
    let painter = context.layer_painter(egui::LayerId::new(
        egui::Order::Tooltip,
        Id::new("fluxa-player-toast"),
    ));
    let font = FontId::proportional(size);
    let galley =
        painter.layout_no_wrap(text.to_owned(), font, metrics.ink((255.0 * opacity) as u8));
    let text_width = galley.size().x;
    let width = text_width + pad * 2.0 + level.map_or(0.0, |_| bar + 14.0);
    let rect = Rect::from_min_size(
        Pos2::new(top_center.x - width * 0.5, top_center.y),
        Vec2::new(width, size * 2.6),
    );
    painter.rect_filled(
        rect,
        metrics.dialog_radius,
        metrics.surface.gamma_multiply(245.0 / 255.0 * opacity),
    );
    painter.rect_stroke(
        rect,
        metrics.dialog_radius,
        egui::Stroke::new(stroke::hairline(), metrics.ink((20.0 * opacity) as u8)),
        egui::StrokeKind::Inside,
    );
    let mut cells = vec![crate::flex::slot(text_width).id(0)];
    if level.is_some() {
        cells.push(crate::flex::slot(bar).id(1));
    }
    let placed = crate::flex::row(cells)
        .gap(14.0)
        .align(crate::flex::Align::Stretch)
        .solve(rect.shrink2(Vec2::new(pad, 0.0)));
    let label = placed.find(0).unwrap_or(rect);
    let meter = placed.find(1);
    let text_pos = Pos2::new(label.left(), rect.center().y - galley.size().y * 0.5);
    painter.galley(text_pos, galley, metrics.text_primary);
    if let (Some(level), Some(meter)) = (level, meter) {
        let track = Rect::from_center_size(meter.center(), Vec2::new(bar, 5.0));
        painter.rect_filled(track, radius::BAR, metrics.ink((60.0 * opacity) as u8));
        let filled = Rect::from_min_size(track.min, Vec2::new(bar * level.clamp(0.0, 1.0), 5.0));
        painter.rect_filled(filled, radius::BAR, metrics.ink((255.0 * opacity) as u8));
    }
}

pub(super) fn calendar_release_row(
    ui: &mut Ui,
    card: &HomeCard,
    width: f32,
    metrics: UiMetrics,
    assets: &mut impl HomeAssets,
) -> Response {
    let height = metrics.screen_control_height.max(64.0);
    let (rect, response) = ui.allocate_exact_size(Vec2::new(width, height), Sense::click());
    ui.painter()
        .rect_filled(rect, metrics.card_radius, metrics.surface_raised);
    let gap = metrics.control_gap;
    let placed = crate::flex::row([
        crate::flex::slot(height * 0.7).id(0),
        crate::flex::fill().id(1),
        crate::flex::slot(gap).id(2),
    ])
    .gap(gap)
    .align(crate::flex::Align::Stretch)
    .solve(rect);
    let thumb = placed.find(0).unwrap_or(rect);
    let text = placed.find(1).unwrap_or(rect);
    artwork_image(
        ui.painter(),
        thumb,
        card.artwork_url.as_deref(),
        [96, 128],
        ArtworkPriority::Visible,
        Color32::WHITE,
        assets,
    );
    let text = text.shrink2(Vec2::new(0.0, gap));
    draw_text(
        ui.painter(),
        text.left_top(),
        Align2::LEFT_TOP,
        &card.title,
        crate::fonts::medium(metrics.screen_card_title_size),
        metrics.text_primary,
        Some(text.width().max(1.0)),
    );
    draw_text(
        ui.painter(),
        text.left_bottom(),
        Align2::LEFT_BOTTOM,
        &card.subtitle,
        crate::fonts::regular(metrics.screen_card_subtitle_size),
        metrics.text_secondary,
        Some(text.width().max(1.0)),
    );
    response
}

/// Shared image path for UI components: request the appropriately sized
/// cached texture, calculate cover-crop UVs from its decoded dimensions, and
/// paint it consistently. Screens should compose this instead of creating an
/// independent image/loading path.
pub(super) fn artwork_image(
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

fn near_screen(painter: &Painter, rect: Rect) -> bool {
    let screen = painter.ctx().content_rect();
    screen.expand2(screen.size()).intersects(rect)
}

#[allow(clippy::too_many_arguments)]
fn loading_fill(painter: &Painter) -> Color32 {
    let time = painter.ctx().input(|input| input.time) as f32;
    painter
        .ctx()
        .request_repaint_after(std::time::Duration::from_millis(33));
    let wave = (time * std::f32::consts::TAU / 1.4).sin() * 0.5 + 0.5;
    ink(tone::faint() + (wave * f32::from(tone::faint())) as u8)
}

pub(super) fn loading_rect(painter: &Painter, rect: Rect, radius: f32) {
    painter.rect_filled(rect, radius, loading_fill(painter));
}

pub(super) fn spinner(painter: &Painter, center: Pos2, radius: f32) {
    let time = painter.ctx().input(|input| input.time) as f32;
    painter
        .ctx()
        .request_repaint_after(std::time::Duration::from_millis(16));
    let start = time * std::f32::consts::TAU * 0.9;
    let points: Vec<Pos2> = (0..=24)
        .map(|step| {
            let angle = start + step as f32 / 24.0 * std::f32::consts::PI * 1.5;
            center + Vec2::angled(angle) * radius
        })
        .collect();
    painter.add(egui::Shape::line(
        points,
        egui::Stroke::new(radius * 0.25, text_primary()),
    ));
}

pub(super) fn loading_circle(painter: &Painter, center: Pos2, radius: f32) {
    painter.circle_filled(center, radius, loading_fill(painter));
}

pub(super) fn rounded_artwork(
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
                painter.rect_filled(rect, radius, ink(tone::faint()));
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

pub(super) fn season_tile(
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
    let image = Rect::from_min_size(rect.min, size);
    painter.rect_filled(image, radius::control(), metrics.ink(tone::faint()));
    let tint = if selected {
        metrics.text_primary
    } else {
        metrics.ink(tone::subtle())
    };
    rounded_artwork(
        painter,
        image,
        8.0,
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
            egui::Stroke::new(stroke::emphasis(), accent()),
            egui::StrokeKind::Inside,
        );
    }
    let font = FontId::proportional(metrics.text.meta);
    draw_text(
        painter,
        Pos2::new(rect.center().x, image.bottom() + 6.0),
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

pub(super) fn episode_artwork(
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
            10.0,
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
                Rect::from_center_size(rect.center(), Vec2::splat(34.0)),
                Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                metrics.text_primary,
            );
        }
    }
    if episode.watched {
        let center = rect.right_top() + Vec2::new(-16.0, 16.0);
        painter.circle_filled(center, 10.0, metrics.text_primary);
        if let Some(icon) = assets.icon("Check") {
            painter.image(
                icon,
                Rect::from_center_size(center, Vec2::splat(14.0)),
                Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                background(),
            );
        }
    } else if episode.progress > 0.0 {
        let bar = Rect::from_min_size(
            Pos2::new(rect.left(), rect.bottom() - 3.0),
            Vec2::new(rect.width() * episode.progress, 3.0),
        );
        painter.rect_filled(bar, 0.0, accent());
    }
    if selected {
        painter.rect_stroke(
            rect,
            radius::control(),
            egui::Stroke::new(stroke::emphasis(), accent()),
            egui::StrokeKind::Inside,
        );
    }
}

pub(super) fn trailer_card(
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
        10.0,
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
    painter.circle_filled(center, 22.0, shade(Shade::Veil));
    if let Some(icon) = assets.icon("PlayFilled") {
        painter.image(
            icon,
            Rect::from_center_size(center, Vec2::splat(22.0)),
            Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
            metrics.text_primary,
        );
    }
    draw_text(
        painter,
        Pos2::new(rect.left(), rect.bottom() + 10.0),
        Align2::LEFT_TOP,
        &trailer.title,
        FontId::proportional(15.0),
        text_primary(),
        Some(rect.width()),
    );
}

pub(super) fn episode_text(
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
    let font = FontId::proportional(15.0);
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
        let color = ink(tone::muted());
        let galley = wrapped_text(painter, overview, 13.0, color, width, rows);
        painter.galley(Pos2::new(origin.x, origin.y + 22.0), galley, color);
    }
}

pub(super) fn title_logo(
    ppp: f32,
    url: Option<&str>,
    bounds: Vec2,
    priority: ArtworkPriority,
    assets: &mut impl HomeAssets,
) -> Option<(egui::TextureId, Vec2)> {
    let url = url.filter(|url| !url.trim().is_empty());
    let texture = assets.texture_for(url, artwork_target_size(bounds, ppp), priority)?;
    let [width, height] = assets.texture_size(url)?;
    let aspect = width as f32 / height.max(1) as f32;
    let size = if bounds.y * aspect > bounds.x {
        Vec2::new(bounds.x, bounds.x / aspect)
    } else {
        Vec2::new(bounds.y * aspect, bounds.y)
    };
    Some((texture, size))
}

pub(super) fn avatar(
    painter: &Painter,
    assets: &mut impl HomeAssets,
    center: egui::Pos2,
    radius: f32,
    _name: &str,
    url: Option<&str>,
    metrics: UiMetrics,
) {
    avatar_sized(painter, assets, center, radius, radius, _name, url, metrics);
}

pub(super) fn avatar_sized(
    painter: &Painter,
    assets: &mut impl HomeAssets,
    center: egui::Pos2,
    radius: f32,
    texture_radius: f32,
    _name: &str,
    url: Option<&str>,
    _metrics: UiMetrics,
) {
    if dom_avatar(painter, assets, center, radius, url) {
        return;
    }
    let rect = Rect::from_center_size(center, Vec2::splat(radius * 2.0));
    let target = artwork_target_size(
        Vec2::splat(texture_radius * 2.0),
        painter.ctx().pixels_per_point(),
    );
    if rounded_artwork(
        painter,
        rect,
        radius,
        url,
        target,
        ArtworkPriority::Visible,
        Color32::WHITE,
        assets,
    ) {
        animated_avatar(painter, assets, center, radius, texture_radius, url);
        return;
    }
    if url.is_some() {
        loading_circle(painter, center, radius);
        return;
    }
    painter.circle_filled(center, radius, ink(tone::raised()));
}

pub(crate) fn animated_avatar(
    painter: &Painter,
    assets: &mut impl HomeAssets,
    center: Pos2,
    radius: f32,
    texture_radius: f32,
    url: Option<&str>,
) {
    let rect = Rect::from_center_size(center, Vec2::splat(radius * 2.0));
    let target = artwork_target_size(
        Vec2::splat(texture_radius * 2.0),
        painter.ctx().pixels_per_point(),
    );
    let Some(animated) = assets.animated_texture_for(
        url,
        fluxa_artwork::animation_target_size(target),
        ArtworkPriority::Visible,
    ) else {
        return;
    };
    let crop = cover_uv(animated.image_size, rect);
    let tile = animated.uv;
    let uv = Rect::from_min_max(
        Pos2::new(
            tile.min.x + crop.min.x * tile.width(),
            tile.min.y + crop.min.y * tile.height(),
        ),
        Pos2::new(
            tile.min.x + crop.max.x * tile.width(),
            tile.min.y + crop.max.y * tile.height(),
        ),
    );
    painter.circle_filled(center, radius, ink(tone::raised()));
    rounded_image(painter, animated.texture, rect, radius, uv, Color32::WHITE);
}

pub(super) fn icon_button(
    ui: &mut Ui,
    icon: Option<egui::TextureId>,
    size: f32,
    color: Color32,
    framed: bool,
    active: bool,
    enabled: bool,
) -> Response {
    icon_button_sized(ui, icon, Vec2::splat(size), color, framed, active, enabled)
}

pub(super) fn icon_button_sized(
    ui: &mut Ui,
    icon: Option<egui::TextureId>,
    size: Vec2,
    color: Color32,
    framed: bool,
    active: bool,
    enabled: bool,
) -> Response {
    let radius = size.y * 0.5;
    let (rect, response) = ui.allocate_exact_size(
        size,
        if enabled {
            Sense::click()
        } else {
            Sense::hover()
        },
    );
    let hovered = enabled && response.hovered();
    if framed {
        let fill = if active {
            46
        } else if hovered {
            34
        } else {
            18
        };
        ui.painter().rect_filled(rect, radius, ink(fill));
        ui.painter().rect_stroke(
            rect.shrink(0.5),
            radius,
            egui::Stroke::new(stroke::hairline(), ink(if active { 90 } else { 36 })),
            egui::StrokeKind::Inside,
        );
    } else if hovered {
        ui.painter().rect_filled(rect, radius, ink(tone::soft()));
    }
    let tint = if !enabled {
        color.gamma_multiply(0.4)
    } else if hovered {
        text_primary()
    } else {
        color
    };
    if let Some(icon) = icon {
        let placed = crate::flex::stack(
            Align2::CENTER_CENTER,
            [crate::flex::leaf(Vec2::splat(size.y * crate::style::glyph_ratio())).id(0)],
        )
        .solve(rect);
        if let Some(glyph) = placed.find(0) {
            ui.painter().image(icon, glyph, full_uv(), tint);
        }
    }
    response
}

pub(super) struct Banner<'a> {
    pub title: &'a str,
    pub trailing: &'a str,
    pub action: Option<(u64, &'a str)>,
    pub close: Option<(u64, egui::TextureId)>,
    pub progress: Option<f32>,
    pub inset_right: f32,
}

pub(super) struct BannerOutcome {
    pub action_clicked: bool,
    pub close_clicked: bool,
}

pub(super) fn banner(
    context: &egui::Context,
    id: Id,
    rect: Rect,
    opacity: f32,
    content: Banner,
    metrics: UiMetrics,
    mut targets: Option<&mut HomeLayout>,
) -> BannerOutcome {
    let mut close_slot = Rect::NOTHING;
    let mut outcome = BannerOutcome {
        action_clicked: false,
        close_clicked: false,
    };
    egui::Area::new(id)
        .constrain(false)
        .fixed_pos(rect.min)
        .order(egui::Order::Middle)
        .show(context, |ui| {
            ui.multiply_opacity(opacity);
            ui.allocate_rect(rect, Sense::hover());
            let painter = ui.painter().clone();
            painter.rect_filled(rect, 0.0, metrics.ink(tone::soft()));
            let control = rect.height() - 14.0;
            let size = metrics.nav_label_size + 2.0;
            let label = crate::fonts::medium(size);
            let inner = Rect::from_min_max(
                Pos2::new(rect.left() + 16.0, rect.top()),
                Pos2::new(rect.right() - 8.0 - content.inset_right, rect.bottom()),
            );
            let mut cells = vec![crate::flex::fill().id(0)];
            let trailing_width =
                crate::paint::text_width(&painter, content.trailing, label.clone());
            if trailing_width > 0.0 {
                cells.push(crate::flex::slot(trailing_width).id(1));
            }
            if let Some((_, text)) = content.action {
                let width = crate::paint::text_width(&painter, text, FontId::proportional(size));
                cells.push(crate::flex::slot(width + 36.0).id(2));
            }
            if content.close.is_some() {
                cells.push(crate::flex::slot(control).id(3));
            }
            let placed = crate::flex::row(cells)
                .gap(10.0)
                .align(crate::flex::Align::Stretch)
                .solve(inner);
            let title = placed.find(0).unwrap_or(inner);
            let trailing = placed
                .find(1)
                .unwrap_or_else(|| Rect::from_min_max(inner.right_top(), inner.right_bottom()));
            let action = placed.find(2).unwrap_or(Rect::NOTHING);
            let close = placed.find(3).unwrap_or(Rect::NOTHING);
            if let Some((node, _)) = content.close {
                close_slot = Rect::from_center_size(close.center(), Vec2::splat(control));
                outcome.close_clicked = tap(
                    ui,
                    id.with("close"),
                    close_slot,
                    control * 0.5,
                    node,
                    targets.as_deref_mut(),
                )
                .clicked();
            }
            if let Some((node, text)) = content.action {
                let button =
                    Rect::from_center_size(action.center(), Vec2::new(action.width(), control));
                outcome.action_clicked = ui
                    .scope_builder(egui::UiBuilder::new().max_rect(button), |ui| {
                        pill_button(
                            ui,
                            None,
                            text,
                            Some(button.width()),
                            control,
                            size,
                            true,
                            None,
                        )
                    })
                    .inner
                    .clicked();
                if let Some(layout) = targets.as_deref_mut() {
                    layout.focusable.push((node, button));
                }
            }
            let row = |painter: &egui::Painter, ink: Color32| {
                if let Some((_, icon)) = content.close {
                    painter.image(
                        icon,
                        Rect::from_center_size(close_slot.center(), Vec2::splat(18.0)),
                        full_uv(),
                        ink.gamma_multiply(0.7),
                    );
                }
                draw_text(
                    painter,
                    trailing.right_center(),
                    Align2::RIGHT_CENTER,
                    content.trailing,
                    label.clone(),
                    ink.gamma_multiply(0.8),
                    None,
                );
                draw_text(
                    painter,
                    title.left_center(),
                    Align2::LEFT_CENTER,
                    content.title,
                    crate::fonts::medium(size + 1.0),
                    ink,
                    Some(title.width().max(1.0)),
                );
            };
            row(&painter, metrics.text_primary);
            let progress = crate::motion::ease(
                context,
                id.with("fill"),
                content.progress.unwrap_or(0.0),
                crate::motion::slow(),
                crate::motion::Ease::Linear,
            );
            if content.progress.is_some()
                && let Some(filled) =
                    crate::paint::fill_to(&painter, rect, 0.0, progress, metrics.text_primary)
            {
                row(&filled, metrics.background);
            }
        });
    outcome
}

pub(super) fn tap(
    ui: &Ui,
    id: Id,
    rect: Rect,
    radius: f32,
    node: u64,
    targets: Option<&mut HomeLayout>,
) -> Response {
    let response = ui.interact(rect, id, Sense::click());
    state_layer(ui, rect, radius, &response);
    if let Some(layout) = targets {
        layout.focusable.push((node, rect));
    }
    response
}

pub(super) fn state_layer(ui: &Ui, rect: Rect, radius: f32, response: &Response) {
    let strength = crate::motion::hover(ui.ctx(), response);
    if strength > 0.0 {
        ui.painter()
            .rect_filled(rect, radius, ink(tone::raised()).gamma_multiply(strength));
    }
}

pub(super) fn back_button(
    ui: &mut Ui,
    icon: Option<egui::TextureId>,
    metrics: UiMetrics,
) -> Response {
    let size = metrics.icon_button_size;
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(size), Sense::click());
    let painter = ui.painter();
    painter.circle_filled(rect.center(), size * 0.5, metrics.surface);
    state_layer(ui, rect, size * 0.5, &response);
    painter.circle_stroke(
        rect.center(),
        size * 0.5 - 0.5,
        egui::Stroke::new(stroke::hairline(), ink(tone::raised())),
    );
    if let Some(icon) = icon {
        painter.image(
            icon,
            Rect::from_center_size(rect.center(), Vec2::splat(size * 0.5)),
            full_uv(),
            text_primary(),
        );
    }
    response
}

pub(super) struct StickyAction {
    pub node: u64,
    pub icon: Option<egui::TextureId>,
    pub active: bool,
    pub hint: String,
}

pub(super) struct StickyBar<'a> {
    pub back_node: u64,
    pub back_icon: Option<egui::TextureId>,
    pub logo: Option<(egui::TextureId, Vec2)>,
    pub title: &'a str,
    pub actions: Vec<StickyAction>,
}

pub(super) struct StickyOutput {
    pub clicked: Option<u64>,
    pub rects: Vec<(u64, Rect)>,
}

pub(super) fn sticky_bar(
    context: &egui::Context,
    id: &'static str,
    viewport: Viewport,
    margin: f32,
    shown: bool,
    bar: StickyBar,
    metrics: UiMetrics,
) -> Option<StickyOutput> {
    let fade = crate::motion::toggle(
        context,
        Id::new((id, "sticky-fade")),
        shown,
        crate::motion::fast(),
    );
    if fade <= 0.0 {
        return None;
    }
    let top = if viewport.is_compact() {
        metrics.content_header_top_mobile - 8.0
    } else {
        metrics.detail_back_top
    };
    let height = top + 48.0;
    egui::Area::new(Id::new((id, "sticky")))
        .constrain(false)
        .fixed_pos(Pos2::ZERO)
        .order(egui::Order::Foreground)
        .show(context, |ui| {
            ui.set_opacity(fade);
            let strip = Rect::from_min_size(Pos2::ZERO, Vec2::new(viewport.width, height));
            ui.interact(
                strip,
                Id::new((id, "sticky-block")),
                Sense::click_and_drag(),
            );
            ui.painter().rect_filled(strip, 0.0, metrics.background);
            ui.painter().line_segment(
                [strip.left_bottom(), strip.right_bottom()],
                egui::Stroke::new(stroke::hairline(), ink(tone::soft())),
            );
            let mut output = StickyOutput {
                clicked: None,
                rects: Vec::new(),
            };
            let button = metrics.icon_button_size;
            let line = Rect::from_min_max(
                Pos2::new(margin, top + 4.0),
                Pos2::new(viewport.width - margin, top + 4.0 + button),
            );
            let count = bar.actions.len();
            let trailing = button * count as f32 + 8.0 * count.saturating_sub(1) as f32;
            let mut cells = vec![crate::flex::slot(button).id(0), crate::flex::fill().id(1)];
            if count > 0 {
                cells.push(crate::flex::slot(trailing).id(2));
            }
            let placed = crate::flex::row(cells)
                .gap(12.0)
                .align(crate::flex::Align::Stretch)
                .solve(line);
            let back_rect = placed.find(0).unwrap_or(line);
            let title = placed.find(1).unwrap_or(line);
            let actions = placed.find(2).unwrap_or(Rect::NOTHING);
            let back = ui
                .scope_builder(egui::UiBuilder::new().max_rect(back_rect), |ui| {
                    back_button(ui, bar.back_icon, metrics)
                })
                .inner;
            output.rects.push((bar.back_node, back_rect));
            if back.clicked() {
                output.clicked = Some(bar.back_node);
            }
            let slots =
                crate::flex::row((0..count).map(|index| crate::flex::slot(button).id(index)))
                    .gap(8.0)
                    .align(crate::flex::Align::Stretch)
                    .solve(actions)
                    .children
                    .into_iter()
                    .map(|slot| slot.rect)
                    .collect::<Vec<_>>();
            for (action, rect) in bar.actions.iter().zip(slots) {
                let response = ui
                    .scope_builder(egui::UiBuilder::new().max_rect(rect), |ui| {
                        icon_button(
                            ui,
                            action.icon,
                            button,
                            text_primary(),
                            true,
                            action.active,
                            true,
                        )
                    })
                    .inner
                    .on_hover_text(&action.hint);
                output.rects.push((action.node, rect));
                if response.clicked() {
                    output.clicked = Some(action.node);
                }
            }
            let left = title.left();
            let room = title.width();
            let center = top + 24.0;
            match bar.logo {
                Some((texture, size)) => {
                    let scale = (room / size.x).min(1.0);
                    let size = size * scale;
                    ui.painter().image(
                        texture,
                        Rect::from_min_size(Pos2::new(left, center - size.y * 0.5), size),
                        full_uv(),
                        Color32::WHITE,
                    );
                }
                None => {
                    let font = crate::fonts::bold(metrics.text.subtitle);
                    draw_text(
                        ui.painter(),
                        Pos2::new(left, center),
                        Align2::LEFT_CENTER,
                        bar.title,
                        font,
                        text_primary(),
                        Some(room),
                    );
                }
            }
            output
        })
        .inner
        .into()
}

pub(super) fn labeled_action(
    ui: &mut Ui,
    icon: Option<egui::TextureId>,
    label: &str,
    width: f32,
    text_size: f32,
    active: bool,
) -> Response {
    let (rect, response) =
        ui.allocate_exact_size(Vec2::new(width, 40.0 + text_size), Sense::click());
    let painter = ui.painter();
    let center = Pos2::new(rect.center().x, rect.top() + 18.0);
    if active {
        painter.circle_filled(center, 18.0, text_primary());
    } else if response.hovered() {
        painter.circle_filled(center, 18.0, ink(tone::soft()));
    }
    if let Some(icon) = icon {
        painter.image(
            icon,
            Rect::from_center_size(center, Vec2::splat(20.0)),
            full_uv(),
            if active { background() } else { text_primary() },
        );
    }
    draw_text(
        painter,
        Pos2::new(rect.center().x, rect.bottom()),
        Align2::CENTER_BOTTOM,
        label,
        FontId::proportional(text_size),
        text_primary(),
        None,
    );
    response
}

pub(super) fn wrapped_text(
    painter: &Painter,
    text: &str,
    size: f32,
    color: Color32,
    width: f32,
    rows: usize,
) -> std::sync::Arc<egui::Galley> {
    let mut job =
        egui::text::LayoutJob::simple(text.to_owned(), FontId::proportional(size), color, width);
    job.wrap.max_rows = rows;
    painter.layout_job(job)
}

pub(super) fn texture_image(
    painter: &Painter,
    texture: egui::TextureId,
    rect: Rect,
    uv: Rect,
    tint: Color32,
) {
    painter.image(texture, rect, uv, tint);
}

pub(super) fn rounded_image(
    painter: &Painter,
    texture: egui::TextureId,
    rect: Rect,
    radius: impl Into<egui::CornerRadius>,
    uv: Rect,
    tint: Color32,
) {
    painter.add(egui::epaint::RectShape::filled(rect, radius, tint).with_texture(texture, uv));
}

pub(super) fn empty_state(
    painter: &Painter,
    rect: Rect,
    icon: Option<egui::TextureId>,
    title: &str,
    description: &str,
    metrics: UiMetrics,
) {
    let center = rect.center();
    let icon_size = 36.0;
    let title_font = crate::fonts::bold(metrics.screen_section_title_size);
    let description_font = crate::fonts::regular(metrics.screen_body_size);
    let mut y = center.y - (icon_size + metrics.section_gap + title_font.size * 2.6) * 0.5;
    if let Some(icon) = icon {
        let icon_rect = Rect::from_min_size(
            egui::Pos2::new(center.x - icon_size * 0.5, y),
            Vec2::splat(icon_size),
        );
        texture_image(painter, icon, icon_rect, full_uv(), metrics.text_muted);
        y += icon_size + metrics.section_gap;
    }
    let width = rect.width() - metrics.section_gap * 2.0;
    draw_text(
        painter,
        egui::Pos2::new(center.x, y),
        Align2::CENTER_TOP,
        title,
        title_font.clone(),
        metrics.text_primary,
        Some(width),
    );
    draw_text(
        painter,
        egui::Pos2::new(center.x, y + title_font.size * 1.5),
        Align2::CENTER_TOP,
        description,
        description_font,
        metrics.text_secondary,
        Some(width),
    );
}

pub(super) fn text_tabs(
    ui: &mut Ui,
    labels: &[String],
    selected: usize,
    height: f32,
    metrics: UiMetrics,
) -> Vec<Response> {
    let font = FontId::proportional(metrics.nav_label_size + 1.0);
    let gap = metrics.control_gap * 2.5;
    let mut responses = Vec::with_capacity(labels.len());
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = gap;
        for (index, label) in labels.iter().enumerate() {
            let active = index == selected;
            let galley = ui
                .painter()
                .layout_no_wrap(label.clone(), font.clone(), Color32::WHITE);
            let (rect, response) =
                ui.allocate_exact_size(Vec2::new(galley.size().x, height), Sense::click());
            let color = if active {
                accent()
            } else if response.hovered() {
                metrics.ink(tone::body())
            } else {
                metrics.ink(tone::muted())
            };
            ui.painter().galley(
                egui::Pos2::new(rect.left(), rect.center().y - galley.size().y * 0.5),
                galley,
                color,
            );
            if active {
                ui.painter().rect_filled(
                    Rect::from_min_max(
                        egui::Pos2::new(rect.left(), rect.bottom() - 2.0),
                        rect.right_bottom(),
                    ),
                    1.0,
                    accent(),
                );
            }
            responses.push(response);
        }
    });
    responses
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ButtonKind {
    Primary,
    Secondary,
    Selected,
    Danger,
}

pub(super) fn button(
    ui: &mut Ui,
    label: &str,
    width: f32,
    height: f32,
    kind: ButtonKind,
    metrics: UiMetrics,
) -> Response {
    button_with_text_size(
        ui,
        label,
        width,
        height,
        kind,
        metrics.nav_label_size + 2.0,
        metrics,
    )
}

pub(super) fn button_auto_width(
    ui: &mut Ui,
    label: &str,
    kind: ButtonKind,
    text_size: f32,
    metrics: UiMetrics,
) -> Response {
    let font = FontId::proportional(text_size);
    let text_width = ui.fonts_mut(|fonts| {
        fonts
            .layout_no_wrap(label.to_owned(), font, Color32::WHITE)
            .size()
            .x
    });
    button_with_text_size(
        ui,
        label,
        text_width + metrics.control_gap * 2.0,
        metrics.screen_control_height,
        kind,
        text_size,
        metrics,
    )
}

pub(crate) fn overlay_chip(painter: &Painter, rect: Rect, outline: Option<Color32>) {
    painter.rect_filled(rect, radius::chip(), badge_fill());
    if let Some(color) = outline {
        painter.rect_stroke(
            rect,
            radius::chip(),
            egui::Stroke::new(stroke::hairline(), color),
            egui::StrokeKind::Inside,
        );
    }
}

pub(super) fn focus_action(layout: &mut HomeLayout, node: u64, response: &Response) {
    layout.focusable.push((node, response.rect));
    if response.clicked() {
        layout.activated = Some(node);
    }
}

pub(super) fn placed<R>(
    context: &egui::Context,
    id: Id,
    pos: Pos2,
    add: impl FnOnce(&mut Ui) -> R,
) -> egui::InnerResponse<R> {
    egui::Area::new(id)
        .constrain(false)
        .fixed_pos(pos)
        .show(context, add)
}

pub(super) fn placed_front<R>(
    context: &egui::Context,
    id: Id,
    pos: Pos2,
    add: impl FnOnce(&mut Ui) -> R,
) -> egui::InnerResponse<R> {
    egui::Area::new(id)
        .constrain(false)
        .fixed_pos(pos)
        .order(egui::Order::Foreground)
        .show(context, add)
}

pub(super) fn button_with_text_size(
    ui: &mut Ui,
    label: &str,
    width: f32,
    height: f32,
    kind: ButtonKind,
    text_size: f32,
    metrics: UiMetrics,
) -> Response {
    button_core(
        ui,
        label,
        width,
        height,
        kind,
        text_size,
        true,
        Some(metrics),
    )
}

fn kind_colors(
    kind: ButtonKind,
    hovered: bool,
    enabled: bool,
    metrics: Option<UiMetrics>,
) -> (Color32, Color32) {
    let (fill, text) = match (kind, metrics) {
        (ButtonKind::Primary, _) | (_, None) if kind != ButtonKind::Danger => {
            (ink(if hovered { 225 } else { 255 }), background())
        }
        (ButtonKind::Secondary, _) => (ink(if hovered { 46 } else { 28 }), text_primary()),
        (ButtonKind::Danger, _) => (
            if hovered {
                danger_color().gamma_multiply(0.85)
            } else {
                danger_color()
            },
            text_primary(),
        ),
        (_, Some(metrics)) => (
            if hovered {
                metrics.accent.gamma_multiply(0.85)
            } else {
                metrics.accent
            },
            metrics.accent_foreground,
        ),
        (_, None) => (ink(tone::raised()), text_primary()),
    };
    if enabled {
        (fill, text)
    } else {
        (ink(tone::soft()), ink(tone::dim()))
    }
}

fn button_core(
    ui: &mut Ui,
    label: &str,
    width: f32,
    height: f32,
    kind: ButtonKind,
    text_size: f32,
    enabled: bool,
    metrics: Option<UiMetrics>,
) -> Response {
    let (rect, response) = ui.allocate_exact_size(
        Vec2::new(width, height),
        if enabled {
            Sense::click()
        } else {
            Sense::hover()
        },
    );
    let (fill, text) = kind_colors(kind, enabled && response.hovered(), enabled, metrics);
    let painter = ui.painter();
    crate::motion::press_scale(painter, rect, || {
        painter.rect_filled(rect, height * 0.5, fill);
        draw_text(
            painter,
            rect.center(),
            Align2::CENTER_CENTER,
            label,
            crate::fonts::bold(text_size),
            text,
            None,
        );
    });
    if enabled && response.has_focus() {
        let (expand, width) =
            metrics.map_or((3.0, 2.0), |m| (m.focus_ring_expand, m.focus_ring_width));
        painter.rect_stroke(
            rect.expand(expand),
            height * 0.5 + expand,
            egui::Stroke::new(width, accent()),
            egui::StrokeKind::Outside,
        );
    }
    response
}

pub(super) fn play_button(
    ui: &mut Ui,
    assets: &mut impl HomeAssets,
    label: &str,
    width: Option<f32>,
    height: f32,
    text_size: f32,
    progress: Option<f32>,
) -> Response {
    let unavailable = assets.playback_unavailable().map(str::to_owned);
    let width = match (&unavailable, width) {
        (Some(reason), Some(limit)) => Some(limit.max(
            crate::paint::text_width(ui.painter(), reason, FontId::proportional(text_size)) + 70.0,
        )),
        _ => width,
    };
    pill_button_styled(
        ui,
        assets.icon("PlayFilled"),
        unavailable.as_deref().unwrap_or(label),
        width,
        height,
        text_size,
        true,
        progress.filter(|_| unavailable.is_none()),
        unavailable.is_some(),
    )
}

pub(super) fn link_tile(
    ui: &mut Ui,
    size: Vec2,
    icon: Option<egui::TextureId>,
    arrow: Option<egui::TextureId>,
    title: &str,
    subtitle: &str,
    framed: bool,
    metrics: UiMetrics,
) -> Response {
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());
    let painter = ui.painter();
    if framed {
        painter.rect_filled(rect, metrics.card_radius, metrics.surface_raised);
        painter.rect_stroke(
            rect,
            metrics.card_radius,
            egui::Stroke::new(stroke::hairline(), ink(34)),
            egui::StrokeKind::Inside,
        );
    }
    if response.hovered() {
        painter.rect_filled(
            rect,
            if framed { metrics.card_radius } else { 0.0 },
            Color32::from_white_alpha(10),
        );
    }
    let badge =
        Rect::from_center_size(rect.left_center() + Vec2::new(38.0, 0.0), Vec2::splat(40.0));
    painter.rect_filled(badge, 12.0, Color32::from_white_alpha(12));
    if let Some(icon) = icon {
        painter.image(
            icon,
            Rect::from_center_size(badge.center(), Vec2::splat(22.0)),
            full_uv(),
            Color32::WHITE,
        );
    }
    let text_x = badge.right() + 14.0;
    let title_size = metrics.screen_card_subtitle_size + 3.0;
    painter.text(
        Pos2::new(text_x, rect.center().y - 2.0),
        Align2::LEFT_BOTTOM,
        title,
        crate::fonts::bold(title_size),
        text_primary(),
    );
    painter.text(
        Pos2::new(text_x, rect.center().y + 2.0),
        Align2::LEFT_TOP,
        subtitle,
        crate::fonts::regular(metrics.screen_card_subtitle_size + 1.0),
        metrics.text_secondary,
    );
    if let Some(arrow) = arrow {
        painter.image(
            arrow,
            Rect::from_center_size(
                rect.right_center() - Vec2::new(26.0, 0.0),
                Vec2::splat(18.0),
            ),
            full_uv(),
            ink(110),
        );
    }
    response
}

pub(super) fn step_button(ui: &mut Ui, size: f32, plus: bool, enabled: bool) -> Response {
    let (rect, response) = ui.allocate_exact_size(
        Vec2::splat(size),
        if enabled {
            Sense::click()
        } else {
            Sense::hover()
        },
    );
    let painter = ui.painter();
    let alpha = if enabled { 255 } else { 80 };
    let fill = if enabled && response.hovered() {
        Color32::from_white_alpha(28)
    } else {
        Color32::from_white_alpha(16)
    };
    painter.circle_filled(rect.center(), size * 0.5, fill);
    let line = egui::Stroke::new(1.6, text_primary().gamma_multiply(alpha as f32 / 255.0));
    let arm = size * 0.2;
    painter.line_segment(
        [
            rect.center() - Vec2::new(arm, 0.0),
            rect.center() + Vec2::new(arm, 0.0),
        ],
        line,
    );
    if plus {
        painter.line_segment(
            [
                rect.center() - Vec2::new(0.0, arm),
                rect.center() + Vec2::new(0.0, arm),
            ],
            line,
        );
    }
    response
}

pub(super) fn pill_button(
    ui: &mut Ui,
    icon: Option<egui::TextureId>,
    label: &str,
    width: Option<f32>,
    height: f32,
    text_size: f32,
    primary: bool,
    progress: Option<f32>,
) -> Response {
    pill_button_styled(
        ui, icon, label, width, height, text_size, primary, progress, false,
    )
}

fn pill_button_styled(
    ui: &mut Ui,
    icon: Option<egui::TextureId>,
    label: &str,
    width: Option<f32>,
    height: f32,
    text_size: f32,
    primary: bool,
    progress: Option<f32>,
    muted: bool,
) -> Response {
    let font = FontId::proportional(text_size);
    let lead = if icon.is_some() { 44.0 } else { 16.0 };
    let trail = if icon.is_some() { 26.0 } else { 16.0 };
    let chrome = lead + trail;
    let label = match width {
        Some(limit) => truncate_to_width(ui.painter(), label, &font, limit - chrome),
        None => label.to_owned(),
    };
    let label = label.as_str();
    let measure = |text: &str| {
        ui.painter()
            .layout_no_wrap(text.to_owned(), font.clone(), Color32::BLACK)
            .size()
            .x
            + chrome
    };
    let natural = measure(label);
    let width = width.unwrap_or(natural).max(natural);
    let inset = (width - natural) * 0.5;
    let (rect, response) = ui.allocate_exact_size(Vec2::new(width, height), Sense::click());
    let painter = ui.painter();
    crate::motion::press_scale(painter, rect, || {
        let (fill, ink) = if muted {
            (ink(tone::soft()), ink(tone::muted()))
        } else {
            match (primary, response.hovered()) {
                (true, false) => (text_primary(), background()),
                (true, true) => (ink(tone::body()), background()),
                (false, false) => (ink(tone::strong()), text_primary()),
                (false, true) => (ink(tone::dim()), text_primary()),
            }
        };
        if primary && !muted {
            painter.rect_filled(rect, height * 0.5, fill);
        } else {
            glass(painter, rect, height * 0.5, fill);
            painter.rect_stroke(
                rect,
                height * 0.5,
                egui::Stroke::new(stroke::hairline(), Color32::from_white_alpha(18)),
                egui::StrokeKind::Inside,
            );
        }
        let lift = if progress.is_some() { 4.0 } else { 0.0 };
        if let Some(icon) = icon {
            painter.image(
                icon,
                Rect::from_center_size(
                    rect.left_center() + Vec2::new(inset + 28.0, -lift),
                    Vec2::splat(18.0),
                ),
                full_uv(),
                ink,
            );
        }
        draw_text(
            painter,
            rect.left_center() + Vec2::new(inset + lead, -lift),
            Align2::LEFT_CENTER,
            label,
            font,
            ink,
            None,
        );
        if let Some(progress) = progress {
            let track = Rect::from_min_max(
                Pos2::new(rect.left() + lead, rect.center().y + text_size * 0.5 + 2.0),
                Pos2::new(rect.right() - 24.0, rect.center().y + text_size * 0.5 + 5.0),
            );
            painter.rect_filled(track, radius::BAR, ink.gamma_multiply(0.16));
            painter.rect_filled(
                Rect::from_min_size(
                    track.min,
                    Vec2::new(track.width() * progress.clamp(0.0, 1.0), track.height()),
                ),
                1.5,
                ink,
            );
        }
    });
    response
}

pub(super) fn progress_pill(
    ui: &mut Ui,
    rect: Rect,
    label: &str,
    text_size: f32,
    fraction: f32,
) -> Response {
    let response = ui.interact(
        rect,
        Id::new(("fluxa-progress-pill", label)),
        Sense::click(),
    );
    let painter = ui.painter();
    let radius = rect.height() * 0.5;
    let font = FontId::proportional(text_size);
    crate::motion::press_scale(painter, rect, || {
        let base = if response.hovered() { 46 } else { 28 };
        painter.rect_filled(rect, radius, shade(Shade::Scrim));
        glass(painter, rect, radius, ink(base));
        draw_text(
            painter,
            rect.center(),
            Align2::CENTER_CENTER,
            label,
            font.clone(),
            text_primary(),
            None,
        );
        let filled = rect.width() * fraction.clamp(0.0, 1.0);
        if filled > 0.5 {
            let covered = painter.with_clip_rect(Rect::from_min_size(
                rect.min,
                Vec2::new(filled, rect.height()),
            ));
            covered.rect_filled(rect, radius, text_primary());
            covered.text(
                rect.center(),
                Align2::CENTER_CENTER,
                label,
                font,
                background(),
            );
        }
    });
    response
}

pub(crate) fn brand_lockup(
    painter: &Painter,
    center: Pos2,
    height: f32,
    alpha: u8,
    assets: &impl HomeAssets,
) {
    let Some((texture, aspect)) = assets.logo("wordmark") else {
        return;
    };
    painter.image(
        texture,
        Rect::from_center_size(center, Vec2::new(height * aspect.x, height)),
        full_uv(),
        Color32::WHITE.gamma_multiply_u8(alpha),
    );
}

pub(super) fn toggle(
    context: &egui::Context,
    painter: &Painter,
    id: Id,
    track: Rect,
    on: bool,
    metrics: UiMetrics,
) {
    let seen = id.with("haptic");
    let before = context.data_mut(|data| {
        let before = data.get_temp::<bool>(seen);
        data.insert_temp(seen, on);
        before
    });
    if before.is_some_and(|before| before != on) {
        crate::buzz(if on {
            crate::Haptic::ToggleOn
        } else {
            crate::Haptic::ToggleOff
        });
    }
    let t = crate::motion::presence(context, id, on, crate::motion::normal());
    let off_track = metrics.ink(tone::strong());
    painter.rect_filled(
        track,
        track.height() * 0.5,
        lerp_color(off_track, metrics.accent, t),
    );
    painter.rect_stroke(
        track,
        track.height() * 0.5,
        egui::Stroke::new(
            stroke::hairline(),
            metrics.ink(tone::dim()).gamma_multiply(1.0 - t),
        ),
        egui::StrokeKind::Inside,
    );
    let radius = metrics
        .settings_toggle_knob_radius
        .min(track.height() * 0.5 - 2.0);
    let travel = track.width() - (radius + 2.0) * 2.0;
    let stretch = (t * (1.0 - t) * 4.0) * radius * 0.5;
    let x = track.left() + radius + 2.0 + travel * t;
    let knob = Rect::from_center_size(
        Pos2::new(x, track.center().y),
        Vec2::new(radius * 2.0 + stretch, radius * 2.0),
    );
    let knob_color = lerp_color(
        metrics.ink(tone::subtle()),
        if metrics.accent_foreground == metrics.text_primary {
            metrics.text_primary
        } else {
            metrics.background
        },
        t,
    );
    painter.rect_filled(knob, radius, knob_color);
}

fn lerp_color(a: Color32, b: Color32, t: f32) -> Color32 {
    let mix = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * t).round() as u8;
    Color32::from_rgba_unmultiplied(
        mix(a.r(), b.r()),
        mix(a.g(), b.g()),
        mix(a.b(), b.b()),
        mix(a.a(), b.a()),
    )
}

pub(super) fn focus_ring(
    painter: &Painter,
    focusable: &[(u64, Rect)],
    focused: Option<u64>,
    viewport: Viewport,
    metrics: UiMetrics,
) {
    if viewport.is_compact() || focused.is_some_and(is_text_node) {
        return;
    }
    if let Some((_, rect)) = focusable.iter().find(|(id, _)| Some(*id) == focused) {
        let rect = rect.expand(metrics.focus_ring_expand);
        let radius = if is_pill_node(focused.unwrap_or_default()) {
            rect.height() * 0.5
        } else {
            metrics.focus_ring_radius
        };
        painter.rect_stroke(
            rect,
            radius,
            egui::Stroke::new(metrics.focus_ring_width, accent()),
            egui::StrokeKind::Outside,
        );
    }
}

fn is_pill_node(node: u64) -> bool {
    use super::*;
    matches!(
        node,
        NODE_HOME
            | NODE_LIBRARY
            | NODE_DISCOVER
            | NODE_CALENDAR
            | NODE_PROFILE
            | NODE_SHORTS
            | NODE_PLAY
            | NODE_HERO_WATCHLIST
            | NODE_HOME_ADD_ADDONS
            | NODE_DETAIL_BACK
            | NODE_DETAIL_PLAY
            | NODE_DETAIL_WATCHLIST
            | NODE_DETAIL_COMPLETED
            | NODE_DETAIL_DROPPED
            | NODE_DETAIL_FAVORITE
            | NODE_DETAIL_SHUFFLE
            | NODE_SHORTS_PLAY
            | NODE_SHORTS_WATCHLIST
            | NODE_SHORTS_INFO
            | NODE_SETTINGS_BACK
            | NODE_SETTINGS_SWITCH_PROFILE
            | NODE_SETTINGS_LOG_OUT
            | NODE_FOLDER_BACK
    )
}

pub fn is_text_node(node: u64) -> bool {
    matches!(
        node,
        super::NODE_LIBRARY_SEARCH
            | super::NODE_DISCOVER_SEARCH
            | super::NODE_SIDEBAR_SEARCH
            | super::NODE_SETTINGS_SEARCH
            | super::NODE_SETTINGS_ADDON_URL
            | super::NODE_SETTINGS_PLUGIN_URL
            | super::NODE_SETTINGS_WATCH_CODE
    ) || super::poster_field(node).is_some()
        || super::server_input(node).is_some()
        || super::badge_input(node).is_some()
}

static LIQUID_GLASS: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(true);

pub fn set_liquid_glass(enabled: bool) {
    LIQUID_GLASS.store(enabled, std::sync::atomic::Ordering::Relaxed);
}

pub(crate) fn liquid_glass() -> bool {
    LIQUID_GLASS.load(std::sync::atomic::Ordering::Relaxed)
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ImageFit {
    Cover,
    Contain,
}

#[derive(Clone, Debug)]
pub struct ImageHole {
    pub url: String,
    pub radius: f32,
    pub fit: ImageFit,
    pub anchor: [f32; 2],
}

pub const DOM_TRAILER: egui::TextureId = egui::TextureId::User(u64::MAX);

#[derive(Clone, Debug)]
pub struct VideoHole;

pub(crate) fn dom_video(painter: &Painter, rect: Rect) {
    painter.add(egui::Shape::Callback(egui::PaintCallback {
        rect,
        callback: std::sync::Arc::new(VideoHole),
    }));
}

pub(crate) fn dom_image(painter: &Painter, rect: Rect, radius: f32, url: &str) {
    dom_image_fit(painter, rect, radius, url, ImageFit::Cover, [0.5, 0.5]);
}

pub(crate) fn dom_image_fit(
    painter: &Painter,
    rect: Rect,
    radius: f32,
    url: &str,
    fit: ImageFit,
    anchor: [f32; 2],
) {
    painter.add(egui::Shape::Callback(egui::PaintCallback {
        rect,
        callback: std::sync::Arc::new(ImageHole {
            url: url.to_owned(),
            radius,
            fit,
            anchor,
        }),
    }));
}

pub(crate) fn failed_dom_image(
    painter: &Painter,
    assets: &impl HomeAssets,
    rect: Rect,
    radius: f32,
    url: Option<&str>,
    target_size: [u32; 2],
    fit: ImageFit,
    anchor: [f32; 2],
) -> bool {
    let Some(url) = url.filter(|url| !url.trim().is_empty()) else {
        return false;
    };
    if !assets.dom_images() || !assets.artwork_failed(Some(url), target_size) {
        return false;
    }
    dom_image_fit(painter, rect, radius, url, fit, anchor);
    true
}

pub(crate) fn failed_logo_slot(
    ui: &mut Ui,
    assets: &impl HomeAssets,
    url: Option<&str>,
    bounds: Vec2,
    anchor: [f32; 2],
) -> bool {
    let target = artwork_target_size(bounds, ui.ctx().pixels_per_point());
    let failed = url.is_some_and(|url| {
        !url.trim().is_empty() && assets.dom_images() && assets.artwork_failed(Some(url), target)
    });
    if !failed {
        return false;
    }
    let (rect, _) = ui.allocate_exact_size(bounds, Sense::hover());
    failed_dom_image(
        ui.painter(),
        assets,
        rect,
        0.0,
        url,
        target,
        ImageFit::Contain,
        anchor,
    )
}

pub(crate) fn dom_avatar(
    painter: &Painter,
    assets: &impl HomeAssets,
    center: Pos2,
    radius: f32,
    url: Option<&str>,
) -> bool {
    let Some(url) = url.filter(|url| !url.trim().is_empty() && assets.dom_images()) else {
        return false;
    };
    let rect = Rect::from_center_size(center, Vec2::splat(radius * 2.0));
    dom_image(painter, rect, radius, url);
    true
}

#[derive(Clone, Copy, Debug)]
pub struct Glass {
    pub radius: f32,
    pub tint: Color32,
    pub refraction: f32,
    pub bevel: f32,
    pub rim: f32,
}

pub(crate) fn glass_shape(rect: Rect, radius: f32, fill: Color32) -> egui::Shape {
    if !liquid_glass() {
        return egui::Shape::rect_filled(rect, radius, fill);
    }
    let tint = if fill.a() > 200 {
        fill.gamma_multiply(0.3)
    } else {
        fill
    };
    let radius = radius.min(rect.height() * 0.5).min(rect.width() * 0.5);
    egui::Shape::Callback(egui::PaintCallback {
        rect,
        callback: std::sync::Arc::new(Glass {
            radius,
            tint,
            refraction: (rect.height() * 0.3).clamp(6.0, 22.0),
            bevel: (radius * 0.8).clamp(8.0, 26.0),
            rim: 1.0,
        }),
    })
}

pub(crate) fn glass(painter: &Painter, rect: Rect, radius: f32, fill: Color32) {
    painter.add(glass_shape(rect, radius, fill));
}

pub(super) fn heading(
    ui: &mut egui::Ui,
    eyebrow: &str,
    title: &str,
    subtitle: &str,
    metrics: UiMetrics,
) {
    ui.label(
        RichText::new(eyebrow.to_uppercase())
            .size(metrics.text.label)
            .color(metrics.text_muted),
    );
    ui.add_space(10.0);
    ui.label(
        RichText::new(title)
            .size(36.0)
            .strong()
            .color(metrics.text_primary),
    );
    ui.add_space(8.0);
    ui.label(
        RichText::new(subtitle)
            .size(metrics.text.body)
            .color(metrics.text_muted),
    );
    ui.add_space(28.0);
}

pub(super) fn field_label(ui: &mut egui::Ui, text: &str, metrics: UiMetrics) {
    ui.label(
        RichText::new(text.to_uppercase())
            .size(metrics.text.label)
            .color(metrics.text_muted),
    );
    ui.add_space(6.0);
}

pub(super) fn note(ui: &mut egui::Ui, text: &str, metrics: UiMetrics) {
    ui.add_space(16.0);
    ui.label(
        RichText::new(text)
            .size(metrics.text.label)
            .color(metrics.text_secondary),
    );
    ui.add_space(10.0);
}

pub(super) fn secondary_button(ui: &mut egui::Ui, label: &str, width: f32) -> egui::Response {
    button_core(
        ui,
        label,
        width,
        button_height(),
        ButtonKind::Secondary,
        size::BUTTON_TEXT,
        true,
        None,
    )
}

pub(super) fn primary_button(
    ui: &mut egui::Ui,
    label: &str,
    width: f32,
    enabled: bool,
) -> egui::Response {
    button_core(
        ui,
        label,
        width,
        button_height(),
        ButtonKind::Primary,
        size::BUTTON_TEXT,
        enabled,
        None,
    )
}

pub(super) fn danger_button(
    ui: &mut egui::Ui,
    label: &str,
    width: f32,
    metrics: UiMetrics,
) -> egui::Response {
    button_core(
        ui,
        label,
        width,
        button_height(),
        ButtonKind::Danger,
        size::BUTTON_TEXT,
        true,
        Some(metrics),
    )
}

pub(super) fn panel(ui: &mut egui::Ui, add: impl FnOnce(&mut egui::Ui), metrics: UiMetrics) {
    egui::Frame::NONE
        .fill(metrics.surface)
        .stroke(egui::Stroke::new(stroke::hairline(), metrics.border))
        .corner_radius(metrics.dialog_radius)
        .inner_margin(egui::Margin::same(22))
        .show(ui, add);
}

pub(super) fn modal(
    context: &egui::Context,
    screen: Rect,
    id: &str,
    add: impl FnOnce(&mut egui::Ui),
    metrics: UiMetrics,
) {
    modal_width(context, screen, id, 340.0, add, metrics);
}

pub(super) fn modal_width(
    context: &egui::Context,
    screen: Rect,
    id: &str,
    width: f32,
    add: impl FnOnce(&mut egui::Ui),
    metrics: UiMetrics,
) {
    context
        .layer_painter(egui::LayerId::new(
            egui::Order::Middle,
            Id::new((id, "scrim")),
        ))
        .rect_filled(screen, 0.0, shade(Shade::Scrim));
    egui::Area::new(Id::new(id))
        .order(egui::Order::Foreground)
        .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
        .show(context, |ui| {
            egui::Frame::NONE
                .fill(metrics.surface_raised)
                .stroke(egui::Stroke::new(stroke::hairline(), metrics.border))
                .corner_radius(metrics.dialog_radius)
                .inner_margin(egui::Margin::same(26))
                .show(ui, |ui| {
                    ui.set_width(width);
                    add(ui);
                });
        });
}

pub(super) fn text_button(ui: &mut Ui, label: &str, size: f32, alpha: u8) -> Response {
    ui.add(egui::Button::new(RichText::new(label).size(size).color(ink(alpha))).frame(false))
}

pub(super) fn surface(
    ui: &mut Ui,
    fill: Color32,
    radius: f32,
    margin: egui::Margin,
    add: impl FnOnce(&mut Ui),
    metrics: UiMetrics,
) {
    egui::Frame::NONE
        .fill(fill)
        .stroke(egui::Stroke::new(stroke::hairline(), metrics.border))
        .corner_radius(radius)
        .inner_margin(margin)
        .show(ui, add);
}

pub(super) fn screen_bar(
    ui: &mut egui::Ui,
    assets: &impl HomeAssets,
    title: &str,
    back_label: &str,
    metrics: UiMetrics,
) -> bool {
    ui.add_space(metrics.section_gap);
    let mut back = false;
    ui.horizontal(|ui| {
        ui.add_space(metrics.control_gap);
        back = back_button(ui, assets.icon("ArrowLeft"), metrics)
            .on_hover_text(back_label)
            .clicked();
        ui.add_space(crate::Space::S4.px(&metrics));
        ui.label(
            RichText::new(title)
                .size(metrics.text.title)
                .strong()
                .color(text_primary()),
        );
    });
    ui.add_space(metrics.section_gap);
    back
}

pub(super) fn close_button(
    ui: &mut egui::Ui,
    assets: &impl HomeAssets,
    label: &str,
    metrics: UiMetrics,
) -> bool {
    ui.add_space(metrics.section_gap);
    let mut clicked = false;
    ui.horizontal(|ui| {
        ui.add_space(metrics.control_gap);
        clicked = icon_button(
            ui,
            assets.icon("Close"),
            metrics.icon_button_size,
            ink(tone::body()),
            true,
            false,
            true,
        )
        .on_hover_text(label)
        .clicked();
    });
    ui.add_space(metrics.section_gap);
    clicked
}

pub(super) fn section(
    ui: &mut egui::Ui,
    flat: bool,
    metrics: UiMetrics,
    add: impl FnOnce(&mut egui::Ui),
) {
    if flat {
        ui.vertical(add);
    } else {
        panel(ui, add, metrics);
    }
}

pub(crate) fn rating_row(
    ui: &mut Ui,
    ratings: &[(String, String)],
    size: f32,
    color: Color32,
    muted: Color32,
    assets: &mut impl HomeAssets,
) {
    for (source, score) in ratings {
        let logo = crate::rating_logo(source, score).and_then(|name| assets.logo(name));
        if let Some((texture, aspect)) = logo {
            ui.add(egui::Image::new((texture, aspect * size * 1.15)))
                .on_hover_text(source.as_str());
        } else {
            ui.label(RichText::new(source).size(size).color(muted));
        }
        ui.label(RichText::new(score).size(size).strong().color(color));
        ui.add_space(size * 0.4);
    }
}
