use crate::addons::sources::repository::addon_streams_with_provider_json;
use serde_json::{Value, json};

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
    resource_url: Option<&str>,
) -> Value {
    match kind {
        "catalogPage" | "discover" | "search" => {
            let mut response = response;
            let items = match response.get_mut("metas").map(Value::take) {
                Some(Value::Array(items)) => items,
                _ => Vec::new(),
            };
            json!({ "items": items })
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
            let subtitles = response
                .get("subtitles")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default();
            let subtitles = resource_url
                .map(|url| {
                    crate::addons::streams::resource::normalize_addon_subtitles_json(
                        &Value::Array(subtitles.clone()).to_string(),
                        url,
                    )
                })
                .and_then(|json| serde_json::from_str::<Value>(&json).ok())
                .unwrap_or(Value::Array(subtitles));
            json!({ "subtitles": subtitles })
        }
        _ => response,
    }
}
pub fn parse_and_plan_addon_resource(
    resource: &str,
    url: &str,
    status_code: i32,
    body: Option<&str>,
    kind: &str,
    addon_name: Option<&str>,
    season: Option<i64>,
) -> Value {
    match crate::addons::streams::resource::parse_addon_body(resource, url, status_code, body) {
        crate::addons::streams::resource::ParsedAddonBody::Error(err_json) => {
            serde_json::from_str(&err_json).unwrap_or(Value::Null)
        }
        crate::addons::streams::resource::ParsedAddonBody::Success { payload } => {
            let wrapped = crate::addons::streams::resource::wrap_addon_resource_response_value(
                resource, payload,
            );
            let value = resource_parse_plan_value(kind, wrapped, addon_name, season, Some(url));
            json!({
                "kind": "success",
                "url": url,
                "statusCode": status_code,
                "value": value,
            })
        }
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
    let mut result = parse_and_plan_addon_resource(
        resource,
        url,
        status_code,
        body,
        kind,
        addon_name,
        season,
    );
    if result.get("kind").and_then(Value::as_str) == Some("success") {
        let value = &result["value"];
        let key = match kind {
            "catalogPage" | "discover" | "search" => Some("items"),
            "metaDetail" => Some("meta"),
            "streams" => Some("streams"),
            "seasonEpisodes" => Some("episodes"),
            "subtitles" => Some("subtitles"),
            _ => None,
        };
        let value_json = match key {
            Some(key) => value.get(key),
            None => Some(value),
        }
        .map_or_else(|| "null".to_owned(), Value::to_string);
        result["valueJson"] = Value::String(value_json);
    }
    result.to_string()
}
