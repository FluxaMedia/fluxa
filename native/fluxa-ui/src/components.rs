//! Reusable Fluxa UI components.
//!
//! Components own visual paint and typography. Screens own composition and
//! navigation only. This keeps desktop, Android, and TV on the same card and
//! control implementation while allowing each host to provide a different
//! viewport and input adapter.

use egui::{Align2, Color32, FontId, Id, Painter, Pos2, Rect, Response, RichText, Sense, Ui, Vec2};
use std::hash::{Hash, Hasher};

use super::{
    ArtworkPriority, DetailEpisode, HomeAssets, HomeCard, UiMetrics, Viewport, artwork_target_size,
    cover_uv, full_uv, paint_vertical_gradient, truncate_to_width,
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
    let galley = painter.layout_no_wrap(
        text.to_owned(),
        font,
        Color32::from_white_alpha((255.0 * opacity) as u8),
    );
    let gap = 14.0;
    let width = galley.size().x + pad * 2.0 + level.map_or(0.0, |_| bar + gap);
    let rect = Rect::from_min_size(
        Pos2::new(top_center.x - width * 0.5, top_center.y),
        Vec2::new(width, size * 2.6),
    );
    painter.rect_filled(
        rect,
        16.0,
        Color32::from_rgba_unmultiplied(0x13, 0x13, 0x13, (245.0 * opacity) as u8),
    );
    painter.rect_stroke(
        rect,
        16.0,
        egui::Stroke::new(1.0, Color32::from_white_alpha((20.0 * opacity) as u8)),
        egui::StrokeKind::Inside,
    );
    let text_pos = Pos2::new(rect.left() + pad, rect.center().y - galley.size().y * 0.5);
    let text_width = galley.size().x;
    painter.galley(text_pos, galley, Color32::WHITE);
    if let Some(level) = level {
        let track = Rect::from_min_size(
            Pos2::new(text_pos.x + text_width + gap, rect.center().y - 2.5),
            Vec2::new(bar, 5.0),
        );
        painter.rect_filled(
            track,
            2.5,
            Color32::from_white_alpha((60.0 * opacity) as u8),
        );
        let filled = Rect::from_min_size(track.min, Vec2::new(bar * level.clamp(0.0, 1.0), 5.0));
        painter.rect_filled(
            filled,
            2.5,
            Color32::from_white_alpha((255.0 * opacity) as u8),
        );
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
    let thumb = Rect::from_min_size(rect.min, Vec2::new(height * 0.7, height));
    artwork_image(
        ui.painter(),
        thumb,
        card.artwork_url.as_deref(),
        [96, 128],
        ArtworkPriority::Visible,
        Color32::WHITE,
        assets,
    );
    let text_width = (rect.width() - thumb.width() - metrics.control_gap * 3.0).max(1.0);
    let text_x = thumb.right() + metrics.control_gap;
    let title_font = FontId::proportional(metrics.screen_card_title_size);
    let subtitle_font = crate::fonts::regular(metrics.screen_card_subtitle_size);
    ui.painter().text(
        egui::Pos2::new(text_x, rect.top() + metrics.control_gap),
        Align2::LEFT_TOP,
        truncate_to_width(ui.painter(), &card.title, &title_font, text_width),
        title_font,
        Color32::WHITE,
    );
    ui.painter().text(
        egui::Pos2::new(text_x, rect.bottom() - metrics.control_gap),
        Align2::LEFT_BOTTOM,
        truncate_to_width(ui.painter(), &card.subtitle, &subtitle_font, text_width),
        subtitle_font,
        metrics.text_secondary,
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

#[allow(clippy::too_many_arguments)]
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
    let Some(texture) = assets.texture_for(url, target_size, priority) else {
        return false;
    };
    let uv = assets
        .texture_size(url)
        .map(|size| cover_uv(size, rect))
        .unwrap_or_else(full_uv);
    if radius > 0.0 {
        painter.add(egui::epaint::RectShape::filled(rect, radius, tint).with_texture(texture, uv));
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
    painter.rect_filled(image, 8.0, Color32::from_white_alpha(14));
    let tint = if selected {
        Color32::WHITE
    } else {
        Color32::from_white_alpha(150)
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
            8.0,
            egui::Stroke::new(2.0, Color32::WHITE),
            egui::StrokeKind::Inside,
        );
    }
    let font = FontId::proportional(metrics.text.meta);
    painter.text(
        Pos2::new(rect.center().x, image.bottom() + 6.0),
        Align2::CENTER_TOP,
        truncate_to_width(painter, label, &font, size.x),
        font,
        if selected {
            metrics.text_primary
        } else {
            metrics.text_muted
        },
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
    painter.rect_filled(rect, 10.0, Color32::from_white_alpha(14));
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
        painter.text(
            rect.center(),
            Align2::CENTER_CENTER,
            format!("E{}", episode.number),
            FontId::proportional(22.0),
            metrics.text_muted,
        );
    }
    if hovered {
        painter.rect_filled(rect, 10.0, Color32::from_black_alpha(90));
        if let Some(icon) = assets.icon("PlayFilled") {
            painter.image(
                icon,
                Rect::from_center_size(rect.center(), Vec2::splat(34.0)),
                Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                Color32::WHITE,
            );
        }
    }
    if episode.watched {
        let center = rect.right_top() + Vec2::new(-16.0, 16.0);
        painter.circle_filled(center, 10.0, Color32::WHITE);
        painter.text(
            center,
            Align2::CENTER_CENTER,
            "✓",
            FontId::proportional(13.0),
            Color32::BLACK,
        );
    } else if episode.progress > 0.0 {
        let bar = Rect::from_min_size(
            Pos2::new(rect.left(), rect.bottom() - 3.0),
            Vec2::new(rect.width() * episode.progress, 3.0),
        );
        painter.rect_filled(bar, 0.0, Color32::WHITE);
    }
    if selected {
        painter.rect_stroke(
            rect,
            10.0,
            egui::Stroke::new(2.0, Color32::WHITE),
            egui::StrokeKind::Inside,
        );
    }
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
    painter.text(
        origin,
        Align2::LEFT_TOP,
        truncate_to_width(painter, &title, &font, width),
        font,
        Color32::WHITE,
    );
    if let Some((overview, rows)) = overview {
        let color = Color32::from_white_alpha(140);
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
    name: &str,
    url: Option<&str>,
    metrics: UiMetrics,
) {
    let rect = Rect::from_center_size(center, Vec2::splat(radius * 2.0));
    let target = artwork_target_size(rect.size(), painter.ctx().pixels_per_point());
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
        return;
    }
    let initials = name
        .split_whitespace()
        .take(2)
        .filter_map(|word| word.chars().next())
        .flat_map(char::to_uppercase)
        .collect::<String>();
    painter.circle_filled(center, radius, Color32::from_rgb(44, 44, 44));
    painter.text(
        center,
        Align2::CENTER_CENTER,
        initials,
        FontId::proportional(radius * 0.72),
        metrics.text_primary,
    );
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
        ui.painter()
            .rect_filled(rect, radius, Color32::from_white_alpha(fill));
        ui.painter().rect_stroke(
            rect.shrink(0.5),
            radius,
            egui::Stroke::new(1.0, Color32::from_white_alpha(if active { 90 } else { 36 })),
            egui::StrokeKind::Inside,
        );
    } else if hovered {
        ui.painter()
            .rect_filled(rect, radius, Color32::from_white_alpha(18));
    }
    let tint = if !enabled {
        color.gamma_multiply(0.4)
    } else if hovered {
        Color32::WHITE
    } else {
        color
    };
    if let Some(icon) = icon {
        ui.painter().image(
            icon,
            Rect::from_center_size(rect.center(), Vec2::splat(size.y * 0.44)),
            full_uv(),
            tint,
        );
    }
    response
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
        painter.circle_filled(center, 18.0, Color32::WHITE);
    } else if response.hovered() {
        painter.circle_filled(center, 18.0, Color32::from_white_alpha(24));
    }
    if let Some(icon) = icon {
        painter.image(
            icon,
            Rect::from_center_size(center, Vec2::splat(20.0)),
            full_uv(),
            if active {
                Color32::BLACK
            } else {
                Color32::WHITE
            },
        );
    }
    painter.text(
        Pos2::new(rect.center().x, rect.bottom()),
        Align2::CENTER_BOTTOM,
        label,
        FontId::proportional(text_size),
        Color32::WHITE,
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
    let title_font = FontId::proportional(metrics.screen_section_title_size);
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
    painter.text(
        egui::Pos2::new(center.x, y),
        Align2::CENTER_TOP,
        truncate_to_width(painter, title, &title_font, width),
        title_font.clone(),
        metrics.text_primary,
    );
    painter.text(
        egui::Pos2::new(center.x, y + title_font.size * 1.5),
        Align2::CENTER_TOP,
        truncate_to_width(painter, description, &description_font, width),
        description_font,
        metrics.text_secondary,
    );
}

pub fn set_input_caret(context: &egui::Context, rect: Option<Rect>) {
    context.data_mut(|data| data.insert_temp(Id::new("fluxa-input-caret"), rect));
}

fn paint_caret(ui: &Ui, field: Rect, text_rect: Rect, text: &str, font: FontId) {
    let focused = ui
        .ctx()
        .data(|data| data.get_temp::<Option<Rect>>(Id::new("fluxa-input-caret")))
        .flatten()
        .is_some_and(|rect| field.contains(rect.center()));
    if !focused {
        return;
    }
    let millis = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_millis());
    if millis / 530 % 2 == 1 {
        return;
    }
    let width = ui
        .painter()
        .layout_no_wrap(text.to_owned(), font.clone(), Color32::WHITE)
        .size()
        .x;
    let x = (text_rect.left() + width + 1.0).min(text_rect.right());
    let half = font.size * 0.62;
    ui.painter().line_segment(
        [
            egui::Pos2::new(x, text_rect.center().y - half),
            egui::Pos2::new(x, text_rect.center().y + half),
        ],
        egui::Stroke::new(1.6, Color32::WHITE),
    );
}

