use egui::{Align2, Color32, FontId, Id, Pos2, Rect, RichText, Sense, TextureId, Vec2};
use fluxa_theme::theme::active_snapshot;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    hash::{Hash, Hasher},
    sync::OnceLock,
    time::Duration,
};

mod ambient;
mod calendar;
mod card_logos;
mod components;
mod detail;
mod discover;
mod draw;
mod effects;
mod emoji;
mod flex;
mod flow;
mod folder;
pub mod fonts;
mod gallery;
mod haptics;
mod home;
mod home_category;
mod interact;
mod layout;
mod library;
mod metrics;
mod model;
mod motion;
mod navbar;
mod navigation;
mod nodes;
mod paint;
mod player;
mod poster_overlay;
mod profiles;
mod scroll_physics;
mod settings;
mod shorts;
mod sidebar;
mod style;
mod update;
mod watch_together;
mod welcome;
mod window_button;

pub use card_logos::{enable_card_logos, resolve_card_logos, take_wanted_logos};
pub use components::{
    DetailModalOutcome, draw_detail_modal, set_detail_modal_open, set_screen_covered, set_ui_settled,
    take_detail_modal_outcome,
};
use draw::*;
use flex::*;
pub use home::*;
pub use home_category::{HomeCategory, home_category, set_home_category};
use interact::FocusNode;
pub use interact::{
    Dir, Neighbors, Pinch, begin_focus_frame, pinch, take_focus_neighbors, take_focus_targets,
};
pub use layout::*;
pub use metrics::*;
pub use model::*;
pub use navbar::{nav_bar_rect, nav_node_at, nav_slot_center};
pub use navigation::*;
pub use nodes::*;
use paint::*;
pub use scroll_physics::{ScrollFeel, ScrollMotion, VelocityTracker, drag as scroll_drag};
pub use sidebar::{set_nav_focus, set_nav_search, set_nav_sidebar, take_nav_search};
use style::*;
pub use update::{draw_update_popup, set_nav_update};
pub use window_button::draw_window_fullscreen_button;

pub use ambient::{
    AmbientView, MOTION_COUNT as AMBIENT_MOTIONS, OVERLAY_IN as AMBIENT_FADE_IN,
    OVERLAY_OUT as AMBIENT_FADE_OUT, SLIDE_SECONDS as AMBIENT_SLIDE_SECONDS, draw_ambient,
    next_motion as next_ambient_motion,
};
pub use calendar::draw_calendar;
pub use components::{
    ActionMenuItem, ActionMenuLayout, ActionMenuOutcome, DOM_TRAILER, FocusedField, Glass,
    ImageFit, ImageHole, InputKind, NODE_DISCOVER_RECENT_BASE, SubtitleLook, VideoHole,
    action_menu_hit, action_menu_layout, action_menu_scroll_to, action_menu_width, caret_rect,
    discover_recent, draw_action_menu, draw_action_menu_closing, draw_mini_player_overlay,
    focused_field, input_kind, is_text_node, keyboard_inset, set_input_caret, set_keyboard_inset,
    set_liquid_glass, set_native_field, set_pending_field_text,
};
pub use haptics::{Haptic, buzz, take_haptic};
pub use motion::{page_transition, reduce_motion, set_reduce_motion};
pub use poster_overlay::{
    Enrichment, NumberColor, NumberStyle, Personal, PersonalIndex, PosterOverlays, TopNumbers,
    set_poster_enrichment, set_poster_landscape, set_poster_overlays, set_poster_personal,
    set_rating_logos, set_top_numbers,
};

pub use detail::{detail_row_at_y, detail_row_scroll_max, detail_scroll_max, draw_detail};
pub use discover::draw_discover;
pub use folder::{draw_folder, folder_row_at_y, folder_row_scroll_max, folder_scroll_max};
pub use gallery::draw_gallery;
pub use library::draw_library;
pub use player::{
    ChapterSpan, NextEpisodeCard, PanelRow, PanelStep, PlayerModel, PlayerOptions, PlayerPanel,
    PlayerSource, PlayerToast, SegmentSpan, SkipCard, SkipKind, SourceBadge, StatsSection,
    SyncRole, ToastGauge, content_warning_duration, draw_player, format_time, is_sources_modal,
    subtitle_lift, torrent_status_lines,
};
pub use profiles::{
    PinFailure, PinPrompt, PinPurpose, ProfileAvatar, ProfileAvatarPack, ProfileDraft,
    ProfileEntry, ProfilePickerSettings, ProfilesMode, ProfilesModel, ProfilesRequest,
    draw_profiles,
};
use settings::settings_card_height;
pub use settings::{
    ACCENT_COLOR_KEY, ACCOUNT_PROVIDERS, ADDON_NAME_LIMIT, APP_ICON_KEY, AccountPrompt,
    AmbientCatalogRow, BadgeCustomRow, BadgePackRow, CollectionEdit, CollectionListRow,
    DetailSectionRow, FieldKind, FolderEdit, FormatterPresetRow, FormatterRow, POSTER_FIELDS,
    PlayerBadgeChoice, REPORT_BODY_LIMIT, REPORT_TITLE_LIMIT, SETTINGS_SECTIONS, SettingsModel,
    SettingsRow, SettingsSection, ShortcutRow, UpdateState, UpdateView, accent_swatches,
    account_source, account_source_state, addon_action, addon_action_node, addon_enabled,
    addon_title, addon_transport_url, addons_page, badge_input, badge_input_label, category_pages,
    collection_input, collection_input_label, collection_view_modes, draw_settings, kit_section,
    option_label, plugin_config_fields, plugin_config_input, poster_field, server_index,
    server_input, settings_link_url, settings_model_from_core_snapshot, settings_page_for_node,
    settings_row_by_index, settings_row_label, setup_accepts, setup_export, subtitle_style_rows,
};
pub use shorts::{ShortsModel, draw_shorts, shorts_feed};
pub use watch_together::{
    WatchChatView, WatchMemberView, WatchTicket, WatchTogetherModel, WatchTogetherRequest,
    draw_watch_side_panel, draw_watch_together, waiting_member,
};
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

pub fn draw_hover_tip(context: &egui::Context, viewport: Viewport, tip: &HoverTip) {
    let metrics = UiMetrics::for_viewport(viewport);
    let height = metrics.toast_text_size * 2.6;
    let gap = 8.0;
    let below = tip.rect.bottom() + gap;
    let top = if below + height > viewport.height {
        tip.rect.top() - gap - height
    } else {
        below
    };
    let x = tip
        .rect
        .center()
        .x
        .clamp(96.0, (viewport.width - 96.0).max(96.0));
    components::toast(
        context,
        viewport,
        egui::Pos2::new(x, top),
        &tip.text,
        None,
        1.0,
    );
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

pub fn draw_focus(
    context: &egui::Context,
    viewport: Viewport,
    layout: &HomeLayout,
    focused: Option<u64>,
    in_player: bool,
) {
    let painter = context.layer_painter(egui::LayerId::new(
        egui::Order::Debug,
        egui::Id::new("fluxa-focus"),
    ));
    components::focus_ring(
        &painter,
        &layout.focusable,
        focused,
        viewport,
        in_player,
        UiMetrics::for_viewport(viewport),
    );
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
