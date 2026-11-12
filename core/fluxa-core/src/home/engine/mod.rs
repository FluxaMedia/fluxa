use crate::headless_engine::helpers::{
    active_profile_id, error_code, normalize_error, with_normalized_meta_trailers,
};
use crate::headless_engine::player;
use crate::headless_engine::state::GenerationKey;
use crate::headless_engine::{EffectResultInput, HeadlessEngine};
use crate::runtime::{EffectEnvelope, EffectKind};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;

mod progressive;

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", default)]
pub(crate) struct HomeState {
    is_loading: bool,
    is_stale: bool,
    categories: Vec<Arc<Value>>,
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
    pending_feeds: u32,
    continue_pending: bool,
    #[serde(skip)]
    language: String,
    #[serde(skip)]
    plan: Option<progressive::HomePlan>,
}

impl Default for HomeState {
    fn default() -> Self {
        Self {
            is_loading: false,
            is_stale: false,
            categories: Vec::new(),
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
            pending_feeds: 0,
            continue_pending: false,
            language: String::new(),
            plan: None,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", default)]
pub(crate) struct HomePaging {
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
    progressive: bool,
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

fn normalized_row(mut row: Value) -> Arc<Value> {
    if let Some(items) = row.get_mut("items").and_then(Value::as_array_mut) {
        for item in items {
            *item = with_normalized_meta_trailers(item.take());
        }
    }
    Arc::new(row)
}

fn normalized_rows(categories: Value) -> Vec<Arc<Value>> {
    match categories {
        Value::Array(rows) => rows.into_iter().map(normalized_row).collect(),
        _ => Vec::new(),
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct HomeRest<'a> {
    is_loading: bool,
    is_stale: bool,
    continue_watching: &'a Value,
    user_addons: &'a Value,
    metadata_feeds: &'a Value,
    billboard: &'a Value,
    watchlist: &'a Value,
    is_direct_loading: bool,
    active_profile: &'a Value,
    external_continue_watching: &'a Value,
    paging: &'a HomePaging,
    error: &'a Value,
    generation: u64,
    pending_feeds: u32,
    continue_pending: bool,
}

impl HomeState {
    pub(crate) fn slice(&self) -> crate::snapshot::HomeSlice {
        let rest = HomeRest {
            is_loading: self.is_loading,
            is_stale: self.is_stale,
            continue_watching: &self.continue_watching,
            user_addons: &self.user_addons,
            metadata_feeds: &self.metadata_feeds,
            billboard: &self.billboard,
            watchlist: &self.watchlist,
            is_direct_loading: self.is_direct_loading,
            active_profile: &self.active_profile,
            external_continue_watching: &self.external_continue_watching,
            paging: &self.paging,
            error: &self.error,
            generation: self.generation,
            pending_feeds: self.pending_feeds,
            continue_pending: self.continue_pending,
        };
        crate::snapshot::HomeSlice::new(
            serde_json::to_value(rest).unwrap_or(Value::Null),
            self.categories.clone(),
        )
    }
}

fn add_continue_watching_episode_labels(mut items: Value) -> Value {
    let Some(items) = items.as_array_mut() else {
        return items;
    };
    for item in items.iter_mut() {
        let label = crate::library::state::format_episode_line_json(
            item.get("lastEpisodeName").and_then(Value::as_str),
            item.get("lastEpisodeSeason").and_then(Value::as_i64),
            item.get("lastEpisodeNumber").and_then(Value::as_i64),
            item.get("lastVideoId").and_then(Value::as_str),
        );
        if let Some(object) = item.as_object_mut() {
            if label.is_empty() {
                object.remove("episodeLabel");
            } else {
                object.insert("episodeLabel".to_owned(), Value::String(label));
            }
        }
    }
    Value::Array(items.clone())
}

#[cfg(test)]
mod tests;

pub(crate) fn dispatch_refresh_continue_watching(
    engine: &mut HeadlessEngine,
    profile: Option<Value>,
    language: Option<String>,
    source: Option<String>,
) -> Vec<EffectEnvelope> {
    let profile_value = profile.unwrap_or_else(|| engine.state.profile.active.clone());
    let profile_id = active_profile_id(&engine.state, &profile_value);
    let generation = engine.bump_generation(GenerationKey::Home);
    vec![
        engine.effect(
            EffectKind::RefreshContinueWatching,
            generation,
            RefreshContinueWatchingPayload {
                profile_id,
                profile: profile_value,
                language: language
                    .or_else(|| Some(engine.state.home.language.clone()).filter(|l| !l.is_empty()))
                    .unwrap_or_else(|| "en".to_string()),
                source,
            },
        ),
    ]
}

pub(crate) fn remove_from_continue_watching(engine: &mut HeadlessEngine, dropped_id: &str) {
    if let Some(items) = engine.state.home.continue_watching.as_array_mut() {
        items.retain(|item| item.get("id").and_then(Value::as_str) != Some(dropped_id));
    }
}

pub(crate) fn mirror_active_profile(engine: &mut HeadlessEngine, profile: Value) {
    engine.state.home.active_profile = profile;
}

pub(crate) fn set_user_addons(engine: &mut HeadlessEngine, addons: Value) {
    engine.state.home.user_addons = addons;
}

pub(crate) fn reload(engine: &mut HeadlessEngine) -> Vec<EffectEnvelope> {
    let language = Some(engine.state.home.language.clone()).filter(|l| !l.is_empty());
    dispatch_load(engine, None, language, Some(true))
}

pub(crate) fn dispatch_load(
    engine: &mut HeadlessEngine,
    profile: Option<Value>,
    language: Option<String>,
    force: Option<bool>,
) -> Vec<EffectEnvelope> {
    let generation = engine.bump_generation(GenerationKey::Home);
    let profile_value = profile.unwrap_or_else(|| engine.state.profile.active.clone());
    if !profile_value.is_null() {
        // Home-load callers carry the selected profile explicitly. Mirror it
        // into ambient Core state as well so snapshot consumers (including
        // native renderers) see the same profile preferences as platform UI.
        crate::headless_engine::profile::update_active(engine, profile_value.clone());
    }
    let profile_id = active_profile_id(&engine.state, &profile_value);
    let force = force.unwrap_or(false);
    if force {
        let mut home = (*engine.state.home).clone();
        home.is_loading = true;
        home.is_stale = false;
        home.error = Value::Null;
        home.generation = generation;
        home.plan = None;
        *engine.state.home = home;
    } else {
        *engine.state.home = HomeState {
            is_loading: true,
            generation,
            ..HomeState::default()
        };
    }
    let language_value = language.unwrap_or_else(|| "en".to_string());
    engine.state.home.language = language_value.clone();
    engine.state.home.continue_pending = true;
    let bootstrap = engine.effect(
        EffectKind::ReadHomeBootstrap,
        generation,
        ReadHomeBootstrapPayload {
            profile_id: profile_id.clone(),
            profile: profile_value.clone(),
            language: language_value.clone(),
            force,
            progressive: true,
        },
    );
    let continue_watching = engine.effect(
        EffectKind::RefreshContinueWatching,
        generation,
        RefreshContinueWatchingPayload {
            profile_id,
            profile: profile_value,
            language: language_value,
            source: None,
        },
    );
    vec![bootstrap, continue_watching]
}

pub(crate) fn dispatch_direct_playback(
    engine: &mut HeadlessEngine,
    meta: Value,
    language: Option<String>,
    profile: Option<Value>,
) -> Vec<EffectEnvelope> {
    let generation = engine.bump_generation(GenerationKey::PlaybackPrep);
    engine.state.home.is_direct_loading = true;
    player::reset_for_direct_playback(engine);
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
pub(crate) fn dispatch_catalog_page(
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
    let generation = engine.bump_generation(GenerationKey::HomePaging);
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

// A stale bootstrap is the cached copy rendered while the real one is still
// missing; without this follow-up the cache would be all the client ever sees.
fn revalidate(engine: &mut HeadlessEngine) -> Vec<EffectEnvelope> {
    let generation = engine.bump_generation(GenerationKey::Home);
    let profile_value = engine.state.profile.active.clone();
    let profile_id = active_profile_id(&engine.state, &profile_value);
    let language = engine.state.home.language.clone();
    engine.state.home.generation = generation;
    vec![engine.effect(
        EffectKind::ReadHomeBootstrap,
        generation,
        ReadHomeBootstrapPayload {
            profile_id,
            profile: profile_value,
            language: if language.is_empty() {
                "en".to_string()
            } else {
                language
            },
            force: true,
            progressive: true,
        },
    )]
}

fn item_key(item: &Value) -> (Option<String>, Option<String>) {
    let text = |key| item.get(key).and_then(Value::as_str).map(str::to_owned);
    (text("id"), text("type"))
}

fn append_page(categories: &mut [Arc<Value>], category_id: &str, page: &[Value]) {
    let Some(row) = categories
        .iter_mut()
        .find(|row| row.get("id").and_then(Value::as_str) == Some(category_id))
    else {
        return;
    };
    let mut seen: std::collections::HashSet<(Option<String>, Option<String>)> = row
        .get("items")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .map(item_key)
        .collect();
    let fresh: Vec<&Value> = page
        .iter()
        .filter(|item| seen.insert(item_key(item)))
        .collect();
    let can_load_more = Value::Bool(!fresh.is_empty());
    if fresh.is_empty() && row.get("canLoadMore") == Some(&can_load_more) {
        return;
    }
    let Some(object) = Arc::make_mut(row).as_object_mut() else {
        return;
    };
    let existing = object
        .entry("items")
        .or_insert_with(|| Value::Array(Vec::new()));
    if !existing.is_array() {
        *existing = Value::Array(Vec::new());
    }
    if let Some(list) = existing.as_array_mut() {
        list.extend(fresh.into_iter().cloned());
    }
    object.insert("canLoadMore".to_owned(), can_load_more);
}

pub(crate) fn complete(
    engine: &mut HeadlessEngine,
    effect_type: &str,
    generation: u64,
    result: &EffectResultInput,
    effect_payload: &Value,
) -> Vec<EffectEnvelope> {
    if effect_type == "fetchCatalogPage" {
        if result.status.is_ok() {
            let category_id = effect_payload
                .get("categoryId")
                .and_then(Value::as_str)
                .unwrap_or_default();
            let items = result
                .value
                .get("items")
                .cloned()
                .unwrap_or_else(|| result.value.clone());
            if let Some(page) = items.as_array() {
                append_page(&mut engine.state.home.categories, category_id, page);
            }
            if engine.state.home.paging.category_id == category_id {
                engine.state.home.paging.is_loading = false;
                engine.state.home.paging.items = items;
                engine.state.home.paging.error = Value::Null;
            }
        } else {
            let category_id = effect_payload
                .get("categoryId")
                .and_then(Value::as_str)
                .unwrap_or_default();
            let error = normalize_error(result.error.clone());
            if let Some(category) = engine
                .state
                .home
                .categories
                .iter_mut()
                .find(|category| {
                    category.get("id").and_then(Value::as_str) == Some(category_id)
                        && category.get("type").and_then(Value::as_str)
                            == Some("collection_folder_source")
                })
                .map(Arc::make_mut)
                .and_then(Value::as_object_mut)
            {
                category.insert("canLoadMore".to_owned(), Value::Bool(false));
                category.insert("error".to_owned(), error.clone());
            }
            if engine.state.home.paging.category_id == category_id {
                engine.state.home.paging.is_loading = false;
                engine.state.home.paging.error = error;
            }
        }
        return vec![];
    }
    match effect_type {
        "refreshContinueWatching" => {
            if result.status.is_ok()
                && let Some(cw) = result.value.get("continueWatching")
            {
                engine.state.home.continue_watching =
                    add_continue_watching_episode_labels(cw.clone());
            }
            return progressive::continue_done(engine, result);
        }
        "fetchHomeFeed" => {
            return progressive::feed_done(engine, generation, result, effect_payload);
        }
        "fetchHomeHeroRatings" => {
            return progressive::ratings_done(engine, generation, result, effect_payload);
        }
        "readHomeBootstrap" => {
            if generation == engine.state.runtime.get(GenerationKey::Home) {
                if result.status.is_ok() && result.value.get("plan").is_some() {
                    return progressive::begin(engine, generation, result, effect_payload);
                }
                if result.status.is_ok() {
                    let stale = result
                        .value
                        .get("stale")
                        .and_then(Value::as_bool)
                        .unwrap_or(false);
                    if let Some(categories) = result.value.get("categories").cloned() {
                        engine.state.home.categories = normalized_rows(categories);
                    }
                    if let Some(continue_watching) = result.value.get("continueWatching").cloned() {
                        engine.state.home.continue_watching =
                            add_continue_watching_episode_labels(continue_watching);
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
                    if stale {
                        return revalidate(engine);
                    }
                } else {
                    engine.state.home.is_loading = false;
                    engine.state.home.is_stale = false;
                    engine.state.home.error = normalize_error(result.error.clone());
                }
            }
        }
        "prepareDirectPlayback"
            if generation == engine.state.runtime.get(GenerationKey::PlaybackPrep) =>
        {
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
        _ => {}
    }
    vec![]
}
