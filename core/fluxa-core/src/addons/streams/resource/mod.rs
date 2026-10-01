use crate::addons;
use serde_json::{Value, json};

fn resource_payload(resource: &str, root: &Value) -> Option<Value> {
    match resource {
        "stream" | "streams" => root.get("streams").cloned(),
        "catalog" | "metas" => root.get("metas").cloned(),
        "meta" => root.get("meta").cloned(),
        "subtitles" | "subtitle" => root.get("subtitles").cloned(),
        other => root.get(other).cloned(),
    }
}

fn payload_is_empty(payload: &Value) -> bool {
    match payload {
        Value::Null => true,
        Value::Array(values) => values.is_empty(),
        Value::Object(values) => values.is_empty(),
        _ => false,
    }
}

pub(crate) enum ParsedAddonBody {
    Error(String),
    Success { payload: Value, root: Value },
}

pub(crate) fn parse_addon_body(
    resource: &str,
    url: &str,
    status_code: i32,
    body: Option<&str>,
) -> ParsedAddonBody {
    if !(200..=299).contains(&status_code) {
        return ParsedAddonBody::Error(
            json!({
                "kind": "network_error",
                "url": url,
                "statusCode": status_code
            })
            .to_string(),
        );
    }

    let Some(body) = body.map(str::trim).filter(|value| !value.is_empty()) else {
        return ParsedAddonBody::Error(
            json!({
                "kind": "empty",
                "url": url,
                "statusCode": status_code
            })
            .to_string(),
        );
    };

    let root: Value = match serde_json::from_str(body) {
        Ok(root) => root,
        Err(error) => {
            return ParsedAddonBody::Error(
                json!({
                    "kind": "parse_error",
                    "url": url,
                    "statusCode": status_code,
                    "error": error.to_string()
                })
                .to_string(),
            );
        }
    };

    let Some(payload) = resource_payload(resource, &root) else {
        return ParsedAddonBody::Error(
            json!({
                "kind": "empty",
                "url": url,
                "statusCode": status_code
            })
            .to_string(),
        );
    };

    if payload_is_empty(&payload) {
        return ParsedAddonBody::Error(
            json!({
                "kind": "empty",
                "url": url,
                "statusCode": status_code
            })
            .to_string(),
        );
    }

    ParsedAddonBody::Success { payload, root }
}

pub(crate) fn wrap_addon_resource_response_value(resource: &str, payload: Value) -> Value {
    match resource {
        "catalog" | "metas" => json!({ "metas": payload }),
        "stream" | "streams" => json!({ "streams": payload }),
        "meta" => json!({ "meta": payload }),
        "subtitle" | "subtitles" => json!({ "subtitles": payload }),
        _ => payload,
    }
}

pub(crate) fn normalize_addon_subtitles_json(subtitles_json: &str, resource_url: &str) -> String {
    let subtitles = serde_json::from_str::<Vec<Value>>(subtitles_json)
        .unwrap_or_default()
        .into_iter()
        .filter_map(|mut subtitle| {
            let object = subtitle.as_object_mut()?;
            let explicit_url = object
                .get("url")
                .and_then(Value::as_str)
                .filter(|value| !value.trim().is_empty())
                .map(str::to_string);
            let attribute_url = object
                .get("attributes")
                .and_then(Value::as_object)
                .and_then(|attributes| attributes.get("url"))
                .and_then(Value::as_str)
                .filter(|value| !value.trim().is_empty())
                .map(str::to_string);
            let resolved_url =
                resolve_resource_asset_url(explicit_url.or(attribute_url), resource_url)?;

            object.insert("url".to_string(), Value::String(resolved_url.clone()));
            let lang = object
                .get("lang")
                .and_then(Value::as_str)
                .filter(|value| !value.trim().is_empty())
                .map(str::to_string);
            let attributes = object
                .entry("attributes".to_string())
                .or_insert_with(|| json!({}));
            if !attributes.is_object() {
                *attributes = json!({});
            }
            let attributes = attributes.as_object_mut()?;
            let attribute_url_is_blank = attributes
                .get("url")
                .and_then(Value::as_str)
                .map(str::trim)
                .unwrap_or_default()
                .is_empty();
            if attribute_url_is_blank {
                attributes.insert("url".to_string(), Value::String(resolved_url));
            }
            let languages_empty = attributes
                .get("languages")
                .and_then(Value::as_array)
                .map(Vec::is_empty)
                .unwrap_or(true);
            if languages_empty && let Some(lang) = lang {
                attributes.insert("languages".to_string(), json!([lang]));
            }
            Some(subtitle)
        })
        .collect::<Vec<_>>();
    Value::Array(subtitles).to_string()
}

pub(crate) fn subtitle_tracks_json(input: &str) -> Option<String> {
    let args: Value = serde_json::from_str(input).ok()?;
    let subtitles = args.get("subtitles")?.as_array()?;
    let mut seen = std::collections::HashSet::new();
    let tracks = subtitles.iter().filter_map(|subtitle| {
        let object = subtitle.as_object()?;
        let attributes = object.get("attributes").and_then(Value::as_object);
        let url = object.get("url").and_then(Value::as_str)
            .or_else(|| attributes.and_then(|a| a.get("url")).and_then(Value::as_str))?.trim();
        if url.is_empty() || !seen.insert(url.to_string()) { return None; }
        let lang = object.get("lang").and_then(Value::as_str)
            .or_else(|| attributes.and_then(|a| a.get("language")).and_then(Value::as_str))
            .or_else(|| attributes.and_then(|a| a.get("lang")).and_then(Value::as_str));
        let label = object.get("label").and_then(Value::as_str)
            .or_else(|| object.get("name").and_then(Value::as_str))
            .or_else(|| attributes.and_then(|a| a.get("name")).and_then(Value::as_str))
            .or(lang).unwrap_or("Subtitle");
        Some(json!({"url": url, "lang": lang, "label": label, "addonName": object.get("addonName").and_then(Value::as_str)}))
    }).collect::<Vec<_>>();
    Some(Value::Array(tracks).to_string())
}

fn resolve_resource_asset_url(asset: Option<String>, resource_url: &str) -> Option<String> {
    let secure = addons::sources::protocol::prefer_https_asset_url(asset?.as_str())?;
    if addons::sources::protocol::is_http_url(&secure) {
        return Some(secure);
    }
    if secure.starts_with('/') {
        let scheme_end = resource_url.find("://").map(|index| index + 3)?;
        let host_end = resource_url[scheme_end..]
            .find('/')
            .map(|index| scheme_end + index)
            .unwrap_or(resource_url.len());
        return addons::sources::protocol::prefer_https_asset_url(&format!(
            "{}{}",
            &resource_url[..host_end],
            secure
        ));
    }
    let base = resource_url
        .rsplit_once('/')
        .map(|(base, _)| format!("{base}/"))
        .unwrap_or_default();
    addons::sources::protocol::prefer_https_asset_url(&format!("{base}{secure}"))
}

#[cfg(test)]
mod tests;
