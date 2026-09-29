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
    let empty = parse_addon_resource_result_json("catalog", "url", 200, Some(r#"{"metas":[]}"#));
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
