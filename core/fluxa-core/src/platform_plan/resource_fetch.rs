use crate::addon_protocol::{
    build_resource_url, catalog_supports_extra as manifest_catalog_supports_extra,
    supports_resource,
};
use crate::content_identity::{parse_extra_args_json, stable_feed_part};
use serde::Deserialize;
use serde_json::{Map, Value, json};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ResourceFetchPlanRequest {
    #[serde(default)]
    kind: String,
    #[serde(default)]
    addons: Vec<Value>,
    #[serde(default)]
    transport_url: Option<String>,
    #[serde(default)]
    resource: Option<String>,
    #[serde(default)]
    content_type: Option<String>,
    #[serde(default)]
    catalog_id: Option<String>,
    #[serde(default)]
    catalog_key: Option<String>,
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    request_ids: Vec<String>,
    #[serde(default)]
    extra: Map<String, Value>,
    #[serde(default)]
    extra_raw: String,
    #[serde(default)]
    query: Option<String>,
    #[serde(default)]
    genre: Option<String>,
    #[serde(default)]
    skip: Option<i64>,
}

pub(crate) fn resource_fetch_plan_json(request_json: &str) -> Option<String> {
    let request = serde_json::from_str::<ResourceFetchPlanRequest>(request_json).ok()?;
    let mut requests = Vec::<Value>::new();

    match request.kind.as_str() {
        "catalogPage" => {
            let transport_url = request.transport_url.as_deref()?;
            let content_type = request.content_type.as_deref()?;
            let catalog_id = request.catalog_id.as_deref()?;
            requests.push(json!({
                "url": build_resource_url(transport_url, "catalog", content_type, catalog_id, extra_json(&request).as_deref()),
                "kind": "catalogPage"
            }));
        }
        "search" => {
            let query = request.query.as_deref().unwrap_or("");
            for addon in &request.addons {
                let Some(transport_url) = addon_transport_url(addon) else {
                    continue;
                };
                for catalog in addon_catalogs(addon) {
                    if !catalog_supports_search(&catalog) {
                        continue;
                    }
                    let Some(content_type) = catalog.get("type").and_then(Value::as_str) else {
                        continue;
                    };
                    let Some(id) = catalog.get("id").and_then(Value::as_str) else {
                        continue;
                    };
                    requests.push(json!({
                        "url": build_resource_url(transport_url, "catalog", content_type, id, Some(&json!({"search": query}).to_string())),
                        "kind": "search",
                        "addonName": addon_display_name(addon),
                        "transportUrl": transport_url,
                        "catalogId": id,
                        "catalogType": content_type,
                        "categoryId": format!("{}:{}:{}", transport_url, content_type, id),
                        "categoryName": search_category_name(addon, &catalog, content_type)
                    }));
                }
            }
        }
        "discover" => {
            let catalog_key = request.catalog_key.as_deref()?;
            for addon in &request.addons {
                let Some(transport_url) = addon_transport_url(addon) else {
                    continue;
                };
                for catalog in addon_catalogs(addon) {
                    let Some(content_type) = catalog.get("type").and_then(Value::as_str) else {
                        continue;
                    };
                    let Some(id) = catalog.get("id").and_then(Value::as_str) else {
                        continue;
                    };
                    let key = format!(
                        "discover:{}:{}:{}",
                        stable_feed_part(transport_url),
                        stable_feed_part(content_type),
                        stable_feed_part(id),
                    );
                    if key != catalog_key {
                        continue;
                    }
                    let extra = request
                        .extra
                        .iter()
                        .filter(|(name, _)| catalog_supports_extra(&catalog, name))
                        .map(|(name, value)| (name.clone(), value.clone()))
                        .collect::<Map<_, _>>();
                    let extra = (!extra.is_empty()).then(|| Value::Object(extra).to_string());
                    requests.push(json!({
                        "url": build_resource_url(transport_url, "catalog", content_type, id, extra.as_deref()),
                        "kind": "discover",
                        "catalogKey": key
                    }));
                    break;
                }
                if !requests.is_empty() {
                    break;
                }
            }
        }
        "metaDetail" => {
            let content_type = request.content_type.as_deref()?;
            let id = request.id.as_deref()?;
            let source_transport_url = request
                .transport_url
                .as_deref()
                .filter(|url| !url.is_empty());
            let source_addons: Vec<&Value> = match source_transport_url {
                Some(url) => request
                    .addons
                    .iter()
                    .filter(|addon| {
                        addon_transport_url(addon) == Some(url)
                            && addon_supports(addon, "meta", content_type, Some(id))
                    })
                    .collect(),
                None => Vec::new(),
            };
            let candidate_addons: Vec<&Value> = if source_addons.is_empty() {
                // No known source addon (deep link, search, library nav), it's no
                // longer configured, or it doesn't implement the meta resource at all
                // (e.g. a torrent-indexing addon): fall back to racing every
                // meta-capable addon instead of coming back empty.
                request.addons.iter().collect()
            } else {
                source_addons
            };
            for addon in &candidate_addons {
                if !addon_supports(addon, "meta", content_type, Some(id)) {
                    continue;
                }
                let Some(transport_url) = addon_transport_url(addon) else {
                    continue;
                };
                requests.push(json!({
                    "url": build_resource_url(transport_url, "meta", content_type, id, None),
                    "kind": "metaDetail",
                    "addonName": addon_display_name(addon),
                    "stopOnFirstResult": true
                }));
            }
        }
        "streams" => {
            let content_type = request.content_type.as_deref()?;
            for addon in &request.addons {
                if !addon_supports(addon, "stream", content_type, None) {
                    continue;
                }
                let Some(transport_url) = addon_transport_url(addon) else {
                    continue;
                };
                for id in &request.request_ids {
                    requests.push(json!({
                        "url": build_resource_url(transport_url, "stream", content_type, id, None),
                        "kind": "streams",
                        "addonName": addon_display_name(addon)
                    }));
                }
            }
        }
        "seasonEpisodes" => {
            let series_id = request.id.as_deref()?;
            for addon in &request.addons {
                if !addon_supports(addon, "meta", "series", Some(series_id)) {
                    continue;
                }
                let Some(transport_url) = addon_transport_url(addon) else {
                    continue;
                };
                requests.push(json!({
                    "url": build_resource_url(transport_url, "meta", "series", series_id, None),
                    "kind": "seasonEpisodes",
                    "addonName": addon_display_name(addon),
                    "stopOnFirstResult": true
                }));
            }
        }
        "subtitles" => {
            let content_type = request.content_type.as_deref()?;
            let id = request.id.as_deref()?;
            for addon in &request.addons {
                if !addon_supports(addon, "subtitles", content_type, Some(id)) {
                    continue;
                }
                let Some(transport_url) = addon_transport_url(addon) else {
                    continue;
                };
                requests.push(json!({
                    "url": build_resource_url(transport_url, "subtitles", content_type, id, None),
                    "kind": "subtitles",
                    "addonName": addon_display_name(addon)
                }));
                if !request.extra_raw.trim().is_empty() {
                    requests.push(json!({
                        "url": build_resource_url(
                            transport_url,
                            "subtitles",
                            content_type,
                            id,
                            parse_extra_args_json(&request.extra_raw).as_deref()
                        ),
                        "kind": "subtitles",
                        "addonName": addon_display_name(addon)
                    }));
                }
            }
        }
        _ => {
            let transport_url = request.transport_url.as_deref()?;
            let resource = request.resource.as_deref()?;
            let content_type = request.content_type.as_deref()?;
            let id = request.id.as_deref()?;
            requests.push(json!({
                "url": build_resource_url(transport_url, resource, content_type, id, extra_json(&request).as_deref()),
                "kind": request.kind
            }));
        }
    }

    serde_json::to_string(&json!({ "requests": requests })).ok()
}

