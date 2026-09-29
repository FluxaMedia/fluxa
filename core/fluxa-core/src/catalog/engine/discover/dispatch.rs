use super::*;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct FetchDiscoverPagePayload {
    transport_url: Option<String>,
    content_type: String,
    catalog_id: String,
    skip: i32,
    genre: Option<String>,
    search: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct RunDiscoverPayload {
    content_type: String,
    filters: Value,
    profile_id: String,
    profile: Value,
    language: String,
}

pub(super) fn selected_catalog_page_source(
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

pub(super) fn resolve_required_discover_extras(filters: &mut Value, catalogs: &Value) {
    let Some(filters) = filters.as_object_mut() else {
        return;
    };
    let Some(catalog_key) = filters.get("catalogKey").and_then(Value::as_str) else {
        return;
    };
    let Some(catalog) = catalogs
        .as_array()
        .into_iter()
        .flatten()
        .find(|catalog| catalog.get("key").and_then(Value::as_str) == Some(catalog_key))
    else {
        return;
    };
    let Some(extras) = catalog.get("extras").and_then(Value::as_array) else {
        return;
    };
    let extra_filters = filters
        .entry("extra".to_owned())
        .or_insert_with(|| serde_json::json!({}));
    let Some(extra_filters) = extra_filters.as_object_mut() else {
        return;
    };
    for extra in extras
        .iter()
        .filter(|extra| extra.get("isRequired").and_then(Value::as_bool) == Some(true))
    {
        let Some(name) = extra.get("name").and_then(Value::as_str) else {
            continue;
        };
        let has_value = extra_filters
            .get(name)
            .and_then(Value::as_str)
            .is_some_and(|value| !value.trim().is_empty());
        if has_value {
            continue;
        }
        let options = extra.get("options").and_then(Value::as_array);
        let is_valid_option = |candidate: &str| {
            options.is_some_and(|options| {
                options
                    .iter()
                    .any(|option| option.as_str() == Some(candidate))
            })
        };
        let default = extra
            .get("default")
            .and_then(Value::as_str)
            .filter(|value| is_valid_option(value));
        let first = options.into_iter().flatten().find_map(Value::as_str);
        if let Some(value) = default.or(first) {
            extra_filters.insert(name.to_owned(), Value::String(value.to_owned()));
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ReadDiscoverCatalogFiltersPayload {
    profile: Value,
}

pub(crate) fn dispatch_discover(
    engine: &mut HeadlessEngine,
    content_type: String,
    filters: Option<Value>,
    profile: Option<Value>,
    language: Option<String>,
    load_catalog_filters: bool,
) -> Vec<EffectEnvelope> {
    let content_type = if content_type.trim().is_empty() {
        "movie".to_owned()
    } else {
        content_type
    };
    if load_catalog_filters {
        engine.bump_generation(GenerationKey::Discover);
        engine.bump_generation(GenerationKey::DiscoverPaging);
        let mut filters_value = filters.unwrap_or_else(|| serde_json::json!({}));
        let selected_catalog_key = filters_value
            .get("catalogKey")
            .and_then(Value::as_str)
            .map(str::to_owned);
        let profile_value = profile.unwrap_or_else(|| engine.state.profile.active.clone());
        let language_value = language.unwrap_or_else(|| "en".to_owned());
        let effects = dispatch_catalog_filters(
            engine,
            content_type,
            selected_catalog_key,
            Some(profile_value.clone()),
        );
        if !filters_value.is_object() {
            filters_value = serde_json::json!({});
        }
        engine.state.discover.filters = filters_value.clone();
        engine.state.discover.results = serde_json::json!([]);
        engine.state.discover.is_loading = true;
        engine.state.discover.pending_discover_after_catalogs =
            Some(PendingDiscoverAfterCatalogs {
                filters: filters_value,
                profile: profile_value,
                language: language_value,
            });
        return effects;
    }
    engine.bump_generation(GenerationKey::DiscoverFilters);
    engine.state.discover.pending_discover_after_catalogs = None;
    let generation = engine.bump_generation(GenerationKey::Discover);
    engine.bump_generation(GenerationKey::DiscoverPaging);
    let profile_value = profile.unwrap_or_else(|| engine.state.profile.active.clone());
    let profile_id = active_profile_id(&engine.state, &profile_value);
    let mut filters_value = filters.unwrap_or(Value::Null);
    resolve_required_discover_extras(&mut filters_value, &engine.state.discover.catalogs);
    if let Some(filters) = filters_value.as_object_mut() {
        let catalog_key = filters
            .get("catalogKey")
            .and_then(Value::as_str)
            .map(str::to_owned);
        let (transport_url, catalog_id) =
            selected_catalog_page_source(&engine.state.discover.catalogs, catalog_key.as_deref());
        if let Some(transport_url) = transport_url {
            filters.insert("transportUrl".to_owned(), Value::String(transport_url));
        }
        if let Some(catalog_id) = catalog_id {
            filters.insert("catalogId".to_owned(), Value::String(catalog_id));
        }
        if let Some(catalog_type) = engine
            .state
            .discover
            .catalogs
            .as_array()
            .into_iter()
            .flatten()
            .find(|catalog| catalog.get("key").and_then(Value::as_str) == catalog_key.as_deref())
            .and_then(|catalog| catalog.get("type"))
            .and_then(Value::as_str)
        {
            filters.insert(
                "catalogType".to_owned(),
                Value::String(catalog_type.to_owned()),
            );
        }
    }
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
        selected_catalog_key: engine.state.discover.selected_catalog_key.clone(),
        error: Value::Null,
        generation,
        paging: DiscoverPaging::default(),
        last_page_appended: Vec::new(),
        last_page_sources: serde_json::json!({}),
        pending_discover_after_catalogs: None,
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

pub(crate) fn dispatch_catalog_filters(
    engine: &mut HeadlessEngine,
    content_type: String,
    selected_catalog_key: Option<String>,
    profile: Option<Value>,
) -> Vec<EffectEnvelope> {
    let content_type = if content_type.trim().is_empty() {
        "movie".to_owned()
    } else {
        content_type
    };
    let generation = engine.bump_generation(GenerationKey::DiscoverFilters);
    let profile_value = profile.unwrap_or_else(|| engine.state.profile.active.clone());
    engine.state.discover.content_type = content_type.clone();
    engine.state.discover.selected_catalog_key = selected_catalog_key.clone();
    engine.state.discover.catalogs = serde_json::json!([]);
    engine.state.discover.catalogs_loading = true;
    engine.state.discover.pending_discover_after_catalogs = None;
    vec![engine.effect(
        EffectKind::ReadDiscoverCatalogFilters,
        generation,
        ReadDiscoverCatalogFiltersPayload {
            profile: profile_value,
        },
    )]
}

pub(crate) fn dispatch_discover_page(
    engine: &mut HeadlessEngine,
    transport_url: Option<String>,
    content_type: String,
    catalog_id: String,
    skip: Option<i32>,
    genre: Option<String>,
    search: Option<String>,
) -> Vec<EffectEnvelope> {
    let paging = &engine.state.discover.paging;
    let requested_skip = skip.unwrap_or(0).max(0);
    let filters = &engine.state.discover.filters;
    if paging.is_loading
        || !paging.has_more
        || !paging.error.is_null()
        || requested_skip != paging.next_skip
        || filters.get("transportUrl").and_then(Value::as_str) != transport_url.as_deref()
        || filters.get("catalogId").and_then(Value::as_str) != Some(catalog_id.as_str())
        || engine.state.discover.content_type != content_type
    {
        return vec![];
    }
    let generation = engine.bump_generation(GenerationKey::DiscoverPaging);
    engine.state.discover.paging.is_loading = true;
    engine.state.discover.paging.items = Value::Null;
    engine.state.discover.paging.requested_skip = requested_skip;
    vec![engine.effect(
        EffectKind::FetchDiscoverPage,
        generation,
        FetchDiscoverPagePayload {
            transport_url,
            content_type,
            catalog_id,
            skip: requested_skip,
            genre,
            search,
        },
    )]
}
