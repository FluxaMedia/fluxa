use super::*;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct DiscoverCatalog {
    pub key: String,
    pub label: String,
    pub content_type: String,
    pub extras: Vec<(String, Vec<String>)>,
    pub required_extras: Vec<String>,
}

#[derive(Clone, Debug, Default)]
pub struct DiscoverModel {
    pub language: String,
    pub query: String,
    pub content_type: String,
    pub content_types: Vec<String>,
    pub selected_catalog_key: String,
    pub selected_extra_name: String,
    pub selected_extra_value: String,
    pub catalogs: Vec<DiscoverCatalog>,
    pub results: Vec<HomeCard>,
    pub generation: u64,
    pub next_page: Option<serde_json::Value>,
    pub is_loading: bool,
    pub catalogs_loading: bool,
    pub error: Option<String>,
    pub sections: Vec<DiscoverSection>,
}

#[derive(Clone, Debug, Default)]
pub struct DiscoverSection {
    pub title: String,
    pub start: usize,
    pub len: usize,
}

pub fn discover_model_from_core_snapshot(snapshot: &serde_json::Value) -> DiscoverModel {
    let mut model = discover_catalog_model(snapshot);
    let search = snapshot.get("search").unwrap_or(&serde_json::Value::Null);
    let query = value_string(search, "query").unwrap_or_default();
    if !query.trim().is_empty() {
        model.results.clear();
        model.sections.clear();
        for category in search
            .get("categories")
            .and_then(serde_json::Value::as_array)
            .into_iter()
            .flatten()
        {
            let items = category
                .get("items")
                .and_then(serde_json::Value::as_array)
                .map(Vec::as_slice)
                .unwrap_or_default();
            if items.is_empty() {
                continue;
            }
            let name = value_string(category, "name").unwrap_or_default();
            let addon = value_string(category, "addonName").unwrap_or_default();
            let title = if addon.is_empty() || name.contains(&addon) {
                name
            } else if name.is_empty() {
                addon
            } else {
                format!("{name} · {addon}")
            };
            model.sections.push(DiscoverSection {
                title,
                start: model.results.len(),
                len: items.len(),
            });
            model.results.extend(items.iter().map(core_home_card));
        }
        model.is_loading = search
            .get("isLoading")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false);
        model.error = search
            .get("error")
            .and_then(serde_json::Value::as_str)
            .map(ToOwned::to_owned);
        model.next_page = None;
        model.generation = search
            .get("generation")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0)
            | 1 << 63;
        model.query = query;
    }
    model
}

pub(crate) fn discover_catalog_model(snapshot: &serde_json::Value) -> DiscoverModel {
    let discover = snapshot.get("discover").unwrap_or(&serde_json::Value::Null);
    let filters = discover.get("filters").unwrap_or(&serde_json::Value::Null);
    let paging = discover.get("paging").unwrap_or(&serde_json::Value::Null);
    let next_page = if paging.get("hasMore").and_then(serde_json::Value::as_bool) == Some(true)
        && paging.get("isLoading").and_then(serde_json::Value::as_bool) != Some(true)
        && paging.get("error").is_none_or(serde_json::Value::is_null)
    {
        filters
            .get("transportUrl")
            .and_then(serde_json::Value::as_str)
            .zip(filters.get("catalogId").and_then(serde_json::Value::as_str))
            .map(|(transport_url, catalog_id)| {
                serde_json::json!({
                    "type": "discoverPageRequested",
                    "transportUrl": transport_url,
                    "contentType": discover.get("contentType"),
                    "catalogId": catalog_id,
                    "skip": paging.get("nextSkip"),
                    "genre": filters.pointer("/extra/genre"),
                    "search": filters.pointer("/extra/search"),
                })
            })
    } else {
        None
    };
    let catalogs = discover
        .get("catalogs")
        .and_then(serde_json::Value::as_array)
        .map(|items| {
            items
                .iter()
                .take(24)
                .filter_map(|item| {
                    let key = value_string(item, "key")?;
                    Some(DiscoverCatalog {
                        label: first_value_string(item, &["label", "name", "id"])
                            .unwrap_or_else(|| "Catalog".to_owned()),
                        key,
                        content_type: value_string(item, "type")
                            .unwrap_or_else(|| "movie".to_owned()),
                        extras: item
                            .get("extras")
                            .and_then(serde_json::Value::as_array)
                            .map(|extras| {
                                extras
                                    .iter()
                                    .filter_map(|extra| {
                                        let name = value_string(extra, "name")?;
                                        let options = extra
                                            .get("options")
                                            .and_then(serde_json::Value::as_array)?
                                            .iter()
                                            .filter_map(serde_json::Value::as_str)
                                            .map(ToOwned::to_owned)
                                            .collect::<Vec<_>>();
                                        Some((name, options))
                                    })
                                    .collect()
                            })
                            .unwrap_or_default(),
                        required_extras: item
                            .get("extras")
                            .and_then(serde_json::Value::as_array)
                            .into_iter()
                            .flatten()
                            .filter(|extra| {
                                extra.get("isRequired").and_then(serde_json::Value::as_bool)
                                    == Some(true)
                            })
                            .filter_map(|extra| value_string(extra, "name"))
                            .collect(),
                    })
                })
                .collect()
        })
        .unwrap_or_default();
    DiscoverModel {
        language: snapshot_language(snapshot),
        content_type: value_string(discover, "contentType").unwrap_or_else(|| "movie".to_owned()),
        content_types: discover
            .get("contentTypes")
            .and_then(serde_json::Value::as_array)
            .map(|items| {
                items
                    .iter()
                    .filter_map(serde_json::Value::as_str)
                    .map(ToOwned::to_owned)
                    .collect()
            })
            .unwrap_or_default(),
        selected_catalog_key: discover
            .pointer("/filters/catalogKey")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .to_owned(),
        selected_extra_name: discover
            .pointer("/filters/extra")
            .and_then(serde_json::Value::as_object)
            .and_then(|extra| extra.keys().find(|key| key.as_str() != "search"))
            .cloned()
            .unwrap_or_default(),
        selected_extra_value: discover
            .pointer("/filters/extra")
            .and_then(serde_json::Value::as_object)
            .and_then(|extra| {
                extra
                    .iter()
                    .find(|(key, _)| key.as_str() != "search")
                    .and_then(|(_, value)| value.as_str())
            })
            .unwrap_or_default()
            .to_owned(),
        query: discover
            .pointer("/filters/extra/search")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .to_owned(),
        catalogs,
        results: discover
            .get("results")
            .and_then(serde_json::Value::as_array)
            .map(|items| items.iter().map(core_home_card).collect())
            .unwrap_or_default(),
        generation: discover
            .get("generation")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0),
        next_page,
        is_loading: discover
            .get("isLoading")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false),
        catalogs_loading: discover
            .get("catalogsLoading")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false),
        error: discover
            .get("error")
            .and_then(serde_json::Value::as_str)
            .map(ToOwned::to_owned),
        sections: Vec::new(),
    }
}

