//! Reusable Fluxa UI components.
//!
//! Components own visual paint and typography. Screens own composition and
//! navigation only. This keeps desktop, Android, and TV on the same card and
//! control implementation while allowing each host to provide a different
//! viewport and input adapter.

use egui::{Align2, Color32, FontId, Id, Painter, Rect, Response, RichText, Sense, Ui, Vec2};
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
    let icon_center = egui::Pos2::new(rect.left() + 15.0, rect.center().y);
    if let Some(icon) = icon {
        let icon_rect = Rect::from_center_size(icon_center, Vec2::splat(24.0));
        texture_image(painter, icon, icon_rect, full_uv(), metrics.text_secondary);
    }
    let text_x = rect.left() + 38.0;
    let text_width = (rect.right() - text_x).max(1.0);
    let title_font = FontId::proportional(metrics.screen_section_title_size);
    let description_font = FontId::proportional(metrics.screen_body_size);
    painter.text(
        egui::Pos2::new(text_x, icon_center.y - metrics.control_gap * 0.35),
        Align2::LEFT_BOTTOM,
        truncate_to_width(painter, title, &title_font, text_width),
        title_font,
        metrics.text_primary,
    );
    painter.text(
        egui::Pos2::new(text_x, icon_center.y + metrics.control_gap * 0.3),
        Align2::LEFT_TOP,
        truncate_to_width(painter, description, &description_font, text_width),
        description_font,
        metrics.text_secondary,
    );
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ButtonKind {
    Primary,
    Secondary,
    Subtle,
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
        ButtonKind::Subtle => Color32::from_white_alpha(24),
        ButtonKind::Accent | ButtonKind::Selected => metrics.accent,
    };
    let text_color = match kind {
        ButtonKind::Primary => metrics.accent_foreground,
        ButtonKind::Secondary | ButtonKind::Subtle => Color32::WHITE,
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

/// Gives egui combo-box popups the same dark, rounded surface as their trigger.
/// `ComboBox` otherwise creates its popup using the context's global style,
/// which made settings menus look like unrelated native widgets.
pub(super) fn dropdown_popup_style(metrics: UiMetrics) -> egui::style::StyleModifier {
    egui::style::StyleModifier::new(move |style| {
        let visuals = &mut style.visuals;
        // Keep the popup visually attached to the trigger instead of falling
        // back to egui's platform-default (blue) selection and gray menu.
        let dropdown_surface = Color32::from_rgb(60, 60, 60);
        visuals.window_fill = dropdown_surface;
        visuals.window_stroke = egui::Stroke::new(1.0, Color32::from_white_alpha(34));
        visuals.window_corner_radius = egui::CornerRadius::same(10);
        visuals.widgets.noninteractive.bg_fill = dropdown_surface;
        visuals.widgets.noninteractive.fg_stroke.color = Color32::from_rgb(232, 233, 236);
        visuals.widgets.inactive.bg_fill = Color32::TRANSPARENT;
        visuals.widgets.inactive.bg_stroke = egui::Stroke::NONE;
        visuals.widgets.hovered.bg_fill = Color32::from_white_alpha(16);
        visuals.widgets.hovered.bg_stroke = egui::Stroke::NONE;
        visuals.widgets.active.bg_fill = metrics.accent;
        visuals.widgets.active.fg_stroke.color = metrics.accent_foreground;
        visuals.widgets.open.bg_fill = Color32::from_white_alpha(12);
        visuals.widgets.open.bg_stroke = egui::Stroke::new(1.0, metrics.accent);
        visuals.widgets.open.fg_stroke.color = Color32::from_rgb(245, 245, 247);
        visuals.selection.bg_fill = metrics.accent;
        visuals.selection.stroke.color = metrics.accent_foreground;
        for widget in [
            &mut visuals.widgets.inactive,
            &mut visuals.widgets.hovered,
            &mut visuals.widgets.active,
            &mut visuals.widgets.open,
        ] {
            widget.corner_radius = egui::CornerRadius::same(7);
        }
        style.spacing.item_spacing.y = 3.0;
        style.spacing.button_padding = egui::vec2(10.0, 7.0);
    })
}

pub(super) fn dropdown_frame() -> egui::Frame {
    egui::Frame::new()
        .fill(Color32::from_rgb(60, 60, 60))
        .stroke(egui::Stroke::new(1.0, Color32::from_white_alpha(34)))
        .corner_radius(9.0)
        .inner_margin(0.0)
        .shadow(egui::epaint::Shadow::NONE)
}

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
    // Match the actual label inset, reserved chevron area, and gap between
    // them; don't add popup padding to the closed control's natural width.
    let chrome = metrics.control_gap * 3.0;
    (text_width + chrome).clamp(min_width, max_width.max(min_width))
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
    let popup_id = Id::new(("fluxa-shared-dropdown-popup", id));
    let sense = if enabled {
        Sense::click()
    } else {
        Sense::hover()
    };
    let (rect, response) = ui.allocate_exact_size(Vec2::new(width.max(1.0), height), sense);
    let is_open = enabled && egui::Popup::is_id_open(ui.ctx(), popup_id);
    let fill = if !enabled {
        Color32::from_rgb(40, 40, 40)
    } else if is_open || response.hovered() {
        Color32::from_rgb(68, 68, 68)
    } else {
        Color32::from_rgb(60, 60, 60)
    };
    ui.painter()
        .rect_filled(rect, metrics.screen_control_radius, fill);
    ui.painter().rect_stroke(
        rect,
        metrics.screen_control_radius,
        egui::Stroke::new(
            1.0,
            if is_open {
                metrics.accent
            } else {
                Color32::from_white_alpha(if enabled { 28 } else { 15 })
            },
        ),
        egui::StrokeKind::Inside,
    );
    let text_color = if enabled {
        Color32::from_rgb(244, 244, 246)
    } else {
        Color32::from_white_alpha(112)
    };
    let font = FontId::proportional(metrics.nav_label_size + 1.0);
    ui.painter().text(
        rect.left_center() + Vec2::new(metrics.control_gap, 0.0),
        Align2::LEFT_CENTER,
        truncate_to_width(
            ui.painter(),
            selected_label,
            &font,
            (rect.width() - metrics.control_gap * 3.0).max(1.0),
        ),
        font,
        text_color,
    );
    let chevron_center = rect.right_center() - Vec2::new(metrics.control_gap + 2.0, 0.0);
    let chevron_color = if enabled {
        Color32::from_white_alpha(190)
    } else {
        Color32::from_white_alpha(82)
    };
    ui.painter().line_segment(
        [
            chevron_center + Vec2::new(-4.0, -1.5),
            chevron_center + Vec2::new(0.0, 2.5),
        ],
        egui::Stroke::new(1.6, chevron_color),
    );
    ui.painter().line_segment(
        [
            chevron_center + Vec2::new(0.0, 2.5),
            chevron_center + Vec2::new(4.0, -1.5),
        ],
        egui::Stroke::new(1.6, chevron_color),
    );

    let mut value = selected.to_owned();
    let before = value.clone();
    if enabled {
        egui::Popup::menu(&response)
            .id(popup_id)
            .width(popup_width)
            .style(dropdown_popup_style(metrics))
            .frame(dropdown_frame())
            .show(|ui| {
                ui.set_min_width(popup_width - 2.0);
                ui.spacing_mut().item_spacing.y = 3.0;
                egui::ScrollArea::vertical()
                    .max_height(280.0)
                    .show(ui, |ui| {
                        for (option, label) in options {
                            let item = ui.selectable_label(option == selected, label);
                            if item.clicked() {
                                value.clone_from(option);
                            }
                        }
                    });
            });
    }
    (response, (value != before).then_some(value))
}
