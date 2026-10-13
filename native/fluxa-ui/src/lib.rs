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
mod flow;
mod folder;
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
mod sidebar;
mod style;
mod welcome;

pub use home::*;
pub use layout::*;
pub use metrics::*;
pub use model::*;
pub use navigation::*;
pub use nodes::*;
use paint::*;
pub use sidebar::{set_nav_focus, set_nav_search, set_nav_sidebar, take_nav_search};
use style::*;

pub use calendar::draw_calendar;
pub use components::{
    ActionMenuItem, ActionMenuLayout, ActionMenuOutcome, FocusedField, Glass, InputKind,
    NODE_DISCOVER_RECENT_BASE, action_menu_layout, caret_rect, discover_recent, draw_action_menu,
    focused_field, input_kind, is_text_node, set_input_caret, set_liquid_glass,
};
pub use motion::page_transition;
pub use poster_overlay::{
    Enrichment, NumberColor, NumberStyle, Personal, PersonalIndex, PosterOverlays, TopNumbers,
    set_poster_enrichment, set_poster_landscape, set_poster_overlays, set_poster_personal,
    set_rating_logos, set_top_numbers,
};

pub use detail::{detail_row_at_y, detail_row_scroll_max, detail_scroll_max, draw_detail};
pub use discover::draw_discover;
pub use folder::{draw_folder, folder_row_at_y, folder_row_scroll_max, folder_scroll_max};
pub use library::draw_library;
pub use player::{
    ChapterSpan, NextEpisodeCard, PanelRow, PlayerModel, PlayerOptions, PlayerPanel, PlayerSource,
    PlayerToast, SegmentSpan, SkipCard, SkipKind, SourceBadge, StatsSection,
    content_warning_duration, draw_player, format_time, torrent_status_lines,
};
pub use profiles::{
    PROFILE_SPLASH_SECS, PinFailure, PinPrompt, PinPurpose, ProfileAvatar, ProfileAvatarPack, ProfileDraft,
    ProfileEntry, ProfilePickerSettings, ProfilesMode, ProfilesModel, ProfilesRequest,
    draw_profile_splash, draw_profiles,
};
use settings::settings_card_height;
pub use settings::{
    ACCOUNT_PROVIDERS, AccountPrompt, BadgeCustomRow, BadgePackRow, CollectionEdit,
    CollectionListRow, FolderEdit, FormatterPresetRow, FormatterRow, POSTER_FIELDS,
    SETTINGS_SECTIONS, SettingsModel, SettingsRow, SettingsSection, ShortcutRow, UpdateState,
    UpdateView, accent_swatches, account_source, account_source_state, addon_action,
    addon_transport_url, addons_page, badge_input, badge_input_label, category_pages,
    collection_input, collection_input_label, collection_view_modes, draw_settings, option_label,
    poster_field, server_index, server_input, settings_model_from_core_snapshot,
    settings_page_for_node, settings_row_by_index, settings_row_label,
};
pub use shorts::{ShortsModel, draw_shorts, shorts_feed};
pub use welcome::{WelcomeModel, WelcomeQr, WelcomeRequest, draw_welcome};

pub fn draw_notice(context: &egui::Context, viewport: Viewport, text: &str, opacity: f32) {
    let top = (viewport.height * 0.08).max(48.0);
    components::toast(
        context,
        viewport,
        egui::Pos2::new(viewport.width * 0.5, top),
        text,
        None,
        opacity,
    );
}

const UPDATE_BANNER_HEIGHT: f32 = 44.0;

pub fn update_reveal(context: &egui::Context, shown: bool) -> f32 {
    motion::presence(
        context,
        egui::Id::new("fluxa-update-reveal"),
        shown,
        crate::motion::slow(),
    )
}

pub fn update_banner_space(top: f32) -> f32 {
    top + UPDATE_BANNER_HEIGHT
}

pub fn draw_target_outlines(context: &egui::Context, layout: &HomeLayout) {
    let painter = context.layer_painter(egui::LayerId::new(
        egui::Order::Debug,
        egui::Id::new("fluxa-target-outlines"),
    ));
    for (node, rect) in &layout.focusable {
        painter.rect_stroke(
            *rect,
            0.0,
            egui::Stroke::new(1.0, egui::Color32::from_rgb(255, 64, 160)),
            egui::StrokeKind::Inside,
        );
        painter.text(
            rect.left_top(),
            egui::Align2::LEFT_TOP,
            node.to_string(),
            egui::FontId::monospace(9.0),
            egui::Color32::from_rgb(255, 64, 160),
        );
    }
}