pub(super) fn search_field(
    ui: &mut Ui,
    query: &mut String,
    hint: &str,
    width: f32,
    height: f32,
    metrics: UiMetrics,
) -> Response {
    let radius = height * 0.5;
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, height), Sense::hover());
    ui.painter()
        .rect_filled(rect, radius, Color32::from_white_alpha(12));
    let icon_color = Color32::from_white_alpha(140);
    let icon = egui::Pos2::new(rect.left() + radius, rect.center().y);
    ui.painter().circle_stroke(
        icon - Vec2::splat(1.5),
        6.0,
        egui::Stroke::new(1.6, icon_color),
    );
    ui.painter().line_segment(
        [icon + Vec2::splat(3.0), icon + Vec2::splat(7.0)],
        egui::Stroke::new(1.6, icon_color),
    );
    let text_rect = Rect::from_min_max(
        egui::Pos2::new(rect.left() + radius + 16.0, rect.top()),
        egui::Pos2::new(rect.right() - radius * 0.6, rect.bottom()),
    );
    let response = ui.put(
        text_rect,
        egui::TextEdit::singleline(query)
            .frame(egui::Frame::NONE)
            .font(crate::fonts::regular(metrics.screen_body_size + 1.0))
            .vertical_align(egui::Align::Center)
            .hint_text(RichText::new(hint).color(metrics.text_muted))
            .text_color(Color32::WHITE)
            .margin(Vec2::ZERO),
    );
    paint_caret(
        ui,
        rect,
        text_rect,
        query,
        crate::fonts::regular(metrics.screen_body_size + 1.0),
    );
    let stroke = if response.has_focus() {
        egui::Stroke::new(1.0, Color32::from_white_alpha(120))
    } else {
        egui::Stroke::new(1.0, metrics.border)
    };
    ui.painter()
        .rect_stroke(rect, radius, stroke, egui::StrokeKind::Inside);
    response
}

pub(super) fn text_field(
    ui: &mut Ui,
    value: &mut String,
    hint: &str,
    size: Vec2,
    font_size: f32,
    metrics: UiMetrics,
) -> Response {
    text_edit(ui, value, hint, size, font_size, false, metrics)
}

pub(super) fn text_input(
    ui: &mut Ui,
    value: &mut String,
    hint: &str,
    width: f32,
    password: bool,
    metrics: UiMetrics,
) -> Response {
    text_edit(
        ui,
        value,
        hint,
        Vec2::new(width, 44.0),
        14.0,
        password,
        metrics,
    )
}

