use super::helpers::{active_profile_id, normalize_error};
use super::state::GenerationKey;
use super::{EffectResultInput, HeadlessEngine};
use crate::runtime::{EffectEnvelope, EffectKind};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", default)]
pub(super) struct DiscoverState {
    content_type: String,
    filters: Value,
    is_loading: bool,
    catalogs_loading: bool,
    results: Value,
    result_sources: Value,
    catalogs: Value,
    genres: Value,
    content_types: Value,
    error: Value,
    generation: u64,
    paging: DiscoverPaging,
    #[serde(skip)]
    initial_paging: InitialDiscoverPaging,
}

#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", default)]
pub(super) struct DiscoverPaging {
    is_loading: bool,
    items: Value,
    error: Value,
}

#[derive(Clone, Debug, Default, PartialEq)]
struct InitialDiscoverPaging {
    transport_url: String,
    content_type: String,
    catalog_id: String,
    genre: Option<String>,
    next_skip: i32,
    remaining_pages: u8,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct FetchDiscoverPagePayload {
    transport_url: Option<String>,
    content_type: String,
    catalog_id: String,
    skip: i32,
    genre: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct RunDiscoverPayload {
    content_type: String,
    filters: Value,
    profile_id: String,
    profile: Value,
    language: String,
}

fn selected_catalog_page_source(
    catalogs: &Value,
    catalog_key: Option<&str>,
) -> (Option<String>, Option<String>) {
    let Some(catalog_key) = catalog_key else {
        return (None, None);
    };
    let Some(catalog) = catalogs.as_array().and_then(|catalogs| {
        catalogs
            .iter()
            .find(|catalog| catalog.get("key").and_then(Value::as_str) == Some(catalog_key))
    }) else {
        return (None, None);
    };
    (
        catalog
            .get("transportUrl")
            .and_then(Value::as_str)
            .map(str::to_owned),
        catalog.get("id").and_then(Value::as_str).map(str::to_owned),
    )
}

fn initial_paging(filters: &Value, catalogs: &Value, content_type: &str) -> InitialDiscoverPaging {
    let catalog_key = filters.get("catalogKey").and_then(Value::as_str);
    let (transport_url, catalog_id) = selected_catalog_page_source(catalogs, catalog_key);
    match (transport_url, catalog_id) {
        (Some(transport_url), Some(catalog_id)) => InitialDiscoverPaging {
            transport_url,
            content_type: content_type.to_string(),
            catalog_id,
            genre: filters
                .get("extra")
                .and_then(|extra| extra.get("genre"))
                .and_then(Value::as_str)
                .map(str::to_owned),
            next_skip: 20,
            remaining_pages: 2,
        },
        _ => InitialDiscoverPaging::default(),
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ReadDiscoverCatalogFiltersPayload {
    content_type: String,
    selected_catalog_key: Option<String>,
    profile_id: String,
    profile: Value,
    language: String,
}

pub(super) fn dispatch_discover(
    engine: &mut HeadlessEngine,
    content_type: String,
    filters: Option<Value>,
    profile: Option<Value>,
    language: Option<String>,
) -> Vec<EffectEnvelope> {
    let generation = engine.bump_generation(GenerationKey::Discover);
    engine.bump_generation(GenerationKey::DiscoverPaging);
    let profile_value = profile.unwrap_or_else(|| engine.state.profile.active.clone());
    let profile_id = active_profile_id(&engine.state, &profile_value);
    let filters_value = filters.unwrap_or(Value::Null);
    let initial_paging = initial_paging(
        &filters_value,
        &engine.state.discover.catalogs,
        &content_type,
    );
    *engine.state.discover = DiscoverState {
        content_type: content_type.clone(),
        filters: filters_value.clone(),
        is_loading: true,
        catalogs_loading: engine.state.discover.catalogs_loading,
        results: serde_json::json!([]),
        result_sources: Value::Null,
        catalogs: engine.state.discover.catalogs.clone(),
        genres: engine.state.discover.genres.clone(),
        content_types: engine.state.discover.content_types.clone(),
        error: Value::Null,
        generation,
        paging: DiscoverPaging::default(),
        initial_paging,
    };
    vec![engine.effect(
        EffectKind::RunDiscover,
        generation,
        RunDiscoverPayload {
            content_type,
            filters: filters_value,
            profile_id,
            profile: profile_value,
            language: language.unwrap_or_else(|| "en".to_string()),
        },
    )]
}

pub(super) fn dispatch_catalog_filters(
    engine: &mut HeadlessEngine,
    content_type: String,
    selected_catalog_key: Option<String>,
    profile: Option<Value>,
    language: Option<String>,
) -> Vec<EffectEnvelope> {
    let generation = engine.bump_generation(GenerationKey::DiscoverFilters);
    let profile_value = profile.unwrap_or_else(|| engine.state.profile.active.clone());
    let profile_id = active_profile_id(&engine.state, &profile_value);
    engine.state.discover.content_type = content_type.clone();
    engine.state.discover.catalogs = serde_json::json!([]);
    engine.state.discover.catalogs_loading = true;
    vec![engine.effect(
        EffectKind::ReadDiscoverCatalogFilters,
        generation,
        ReadDiscoverCatalogFiltersPayload {
            content_type,
            selected_catalog_key,
            profile_id,
            profile: profile_value,
            language: language.unwrap_or_else(|| "en".to_string()),
        },
    )]
}

pub(super) fn dispatch_discover_page(
    engine: &mut HeadlessEngine,
    transport_url: Option<String>,
    content_type: String,
    catalog_id: String,
    skip: Option<i32>,
    genre: Option<String>,
) -> Vec<EffectEnvelope> {
    engine.state.discover.initial_paging = InitialDiscoverPaging::default();
    let generation = engine.bump_generation(GenerationKey::DiscoverPaging);
    engine.state.discover.paging = DiscoverPaging {
        is_loading: true,
        items: Value::Null,
        error: Value::Null,
    };
    vec![engine.effect(
        EffectKind::FetchDiscoverPage,
        generation,
        FetchDiscoverPagePayload {
            transport_url,
            content_type,
            catalog_id,
            skip: skip.unwrap_or(0).max(0),
            genre,
        },
    )]
}

fn dispatch_initial_pages(engine: &mut HeadlessEngine) -> Vec<EffectEnvelope> {
    let initial_paging = std::mem::take(&mut engine.state.discover.initial_paging);
    if initial_paging.remaining_pages == 0 {
        return vec![];
    }
    let generation = engine.bump_generation(GenerationKey::DiscoverPaging);
    engine.state.discover.paging = DiscoverPaging {
        is_loading: true,
        items: Value::Null,
        error: Value::Null,
    };
    let mut effects = Vec::new();
    for page in 0..initial_paging.remaining_pages {
        let skip = initial_paging.next_skip + page as i32 * 20;
        effects.push(engine.effect(
            EffectKind::FetchDiscoverPage,
            generation,
            FetchDiscoverPagePayload {
                transport_url: Some(initial_paging.transport_url.clone()),
                content_type: initial_paging.content_type.clone(),
                catalog_id: initial_paging.catalog_id.clone(),
                skip,
                genre: initial_paging.genre.clone(),
            },
        ));
    }
    effects
}

pub(super) fn complete(
    engine: &mut HeadlessEngine,
    effect_type: &str,
    generation: u64,
    result: &EffectResultInput,
) -> Vec<EffectEnvelope> {
    match effect_type {
        "runDiscover" => {
            if generation == engine.state.runtime.get(GenerationKey::Discover) {
                engine.state.discover.is_loading = false;
                if result.status.is_ok() {
                    engine.state.discover.results = result
                        .value
                        .get("results")
                        .cloned()
                        .unwrap_or_else(|| result.value.clone());
                    engine.state.discover.result_sources = result
                        .value
                        .get("resultSources")
                        .cloned()
                        .unwrap_or_else(|| serde_json::json!({}));
                    engine.state.discover.error = Value::Null;
                    return dispatch_initial_pages(engine);
                } else {
                    engine.state.discover.error = normalize_error(result.error.clone());
                }
            }
        }
        "readDiscoverCatalogFilters" => {
            if generation == engine.state.runtime.get(GenerationKey::DiscoverFilters) {
                engine.state.discover.catalogs_loading = false;
                if result.status.is_ok() {
                    engine.state.discover.catalogs = result
                        .value
                        .get("catalogs")
                        .cloned()
                        .unwrap_or_else(|| serde_json::json!([]));
                    engine.state.discover.genres = result
                        .value
                        .get("genres")
                        .cloned()
                        .unwrap_or_else(|| serde_json::json!([]));
                    engine.state.discover.content_types = result
                        .value
                        .get("contentTypes")
                        .cloned()
                        .unwrap_or_else(|| serde_json::json!([]));
                    engine.state.discover.error = Value::Null;
                } else {
                    engine.state.discover.error = normalize_error(result.error.clone());
                }
            }
        }
        "fetchDiscoverPage" => {
            if generation == engine.state.runtime.get(GenerationKey::DiscoverPaging) {
                engine.state.discover.paging.is_loading = false;
                if result.status.is_ok() {
                    let items = result
                        .value
                        .get("items")
                        .cloned()
                        .unwrap_or_else(|| serde_json::json!([]));
                    engine.state.discover.paging.items = items.clone();
                    engine.state.discover.paging.error = Value::Null;
                    if let Some(results) = engine.state.discover.results.as_array_mut() {
                        results.extend(items.as_array().into_iter().flatten().cloned());
                    }
                } else {
                    engine.state.discover.paging.error = normalize_error(result.error.clone());
                }
            }
        }
        _ => {}
    }
    vec![]
}
