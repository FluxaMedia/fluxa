//! Reusable Fluxa UI components.
//!
//! Components own visual paint and typography. Screens own composition and
//! navigation only. This keeps desktop, Android, and TV on the same card and
//! control implementation while allowing each host to provide a different
//! viewport and input adapter.

use egui::{Align2, Color32, FontId, Id, Painter, Pos2, Rect, Response, RichText, Sense, Ui, Vec2};
use std::hash::{Hash, Hasher};

use super::{
    ArtworkPriority, HomeAssets, HomeCard, UiMetrics, Viewport, artwork_target_size, cover_uv,
    full_uv, paint_vertical_gradient, truncate_to_width,
};

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
    let subtitle_font = FontId::proportional(metrics.screen_card_subtitle_size);
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
    let Some(texture) = assets.texture_for(url, target_size, priority) else {
        return false;
    };
    let uv = assets
        .texture_size(url)
        .map(|size| cover_uv(size, rect))
        .unwrap_or_else(full_uv);
    texture_image(painter, texture, rect, uv, tint);
    true
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
    let description_font = FontId::proportional(metrics.screen_body_size);
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
            .font(FontId::proportional(metrics.screen_body_size + 1.0))
            .vertical_align(egui::Align::Center)
            .hint_text(RichText::new(hint).color(Color32::from_white_alpha(110)))
            .text_color(Color32::WHITE)
            .margin(Vec2::ZERO),
    );
    let stroke = if response.has_focus() {
        egui::Stroke::new(1.0, Color32::from_white_alpha(120))
    } else {
        egui::Stroke::new(1.0, Color32::from_white_alpha(22))
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
        ButtonKind::Primary => metrics.accent,
        ButtonKind::Secondary => Color32::from_white_alpha(52),
        ButtonKind::Accent | ButtonKind::Selected => metrics.accent,
    };
    let text_color = match kind {
        ButtonKind::Primary => metrics.accent_foreground,
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
        .stroke(egui::Stroke::new(
            1.0,
            Color32::from_white_alpha(
                if matches!(
                    kind,
                    ButtonKind::Primary | ButtonKind::Accent | ButtonKind::Selected
                ) {
                    0
                } else {
                    55
                },
            ),
        ))
        .corner_radius(metrics.screen_control_radius),
    )
}

