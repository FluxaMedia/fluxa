use super::*;
use serde_json::Value;

#[test]
fn search_grouping_separates_movies_series_and_other() {
    let result: Value = serde_json::from_str(
        &search_result_grouping_json(
            r#"{"query":"breaking","results":[
                {"id":"tt1","type":"series","name":"Breaking Bad"},
                {"id":"tt2","type":"movie","name":"Breaking"},
                {"id":"tt3","type":"other","name":"Another"}
            ]}"#,
        )
        .unwrap(),
    )
    .unwrap();
    let groups = result["groups"].as_array().unwrap();
    assert_eq!(groups[0]["type"], "movie");
    assert_eq!(groups[1]["type"], "series");
    assert_eq!(groups[2]["type"], "other");
}





#[test]
fn metadata_feed_options_preserve_custom_stremio_catalog_types() {
    let result: Value = serde_json::from_str(
        &build_metadata_feed_options_json(
            r#"[{"transportUrl":"https://aio.example/stremio/u/manifest.json","manifest":{"id":"aiometadata","name":"AIOMetadata","resources":["catalog"],"catalogs":[{"type":"anime.movie","id":"mal.top","name":"MAL Top"},{"type":"Trakt","id":"trakt.upnext","name":"Up Next"}]}}]"#,
        )
        .unwrap(),
    )
    .unwrap();
    let feeds = result.as_array().unwrap();
    assert_eq!(feeds.len(), 2);
    assert_eq!(feeds[0]["type"], "anime.movie");
    assert_eq!(feeds[1]["type"], "Trakt");
}

#[test]
fn collection_sources_match_normalized_aio_ids_and_dynamic_catalogs() {
    let source = serde_json::json!({
        "addonId": "aio-metadata",
        "catalogId": "tmdb.discover.movie.streaming.netflix",
        "type": "movie",
    });
    let addons = serde_json::json!([{
        "transportUrl": "https://aio.example/configured/manifest.json",
        "manifest": {
            "id": "com.aio.metadata",
            "catalogs": [{ "id": "tmdb.top", "type": "movie" }]
        }
    }]);

    assert_eq!(
        resolve_transport_url_json(&source.to_string(), &addons.to_string()),
        Some("\"https://aio.example/configured/manifest.json\"".to_string())
    );
}

#[test]
fn discover_catalog_options_expose_genre_extra_as_flat_list() {
    let result: Value = serde_json::from_str(
        &discover_catalog_options_json(
            r#"[{"transportUrl":"https://aio.example/stremio/u/manifest.json","manifest":{
                "id":"aiometadata","name":"AIOMetadata","resources":["catalog"],
                "catalogs":[{
                    "type":"movie","id":"tmdb.top","name":"TMDB Popular",
                    "extra":[{"name":"genre","options":["Action","Comedy"],"isRequired":false}]
                },{
                    "type":"movie","id":"tmdb.year","name":"TMDB By Year",
                    "extra":[{"name":"genre","options":["2026","2025"],"isRequired":true,"default":"2026"}]
                }]
            }}]"#,
            "movie",
        )
        .unwrap(),
    )
    .unwrap();
    let options = result.as_array().unwrap();
    assert_eq!(options.len(), 2);
    assert_eq!(
        options[0]["genres"].as_array().unwrap(),
        &[Value::from("Action"), Value::from("Comedy")]
    );
    assert_eq!(options[0]["requiresGenre"], false);
    assert!(options[0]["defaultGenre"].is_null());
    assert_eq!(options[1]["requiresGenre"], true);
    assert_eq!(options[1]["defaultGenre"], "2026");
}





