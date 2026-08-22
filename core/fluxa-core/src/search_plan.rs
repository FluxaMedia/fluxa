mod addon_catalog;
mod detail_nav;
mod discover;
mod library_sort;
mod search;

pub(crate) use addon_catalog::{
    build_metadata_feed_options_json, discover_catalog_options_json, discover_content_types_json,
    resolve_feed_option_genre_json, resolve_transport_url_json,
};
pub(crate) use detail_nav::{detail_season_load_plan_json, detail_series_lookup_id};
pub(crate) use discover::{
    discover_selection_plan_json, discover_sort_plan_json, merge_discover_pages_json,
};
pub(crate) use library_sort::library_sort_plan_json;
pub(crate) use search::{
    merge_search_sources_json, recent_searches_plan_json, search_result_grouping_json,
    search_screen_plan_json, search_suggestions_plan_json,
};
#[cfg(test)]
mod tests {
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
    fn discover_sort_filters_by_content_type() {
        let result: Value = serde_json::from_str(
            &discover_sort_plan_json(
                r#"{"contentTypeFilter":"movie","items":[{"id":"tt1","type":"movie"},{"id":"tt2","type":"series"}]}"#,
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(result["items"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn library_sort_filters_by_type() {
        let result: Value = serde_json::from_str(
            &library_sort_plan_json(
                r#"{"typeFilter":"movie","items":[{"id":"tt1","type":"movie"},{"id":"tt2","type":"series"}]}"#,
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(result["items"].as_array().unwrap().len(), 1);
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

    #[test]
    fn discover_selection_plan_falls_back_to_default_when_extra_is_required() {
        let catalogs = serde_json::json!([{
            "key": "tmdb.year", "type": "movie",
            "extras": [{"name": "genre", "options": ["2026", "2025"], "isRequired": true, "default": "2026"}]
        }]);
        let result: Value = serde_json::from_str(
            &discover_selection_plan_json(
                &serde_json::json!({"contentType": "movie", "catalogs": catalogs}).to_string(),
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(result["extraValue"], "2026");

        let optional_catalogs = serde_json::json!([{
            "key": "tmdb.top", "type": "movie",
            "extras": [{"name": "genre", "options": ["Action", "Comedy"], "isRequired": false}]
        }]);
        let optional_result: Value = serde_json::from_str(
            &discover_selection_plan_json(
                &serde_json::json!({"contentType": "movie", "catalogs": optional_catalogs})
                    .to_string(),
            )
            .unwrap(),
        )
        .unwrap();
        assert!(optional_result["extraValue"].is_null());
    }

    #[test]
    fn series_lookup_id_extracts_imdb_id() {
        assert_eq!(detail_series_lookup_id("tt1234567:1:2"), "tt1234567");
        assert_eq!(detail_series_lookup_id("tt9999999"), "tt9999999");
    }

    #[test]
    fn series_lookup_id_strips_episode_parts_for_non_imdb() {
        assert_eq!(detail_series_lookup_id("kitsu:777:1:2"), "kitsu:777");
        assert_eq!(detail_series_lookup_id("tmdb:12345:1:2"), "tmdb:12345");
    }

    #[test]
    fn season_load_plan_uses_saved_season_when_valid() {
        let result: Value = serde_json::from_str(
            &detail_season_load_plan_json(r#"{"savedVideoId":"tt1:3:2","seasonsCount":5}"#)
                .unwrap(),
        )
        .unwrap();
        assert_eq!(result["firstSeasonToLoad"], 3);
    }

    #[test]
    fn season_load_plan_defaults_to_season_1_when_no_saved() {
        let result: Value =
            serde_json::from_str(&detail_season_load_plan_json(r#"{"seasonsCount":5}"#).unwrap())
                .unwrap();
        assert_eq!(result["firstSeasonToLoad"], 1);
    }
}
