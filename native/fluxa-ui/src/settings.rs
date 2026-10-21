use super::*;
use crate::draw_text;

mod about;
mod account;
mod addons;
mod ambient;
mod hero;
mod badges;
mod collections;
mod detail_sections;
mod model;
mod sections;
pub use account::*;
pub use addons::{addon_action, addon_transport_url, server_index, server_input};
pub use badges::{badge_input, badge_input_label};
pub use collections::{collection_input, collection_input_label, collection_view_modes};
pub use model::*;
pub use sections::*;

fn settings_category_icon(
    assets: &impl HomeAssets,
    painter: &egui::Painter,
    center: Pos2,
    name: &str,
    color: Color32,
) {
    if let Some(icon) = assets.icon(name) {
        let rect = Rect::from_center_size(center, Vec2::splat(19.0));
        painter.image(
            icon,
            rect,
            Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
            color,
        );
    }
}

const APPEARANCE_PAGE_HEADER_HEIGHT: f32 = 66.0;
const APPEARANCE_GROUP_HEADING_HEIGHT: f32 = 22.0;
const APPEARANCE_GROUP_GAP: f32 = 22.0;
const FIELD_HEIGHT: f32 = 118.0;
const SUBTITLE_PREVIEW_HEIGHT: f32 = 168.0;

pub(super) fn group_heading(
    painter: &egui::Painter,
    rect: Rect,
    top: f32,
    title: &str,
    metrics: UiMetrics,
) {
    let compact = metrics.compact;
    draw_text(
        painter,
        Pos2::new(rect.left() + if compact { 16.0 } else { 8.0 }, top),
        Align2::LEFT_TOP,
        if compact {
            title.to_owned()
        } else {
            title.to_uppercase()
        },
        crate::fonts::regular(metrics.screen_card_subtitle_size + if compact { 1.0 } else { 0.0 }),
        ink(if compact { 120 } else { 145 }),
        None,
    );
}

fn wrapped_regular(
    painter: &egui::Painter,
    text: &str,
    size: f32,
    color: Color32,
    width: f32,
    rows: usize,
) -> std::sync::Arc<egui::Galley> {
    let mut job =
        egui::text::LayoutJob::simple(text.to_owned(), crate::fonts::regular(size), color, width);
    job.wrap.max_rows = rows;
    painter.layout_job(job)
}

fn settings_group_card_height(row_count: usize, metrics: UiMetrics) -> f32 {
    metrics.settings_row_height
        + row_count.saturating_sub(1) as f32 * metrics.settings_row_spacing
        + metrics.settings_card_padding
}

const ROW_SUBTITLE_EXTRA: f32 = 18.0;
const SWATCH_STRIP_EXTRA: f32 = 56.0;

fn row_extra(index: usize, width: f32, language: &str, metrics: UiMetrics) -> f32 {
    let Some(row) = settings_row_by_index(index) else {
        return 0.0;
    };
    if row.key == "accentColorArgb" {
        return SWATCH_STRIP_EXTRA;
    }
    if !metrics.compact || !row.options.is_empty() {
        return 0.0;
    }
    let text_width =
        width - metrics.settings_row_inset * 2.0 - metrics.settings_toggle_width - 19.0;
    let size = metrics.screen_card_subtitle_size + 2.0;
    let description = localized(&format!("settings.row.{}.description", row.key), language);
    if description.chars().count() as f32 * size * 0.45 > text_width {
        ROW_SUBTITLE_EXTRA
    } else {
        0.0
    }
}

fn rows_extra(rows: &[usize], width: f32, language: &str, metrics: UiMetrics) -> f32 {
    rows.iter()
        .map(|&index| row_extra(index, width, language, metrics))
        .sum()
}

fn described(mut metrics: UiMetrics) -> UiMetrics {
    metrics.settings_row_height += ROW_SUBTITLE_EXTRA;
    metrics.settings_row_spacing += ROW_SUBTITLE_EXTRA;
    metrics
}

pub(super) fn row_applies(key: &str, viewport: Viewport) -> bool {
    let compact = viewport.is_compact();
    let desktop = !compact && !viewport.is_tv();
    match key {
        "navLayout" | "navMode" | "navSidebarMode" | "interfaceDensity" | "navFloating"
        | "navLabels" | "posterHoverPreview" => !compact,
        "discordRichPresenceEnabled" => desktop,
        "detailSeasonPostersOnHero" => false,
        "automaticUpdates" => matches!(
            viewport.platform,
            UiPlatform::Android | UiPlatform::Windows | UiPlatform::Macos | UiPlatform::Linux
        ),
        "backgroundPlayback" | "pictureInPicture" | "torrentWifiOnly" => !desktop,
        "playerShowSettings" | "playerShowVolume" | "playerShowFullscreen" => desktop,
        "playerGestures" | "playerDoubleTapSeek" | "playerCenterControls" => compact,
        _ => true,
    }
}

pub fn section_applies(title: &str, viewport: Viewport) -> bool {
    match title {
        "Shortcuts" => !(viewport.is_compact() || viewport.is_tv()),
        _ => true,
    }
}

pub fn settings_page_for_node(node: u64) -> Option<usize> {
    let count = SETTINGS_SECTIONS.len() as u64;
    [NODE_SETTINGS_SECTION_BASE, NODE_SETTINGS_PAGE_BASE]
        .into_iter()
        .find(|base| (*base..base + count).contains(&node))
        .map(|base| (node - base) as usize)
}

pub fn addons_page() -> usize {
    SETTINGS_SECTIONS
        .iter()
        .position(|section| section.title == "Add-ons")
        .unwrap_or_default()
}

pub fn category_pages(index: usize) -> impl Iterator<Item = usize> {
    let category = SETTINGS_SECTIONS[index.min(SETTINGS_SECTIONS.len() - 1)].category;
    (0..SETTINGS_SECTIONS.len()).filter(move |&page| SETTINGS_SECTIONS[page].category == category)
}

pub fn categories(viewport: Viewport) -> impl Iterator<Item = usize> {
    (0..SETTINGS_SECTIONS.len()).filter(move |&index| {
        let section = &SETTINGS_SECTIONS[index];
        section_applies(section.title, viewport)
            && (index == 0 || SETTINGS_SECTIONS[index - 1].category != section.category)
    })
}

fn shows_hub(settings: &SettingsModel) -> bool {
    settings.search.trim().is_empty()
        && !settings.page_open
        && category_pages(settings.active_section).count() > 1
}

fn page_card_height(metrics: UiMetrics) -> f32 {
    metrics.settings_page_card_height
}

fn subpages(active: usize) -> impl Iterator<Item = usize> {
    category_pages(active).filter(move |&page| page != active)
}

fn links_height(metrics: UiMetrics, settings: &SettingsModel) -> f32 {
    if shows_hub(settings) {
        subpages(settings.active_section).count() as f32 * page_card_height(metrics)
            + APPEARANCE_GROUP_GAP
    } else {
        0.0
    }
}

fn draw_page_cards(
    context: &egui::Context,
    viewport: Viewport,
    painter: &egui::Painter,
    active: usize,
    language: &str,
    metrics: UiMetrics,
    top_left: Pos2,
    width: f32,
    layout: &mut HomeLayout,
) {
    let height = page_card_height(metrics);
    let pages: Vec<usize> = subpages(active).collect();
    let card = Rect::from_min_size(top_left, Vec2::new(width, pages.len() as f32 * height));
    painter.rect_filled(card, metrics.card_radius, metrics.surface);
    painter.rect_stroke(
        card,
        metrics.card_radius,
        egui::Stroke::new(stroke::hairline(), metrics.border),
        egui::StrokeKind::Inside,
    );
    let title_size = if viewport.is_tv() {
        metrics.nav_label_size + 4.0
    } else {
        metrics.nav_label_size + 3.0
    };
    let subtitle_size = metrics.screen_card_subtitle_size + 2.0;
    for (slot, page) in pages.into_iter().enumerate() {
        let node = NODE_SETTINGS_PAGE_BASE + page as u64;
        let row = Rect::from_min_size(
            top_left + Vec2::new(0.0, slot as f32 * height),
            Vec2::new(width, height),
        );
        if slot > 0 {
            painter.line_segment(
                [
                    Pos2::new(row.left() + 20.0, row.top()),
                    Pos2::new(row.right(), row.top()),
                ],
                egui::Stroke::new(stroke::hairline(), ink(tone::faint())),
            );
        }
        let title = section_label(SETTINGS_SECTIONS[page].title, language);
        let description = section_description(SETTINGS_SECTIONS[page].title, language);
        let mut clicked = false;
        components::placed(
            context,
            Id::new(("fluxa-settings-page-card", node)),
            row.min,
            |ui| {
                let (rect, response) = ui.allocate_exact_size(row.size(), Sense::click());
                if response.hovered() || response.is_pointer_button_down_on() {
                    ui.painter()
                        .rect_filled(rect, metrics.card_radius, ink(tone::faint()));
                }
                let text_width = (rect.width() - 72.0).max(1.0);
                let title_font = crate::fonts::regular(title_size);
                let subtitle_font = crate::fonts::regular(subtitle_size);
                let text_top = rect.center().y - (title_size + 4.0 + subtitle_size) * 0.5;
                draw_text(
                    ui.painter(),
                    Pos2::new(rect.left() + 20.0, text_top),
                    Align2::LEFT_TOP,
                    &title,
                    title_font,
                    text_primary(),
                    Some(text_width),
                );
                draw_text(
                    ui.painter(),
                    Pos2::new(rect.left() + 20.0, text_top + title_size + 4.0),
                    Align2::LEFT_TOP,
                    &description,
                    subtitle_font,
                    metrics.text_muted,
                    Some(text_width),
                );
                let tip = rect.right_center() - Vec2::new(22.0, 0.0);
                let stroke = egui::Stroke::new(stroke::icon(), ink(tone::muted()));
                ui.painter()
                    .line_segment([tip + Vec2::new(-5.0, -6.0), tip], stroke);
                ui.painter()
                    .line_segment([tip + Vec2::new(-5.0, 6.0), tip], stroke);
                clicked = response.clicked();
            },
        );
        layout.focusable.push((node, row));
        if clicked {
            layout.activated = Some(node);
        }
    }
}

fn section_key(title: &str) -> String {
    format!(
        "settings.section.{}",
        title.to_lowercase().replace(' ', "_")
    )
}

fn section_label(title: &str, language: &str) -> String {
    localized(&section_key(title), language)
}

fn section_description(title: &str, language: &str) -> String {
    localized(&format!("{}.description", section_key(title)), language)
}

fn page_top(rect: Rect, metrics: UiMetrics) -> f32 {
    rect.top()
        + if metrics.compact {
            8.0
        } else {
            APPEARANCE_PAGE_HEADER_HEIGHT
        }
}

fn field_group_height(fields: usize) -> f32 {
    APPEARANCE_GROUP_HEADING_HEIGHT + fields as f32 * FIELD_HEIGHT + APPEARANCE_GROUP_GAP
}

fn groups_lead(section: &SettingsSection, settings: &SettingsModel, metrics: UiMetrics) -> f32 {
    if !settings.search.trim().is_empty() {
        return 0.0;
    }
    match section.title {
        "TMDB" => field_group_height(1),
        "Account" => account_height(metrics) - APPEARANCE_PAGE_HEADER_HEIGHT,
        _ => 0.0,
    }
}

fn tmdb_ready(settings: &SettingsModel) -> bool {
    settings
        .poster_fields
        .get(1)
        .is_some_and(|key| !key.trim().is_empty())
}

fn row_enabled(key: &str, settings: &SettingsModel) -> bool {
    match key {
        "reuseLastStreamTtlHours" => settings.bool_value("reuseLastStream"),
        "homeSeasonPostersOnHero" => {
            settings
                .value("homeHeroArtwork")
                .and_then(serde_json::Value::as_str)
                == Some("poster")
        }
        _ => true,
    }
}