fn text_edit(
    ui: &mut Ui,
    value: &mut String,
    hint: &str,
    size: Vec2,
    font_size: f32,
    password: bool,
    metrics: UiMetrics,
) -> Response {
    let radius = size.y * 0.5;
    let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
    ui.painter()
        .rect_filled(rect, radius, Color32::from_white_alpha(10));
    let text_rect = rect.shrink2(Vec2::new(radius.max(12.0), 0.0));
    let response = ui.put(
        text_rect,
        egui::TextEdit::singleline(value)
            .frame(egui::Frame::NONE)
            .password(password)
            .font(FontId::proportional(font_size))
            .vertical_align(egui::Align::Center)
            .hint_text(RichText::new(hint).color(metrics.text_muted))
            .text_color(Color32::WHITE)
            .margin(Vec2::ZERO),
    );
    let shown = if password {
        "\u{2022}".repeat(value.chars().count())
    } else {
        value.clone()
    };
    paint_caret(ui, rect, text_rect, &shown, FontId::proportional(font_size));
    let stroke = if response.has_focus() {
        egui::Stroke::new(1.0, Color32::from_white_alpha(120))
    } else {
        egui::Stroke::new(1.0, metrics.border)
    };
    ui.painter()
        .rect_stroke(rect, radius, stroke, egui::StrokeKind::Inside);
    response
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
                Color32::WHITE
            } else if response.hovered() {
                Color32::from_white_alpha(200)
            } else {
                Color32::from_white_alpha(130)
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
                    Color32::WHITE,
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
    Accent,
    Selected,
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

pub(super) fn button_with_text_size(
    ui: &mut Ui,
    label: &str,
    width: f32,
    height: f32,
    kind: ButtonKind,
    text_size: f32,
    metrics: UiMetrics,
) -> Response {
    let fill = match kind {
        ButtonKind::Primary => Color32::WHITE,
        ButtonKind::Secondary => Color32::from_white_alpha(28),
        ButtonKind::Accent | ButtonKind::Selected => metrics.accent,
    };
    let text_color = match kind {
        ButtonKind::Primary => Color32::from_rgb(10, 10, 10),
        ButtonKind::Secondary => Color32::WHITE,
        ButtonKind::Accent | ButtonKind::Selected => metrics.accent_foreground,
    };
    ui.add_sized(
        [width, height],
        egui::Button::new(
            RichText::new(label)
                .size(text_size)
                .strong()
                .color(text_color),
        )
        .fill(fill)
        .corner_radius(height * 0.5),
    )
}

pub(super) fn poster_card(
    painter: &Painter,
    rect: Rect,
    card: &HomeCard,
    metrics: UiMetrics,
    assets: &mut impl HomeAssets,
    motion_active: bool,
) {
    crate::motion::press_scale(painter, rect, || {
        poster_card_body(painter, rect, card, metrics, assets, motion_active, true)
    });
}

const PREVIEW_PAD: f32 = 18.0;
const PREVIEW_BUTTON_HEIGHT: f32 = 44.0;

fn preview_body_size(metrics: UiMetrics) -> f32 {
    (metrics.screen_card_subtitle_size + 3.0).max(15.0)
}

pub(super) fn hover_preview_reset(context: &egui::Context) {
    context.animate_bool_with_time(Id::new("fluxa-hover-preview-appear"), false, 0.0);
}

pub(super) fn hover_preview_rect(
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
        egui::TextFormat::simple(crate::fonts::regular(body), Color32::from_white_alpha(150)),
    );
    job.wrap.max_width = width - PREVIEW_PAD * 2.0;
    job.wrap.max_rows = rows;
    let galley = context.fonts_mut(|fonts| fonts.layout_job(job));
    let height = fixed + galley.size().y;
    let x = (anchor.center().x - width * 0.5).clamp(
        screen.left() + margin,
        (screen.right() - margin - width).max(screen.left()),
    );
    let y = (anchor.center().y - height * 0.5).clamp(
        screen.top() + margin,
        (screen.bottom() - margin - height).max(screen.top()),
    );
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
pub(super) fn hover_preview(
    context: &egui::Context,
    rect: Rect,
    card: &HomeCard,
    meta: &str,
    synopsis: std::sync::Arc<egui::Galley>,
    play_label: &str,
    progress: Option<f32>,
    metrics: UiMetrics,
    assets: &mut impl HomeAssets,
) -> (Response, Option<PreviewAction>) {
    let saved = card
        .id
        .as_deref()
        .and_then(|id| super::poster_overlay::personal_for(context, id))
        .is_some_and(|personal| personal.saved);
    let title_size = (metrics.screen_card_title_size + 7.0).max(22.0);
    let body_size = preview_body_size(metrics);
    let button_size = body_size + 1.0;
    let appear = context.animate_bool_with_time_and_easing(
        Id::new("fluxa-hover-preview-appear"),
        true,
        0.18,
        egui::emath::easing::cubic_out,
    );
    let scale = 0.92 + 0.08 * appear;
    context.set_transform_layer(
        egui::LayerId::new(egui::Order::Tooltip, Id::new("fluxa-hover-preview")),
        egui::emath::TSTransform::new(rect.center().to_vec2() * (1.0 - scale), scale),
    );
    egui::Area::new(Id::new("fluxa-hover-preview"))
        .order(egui::Order::Tooltip)
        .fixed_pos(rect.min)
        .constrain(false)
        .show(context, |ui| {
            ui.set_opacity(appear);
            let response = ui.allocate_rect(rect, Sense::click());
            let radius = metrics.card_radius;
            let corner = egui::CornerRadius::from(radius);
            let painter = ui.painter().clone();
            painter.rect_filled(
                rect.expand(1.0),
                radius + 1.0,
                Color32::from_black_alpha(120),
            );
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
                egui::Stroke::new(1.0, metrics.border),
                egui::StrokeKind::Inside,
            );

            let left = rect.left() + PREVIEW_PAD;
            let text_width = rect.width() - PREVIEW_PAD * 2.0;
            let logo = title_logo(
                context.pixels_per_point(),
                card.logo_url.as_deref(),
                Vec2::new(rect.width() * 0.5, image_height * 0.32),
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
                None => paint_elided_text(
                    &painter,
                    Pos2::new(left, info.top() - 14.0 - title_size),
                    &card.title,
                    FontId::proportional(title_size),
                    text_width,
                    Color32::WHITE,
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
                    Color32::from_white_alpha(200),
                );
            }
            y += body_size + 12.0;
            painter.galley(Pos2::new(left, y), synopsis, Color32::WHITE);

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
            painter.rect_filled(
                icon_rect,
                PREVIEW_BUTTON_HEIGHT * 0.5,
                Color32::from_white_alpha(fill),
            );
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

pub(super) fn library_row(
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
        egui::Stroke::new(1.0, metrics.border),
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
        FontId::proportional(title_size),
        text_width,
        Color32::WHITE,
    );
    paint_elided_text(
        painter,
        egui::Pos2::new(text_x, title_y + title_size + 4.0),
        &card.subtitle,
        crate::fonts::regular(metrics.screen_card_subtitle_size),
        text_width,
        Color32::from_white_alpha(185),
    );
    if card.progress > 0.0 {
        let bar = Rect::from_min_size(
            egui::Pos2::new(text_x, thumb.bottom() - metrics.card_progress_height),
            Vec2::new(text_width.min(220.0), metrics.card_progress_height),
        );
        painter.rect_filled(bar, 2.0, Color32::from_white_alpha(40));
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
    let has_static_artwork = artwork_image(
        painter,
        rect,
        artwork,
        artwork_target_size(
            Vec2::new(rect.width(), rect.height()),
            painter.ctx().pixels_per_point(),
        ),
        ArtworkPriority::Visible,
        Color32::from_white_alpha(235),
        assets,
    );
    let has_static_artwork = match super::poster_overlay::custom_url(painter.ctx(), card) {
        Some(url) => {
            artwork_image(
                painter,
                rect,
                Some(&url),
                artwork_target_size(
                    Vec2::new(rect.width(), rect.height()),
                    painter.ctx().pixels_per_point(),
                ),
                ArtworkPriority::Visible,
                Color32::from_white_alpha(235),
                assets,
            ) || has_static_artwork
        }
        None => has_static_artwork,
    };
    let motion_texture = if motion_active
        && (has_static_artwork || card.artwork_url.is_none())
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
        let crop = super::cover_uv(animated.image_size, rect);
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
        if super::poster_overlay::custom_url(painter.ctx(), card).is_none() {
            landscape_logo(painter, rect, card, assets);
        }
        super::poster_overlay::paint_landscape_band(painter, rect, card);
    } else {
        let tones = assets.artwork_tones(card.artwork_url.as_deref());
        super::poster_overlay::paint(painter, rect, card, metrics.card_radius, tones);
    }

    if !labels || card.hide_title && card.row_kind == super::HomeRowKind::Collection {
        return;
    }
    let title_font = FontId::proportional(metrics.screen_card_title_size);
    let subtitle_font = crate::fonts::regular(metrics.screen_card_subtitle_size);
    let label_y = rect.bottom() + metrics.control_gap;
    paint_elided_text(
        painter,
        egui::Pos2::new(rect.left(), label_y),
        &card.title,
        title_font,
        rect.width(),
        Color32::WHITE,
    );
    paint_elided_text(
        painter,
        egui::Pos2::new(rect.left(), label_y + metrics.screen_card_title_size + 2.0),
        &card.subtitle,
        subtitle_font,
        rect.width(),
        Color32::from_white_alpha(185),
    );
}

fn landscape_logo(painter: &Painter, rect: Rect, card: &HomeCard, assets: &mut impl HomeAssets) {
    let Some((texture, logo_size)) = title_logo(
        painter.ctx().pixels_per_point(),
        card.logo_url.as_deref(),
        Vec2::new(rect.width() * 0.5, rect.height() * 0.3),
        ArtworkPriority::Visible,
        assets,
    ) else {
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
fn paint_elided_text(
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

pub(super) fn continue_card(
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
        Color32::from_white_alpha(235),
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
        Color32::from_black_alpha(190),
    );
    paint_vertical_gradient(
        painter,
        Rect::from_min_max(egui::Pos2::new(rect.left(), scrim_mid), rect.right_bottom()),
        Color32::from_black_alpha(190),
        Color32::from_black_alpha(245),
    );
    let title_font = FontId::proportional(metrics.screen_card_title_size);
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
    let logo_box = Vec2::new(content_width * 0.6, title_height * 1.6);
    let logo = title_logo(
        painter.ctx().pixels_per_point(),
        card.logo_url.as_deref(),
        logo_box,
        ArtworkPriority::Visible,
        assets,
    );
    if let Some((texture, logo_size)) = logo {
        let logo_rect = Rect::from_min_size(
            egui::Pos2::new(
                rect.left() + metrics.card_content_padding,
                title_top + title_height - logo_size.y,
            ),
            logo_size,
        );
        texture_image(
            painter,
            texture,
            logo_rect,
            Rect::from_min_max(egui::Pos2::ZERO, egui::Pos2::new(1.0, 1.0)),
            Color32::WHITE,
        );
    } else {
        painter.text(
            egui::Pos2::new(rect.left() + metrics.card_content_padding, title_top),
            Align2::LEFT_TOP,
            truncate_to_width(painter, &card.title, &title_font, content_width),
            title_font,
            Color32::WHITE,
        );
    }
    if has_subtitle {
        painter.text(
            egui::Pos2::new(rect.left() + metrics.card_content_padding, subtitle_top),
            Align2::LEFT_TOP,
            truncate_to_width(painter, &card.subtitle, &subtitle_font, content_width),
            subtitle_font.clone(),
            Color32::from_white_alpha(210),
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
        painter.rect_filled(badge, 3.0, Color32::from_black_alpha(190));
        painter.galley(badge.center() - galley.size() * 0.5, galley, Color32::WHITE);
    }
    let bar_origin = egui::Pos2::new(rect.left() + metrics.card_content_padding, bar_top);
    painter.rect_filled(
        Rect::from_min_size(
            bar_origin,
            Vec2::new(content_width, metrics.card_progress_height),
        ),
        2.0,
        Color32::from_white_alpha(70),
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

const DROPDOWN_CHROME: f32 = 44.0;

pub(super) fn dropdown_width_for_label(
    ui: &Ui,
    label: &str,
    metrics: UiMetrics,
    min_width: f32,
    max_width: f32,
) -> f32 {
    let font = FontId::proportional(metrics.nav_label_size + 1.0);
    let text_width = ui
        .painter()
        .layout_no_wrap(label.to_owned(), font, Color32::WHITE)
        .size()
        .x;
    (text_width + DROPDOWN_CHROME + 2.0).clamp(min_width, max_width.max(min_width))
}

fn dropdown_popup_width(
    ui: &Ui,
    options: &[(String, String)],
    minimum: f32,
    metrics: UiMetrics,
) -> f32 {
    let font = FontId::proportional(metrics.nav_label_size + 1.0);
    let widest_label = options
        .iter()
        .map(|(_, label)| {
            ui.painter()
                .layout_no_wrap(label.clone(), font.clone(), Color32::WHITE)
                .size()
                .x
        })
        .fold(0.0_f32, f32::max);
    (widest_label + metrics.control_gap * 3.0 + 24.0)
        .max(minimum)
        .min(600.0)
}

pub(super) fn sheet_choice(
    node: u64,
    key: &'static str,
    title: String,
    options: &[(String, String)],
    selected: &str,
) -> Option<(u64, crate::ChoiceRequest)> {
    (!options.is_empty()).then(|| {
        (
            node,
            crate::ChoiceRequest {
                key,
                title,
                options: options.to_vec(),
                selected: selected.to_owned(),
            },
        )
    })
}

pub(super) fn dropdown(
    ui: &mut Ui,
    id: impl std::hash::Hash,
    selected: &str,
    options: &[(String, String)],
    width: f32,
    sheet: bool,
    metrics: UiMetrics,
) -> (Response, Option<String>) {
    let selected_label = options
        .iter()
        .find(|(option, _)| option == selected)
        .map(|(_, label)| label.as_str())
        .unwrap_or(selected);
    let popup_width = dropdown_popup_width(ui, options, width, metrics);
    dropdown_inner(
        ui,
        id,
        selected,
        selected_label,
        options,
        width,
        popup_width,
        !options.is_empty(),
        sheet,
        metrics,
    )
}

pub(super) fn dropdown_placeholder(
    ui: &mut Ui,
    id: impl std::hash::Hash,
    placeholder: &str,
    width: f32,
    metrics: UiMetrics,
) -> Response {
    dropdown_inner(
        ui,
        id,
        "",
        placeholder,
        &[],
        width,
        width,
        false,
        false,
        metrics,
    )
    .0
}

fn dropdown_inner(
    ui: &mut Ui,
    id: impl std::hash::Hash,
    selected: &str,
    selected_label: &str,
    options: &[(String, String)],
    width: f32,
    popup_width: f32,
    enabled: bool,
    sheet: bool,
    metrics: UiMetrics,
) -> (Response, Option<String>) {
    let height = metrics.screen_control_height.max(38.0);
    let mut value = selected.to_owned();
    let response = choice_field(
        ui,
        Id::new(("fluxa-shared-dropdown", id)),
        Vec2::new(width.max(1.0), height),
        metrics.nav_label_size + 1.0,
        selected_label,
        options,
        &mut value,
        popup_width,
        enabled,
        sheet,
        metrics,
    );
    let changed = value != selected;
    (response, changed.then_some(value))
}

fn paint_chevron(painter: &Painter, center: Pos2, open: f32, color: Color32) {
    let flip = 1.0 - 2.0 * open;
    let stroke = egui::Stroke::new(1.6, color);
    let tip = center + Vec2::new(0.0, 2.5 * flip);
    painter.line_segment([center + Vec2::new(-4.5, -2.0 * flip), tip], stroke);
    painter.line_segment([tip, center + Vec2::new(4.5, -2.0 * flip)], stroke);
}

pub(super) fn choice_field(
    ui: &mut Ui,
    id: Id,
    size: Vec2,
    text_size: f32,
    current: &str,
    choices: &[(String, String)],
    selected: &mut String,
    popup_width: f32,
    enabled: bool,
    sheet: bool,
    metrics: UiMetrics,
) -> Response {
    let sense = if enabled {
        Sense::click()
    } else {
        Sense::hover()
    };
    let (rect, response) = ui.allocate_exact_size(size, sense);
    let popup_id = id.with("popup");
    let open = enabled && egui::Popup::is_id_open(ui.ctx(), popup_id);
    let turn = ui
        .ctx()
        .animate_bool_with_time(id.with("chevron"), open, 0.14);
    let radius = (rect.height() * 0.5).min(12.0);
    let fill = if !enabled {
        Color32::from_white_alpha(5)
    } else if open {
        Color32::from_white_alpha(22)
    } else if response.hovered() {
        Color32::from_white_alpha(16)
    } else {
        Color32::from_white_alpha(10)
    };
    let painter = ui.painter();
    painter.rect(
        rect,
        radius,
        fill,
        egui::Stroke::new(1.0, Color32::from_white_alpha(if open { 70 } else { 26 })),
        egui::StrokeKind::Inside,
    );
    let font = FontId::proportional(text_size);
    let text_color = if enabled {
        Color32::from_rgb(242, 243, 246)
    } else {
        Color32::from_white_alpha(110)
    };
    painter.text(
        rect.left_center() + Vec2::new(14.0, 0.0),
        Align2::LEFT_CENTER,
        truncate_to_width(
            painter,
            current,
            &font,
            (rect.width() - DROPDOWN_CHROME).max(1.0),
        ),
        font.clone(),
        text_color,
    );
    paint_chevron(
        painter,
        rect.right_center() - Vec2::new(18.0, 0.0),
        turn,
        Color32::from_white_alpha(if enabled { 190 } else { 80 }),
    );
    if !enabled || sheet {
        return response;
    }
    let row_height = (text_size + 18.0).max(34.0);
    egui::Popup::from_toggle_button_response(&response)
        .id(popup_id)
        .gap(6.0)
        .width(popup_width.max(rect.width()))
        .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
        .frame(
            egui::Frame::new()
                .fill(if liquid_glass() {
                    Color32::TRANSPARENT
                } else {
                    metrics.surface_raised
                })
                .stroke(egui::Stroke::new(1.0, metrics.border))
                .corner_radius(metrics.dialog_radius)
                .inner_margin(egui::Margin::same(5))
                .shadow(egui::epaint::Shadow {
                    offset: [0, 10],
                    blur: 28,
                    spread: 0,
                    color: Color32::from_black_alpha(150),
                }),
        )
        .show(|ui| {
            let backdrop = ui.painter().clone().with_clip_rect(Rect::EVERYTHING);
            let slot = liquid_glass().then(|| backdrop.add(egui::Shape::Noop));
            egui::ScrollArea::vertical()
                .max_height(300.0)
                .show(ui, |ui| {
                    ui.spacing_mut().item_spacing.y = 2.0;
                    for (value, label) in choices {
                        let (row, row_response) = ui.allocate_exact_size(
                            Vec2::new(ui.available_width(), row_height),
                            Sense::click(),
                        );
                        let active = value == selected;
                        if row_response.hovered() || active {
                            ui.painter().rect_filled(
                                row,
                                8.0,
                                Color32::from_white_alpha(if row_response.hovered() {
                                    18
                                } else {
                                    9
                                }),
                            );
                        }
                        ui.painter().text(
                            row.left_center() + Vec2::new(12.0, 0.0),
                            Align2::LEFT_CENTER,
                            truncate_to_width(
                                ui.painter(),
                                label,
                                &font,
                                (row.width() - 48.0).max(1.0),
                            ),
                            font.clone(),
                            if active {
                                Color32::WHITE
                            } else {
                                Color32::from_white_alpha(195)
                            },
                        );
                        if active {
                            let c = row.right_center() - Vec2::new(18.0, 0.0);
                            let stroke = egui::Stroke::new(1.8, Color32::WHITE);
                            ui.painter().line_segment(
                                [c + Vec2::new(-5.0, 0.0), c + Vec2::new(-1.5, 3.5)],
                                stroke,
                            );
                            ui.painter().line_segment(
                                [c + Vec2::new(-1.5, 3.5), c + Vec2::new(5.0, -3.5)],
                                stroke,
                            );
                        }
                        if row_response.clicked() {
                            selected.clone_from(value);
                            egui::Popup::close_id(ui.ctx(), popup_id);
                        }
                    }
                });
            if let Some(slot) = slot {
                backdrop.set(
                    slot,
                    glass_shape(ui.min_rect().expand(5.0), 12.0, metrics.surface_raised),
                );
            }
        });
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
    pill_button(
        ui,
        assets.icon("PlayFilled"),
        label,
        width,
        height,
        text_size,
        true,
        progress,
    )
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
    let font = FontId::proportional(text_size);
    let measure = |text: &str| {
        ui.painter()
            .layout_no_wrap(text.to_owned(), font.clone(), Color32::BLACK)
            .size()
            .x
            + 70.0
    };
    let mut label = label.to_owned();
    if let Some(limit) = width {
        while measure(&label) > limit && label.chars().count() > 1 {
            label = label.trim_end_matches('…').to_owned();
            label.pop();
            label = format!("{}…", label.trim_end());
        }
    }
    let label = label.as_str();
    let natural = measure(label);
    let width = width.unwrap_or(natural).max(natural);
    let inset = (width - natural) * 0.5;
    let (rect, response) = ui.allocate_exact_size(Vec2::new(width, height), Sense::click());
    let painter = ui.painter();
    crate::motion::press_scale(painter, rect, || {
        let (fill, ink) = match (primary, response.hovered()) {
            (true, false) => (Color32::WHITE, Color32::BLACK),
            (true, true) => (Color32::from_gray(225), Color32::BLACK),
            (false, false) => (Color32::from_white_alpha(28), Color32::WHITE),
            (false, true) => (Color32::from_white_alpha(46), Color32::WHITE),
        };
        if primary {
            painter.rect_filled(rect, height * 0.5, fill);
        } else {
            glass(painter, rect, height * 0.5, fill);
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
        painter.text(
            rect.left_center() + Vec2::new(inset + 44.0, -lift),
            Align2::LEFT_CENTER,
            label,
            font,
            ink,
        );
        if let Some(progress) = progress {
            let track = Rect::from_min_max(
                Pos2::new(rect.left() + 44.0, rect.center().y + text_size * 0.5 + 2.0),
                Pos2::new(rect.right() - 24.0, rect.center().y + text_size * 0.5 + 5.0),
            );
            painter.rect_filled(track, 1.5, ink.gamma_multiply(0.16));
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

pub struct ActionMenuItem {
    pub icon: &'static str,
    pub label: String,
    pub app_icon: Option<&'static str>,
}

pub enum ActionMenuOutcome {
    Pick(usize),
    Dismiss,
}

pub struct ActionMenuLayout {
    pub panel: Rect,
    pub rows: Vec<Rect>,
    header: Option<Rect>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ActionMenuStyle {
    Sheet,
    Panel,
    Popover,
}

impl ActionMenuStyle {
    fn for_viewport(viewport: Viewport) -> Self {
        if viewport.is_compact() {
            Self::Sheet
        } else if viewport.is_tv() {
            Self::Panel
        } else {
            Self::Popover
        }
    }

    fn has_scrim(self) -> bool {
        self != Self::Popover
    }

    fn large_rows(self) -> bool {
        self != Self::Popover
    }

    fn icon_size(self) -> f32 {
        if self.large_rows() { 24.0 } else { 18.0 }
    }

    fn row_radius(self) -> f32 {
        if self == Self::Sheet { 12.0 } else { 8.0 }
    }

    fn row_inset(self) -> f32 {
        if self == Self::Sheet { 16.0 } else { 12.0 }
    }

    fn header_inset(self) -> f32 {
        if self == Self::Sheet { 24.0 } else { 12.0 }
    }

    fn rule_shrink(self) -> f32 {
        if self == Self::Sheet { 0.0 } else { 16.0 }
    }
}

pub fn action_menu_layout(viewport: Viewport, count: usize, anchor: Pos2) -> ActionMenuLayout {
    let count = count as f32;
    let style = ActionMenuStyle::for_viewport(viewport);
    if style == ActionMenuStyle::Sheet {
        let (header, row) = (60.0, 56.0);
        let columns = if count > 8.0 { 2 } else { 1 };
        let lines = (count / columns as f32).ceil();
        let column_width = (viewport.width - 16.0) / columns as f32;
        let height = 20.0 + header + row * lines + viewport.safe_bottom + 12.0;
        let panel = Rect::from_min_size(
            Pos2::new(0.0, viewport.height - height),
            Vec2::new(viewport.width, height),
        );
        let top = panel.top() + 20.0 + header;
        ActionMenuLayout {
            panel,
            header: Some(Rect::from_min_size(
                Pos2::new(panel.left(), panel.top() + 20.0),
                Vec2::new(panel.width(), header),
            )),
            rows: (0..count as usize)
                .map(|index| {
                    let (line, column) = (index / columns, index % columns);
                    Rect::from_min_size(
                        Pos2::new(8.0 + column_width * column as f32, top + row * line as f32),
                        Vec2::new(column_width, row),
                    )
                })
                .collect(),
        }
    } else if style == ActionMenuStyle::Panel {
        let (header, row, pad, width) = (72.0, 60.0, 16.0, 520.0);
        let height = header + row * count + pad * 2.0;
        let panel = Rect::from_center_size(
            Pos2::new(viewport.width * 0.5, viewport.height * 0.5),
            Vec2::new(width, height),
        );
        ActionMenuLayout {
            panel,
            header: Some(Rect::from_min_size(
                Pos2::new(panel.left() + pad, panel.top() + pad),
                Vec2::new(width - pad * 2.0, header),
            )),
            rows: (0..count as usize)
                .map(|index| {
                    Rect::from_min_size(
                        Pos2::new(
                            panel.left() + pad,
                            panel.top() + pad + header + row * index as f32,
                        ),
                        Vec2::new(width - pad * 2.0, row),
                    )
                })
                .collect(),
        }
    } else {
        let (row, pad, width) = (40.0, 6.0, 240.0);
        let height = row * count + pad * 2.0;
        let x = if anchor.x + width > viewport.width - 8.0 {
            anchor.x - width
        } else {
            anchor.x
        };
        let y = if anchor.y + height > viewport.height - 8.0 {
            anchor.y - height
        } else {
            anchor.y
        };
        let panel =
            Rect::from_min_size(Pos2::new(x.max(8.0), y.max(8.0)), Vec2::new(width, height));
        ActionMenuLayout {
            panel,
            header: None,
            rows: (0..count as usize)
                .map(|index| {
                    Rect::from_min_size(
                        Pos2::new(panel.left() + pad, panel.top() + pad + row * index as f32),
                        Vec2::new(width - pad * 2.0, row),
                    )
                })
                .collect(),
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn draw_action_menu(
    context: &egui::Context,
    viewport: Viewport,
    metrics: UiMetrics,
    assets: &impl HomeAssets,
    title: &str,
    items: &[ActionMenuItem],
    selected: Option<usize>,
    anchor: Pos2,
    serial: u64,
) -> Option<ActionMenuOutcome> {
    let layout = action_menu_layout(viewport, items.len(), anchor);
    let style = ActionMenuStyle::for_viewport(viewport);
    let sheet = style == ActionMenuStyle::Sheet;
    let tv = style == ActionMenuStyle::Panel;
    if context.data(|data| data.get_temp::<u64>(Id::new("fluxa-action-menu-serial")))
        != Some(serial)
    {
        context.data_mut(|data| data.insert_temp(Id::new("fluxa-action-menu-serial"), serial));
        context.animate_bool_with_time(Id::new(("fluxa-action-menu", serial)), false, 0.0);
    }
    let reveal = context.animate_bool_with_time(Id::new(("fluxa-action-menu", serial)), true, 0.25);
    let eased = 1.0 - (1.0 - reveal).powi(3);
    let offset = if sheet {
        Vec2::new(0.0, (1.0 - eased) * layout.panel.height())
    } else {
        Vec2::ZERO
    };
    let mut outcome = None;
    egui::Area::new(Id::new("fluxa-action-menu-area"))
        .fixed_pos(Pos2::ZERO)
        .order(egui::Order::Debug)
        .show(context, |ui| {
            let screen =
                Rect::from_min_size(Pos2::ZERO, Vec2::new(viewport.width, viewport.height));
            let painter = ui.painter().clone().with_clip_rect(Rect::EVERYTHING);
            let backdrop = ui.interact(
                screen,
                Id::new("fluxa-action-menu-backdrop"),
                Sense::click(),
            );
            if style.has_scrim() {
                painter.rect_filled(
                    screen,
                    0.0,
                    Color32::from_black_alpha((150.0 * eased) as u8),
                );
            }
            let panel = layout.panel.translate(offset);
            let fill = Color32::from_rgb(20, 20, 22);
            let border = egui::Stroke::new(1.0, metrics.border);
            if sheet && liquid_glass() {
                glass(&painter, panel.with_max_y(panel.max.y + 40.0), 20.0, fill);
                painter.rect_filled(
                    Rect::from_center_size(
                        Pos2::new(panel.center().x, panel.top() + 10.0),
                        Vec2::new(36.0, 4.0),
                    ),
                    2.0,
                    Color32::from_white_alpha(60),
                );
            } else if liquid_glass() {
                glass(&painter, panel, 12.0, fill);
            } else if sheet {
                painter.rect(
                    panel.with_max_y(panel.max.y + 40.0),
                    egui::CornerRadius {
                        nw: 20,
                        ne: 20,
                        sw: 0,
                        se: 0,
                    },
                    fill,
                    border,
                    egui::StrokeKind::Inside,
                );
                painter.rect_filled(
                    Rect::from_center_size(
                        Pos2::new(panel.center().x, panel.top() + 10.0),
                        Vec2::new(36.0, 4.0),
                    ),
                    2.0,
                    Color32::from_white_alpha(60),
                );
            } else {
                painter.rect(panel, 12.0, fill, border, egui::StrokeKind::Inside);
            }
            if let Some(header) = layout.header.map(|header| header.translate(offset)) {
                let inset = style.header_inset();
                painter.text(
                    Pos2::new(header.left() + inset, header.center().y),
                    Align2::LEFT_CENTER,
                    truncate_to_width(
                        &painter,
                        title,
                        &FontId::proportional(metrics.text.title),
                        header.width() - inset * 2.0,
                    ),
                    FontId::proportional(metrics.text.title),
                    Color32::WHITE,
                );
                painter.hline(
                    panel.x_range().shrink(style.rule_shrink()),
                    header.bottom() - 0.5,
                    egui::Stroke::new(1.0, metrics.border),
                );
            }
            let label_size = if style.large_rows() {
                metrics.text.subtitle
            } else {
                metrics.text.body
            };
            let icon_size = style.icon_size();
            for (index, (item, row)) in items.iter().zip(&layout.rows).enumerate() {
                let row = row.translate(offset);
                let response = ui.interact(
                    row,
                    Id::new(("fluxa-action-menu-row", index)),
                    Sense::click(),
                );
                let focused = selected == Some(index);
                let (bg, ink) = if tv && focused {
                    (Color32::WHITE, Color32::BLACK)
                } else if focused || response.hovered() {
                    (Color32::from_white_alpha(18), Color32::WHITE)
                } else {
                    (Color32::TRANSPARENT, Color32::WHITE)
                };
                painter.rect_filled(row, style.row_radius(), bg);
                let inset = style.row_inset();
                if let Some(texture) = item.app_icon.and_then(|id| assets.app_icon(id)) {
                    let size = icon_size + 12.0;
                    painter.image(
                        texture,
                        Rect::from_center_size(
                            Pos2::new(row.left() + inset + icon_size * 0.5, row.center().y),
                            Vec2::splat(size),
                        ),
                        full_uv(),
                        Color32::WHITE,
                    );
                    if let Some(check) = assets.icon(item.icon) {
                        painter.image(
                            check,
                            Rect::from_center_size(
                                Pos2::new(row.right() - inset - icon_size * 0.5, row.center().y),
                                Vec2::splat(icon_size),
                            ),
                            full_uv(),
                            ink,
                        );
                    }
                } else if let Some(icon) = assets.icon(item.icon) {
                    painter.image(
                        icon,
                        Rect::from_center_size(
                            Pos2::new(row.left() + inset + icon_size * 0.5, row.center().y),
                            Vec2::splat(icon_size),
                        ),
                        full_uv(),
                        ink,
                    );
                }
                painter.text(
                    Pos2::new(row.left() + inset * 2.0 + icon_size, row.center().y),
                    Align2::LEFT_CENTER,
                    &item.label,
                    FontId::proportional(label_size),
                    ink,
                );
                if response.clicked() {
                    outcome = Some(ActionMenuOutcome::Pick(index));
                }
            }
            if outcome.is_none()
                && (backdrop.clicked() || backdrop.secondary_clicked())
                && !panel.contains(backdrop.interact_pointer_pos().unwrap_or(panel.center()))
            {
                outcome = Some(ActionMenuOutcome::Dismiss);
            }
        });
    if reveal < 1.0 {
        context.request_repaint();
    }
    outcome
}

pub(crate) fn brand_lockup(
    painter: &Painter,
    center: Pos2,
    mark_size: f32,
    font_size: f32,
    alpha: u8,
    assets: &impl HomeAssets,
) {
    let gap = mark_size * 0.22;
    let galley = painter.layout_no_wrap(
        "fluxa".to_owned(),
        FontId::proportional(font_size),
        Color32::WHITE,
    );
    let width = mark_size + gap + galley.size().x;
    let left = center.x - width * 0.5;
    let tint = Color32::from_white_alpha(alpha);
    if let Some(mark) = assets.brand_mark() {
        painter.image(
            mark,
            Rect::from_center_size(
                Pos2::new(left + mark_size * 0.5, center.y),
                Vec2::splat(mark_size),
            ),
            full_uv(),
            tint,
        );
    }
    let text_pos = Pos2::new(left + mark_size + gap, center.y - galley.size().y * 0.5);
    painter.galley(text_pos, galley, tint);
}

pub(super) fn toggle(
    context: &egui::Context,
    painter: &Painter,
    id: Id,
    track: Rect,
    on: bool,
    metrics: UiMetrics,
) {
    let t = context.animate_bool_with_time_and_easing(id, on, 0.22, egui::emath::easing::cubic_out);
    let off_track = Color32::from_rgb(54, 56, 59);
    painter.rect_filled(
        track,
        track.height() * 0.5,
        lerp_color(off_track, metrics.accent, t),
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
        Color32::from_rgb(168, 172, 178),
        if metrics.accent_foreground == Color32::WHITE {
            Color32::WHITE
        } else {
            Color32::from_rgb(24, 25, 28)
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
        painter.rect_stroke(
            rect.expand(metrics.focus_ring_expand),
            metrics.focus_ring_radius,
            egui::Stroke::new(metrics.focus_ring_width, Color32::WHITE),
            egui::StrokeKind::Outside,
        );
    }
}

pub fn is_text_node(node: u64) -> bool {
    matches!(
        node,
        super::NODE_LIBRARY_SEARCH
            | super::NODE_DISCOVER_SEARCH
            | super::NODE_SETTINGS_SEARCH
            | super::NODE_SETTINGS_ADDON_URL
            | super::NODE_SETTINGS_PLUGIN_URL
    ) || super::poster_field(node).is_some()
        || super::server_input(node).is_some()
}

static LIQUID_GLASS: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(true);

pub fn set_liquid_glass(enabled: bool) {
    LIQUID_GLASS.store(enabled, std::sync::atomic::Ordering::Relaxed);
}

pub(crate) fn liquid_glass() -> bool {
    LIQUID_GLASS.load(std::sync::atomic::Ordering::Relaxed)
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
        Color32::WHITE,
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
        Color32::WHITE
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
        let color = badge
            .border
            .or(badge.fill)
            .unwrap_or(Color32::from_white_alpha(90));
        painter.rect_stroke(
            chip,
            rounding,
            egui::Stroke::new(1.0, color),
            egui::StrokeKind::Inside,
        );
        badge.text.or(badge.fill).unwrap_or(Color32::WHITE)
    } else {
        let fill = badge.fill.unwrap_or(Color32::from_white_alpha(24));
        painter.rect_filled(chip, rounding, fill);
        if let Some(border) = badge.border {
            painter.rect_stroke(
                chip,
                rounding,
                egui::Stroke::new(1.0, border),
                egui::StrokeKind::Inside,
            );
        }
        badge.text.unwrap_or(match badge.fill {
            Some(fill) => readable_on(fill),
            None => Color32::from_white_alpha(230),
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
            Color32::WHITE,
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
    assets: &mut impl HomeAssets,
) -> Response {
    let pad = 16.0;
    let text_width = (width - pad * 2.0).max(1.0);
    let painter = ui.painter().clone();
    let addon = (show_addon && !source.name.starts_with(&source.addon)).then(|| {
        painter.layout_no_wrap(
            source.addon.clone(),
            crate::fonts::regular(metrics.screen_card_subtitle_size - 2.0),
            metrics.text_muted,
        )
    });
    let addon_width = addon.as_ref().map_or(0.0, |galley| galley.size().x + 12.0);
    let mut name_job = crate::emoji::job(
        &source.name,
        FontId::proportional(metrics.screen_card_title_size),
        Color32::PLACEHOLDER,
        text_width - addon_width,
    );
    name_job.wrap.max_rows = 3;
    let name = painter.layout_job(name_job);
    let detail = (!source.detail.is_empty()).then(|| {
        painter.layout_job(crate::emoji::job(
            &source.detail,
            crate::fonts::regular(metrics.screen_card_subtitle_size - 1.0),
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

pub(super) fn stream_row(
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
        egui::Stroke::new(1.0, metrics.border),
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
        let pos = egui::Pos2::new(rect.right() - pad - addon.size().x, origin.y + 2.0);
        painter.galley(pos, addon, Color32::WHITE);
    }
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
        || super::badge_input(node).is_some()
    response
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
            .size(11.0)
            .color(metrics.text_muted),
    );
    ui.add_space(10.0);
    ui.label(
        RichText::new(title)
            .size(36.0)
            .strong()
            .color(Color32::WHITE),
    );
    ui.add_space(8.0);
    ui.label(RichText::new(subtitle).size(14.0).color(metrics.text_muted));
    ui.add_space(28.0);
}

pub(super) fn field_label(ui: &mut egui::Ui, text: &str, metrics: UiMetrics) {
    ui.label(
        RichText::new(text.to_uppercase())
            .size(10.0)
            .color(metrics.text_muted),
    );
    ui.add_space(6.0);
}

pub(super) fn note(ui: &mut egui::Ui, text: &str, metrics: UiMetrics) {
    ui.add_space(4.0);
    ui.label(RichText::new(text).size(12.0).color(metrics.text_secondary));
}

pub(super) fn secondary_button(ui: &mut egui::Ui, label: &str, width: f32) -> egui::Response {
    ui.add_sized(
        [width, 44.0],
        egui::Button::new(
            RichText::new(label)
                .size(13.0)
                .strong()
                .color(Color32::WHITE),
        )
        .fill(Color32::from_white_alpha(28))
        .corner_radius(22.0),
    )
}

pub(super) fn primary_button(
    ui: &mut egui::Ui,
    label: &str,
    width: f32,
    enabled: bool,
) -> egui::Response {
    ui.add_enabled(
        enabled,
        egui::Button::new(RichText::new(label).size(13.0).strong().color(if enabled {
            Color32::BLACK
        } else {
            Color32::from_white_alpha(80)
        }))
        .fill(if enabled {
            Color32::WHITE
        } else {
            Color32::from_white_alpha(24)
        })
        .corner_radius(22.0)
        .min_size(Vec2::new(width, 44.0)),
    )
}

pub(super) fn danger_button(
    ui: &mut egui::Ui,
    label: &str,
    width: f32,
    metrics: UiMetrics,
) -> egui::Response {
    ui.add_sized(
        [width, 44.0],
        egui::Button::new(
            RichText::new(label)
                .size(13.0)
                .strong()
                .color(Color32::WHITE),
        )
        .fill(metrics.danger)
        .corner_radius(22.0),
    )
}

pub(super) fn panel(ui: &mut egui::Ui, add: impl FnOnce(&mut egui::Ui), metrics: UiMetrics) {
    egui::Frame::NONE
        .fill(metrics.surface)
        .stroke(egui::Stroke::new(1.0, metrics.border))
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
    context
        .layer_painter(egui::LayerId::new(
            egui::Order::Middle,
            Id::new((id, "scrim")),
        ))
        .rect_filled(screen, 0.0, Color32::from_black_alpha(170));
    egui::Area::new(Id::new(id))
        .order(egui::Order::Foreground)
        .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
        .show(context, |ui| {
            egui::Frame::NONE
                .fill(metrics.surface_raised)
                .stroke(egui::Stroke::new(1.0, metrics.border))
                .corner_radius(metrics.dialog_radius)
                .inner_margin(egui::Margin::same(26))
                .show(ui, |ui| {
                    ui.set_width(340.0);
                    add(ui);
                });
        });
}

pub(super) fn text_button(ui: &mut Ui, label: &str, size: f32, alpha: u8) -> Response {
    ui.add(
        egui::Button::new(
            RichText::new(label)
                .size(size)
                .color(Color32::from_white_alpha(alpha)),
        )
        .frame(false),
    )
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
        .stroke(egui::Stroke::new(1.0, metrics.border))
        .corner_radius(radius)
        .inner_margin(margin)
        .show(ui, add);
}
