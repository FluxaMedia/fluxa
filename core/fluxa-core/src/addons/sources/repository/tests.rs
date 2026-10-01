use super::*;




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
        r#"[{"title":"A","headers":{"Referer":"https://hdfilmizle.example/"},"behaviorHints":{"videoHash":"abc","videoSize":12,"proxyHeaders":{"request":{"X-Proxy":"1"}}}},{"title":"B"}]"#,
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
    assert_eq!(
        value[0]["behaviorHints"]["requestHeaders"]["Referer"].as_str(),
        Some("https://hdfilmizle.example/")
    );
    assert_eq!(value[1]["title"].as_str(), Some("B"));
}

#[test]
fn usenet_and_archive_only_streams_are_dropped() {
    let value: Value = serde_json::from_str(&addon_streams_with_provider_json(
        r#"[{"nzbUrl":"https://x/a.nzb","servers":["nntps://h"]},{"rarUrls":[{"url":"https://x/a.rar"}]},{"url":"https://x/a.mkv","rarUrls":[]},{"infoHash":"abc"}]"#,
        "Addon",
    ))
    .unwrap();
    assert_eq!(value.as_array().unwrap().len(), 2);
}

#[test]
fn stream_provider_normalization_uses_legacy_title_as_description() {
    let streams = addon_streams_with_provider_json(
        r#"[{"name":"4k DV","title":"Torrentio details"},{"name":"1080p","title":"Legacy","description":"Original description"}]"#,
        "Torrentio",
    );
    let value: Value = serde_json::from_str(&streams).unwrap();
    assert_eq!(value[0]["title"].as_str(), Some("Torrentio details"));
    assert_eq!(value[0]["description"].as_str(), Some("Torrentio details"));
    assert_eq!(
        value[1]["description"].as_str(),
        Some("Original description")
    );
}
