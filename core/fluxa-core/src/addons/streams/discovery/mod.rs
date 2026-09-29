use crate::addons;
use crate::catalog;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct StreamDiscoveryPlanRequest {
    #[serde(rename = "type")]
    content_type: String,
    id: String,
    language: String,
    #[serde(default)]
    prefer_fast_start: bool,
    #[serde(default)]
    addon_request_timeout_ms: i64,
    #[serde(default)]
    fast_addon_request_timeout_ms: i64,
    #[serde(default)]
    cloudstream_timeout_ms: i64,
    #[serde(default)]
    addons: Vec<StreamDiscoveryAddon>,
    #[serde(default)]
    cs3_plugin_names: Vec<String>,
    cs3_search_query: Option<String>,
    cs3_original_name: Option<String>,
    cs3_year: Option<i64>,
    #[serde(default)]
    max_concurrent_addon_requests: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct StreamDiscoveryAddon {
    transport_url: String,
    manifest: Value,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct StreamDiscoveryPlan {
    cache_key: String,
    addon_requests: Vec<StreamAddonRequest>,
    cloudstream_request: Option<CloudstreamRequest>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct StreamDiscoveryExecutionPolicy {
    cache_key: String,
    cache_lookup_prefix: String,
    max_concurrent_addon_requests: i64,
    cache_write_minimum_result_count: i64,
    emit_cached_result: bool,
    emit_partial_non_empty_results: bool,
    addon_requests: Vec<StreamAddonRequest>,
    cloudstream_request: Option<CloudstreamRequest>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct StreamAddonRequest {
    transport_url: String,
    addon_name: String,
    #[serde(rename = "type")]
    content_type: String,
    id: String,
    timeout_ms: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CloudstreamRequest {
    id: String,
    title: String,
    year: Option<i64>,
    #[serde(rename = "type")]
    content_type: String,
    season: Option<i32>,
    episode: Option<i32>,
    original_name: Option<String>,
    timeout_ms: i64,
}

pub(crate) fn stream_discovery_plan_json(request_json: &str) -> Option<String> {
    let request = serde_json::from_str::<StreamDiscoveryPlanRequest>(request_json).ok()?;
    let plan = build_stream_discovery_plan(request)?;
    serde_json::to_string(&StreamDiscoveryPlan {
        cache_key: plan.cache_key,
        addon_requests: plan.addon_requests,
        cloudstream_request: plan.cloudstream_request,
    })
    .ok()
}

pub(crate) fn stream_discovery_cache_prefix(
    content_type: &str,
    id: &str,
    language: &str,
) -> String {
    format!("{content_type}|{id}|{language}")
}

pub(crate) fn stream_discovery_execution_policy_json(request_json: &str) -> Option<String> {
    let request = serde_json::from_str::<StreamDiscoveryPlanRequest>(request_json).ok()?;
    let requested_concurrency = request.max_concurrent_addon_requests;
    let cache_lookup_prefix =
        stream_discovery_cache_prefix(&request.content_type, &request.id, &request.language);
    let plan = build_stream_discovery_plan(request)?;
    let max_concurrent_addon_requests = if requested_concurrency <= 0 {
        (plan.addon_requests.len() as i64).clamp(1, 32)
    } else {
        requested_concurrency.clamp(1, 64)
    };
    serde_json::to_string(&StreamDiscoveryExecutionPolicy {
        cache_key: plan.cache_key,
        cache_lookup_prefix,
        max_concurrent_addon_requests,
        cache_write_minimum_result_count: 1,
        emit_cached_result: true,
        emit_partial_non_empty_results: true,
        addon_requests: plan.addon_requests,
        cloudstream_request: plan.cloudstream_request,
    })
    .ok()
}

fn build_stream_discovery_plan(request: StreamDiscoveryPlanRequest) -> Option<StreamDiscoveryPlan> {
    let addon_signatures = request
        .addons
        .iter()
        .map(|addon| {
            let id = addon
                .manifest
                .get("id")
                .and_then(Value::as_str)
                .unwrap_or_default();
            format!("{id}@{}", addon.transport_url)
        })
        .collect::<Vec<_>>();
    let cache_key = catalog::identity::stream_discovery_cache_key(
        &serde_json::json!({
            "type": request.content_type,
            "id": request.id,
            "language": request.language,
            "cs3SearchQuery": request.cs3_search_query,
            "cs3Year": request.cs3_year,
            "cs3OriginalName": request.cs3_original_name,
            "addonSignatures": addon_signatures,
            "cs3PluginNames": request.cs3_plugin_names,
        })
        .to_string(),
    )?;
    let timeout_ms = if request.prefer_fast_start {
        request.fast_addon_request_timeout_ms
    } else {
        request.addon_request_timeout_ms
    };
    let addon_requests = request
        .addons
        .iter()
        .filter(|addon| {
            addons::sources::protocol::supports_resource(
                &addon.manifest.to_string(),
                "stream",
                Some(&request.content_type),
                Some(&request.id),
            )
        })
        .map(|addon| StreamAddonRequest {
            transport_url: addon.transport_url.clone(),
            addon_name: addon
                .manifest
                .get("name")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string(),
            content_type: request.content_type.clone(),
            id: request.id.clone(),
            timeout_ms,
        })
        .collect::<Vec<_>>();
    let episode_locator = catalog::identity::parse_episode_locator(&request.id);
    let cloudstream_request = if request.cs3_plugin_names.is_empty() {
        None
    } else {
        request
            .cs3_search_query
            .as_ref()
            .map(|title| CloudstreamRequest {
                id: request.id.clone(),
                title: title.clone(),
                year: request.cs3_year,
                content_type: request.content_type.clone(),
                season: episode_locator.as_ref().map(|(_, season, _)| *season),
                episode: episode_locator.as_ref().map(|(_, _, episode)| *episode),
                original_name: request.cs3_original_name.clone(),
                timeout_ms: request.cloudstream_timeout_ms,
            })
    };
    Some(StreamDiscoveryPlan {
        cache_key,
        addon_requests,
        cloudstream_request,
    })
}

#[cfg(test)]
mod tests;
