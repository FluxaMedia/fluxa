use crate::repository_flow::addon_streams_with_provider_json;
use serde::Deserialize;
use serde_json::{Value, json};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ResourceParseRequest {
    #[serde(default)]
    kind: String,
    #[serde(default)]
    response: Value,
    #[serde(default)]
    addon_name: Option<String>,
    #[serde(default)]
    season: Option<i64>,
}

pub(crate) fn resource_parse_plan_json(request_json: &str) -> Option<String> {
    let request = serde_json::from_str::<ResourceParseRequest>(request_json).ok()?;
    let value = resource_parse_plan_value(
        &request.kind,
        request.response,
        request.addon_name.as_deref(),
        request.season,
    );
    serde_json::to_string(&value).ok()
}

/// Maps a request `kind` to the addon resource name used in URLs and responses.
/// `request_resource` and `item_resource` are optional overrides from the request.
pub(crate) fn resource_kind_to_resource(
    kind: &str,
    request_resource: Option<&str>,
    item_resource: Option<&str>,
) -> String {
    let explicit = item_resource
        .filter(|s| !s.trim().is_empty())
        .or_else(|| request_resource.filter(|s| !s.trim().is_empty()));
    if let Some(r) = explicit {
        return r.to_string();
    }
    match kind {
        "catalogPage" | "discover" | "search" => "catalog",
        "metaDetail" | "seasonEpisodes" => "meta",
        "streams" => "stream",
        "subtitles" => "subtitles",
        other if !other.trim().is_empty() => other,
        _ => "catalog",
    }
    .to_string()
}
fn resource_parse_plan_value(
    kind: &str,
    response: Value,
    addon_name: Option<&str>,
    season: Option<i64>,
) -> Value {
    match kind {
        "catalogPage" | "discover" | "search" => {
            json!({ "items": response.get("metas").and_then(Value::as_array).cloned().unwrap_or_default() })
        }
        "metaDetail" => json!({ "meta": response.get("meta").cloned().unwrap_or(Value::Null) }),
        "streams" => {
            let streams = response
                .get("streams")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default();
            let normalized = addon_streams_with_provider_json(
                &Value::Array(streams).to_string(),
                addon_name.unwrap_or(""),
            );
            json!({ "streams": serde_json::from_str::<Value>(&normalized).unwrap_or(Value::Array(vec![])) })
        }
        "seasonEpisodes" => {
            let videos = response
                .get("meta")
                .and_then(|meta| meta.get("videos"))
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default()
                .into_iter()
                .filter(|video| {
                    season.is_none() || video.get("season").and_then(Value::as_i64) == season
                })
                .collect::<Vec<_>>();
            json!({ "episodes": videos })
        }
        "subtitles" => {
            json!({ "subtitles": response.get("subtitles").and_then(Value::as_array).cloned().unwrap_or_default() })
        }
        _ => response,
    }
}
pub(crate) fn parse_and_plan_addon_resource_json(
    resource: &str,
    url: &str,
    status_code: i32,
    body: Option<&str>,
    kind: &str,
    addon_name: Option<&str>,
    season: Option<i64>,
) -> String {
    match crate::addon_resource::parse_addon_body(resource, url, status_code, body) {
        crate::addon_resource::ParsedAddonBody::Error(err_json) => err_json,
        crate::addon_resource::ParsedAddonBody::Success { payload, .. } => {
            let wrapped =
                crate::addon_resource::wrap_addon_resource_response_value(resource, payload);
            let value = resource_parse_plan_value(kind, wrapped, addon_name, season);
            json!({ "kind": "success", "value": value }).to_string()
        }
    }
}