pub fn draw_top_focus(
    context: &egui::Context,
    viewport: Viewport,
    layout: &HomeLayout,
    focused: Option<u64>,
) {
    let focused = focused.filter(|node| {
        navigation::is_navigation_node(*node)
            || matches!(*node, NODE_UPDATE_INSTALL | NODE_UPDATE_LATER)
    });
    let painter = context.layer_painter(egui::LayerId::new(
        egui::Order::Tooltip,
        egui::Id::new("fluxa-top-focus"),
    ));
    components::focus_ring(
        &painter,
        &layout.focusable,
        focused,
        viewport,
        UiMetrics::for_viewport(viewport),
    );
}

pub fn draw_update_prompt(
    context: &egui::Context,
    viewport: Viewport,
    update: &UpdateView,
    scroll: f32,
    language: &str,
    assets: &impl HomeAssets,
    layout: &mut HomeLayout,
) {
    if update.reveal <= 0.0 {
        return;
    }
    let id = egui::Id::new("fluxa-update-prompt");
    if let UpdateState::Available(version) | UpdateState::Ready(version) = &update.state {
        context.data_mut(|data| data.insert_temp(id, version.clone()));
    }
    let version = context
        .data(|data| data.get_temp::<String>(id))
        .unwrap_or_default();
    let (title, trailing, action, progress) = match &update.state {
        UpdateState::Available(_) | UpdateState::Ready(_) => (
            format!("v{version}"),
            String::new(),
            Some("settings.updates.install"),
            None,
        ),
        UpdateState::Downloading(percent) => (
            format!("v{version}"),
            format!("{percent}%"),
            None,
            Some(f32::from(*percent) / 100.0),
        ),
        UpdateState::Failed(_) => (
            localized("settings.updates.failed_detail", language),
            String::new(),
            Some("common.retry"),
            None,
        ),
        _ => return,
    };
    let metrics = UiMetrics::for_viewport(viewport);
    let rect = if viewport.is_tv() {
        let top = update.top - (1.0 - update.reveal) * (UPDATE_BANNER_HEIGHT + update.top);
        egui::Rect::from_min_size(
            egui::Pos2::new(48.0, top),
            egui::vec2(viewport.width - 96.0, UPDATE_BANNER_HEIGHT),
        )
    } else {
        let bottom = update_banner_space(update.top) * update.reveal - scroll;
        egui::Rect::from_min_max(
            egui::Pos2::new(0.0, bottom - UPDATE_BANNER_HEIGHT),
            egui::Pos2::new(viewport.width, bottom),
        )
    };
    let action = action.map(|key| localized(key, language));
    let live = update.prompt && rect.bottom() > 0.0;
    let outcome = components::banner(
        context,
        id,
        rect,
        update.reveal,
        components::Banner {
            title: &title,
            trailing: &trailing,
            action: action.as_deref().map(|label| (NODE_UPDATE_INSTALL, label)),
            close: assets.icon("Close").map(|icon| (NODE_UPDATE_LATER, icon)),
            progress,
        },
        metrics,
        live.then_some(&mut *layout),
    );
    if !live {
        return;
    }
    if outcome.action_clicked {
        layout.activated = Some(NODE_UPDATE_INSTALL);
    } else if outcome.close_clicked {
        layout.activated = Some(NODE_UPDATE_LATER);
    }
}

pub fn localized(key: &str, language: &str) -> String {
    localized_or(key, key, language)
}

pub fn localized_or(key: &str, fallback: &str, language: &str) -> String {
    static ENGLISH: OnceLock<serde_json::Value> = OnceLock::new();
    static TURKISH: OnceLock<serde_json::Value> = OnceLock::new();
    static ARABIC: OnceLock<serde_json::Value> = OnceLock::new();
    static HEBREW: OnceLock<serde_json::Value> = OnceLock::new();
    let english = ENGLISH.get_or_init(|| {
        serde_json::from_str(include_str!("../../../shared/i18n/english_us.json"))
            .expect("English translations")
    });
    let selected = if language.starts_with("tr") {
        TURKISH.get_or_init(|| {
            serde_json::from_str(include_str!("../../../shared/i18n/tr_tr.json"))
                .expect("Turkish translations")
        })
    } else if language.starts_with("ar") {
        ARABIC.get_or_init(|| {
            serde_json::from_str(include_str!("../../../shared/i18n/ar.json"))
                .expect("Arabic translations")
        })
    } else if language.starts_with("he") || language.starts_with("iw") {
        HEBREW.get_or_init(|| {
            serde_json::from_str(include_str!("../../../shared/i18n/he.json"))
                .expect("Hebrew translations")
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
