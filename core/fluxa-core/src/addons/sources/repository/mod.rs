use crate::addons::sources::protocol::build_resource_url;
use crate::catalog::identity::parse_extra_args_json;
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
        .filter(|stream| !is_unsupported_source(stream))
        .map(|stream| normalize_stream(stream, addon_name))
        .collect::<Vec<_>>();
    Value::Array(streams).to_string()
}

pub(crate) fn is_unsupported_source(stream: &Value) -> bool {
    let has = |key| stream.get(key).is_some_and(|value| !value.is_null());
    let archive_only = [
        "nzbUrl", "rarUrls", "zipUrls", "7zipUrls", "tgzUrls", "tarUrls",
    ]
    .into_iter()
    .any(has);
    archive_only
        && ![
            "url",
            "ytId",
            "yt_ID",
            "infoHash",
            "externalUrl",
            "playerFrameUrl",
        ]
        .into_iter()
        .any(has)
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
    let description_is_blank = stream_object
        .get("description")
        .map(|value| {
            value.is_null() || value.as_str().map(str::trim).unwrap_or_default().is_empty()
        })
        .unwrap_or(true);
    if description_is_blank {
        if let Some(title) = stream_object
            .get("title")
            .filter(|value| {
                value
                    .as_str()
                    .map(str::trim)
                    .is_some_and(|title| !title.is_empty())
            })
            .cloned()
        {
            stream_object.insert("description".to_string(), title);
        }
    }
    let had_behavior_hints = stream_object
        .get("behaviorHints")
        .and_then(Value::as_object)
        .is_some();
    let behavior_hints = stream_object
        .get("behaviorHints")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();

    let mut headers = Map::new();
    collect_headers(stream_object.get("headers"), &mut headers);
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
    if had_behavior_hints || !final_hints.is_empty() {
        stream_object.insert("behaviorHints".to_string(), Value::Object(final_hints));
    }
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
mod tests;