pub(super) fn poster_card(
    painter: &Painter,
    rect: Rect,
    card: &HomeCard,
    _column_index: usize,
    viewport: Viewport,
    metrics: UiMetrics,
    assets: &mut impl HomeAssets,
    motion_active: bool,
) {
    painter.rect_filled(rect, metrics.card_radius, Color32::from_rgb(28, 28, 34));

    // Show the ordinary poster as soon as it is ready. Animated artwork can
    // take substantially longer to prepare, so don't let its decode leave a
    // blank tile or occupy the first artwork requests for an entire row.
    let has_static_artwork = artwork_image(
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
    super::poster_overlay::paint(painter, rect, card, metrics.card_radius);

    if card.hide_title && card.row_kind == super::HomeRowKind::Collection {
        return;
    }
    let title_font = FontId::proportional(if viewport.is_tv() {
        metrics.screen_card_title_size_tv
    } else {
        metrics.screen_card_title_size
    });
    let subtitle_font = FontId::proportional(metrics.screen_card_subtitle_size);
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
    _column_index: usize,
    viewport: Viewport,
    metrics: UiMetrics,
    assets: &mut impl HomeAssets,
) {
    painter.rect_filled(rect, metrics.card_radius, Color32::from_rgb(28, 28, 34));

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

    paint_vertical_gradient(
        painter,
        rect,
        Color32::TRANSPARENT,
        Color32::from_black_alpha(210),
    );
    let title_font = FontId::proportional(if viewport.is_tv() {
        metrics.screen_card_title_size_tv
    } else {
        metrics.screen_card_title_size
    });
    let subtitle_font = FontId::proportional(metrics.screen_card_subtitle_size);
    let content_width = (rect.width() - metrics.card_content_padding * 2.0).max(1.0);
    let bar_top = rect.bottom() - metrics.card_content_padding - metrics.card_progress_height;
    let subtitle_height = metrics.screen_card_subtitle_size * 1.25;
    let title_height = if viewport.is_tv() {
        metrics.screen_card_title_size_tv * 1.25
    } else {
        metrics.screen_card_title_size * 1.25
    };
    let subtitle_top = bar_top - metrics.control_gap * 0.5 - subtitle_height;
    let title_top = subtitle_top - metrics.control_gap * 0.35 - title_height;
    painter.text(
        egui::Pos2::new(rect.left() + metrics.card_content_padding, title_top),
        Align2::LEFT_TOP,
        truncate_to_width(painter, &card.title, &title_font, content_width),
        title_font,
        Color32::WHITE,
    );
    painter.text(
        egui::Pos2::new(rect.left() + metrics.card_content_padding, subtitle_top),
        Align2::LEFT_TOP,
        truncate_to_width(painter, &card.subtitle, &subtitle_font, content_width),
        subtitle_font,
        Color32::from_white_alpha(185),
    );
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

pub(super) fn dropdown(
    ui: &mut Ui,
    id: impl std::hash::Hash,
    selected: &str,
    options: &[(String, String)],
    width: f32,
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
    dropdown_inner(ui, id, "", placeholder, &[], width, width, false, metrics).0
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
) -> Response {
    let sense = if enabled { Sense::click() } else { Sense::hover() };
    let (rect, response) = ui.allocate_exact_size(size, sense);
    let popup_id = id.with("popup");
    let open = enabled && egui::Popup::is_id_open(ui.ctx(), popup_id);
    let turn = ui.ctx().animate_bool_with_time(id.with("chevron"), open, 0.14);
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
        truncate_to_width(painter, current, &font, (rect.width() - DROPDOWN_CHROME).max(1.0)),
        font.clone(),
        text_color,
    );
    paint_chevron(
        painter,
        rect.right_center() - Vec2::new(18.0, 0.0),
        turn,
        Color32::from_white_alpha(if enabled { 190 } else { 80 }),
    );
    if !enabled {
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
                .fill(Color32::from_rgb(24, 24, 24))
                .stroke(egui::Stroke::new(1.0, Color32::from_white_alpha(26)))
                .corner_radius(12.0)
                .inner_margin(egui::Margin::same(5))
                .shadow(egui::epaint::Shadow {
                    offset: [0, 10],
                    blur: 28,
                    spread: 0,
                    color: Color32::from_black_alpha(150),
                }),
        )
        .show(|ui| {
            egui::ScrollArea::vertical().max_height(300.0).show(ui, |ui| {
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
                            Color32::from_white_alpha(if row_response.hovered() { 18 } else { 9 }),
                        );
                    }
                    ui.painter().text(
                        row.left_center() + Vec2::new(12.0, 0.0),
                        Align2::LEFT_CENTER,
                        truncate_to_width(ui.painter(), label, &font, (row.width() - 48.0).max(1.0)),
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
                        ui.painter()
                            .line_segment([c + Vec2::new(-5.0, 0.0), c + Vec2::new(-1.5, 3.5)], stroke);
                        ui.painter()
                            .line_segment([c + Vec2::new(-1.5, 3.5), c + Vec2::new(5.0, -3.5)], stroke);
                    }
                    if row_response.clicked() {
                        selected.clone_from(value);
                        egui::Popup::close_id(ui.ctx(), popup_id);
                    }
                }
            });
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
    let font = FontId::proportional(text_size);
    let width = width.unwrap_or_else(|| {
        ui.painter()
            .layout_no_wrap(label.to_owned(), font.clone(), Color32::BLACK)
            .size()
            .x
            + 70.0
    });
    let (rect, response) = ui.allocate_exact_size(Vec2::new(width, height), Sense::click());
    let painter = ui.painter();
    painter.rect_filled(
        rect,
        height * 0.5,
        if response.hovered() {
            Color32::from_gray(225)
        } else {
            Color32::WHITE
        },
    );
    let lift = if progress.is_some() { 4.0 } else { 0.0 };
    if let Some(icon) = assets.icon("PlayFilled") {
        painter.image(
            icon,
            Rect::from_center_size(rect.left_center() + Vec2::new(28.0, 0.0), Vec2::splat(18.0)),
            full_uv(),
            Color32::BLACK,
        );
    }
    painter.text(
        rect.left_center() + Vec2::new(44.0, -lift),
        Align2::LEFT_CENTER,
        label,
        font,
        Color32::BLACK,
    );
    if let Some(progress) = progress {
        let track = Rect::from_min_max(
            Pos2::new(rect.left() + 44.0, rect.center().y + text_size * 0.5 + 2.0),
            Pos2::new(rect.right() - 24.0, rect.center().y + text_size * 0.5 + 5.0),
        );
        painter.rect_filled(track, 1.5, Color32::from_black_alpha(40));
        painter.rect_filled(
            Rect::from_min_size(track.min, Vec2::new(track.width() * progress.clamp(0.0, 1.0), track.height())),
            1.5,
            Color32::BLACK,
        );
    }
    response
}
