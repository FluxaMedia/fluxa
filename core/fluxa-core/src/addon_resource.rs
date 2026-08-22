use crate::addon_protocol;
use crate::repository_flow::normalize_stream;
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

fn cache_value(root: &Value, key: &str) -> Value {
    root.get(key)
        .and_then(Value::as_i64)
        .map(|value| json!(value))
        .unwrap_or(Value::Null)
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

pub(crate) fn parse_addon_resource_result_json(
    resource: &str,
    url: &str,
    status_code: i32,
    body: Option<&str>,
) -> String {
    match parse_addon_body(resource, url, status_code, body) {
        ParsedAddonBody::Error(json_str) => json_str,
        ParsedAddonBody::Success { payload, root } => json!({
            "kind": "success",
            "url": url,
            "statusCode": status_code,
            "cacheMaxAge": cache_value(&root, "cacheMaxAge"),
            "staleRevalidate": cache_value(&root, "staleRevalidate"),
            "staleError": cache_value(&root, "staleError"),
            "valueJson": payload.to_string()
        })
        .to_string(),
    }
}

pub(crate) fn parse_addon_stream_result_json(
    url: &str,
    status_code: i32,
    body: Option<&str>,
    addon_name: &str,
) -> String {
    match parse_addon_body("stream", url, status_code, body) {
        ParsedAddonBody::Error(json_str) => json_str,
        ParsedAddonBody::Success { payload, root } => {
            let streams = match payload {
                Value::Array(items) => items
                    .into_iter()
                    .map(|stream| normalize_stream(stream, addon_name))
                    .collect(),
                other => vec![normalize_stream(other, addon_name)],
            };
            json!({
                "kind": "success",
                "url": url,
                "statusCode": status_code,
                "cacheMaxAge": cache_value(&root, "cacheMaxAge"),
                "staleRevalidate": cache_value(&root, "staleRevalidate"),
                "staleError": cache_value(&root, "staleError"),
                "valueJson": Value::Array(streams).to_string()
            })
            .to_string()
        }
    }
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

pub(crate) fn parse_catalog_items_json(body: &str, fallback_type: &str) -> Option<String> {
    let root: Value = serde_json::from_str(body).ok()?;
    let metas = root.get("metas")?.as_array()?;
    let mut items = Vec::with_capacity(metas.len());
    for meta in metas {
        let object = meta.as_object()?;
        let id = object.get("id")?.as_str()?.to_string();
        let title = object.get("name")?.as_str()?.to_string();
        let content_type = object
            .get("type")
            .and_then(Value::as_str)
            .filter(|value| !value.trim().is_empty())
            .unwrap_or(fallback_type)
            .to_string();
        items.push(json!({
            "id": id,
            "type": content_type,
            "title": title,
            "subtitle": object.get("releaseInfo").and_then(Value::as_str).unwrap_or(""),
            "artworkUrl": object.get("poster").and_then(Value::as_str),
            "logoUrl": object.get("logo").and_then(Value::as_str),
            "backgroundUrl": object.get("background").and_then(Value::as_str),
            "description": object.get("description").and_then(Value::as_str)
        }));
    }
    serde_json::to_string(&items).ok()
}

pub(crate) fn parse_direct_streams_json(body: &str) -> Option<String> {
    let root: Value = serde_json::from_str(body).ok()?;
    let streams = root.get("streams")?.as_array()?;
    let parsed = streams
        .iter()
        .filter_map(|stream| {
            let object = stream.as_object()?;
            let playable_url = object
                .get("url")
                .and_then(Value::as_str)
                .filter(|url| {
                    ["http://", "https://", "magnet://", "stremio://"]
                        .iter()
                        .any(|prefix| url.to_ascii_lowercase().starts_with(prefix))
                })
                .map(str::to_string)
                .or_else(|| {
                    let info_hash = object
                        .get("infoHash")
                        .and_then(Value::as_str)
                        .filter(|value| !value.trim().is_empty())?;
                    let file_index = object.get("fileIdx").and_then(Value::as_i64);
                    Some(match file_index {
                        Some(index) => format!("stremio://torrent/{info_hash}/{index}"),
                        None => format!("stremio://torrent/{info_hash}"),
                    })
                })?;
            let request_headers = object
                .get("behaviorHints")
                .and_then(Value::as_object)
                .and_then(|hints| hints.get("proxyHeaders"))
                .and_then(Value::as_object)
                .and_then(|headers| headers.get("request"))
                .and_then(Value::as_object)
                .map(|headers| {
                    headers
                        .iter()
                        .filter_map(|(name, value)| {
                            value.as_str().map(|value| (name.clone(), Value::String(value.to_string())))
                        })
                        .collect::<serde_json::Map<_, _>>()
                })
                .unwrap_or_default();
            Some(json!({
                "title": object.get("title").and_then(Value::as_str).map(str::trim).filter(|value| !value.is_empty()),
                "playableUrl": playable_url,
                "requestHeaders": request_headers
            }))
        })
        .collect::<Vec<_>>();
    serde_json::to_string(&parsed).ok()
}

fn resolve_resource_asset_url(asset: Option<String>, resource_url: &str) -> Option<String> {
    let secure = addon_protocol::prefer_https_asset_url(asset?.as_str())?;
    if addon_protocol::is_http_url(&secure) {
        return Some(secure);
    }
    if secure.starts_with('/') {
        let scheme_end = resource_url.find("://").map(|index| index + 3)?;
        let host_end = resource_url[scheme_end..]
            .find('/')
            .map(|index| scheme_end + index)
            .unwrap_or(resource_url.len());
        return addon_protocol::prefer_https_asset_url(&format!(
            "{}{}",
            &resource_url[..host_end],
            secure
        ));
    }
    let base = resource_url
        .rsplit_once('/')
        .map(|(base, _)| format!("{base}/"))
        .unwrap_or_default();
    addon_protocol::prefer_https_asset_url(&format!("{base}{secure}"))
}

#[cfg(test)]
mod tests {
    use super::{
        normalize_addon_subtitles_json, parse_addon_resource_result_json, parse_catalog_items_json,
        parse_direct_streams_json,
    };
    use serde_json::Value;

    #[test]
    fn stream_payload_keeps_provider_order_and_content() {
        let result = parse_addon_resource_result_json(
            "stream",
            "https://addon.example/stream/movie/tt1.json",
            200,
            Some(
                r#"{"streams":[{"title":"B"},{"title":"A"}],"cacheMaxAge":3600,"staleRevalidate":120,"staleError":60}"#,
            ),
        );
        let result: Value = serde_json::from_str(&result).expect("result json");
        let value_json = result
            .get("valueJson")
            .and_then(Value::as_str)
            .expect("value json");
        let streams: Value = serde_json::from_str(value_json).expect("streams");

        assert_eq!(result.get("kind").and_then(Value::as_str), Some("success"));
        assert_eq!(
            streams
                .get(0)
                .and_then(|item| item.get("title"))
                .and_then(Value::as_str),
            Some("B")
        );
        assert_eq!(
            streams
                .get(1)
                .and_then(|item| item.get("title"))
                .and_then(Value::as_str),
            Some("A")
        );
        assert_eq!(
            result.get("cacheMaxAge").and_then(Value::as_i64),
            Some(3600)
        );
        assert_eq!(
            result.get("staleRevalidate").and_then(Value::as_i64),
            Some(120)
        );
        assert_eq!(result.get("staleError").and_then(Value::as_i64), Some(60));
    }

    #[test]
    fn catalog_items_apply_the_fallback_type() {
        let items = parse_catalog_items_json(
            r#"{"metas":[{"id":"tt1","name":"Example","poster":"poster.jpg"}]}"#,
            "movie",
        )
        .and_then(|json| serde_json::from_str::<Value>(&json).ok())
        .expect("catalog items");

        assert_eq!(items[0]["id"].as_str(), Some("tt1"));
        assert_eq!(items[0]["type"].as_str(), Some("movie"));
        assert_eq!(items[0]["artworkUrl"].as_str(), Some("poster.jpg"));
    }

    #[test]
    fn direct_streams_accept_torrents_and_proxy_headers() {
        let streams = parse_direct_streams_json(
            r#"{"streams":[{"title":"Torrent","infoHash":"abc","fileIdx":2,"behaviorHints":{"proxyHeaders":{"request":{"Authorization":"Bearer token"}}}}]}"#,
        )
        .and_then(|json| serde_json::from_str::<Value>(&json).ok())
        .expect("direct streams");

        assert_eq!(
            streams[0]["playableUrl"].as_str(),
            Some("stremio://torrent/abc/2")
        );
        assert_eq!(
            streams[0]["requestHeaders"]["Authorization"].as_str(),
            Some("Bearer token")
        );
    }

    #[test]
    fn empty_and_error_states_are_classified_without_platform_code() {
        let empty =
            parse_addon_resource_result_json("catalog", "url", 200, Some(r#"{"metas":[]}"#));
        let network = parse_addon_resource_result_json("catalog", "url", 503, Some("{}"));
        let parse = parse_addon_resource_result_json("catalog", "url", 200, Some("{"));

        assert_eq!(
            serde_json::from_str::<Value>(&empty)
                .ok()
                .and_then(|value| value.get("kind").and_then(Value::as_str).map(str::to_owned))
                .as_deref(),
            Some("empty")
        );
        assert_eq!(
            serde_json::from_str::<Value>(&network)
                .ok()
                .and_then(|value| value.get("kind").and_then(Value::as_str).map(str::to_owned))
                .as_deref(),
            Some("network_error")
        );
        assert_eq!(
            serde_json::from_str::<Value>(&parse)
                .ok()
                .and_then(|value| value.get("kind").and_then(Value::as_str).map(str::to_owned))
                .as_deref(),
            Some("parse_error")
        );
    }

    #[test]
    fn subtitle_payload_is_resolved_without_reordering_valid_entries() {
        let subtitles = normalize_addon_subtitles_json(
            r#"[
                {"id":"one","url":"/subs/one.vtt","lang":"en","attributes":{"languages":[]}},
                {"id":"drop","attributes":{}},
                {"id":"two","attributes":{"url":"two.srt"}}
            ]"#,
            "https://addon.example/subtitles/movie/tt1.json",
        );
        let subtitles: Value = serde_json::from_str(&subtitles).expect("subtitles");

        assert_eq!(subtitles.as_array().map(Vec::len), Some(2));
        assert_eq!(subtitles[0]["id"].as_str(), Some("one"));
        assert_eq!(
            subtitles[0]["url"].as_str(),
            Some("https://addon.example/subs/one.vtt")
        );
        assert_eq!(
            subtitles[0]["attributes"]["languages"][0].as_str(),
            Some("en")
        );
        assert_eq!(subtitles[1]["id"].as_str(), Some("two"));
        assert_eq!(
            subtitles[1]["url"].as_str(),
            Some("https://addon.example/subtitles/movie/two.srt")
        );
    }
}