pub(super) fn visible_groups(
    settings: &SettingsModel,
    viewport: Viewport,
) -> Vec<(String, Vec<usize>)> {
    let language = settings.language();
    let query = settings.search.trim().to_lowercase();
    let mut offset = 0;
    let mut groups = Vec::new();
    if !query.is_empty() {
        for section in &SETTINGS_SECTIONS {
            let rows: Vec<usize> = section
                .rows
                .iter()
                .enumerate()
                .filter(|(_, row)| row_applies(row.key, viewport) && row_enabled(row.key, settings))
                .filter(|(_, row)| {
                    settings_row_label(row, language)
                        .to_lowercase()
                        .contains(&query)
                        || row.label.to_lowercase().contains(&query)
                })
                .map(|(index, _)| offset + index)
                .collect();
            if !rows.is_empty() {
                groups.push((section_label(section.title, language), rows));
            }
            offset += section.rows.len();
        }
        return groups;
    }
    let active = settings.active_section.min(SETTINGS_SECTIONS.len() - 1);
    if SETTINGS_SECTIONS[active].title == "TMDB" && !tmdb_ready(settings) {
        return groups;
    }
    offset = SETTINGS_SECTIONS[..active]
        .iter()
        .map(|section| section.rows.len())
        .sum();
    let rows = SETTINGS_SECTIONS[active].rows;
    let mut start = 0;
    for &(count, key) in SETTINGS_SECTIONS[active].groups {
        let end = start + count;
        let visible: Vec<usize> = (start..end)
            .filter(|&index| {
                row_applies(rows[index].key, viewport) && row_enabled(rows[index].key, settings)
            })
            .map(|index| offset + index)
            .collect();
        if !visible.is_empty() {
            groups.push((localized(key, language), visible));
        }
        start = end;
    }
    groups
}

pub(super) fn group_cards(
    rect: Rect,
    groups: &[(String, Vec<usize>)],
    language: &str,
    metrics: UiMetrics,
) -> Vec<Rect> {
    let mut top = rect.top() + APPEARANCE_PAGE_HEADER_HEIGHT;
    groups
        .iter()
        .map(|(_, rows)| {
            let card = Rect::from_min_size(
                Pos2::new(rect.left(), top + APPEARANCE_GROUP_HEADING_HEIGHT),
                Vec2::new(
                    rect.width(),
                    settings_group_card_height(rows.len(), metrics)
                        + rows_extra(rows, rect.width(), language, metrics),
                ),
            );
            top = card.bottom() + APPEARANCE_GROUP_GAP;
            card
        })
        .collect()
}

fn groups_height(
    groups: &[(String, Vec<usize>)],
    width: f32,
    language: &str,
    metrics: UiMetrics,
) -> f32 {
    APPEARANCE_PAGE_HEADER_HEIGHT
        + groups
            .iter()
            .map(|(_, rows)| {
                APPEARANCE_GROUP_HEADING_HEIGHT
                    + settings_group_card_height(rows.len(), metrics)
                    + rows_extra(rows, width, language, metrics)
                    + APPEARANCE_GROUP_GAP
            })
            .sum::<f32>()
}

pub fn accent_swatches() -> &'static [&'static str] {
    SETTINGS_SECTIONS
        .iter()
        .flat_map(|section| section.rows.iter())
        .find(|row| row.key == "accentColorArgb")
        .map_or(&[], |row| row.options)
}

pub fn settings_row_by_index(index: usize) -> Option<&'static SettingsRow> {
    SETTINGS_SECTIONS
        .iter()
        .flat_map(|section| section.rows.iter())
        .nth(index)
}

const CONTROLLER_BINDINGS: [(&str, &str); 6] = [
    ("settings.controller_navigate", "D-pad / left stick"),
    ("settings.controller_select", "Enter / A"),
    ("settings.controller_back", "Escape / Back / B"),
    ("settings.controller_scroll", "D-pad / left stick"),
    ("settings.controller_seek", "Left / Right"),
    ("settings.controller_volume", "Up / Down"),
];

fn content_width(viewport: Viewport, metrics: UiMetrics) -> f32 {
    let compact = viewport.is_compact();
    let margin = metrics.settings_margin;
    let left = if compact {
        margin
    } else {
        margin + metrics.settings_content_offset
    };
    (viewport.width - left - margin)
        .min(if compact || viewport.is_tv() {
            f32::MAX
        } else {
            metrics.settings_content_max_width_desktop
        })
        .max(1.0)
}

pub(super) fn settings_card_height(
    viewport: Viewport,
    metrics: UiMetrics,
    section: &SettingsSection,
    settings: &SettingsModel,
) -> f32 {
    let base = metrics.settings_card_height;
    let groups = visible_groups(settings, viewport);
    let rows_metrics = described(metrics);
    let width = content_width(viewport, metrics);
    let language = settings.language();
    if !settings.search.trim().is_empty() {
        return base.max(groups_height(&groups, width, language, rows_metrics));
    }
    base.max(content_height(viewport, metrics, section, settings) + links_height(metrics, settings))
}

fn content_height(
    viewport: Viewport,
    metrics: UiMetrics,
    section: &SettingsSection,
    settings: &SettingsModel,
) -> f32 {
    let groups = visible_groups(settings, viewport);
    let rows_metrics = described(metrics);
    let width = content_width(viewport, metrics);
    let language = settings.language();
    match section.title {
        "Account" | "TMDB" => {
            groups_height(&groups, width, language, rows_metrics)
                + groups_lead(section, settings, metrics)
        }
        "MDBList" => {
            APPEARANCE_PAGE_HEADER_HEIGHT
                + field_group_height(1)
                + APPEARANCE_GROUP_HEADING_HEIGHT
                + settings_group_card_height(1, metrics)
                + APPEARANCE_GROUP_GAP
        }
        "Skip Segments" => APPEARANCE_PAGE_HEADER_HEIGHT + field_group_height(3),
        "Watch Party" => {
            APPEARANCE_PAGE_HEADER_HEIGHT
                + components::ticket_height(watch_ticket_width(width))
                + 60.0
        }
        "Summary" => {
            APPEARANCE_PAGE_HEADER_HEIGHT + summary_height(&settings.playback_summary, metrics)
        }
        "About" => {
            groups_height(&groups, width, language, rows_metrics)
                + APPEARANCE_GROUP_HEADING_HEIGHT
                + settings_group_card_height(2, metrics)
                + APPEARANCE_GROUP_GAP
        }
        "Licenses" => {
            APPEARANCE_PAGE_HEADER_HEIGHT
                + APPEARANCE_GROUP_HEADING_HEIGHT
                + about::credits_card_height(
                    width,
                    viewport.is_compact(),
                    settings.language(),
                    metrics,
                )
                + APPEARANCE_GROUP_GAP
        }
        "Shortcuts" => {
            let player = settings.shortcuts.iter().filter(|row| row.player).count();
            let app = settings.shortcuts.len() - player;
            APPEARANCE_PAGE_HEADER_HEIGHT
                + (APPEARANCE_GROUP_HEADING_HEIGHT + APPEARANCE_GROUP_GAP) * 4.0
                + settings_group_card_height(app, metrics)
                + settings_group_card_height(player, metrics)
                + settings_group_card_height(1, metrics)
                + settings_group_card_height(CONTROLLER_BINDINGS.len(), metrics)
        }
        "Posters" => {
            groups_height(&groups, width, language, rows_metrics)
                + APPEARANCE_GROUP_HEADING_HEIGHT
                + FIELD_HEIGHT
                + APPEARANCE_GROUP_GAP
        }
        "Subtitles" => {
            groups_height(&groups, width, language, rows_metrics)
                + APPEARANCE_GROUP_HEADING_HEIGHT
                + SUBTITLE_PREVIEW_HEIGHT
                + APPEARANCE_GROUP_GAP
        }
        "Previews" => {
            groups_height(&groups, width, language, rows_metrics)
                + APPEARANCE_GROUP_HEADING_HEIGHT
                + settings_group_card_height(1, metrics)
                + APPEARANCE_GROUP_GAP
                + APPEARANCE_GROUP_HEADING_HEIGHT
                + FIELD_HEIGHT * 2.0
                + APPEARANCE_GROUP_GAP
        }
        "Badges" => {
            groups_height(&groups, width, language, rows_metrics)
                + badges::badges_height(settings, metrics)
        }
        "Collections" => {
            APPEARANCE_PAGE_HEADER_HEIGHT
                + collections::collections_height(settings, settings.language(), metrics.compact)
        }
        "Ambient Mode" => {
            groups_height(&groups, width, language, rows_metrics)
                + ambient::ambient_height(settings, language, metrics.compact)
        }
        "Home" => {
            groups_height(&groups, width, language, rows_metrics)
                + hero::hero_height(settings, language, metrics.compact)
        }
        "Details" => {
            groups_height(&groups, width, language, rows_metrics)
                + detail_sections::detail_sections_height(settings, language, metrics.compact)
        }
        "Add-ons" => {
            APPEARANCE_PAGE_HEADER_HEIGHT
                + addons::addons_height(settings, metrics, settings.language())
        }
        "Plugins" => {
            APPEARANCE_PAGE_HEADER_HEIGHT
                + addons::plugins_height(settings, metrics, settings.language())
        }
        "Servers" => {
            APPEARANCE_PAGE_HEADER_HEIGHT
                + addons::servers_height(settings, metrics, settings.language())
        }
        _ => groups_height(&groups, width, language, rows_metrics),
    }
}

/// Pure geometry used by input adapters. It intentionally has no egui state
/// or asset dependency, so Android touch/D-pad and desktop mouse hit testing

fn settings_panel_line(
    painter: &egui::Painter,
    rect: Rect,
    y: f32,
    text: impl AsRef<str>,
    color: Color32,
    metrics: UiMetrics,
) {
    let text = text.as_ref();
    let font = crate::fonts::regular(metrics.screen_body_size);
    let row_left = rect.left() + metrics.settings_extended_line_inset;
    let row_right = rect.right() - metrics.settings_extended_line_inset;
    let baseline_y = y + metrics.settings_extended_line_spacing * 0.5;
    if let Some((label, value)) = text.split_once("  ·  ") {
        draw_text(
            painter,
            Pos2::new(row_left + 2.0, baseline_y),
            Align2::LEFT_CENTER,
            label,
            font.clone(),
            color,
            Some((row_right - row_left) * 0.58),
        );
        let value_font = crate::fonts::regular(metrics.screen_card_subtitle_size);
        let value_width = painter
            .layout_no_wrap(value.to_owned(), value_font.clone(), Color32::WHITE)
            .size()
            .x
            .min((row_right - row_left) * 0.40);
        let chip = Rect::from_center_size(
            Pos2::new(row_right - value_width * 0.5 - 10.0, baseline_y),
            Vec2::new(value_width + 20.0, metrics.screen_control_height.min(30.0)),
        );
        painter.rect_filled(chip, metrics.screen_control_radius, raised_color());
        painter.rect_stroke(
            chip,
            metrics.screen_control_radius,
            egui::Stroke::new(stroke::hairline(), metrics.border),
            egui::StrokeKind::Inside,
        );
        draw_text(
            painter,
            chip.center(),
            Align2::CENTER_CENTER,
            value,
            value_font,
            metrics.text_primary,
            Some(chip.width() - 12.0),
        );
        painter.line_segment(
            [
                Pos2::new(row_left, y + metrics.settings_extended_line_spacing - 1.0),
                Pos2::new(row_right, y + metrics.settings_extended_line_spacing - 1.0),
            ],
            egui::Stroke::new(stroke::hairline(), ink(tone::faint())),
        );
    } else {
        draw_text(
            painter,
            Pos2::new(row_left + 2.0, baseline_y),
            Align2::LEFT_CENTER,
            text,
            font,
            color,
            Some(row_right - row_left - 4.0),
        );
    }
}

