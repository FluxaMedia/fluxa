mod collections;
mod library_commands;
mod plans;
mod remote_collection;

pub(crate) use collections::*;
pub(crate) use library_commands::{
    library_apply_mark_watched_json, library_command_plan_json, merge_progress_meta_json,
    playback_progress_write_plan_json,
};
pub(crate) use plans::{
    library_collection_import_validation_json, library_external_merge_plan_json,
    library_offline_grouping_json, playback_progress_merge_plan_json, watchlist_toggle_plan_json,
};
pub(crate) use remote_collection::{
    remote_collection_request_plan_json, remote_collection_response_plan_json,
};
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};

    #[test]
    fn toggle_plan_adds_when_not_in_watchlist() {
        let result: Value = serde_json::from_str(
            &watchlist_toggle_plan_json(
                r#"{"item":{"id":"tt1","type":"movie"},"isCurrentlyInWatchlist":false}"#,
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(result["command"], "add");
        assert_eq!(result["optimisticIsInWatchlist"], true);
    }

    #[test]
    fn toggle_plan_removes_when_in_watchlist() {
        let result: Value = serde_json::from_str(
            &watchlist_toggle_plan_json(
                r#"{"item":{"id":"tt1","type":"movie"},"isCurrentlyInWatchlist":true}"#,
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(result["command"], "remove");
        assert_eq!(result["optimisticIsInWatchlist"], false);
    }

    #[test]
    fn external_merge_deduplicates_preferring_local() {
        let result: Value = serde_json::from_str(
            &library_external_merge_plan_json(
                r#"{"localItems":[{"id":"tt1","source":"local"}],"externalItems":[{"id":"tt1","source":"external"},{"id":"tt2","source":"external"}]}"#,
            )
            .unwrap(),
        )
        .unwrap();
        let merged = result["merged"].as_array().unwrap();
        assert_eq!(merged.len(), 2);
        assert_eq!(merged[0]["source"], "local");
        assert_eq!(merged[1]["source"], "external");
        assert_eq!(merged[1]["id"], "tt2");
    }

    #[test]
    fn progress_meta_merge_keeps_existing_art_when_incoming_is_blank() {
        let merged = merge_progress_meta_json(
            r#"{"id":"tt1","poster":"","background":"","logo":""}"#,
            r#"{"id":"tt1","poster":"p.jpg","background":"b.jpg","logo":"l.png"}"#,
        );
        let result: Value = serde_json::from_str(&merged).unwrap();
        assert_eq!(result["poster"], "p.jpg");
        assert_eq!(result["background"], "b.jpg");
        assert_eq!(result["logo"], "l.png");
    }

    #[test]
    fn collection_import_validation_rejects_missing_id() {
        let result: Value = serde_json::from_str(
            &library_collection_import_validation_json(
                r#"{"collections":[{"title":"My List"},{"id":"c1","title":"Valid"}]}"#,
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(result["isValid"], false);
        assert_eq!(result["validCollections"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn imports_nuvio_trakt_sources_without_dropping_the_list_id() {
        let result: Value = serde_json::from_str(
            &import_collections_json(r#"[{"id":"streaming","title":"Streaming","folders":[{"id":"netflix","title":"Netflix","sources":[{"provider":"trakt","mediaType":"MOVIE","traktListId":34808160,"sortBy":"rank","sortHow":"asc"},{"provider":"trakt","mediaType":"TV","traktListId":34808679,"sortBy":"rank","sortHow":"asc"}]}]}]"#).unwrap(),
        ).unwrap();
        let sources = result[0]["folders"][0]["sources"].as_array().unwrap();
        assert_eq!(sources.len(), 2);
        assert_eq!(sources[0]["traktListId"], 34808160);
        assert_eq!(sources[1]["mediaType"], "TV");
    }

    #[test]
    fn imports_nuvio_addon_sources_and_folder_artwork() {
        let result: Value = serde_json::from_str(
            &import_collections_json(r#"[{"id":"streaming","title":"Streaming","backdropImageUrl":"https://img.example/backdrop.jpg","folders":[{"id":"catalog","title":"Catalog","tileShape":"wide","heroVideoUrl":"https://video.example/hero.mp4","sources":[{"provider":"addon","addonId":"addon.example","type":"series","catalogId":"top","genre":"Drama"}]}]}]"#).unwrap(),
        ).unwrap();
        let collection = &result[0];
        let folder = &collection["folders"][0];
        assert_eq!(
            collection["backdropImageUrl"],
            "https://img.example/backdrop.jpg"
        );
        assert_eq!(folder["heroVideoUrl"], "https://video.example/hero.mp4");
        assert_eq!(folder["shape"], "wide");
        assert_eq!(folder["sources"][0]["addonId"], "addon.example");
        assert_eq!(folder["sources"][0]["genre"], "Drama");
    }

    #[test]
    fn nuvio_collection_round_trip_preserves_nested_fields() {
        let input = r#"[{"id":"collection","title":"Collection","backdropImageUrl":"https://img.example/backdrop.jpg","futureCollectionField":{"enabled":true},"folders":[{"id":"folder","title":"Folder","tileShape":"wide","heroVideoUrl":"https://video.example/hero.mp4","futureFolderField":[1,2],"sources":[{"provider":"tmdb","tmdbSourceType":"LIST","tmdbId":42,"mediaType":"MOVIE","filters":{"withGenres":"28"},"futureSourceField":"kept"}]}]}]"#;
        let imported = import_collections_json(input).expect("imported");
        let exported = export_collections_json(&imported).expect("exported");
        let result: Value = serde_json::from_str(&exported).expect("json");
        let collection = &result[0];
        let folder = &collection["folders"][0];

        assert_eq!(collection["futureCollectionField"]["enabled"], true);
        assert_eq!(folder["heroVideoUrl"], "https://video.example/hero.mp4");
        assert_eq!(folder["futureFolderField"], json!([1, 2]));
        assert_eq!(folder["sources"][0]["filters"]["withGenres"], "28");
        assert_eq!(folder["sources"][0]["futureSourceField"], "kept");
    }

    #[test]
    fn offline_grouping_partitions_by_status() {
        let result: Value = serde_json::from_str(
            &library_offline_grouping_json(
                r#"{"items":[{"id":"a","status":"ready"},{"id":"b","status":"downloading"},{"id":"c","status":"failed"},{"id":"d"}]}"#,
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(result["ready"].as_array().unwrap().len(), 1);
        assert_eq!(result["downloading"].as_array().unwrap().len(), 1);
        assert_eq!(result["failed"].as_array().unwrap().len(), 1);
        assert_eq!(result["queued"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn progress_merge_preserves_existing_fields_when_incoming_is_null() {
        let result: Value = serde_json::from_str(
            &playback_progress_merge_plan_json(
                r#"{"existing":{"lastStreamUrl":"http://old","lastVideoId":"v1","timeOffset":1000},"incoming":{"lastVideoId":"v1","timeOffset":2000,"duration":5000}}"#,
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(result["timeOffset"], 2000);
        assert_eq!(result["lastStreamUrl"], "http://old");
        assert_eq!(result["videoChanged"], false);
    }

    #[test]
    fn progress_merge_keeps_prior_episode_number_on_video_change_with_incomplete_incoming() {
        let result: Value = serde_json::from_str(
            &playback_progress_merge_plan_json(
                r#"{"existing":{"lastVideoId":"v1","lastEpisodeSeason":1,"lastEpisodeNumber":5,"lastEpisodeName":"Old Name"},"incoming":{"lastVideoId":"v2","timeOffset":0,"duration":5000}}"#,
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(result["videoChanged"], true);
        assert_eq!(result["lastEpisodeSeason"], 1);
        assert_eq!(result["lastEpisodeNumber"], 5);
        assert_eq!(result["lastEpisodeName"], "Old Name");
    }
}
