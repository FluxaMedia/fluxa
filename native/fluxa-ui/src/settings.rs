use super::*;
use crate::dp::Dp;
use crate::draw_text;
use crate::flex::{self, Align, Insets, Justify};

mod about;
mod account;
mod addons;
mod ambient;
mod badges;
mod collections;
mod compact;
mod detail_sections;
mod extended;
mod fields;
mod hero;
mod model;
mod page;
mod poster_fields;
mod report;
mod row;
mod sections;
mod summary;
mod watch_party;
use about::draw_about;
pub use about::settings_link_url;
pub use account::*;
pub use addons::{
    ADDON_NAME_LIMIT, addon_action, addon_action_node, addon_enabled, addon_title,
    addon_transport_url, kit_section, plugin_config_fields, plugin_config_input, server_index,
    server_input,
};
pub use badges::{badge_input, badge_input_label};
pub use collections::{collection_input, collection_input_label, collection_view_modes};
pub use compact::*;
use extended::draw_settings_extended_section;
pub use fields::*;
pub use model::*;
pub use poster_fields::*;
pub use report::*;
pub use sections::*;
use summary::*;
use watch_party::*;

fn settings_category_icon(
    assets: &impl HomeAssets,
    painter: &crate::gfx::Painter,
    bounds: Rect,
    name: &str,
    color: Color32,
    size: f32,
) {
    if let Some(icon) = assets.icon(name) {
        let rect = flex::anchored(bounds, Align2::CENTER_CENTER, Vec2::splat(size), 0.0);
        painter.image(icon, rect, full_uv(), color);
    }
}

fn section_icon(title: &'static str, category: &'static str) -> &'static str {
    match title {
        "TMDB" => "Film",
        "MDBList" | "Summary" => "List",
        "Skip Segments" => "SkipForward",
        "Theme" => "Sun",
        "Home" => "Home",
        "Ambient Mode" => "Sparkles",
        "Details" => "Info",
        "Subtitles" => "AudioSubtitles",
        "Player" => "PlayFilled",
        "Previews" => "Film",
        "Streamkit" => "Streamkit",
        "Metadatakit" => "Metadatakit",
        "Metadatakit Catalogs" => "List",
        "Badges" => "Flag",
        "Watch Party" => "Users",
        "Collections" => "LayoutGrid",
        "Add-ons" => "Plugins",
        "Plugins" => "Zap",
        "Streamkit Addons" => "Plugins",
        "Licenses" => "Copy",
        _ => category,
    }
}

const PAGE_TILE: usize = 1;
const PAGE_TITLE: usize = 2;
const PAGE_SUBTITLE: usize = 3;
const PAGE_CHEVRON: usize = 4;

fn icon_tile(
    assets: &impl HomeAssets,
    painter: &crate::gfx::Painter,
    tile: Rect,
    metrics: UiMetrics,
    name: &str,
    active: bool,
) {
    painter.rect_filled(
        tile,
        tile.width() * metrics.settings_tile_radius_share,
        Color32::from_white_alpha(if active { 28 } else { 14 }),
    );
    settings_category_icon(
        assets,
        painter,
        tile,
        name,
        if active {
            text_primary()
        } else {
            crate::stylesheet::color("text.body")
        },
        Dp::D20.px(),
    );
}

fn category_group(category: &str) -> &'static str {
    match category {
        "Account" | "Appearance" | "Playback" | "Watch Party" => "general",
        "Content" | "Downloads" => "library",
        "Streamkit" | "Metadatakit" => "sources",
        _ => "app",
    }
}

fn nav_heading(
    painter: &crate::gfx::Painter,
    band: Rect,
    group: &str,
    language: &str,
    metrics: UiMetrics,
) {
    let text = localized(&format!("settings.nav_group.{group}"), language).to_uppercase();
    let font = crate::fonts::medium(metrics.settings_nav_heading_size);
    let height = flex::text_size(painter, &text, &font).y;
    let line = flex::column([flex::leaf(Vec2::new(0.0, height)).id(0)])
        .align(Align::Stretch)
        .padding(Insets {
            top: band.height() * metrics.settings_nav_heading_share - height * 0.5,
            ..Insets::default()
        })
        .solve(band);
    if let Some(rect) = line.find(0) {
        flex::paint_text(
            painter,
            rect,
            &text,
            &font,
            metrics.text_muted,
            Align2::LEFT_CENTER,
        );
    }
}

