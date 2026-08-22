use super::helpers::{
    active_profile_id, error_code, normalize_error, with_normalized_meta_trailers,
};
use super::player;
use super::state::GenerationKey;
use super::{EffectResultInput, HeadlessEngine};
use crate::runtime::{EffectEnvelope, EffectKind};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", default)]
pub(super) struct HomeState {
    is_loading: bool,
    is_stale: bool,
    categories: Value,
    continue_watching: Value,
    user_addons: Value,
    metadata_feeds: Value,
    billboard: Value,
    watchlist: Value,
    is_direct_loading: bool,
    active_profile: Value,
    external_continue_watching: Value,
    paging: HomePaging,
    error: Value,
    generation: u64,
}

impl Default for HomeState {
    fn default() -> Self {
        Self {
            is_loading: false,
            is_stale: false,
            categories: serde_json::json!([]),
            continue_watching: serde_json::json!([]),
            user_addons: serde_json::json!([]),
            metadata_feeds: serde_json::json!([]),
            billboard: Value::Null,
            watchlist: serde_json::json!([]),
            is_direct_loading: false,
            active_profile: Value::Null,
            external_continue_watching: serde_json::json!([]),
            paging: HomePaging::default(),
            error: Value::Null,
            generation: 0,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", default)]
pub(super) struct HomePaging {
    category_id: String,
    is_loading: bool,
    items: Value,
    error: Value,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ReadHomeBootstrapPayload {
    profile_id: String,
    profile: Value,
    language: String,
    force: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct PrepareDirectPlaybackPayload {
    meta: Value,
    language: String,
    profile: Value,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct FetchCatalogPagePayload {
    category_id: String,
    transport_url: Option<String>,
    content_type: String,
    catalog_id: String,
    skip: i32,
    genre: Option<String>,
    search: Option<String>,
    remote_source: Option<Value>,
    profile: Option<Value>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct RefreshContinueWatchingPayload {
    profile_id: String,
    profile: Value,
    language: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    source: Option<String>,
}

fn normalize_categories_trailers(mut categories: Value) -> Value {
    if let Some(cats) = categories.as_array_mut() {
        for category in cats {
            if let Some(items) = category.get_mut("items").and_then(Value::as_array_mut) {
                for item in items {
                    *item = with_normalized_meta_trailers(item.take());
                }
            }
        }
    }
    categories
}

pub(super) fn dispatch_refresh_continue_watching(
    engine: &mut HeadlessEngine,
    profile: Option<Value>,
    language: Option<String>,
    source: Option<String>,
) -> Vec<EffectEnvelope> {
    let profile_value = profile.unwrap_or_else(|| engine.state.profile.active.clone());
    let profile_id = active_profile_id(&engine.state, &profile_value);
    let generation = engine.bump_generation(GenerationKey::Home);
    vec![engine.effect(
        EffectKind::RefreshContinueWatching,
        generation,
        RefreshContinueWatchingPayload {
            profile_id,
            profile: profile_value,
            language: language.unwrap_or_else(|| "en".to_string()),
            source,
        },
    )]
}

pub(super) fn remove_from_continue_watching(engine: &mut HeadlessEngine, dropped_id: &str) {
    if let Some(items) = engine.state.home.continue_watching.as_array_mut() {
        items.retain(|item| item.get("id").and_then(Value::as_str) != Some(dropped_id));
    }
}

pub(super) fn mirror_active_profile(engine: &mut HeadlessEngine, profile: Value) {
    engine.state.home.active_profile = profile;
}

pub(super) fn set_user_addons(engine: &mut HeadlessEngine, addons: Value) {
    engine.state.home.user_addons = addons;
}

pub(super) fn set_external_continue_watching(engine: &mut HeadlessEngine, items: Value) {
    engine.state.home.external_continue_watching = items;
}

pub(super) fn dispatch_load(
    engine: &mut HeadlessEngine,
    profile: Option<Value>,
    language: Option<String>,
    force: Option<bool>,
) -> Vec<EffectEnvelope> {
    let generation = engine.bump_generation(GenerationKey::Home);
    let profile_value = profile.unwrap_or_else(|| engine.state.profile.active.clone());
    let profile_id = active_profile_id(&engine.state, &profile_value);
    let force = force.unwrap_or(false);
    if force {
        let mut home = (*engine.state.home).clone();
        home.is_loading = true;
        home.is_stale = false;
        home.error = Value::Null;
        home.generation = generation;
        *engine.state.home = home;
    } else {
        *engine.state.home = HomeState {
            is_loading: true,
            generation,
            ..HomeState::default()
        };
    }
    let language_value = language.unwrap_or_else(|| "en".to_string());
    vec![engine.effect(
        EffectKind::ReadHomeBootstrap,
        generation,
        ReadHomeBootstrapPayload {
            profile_id,
            profile: profile_value,
            language: language_value,
            force,
        },
    )]
}

pub(super) fn dispatch_direct_playback(
    engine: &mut HeadlessEngine,
    meta: Value,
    language: Option<String>,
    profile: Option<Value>,
) -> Vec<EffectEnvelope> {
    let generation = engine.bump_generation(GenerationKey::PlaybackPrep);
    engine.state.home.is_direct_loading = true;
    vec![engine.effect(
        EffectKind::PrepareDirectPlayback,
        generation,
        PrepareDirectPlaybackPayload {
            meta,
            language: language.unwrap_or_else(|| "en".to_string()),
            profile: profile.unwrap_or(Value::Null),
        },
    )]
}

#[allow(clippy::too_many_arguments)]
pub(super) fn dispatch_catalog_page(
    engine: &mut HeadlessEngine,
    category_id: String,
    transport_url: Option<String>,
    content_type: String,
    catalog_id: String,
    skip: Option<i32>,
    genre: Option<String>,
    search: Option<String>,
    remote_source: Option<Value>,
    profile: Option<Value>,
) -> Vec<EffectEnvelope> {
    let generation = engine.bump_generation(GenerationKey::Home);
    engine.state.home.paging = HomePaging {
        category_id: category_id.clone(),
        is_loading: true,
        items: Value::Null,
        error: Value::Null,
    };
    let skip = skip.unwrap_or(0).max(0);
    vec![engine.effect(
        EffectKind::FetchCatalogPage,
        generation,
        FetchCatalogPagePayload {
            category_id,
            transport_url,
            content_type,
            catalog_id,
            skip,
            genre,
            search,
            remote_source,
            profile,
        },
    )]
}

pub(super) fn complete(
    engine: &mut HeadlessEngine,
    effect_type: &str,
    generation: u64,
    result: &EffectResultInput,
) -> Vec<EffectEnvelope> {
    match effect_type {
        "refreshContinueWatching" => {
            if result.status.is_ok()
                && let Some(cw) = result.value.get("continueWatching")
            {
                engine.state.home.continue_watching = cw.clone();
            }
        }
        "readHomeBootstrap" => {
            if generation == engine.state.runtime.get(GenerationKey::Home) {
                if result.status.is_ok() {
                    let stale = result
                        .value
                        .get("stale")
                        .and_then(Value::as_bool)
                        .unwrap_or(false);
                    if let Some(categories) = result.value.get("categories").cloned() {
                        engine.state.home.categories = normalize_categories_trailers(categories);
                    }
                    if let Some(continue_watching) = result.value.get("continueWatching").cloned() {
                        engine.state.home.continue_watching = continue_watching;
                    }
                    if let Some(watchlist) = result.value.get("watchlist").cloned() {
                        engine.state.home.watchlist = watchlist;
                    }
                    if let Some(user_addons) = result.value.get("userAddons").cloned() {
                        engine.state.home.user_addons = user_addons;
                    }
                    if let Some(metadata_feeds) = result.value.get("metadataFeeds").cloned() {
                        engine.state.home.metadata_feeds = metadata_feeds;
                    }
                    if let Some(billboard) = result.value.get("billboard").cloned() {
                        engine.state.home.billboard = with_normalized_meta_trailers(billboard);
                    }
                    engine.state.home.is_stale = stale;
                    engine.state.home.is_loading = stale;
                    engine.state.home.error = Value::Null;
                } else {
                    engine.state.home.is_loading = false;
                    engine.state.home.is_stale = false;
                    engine.state.home.error = normalize_error(result.error.clone());
                }
            }
        }
        "prepareDirectPlayback" => {
            if generation == engine.state.runtime.get(GenerationKey::PlaybackPrep) {
                engine.state.home.is_direct_loading = false;
                if result.status.is_ok() {
                    player::complete_direct_playback(engine, result.value.clone(), Value::Null);
                } else {
                    player::complete_direct_playback(
                        engine,
                        Value::Null,
                        Value::String(error_code(&result.error)),
                    );
                }
            }
        }
        "fetchCatalogPage" => {
            if generation == engine.state.runtime.get(GenerationKey::Home) {
                engine.state.home.paging.is_loading = false;
                if result.status.is_ok() {
                    engine.state.home.paging.items = result
                        .value
                        .get("items")
                        .cloned()
                        .unwrap_or_else(|| result.value.clone());
                    engine.state.home.paging.error = Value::Null;
                } else {
                    engine.state.home.paging.error = normalize_error(result.error.clone());
                }
            }
        }
        _ => {}
    }
    vec![]
}
