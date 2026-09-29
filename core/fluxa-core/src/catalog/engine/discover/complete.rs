use super::*;

pub(crate) fn complete(
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
                    let filters = &engine.state.discover.filters;
                    let has_source = filters
                        .get("transportUrl")
                        .and_then(Value::as_str)
                        .is_some()
                        && filters.get("catalogId").and_then(Value::as_str).is_some();
                    let result_count = engine.state.discover.results.as_array().map_or(0, Vec::len);
                    engine.state.discover.paging.next_skip = result_count as i32;
                    engine.state.discover.paging.has_more = has_source && result_count > 0;
                } else {
                    engine.state.discover.error = normalize_error(result.error.clone());
                }
            }
        }
        "readDiscoverCatalogFilters" => {
            if generation == engine.state.runtime.get(GenerationKey::DiscoverFilters) {
                engine.state.discover.catalogs_loading = false;
                if result.status.is_ok() {
                    let addons = result
                        .value
                        .get("addons")
                        .filter(|value| value.is_array())
                        .cloned()
                        .unwrap_or_else(|| serde_json::json!([]));
                    let addons_json = addons.to_string();
                    let catalogs = crate::catalog::search::discover_catalog_options_json(
                        &addons_json,
                        &engine.state.discover.content_type,
                    )
                    .and_then(|raw| serde_json::from_str(&raw).ok())
                    .unwrap_or_else(|| serde_json::json!([]));
                    let content_types =
                        crate::catalog::search::discover_content_types_json(&addons_json)
                            .and_then(|raw| serde_json::from_str(&raw).ok())
                            .unwrap_or_else(|| serde_json::json!(["movie", "series"]));
                    engine.state.discover.catalogs = catalogs.clone();
                    let selected_catalog = catalogs
                        .as_array()
                        .into_iter()
                        .flatten()
                        .find(|catalog| {
                            catalog.get("key").and_then(Value::as_str)
                                == engine.state.discover.selected_catalog_key.as_deref()
                        })
                        .or_else(|| catalogs.as_array()?.first());
                    engine.state.discover.genres = selected_catalog
                        .map(|catalog| {
                            let mut genres = Vec::new();
                            if catalog.get("requiresGenre").and_then(Value::as_bool) != Some(true) {
                                genres.push(serde_json::json!({"id": null, "label": ""}));
                            }
                            genres.extend(
                                catalog
                                    .get("genres")
                                    .and_then(Value::as_array)
                                    .into_iter()
                                    .flatten()
                                    .filter_map(Value::as_str)
                                    .map(|genre| serde_json::json!({"id": genre, "label": genre})),
                            );
                            Value::Array(genres)
                        })
                        .unwrap_or_else(|| serde_json::json!([]));
                    engine.state.discover.content_types = content_types;
                    engine.state.discover.error = Value::Null;
                    if let Some(mut pending) =
                        engine.state.discover.pending_discover_after_catalogs.take()
                    {
                        let selected_key = engine
                            .state
                            .discover
                            .catalogs
                            .as_array()
                            .into_iter()
                            .flatten()
                            .find(|catalog| {
                                catalog.get("key").and_then(Value::as_str)
                                    == pending.filters.get("catalogKey").and_then(Value::as_str)
                            })
                            .or_else(|| engine.state.discover.catalogs.as_array()?.first())
                            .and_then(|catalog| catalog.get("key"))
                            .and_then(Value::as_str)
                            .map(str::to_owned);
                        if let Some(selected_key) = selected_key {
                            if let Some(filters) = pending.filters.as_object_mut() {
                                filters.insert(
                                    "catalogKey".to_owned(),
                                    Value::String(selected_key.clone()),
                                );
                            }
                            engine.state.discover.selected_catalog_key = Some(selected_key);
                            let content_type = engine.state.discover.content_type.clone();
                            return dispatch_discover(
                                engine,
                                content_type,
                                Some(pending.filters),
                                Some(pending.profile),
                                Some(pending.language),
                                false,
                            );
                        } else {
                            engine.state.discover.is_loading = false;
                        }
                    }
                } else {
                    engine.state.discover.is_loading = false;
                    engine.state.discover.error = normalize_error(result.error.clone());
                    engine.state.discover.pending_discover_after_catalogs = None;
                }
            }
        }
        "fetchDiscoverPage" => {
            if generation == engine.state.runtime.get(GenerationKey::DiscoverPaging) {
                engine.state.discover.last_page_appended.clear();
                engine.state.discover.last_page_sources = serde_json::json!({});
                engine.state.discover.paging.is_loading = false;
                if result.status.is_ok() {
                    let items = result
                        .value
                        .get("items")
                        .cloned()
                        .unwrap_or_else(|| serde_json::json!([]));
                    engine.state.discover.paging.items = items.clone();
                    engine.state.discover.paging.error = Value::Null;
                    let page_items = items.as_array().cloned().unwrap_or_default();
                    let appended = page_items.clone();
                    let discover = &mut engine.state.discover;
                    if let Some(results) = discover.results.as_array_mut() {
                        results.extend(page_items.iter().cloned());
                    }
                    let filters = &engine.state.discover.filters;
                    let content_type = engine.state.discover.content_type.clone();
                    let source = serde_json::json!({
                        "transportUrl": filters.get("transportUrl"),
                        "catalogId": filters.get("catalogId"),
                        "type": content_type,
                        "genre": filters.pointer("/extra/genre"),
                    });
                    let mut page_sources = serde_json::Map::new();
                    if let Some(sources) = engine.state.discover.result_sources.as_object_mut() {
                        for item in &appended {
                            if let Some(id) = item.get("id").and_then(Value::as_str) {
                                let item_type = item
                                    .get("type")
                                    .and_then(Value::as_str)
                                    .unwrap_or(&content_type);
                                let typed_id = format!("{item_type}:{id}");
                                sources.entry(typed_id.clone()).or_insert(source.clone());
                                sources.entry(id.to_owned()).or_insert(source.clone());
                                page_sources.entry(typed_id).or_insert(source.clone());
                                page_sources.entry(id.to_owned()).or_insert(source.clone());
                            }
                        }
                    }
                    engine.state.discover.last_page_appended = appended;
                    engine.state.discover.last_page_sources = Value::Object(page_sources);
                    engine.state.discover.paging.next_skip = engine
                        .state
                        .discover
                        .paging
                        .requested_skip
                        .saturating_add(page_items.len() as i32);
                    engine.state.discover.paging.has_more = !page_items.is_empty();
                } else {
                    engine.state.discover.paging.error = normalize_error(result.error.clone());
                }
            }
        }
        _ => {}
    }
    vec![]
}