pub(super) fn group_heading(
    painter: &crate::gfx::Painter,
    rect: Rect,
    top: f32,
    title: &str,
    metrics: UiMetrics,
) {
    let compact = metrics.compact;
    draw_text(
        painter,
        Pos2::new(rect.left() + metrics.settings_group_title_inset, top),
        Align2::LEFT_TOP,
        title.to_owned(),
        crate::fonts::bold(metrics.screen_card_subtitle_size + metrics.settings_group_title_boost),
        ink(if compact {
            metrics.settings_group_title_alpha_compact
        } else {
            metrics.settings_group_title_alpha
        }),
        None,
    );
}

fn wrapped_regular(
    painter: &crate::gfx::Painter,
    text: &str,
    size: f32,
    color: Color32,
    width: f32,
    rows: usize,
) -> std::sync::Arc<crate::gfx::Galley> {
    let mut job =
        crate::gfx::text::LayoutJob::simple(text.to_owned(), crate::fonts::regular(size), color, width);
    job.wrap.max_rows = rows;
    painter.layout_job(job)
}

fn settings_group_card_height(row_count: usize, metrics: UiMetrics) -> f32 {
    metrics.settings_row_height
        + row_count.saturating_sub(1) as f32 * metrics.settings_row_spacing
        + metrics.settings_card_padding
}

fn row_extra(
    painter: &crate::gfx::Painter,
    index: usize,
    width: f32,
    language: &str,
    metrics: UiMetrics,
) -> f32 {
    let Some(row) = settings_row_by_index(index) else {
        return 0.0;
    };
    if row.key == ACCENT_COLOR_KEY {
        return metrics.settings_swatch_strip_extra;
    }
    if slider_range(row.key).is_some() {
        return metrics.settings_slider_extra;
    }
    if !metrics.compact {
        let row_width = width - metrics.settings_row_inset * 2.0;
        return if row::description_overflows(painter, row, language, row_width, metrics) {
            metrics.settings_row_subtitle_extra
        } else {
            0.0
        };
    }
    if !row.choices().is_empty() {
        return 0.0;
    }
    let text_width =
        width - metrics.settings_row_inset * 2.0 - metrics.settings_toggle_width - 19.0;
    let size = metrics.screen_card_subtitle_size + 2.0;
    let description = localized(&format!("settings.row.{}.description", row.key), language);
    if description.chars().count() as f32 * size * 0.45 > text_width {
        metrics.settings_row_subtitle_extra
    } else {
        0.0
    }
}

fn rows_extra(
    painter: &crate::gfx::Painter,
    rows: &[usize],
    width: f32,
    language: &str,
    metrics: UiMetrics,
) -> f32 {
    rows.iter()
        .map(|&index| row_extra(painter, index, width, language, metrics))
        .sum()
}

fn described(mut metrics: UiMetrics) -> UiMetrics {
    let extra = if metrics.compact {
        metrics.settings_row_subtitle_extra
    } else {
        metrics.settings_row_subtitle_extra - 8.0
    };
    metrics.settings_row_height += extra;
    metrics.settings_row_spacing += extra;
    metrics
}

pub(super) fn row_applies(key: &str, viewport: Viewport) -> bool {
    let compact = viewport.is_compact();
    let desktop = !compact && !viewport.is_tv();
    match key {
        "navLayout" | "navMode" | "navSidebarMode" | "homeCategoryBar" | "interfaceDensity"
        | "posterHoverPreview" => !compact,
        "navLabels" => compact,
        "posterHoverPreviewDelay" => desktop,
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
            + metrics.settings_group_gap
    } else {
        0.0
    }
}