/// Wraps `resource_fetch_plan_json` with the execution policy for running its
/// requests: whether to race them (all `stopOnFirstResult`, take the first non-empty
/// result) or fan them out with bounded concurrency, and the retry/timeout budget for
/// stream requests specifically (addon stream endpoints are the flakiest resource kind).
pub(crate) fn resource_fetch_execution_policy_json(request_json: &str) -> Option<String> {
    let plan: Value = serde_json::from_str(&resource_fetch_plan_json(request_json)?).ok()?;
    let requests = plan.get("requests")?.as_array()?.clone();
    let mode = if requests.len() > 1
        && requests
            .iter()
            .all(|r| r.get("stopOnFirstResult").and_then(Value::as_bool) == Some(true))
    {
        "race"
    } else {
        "fanout"
    };
    serde_json::to_string(&json!({
        "requests": requests,
        "mode": mode,
        "concurrency": 12,
        "streamRetry": {
            "maxAttempts": 3,
            "fetchTimeoutMs": 60_000,
            "retryTimeoutMs": 20_000,
        },
    }))
    .ok()
}

fn extra_json(request: &ResourceFetchPlanRequest) -> Option<String> {
    let mut extra = request.extra.clone();
    if let Some(genre) = request.genre.as_ref().filter(|value| !value.is_empty()) {
        extra.insert("genre".to_string(), Value::String(genre.clone()));
    }
    if let Some(search) = request.query.as_ref().filter(|value| !value.is_empty()) {
        extra.insert("search".to_string(), Value::String(search.clone()));
    }
    if let Some(skip) = request.skip.filter(|value| *value > 0) {
        extra.insert("skip".to_string(), Value::Number(skip.into()));
    }
    (!extra.is_empty()).then(|| Value::Object(extra).to_string())
}
// The TMDB builtin pseudo-addon uses this sentinel transportUrl so the JS side
// can recognize it, but it has no real HTTP resource server behind it — the
// host resolves it via a dedicated builtin request path instead. Addons with
// this transportUrl must never be turned into a generic per-addon HTTP
// request here, or the sentinel gets naively joined into a bogus URL.
const BUILTIN_TMDB_TRANSPORT_URL: &str = "tmdb://builtin";
fn addon_transport_url(addon: &Value) -> Option<&str> {
    addon
        .get("transportUrl")
        .and_then(Value::as_str)
        .filter(|url| *url != BUILTIN_TMDB_TRANSPORT_URL)
}
fn addon_manifest(addon: &Value) -> Value {
    addon
        .get("manifest")
        .cloned()
        .unwrap_or_else(|| addon.clone())
}
fn addon_catalogs(addon: &Value) -> Vec<Value> {
    addon_manifest(addon)
        .get("catalogs")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
}
fn addon_supports(addon: &Value, resource: &str, content_type: &str, id: Option<&str>) -> bool {
    let manifest = addon_manifest(addon);
    supports_resource(&manifest.to_string(), resource, Some(content_type), id)
}
fn addon_display_name(addon: &Value) -> String {
    addon
        .get("name")
        .or_else(|| {
            addon
                .get("manifest")
                .and_then(|manifest| manifest.get("name"))
        })
        .and_then(Value::as_str)
        .unwrap_or("Unknown Addon")
        .to_string()
}
fn catalog_supports_extra(catalog: &Value, name: &str) -> bool {
    serde_json::to_string(catalog)
        .ok()
        .is_some_and(|json| manifest_catalog_supports_extra(&json, name))
}
fn catalog_supports_search(catalog: &Value) -> bool {
    catalog_supports_extra(catalog, "search")
}
fn search_category_name(addon: &Value, catalog: &Value, content_type: &str) -> String {
    let addon_name = addon_display_name(addon);
    let catalog_name = catalog
        .get("name")
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .unwrap_or(match content_type {
            "movie" => "Movies",
            "series" => "Series",
            other => other,
        });
    format!("{addon_name} - {catalog_name}")
}
