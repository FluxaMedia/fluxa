use crate::addon_protocol::build_resource_url;
use crate::content_identity::parse_extra_args_json;
use serde::Deserialize;
use serde_json::{Map, Value, json};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct MetaDetailPlanRequest {
    use_configured_addons: bool,
    auth_key: String,
    #[serde(default)]
    local_addons: Vec<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ManifestFetchDecisionRequest {
    force_refresh: bool,
    memory_hit: bool,
    persistent_hit: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct AddonResourceRequestPlan {
    transport_url: String,
    resource: String,
    content_type: String,
    id: String,
    #[serde(default)]
    extra_args: Map<String, Value>,
    #[serde(default)]
    extra_raw: String,
}

pub(crate) fn repository_meta_detail_plan_json(request_json: &str) -> Option<String> {
    let request: MetaDetailPlanRequest = serde_json::from_str(request_json).ok()?;
    let has_configured_source = !request.auth_key.trim().is_empty()
        || request
            .local_addons
            .iter()
            .any(|addon| !addon.trim().is_empty());
    serde_json::to_string(&json!({
        "preferAddonMetaDetail": request.use_configured_addons && has_configured_source,
        "fallbackToStremioMetaDetail": true
    }))
    .ok()
}

pub(crate) fn repository_season_videos_json(meta_detail_json: &str, season_number: i32) -> String {
    let videos = serde_json::from_str::<Value>(meta_detail_json)
        .ok()
        .and_then(|value| value.get("videos").cloned())
        .and_then(|value| value.as_array().cloned())
        .unwrap_or_default()
        .into_iter()
        .filter(|video| {
            video
                .get("season")
                .and_then(Value::as_i64)
                .map(|season| season == season_number as i64)
                .unwrap_or(false)
        })
        .collect::<Vec<_>>();
    Value::Array(videos).to_string()
}

pub(crate) fn manifest_fetch_decision_json(request_json: &str) -> Option<String> {
    let request: ManifestFetchDecisionRequest = serde_json::from_str(request_json).ok()?;
    let phase = if request.force_refresh {
        "fetch"
    } else if request.memory_hit {
        "memory"
    } else if request.persistent_hit {
        "persistent"
    } else {
        "fetch"
    };
    serde_json::to_string(&json!({
        "phase": phase,
        "allowStaleFallback": true
    }))
    .ok()
}

pub(crate) fn addon_resource_request_plan_json(request_json: &str) -> Option<String> {
    let request: AddonResourceRequestPlan = serde_json::from_str(request_json).ok()?;
    let mut urls = Vec::new();
    if request.resource == "subtitles" || request.resource == "subtitle" {
        urls.push(build_resource_url(
            &request.transport_url,
            "subtitles",
            &request.content_type,
            &request.id,
            None,
        ));
        if !request.extra_raw.trim().is_empty() {
            urls.push(build_resource_url(
                &request.transport_url,
                "subtitles",
                &request.content_type,
                &request.id,
                parse_extra_args_json(&request.extra_raw).as_deref(),
            ));
        }
    } else {
        urls.push(build_resource_url(
            &request.transport_url,
            &request.resource,
            &request.content_type,
            &request.id,
            Some(&Value::Object(request.extra_args).to_string()),
        ));
    }
    urls.dedup();
    serde_json::to_string(&json!({ "urls": urls })).ok()
}

pub(crate) fn addon_streams_with_provider_json(streams_json: &str, addon_name: &str) -> String {
    let streams = serde_json::from_str::<Vec<Value>>(streams_json)
        .unwrap_or_default()
        .into_iter()
        .map(|stream| normalize_stream(stream, addon_name))
        .collect::<Vec<_>>();
    Value::Array(streams).to_string()
}

pub(crate) fn normalize_stream(mut stream: Value, addon_name: &str) -> Value {
    let Some(stream_object) = stream.as_object_mut() else {
        return stream;
    };
    if !addon_name.trim().is_empty() {
        stream_object.insert(
            "addonName".to_string(),
            Value::String(addon_name.to_string()),
        );
    }
    let Some(behavior_hints) = stream_object
        .get("behaviorHints")
        .and_then(Value::as_object)
        .cloned()
    else {
        return stream;
    };

    let mut headers = Map::new();
    collect_headers(behavior_hints.get("requestHeaders"), &mut headers);
    let proxy_request = behavior_hints
        .get("proxyHeaders")
        .and_then(|proxy| proxy.get("request"));
    collect_headers(proxy_request, &mut headers);

    let mut final_hints = behavior_hints;
    if !headers.is_empty() {
        final_hints.insert("requestHeaders".to_string(), Value::Object(headers));
    }
    fill_from_hint(stream_object, &final_hints, "videoHash");
    fill_from_hint(stream_object, &final_hints, "videoSize");
    fill_from_hint(stream_object, &final_hints, "filename");
    stream_object.insert("behaviorHints".to_string(), Value::Object(final_hints));
    stream
}

fn collect_headers(value: Option<&Value>, headers: &mut Map<String, Value>) {
    let Some(map) = value.and_then(Value::as_object) else {
        return;
    };
    for (key, value) in map {
        if key.is_empty() {
            continue;
        }
        let text = value
            .as_str()
            .map(str::to_string)
            .unwrap_or_else(|| value.to_string());
        if !text.is_empty() {
            headers.insert(key.clone(), Value::String(text));
        }
    }
}

fn fill_from_hint(stream_object: &mut Map<String, Value>, hints: &Map<String, Value>, key: &str) {
    if stream_object
        .get(key)
        .filter(|value| !value.is_null())
        .is_some()
    {
        return;
    }
    if let Some(value) = hints.get(key) {
        stream_object.insert(key.to_string(), value.clone());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repository_meta_detail_plan_prefers_addons_only_when_configured_source_exists() {
        let plan = repository_meta_detail_plan_json(
            r#"{"useConfiguredAddons":true,"authKey":"","localAddons":[]}"#,
        )
        .unwrap();
        assert!(plan.contains(r#""preferAddonMetaDetail":false"#));

        let plan = repository_meta_detail_plan_json(
            r#"{"useConfiguredAddons":true,"authKey":"","localAddons":["https://addon/manifest.json"]}"#,
        )
        .unwrap();
        assert!(plan.contains(r#""preferAddonMetaDetail":true"#));
    }

    #[test]
    fn repository_season_videos_filters_by_season_without_reordering() {
        let videos = repository_season_videos_json(
            r#"{"videos":[{"id":"s2e1","season":2},{"id":"s1e1","season":1},{"id":"s2e2","season":2}]}"#,
            2,
        );
        assert_eq!(
            videos,
            r#"[{"id":"s2e1","season":2},{"id":"s2e2","season":2}]"#
        );
    }

    #[test]
    fn manifest_fetch_decision_uses_cache_before_network_unless_forced() {
        assert!(
            manifest_fetch_decision_json(
                r#"{"forceRefresh":false,"memoryHit":true,"persistentHit":true}"#
            )
            .unwrap()
            .contains(r#""phase":"memory""#)
        );
        assert!(
            manifest_fetch_decision_json(
                r#"{"forceRefresh":true,"memoryHit":true,"persistentHit":true}"#
            )
            .unwrap()
            .contains(r#""phase":"fetch""#)
        );
    }

    #[test]
    fn addon_resource_plan_builds_subtitle_and_catalog_urls() {
        let subtitles = addon_resource_request_plan_json(
            r#"{"transportUrl":"https://addon.example/manifest.json","resource":"subtitles","contentType":"movie","id":"tt1","extraRaw":"videoHash=abc&filename=File Name.mkv"}"#,
        )
        .unwrap();
        assert!(subtitles.contains("https://addon.example/subtitles/movie/tt1.json"));
        assert!(subtitles.contains("videoHash=abc"));
        assert!(subtitles.contains("filename=File%20Name.mkv"));

        let catalog = addon_resource_request_plan_json(
            r#"{"transportUrl":"https://addon.example/manifest.json","resource":"catalog","contentType":"movie","id":"top","extraArgs":{"skip":"100","search":"matrix"}}"#,
        )
        .unwrap();
        assert!(catalog.contains("catalog/movie/top/"));
        assert!(catalog.contains("skip=100"));
        assert!(catalog.contains("search=matrix"));
    }

    #[test]
    fn stream_provider_normalization_merges_headers_and_hints_without_reordering() {
        let streams = addon_streams_with_provider_json(
            r#"[{"title":"A","behaviorHints":{"videoHash":"abc","videoSize":12,"proxyHeaders":{"request":{"X-Proxy":"1"}}}},{"title":"B"}]"#,
            "Addon",
        );
        let value: Value = serde_json::from_str(&streams).unwrap();
        assert_eq!(value[0]["title"].as_str(), Some("A"));
        assert_eq!(value[0]["addonName"].as_str(), Some("Addon"));
        assert_eq!(value[0]["videoHash"].as_str(), Some("abc"));
        assert_eq!(value[0]["videoSize"].as_i64(), Some(12));
        assert_eq!(
            value[0]["behaviorHints"]["requestHeaders"]["X-Proxy"].as_str(),
            Some("1")
        );
        assert_eq!(value[1]["title"].as_str(), Some("B"));
    }
}