fn draw_page_cards(
    context: &crate::gfx::Context,
    viewport: Viewport,
    assets: &impl HomeAssets,
    painter: &crate::gfx::Painter,
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
    components::raised_card(painter, card, metrics);
    let title_size = metrics.nav_label_size + metrics.settings_page_title_boost;
    let subtitle_size = metrics.screen_card_subtitle_size + Dp::D2.px();
    let inset = metrics.settings_row_inset + Space::S4.px(&metrics);
    for (slot, page) in pages.into_iter().enumerate() {
        let node = NODE_SETTINGS_PAGE_BASE + page as u64;
        let row = flex::column([flex::leaf(Vec2::ZERO).width(width).height(height).id(0)])
            .padding(Insets {
                top: slot as f32 * height,
                ..Insets::default()
            })
            .solve(Rect::from_min_size(
                top_left,
                Vec2::new(width, (slot + 1) as f32 * height),
            ))
            .find(0)
            .unwrap_or(card);
        if slot > 0 {
            painter.line_segment(
                [
                    Pos2::new(row.left() + inset, row.top()),
                    Pos2::new(row.right(), row.top()),
                ],
                crate::gfx::Stroke::new(stroke::hairline(), crate::stylesheet::color("well")),
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
                    ui.painter().rect_filled(
                        rect,
                        metrics.card_radius,
                        crate::stylesheet::color("well"),
                    );
                }
                let line = flex::row([
                    flex::leaf(Vec2::splat(Dp::D40.px())).id(PAGE_TILE),
                    flex::column([
                        flex::leaf(Vec2::new(0.0, title_size)).id(PAGE_TITLE),
                        flex::leaf(Vec2::new(0.0, subtitle_size)).id(PAGE_SUBTITLE),
                    ])
                    .align(Align::Stretch)
                    .gap(Space::S4.px(&metrics))
                    .grow(1.0),
                    flex::row([
                        flex::leaf(Vec2::new(Dp::D4.px(), Dp::D6.px() * 2.0)).id(PAGE_CHEVRON)
                    ])
                    .justify(Justify::End)
                    .align(Align::Center)
                    .padding(Insets {
                        right: Dp::D20.px(),
                        ..Insets::default()
                    })
                    .width(Dp::D48.px()),
                ])
                .align(Align::Center)
                .gap(Dp::D14.px())
                .padding(Insets {
                    left: inset,
                    ..Insets::default()
                })
                .solve(rect);
                if let Some(tile) = line.find(PAGE_TILE) {
                    icon_tile(
                        assets,
                        ui.painter(),
                        tile,
                        metrics,
                        section_icon(
                            SETTINGS_SECTIONS[page].title,
                            SETTINGS_SECTIONS[page].category,
                        ),
                        false,
                    );
                }
                if let Some(rect) = line.find(PAGE_TITLE) {
                    flex::paint_text(
                        ui.painter(),
                        rect,
                        &title,
                        &crate::fonts::regular(title_size),
                        text_primary(),
                        Align2::LEFT_TOP,
                    );
                }
                if let Some(rect) = line.find(PAGE_SUBTITLE) {
                    flex::paint_text(
                        ui.painter(),
                        rect,
                        &description,
                        &crate::fonts::regular(subtitle_size),
                        metrics.text_muted,
                        Align2::LEFT_TOP,
                    );
                }
                if let Some(chevron) = line.find(PAGE_CHEVRON) {
                    row::chevron(ui.painter(), chevron, crate::stylesheet::color("glyph"));
                }
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

pub(super) fn flow_area<R>(
    context: &crate::gfx::Context,
    id: impl std::hash::Hash + std::fmt::Debug,
    rect: Rect,
    top: f32,
    add: impl FnOnce(&mut crate::gfx::Ui) -> R,
) -> R {
    crate::gfx::Area::new(Id::new(id))
        .constrain(false)
        .fixed_pos(Pos2::new(rect.left(), top))
        .interactable(false)
        .show(context, |ui| {
            let mut flow = ui.new_child(crate::gfx::UiBuilder::new().max_rect(Rect::from_min_size(
                Pos2::new(rect.left(), top),
                Vec2::new(rect.width(), f32::INFINITY),
            )));
            flow.spacing_mut().item_spacing = Vec2::ZERO;
            let result = add(&mut flow);
            let key = Id::new("fluxa-settings-content-bottom");
            let bottom = flow.cursor().top();
            context.data_mut(|data| {
                let seen = data.get_temp::<f32>(key).unwrap_or(f32::MIN);
                data.insert_temp(key, seen.max(bottom));
            });
            result
        })
        .inner
}

fn take_content_bottom(context: &crate::gfx::Context) -> Option<f32> {
    let key = Id::new("fluxa-settings-content-bottom");
    context.data_mut(|data| {
        let seen = data.get_temp::<f32>(key);
        data.remove::<f32>(key);
        seen
    })
}

fn page_top(rect: Rect, metrics: UiMetrics) -> f32 {
    rect.top() + metrics.settings_page_lead
}

fn field_group_height(fields: usize, metrics: UiMetrics) -> f32 {
    metrics.settings_group_heading_height
        + fields as f32 * metrics.settings_field_height
        + metrics.settings_group_gap
}

struct FieldSlot {
    after: usize,
    title: &'static str,
    fields: &'static [usize],
}

fn field_slots(settings: &SettingsModel, title: &str) -> &'static [FieldSlot] {
    if !settings.search.trim().is_empty() {
        return &[];
    }
    match title {
        "Metadatakit" => &[
            FieldSlot {
                after: 0,
                title: "settings.group.kit_name",
                fields: &[24],
            },
            FieldSlot {
                after: 1,
                title: "settings.group.api_key",
                fields: &[1, 10, 11, 12, 13, 14, 15],
            },
            FieldSlot {
                after: 4,
                title: "settings.group.kit_tools",
                fields: &[37, 38],
            },
        ],
        "Metadatakit Catalogs" => &[FieldSlot {
            after: 3,
            title: "settings.group.catalogs_lists",
            fields: &[16, 17, 18, 19, 20, 21, 22, 40, 41],
        }],
        "Streamkit" => &[
            FieldSlot {
                after: 0,
                title: "settings.group.kit_name",
                fields: &[23],
            },
            FieldSlot {
                after: 2,
                title: "settings.group.source_filters",
                fields: &[26, 27, 28, 29, 30, 31],
            },
            FieldSlot {
                after: 3,
                title: "settings.group.advanced_rules",
                fields: &[8, 9, 32, 33, 34, 39],
            },
            FieldSlot {
                after: 4,
                title: "settings.group.debrid_key",
                fields: &[25, 35, 36],
            },
        ],
        _ => &[],
    }
}

fn slot_tops(
    settings: &SettingsModel,
    title: &str,
    rect: Rect,
    cards: &[Rect],
    metrics: UiMetrics,
) -> Vec<(&'static FieldSlot, f32)> {
    field_slots(settings, title)
        .iter()
        .map(|slot| {
            let top = match slot.after.min(cards.len()).checked_sub(1) {
                None => rect.top() + metrics.settings_appearance_header_height,
                Some(previous) => cards[previous].bottom() + metrics.settings_group_gap,
            };
            (slot, top)
        })
        .collect()
}

fn kit_off(settings: &SettingsModel, title: &str) -> bool {
    match title {
        "Streamkit" => !settings.bool_value("streamkitEnabled"),
        "Metadatakit" => !settings.bool_value("metadatakitEnabled"),
        _ => false,
    }
}

fn lead_id(title: &str) -> Id {
    Id::new(("fluxa-settings-measured-lead", title))
}

fn groups_lead(
    context: &crate::gfx::Context,
    section: &SettingsSection,
    settings: &SettingsModel,
) -> f32 {
    if !settings.search.trim().is_empty() {
        return 0.0;
    }
    match section.title {
        "TMDB" | "Account" | "About" => context
            .data(|data| data.get_temp::<f32>(lead_id(section.title)))
            .unwrap_or(0.0),
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
        "posterHoverPreviewDelay" => settings.bool_value("posterHoverPreview"),
        "homeCategoryBar" => {
            settings
                .value("navLayout")
                .and_then(serde_json::Value::as_str)
                == Some("sidebar")
        }
        "trickplayMode" | "trickplayInterval" => !settings.bool_value("trickplayKeyframesOnly"),
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

pub fn accent_swatches() -> &'static [&'static str] {
    SETTINGS_SECTIONS
        .iter()
        .flat_map(|section| section.rows.iter())
        .find(|row| row.key == ACCENT_COLOR_KEY)
        .map_or(&[], |row| row.choices())
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
    ("settings.controller_back", "Esc / B"),
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

/// Pure geometry used by input adapters. It intentionally has no the backend state
/// or asset dependency, so Android touch/D-pad and desktop mouse hit testing

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
        .map(|row| (row.key, settings_row_label(row, language), row.choices()))
        .collect()
}

pub fn option_label(key: &str, value: &str, language: &str) -> String {
    let named = match (key, value) {
        (_, "off") => Some("settings.off"),
        ("themeId", _) => match fluxa_theme::user::pack_name(value) {
            Some(name) => return name,
            None => fluxa_theme::theme::theme(value)
                .ok()
                .map(|theme| theme.name_key.as_str()),
        },
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
            ACCENT_COLOR_KEY | "subtitleColor" | "subtitleBackgroundColor" | "subtitleOutlineColor",
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
        | "posterHoverPreviewDelay"
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

fn settings_panel_input(
    context: &crate::gfx::Context,
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
    context: &crate::gfx::Context,
    layout: &mut HomeLayout,
    id: u64,
    rect: Rect,
    label: &str,
    size: f32,
) {
    pill_button_styled(context, layout, id, rect, label, size, false);
}

fn pill_button_styled(
    context: &crate::gfx::Context,
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

pub fn draw_settings(
    context: &crate::gfx::Context,
    viewport: Viewport,
    settings: &SettingsModel,
    assets: &mut impl HomeAssets,
) -> HomeLayout {
    let mut layout = draw_settings_page(context, viewport, settings, assets);
    draw_report_modal(context, viewport, settings, &mut layout);
    addons::draw_addon_edit_modal(context, viewport, settings, &mut layout);
    context.move_to_top(crate::gfx::LayerId::new(
        crate::gfx::Order::Foreground,
        Id::new("fluxa-shared-top-bar"),
    ));
    layout
}

fn draw_settings_page(
    context: &crate::gfx::Context,
    viewport: Viewport,
    settings: &SettingsModel,
    assets: &mut impl HomeAssets,
) -> HomeLayout {
    let language = settings
        .values
        .get("language")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("en");
    let mut metrics = UiMetrics::for_viewport(viewport);
    metrics.accent = settings
        .value(ACCENT_COLOR_KEY)
        .and_then(serde_json::Value::as_str)
        .and_then(parse_settings_color)
        .unwrap_or(text_primary());
    metrics.accent_foreground = if accent_needs_dark_foreground(metrics.accent) {
        background()
    } else {
        text_primary()
    };
    let screen = page::screen(viewport);
    let painter = context.layer_painter(crate::gfx::LayerId::background());
    painter.rect_filled(screen, 0.0, background());
    let mut layout = HomeLayout::default();
    let mut early = false;
    crate::kit::screen_body(context, viewport, "settings", (), |ui| {
        let scroll_y = -ui.cursor().min.y;
        let compact = viewport.is_compact();
        let tv = viewport.is_tv();
        let desktop = !compact && !tv;
        layout.activated = draw_navigation_bar(context, viewport, 4, assets);
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
        let content_width = content_width(viewport, metrics);
        let content_x = if compact {
            margin
        } else {
            let left = margin + metrics.settings_content_offset;
            let spare = (viewport.width - left - margin - content_width).max(0.0);
            left + if tv { 0.0 } else { spare * 0.5 }
        };
        let searching = !settings.search.trim().is_empty();
        let frame = page::Frame {
            context,
            viewport,
            settings,
            language: &language,
            metrics,
            painter: &painter,
            scroll_y,
            section_top,
            active_section,
            searching,
        };
        if compact {
            if page::draw_compact_header(frame, assets, &mut layout) {
                early = true;
                ui.allocate_space(Vec2::new(
                    viewport.width,
                    section_top
                        + compact_settings_list_height(viewport, metrics)
                        + metrics.page_padding,
                ));
                return;
            }
        } else {
            page::draw_wide_nav(frame, assets, &mut layout);
        }
        let card_width = content_width;
        let section = &SETTINGS_SECTIONS[active_section];
        let groups = visible_groups(settings, viewport);
        let results = searching && !groups.is_empty();
        let shift = if results {
            -metrics.settings_appearance_header_height
        } else if compact && !groups.is_empty() {
            let lead = if groups[0].0 == section_label(section.title, language) {
                metrics.settings_group_heading_height
            } else {
                0.0
            };
            Dp::D8.px() - metrics.settings_appearance_header_height - lead
        } else {
            0.0
        };
        ui.allocate_space(Vec2::new(viewport.width, (section_top + shift).max(0.0)));
        let rect_top = ui.cursor().min.y;
        let rect = page::at(
            Rect::from_min_size(
                Pos2::new(content_x, rect_top),
                Vec2::new(card_width, metrics.settings_card_height),
            ),
            0.0,
            0.0,
            Vec2::new(card_width, metrics.settings_card_height),
        );
        let header = rect;
        let row_metrics = described(metrics);
        let slots = field_slots(settings, section.title);
        let mut stack = vec![
            flex::leaf(Vec2::ZERO)
                .height(
                    metrics.settings_appearance_header_height
                        + groups_lead(context, section, settings),
                )
                .width(card_width),
        ];
        let mut card_ids = Vec::with_capacity(groups.len());
        for (index, (_, rows)) in groups.iter().enumerate() {
            for slot in slots.iter().filter(|slot| slot.after == index) {
                stack.push(
                    flex::leaf(Vec2::ZERO)
                        .width(card_width)
                        .height(field_group_height(slot.fields.len(), metrics)),
                );
            }
            stack.push(
                flex::leaf(Vec2::ZERO)
                    .width(card_width)
                    .height(metrics.settings_group_heading_height),
            );
            let height = settings_group_card_height(rows.len(), row_metrics)
                + rows_extra(ui.painter(), rows, card_width, language, row_metrics);
            card_ids.push(stack.len());
            stack.push(
                flex::leaf(Vec2::ZERO)
                    .width(card_width)
                    .height(height)
                    .id(stack.len()),
            );
            stack.push(
                flex::leaf(Vec2::ZERO)
                    .width(card_width)
                    .height(metrics.settings_group_gap),
            );
        }
        let stack = flex::column(stack);
        let stacked = stack.outer_size();
        let placed = stack.solve(Rect::from_min_size(Pos2::new(content_x, rect_top), stacked));
        ui.allocate_space(stacked);
        let cards: Vec<Rect> = card_ids
            .iter()
            .map(|id| placed.find(*id).unwrap_or(rect))
            .collect();
        let page_title = section_label(section.title, language);
        for ((title, _), card) in groups.iter().zip(&cards) {
            if !(compact && !searching && *title == page_title) {
                group_heading(
                    &painter,
                    rect,
                    card.top() - metrics.settings_group_heading_height,
                    title,
                    metrics,
                );
            }
            painter.rect_filled(*card, metrics.card_radius, metrics.surface_raised);
            painter.rect_stroke(
                *card,
                metrics.card_radius,
                crate::gfx::Stroke::new(stroke::hairline(), ink(34)),
                crate::gfx::StrokeKind::Inside,
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
        let heading_font = FontId::proportional(if tv {
            metrics.screen_section_title_size_tv - metrics.control_gap
        } else if !compact {
            metrics.settings_section_title_size_desktop
        } else {
            metrics.screen_section_title_size
        });
        draw_text(
            &painter,
            page::at(
                header,
                if compact || tv {
                    metrics.settings_card_padding
                } else {
                    Dp::D6.px()
                },
                metrics.settings_title_top,
                Vec2::ZERO,
            )
            .min,
            Align2::LEFT_TOP,
            heading,
            heading_font,
            text_primary(),
            None,
        );
        if tv && !searching {
            draw_text(
                &painter,
                page::at(
                    header,
                    metrics.settings_card_padding,
                    metrics.settings_description_top,
                    Vec2::ZERO,
                )
                .min,
                Align2::LEFT_TOP,
                section_description(section.title, language),
                crate::fonts::regular(metrics.screen_card_subtitle_size),
                metrics.text_muted,
                None,
            );
        }
        take_content_bottom(context);
        if !searching {
            draw_settings_extended_section(
                context,
                viewport,
                settings,
                assets,
                section.title,
                language,
                rect,
                &cards,
                metrics,
                &mut layout,
            );
        }
        let content_bottom = take_content_bottom(context);
        if matches!(section.title, "TMDB" | "Account" | "About") {
            let trailing = if section.title == "About" {
                metrics.settings_group_gap
            } else {
                0.0
            };
            let lead = content_bottom.map_or(0.0, |bottom| {
                (bottom + trailing - (rect_top + metrics.settings_appearance_header_height))
                    .max(0.0)
            });
            let id = lead_id(section.title);
            if context.data(|data| data.get_temp::<f32>(id)) != Some(lead) {
                context.data_mut(|data| data.insert_temp(id, lead));
                context.request_repaint();
            }
        }
        let mut bottom = ui.cursor().min.y.max(content_bottom.unwrap_or(f32::MIN));
        if shows_hub(settings) {
            let top = if groups.is_empty() {
                content_bottom.unwrap_or(page_top(rect, metrics))
            } else {
                bottom
            };
            draw_page_cards(
                context,
                viewport,
                &*assets,
                &painter,
                active_section,
                language,
                metrics,
                Pos2::new(rect.left(), top),
                rect.width(),
                &mut layout,
            );
            bottom = top + links_height(metrics, settings);
        }
        let end = bottom.max(rect_top + metrics.settings_card_height);
        let gap = (end - ui.cursor().min.y).max(0.0) + metrics.page_padding;
        ui.allocate_space(Vec2::new(card_width, gap));
        let row_metrics = described(metrics);
        let rows = groups
            .iter()
            .zip(&cards)
            .flat_map(|((_, rows), card)| row::place_rows(ui, *card, rows, language, row_metrics))
            .collect::<Vec<_>>();
        for row in rows {
            row::draw_row(frame, assets, &mut layout, row);
        }
        if !searching && kit_off(settings, section.title) && cards.len() > 1 {
            let slots = slot_tops(settings, section.title, rect, &cards, metrics);
            let top = slots
                .iter()
                .map(|(_, top)| *top)
                .chain([cards[0].bottom() + metrics.settings_group_gap])
                .fold(f32::MAX, f32::min);
            let bottom = slots
                .iter()
                .map(|(slot, top)| top + field_group_height(slot.fields.len(), metrics))
                .chain(
                    cards
                        .last()
                        .map(|card| card.bottom() + metrics.settings_group_gap),
                )
                .fold(0.0, f32::max);
            let clear = metrics.icon_button_size + Dp::D28.px();
            let region = Rect::from_min_max(
                Pos2::new(rect.left() - Space::S8.px(&metrics), top.max(clear)),
                Pos2::new(
                    rect.right() + Space::S8.px(&metrics),
                    bottom.min(viewport.height - metrics.settings_bottom_reserve),
                ),
            );
            if region.height() > 0.0 {
                layout
                    .focusable
                    .retain(|(_, focus)| !region.contains(focus.center()));
                let layer =
                    crate::gfx::LayerId::new(crate::gfx::Order::Tooltip, Id::new("fluxa-settings-inert"));
                context
                    .layer_painter(layer)
                    .with_clip_rect(region)
                    .rect_filled(
                        Rect::from_min_max(Pos2::new(region.left(), top), region.right_bottom()),
                        metrics.card_radius,
                        background().gamma_multiply(0.78),
                    );
                crate::gfx::Area::new(Id::new("fluxa-settings-inert-area"))
                    .constrain(false)
                    .fixed_pos(region.min)
                    .order(crate::gfx::Order::Tooltip)
                    .show(context, |ui| {
                        ui.allocate_rect(region, Sense::click_and_drag());
                    });
            }
        }
        if let Some(error) = settings.last_write_error.as_deref() {
            let font = crate::fonts::regular(metrics.text.body);
            let slot = flex::stack(
                Align2::LEFT_BOTTOM,
                [flex::leaf(flex::text_size(&painter, error, &font)).id(0)],
            )
            .padding(Insets {
                left: margin,
                bottom: Dp::D24.px(),
                ..Insets::default()
            })
            .solve(screen)
            .find(0)
            .unwrap_or(screen);
            flex::paint_text(
                &painter,
                slot,
                error,
                &font,
                danger_color(),
                Align2::LEFT_BOTTOM,
            );
        }
    });
    if early {
        return layout;
    }
    layout
        .focusable
        .extend(navigation_focus_rects(viewport, metrics));
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

#[cfg(test)]
mod setup_tests {
    use super::*;

    #[test]
    fn exports_leave_out_keys_and_foreign_settings() {
        let values = serde_json::json!({
            "metadatakitLanguage": "tr-TR", "tmdbApiKey": "x", "metadatakitCacheEpoch": "1",
            "catalogMalTop": true, "streamLimit": "5", "debridApiKey": "k"
        });
        let text = setup_export("metadatakitSetup", &values);
        assert!(text.contains("metadatakitLanguage") && text.contains("catalogMalTop"));
        assert!(
            !text.contains("tmdbApiKey")
                && !text.contains("Epoch")
                && !text.contains("streamLimit")
        );
        assert!(setup_export("streamkitSetup", &values).contains("streamLimit"));
    }
}
