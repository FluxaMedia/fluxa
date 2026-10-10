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
mod style;

pub use home::*;
pub use layout::*;
pub use metrics::*;
pub use model::*;
pub use navigation::*;
pub use nodes::*;
use paint::*;
use style::*;

pub use calendar::draw_calendar;
pub use components::{
    ActionMenuItem, ActionMenuLayout, ActionMenuOutcome, FocusedField, Glass, InputKind,
    action_menu_layout, caret_rect, draw_action_menu, focused_field, input_kind, is_text_node,
    set_input_caret, set_liquid_glass,
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
    PROFILE_SPLASH_SECS, PinPrompt, PinPurpose, ProfileAvatar, ProfileAvatarPack, ProfileDraft,
    ProfileEntry, ProfilePickerSettings, ProfilesMode, ProfilesModel, ProfilesRequest,
    draw_profile_splash, draw_profiles,
};
use settings::settings_card_height;
pub use settings::{
    ACCOUNT_PROVIDERS, AccountPrompt, BadgeCustomRow, BadgePackRow, CollectionEdit,
    CollectionListRow, FolderEdit, FormatterPresetRow, FormatterRow, POSTER_FIELDS,
    SETTINGS_SECTIONS, SettingsModel, SettingsRow, SettingsSection, ShortcutRow, UpdateState,
    UpdateView, account_source, account_source_state, addon_action, addon_transport_url,
    addons_page, badge_input, badge_input_label, category_pages, collection_input,
    collection_input_label, collection_view_modes, draw_settings, option_label, poster_field,
    server_index, server_input, settings_model_from_core_snapshot, settings_page_for_node,
    accent_swatches, settings_row_by_index, settings_row_label,
};
pub use shorts::{ShortsModel, draw_shorts, shorts_feed};

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

pub fn draw_update_prompt(
    context: &egui::Context,
    viewport: Viewport,
    update: &UpdateView,
    language: &str,
    layout: &mut HomeLayout,
) {
    if !update.prompt {
        return;
    }
    let metrics = UiMetrics::for_viewport(viewport);
    let (title, version, percent) = match &update.state {
        UpdateState::Available(version) | UpdateState::Ready(version) => {
            ("settings.updates.available", version.as_str(), None)
        }
        UpdateState::Downloading(percent) => ("settings.updates.downloading", "", Some(*percent)),
        _ => return,
    };
    let width = (viewport.width - 32.0).min(440.0);
    let pad = 18.0;
    let title_size = metrics.nav_label_size + 3.0;
    let body_size = metrics.screen_card_subtitle_size + 2.0;
    let button_height = 40.0;
    let height = pad * 2.0
        + title_size
        + 6.0
        + body_size
        + if percent.is_some() { 20.0 } else { 16.0 + button_height };
    let card = egui::Rect::from_min_size(
        egui::Pos2::new(
            (viewport.width - width) * 0.5,
            (viewport.height * 0.06).max(40.0),
        ),
        egui::vec2(width, height),
    );
    let left = card.left() + pad;
    let body_top = card.top() + pad + title_size + 6.0;
    let body = match percent {
        Some(percent) => format!("{percent}%"),
        None => format!("Fluxa {version}"),
    };
    let gap = 10.0;
    let button_width = (width - pad * 2.0 - gap) * 0.5;
    let mut clicked = None;
    egui::Area::new(egui::Id::new("fluxa-update-prompt"))
        .constrain(false)
        .fixed_pos(card.min)
        .order(egui::Order::Tooltip)
        .show(context, |ui| {
            ui.allocate_rect(card, egui::Sense::click());
            let painter = ui.painter().clone();
            painter.rect_filled(card, metrics.dialog_radius, metrics.surface_raised);
            painter.rect_stroke(
                card,
                metrics.dialog_radius,
                egui::Stroke::new(stroke::HAIRLINE, metrics.border),
                egui::StrokeKind::Inside,
            );
            draw_text(
                &painter,
                egui::Pos2::new(left, card.top() + pad),
                egui::Align2::LEFT_TOP,
                localized(title, language),
                fonts::medium(title_size),
                metrics.text_primary,
                None,
            );
            draw_text(
                &painter,
                egui::Pos2::new(left, body_top),
                egui::Align2::LEFT_TOP,
                body,
                fonts::regular(body_size),
                metrics.text_muted,
                None,
            );
            if let Some(percent) = percent {
                let track = egui::Rect::from_min_size(
                    egui::Pos2::new(left, body_top + body_size + 12.0),
                    egui::vec2(width - pad * 2.0, 5.0),
                );
                painter.rect_filled(track, radius::BAR, metrics.ink(tone::STRONG));
                painter.rect_filled(
                    egui::Rect::from_min_size(
                        track.min,
                        egui::vec2(track.width() * f32::from(percent.min(100)) / 100.0, 5.0),
                    ),
                    radius::BAR,
                    metrics.text_primary,
                );
                layout.focusable.push((NODE_UPDATE_INSTALL, card));
                return;
            }
            let top = card.bottom() - pad - button_height;
            for (slot, (node, key, primary)) in [
                (NODE_UPDATE_LATER, "settings.updates.later", false),
                (NODE_UPDATE_INSTALL, "settings.updates.install", true),
            ]
            .into_iter()
            .enumerate()
            {
                let rect = egui::Rect::from_min_size(
                    egui::Pos2::new(left + slot as f32 * (button_width + gap), top),
                    egui::vec2(button_width, button_height),
                );
                let response = ui
                    .scope_builder(egui::UiBuilder::new().max_rect(rect), |ui| {
                        components::pill_button(
                            ui,
                            None,
                            &localized(key, language),
                            Some(button_width),
                            button_height,
                            metrics.nav_label_size + 2.0,
                            primary,
                            None,
                        )
                    })
                    .inner;
                layout.focusable.push((node, rect));
                if response.clicked() {
                    clicked = Some(node);
                }
            }
        });
    if clicked.is_some() {
        layout.activated = clicked;
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
