use super::*;

#[test]
fn manifest_requires_name_version_and_scrapers() {
    assert!(parse_plugin_manifest_json(r#"{"name":"","version":"1.0","scrapers":[]}"#).is_err());
    assert!(parse_plugin_manifest_json(r#"{"name":"Repo","version":"","scrapers":[]}"#).is_err());
    assert!(
        parse_plugin_manifest_json(r#"{"name":"Repo","version":"1.0","scrapers":[]}"#).is_err()
    );
}

#[test]
fn manifest_accepts_a_well_formed_payload() {
    let payload = r#"{
        "name": "Phisher's Repo",
        "version": "1.0.0",
        "scrapers": [
            {"id":"MoviesDrive","name":"MoviesDrive","version":"1.1.1","filename":"src/providers/moviesdrive.js"}
        ]
    }"#;
    let result = parse_plugin_manifest_json(payload);
    assert!(result.is_ok());
    let manifest: PluginManifest = serde_json::from_str(&result.unwrap()).unwrap();
    assert_eq!(manifest.scrapers[0].supported_types, vec!["movie", "tv"]);
    assert!(manifest.scrapers[0].enabled);
}

#[test]
fn execution_plan_filters_disabled_scrapers_and_parses_episode_locator() {
    let plan = plugin_execution_plan_json(
        r#"{"contentId":"tt0944947:2:3","mediaType":"series","scrapers":[{"id":"enabled","filename":"enabled.js","enabled":true},{"id":"disabled","filename":"disabled.js","enabled":false}]}"#,
    )
    .unwrap();
    let value: Value = serde_json::from_str(&plan).unwrap();
    assert_eq!(value["contentId"], "tt0944947");
    assert_eq!(value["season"], 2);
    assert_eq!(value["episode"], 3);
    assert_eq!(value["scrapers"].as_array().unwrap().len(), 1);
}

#[test]
fn update_plan_returns_only_newer_installed_plugins() {
    let plan = plugin_update_plan_json(
        r#"{"installed":[{"internalName":"one","version":2},{"internalName":"two","version":4}],"available":[{"internalName":"one","version":3},{"internalName":"two","version":4},{"internalName":"missing","version":9}]}"#,
    )
    .unwrap();
    let value: Value = serde_json::from_str(&plan).unwrap();
    assert_eq!(value["updates"][0]["internalName"], "one");
    assert_eq!(value["updates"][0]["version"], 3);
    assert_eq!(value["updates"].as_array().unwrap().len(), 1);
}

#[test]
fn stream_results_accept_string_and_object_url_shapes() {
    let raw = r#"[
        {"title":"1080p","url":"https://example.com/a.mp4"},
        {"name":"720p","url":{"url":"https://example.com/b.mp4"}}
    ]"#;
    let parsed = parse_plugin_stream_results_json(raw);
    let results: Vec<PluginStreamResult> = serde_json::from_str(&parsed).unwrap();
    assert_eq!(results.len(), 2);
    assert_eq!(results[0].title, "1080p");
    assert_eq!(results[1].title, "720p");
    assert_eq!(results[1].url, "https://example.com/b.mp4");
}

#[test]
fn stream_results_drop_entries_without_a_usable_url() {
    let raw = r#"[{"title":"no url"},{"title":"blank","url":""}]"#;
    assert_eq!(parse_plugin_stream_results_json(raw), "[]");
}

#[test]
fn empty_stream_results_produce_an_unavailable_stream_marker() {
    let parsed = plugin_stream_results_to_streams_json("[]");
    let streams: Vec<Stream> = serde_json::from_str(&parsed).unwrap();
    assert_eq!(streams.len(), 1);
    assert!(streams[0].url.is_none());
    assert_eq!(streams[0].extra["pluginUnavailable"], true);
    assert_eq!(
        streams[0].extra["pluginUnavailableReason"],
        "no_playable_stream"
    );
}

#[test]
fn unavailable_reason_is_preserved_on_the_stream_marker() {
    let parsed =
        plugin_stream_results_to_streams_json(r#"[{"unavailableReason":"Content was removed."}]"#);
    let streams: Vec<Stream> = serde_json::from_str(&parsed).unwrap();
    assert_eq!(
        streams[0].extra["pluginUnavailableReason"],
        "Content was removed."
    );
}

#[test]
fn stream_results_map_onto_the_addon_stream_shape() {
    let raw = r#"[{
        "title": "1080p",
        "url": "https://example.com/a.mp4",
        "quality": "1080p",
        "provider": "MoviesDrive",
        "seeders": 12,
        "headers": {"Referer": "https://example.com"},
        "subtitles": [{"url": "https://example.com/sub.srt", "language": "en", "name": "English"}]
    }]"#;
    let parsed = plugin_stream_results_to_streams_json(raw);
    let streams: Vec<Stream> = serde_json::from_str(&parsed).unwrap();
    assert_eq!(streams.len(), 1);
    let stream = &streams[0];
    assert_eq!(stream.url.as_deref(), Some("https://example.com/a.mp4"));
    assert_eq!(stream.name.as_deref(), Some("MoviesDrive"));
    assert_eq!(stream.title.as_deref(), Some("1080p"));
    assert_eq!(stream.extra["quality"], "1080p");
    assert_eq!(stream.extra["seeders"], 12);
    assert_eq!(
        stream
            .headers
            .as_ref()
            .unwrap()
            .get("Referer")
            .map(String::as_str),
        Some("https://example.com")
    );
    let subs = stream.subtitle_tracks.as_ref().unwrap();
    assert_eq!(subs[0].lang, "en");
    assert_eq!(subs[0].url, "https://example.com/sub.srt");
    let subs_alias = stream.subtitles.as_ref().unwrap();
    assert_eq!(subs_alias[0].url, "https://example.com/sub.srt");
}

#[test]
fn hdfilmizle_style_headers_survive_the_full_desktop_stream_pipeline() {
    let raw = r#"[{
        "name":"HDFilmizle",
        "title":"1080p",
        "url":"https://cdn.example/master.m3u8",
        "type":"hls",
        "headers":{
            "Referer":"https://vidrame.example/embed/abc",
            "Origin":"https://vidrame.example",
            "User-Agent":"Mozilla/5.0"
        }
    }]"#;
    let streams = plugin_stream_results_to_streams_json(raw);
    let stream = serde_json::from_str::<Vec<Stream>>(&streams)
        .unwrap()
        .pop()
        .unwrap();
    let plan =
        crate::player::playback::policy::stream_shell_plan_json(&stream_to_json(&stream)).unwrap();
    let plan: Value = serde_json::from_str(&plan).unwrap();

    assert_eq!(
        plan["requestHeaders"]["Referer"],
        "https://vidrame.example/embed/abc"
    );
    assert_eq!(plan["requestHeaders"]["Origin"], "https://vidrame.example");
    assert_eq!(plan["requestHeaders"]["User-Agent"], "Mozilla/5.0");
}

fn stream_to_json(stream: &Stream) -> String {
    serde_json::to_string(stream).unwrap()
}

#[test]
fn stream_results_tolerate_malformed_input() {
    assert_eq!(parse_plugin_stream_results_json("not json"), "[]");
    assert_eq!(parse_plugin_stream_results_json("{}"), "[]");
}
