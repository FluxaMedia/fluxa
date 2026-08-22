use crate::addon_protocol;
use crate::content_identity;
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
    let cache_key = content_identity::stream_discovery_cache_key(
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
            addon_protocol::supports_resource(
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
    let episode_locator = content_identity::parse_episode_locator(&request.id);
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
mod tests {
    use super::{
        stream_discovery_cache_prefix, stream_discovery_execution_policy_json,
        stream_discovery_plan_json,
    };
    use serde_json::Value;

    #[test]
    fn stream_discovery_plan_filters_by_manifest_without_reordering_addons() {
        let plan = stream_discovery_plan_json(
            r#"{
                "type":"series",
                "id":"tt1:2:7",
                "language":"en",
                "preferFastStart":true,
                "addonRequestTimeoutMs":1000,
                "fastAddonRequestTimeoutMs":3000,
                "cloudstreamTimeoutMs":5000,
                "cs3PluginNames":["cs"],
                "cs3SearchQuery":"Show",
                "cs3OriginalName":"Original Show",
                "cs3Year":2024,
                "addons":[
                    {"transportUrl":"https://a/manifest.json","manifest":{"id":"a","name":"A","resources":["stream"],"types":["movie"]}},
                    {"transportUrl":"https://b/manifest.json","manifest":{"id":"b","name":"B","resources":["stream"],"types":["series"]}},
                    {"transportUrl":"https://c/manifest.json","manifest":{"id":"c","name":"C","resources":[{"name":"stream","types":["series"]}],"types":["series"]}}
                ]
            }"#,
        )
        .expect("plan");
        let plan: Value = serde_json::from_str(&plan).expect("plan json");
        let addon_requests = plan["addonRequests"].as_array().expect("addon requests");

        assert_eq!(addon_requests.len(), 2);
        assert_eq!(addon_requests[0]["addonName"].as_str(), Some("B"));
        assert_eq!(addon_requests[1]["addonName"].as_str(), Some("C"));
        assert_eq!(addon_requests[0]["timeoutMs"].as_i64(), Some(3000));
        assert_eq!(plan["cloudstreamRequest"]["season"].as_i64(), Some(2));
        assert_eq!(plan["cloudstreamRequest"]["episode"].as_i64(), Some(7));
    }

    #[test]
    fn stream_discovery_execution_policy_owns_cache_and_progress_rules() {
        let policy = stream_discovery_execution_policy_json(
            r#"{
                "type":"movie",
                "id":"tt1",
                "language":"tr",
                "addonRequestTimeoutMs":1000,
                "fastAddonRequestTimeoutMs":3000,
                "cloudstreamTimeoutMs":5000,
                "maxConcurrentAddonRequests":64,
                "addons":[
                    {"transportUrl":"https://a/manifest.json","manifest":{"id":"a","name":"A","resources":["stream"],"types":["movie"]}},
                    {"transportUrl":"https://b/manifest.json","manifest":{"id":"b","name":"B","resources":["stream"],"types":["movie"]}}
                ],
                "cs3PluginNames":["cs"],
                "cs3SearchQuery":"Movie"
            }"#,
        )
        .expect("policy");
        let policy: Value = serde_json::from_str(&policy).expect("policy json");
        let addon_requests = policy["addonRequests"].as_array().expect("addon requests");

        assert_eq!(policy["cacheLookupPrefix"].as_str(), Some("movie|tt1|tr"));
        assert_eq!(policy["maxConcurrentAddonRequests"].as_i64(), Some(64));
        assert_eq!(policy["cacheWriteMinimumResultCount"].as_i64(), Some(1));
        assert_eq!(policy["emitCachedResult"].as_bool(), Some(true));
        assert_eq!(policy["emitPartialNonEmptyResults"].as_bool(), Some(true));
        assert_eq!(addon_requests[0]["addonName"].as_str(), Some("A"));
        assert_eq!(addon_requests[1]["addonName"].as_str(), Some("B"));
        assert!(policy["cloudstreamRequest"].is_object());
    }

    #[test]
    fn stream_discovery_execution_policy_auto_concurrency_uses_addon_count() {
        let policy = stream_discovery_execution_policy_json(
            r#"{
                "type":"movie",
                "id":"tt2",
                "language":"en",
                "addonRequestTimeoutMs":1000,
                "fastAddonRequestTimeoutMs":3000,
                "cloudstreamTimeoutMs":5000,
                "maxConcurrentAddonRequests":0,
                "addons":[
                    {"transportUrl":"https://a/manifest.json","manifest":{"id":"a","name":"A","resources":["stream"],"types":["movie"]}},
                    {"transportUrl":"https://b/manifest.json","manifest":{"id":"b","name":"B","resources":["stream"],"types":["movie"]}},
                    {"transportUrl":"https://c/manifest.json","manifest":{"id":"c","name":"C","resources":["stream"],"types":["movie"]}}
                ],
                "cs3PluginNames":[]
            }"#,
        )
        .expect("policy");
        let policy: Value = serde_json::from_str(&policy).expect("policy json");
        // 3 addons, auto mode → concurrency = min(3, 8) = 3
        assert_eq!(policy["maxConcurrentAddonRequests"].as_i64(), Some(3));
    }

    #[test]
    fn stream_discovery_cache_prefix_is_owned_by_core() {
        assert_eq!(
            stream_discovery_cache_prefix("series", "tt1:1:2", "en"),
            "series|tt1:1:2|en"
        );
    }
}