fn color_name(value: &str) -> Option<&'static str> {
    Some(match value {
        "glass" => "settings.liquid_glass",
        "#FFFFFF" => "auto.white",
        "#000000" => "auto.black",
        "#E50914" | "#FF5D5D" => "auto.red",
        "#3F7CFF" => "auto.blue",
        "#54D17A" => "auto.green",
        "#FF8A3D" => "auto.orange",
        "#C084FC" => "auto.purple",
        "#FFE45C" => "auto.yellow",
        _ => return None,
    })
}

pub fn subtitle_style_rows(language: &str) -> Vec<(&'static str, String, &'static [&'static str])> {
    SUBTITLES_SETTINGS[6..]
        .iter()
        .map(|row| (row.key, settings_row_label(row, language), row.options))
        .collect()
}

pub fn option_label(key: &str, value: &str, language: &str) -> String {
    let named = match (key, value) {
        (_, "off") => Some("settings.off"),
        ("animeUpscalingMode", "auto") => Some("settings.auto"),
        ("animeUpscalingQuality", _) => Some(match value {
            "anime4k_s" => "settings.anime4k_s",
            "anime4k_l" => "settings.anime4k_l",
            _ => "settings.anime4k_m",
        }),
        ("animeUpscalingModePreset", "b") => Some("player.anime4k_mode_b"),
        ("animeUpscalingModePreset", "c") => Some("player.anime4k_mode_c"),
        ("animeUpscalingModePreset", _) => Some("player.anime4k_mode_a"),
        ("frameInterpolationMode", "display_resample") => {
            Some("settings.frame_interpolation_display_resample")
        }
        ("trickplayMode", "pregenerate") => Some("settings.trickplay_mode_pregenerate"),
        ("trickplayMode", _) => Some("settings.trickplay_mode_async"),
        ("frameInterpolationMode", "smooth") => Some("settings.frame_interpolation_smooth"),
        ("audioProcessingMode", _) => Some(match value {
            "balanced" => "settings.audio_processing_balanced",
            "night" => "settings.audio_processing_night",
            _ => "settings.audio_processing_reference",
        }),
        ("dolbyVisionMode" | "dolbyVisionTonemap", "auto") => Some("settings.auto"),
        ("dolbyVisionMode", "native") => Some("settings.dolby_vision_native"),
        ("dolbyVisionMode", "convert") => Some("settings.dolby_vision_convert"),
        ("dolbyVisionMode", "base_layer") => Some("settings.dolby_vision_base_layer"),
        ("dolbyVisionMode", "compose") => Some("settings.dolby_vision_compose"),
        ("dolbyVisionTonemap", "no") => Some("settings.dolby_vision_tonemap_no"),
        ("dolbyVisionTonemap", "yes") => Some("settings.dolby_vision_tonemap_yes"),
        ("dolbyVisionTonemap", "sdr") => Some("settings.dolby_vision_tonemap_sdr"),
        ("displayPeakNits", "0") => Some("settings.auto"),
        ("playerBufferCacheMb", "-1") => Some("settings.buffer_cache_infinite"),
        ("subtitlePosition", _) => Some(match value {
            "90" => "settings.subtitle_position_low",
            "80" => "settings.subtitle_position_middle",
            "70" => "settings.subtitle_position_high",
            _ => "settings.subtitle_position_bottom",
        }),
        (
            "accentColorArgb"
            | "subtitleColor"
            | "subtitleBackgroundColor"
            | "subtitleOutlineColor",
            _,
        ) => color_name(value),
        _ => None,
    };
    if let Some(named) = named {
        return localized(named, language);
    }
    match key {
        "uiScale" | "subtitleSize" | "scrobbleWatchedPercent" => format!("{value}%"),
        "subtitleTextOpacity" | "subtitleBackgroundOpacity" | "subtitleOutlineOpacity" => {
            format!("{}%", (value.parse::<f64>().unwrap_or(1.0) * 100.0).round())
        }
        "expandedPostersDelay" | "trailerOnExpandedPostersDelay" if value == "0" => {
            localized("settings.expanded_posters_delay_instant", language)
        }
        "homeHeroAutoplayTrailerDelaySecs"
        | "detailHeroAutoplayTrailerDelaySecs"
        | "expandedPostersDelay"
        | "trailerOnExpandedPostersDelay"
        | "autoPlayCountdownSecs"
        | "playerForwardBufferSeconds"
        | "playerBackBufferSeconds"
        | "trickplayInterval" => format!("{value}s"),
        "reuseLastStreamTtlHours" => match value.parse::<u32>() {
            Ok(hours) if hours >= 24 => format!("{}d", hours / 24),
            _ => format!("{value}h"),
        },
        "holdSpeed" => format!("{value}x"),
        "ambientModeTimeoutMinutes" => localized("ambient.minutes", language).replace("%s", value),
        "playerBufferCacheMb" => format!("{value} MB"),
        _ => localized_or(&format!("settings.option.{value}"), value, language),
    }
}

pub fn settings_row_label(setting: &SettingsRow, language: &str) -> String {
    let legacy_key = match setting.key {
        "autoSkipIntro" => "settings.auto_skip",
        "useAnimeSkip" => "settings.use_animeskip",
        "seekThumbnailEnabled" => "settings.seek_thumbnails",
        "holdToSpeedEnabled" => "settings.hold_to_speed",
        "playbackSpeed" => "auto.playback_speed",
        "discordRichPresenceEnabled" => "settings.discord_rich_presence_enable",
        "animeUpscalingMode" => "settings.anime_upscaling",
        "animeUpscalingModePreset" => "player.anime4k",
        "frameInterpolationMode" => "settings.frame_interpolation",
        "autoPlayCountdownSecs" => "settings.auto_play_countdown",
        "playerBufferCacheMb" => "settings.buffer_cache",
        "playerForwardBufferSeconds" => "settings.forward_buffer",
        "playerBackBufferSeconds" => "settings.back_buffer",
        "subtitleColor" => "settings.subtitle_text",
        "subtitleBackgroundColor" => "settings.subtitle_background",
        "subtitleBackgroundOpacity" => "auto.background_transparency",
        "subtitleOutlineColor" => "settings.subtitle_outline",
        "subtitleOutlineOpacity" => "settings.subtitle.outline_opacity",
        "tmdbCollectionInfoEnabled" => "settings.tmdb_enrich_collection",
        "tmdbEnrichCastCrewEnabled" => "settings.tmdb_enrich_cast_crew",
        "tmdbEnrichNetworkEnabled" => "settings.tmdb_enrich_network",
        "tmdbEnrichOriginTitlesEnabled" => "settings.tmdb_enrich_origin_titles",
        "tmdbEnrichStatusScheduleEnabled" => "settings.tmdb_enrich_status_schedule",
        "tmdbEpisodeImagesEnabled" => "settings.tmdb_enrich_episode_stills",
        "tmdbEnrichArtworkEnabled" => "settings.tmdb_enrich_artwork",
        "tmdbLogosBackdropsEnabled" => "settings.tmdb_logos_backdrops",
        "tmdbCastImagesEnabled" => "settings.tmdb_cast_images",
        "tmdbEnrichDescriptionEnabled" => "settings.tmdb_enrich_description",
        "tmdbEnrichGenresKeywordsEnabled" => "settings.tmdb_enrich_genres_keywords",
        "tmdbEnrichWatchProvidersEnabled" => "settings.tmdb_enrich_watch_providers",
        "tmdbRatingsEnabled" => "settings.tmdb_ratings",
        "tmdbTrailersEnabled" => "settings.tmdb_trailers",
        "tmdbRecommendationsEnabled" => "settings.tmdb_recommendations",
        "tmdbSimilarResultsEnabled" => "settings.tmdb_similar_results",
        "traktCommentsEnabled" => "settings.trakt_comments",
        _ => "",
    };
    if !legacy_key.is_empty() {
        let translated = localized(legacy_key, language);
        if translated != legacy_key {
            return translated;
        }
    }

    let mut snake_case = String::with_capacity(setting.key.len() + 8);
    for character in setting.key.chars() {
        if character.is_ascii_uppercase() {
            snake_case.push('_');
            snake_case.push(character.to_ascii_lowercase());
        } else {
            snake_case.push(character);
        }
    }
    let legacy_key = format!("settings.{snake_case}");
    let translated = localized(&legacy_key, language);
    if translated != legacy_key {
        translated
    } else {
        // Some newer Core preferences have no legacy web translation yet.
        // Show the row's human-readable source label instead of leaking an i18n key.
        setting.label.to_owned()
    }
}

fn settings_panel_button(
    context: &egui::Context,
    layout: &mut HomeLayout,
    id: u64,
    rect: Rect,
    label: impl AsRef<str>,
    selected: bool,
    metrics: UiMetrics,
) {
    let kind = if selected {
        components::ButtonKind::Selected
    } else {
        components::ButtonKind::Secondary
    };
    components::placed_front(
        context,
        Id::new(("fluxa-settings-panel-action", id)),
        rect.min,
        |ui| {
            let response = components::button(
                ui,
                label.as_ref(),
                rect.width(),
                rect.height(),
                kind,
                metrics,
            );
            components::focus_action(layout, id, &response);
        },
    );
}

fn settings_panel_input(
    context: &egui::Context,
    layout: &mut HomeLayout,
    id: u64,
    rect: Rect,
    value: &str,
    hint: impl AsRef<str>,
    metrics: UiMetrics,
) {
    let mut next_value = value.to_owned();
    components::placed_front(
        context,
        Id::new(("fluxa-settings-panel-input", id)),
        rect.min,
        |ui| {
            let response = components::text_field(
                ui,
                &mut next_value,
                hint.as_ref(),
                rect.size(),
                metrics.settings_row_value_size_desktop,
                components::input_kind(id),
                metrics,
            );
            layout.focusable.push((id, response.rect));
            if response.changed() {
                layout.text_input = Some(next_value.clone());
                layout.text_input_node = Some(id);
            }
        },
    );
}

fn pill_button(
    context: &egui::Context,
    layout: &mut HomeLayout,
    id: u64,
    rect: Rect,
    label: &str,
    size: f32,
) {
    pill_button_styled(context, layout, id, rect, label, size, false);
}

fn pill_button_styled(
    context: &egui::Context,
    layout: &mut HomeLayout,
    id: u64,
    rect: Rect,
    label: &str,
    size: f32,
    primary: bool,
) {
    components::placed_front(
        context,
        Id::new(("fluxa-settings-pill", id)),
        rect.min,
        |ui| {
            let response = components::pill_button(
                ui,
                None,
                label,
                Some(rect.width()),
                rect.height(),
                size,
                primary,
                None,
            );
            components::focus_action(layout, id, &response);
        },
    );
}

pub struct PosterField {
    pub key: &'static str,
    pub label: &'static str,
    help: &'static str,
    hint: &'static str,
    pub input: u64,
}

pub const POSTER_FIELDS: [PosterField; 8] = [
    PosterField {
        key: "posterUrlTemplate",
        label: "settings.poster_url_template",
        help: "settings.poster_url_template_help",
        hint: "settings.poster_url_template_hint",
        input: NODE_SETTINGS_POSTER_URL,
    },
    PosterField {
        key: "tmdbApiKey",
        label: "settings.tmdb_api_key",
        help: "settings.poster_tmdb_help",
        hint: "settings.tmdb_api_key_placeholder",
        input: NODE_SETTINGS_POSTER_TMDB_KEY,
    },
    PosterField {
        key: "mdblistApiKey",
        label: "settings.mdblist_api_key",
        help: "settings.poster_mdblist_help",
        hint: "settings.mdblist_api_key_placeholder",
        input: NODE_SETTINGS_POSTER_MDBLIST_KEY,
    },
    PosterField {
        key: "introDbApiKey",
        label: "settings.introdb_api_key",
        help: "settings.introdb_api_key_help",
        hint: "settings.tmdb_api_key_placeholder",
        input: NODE_SETTINGS_INTRODB_KEY,
    },
    PosterField {
        key: "theIntroDbApiKey",
        label: "settings.theintrodb_api_key",
        help: "settings.theintrodb_api_key_help",
        hint: "settings.tmdb_api_key_placeholder",
        input: NODE_SETTINGS_THEINTRODB_KEY,
    },
    PosterField {
        key: "skipDbApiKey",
        label: "settings.skipdb_api_key",
        help: "settings.skipdb_api_key_help",
        hint: "settings.tmdb_api_key_placeholder",
        input: NODE_SETTINGS_SKIPDB_KEY,
    },
    PosterField {
        key: "trickplayRegex",
        label: "settings.trickplay_regex",
        help: "settings.trickplay_regex_help",
        hint: "settings.trickplay_regex_hint",
        input: NODE_SETTINGS_TRICKPLAY_REGEX,
    },
    PosterField {
        key: "trickplayServerUrl",
        label: "settings.trickplay_server_url",
        help: "settings.trickplay_server_url_help",
        hint: "settings.trickplay_server_url_hint",
        input: NODE_SETTINGS_TRICKPLAY_SERVER,
    },
];

pub fn poster_field(input: u64) -> Option<usize> {
    POSTER_FIELDS.iter().position(|field| field.input == input)
}

fn draw_field(
    context: &egui::Context,
    settings: &SettingsModel,
    index: usize,
    area: Rect,
    metrics: UiMetrics,
    compact: bool,
    layout: &mut HomeLayout,
) {
    let language = settings.language();
    let field = &POSTER_FIELDS[index];
    let painter = context.layer_painter(egui::LayerId::background());
    let label_size = if compact {
        metrics.nav_label_size + 2.0
    } else {
        metrics.settings_row_label_size_desktop
    };
    let inner = area.shrink2(Vec2::new(metrics.settings_row_inset + 4.0, 16.0));
    draw_text(
        &painter,
        inner.left_top(),
        Align2::LEFT_TOP,
        localized(field.label, language),
        crate::fonts::regular(label_size),
        metrics.text_primary,
        None,
    );
    let help_font = crate::fonts::regular(metrics.screen_card_subtitle_size + 2.0);
    draw_text(
        &painter,
        inner.left_top() + Vec2::new(0.0, label_size + 6.0),
        Align2::LEFT_TOP,
        truncate_to_width(
            &painter,
            &localized(field.help, language),
            &help_font,
            inner.width(),
        ),
        help_font,
        metrics.text_muted,
        None,
    );
    let height = metrics.screen_control_height;
    let input = Rect::from_min_max(
        Pos2::new(inner.left(), inner.bottom() - height),
        inner.right_bottom(),
    );
    settings_panel_input(
        context,
        layout,
        field.input,
        input,
        &settings.poster_fields[index],
        localized(field.hint, language),
        metrics,
    );
}

fn draw_subtitle_preview(
    context: &egui::Context,
    settings: &SettingsModel,
    rect: Rect,
    top: f32,
    language: &str,
    metrics: UiMetrics,
) {
    let painter = context.layer_painter(egui::LayerId::background());
    let card = account_group(
        &painter,
        rect,
        top,
        SUBTITLE_PREVIEW_HEIGHT,
        &localized("settings.group.subtitle_preview", language),
        metrics,
    );
    let screen = card.shrink(12.0);
    let radius = metrics.card_radius * 0.6;
    painter.rect_filled(screen, radius, Color32::from_rgb(28, 32, 40));
    paint_vertical_gradient(
        &painter,
        Rect::from_min_max(
            Pos2::new(screen.left(), screen.center().y),
            screen.right_bottom(),
        ),
        Color32::TRANSPARENT,
        Color32::from_rgb(8, 9, 12),
    );
    components::subtitle_sample(
        &painter,
        screen,
        &SubtitleLook::from_values(&settings.values),
        &localized("settings.subtitle_preview_text", language),
        screen.height() * 0.17,
    );
}

fn draw_field_group(
    context: &egui::Context,
    settings: &SettingsModel,
    rect: Rect,
    top: f32,
    title: &str,
    fields: &[usize],
    metrics: UiMetrics,
    compact: bool,
    layout: &mut HomeLayout,
) -> Rect {
    let painter = context.layer_painter(egui::LayerId::background());
    let card = account_group(
        &painter,
        rect,
        top,
        fields.len() as f32 * FIELD_HEIGHT,
        title,
        metrics,
    );
    for (slot, &index) in fields.iter().enumerate() {
        let area = Rect::from_min_size(
            card.left_top() + Vec2::new(0.0, slot as f32 * FIELD_HEIGHT),
            Vec2::new(card.width(), FIELD_HEIGHT),
        );
        if slot > 0 {
            painter.line_segment(
                [
                    Pos2::new(area.left() + metrics.settings_row_inset, area.top()),
                    Pos2::new(area.right() - metrics.settings_row_inset, area.top()),
                ],
                egui::Stroke::new(stroke::hairline(), ink(tone::faint())),
            );
        }
        draw_field(context, settings, index, area, metrics, compact, layout);
    }
    card
}

fn draw_shortcut_group(
    context: &egui::Context,
    painter: &egui::Painter,
    rect: Rect,
    top: f32,
    title: &str,
    rows: &[(usize, &ShortcutRow)],
    settings: &SettingsModel,
    language: &str,
    metrics: UiMetrics,
    layout: &mut HomeLayout,
) -> Rect {
    let card = account_group(
        painter,
        rect,
        top,
        settings_group_card_height(rows.len(), metrics),
        title,
        metrics,
    );
    let size = metrics.settings_row_label_size_desktop;
    for (position, (index, shortcut)) in rows.iter().enumerate() {
        let row = account_row(card, position, metrics);
        if position > 0 {
            account_divider(painter, row, metrics);
        }
        draw_text(
            painter,
            row.left_center() + Vec2::new(4.0, 0.0),
            Align2::LEFT_CENTER,
            localized(&format!("shortcuts.{}", shortcut.id), language),
            FontId::proportional(size),
            metrics.text_primary,
            None,
        );
        let recording = settings.shortcut_recording.as_deref() == Some(shortcut.id.as_str());
        let label = if recording {
            localized("settings.shortcuts_recording", language)
        } else if shortcut.label.is_empty() {
            localized("settings.shortcuts_unassigned", language)
        } else {
            shortcut.label.clone()
        };
        let width = painter
            .layout_no_wrap(label.clone(), FontId::proportional(size), Color32::WHITE)
            .size()
            .x
            .max(120.0)
            + 32.0;
        let height = 32.0_f32.min(row.height() - 4.0);
        let button = Rect::from_center_size(
            Pos2::new(row.right() - 4.0 - width * 0.5, row.center().y),
            Vec2::new(width, height),
        );
        pill_button(
            context,
            layout,
            NODE_SETTINGS_SHORTCUT_BASE + *index as u64,
            button,
            &label,
            size,
        );
        if shortcut.custom {
            let reset = localized("settings.shortcuts_reset_one", language);
            let reset_width = painter
                .layout_no_wrap(reset.clone(), FontId::proportional(size), Color32::WHITE)
                .size()
                .x
                + 32.0;
            let reset_button = Rect::from_center_size(
                Pos2::new(button.left() - 10.0 - reset_width * 0.5, row.center().y),
                Vec2::new(reset_width, height),
            );
            pill_button(
                context,
                layout,
                NODE_SETTINGS_SHORTCUT_RESET_BASE + *index as u64,
                reset_button,
                &reset,
                size,
            );
        }
    }
    card
}

fn draw_binding_group(
    painter: &egui::Painter,
    rect: Rect,
    top: f32,
    title: &str,
    rows: &[(&str, &str)],
    language: &str,
    metrics: UiMetrics,
) -> Rect {
    let card = account_group(
        painter,
        rect,
        top,
        settings_group_card_height(rows.len(), metrics),
        title,
        metrics,
    );
    let label_font = FontId::proportional(metrics.settings_row_label_size_desktop);
    let value_font = crate::fonts::regular(metrics.settings_row_value_size_desktop);
    for (index, (action, keys)) in rows.iter().enumerate() {
        let row = account_row(card, index, metrics);
        if index > 0 {
            account_divider(painter, row, metrics);
        }
        draw_text(
            painter,
            row.left_center() + Vec2::new(4.0, 0.0),
            Align2::LEFT_CENTER,
            localized(action, language),
            label_font.clone(),
            metrics.text_primary,
            None,
        );
        draw_text(
            painter,
            row.right_center() - Vec2::new(6.0, 0.0),
            Align2::RIGHT_CENTER,
            *keys,
            value_font.clone(),
            metrics.text_secondary,
            None,
        );
    }
    card
}

fn draw_about(
    context: &egui::Context,
    painter: &egui::Painter,
    settings: &SettingsModel,
    language: &str,
    rect: Rect,
    top: Option<f32>,
    compact: bool,
    metrics: UiMetrics,
    layout: &mut HomeLayout,
) {
    let update = &settings.update;
    let top = top.unwrap_or(
        rect.top()
            + if compact {
                8.0
            } else {
                APPEARANCE_PAGE_HEADER_HEIGHT
            },
    );
    let card = account_group(
        painter,
        rect,
        top,
        settings_group_card_height(2, metrics),
        &localized("settings.about.app", language),
        metrics,
    );
    let label_size = if compact {
        metrics.screen_body_size
    } else {
        metrics.settings_row_label_size_desktop
    };
    let label_font = FontId::proportional(label_size);
    let value_font = crate::fonts::regular(if compact {
        metrics.text.meta
    } else {
        metrics.settings_row_value_size_desktop
    });

    let version_row = account_row(card, 0, metrics);
    let version = localized("settings.updates.version", language);
    if compact {
        draw_text(
            painter,
            version_row.left_center() + Vec2::new(4.0, -1.0),
            Align2::LEFT_BOTTOM,
            version,
            label_font,
            metrics.text_primary,
            None,
        );
        draw_text(
            painter,
            version_row.left_center() + Vec2::new(4.0, 1.0),
            Align2::LEFT_TOP,
            &update.current,
            value_font.clone(),
            metrics.text_secondary,
            None,
        );
    } else {
        draw_text(
            painter,
            version_row.left_center() + Vec2::new(4.0, 0.0),
            Align2::LEFT_CENTER,
            version,
            label_font,
            metrics.text_primary,
            None,
        );
        draw_text(
            painter,
            version_row.right_center() - Vec2::new(6.0, 0.0),
            Align2::RIGHT_CENTER,
            &update.current,
            value_font.clone(),
            metrics.text_secondary,
            None,
        );
    }

    let status_row = account_row(card, 1, metrics);
    account_divider(painter, status_row, metrics);
    let (status, action) = match &update.state {
        _ if !update.supported => (localized("settings.updates.unsupported", language), None),
        UpdateState::Idle => (
            localized("settings.updates.idle", language),
            Some("settings.updates.check"),
        ),
        UpdateState::Checking => (localized("settings.updates.checking", language), None),
        UpdateState::UpToDate => (
            localized("settings.updates.up_to_date", language),
            Some("settings.updates.check"),
        ),
        UpdateState::Available(version) => (
            format!(
                "{} {version}",
                localized("settings.updates.available", language)
            ),
            Some("settings.updates.install"),
        ),
        UpdateState::Downloading(percent) => (
            format!(
                "{} {percent}%",
                localized("settings.updates.downloading", language)
            ),
            None,
        ),
        UpdateState::Ready(version) => (
            format!(
                "{} {version}",
                localized("settings.updates.ready", language)
            ),
            Some("settings.updates.restart"),
        ),
        UpdateState::Failed(reason) => (
            format!(
                "{}: {reason}",
                localized("settings.updates.failed", language)
            ),
            Some("settings.updates.check"),
        ),
    };
    let button_width = action
        .map(|key| {
            painter
                .layout_no_wrap(
                    localized(key, language),
                    FontId::proportional(label_size),
                    Color32::WHITE,
                )
                .size()
                .x
                + 32.0
        })
        .unwrap_or(0.0);
    draw_text(
        painter,
        status_row.left_center() + Vec2::new(4.0, 0.0),
        Align2::LEFT_CENTER,
        status,
        value_font,
        metrics.text_secondary,
        Some((status_row.width() - button_width - 24.0).max(40.0)),
    );
    if let Some(key) = action {
        let button = Rect::from_center_size(
            Pos2::new(
                status_row.right() - 4.0 - button_width * 0.5,
                status_row.center().y,
            ),
            Vec2::new(button_width, 36.0_f32.min(status_row.height() - 4.0)),
        );
        pill_button(
            context,
            layout,
            NODE_SETTINGS_UPDATE_ACTION,
            button,
            &localized(key, language),
            label_size,
        );
    }
}

const WATCH_TICKET_MAX: f32 = 560.0;

fn watch_ticket_width(rect_width: f32) -> f32 {
    rect_width.min(WATCH_TICKET_MAX)
}

fn draw_watch_party(
    context: &egui::Context,
    settings: &SettingsModel,
    language: &str,
    rect: Rect,
    compact: bool,
    metrics: UiMetrics,
    layout: &mut HomeLayout,
    assets: &mut impl HomeAssets,
) {
    let top = if compact {
        8.0 + rect.top()
    } else {
        rect.top() + APPEARANCE_PAGE_HEADER_HEIGHT
    };
    let width = watch_ticket_width(rect.width());
    let origin = Pos2::new(rect.left(), top + 6.0);
    let mut code = settings.watch_code.clone();
    let ticket = &settings.watch_ticket;
    components::placed_front(context, Id::new("fluxa-watch-ticket"), origin, |ui| {
        let output =
            components::watch_ticket(ui, ticket, &mut code, width, language, assets, metrics);
        if let Some(input) = output.code {
            layout
                .focusable
                .push((NODE_SETTINGS_WATCH_CODE, input.rect));
            layout
                .focusable
                .push((NODE_SETTINGS_WATCH_JOIN, input.button));
            if input.response.changed() {
                layout.text_input = Some(code.clone());
                layout.text_input_node = Some(NODE_SETTINGS_WATCH_CODE);
            }
        }
        layout.activated = match output.action {
            Some(components::TicketAction::Join) => Some(NODE_SETTINGS_WATCH_JOIN),
            None => layout.activated,
        };
    });
    let hint = Pos2::new(
        rect.left() + 4.0,
        origin.y + components::ticket_height(width) + 18.0,
    );
    draw_text(
        &context.layer_painter(egui::LayerId::background()),
        hint,
        Align2::LEFT_TOP,
        localized("settings.watch_party.host_help", language),
        crate::fonts::regular(metrics.screen_card_subtitle_size + 2.0),
        metrics.text_muted,
        Some(width),
    );
}

fn summary_height(sections: &[crate::StatsSection], metrics: UiMetrics) -> f32 {
    let groups = sections.len().max(1);
    let rows: f32 = if sections.is_empty() {
        settings_group_card_height(1, metrics)
    } else {
        sections
            .iter()
            .map(|section| settings_group_card_height(section.rows.len(), metrics))
            .sum()
    };
    rows + (APPEARANCE_GROUP_HEADING_HEIGHT + APPEARANCE_GROUP_GAP) * groups as f32
}

fn draw_summary(
    painter: &egui::Painter,
    settings: &SettingsModel,
    language: &str,
    rect: Rect,
    metrics: UiMetrics,
) {
    let mut top = rect.top() + APPEARANCE_PAGE_HEADER_HEIGHT;
    if settings.playback_summary.is_empty() {
        let card = account_group(
            painter,
            rect,
            top,
            settings_group_card_height(1, metrics),
            &localized("settings.summary_title", language),
            metrics,
        );
        draw_text(
            painter,
            account_row(card, 0, metrics).left_center() + Vec2::new(4.0, 0.0),
            Align2::LEFT_CENTER,
            localized("settings.summary_empty", language),
            crate::fonts::regular(metrics.settings_row_value_size_desktop),
            metrics.text_secondary,
            None,
        );
        return;
    }
    for section in &settings.playback_summary {
        let rows: Vec<(&str, &str)> = section
            .rows
            .iter()
            .map(|(label, value)| (label.as_str(), value.as_str()))
            .collect();
        let card = draw_binding_group(
            painter,
            rect,
            top,
            &localized(&section.title, language),
            &rows,
            language,
            metrics,
        );
        top = card.bottom() + APPEARANCE_GROUP_GAP;
    }
}

fn draw_settings_extended_section(
    context: &egui::Context,
    viewport: Viewport,
    settings: &SettingsModel,
    assets: &mut impl HomeAssets,
    section: &str,
    language: &str,
    rect: Rect,
    metrics: UiMetrics,
    layout: &mut HomeLayout,
) {
    let painter = context.layer_painter(egui::LayerId::background());
    match section {
        "Account" => draw_account(
            context, viewport, settings, assets, language, rect, metrics, layout,
        ),
        "TMDB" => {
            let ready = tmdb_ready(settings);
            let card = draw_field_group(
                context,
                settings,
                rect,
                if ready {
                    rect.top() + APPEARANCE_PAGE_HEADER_HEIGHT
                } else {
                    page_top(rect, metrics)
                },
                &localized("settings.group.api_key", language),
                &[1],
                metrics,
                viewport.is_compact(),
                layout,
            );
            if !ready {
                painter.galley(
                    Pos2::new(rect.left() + 16.0, card.bottom() + 12.0),
                    wrapped_regular(
                        &painter,
                        &localized("settings.tmdb_metadata_source_inactive", language),
                        metrics.screen_card_subtitle_size + 1.0,
                        ink(120),
                        rect.width() - 32.0,
                        2,
                    ),
                    ink(120),
                );
            }
        }
        "MDBList" => {
            let card = draw_field_group(
                context,
                settings,
                rect,
                page_top(rect, metrics),
                &localized("settings.group.api_key", language),
                &[2],
                metrics,
                viewport.is_compact(),
                layout,
            );
            account::draw_source_group(
                context,
                viewport,
                settings,
                rect,
                card.bottom() + APPEARANCE_GROUP_GAP,
                (account::MDBLIST_LIST_SOURCE, "settings.group.account_sync"),
                metrics,
                layout,
            );
        }
        "Skip Segments" => {
            draw_field_group(
                context,
                settings,
                rect,
                page_top(rect, metrics),
                &localized("settings.group.api_keys", language),
                &[3, 4, 5],
                metrics,
                viewport.is_compact(),
                layout,
            );
        }
        "Summary" => {
            draw_summary(&painter, settings, language, rect, metrics);
        }
        "Watch Party" => draw_watch_party(
            context,
            settings,
            language,
            rect,
            viewport.is_compact(),
            metrics,
            layout,
            assets,
        ),
        "Licenses" => about::draw_credits(
            &painter,
            assets,
            language,
            rect,
            page_top(rect, metrics),
            viewport.is_compact(),
            metrics,
        ),
        "About" => draw_about(
            context,
            &painter,
            settings,
            language,
            rect,
            group_cards(
                rect,
                &visible_groups(settings, viewport),
                language,
                described(metrics),
            )
            .last()
            .map(|card| card.bottom() + APPEARANCE_GROUP_GAP),
            viewport.is_compact(),
            metrics,
            layout,
        ),
        "Shortcuts" => {
            let mut top = rect.top() + APPEARANCE_PAGE_HEADER_HEIGHT;
            for (title, player) in [
                ("settings.shortcuts_group_general", false),
                ("settings.shortcuts_group_player", true),
            ] {
                let rows = settings
                    .shortcuts
                    .iter()
                    .enumerate()
                    .filter(|(_, row)| row.player == player)
                    .collect::<Vec<_>>();
                let card = draw_shortcut_group(
                    context,
                    &painter,
                    rect,
                    top,
                    &localized(title, language),
                    &rows,
                    settings,
                    language,
                    metrics,
                    layout,
                );
                top = card.bottom() + APPEARANCE_GROUP_GAP;
            }
            let card = account_group(
                &painter,
                rect,
                top,
                settings_group_card_height(1, metrics),
                &localized("settings.shortcuts_reset_all", language),
                metrics,
            );
            let row = account_row(card, 0, metrics);
            let label = localized("settings.shortcuts_reset_all", language);
            let size = metrics.settings_row_label_size_desktop;
            let width = painter
                .layout_no_wrap(label.clone(), FontId::proportional(size), Color32::WHITE)
                .size()
                .x
                + 32.0;
            let button = Rect::from_center_size(
                Pos2::new(row.right() - 4.0 - width * 0.5, row.center().y),
                Vec2::new(width, 32.0_f32.min(row.height() - 4.0)),
            );
            pill_button(
                context,
                layout,
                NODE_SETTINGS_SHORTCUT_RESET_ALL,
                button,
                &label,
                size,
            );
            draw_binding_group(
                &painter,
                rect,
                card.bottom() + APPEARANCE_GROUP_GAP,
                &localized("settings.group.controller", language),
                &CONTROLLER_BINDINGS,
                language,
                metrics,
            );
        }
        "Posters" => {
            let top = group_cards(
                rect,
                &visible_groups(settings, viewport),
                language,
                described(metrics),
            )
            .last()
            .map_or(rect.top() + APPEARANCE_PAGE_HEADER_HEIGHT, |card| {
                card.bottom() + APPEARANCE_GROUP_GAP
            });
            draw_field_group(
                context,
                settings,
                rect,
                top,
                &localized("settings.group.custom_poster", language),
                &[0],
                metrics,
                viewport.is_compact(),
                layout,
            );
        }
        "Subtitles" => {
            let top = group_cards(
                rect,
                &visible_groups(settings, viewport),
                language,
                described(metrics),
            )
            .last()
            .map_or(rect.top() + APPEARANCE_PAGE_HEADER_HEIGHT, |card| {
                card.bottom() + APPEARANCE_GROUP_GAP
            });
            draw_subtitle_preview(context, settings, rect, top, language, metrics);
        }
        "Previews" => {
            let top = group_cards(
                rect,
                &visible_groups(settings, viewport),
                language,
                described(metrics),
            )
            .last()
            .map_or(rect.top() + APPEARANCE_PAGE_HEADER_HEIGHT, |card| {
                card.bottom() + APPEARANCE_GROUP_GAP
            });
            let top = account::draw_source_group(
                context,
                viewport,
                settings,
                rect,
                top,
                (3, "settings.group.seek_preview_source"),
                metrics,
                layout,
            ) + APPEARANCE_GROUP_GAP;
            draw_field_group(
                context,
                settings,
                rect,
                top,
                &localized("settings.group.seek_preview_filter", language),
                &[6, 7],
                metrics,
                viewport.is_compact(),
                layout,
            );
        }
        "Badges" => {
            let top = group_cards(
                rect,
                &visible_groups(settings, viewport),
                language,
                described(metrics),
            )
            .last()
            .map_or(rect.top() + APPEARANCE_PAGE_HEADER_HEIGHT, |card| {
                card.bottom() + APPEARANCE_GROUP_GAP
            });
            badges::draw_badges(
                context, settings, assets, language, rect, top, metrics, layout,
            );
        }
        "Collections" => collections::draw_collections(
            context,
            settings,
            language,
            rect,
            rect.top() + metrics.settings_subpage_header_offset,
            metrics,
            layout,
        ),
        "Home" => {
            let top = group_cards(
                rect,
                &visible_groups(settings, viewport),
                language,
                described(metrics),
            )
            .last()
            .map_or(rect.top() + APPEARANCE_PAGE_HEADER_HEIGHT, |card| {
                card.bottom() + APPEARANCE_GROUP_GAP
            });
            hero::draw_hero_settings(context, settings, language, rect, top, metrics, layout);
        }
        "Ambient Mode" => {
            let top = group_cards(
                rect,
                &visible_groups(settings, viewport),
                language,
                described(metrics),
            )
            .last()
            .map_or(rect.top() + APPEARANCE_PAGE_HEADER_HEIGHT, |card| {
                card.bottom() + APPEARANCE_GROUP_GAP
            });
            ambient::draw_ambient_settings(context, settings, language, rect, top, metrics, layout);
        }
        "Details" => {
            let top = group_cards(
                rect,
                &visible_groups(settings, viewport),
                language,
                described(metrics),
            )
            .last()
            .map_or(rect.top() + APPEARANCE_PAGE_HEADER_HEIGHT, |card| {
                card.bottom() + APPEARANCE_GROUP_GAP
            });
            detail_sections::draw_detail_sections(
                context, settings, language, rect, top, metrics, layout,
            );
        }
        "Add-ons" => addons::draw_addons(
            context,
            settings,
            assets,
            language,
            rect,
            rect.top() + metrics.settings_subpage_header_offset,
            metrics,
            layout,
        ),
        "Plugins" => addons::draw_plugins(
            context,
            settings,
            assets,
            language,
            rect,
            rect.top() + metrics.settings_subpage_header_offset,
            metrics,
            layout,
        ),
        "Servers" => addons::draw_servers(
            context,
            settings,
            assets,
            language,
            rect,
            rect.top() + metrics.settings_subpage_header_offset,
            metrics,
            layout,
        ),
        _ => {}
    }
}

pub const COMPACT_SETTINGS_ROW: f32 = 72.0;
const COMPACT_ICON_TILE: f32 = 40.0;
const COMPACT_TEXT_LEFT: f32 = 72.0;

pub fn compact_settings_content_top(metrics: UiMetrics, settings: &SettingsModel) -> f32 {
    let top = metrics.detail_header_top_mobile;
    if settings.section_open && settings.search.trim().is_empty() {
        top + 40.0 + metrics.section_gap
    } else {
        top + metrics.screen_title_size
            + metrics.section_gap
            + metrics.screen_control_height
            + metrics.section_gap
    }
}

pub fn compact_settings_list_height(viewport: Viewport, metrics: UiMetrics) -> f32 {
    (categories(viewport).count() + 1) as f32 * COMPACT_SETTINGS_ROW + metrics.section_gap
}

fn draw_compact_settings_list(
    context: &egui::Context,
    viewport: Viewport,
    painter: &egui::Painter,
    assets: &impl HomeAssets,
    language: &str,
    metrics: UiMetrics,
    anchor: Rect,
    layout: &mut HomeLayout,
) {
    let entries: Vec<(u64, &str, String, String)> = categories(viewport)
        .map(|index| {
            let section = &SETTINGS_SECTIONS[index];
            (
                NODE_SETTINGS_SECTION_BASE + index as u64,
                section.category,
                section_label(section.category, language),
                section_description(section.category, language),
            )
        })
        .chain(std::iter::once((
            NODE_SETTINGS_SWITCH_PROFILE,
            "Profile",
            localized("settings.switch_profiles", language),
            localized("settings.switch_profiles_desc", language),
        )))
        .collect();
    let sections = categories(viewport).count();
    let group = |top: f32, count: usize| {
        let card = Rect::from_min_size(
            Pos2::new(anchor.left(), top),
            Vec2::new(anchor.width(), count as f32 * COMPACT_SETTINGS_ROW),
        );
        painter.rect_filled(card, metrics.card_radius, metrics.surface);
        painter.rect_stroke(
            card,
            metrics.card_radius,
            egui::Stroke::new(stroke::hairline(), metrics.border),
            egui::StrokeKind::Inside,
        );
    };
    group(anchor.top(), sections);
    let profile_top = anchor.top() + sections as f32 * COMPACT_SETTINGS_ROW + metrics.section_gap;
    group(profile_top, 1);
    for (index, (node, icon, label, description)) in entries.into_iter().enumerate() {
        let top = if index < sections {
            anchor.top() + index as f32 * COMPACT_SETTINGS_ROW
        } else {
            profile_top
        };
        let rect = Rect::from_min_size(
            Pos2::new(anchor.left(), top),
            Vec2::new(anchor.width(), COMPACT_SETTINGS_ROW),
        );
        if index > 0 && index < sections {
            painter.line_segment(
                [
                    Pos2::new(rect.left() + COMPACT_TEXT_LEFT, rect.top()),
                    Pos2::new(rect.right() - 16.0, rect.top()),
                ],
                egui::Stroke::new(stroke::hairline(), ink(tone::faint())),
            );
        }
        let mut clicked = false;
        components::placed(
            context,
            Id::new(("fluxa-settings-compact-row", node)),
            rect.min,
            |ui| {
                let (row, response) = ui.allocate_exact_size(rect.size(), Sense::click());
                if response.is_pointer_button_down_on() {
                    ui.painter()
                        .rect_filled(row, metrics.card_radius, ink(tone::faint()));
                }
                let tile = Rect::from_center_size(
                    row.left_center() + Vec2::new(16.0 + COMPACT_ICON_TILE * 0.5, 0.0),
                    Vec2::splat(COMPACT_ICON_TILE),
                );
                ui.painter()
                    .rect_filled(tile, radius::control(), metrics.surface_raised);
                settings_category_icon(assets, ui.painter(), tile.center(), icon, text_primary());
                let text_width = row.width() - COMPACT_TEXT_LEFT - 44.0;
                let title_size = metrics.nav_label_size + 3.0;
                let detail_size = metrics.screen_card_subtitle_size + 2.0;
                let text_top = row.center().y - (title_size + 5.0 + detail_size) * 0.5;
                draw_text(
                    ui.painter(),
                    Pos2::new(row.left() + COMPACT_TEXT_LEFT, text_top),
                    Align2::LEFT_TOP,
                    label,
                    crate::fonts::medium(title_size),
                    text_primary(),
                    Some(text_width),
                );
                draw_text(
                    ui.painter(),
                    Pos2::new(row.left() + COMPACT_TEXT_LEFT, text_top + title_size + 5.0),
                    Align2::LEFT_TOP,
                    description,
                    crate::fonts::regular(detail_size),
                    metrics.text_muted,
                    Some(text_width),
                );
                let tip = row.right_center() - Vec2::new(20.0, 0.0);
                let stroke = egui::Stroke::new(stroke::icon(), ink(tone::muted()));
                ui.painter()
                    .line_segment([tip + Vec2::new(-5.0, -6.0), tip], stroke);
                ui.painter()
                    .line_segment([tip + Vec2::new(-5.0, 6.0), tip], stroke);
                clicked = response.clicked();
            },
        );
        layout.focusable.push((node, rect));
        if clicked {
            layout.activated = Some(node);
        }
    }
}

pub fn draw_settings(
    context: &egui::Context,
    viewport: Viewport,
    settings: &SettingsModel,
    assets: &mut impl HomeAssets,
    focused: Option<u64>,
) -> HomeLayout {
    let layout = draw_settings_page(context, viewport, settings, assets, focused);
    context.move_to_top(egui::LayerId::new(
        egui::Order::Foreground,
        Id::new("fluxa-shared-top-bar"),
    ));
    layout
}

fn draw_settings_page(
    context: &egui::Context,
    viewport: Viewport,
    settings: &SettingsModel,
    assets: &mut impl HomeAssets,
    focused: Option<u64>,
) -> HomeLayout {
    let language = settings
        .values
        .get("language")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("en");
    let mut metrics = UiMetrics::for_viewport(viewport);
    metrics.accent = settings
        .value("accentColorArgb")
        .and_then(serde_json::Value::as_str)
        .and_then(parse_settings_color)
        .unwrap_or(text_primary());
    metrics.accent_foreground = if accent_needs_dark_foreground(metrics.accent) {
        background()
    } else {
        text_primary()
    };
    let scroll_y = resolve_screen_scroll(
        context,
        viewport,
        Id::new("fluxa-screen-scroll-settings"),
        settings_scroll_max(viewport, settings),
    );
    let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(viewport.width, viewport.height));
    let painter = context.layer_painter(egui::LayerId::background());
    painter.rect_filled(screen, 0.0, background());
    let mut layout = HomeLayout::default();
    layout.activated = draw_navigation_bar(context, viewport, 4, assets);
    let compact = viewport.is_compact();
    let tv = viewport.is_tv();
    let desktop = !compact && !tv;
    let margin = metrics.settings_margin;
    let top = metrics.settings_top;
    let mut header_bottom = top;
    if tv {
        components::placed(
            context,
            Id::new("fluxa-shared-settings-header"),
            Pos2::new(margin, top - scroll_y),
            |ui| {
                ui.label(
                    RichText::new(localized("nav.settings", language))
                        .size(metrics.screen_title_size)
                        .strong()
                        .color(text_primary()),
                );
                ui.add_space(metrics.control_gap);
                ui.label(
                    RichText::new(localized("settings.description", language))
                        .size(metrics.screen_body_size)
                        .color(metrics.text_secondary),
                );
                ui.add_space(metrics.section_gap);
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = metrics.control_gap;
                    let back = components::button_auto_width(
                        ui,
                        &format!("‹  {}", localized("common.back", language)),
                        components::ButtonKind::Secondary,
                        metrics.nav_label_size + 2.0,
                        metrics,
                    );
                    components::focus_action(&mut layout, NODE_SETTINGS_BACK, &back);
                    let switch = components::button_auto_width(
                        ui,
                        &localized("settings.switch_profiles", language),
                        components::ButtonKind::Secondary,
                        metrics.nav_label_size + 2.0,
                        metrics,
                    );
                    layout
                        .focusable
                        .push((NODE_SETTINGS_SWITCH_PROFILE, switch.rect));
                    if switch.clicked() {
                        layout.activated = Some(NODE_SETTINGS_SWITCH_PROFILE);
                    }
                });
                header_bottom = ui.min_rect().bottom() + scroll_y;
            },
        );
    }
    let active_section = settings.active_section.min(SETTINGS_SECTIONS.len() - 1);
    let section_top = if desktop {
        top
    } else if compact {
        compact_settings_content_top(metrics, settings)
    } else {
        header_bottom + metrics.section_gap
    };
    let content_x = if compact {
        margin
    } else {
        margin + metrics.settings_content_offset
    };
    let content_width = content_width(viewport, metrics);
    let searching = !settings.search.trim().is_empty();
    let search_height = metrics.screen_control_height;
    let search_field = |layout: &mut HomeLayout, rect: Rect| {
        components::placed_front(context, Id::new("fluxa-settings-search"), rect.min, |ui| {
            let mut query = settings.search.clone();
            let response = components::search_field(
                ui,
                &mut query,
                &localized("settings.search_placeholder", language),
                rect.width(),
                rect.height(),
                metrics,
            );
            layout.focusable.push((NODE_SETTINGS_SEARCH, response.rect));
            if response.changed() {
                layout.text_input = Some(query);
                layout.text_input_node = Some(NODE_SETTINGS_SEARCH);
            }
        });
    };
    if compact {
        let width = (viewport.width - margin * 2.0).max(1.0);
        let root = !settings.section_open || searching;
        if root {
            draw_text(
                &painter,
                Pos2::new(margin, top - scroll_y),
                Align2::LEFT_TOP,
                localized("nav.settings", language),
                crate::fonts::bold(metrics.screen_title_size),
                text_primary(),
                None,
            );
            search_field(
                &mut layout,
                Rect::from_min_size(
                    Pos2::new(
                        margin,
                        top + metrics.screen_title_size + metrics.section_gap - scroll_y,
                    ),
                    Vec2::new(width, search_height),
                ),
            );
        } else {
            let back_rect = Rect::from_min_size(
                Pos2::new(margin - 8.0, top - scroll_y),
                Vec2::splat(metrics.icon_button_size),
            );
            let page_title = if settings.page_open {
                section_label(SETTINGS_SECTIONS[active_section].title, language)
            } else {
                section_label(SETTINGS_SECTIONS[active_section].category, language)
            };
            let sticky_on = scroll_y > 60.0;
            if !sticky_on {
                components::placed_front(
                    context,
                    Id::new("fluxa-settings-back"),
                    back_rect.min,
                    |ui| {
                        let response =
                            components::back_button(ui, assets.icon("ArrowLeft"), metrics);
                        if response.clicked() {
                            layout.activated = Some(NODE_SETTINGS_BACK);
                        }
                    },
                );
                layout.focusable.push((NODE_SETTINGS_BACK, back_rect));
            }
            if let Some(sticky) = components::sticky_bar(
                context,
                "fluxa-settings",
                viewport,
                margin - 8.0,
                sticky_on,
                components::StickyBar {
                    back_node: NODE_SETTINGS_BACK,
                    back_icon: assets.icon("ArrowLeft"),
                    logo: None,
                    title: &page_title,
                    actions: Vec::new(),
                },
                metrics,
            ) {
                layout.focusable.extend(sticky.rects);
                layout.activated = sticky.clicked.or(layout.activated);
            }
            draw_text(
                &painter,
                back_rect.right_center() + Vec2::new(4.0, 0.0),
                Align2::LEFT_CENTER,
                page_title,
                crate::fonts::bold(metrics.text.title + 2.0),
                text_primary(),
                None,
            );
        }
        if !settings.section_open && !searching {
            draw_compact_settings_list(
                context,
                viewport,
                &painter,
                assets,
                language,
                metrics,
                Rect::from_min_size(
                    Pos2::new(margin, section_top - scroll_y),
                    Vec2::new(width, 0.0),
                ),
                &mut layout,
            );
            return layout;
        }
    } else {
        let nav_width = metrics
            .settings_nav_width
            .min((viewport.width - margin * 2.0).max(1.0) * 0.32);
        let nav_item_height = if tv {
            metrics.screen_control_height
        } else {
            metrics.settings_nav_item_height_desktop
        };
        let nav_item_gap = if tv {
            metrics.control_gap * 0.75
        } else {
            metrics.settings_nav_item_gap_desktop
        };
        let nav_scroll = if desktop { 0.0 } else { scroll_y };
        let nav_x = if desktop { 0.0 } else { margin };
        let nav_inset = if desktop { 16.0 } else { metrics.control_gap };
        let panel_width = if desktop {
            nav_width + margin
        } else {
            nav_width
        };
        let nav_panel = if desktop {
            Rect::from_min_size(Pos2::ZERO, Vec2::new(panel_width, viewport.height))
        } else {
            Rect::from_min_size(
                Pos2::new(margin, section_top - nav_scroll),
                Vec2::new(
                    nav_width,
                    (viewport.height - section_top + nav_scroll - margin).max(nav_item_height),
                ),
            )
        };
        if desktop {
            painter.rect_filled(nav_panel, 0.0, surface_color());
            painter.line_segment(
                [nav_panel.right_top(), nav_panel.right_bottom()],
                egui::Stroke::new(stroke::hairline(), metrics.border),
            );
        } else {
            painter.rect_filled(nav_panel, metrics.card_radius, metrics.surface);
            painter.rect_stroke(
                nav_panel,
                metrics.card_radius,
                egui::Stroke::new(stroke::hairline(), metrics.border),
                egui::StrokeKind::Inside,
            );
        }
        if desktop {
            draw_text(
                &painter,
                Pos2::new(nav_x + nav_inset + 2.0, section_top - nav_scroll + 18.0),
                Align2::LEFT_TOP,
                localized("nav.settings", language),
                crate::fonts::bold(metrics.settings_section_title_size_desktop + 2.0),
                text_primary(),
                None,
            );
        }
        let search_top = if desktop { 58.0 } else { metrics.control_gap };
        search_field(
            &mut layout,
            Rect::from_min_size(
                Pos2::new(nav_x + nav_inset, section_top + search_top - nav_scroll),
                Vec2::new(panel_width - nav_inset * 2.0, search_height),
            ),
        );
        let nav_top_inset = search_top + search_height + metrics.control_gap * 1.5;
        let active_category = SETTINGS_SECTIONS[active_section].category;
        for (slot, index) in categories(viewport).enumerate() {
            let section = &SETTINGS_SECTIONS[index];
            let node = NODE_SETTINGS_SECTION_BASE + index as u64;
            let rect = Rect::from_min_size(
                Pos2::new(
                    nav_x + nav_inset,
                    section_top + nav_top_inset + slot as f32 * (nav_item_height + nav_item_gap)
                        - nav_scroll,
                ),
                Vec2::new(panel_width - nav_inset * 2.0, nav_item_height),
            );
            layout.focusable.push((node, rect));
            let mut clicked = false;
            components::placed(
                context,
                Id::new(("fluxa-settings-category", node)),
                rect.min,
                |ui| {
                    let (item_rect, response) = ui.allocate_exact_size(rect.size(), Sense::click());
                    let active = section.category == active_category && !searching;
                    if active {
                        ui.painter().rect_filled(
                            item_rect,
                            metrics.screen_control_radius,
                            ink(tone::faint()),
                        );
                        ui.painter().rect_filled(
                            Rect::from_min_max(
                                item_rect.left_top() + Vec2::new(1.0, 9.0),
                                Pos2::new(item_rect.left() + 4.0, item_rect.bottom() - 9.0),
                            ),
                            2.0,
                            metrics.accent,
                        );
                    }
                    settings_category_icon(
                        assets,
                        ui.painter(),
                        item_rect.left_center() + Vec2::new(22.0, 0.0),
                        section.category,
                        if active {
                            text_primary()
                        } else {
                            ink(tone::subtle())
                        },
                    );
                    draw_text(
                        ui.painter(),
                        item_rect.left_center() + Vec2::new(42.0, 0.0),
                        Align2::LEFT_CENTER,
                        section_label(section.category, language),
                        FontId::proportional(metrics.nav_label_size + if tv { 0.0 } else { 2.0 }),
                        if active {
                            text_primary()
                        } else {
                            ink(tone::subtle())
                        },
                        None,
                    );
                    clicked = response.clicked();
                },
            );
            if clicked {
                layout.activated = Some(node);
            }
        }
    }
    let nav_bottom = section_top;
    let card_top = nav_bottom - scroll_y;
    let card_width = content_width;
    let section = &SETTINGS_SECTIONS[active_section];
    let card_height = settings_card_height(viewport, metrics, section, settings);
    let rect = Rect::from_min_size(
        Pos2::new(content_x, card_top),
        Vec2::new(card_width, card_height),
    );
    let groups = visible_groups(settings, viewport);
    let results = searching && !groups.is_empty();
    let rect = if results {
        rect.translate(Vec2::new(0.0, -APPEARANCE_PAGE_HEADER_HEIGHT))
    } else if compact && !groups.is_empty() {
        let lead = if groups[0].0 == section_label(section.title, language) {
            APPEARANCE_GROUP_HEADING_HEIGHT
        } else {
            0.0
        };
        rect.translate(Vec2::new(0.0, 8.0 - APPEARANCE_PAGE_HEADER_HEIGHT - lead))
    } else {
        rect
    };
    let header = rect;
    let cards = group_cards(
        rect.translate(Vec2::new(0.0, groups_lead(section, settings, metrics))),
        &groups,
        language,
        described(metrics),
    );
    if shows_hub(settings) {
        let top = rect.top() + content_height(viewport, metrics, section, settings)
            - if groups.is_empty() {
                rect.top() + APPEARANCE_PAGE_HEADER_HEIGHT - page_top(rect, metrics)
            } else {
                0.0
            };
        draw_page_cards(
            context,
            viewport,
            &painter,
            active_section,
            language,
            metrics,
            Pos2::new(rect.left(), top),
            rect.width(),
            &mut layout,
        );
    }
    let page_title = section_label(section.title, language);
    for ((title, _), card) in groups.iter().zip(&cards) {
        if !(compact && !searching && *title == page_title) {
            group_heading(
                &painter,
                rect,
                card.top() - APPEARANCE_GROUP_HEADING_HEIGHT,
                title,
                metrics,
            );
        }
        painter.rect_filled(*card, metrics.card_radius, metrics.surface);
        painter.rect_stroke(
            *card,
            metrics.card_radius,
            egui::Stroke::new(stroke::hairline(), metrics.border),
            egui::StrokeKind::Inside,
        );
    }
    let heading = if compact && !searching {
        String::new()
    } else if !searching && settings.page_open && section.category != section.title {
        format!(
            "{}  ›  {}",
            section_label(section.category, language),
            section_label(section.title, language)
        )
    } else if !searching {
        section_label(section.category, language)
    } else if groups.is_empty() {
        localized("settings.search_no_results", language)
    } else {
        String::new()
    };
    draw_text(
        &painter,
        header.left_top() + Vec2::new(metrics.settings_card_padding, metrics.settings_title_top),
        Align2::LEFT_TOP,
        heading,
        FontId::proportional(if tv {
            metrics.screen_section_title_size_tv - metrics.control_gap
        } else if !compact {
            metrics.settings_section_title_size_desktop
        } else {
            metrics.screen_section_title_size
        }),
        text_primary(),
        None,
    );
    if tv && !searching {
        draw_text(
            &painter,
            header.left_top()
                + Vec2::new(
                    metrics.settings_card_padding,
                    metrics.settings_description_top,
                ),
            Align2::LEFT_TOP,
            section_description(section.title, language),
            crate::fonts::regular(metrics.screen_card_subtitle_size),
            metrics.text_muted,
            None,
        );
    }
    if !searching {
        draw_settings_extended_section(
            context,
            viewport,
            settings,
            assets,
            section.title,
            language,
            rect,
            metrics,
            &mut layout,
        );
    }
    let row_metrics = described(metrics);
    let rows = groups.iter().zip(&cards).flat_map(|((_, rows), card)| {
        let mut offset = 0.0;
        rows.iter().enumerate().map(move |(slot, &index)| {
            let extra = row_extra(index, card.width(), language, row_metrics);
            let top = offset;
            offset += extra;
            (index, slot, *card, top, extra)
        })
    });
    for (index, slot, card, offset, extra) in rows {
        let Some(setting) = settings_row_by_index(index) else {
            continue;
        };
        let node = NODE_SETTINGS_ROW_BASE + index as u64;
        let is_swatches = setting.key == "accentColorArgb";
        let row_rect = Rect::from_min_size(
            card.left_top()
                + Vec2::new(
                    metrics.settings_row_inset,
                    6.0 + slot as f32 * row_metrics.settings_row_spacing + offset,
                ),
            Vec2::new(
                card.width() - metrics.settings_row_inset * 2.0,
                row_metrics.settings_row_height + extra,
            ),
        );
        let wrapped = extra > 0.0 && !is_swatches;
        let text_center_y = if wrapped {
            row_rect.center().y
        } else {
            row_rect.top() + row_metrics.settings_row_height * 0.5
        };
        if slot > 0 {
            let y = row_rect.top()
                - (row_metrics.settings_row_spacing - row_metrics.settings_row_height) * 0.5;
            painter.line_segment(
                [
                    Pos2::new(row_rect.left(), y),
                    Pos2::new(row_rect.right(), y),
                ],
                egui::Stroke::new(stroke::hairline(), ink(tone::faint())),
            );
        }
        let is_toggle = setting.options.is_empty();
        let is_app_icon = setting.key == "appIcon";
        let sheet_row = (compact || tv || is_app_icon) && !is_toggle && !is_swatches;
        let value_below =
            sheet_row && !is_app_icon && compact && viewport.platform == UiPlatform::Android;
        let mut clicked = false;
        if sheet_row {
            layout.focusable.push((node, row_rect));
        }
        if is_toggle {
            layout.focusable.push((node, row_rect));
            components::placed_front(
                context,
                Id::new(("fluxa-settings-row", node)),
                row_rect.min,
                |ui| {
                    let (_, response) = ui.allocate_exact_size(row_rect.size(), Sense::click());
                    clicked = response.clicked();
                },
            );
        }
        let row_label_size = if !compact && !tv {
            metrics.settings_row_label_size_desktop
        } else {
            metrics.nav_label_size + 2.0
        };
        let subtitle_size = metrics.screen_card_subtitle_size + 2.0;
        let value_size = if !compact && !tv {
            metrics.settings_row_value_size_desktop
        } else {
            metrics.screen_card_subtitle_size + 1.0
        };
        let raw_value = (!is_toggle).then(|| {
            settings
                .value(setting.key)
                .and_then(serde_json::Value::as_str)
                .unwrap_or(setting.options[0])
        });
        let value = raw_value.map(|raw| option_label(setting.key, raw, language));
        let value_font = crate::fonts::regular(value_size);
        let value_width = if value_below || is_app_icon {
            0.0
        } else if sheet_row {
            value.as_deref().map_or(0.0, |value| {
                painter
                    .layout_no_wrap(value.to_owned(), value_font.clone(), Color32::WHITE)
                    .size()
                    .x
                    .min(row_rect.width() * 0.4)
            }) + 22.0
        } else {
            setting
                .options
                .iter()
                .map(|option| {
                    painter
                        .layout_no_wrap(
                            option_label(setting.key, option, language),
                            value_font.clone(),
                            text_primary(),
                        )
                        .size()
                        .x
                })
                .fold(0.0, f32::max)
                + 50.0
        }
        .clamp(0.0, row_rect.width() * 0.5)
        .max(if sheet_row { 0.0 } else { 92.0 });
        let value_width = if value_below || is_app_icon {
            0.0
        } else {
            value_width
        };
        let text_left = row_rect.left() + 4.0;
        let control_width = if is_swatches {
            0.0
        } else if is_app_icon {
            (row_rect.height() - 16.0).min(44.0) + 4.0
        } else if is_toggle {
            metrics.settings_toggle_width + 3.0
        } else {
            value_width + 2.0
        };
        let text_width = (row_rect.right() - control_width - 12.0 - text_left).max(1.0);
        let label_font = crate::fonts::regular(row_label_size);
        let subtitle_font = crate::fonts::regular(subtitle_size);
        let text_height =
            row_label_size + 4.0 + subtitle_size + if wrapped { ROW_SUBTITLE_EXTRA } else { 0.0 };
        let text_top = text_center_y - text_height * 0.5;
        draw_text(
            &painter,
            Pos2::new(text_left, text_top),
            Align2::LEFT_TOP,
            truncate_to_width(
                &painter,
                &settings_row_label(setting, language),
                &label_font,
                text_width,
            ),
            label_font.clone(),
            metrics.text_primary,
            None,
        );
        let subtitle_top = text_top + row_label_size + 4.0;
        if is_app_icon {
            let icon = (row_rect.height() - 16.0).min(44.0);
            if let Some(texture) = raw_value.and_then(|id| assets.app_icon(id)) {
                painter.image(
                    texture,
                    Rect::from_min_size(
                        Pos2::new(
                            row_rect.right() - 4.0 - icon,
                            row_rect.center().y - icon * 0.5,
                        ),
                        Vec2::splat(icon),
                    ),
                    Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                    Color32::WHITE,
                );
            }
        }
        if wrapped {
            painter.galley(
                Pos2::new(text_left, subtitle_top),
                wrapped_regular(
                    &painter,
                    &localized(
                        &format!("settings.row.{}.description", setting.key),
                        language,
                    ),
                    subtitle_size,
                    ink(100),
                    text_width,
                    2,
                ),
                ink(100),
            );
        } else {
            draw_text(
                &painter,
                Pos2::new(text_left, subtitle_top),
                Align2::LEFT_TOP,
                truncate_to_width(
                    &painter,
                    &if value_below || is_app_icon || is_swatches {
                        value.clone().unwrap_or_default()
                    } else {
                        localized(
                            &format!("settings.row.{}.description", setting.key),
                            language,
                        )
                    },
                    &subtitle_font,
                    text_width,
                ),
                subtitle_font,
                if value_below || is_app_icon || is_swatches {
                    ink(tone::subtle())
                } else {
                    ink(if compact { 100 } else { 120 })
                },
                None,
            );
        }
        if is_swatches {
            let current = raw_value.unwrap_or_default().to_ascii_uppercase();
            let count = setting.options.len() as f32;
            let gap = if compact { 10.0 } else { 14.0 };
            let available = row_rect.width() - 8.0;
            let size = 40.0_f32.min((available - gap * (count - 1.0)) / count);
            let top = row_rect.top() + row_metrics.settings_row_height + 2.0;
            let strip = Rect::from_min_size(
                Pos2::new(row_rect.left() + 4.0, top),
                Vec2::new(count * size + (count - 1.0) * gap, size),
            );
            for slot in 0..setting.options.len() {
                layout.focusable.push((
                    NODE_SETTINGS_SWATCH_BASE + slot as u64,
                    Rect::from_min_size(
                        strip.min + Vec2::new(slot as f32 * (size + gap), 0.0),
                        Vec2::splat(size),
                    )
                    .expand(gap * 0.5),
                ));
            }
            components::placed_front(
                context,
                Id::new(("fluxa-settings-swatches", node)),
                strip.min,
                |ui| {
                    ui.spacing_mut().item_spacing.x = gap;
                    ui.horizontal(|ui| {
                        for option in setting.options {
                            let (rect, response) =
                                ui.allocate_exact_size(Vec2::splat(size), Sense::click());
                            let glass = *option == "glass";
                            let color = if glass {
                                ink(tone::dim())
                            } else {
                                Color32::from_hex(option).unwrap_or(text_primary())
                            };
                            let active = option.eq_ignore_ascii_case(&current);
                            let grow = crate::motion::toggle(
                                ui.ctx(),
                                Id::new(("fluxa-swatch", *option)),
                                active || response.hovered(),
                                crate::motion::fast(),
                            );
                            let center = rect.center();
                            ui.painter().circle_filled(
                                center,
                                size * 0.5 - 4.0 + 2.0 * grow,
                                color,
                            );
                            if glass {
                                let radius = size * 0.5 - 4.0 + 2.0 * grow;
                                ui.painter().circle_stroke(
                                    center,
                                    radius,
                                    egui::Stroke::new(stroke::icon(), ink(tone::body())),
                                );
                                ui.painter().circle_filled(
                                    center - Vec2::splat(radius * 0.38),
                                    radius * 0.26,
                                    ink(tone::subtle()),
                                );
                            }
                            if active {
                                ui.painter().circle_stroke(
                                    center,
                                    size * 0.5,
                                    egui::Stroke::new(stroke::emphasis(), text_primary()),
                                );
                            } else if response.hovered() {
                                ui.painter().circle_stroke(
                                    center,
                                    size * 0.5,
                                    egui::Stroke::new(stroke::hairline(), ink(tone::dim())),
                                );
                            }
                            let name = match *option {
                                "glass" => "settings.liquid_glass",
                                "#FFFFFF" => "auto.white",
                                "#E50914" => "auto.red",
                                "#3F7CFF" => "auto.blue",
                                "#54D17A" => "auto.green",
                                "#FF8A3D" => "auto.orange",
                                "#C084FC" => "auto.purple",
                                _ => "",
                            };
                            let response = if name.is_empty() {
                                response
                            } else {
                                response.on_hover_text(localized(name, language))
                            };
                            if response.clicked() && !active {
                                layout.setting_change = Some((
                                    setting.key.to_owned(),
                                    serde_json::Value::String((*option).to_owned()),
                                ));
                            }
                        }
                    });
                },
            );
        } else if sheet_row {
            if !value_below && !is_app_icon {
                let tip = row_rect.right_center() - Vec2::new(6.0, 0.0);
                let stroke = egui::Stroke::new(stroke::icon(), ink(tone::muted()));
                painter.line_segment([tip + Vec2::new(-5.0, -6.0), tip], stroke);
                painter.line_segment([tip + Vec2::new(-5.0, 6.0), tip], stroke);
            }
            if !value_below && !is_app_icon {
                let tip = row_rect.right_center() - Vec2::new(6.0, 0.0);
                draw_text(
                    &painter,
                    tip - Vec2::new(16.0, 0.0),
                    Align2::RIGHT_CENTER,
                    truncate_to_width(
                        &painter,
                        value.as_deref().unwrap_or_default(),
                        &value_font,
                        row_rect.width() * 0.4,
                    ),
                    value_font,
                    metrics.text_secondary,
                    None,
                );
            }
        } else if let (Some(raw_value), Some(value)) = (raw_value, value) {
            let value_rect = Rect::from_center_size(
                row_rect.right_center() - Vec2::new(value_width * 0.5 + 2.0, 0.0),
                Vec2::new(
                    value_width,
                    metrics.screen_control_height.min(row_rect.height() - 4.0),
                ),
            );
            let mut selected = raw_value.to_owned();
            components::placed_front(
                context,
                Id::new(("fluxa-settings-dropdown", node)),
                value_rect.min,
                |ui| {
                    ui.set_min_size(value_rect.size());
                    ui.set_max_size(value_rect.size());
                    let choices = setting
                        .options
                        .iter()
                        .map(|option| {
                            let label = option_label(setting.key, option, language);
                            ((*option).to_owned(), label)
                        })
                        .collect::<Vec<_>>();
                    let response = components::choice_field(
                        ui,
                        Id::new(("settings-choice", node)),
                        value_rect.size(),
                        value_size,
                        &value,
                        &choices,
                        &mut selected,
                        value_rect.width(),
                        true,
                        false,
                        metrics,
                    );
                    layout.focusable.push((node, response.rect));
                },
            );
            if selected != raw_value {
                layout.setting_change =
                    Some((setting.key.to_owned(), serde_json::Value::String(selected)));
            }
        } else {
            let enabled = settings.bool_value(setting.key);
            let track_rect = Rect::from_center_size(
                row_rect.right_center() - Vec2::new(metrics.settings_toggle_width * 0.5 + 3.0, 0.0),
                Vec2::new(
                    metrics.settings_toggle_width,
                    metrics.settings_toggle_height,
                ),
            );
            components::toggle(
                context,
                &painter,
                Id::new(("fluxa-settings-toggle", node)),
                track_rect,
                enabled,
                metrics,
            );
        }
        if clicked {
            layout.activated = Some(node);
        }
    }
    if let Some(error) = settings.last_write_error.as_deref() {
        draw_text(
            &painter,
            Pos2::new(margin, viewport.height - 24.0),
            Align2::LEFT_BOTTOM,
            error,
            crate::fonts::regular(metrics.text.body),
            danger_color(),
            None,
        );
    }
    layout
        .focusable
        .extend(navigation_focus_rects(viewport, metrics));
    components::focus_ring(&painter, &layout.focusable, focused, viewport, metrics);
    layout
}

pub(super) fn parse_settings_color(value: &str) -> Option<Color32> {
    let hex = value.strip_prefix('#').unwrap_or(value);
    if hex.len() != 6 {
        return None;
    }
    let rgb = u32::from_str_radix(hex, 16).ok()?;
    Some(Color32::from_rgb(
        ((rgb >> 16) & 0xff) as u8,
        ((rgb >> 8) & 0xff) as u8,
        (rgb & 0xff) as u8,
    ))
}

pub(crate) fn accent_needs_dark_foreground(color: Color32) -> bool {
    let linear = |channel: u8| {
        let value = channel as f32 / 255.0;
        if value <= 0.04045 {
            value / 12.92
        } else {
            ((value + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * linear(color.r()) + 0.7152 * linear(color.g()) + 0.0722 * linear(color.b()) > 0.55
}