/// Refreshes fast-changing Discover state without re-projecting the entire
/// result array. A page response only appends to the current generation, so
/// convert just the new tail and update the paging flags in place.
pub fn refresh_discover_model_from_core_snapshot(
    snapshot: &serde_json::Value,
    model: &mut DiscoverModel,
) -> bool {
    let discover = snapshot.get("discover").unwrap_or(&serde_json::Value::Null);
    let filters = discover.get("filters").unwrap_or(&serde_json::Value::Null);
    let paging = discover.get("paging").unwrap_or(&serde_json::Value::Null);
    let generation = discover
        .get("generation")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(0);
    let content_type = value_string(discover, "contentType").unwrap_or_else(|| "movie".to_owned());
    let selected_catalog_key = filters
        .get("catalogKey")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default();
    let searching = snapshot
        .pointer("/search/query")
        .and_then(serde_json::Value::as_str)
        .is_some_and(|query| !query.trim().is_empty());
    if searching || !model.query.is_empty() {
        return false;
    }
    let Some(items) = discover
        .get("results")
        .and_then(serde_json::Value::as_array)
    else {
        return false;
    };
    if generation != model.generation
        || content_type != model.content_type
        || selected_catalog_key != model.selected_catalog_key
        || items.len() < model.results.len()
    {
        return false;
    }
    let catalog_count = discover
        .get("catalogs")
        .and_then(serde_json::Value::as_array)
        .map(|catalogs| catalogs.iter().take(24).count())
        .unwrap_or_default();
    if catalog_count != model.catalogs.len() {
        return false;
    }

    model
        .results
        .extend(items[model.results.len()..].iter().map(core_home_card));
    model.next_page = if paging.get("hasMore").and_then(serde_json::Value::as_bool) == Some(true)
        && paging.get("isLoading").and_then(serde_json::Value::as_bool) != Some(true)
        && paging.get("error").is_none_or(serde_json::Value::is_null)
    {
        filters
            .get("transportUrl")
            .and_then(serde_json::Value::as_str)
            .zip(filters.get("catalogId").and_then(serde_json::Value::as_str))
            .map(|(transport_url, catalog_id)| {
                serde_json::json!({
                    "type": "discoverPageRequested",
                    "transportUrl": transport_url,
                    "contentType": discover.get("contentType"),
                    "catalogId": catalog_id,
                    "skip": paging.get("nextSkip"),
                    "genre": filters.pointer("/extra/genre"),
                    "search": filters.pointer("/extra/search"),
                })
            })
    } else {
        None
    };
    model.is_loading = discover
        .get("isLoading")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);
    model.catalogs_loading = discover
        .get("catalogsLoading")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);
    model.error = discover
        .get("error")
        .and_then(serde_json::Value::as_str)
        .map(ToOwned::to_owned);
    true
}

/// Convert a batch of Core Discover records into the shared UI card model.
/// Desktop invokes this from a background worker so page-sized projections do
/// not block the render/input thread when a catalog page arrives.
pub fn discover_cards_from_core_values(items: Vec<serde_json::Value>) -> Vec<HomeCard> {
    items.iter().map(core_home_card).collect()
}
