use crate::addon_protocol;
use crate::stream_policy;
use serde::Deserialize;
use serde_json::{Value, json};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProviderAvailabilityRequest {
    #[serde(default)]
    addons: Vec<Value>,
    #[serde(default)]
    plugin_names: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DetailStreamAttempt {
    request_id: String,
    #[serde(default)]
    streams: Vec<Value>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DetailStreamResultRequest {
    #[serde(default)]
    attempts: Vec<DetailStreamAttempt>,
    #[serde(default)]
    has_stream_providers: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PrefetchPlanRequest {
    #[serde(default)]
    streams: Vec<Value>,
}

pub(crate) fn provider_availability_plan_json(request_json: &str) -> Option<String> {
    let request = serde_json::from_str::<ProviderAvailabilityRequest>(request_json).ok()?;
    let has_stremio_stream_addon = request.addons.iter().any(|addon| {
        addon.get("manifest").is_some_and(|manifest| {
            addon_protocol::supports_resource(&manifest.to_string(), "stream", None, None)
        })
    });
    let plugin_names = stable_non_empty_strings(request.plugin_names);
    serde_json::to_string(&json!({
        "hasStremioStreamAddons": has_stremio_stream_addon,
        "hasPluginStreamProviders": !plugin_names.is_empty(),
        "hasStreamProviders": has_stremio_stream_addon || !plugin_names.is_empty(),
        "pluginNames": plugin_names
    }))
    .ok()
}

pub(crate) fn detail_stream_result_plan_json(request_json: &str) -> Option<String> {
    let request = serde_json::from_str::<DetailStreamResultRequest>(request_json).ok()?;
    let selected = request
        .attempts
        .iter()
        .find(|attempt| !attempt.streams.is_empty());
    let streams = selected
        .map(|attempt| attempt.streams.clone())
        .unwrap_or_default();
    let resolved_request_id = selected
        .map(|attempt| Value::String(attempt.request_id.clone()))
        .unwrap_or(Value::Null);
    serde_json::to_string(&json!({
        "streams": streams,
        "availableAddons": available_addons(&streams),
        "resolvedRequestId": resolved_request_id,
        "hasStreamProviders": request.has_stream_providers
    }))
    .ok()
}

pub(crate) fn prefetch_detail_streams_plan_json(request_json: &str) -> Option<String> {
    let request = serde_json::from_str::<PrefetchPlanRequest>(request_json).ok()?;
    let prewarm_url = request.streams.iter().find_map(playable_torrent_url);
    serde_json::to_string(&json!({
        "count": request.streams.len(),
        "prewarmUrl": prewarm_url,
        "shouldPrewarmTorrent": prewarm_url.is_some()
    }))
    .ok()
}

pub(crate) fn direct_playback_policy_json() -> String {
    json!({
        "metaDetailTimeoutMs": 3500,
        "streamDetailTimeoutMs": 2500
    })
    .to_string()
}

fn available_addons(streams: &[Value]) -> Vec<String> {
    streams
        .iter()
        .filter_map(|stream| stream.get("addonName").and_then(Value::as_str))
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .fold(Vec::new(), |mut acc, value| {
            if !acc.iter().any(|existing| existing == value) {
                acc.push(value.to_string());
            }
            acc
        })
}

fn playable_torrent_url(stream: &Value) -> Option<String> {
    let url = stream
        .get("playableUrl")
        .or_else(|| stream.get("url"))
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())?;
    stream_policy::is_torrent_playback_url(url).then(|| url.to_string())
}

fn stable_non_empty_strings(values: Vec<String>) -> Vec<String> {
    values
        .into_iter()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .fold(Vec::new(), |mut acc, value| {
            if !acc.contains(&value) {
                acc.push(value);
            }
            acc
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    #[test]
    fn provider_availability_is_core_planned_from_addons_and_plugins() {
        let value: Value = serde_json::from_str(
            &provider_availability_plan_json(
                r#"{"addons":[{"manifest":{"resources":["catalog",{"name":"stream","types":["movie"]}]}}],"pluginNames":["CS3","CS3"," "]}"#,
            )
            .unwrap(),
        )
        .unwrap();

        assert_eq!(value["hasStremioStreamAddons"], true);
        assert_eq!(value["hasPluginStreamProviders"], true);
        assert_eq!(value["hasStreamProviders"], true);
        assert_eq!(value["pluginNames"], json!(["CS3"]));
    }

    #[test]
    fn detail_stream_result_stops_on_first_non_empty_attempt_without_reordering_streams() {
        let value: Value = serde_json::from_str(
            &detail_stream_result_plan_json(
                r#"{"hasStreamProviders":true,"attempts":[{"requestId":"tt1","streams":[]},{"requestId":"tt1:1:1","streams":[{"url":"b","addonName":"B"},{"url":"a","addonName":"A"},{"url":"b2","addonName":"B"}]}]}"#,
            )
            .unwrap(),
        )
        .unwrap();

        assert_eq!(value["resolvedRequestId"], "tt1:1:1");
        assert_eq!(value["streams"][0]["url"], "b");
        assert_eq!(value["streams"][1]["url"], "a");
        assert_eq!(value["availableAddons"], json!(["B", "A"]));
    }

    #[test]
    fn prefetch_plan_selects_first_torrent_playable_url_only() {
        let value: Value = serde_json::from_str(
            &prefetch_detail_streams_plan_json(
                r#"{"streams":[{"playableUrl":"https://video.example/file.mp4"},{"playableUrl":"magnet:?xt=urn:btih:abc"},{"playableUrl":"magnet:?xt=urn:btih:def"}]}"#,
            )
            .unwrap(),
        )
        .unwrap();

        assert_eq!(value["count"], 3);
        assert_eq!(value["shouldPrewarmTorrent"], true);
        assert_eq!(value["prewarmUrl"], "magnet:?xt=urn:btih:abc");
    }
}
