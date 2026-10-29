use serde::Deserialize;
use serde_json::{Map, Value, json};
use std::collections::BTreeMap;

use crate::addons::plugins::streams::plugin_scraper_streams;

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct GatewayLink {
    source: String,
    name: String,
    url: String,
    referer: String,
    quality: i64,
    #[serde(rename = "type")]
    kind: String,
    headers: BTreeMap<String, String>,
}

pub fn normalize_gateway_url(raw: &str) -> Option<String> {
    let trimmed = raw.trim().trim_end_matches('/');
    if trimmed.is_empty() {
        return None;
    }
    let with_scheme = if trimmed.contains("://") {
        trimmed.to_string()
    } else {
        format!("http://{trimmed}")
    };
    let lower = with_scheme.to_ascii_lowercase();
    if !lower.starts_with("http://") && !lower.starts_with("https://") {
        return None;
    }
    Some(with_scheme)
}

pub fn gateway_links_to_streams(response: &Value, provider: &str) -> Vec<Value> {
    let links = response.get("links").and_then(Value::as_array);
    let items: Vec<Value> = links
        .into_iter()
        .flatten()
        .filter_map(|raw| serde_json::from_value::<GatewayLink>(raw.clone()).ok())
        .filter_map(link_item)
        .collect();
    plugin_scraper_streams(&Value::Array(items), provider)
}

fn link_item(link: GatewayLink) -> Option<Value> {
    if link.url.is_empty() || matches!(link.kind.as_str(), "MAGNET" | "TORRENT") {
        return None;
    }
    let mut headers = Map::new();
    for (key, value) in &link.headers {
        headers.insert(key.clone(), json!(value));
    }
    let has_referer = headers
        .keys()
        .any(|key| key.eq_ignore_ascii_case("referer"));
    if !link.referer.is_empty() && !has_referer {
        headers.insert("Referer".into(), json!(link.referer));
    }
    let name = [link.source.trim(), link.name.trim()]
        .into_iter()
        .find(|part| !part.is_empty())
        .unwrap_or_default();
    let mut item = Map::new();
    item.insert("url".into(), json!(link.url));
    item.insert("name".into(), json!(name));
    if link.quality > 0 {
        item.insert("quality".into(), json!(format!("{}p", link.quality)));
    }
    if !headers.is_empty() {
        item.insert("headers".into(), Value::Object(headers));
    }
    Some(Value::Object(item))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bare_host_gets_http_scheme_and_loses_trailing_slash() {
        assert_eq!(
            normalize_gateway_url(" 192.168.1.20:8787/ ").as_deref(),
            Some("http://192.168.1.20:8787")
        );
    }

    #[test]
    fn https_gateway_url_is_kept() {
        assert_eq!(
            normalize_gateway_url("https://gw.example.com").as_deref(),
            Some("https://gw.example.com")
        );
    }

    #[test]
    fn non_http_scheme_is_rejected() {
        assert!(normalize_gateway_url("ftp://gw.example.com").is_none());
        assert!(normalize_gateway_url("   ").is_none());
    }

    #[test]
    fn link_headers_and_referer_reach_the_stream() {
        let response = json!({"links": [{
            "source": "ExampleSource", "name": "ExampleSource", "url": "https://cdn.example.com/a.m3u8",
            "referer": "https://source.example.com/", "quality": 1080, "type": "M3U8",
            "headers": {"Origin": "https://source.example.com"}
        }]});
        let streams = gateway_links_to_streams(&response, "ExampleSource");
        assert_eq!(streams.len(), 1);
        assert_eq!(streams[0]["url"], "https://cdn.example.com/a.m3u8");
        let headers = &streams[0]["behaviorHints"]["proxyHeaders"]["request"];
        assert_eq!(headers["Referer"], "https://source.example.com/");
        assert_eq!(headers["Origin"], "https://source.example.com");
    }

    #[test]
    fn explicit_referer_header_is_not_duplicated() {
        let response = json!({"links": [{
            "source": "ExampleAnime", "url": "https://cdn.example.com/b.m3u8",
            "referer": "https://other.example/", "headers": {"referer": "https://zoko.example/"}
        }]});
        let streams = gateway_links_to_streams(&response, "ExampleAnime");
        let headers = streams[0]["behaviorHints"]["proxyHeaders"]["request"]
            .as_object()
            .unwrap();
        assert_eq!(headers.len(), 1);
        assert_eq!(headers["referer"], "https://zoko.example/");
    }

    #[test]
    fn magnet_and_empty_links_are_dropped() {
        let response = json!({"links": [
            {"source": "A", "url": "magnet:?xt=urn:btih:abc", "type": "MAGNET"},
            {"source": "B", "url": "", "type": "VIDEO"},
            {"source": "C", "url": "https://cdn.example.com/c.mp4", "type": "VIDEO"}
        ]});
        let streams = gateway_links_to_streams(&response, "X");
        assert_eq!(streams.len(), 1);
        assert_eq!(streams[0]["url"], "https://cdn.example.com/c.mp4");
    }
}
