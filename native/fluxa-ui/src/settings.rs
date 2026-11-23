use super::*;

mod about;
mod account;
mod addons;
mod ambient;
mod badges;
mod blocks;
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
pub use about::settings_link_url;
pub use account::*;
pub use addons::{
    ADDON_NAME_LIMIT, addon_action, addon_action_node, addon_configure_url, addon_enabled,
    addon_title, addon_tracking_consent, addon_transport_url, kit_section, plugin_config_fields,
    plugin_config_input, server_index, server_input,
};
pub use badges::{badge_input, badge_input_label};
pub use collections::{collection_input, collection_input_label, collection_view_modes};
pub use fields::*;
pub use model::*;
pub use poster_fields::*;
pub use report::*;
pub use sections::*;

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

fn category_group(category: &str) -> &'static str {
    match category {
        "Account" | "Appearance" | "Playback" | "Watch Party" => "general",
        "Content" | "Downloads" => "library",
        "Streamkit" | "Metadatakit" => "sources",
        _ => "app",
    }
}

pub(super) fn row_applies(key: &str, viewport: Viewport) -> bool {
    let compact = viewport.is_compact();
    let desktop = !compact && !viewport.is_tv();
    match key {
        "navLayout" | "navMode" | "navSidebarMode" | "homeCategoryBar" | "interfaceDensity"
        | "posterHoverPreview" => !compact,
        "navLabels" => compact,
        "uiScale" => !compact,
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

fn subpages(active: usize) -> impl Iterator<Item = usize> {
    category_pages(active).filter(move |&page| page != active)
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

/// Pure geometry used by input adapters. It intentionally has no the backend state
/// or asset dependency, so Android touch/D-pad and desktop mouse hit testing

fn color_name(value: &str) -> Option<&'static str> {
    Some(match value {
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
            None if fluxa_theme::theme::is_builtin(value) => {
                return localized(&fluxa_theme::theme::name_key(value), language);
            }
            None => None,
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
        ("subtitleColor" | "subtitleBackgroundColor" | "subtitleOutlineColor", _) => {
            color_name(value)
        }
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

pub fn dialog_data(settings: &SettingsModel) -> serde_json::Value {
    let language = settings.language();
    if settings.report_open {
        return serde_json::json!({
            "kind": "report",
            "title": localized("settings.report.title", language),
            "description": localized("settings.report.description", language),
            "titleHint": localized("settings.report.title_hint", language),
            "bodyHint": localized("settings.report.body_hint", language),
            "titleValue": settings.report_title,
            "bodyValue": settings.report_body,
            "counter": format!("{}/{}", settings.report_body.chars().count(), REPORT_BODY_LIMIT),
            "ready": !settings.report_title.trim().is_empty() && !settings.report_body.trim().is_empty(),
            "cancel": localized("common.cancel", language),
            "send": localized("settings.report.send", language),
            "fill": format!("{NODE_REPORT_TITLE}=reportTitle;{NODE_REPORT_BODY}=reportBody"),
            "sendNode": NODE_REPORT_SEND.to_string(),
            "cancelNode": NODE_REPORT_CANCEL.to_string(),
        });
    }
    let Some(addon) = settings
        .addon_edit
        .and_then(|index| settings.addons.get(index))
    else {
        return serde_json::Value::Null;
    };
    serde_json::json!({
        "kind": "addon",
        "title": localized("settings.addon_edit", language),
        "urlLabel": localized("settings.addon_edit_url", language),
        "url": addons::addon_transport_url(addon).unwrap_or_default(),
        "nameLabel": localized("settings.addon_edit_name", language),
        "nameValue": settings.addon_edit_name,
        "nameHint": addon.get("name").and_then(serde_json::Value::as_str).unwrap_or_default(),
        "hint": localized("settings.addon_edit_hint", language),
        "cancel": localized("common.cancel", language),
        "save": localized("settings.addon_edit_save", language),
        "fill": format!("{NODE_ADDON_EDIT_NAME}=addonName"),
        "saveNode": NODE_ADDON_EDIT_SAVE.to_string(),
        "cancelNode": NODE_ADDON_EDIT_CANCEL.to_string(),
    })
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

    #[test]
    fn disconnected_sync_source_shows_the_connected_one() {
        let settings = SettingsModel {
            values: serde_json::json!({"continueWatchingSource": "simkl"}),
            profile: serde_json::json!({"nuvioAccessToken": "t"}),
            ..Default::default()
        };
        let (_, current, label) = account_source_state(&settings, "continueWatchingSource");
        assert_eq!(current, "nuvio");
        assert!(!label.is_empty());
    }
}

pub fn settings_data(settings: &SettingsModel, form: UiFormFactor) -> serde_json::Value {
    let viewport = Viewport::new(1280, 720, form);
    let language = settings.language();
    let active = settings.active_section.min(SETTINGS_SECTIONS.len() - 1);
    let active_category = SETTINGS_SECTIONS[active].category;
    let mut last_group = "";
    let categories: Vec<_> = categories(viewport)
        .map(|index| {
            let section = &SETTINGS_SECTIONS[index];
            let group = category_group(section.category);
            let heading = if group == last_group {
                String::new()
            } else {
                localized(&format!("settings.nav_group.{group}"), language).to_uppercase()
            };
            last_group = group;
            serde_json::json!({
                "icon": section.category,
                "heading": heading,
                "hasHeading": !heading.is_empty(),
                "id": index.to_string(),
                "key": "settingsCategory",
                "label": section_label(section.category, language),
                "selected": section.category == active_category,
            })
        })
        .collect();
    let hub = shows_hub(settings);
    let pages: Vec<_> = subpages(active)
        .filter(|_| hub)
        .map(|index| {
            serde_json::json!({
                "icon": section_icon(SETTINGS_SECTIONS[index].title, SETTINGS_SECTIONS[index].category),
                "description": section_description(SETTINGS_SECTIONS[index].title, language),
                "id": index.to_string(),
                "key": "settingsSection",
                "label": section_label(SETTINGS_SECTIONS[index].title, language),
                "selected": index == active,
            })
        })
        .collect();
    let groups: Vec<_> = visible_groups(settings, viewport)
        .into_iter()
        .map(|(heading, rows)| {
            let rows: Vec<_> = rows
                .into_iter()
                .filter_map(settings_row_by_index)
                .map(|row| {
                    let choices = row.choices();
                    let toggle = choices.is_empty() && slider_range(row.key).is_none();
                    let value = settings.value(row.key);
                    let raw = value
                        .and_then(serde_json::Value::as_str)
                        .map(str::to_owned)
                        .or_else(|| {
                            settings
                                .number_value(row.key)
                                .map(|number| number.to_string())
                        })
                        .unwrap_or_else(|| choices.first().copied().unwrap_or_default().to_owned());
                    let options: Vec<_> = if let Some(range) = slider_range(row.key) {
                        range
                            .marks
                            .iter()
                            .map(|mark| {
                                let text = range.format(*mark);
                                serde_json::json!({"value": text, "label": text})
                            })
                            .collect()
                    } else {
                        choices
                            .iter()
                            .map(|option| {
                                serde_json::json!({
                                    "value": option,
                                    "label": option_label(row.key, option, language),
                                })
                            })
                            .collect()
                    };
                    serde_json::json!({
                        "key": row.key,
                        "label": settings_row_label(row, language),
                        "toggle": toggle,
                        "on": settings.bool_value(row.key),
                        "value": raw,
                        "options": options,
                    })
                })
                .collect();
            serde_json::json!({"heading": heading, "rows": rows})
        })
        .collect();
    let section = &SETTINGS_SECTIONS[active];
    let field_indices: Vec<usize> = match section.title {
        "TMDB" => vec![1],
        "MDBList" => vec![2],
        "Skip Segments" => vec![3, 4, 5],
        "Posters" => vec![0],
        "Previews" => vec![6, 7],
        title => field_slots(settings, title)
            .iter()
            .flat_map(|slot| slot.fields.iter().copied())
            .collect(),
    };
    let inputs: Vec<_> = field_indices
        .into_iter()
        .map(|index| &POSTER_FIELDS[index])
        .filter(|field| field.kind == FieldKind::Text)
        .map(|field| {
            serde_json::json!({
                "key": field.key,
                "label": localized(field.label, language),
                "help": localized(field.help, language),
                "placeholder": localized(field.hint, language),
                "value": settings.str_value(field.key).unwrap_or(""),
                "kind": if field.key.ends_with("ApiKey") { "password" } else { "text" },
                "save": localized("auto.save", language),
            })
        })
        .collect();
    let rows_json = |rows: Vec<(String, String)>| -> Vec<serde_json::Value> {
        rows.into_iter()
            .map(|(label, value)| serde_json::json!({"label": label, "value": value}))
            .collect()
    };
    let lists: Vec<serde_json::Value> = match section.title {
        "Summary" if settings.playback_summary.is_empty() => vec![serde_json::json!({
            "title": localized("settings.summary_title", language),
            "rows": [{"label": localized("settings.summary_empty", language), "value": ""}],
        })],
        "Summary" => settings
            .playback_summary
            .iter()
            .map(|group| {
                serde_json::json!({
                    "title": localized(&group.title, language),
                    "rows": rows_json(group.rows.clone()),
                })
            })
            .collect(),
        "Licenses" => vec![serde_json::json!({
            "title": localized("settings.about.credits", language),
            "rows": rows_json(about::credit_rows(language)),
        })],
        _ => Vec::new(),
    };
    let mut blocks = blocks::page_blocks(settings, section.title, language);
    let lead: Vec<_> = if section.title == "Account" && !blocks.is_empty() {
        vec![blocks.remove(0)]
    } else {
        Vec::new()
    };
    serde_json::json!({
        "lists": lists,
        "lead": lead,
        "blocks": blocks,
        "inputs": inputs,
        "categories": categories,
        "searchPlaceholder": localized("settings.search_placeholder", language),
        "pages": pages,
        "title": section_label(section.title, language),
        "description": section_description(section.title, language),
        "groups": groups,
        "empty": groups.is_empty(),
        "open": settings.section_open || !settings.search.trim().is_empty(),
        "backNode": NODE_SETTINGS_BACK.to_string(),
        "inPage": settings.page_open && category_pages(active).count() > 1,
        "backId": category_pages(active).next().unwrap_or(active).to_string(),
    })
}
