use egui::{Align2, Color32, FontId, Id, Pos2, Rect, RichText, Sense, TextureId, Vec2};
use fluxa_theme::theme::{active_shared_ui_tokens, active_theme};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    hash::{Hash, Hasher},
    sync::OnceLock,
    time::Duration,
};

mod calendar;
mod components;
mod detail;
mod discover;
mod emoji;
pub mod fonts;
mod home;
mod layout;
mod library;
mod metrics;
mod model;
mod motion;
mod navigation;
mod nodes;
mod paint;
mod player;
mod poster_overlay;
mod profiles;
mod settings;
mod shorts;

pub use home::*;
pub use layout::*;
pub use metrics::*;
pub use model::*;
pub use navigation::*;
pub use nodes::*;
use paint::*;

pub use calendar::draw_calendar;
pub use components::{
    ActionMenuItem, ActionMenuLayout, ActionMenuOutcome, Glass, action_menu_layout,
    draw_action_menu, is_text_node, set_input_caret, set_liquid_glass,
};
pub use motion::page_transition;
pub use poster_overlay::{
    Enrichment, NumberColor, NumberStyle, Personal, PersonalIndex, PosterOverlays, TopNumbers,
    set_poster_enrichment, set_poster_landscape, set_poster_overlays, set_poster_personal,
    set_rating_logos, set_top_numbers,
};

pub use detail::{detail_row_at_y, detail_row_scroll_max, detail_scroll_max, draw_detail};
pub use discover::draw_discover;
pub use library::draw_library;
pub use player::{
    ChapterSpan, NextEpisodeCard, PanelRow, PlayerModel, PlayerOptions, PlayerPanel, PlayerSource,
    PlayerToast, SegmentSpan, SkipCard, SkipKind, SourceBadge, StatsSection,
    content_warning_duration, draw_player, format_time, torrent_status_lines,
};
pub use profiles::{
    PinPrompt, PinPurpose, ProfileAvatar, ProfileAvatarPack, ProfileDraft, ProfileEntry,
    ProfilePickerSettings, ProfilesMode, ProfilesModel, ProfilesRequest, draw_profiles,
};
use settings::settings_card_height;
pub use settings::{
    ACCOUNT_PROVIDERS, AccountPrompt, BadgeCustomRow, BadgePackRow, POSTER_FIELDS,
    SETTINGS_SECTIONS, SettingsModel, SettingsRow, SettingsSection, ShortcutRow, account_source,
    account_source_state, addon_action, addon_transport_url, badge_input, badge_input_label,
    category_pages, draw_settings, option_label, poster_field, server_index, server_input,
    settings_model_from_core_snapshot, settings_page_for_node, settings_row_by_index,
    settings_row_label,
};
pub use shorts::{ShortsModel, draw_shorts, shorts_feed};

pub fn localized(key: &str, language: &str) -> String {
    localized_or(key, key, language)
}

pub fn localized_or(key: &str, fallback: &str, language: &str) -> String {
    static ENGLISH: OnceLock<serde_json::Value> = OnceLock::new();
    static TURKISH: OnceLock<serde_json::Value> = OnceLock::new();
    let english = ENGLISH.get_or_init(|| {
        serde_json::from_str(include_str!("../../../shared/i18n/english_us.json"))
            .expect("English translations")
    });
    let selected = if language.starts_with("tr") {
        TURKISH.get_or_init(|| {
            serde_json::from_str(include_str!("../../../shared/i18n/tr_tr.json"))
                .expect("Turkish translations")
        })
    } else {
        english
    };
    selected
        .get(key)
        .or_else(|| english.get(key))
        .and_then(serde_json::Value::as_str)
        .unwrap_or(fallback)
        .to_owned()
}

fn snapshot_language(snapshot: &serde_json::Value) -> String {
    snapshot
        .pointer("/settings/values/language")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("en")
        .to_owned()
}

fn setting_default(key: &str) -> Option<&'static serde_json::Value> {
    static DEFAULTS: OnceLock<HashMap<String, serde_json::Value>> = OnceLock::new();
    DEFAULTS
        .get_or_init(|| {
            let manifest: serde_json::Value = serde_json::from_str(include_str!(
                "../../../core/fluxa-core/settings/settings-manifest.json"
            ))
            .expect("Core settings manifest");
            manifest
                .get("settings")
                .and_then(serde_json::Value::as_array)
                .into_iter()
                .flatten()
                .filter_map(|entry| {
                    Some((
                        entry.get("key")?.as_str()?.to_owned(),
                        entry.get("default")?.clone(),
                    ))
                })
                .collect()
        })
        .get(key)
}

#[cfg(test)]
mod tests;
