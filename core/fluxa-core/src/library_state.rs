mod artwork_diff;
mod continue_watching;
mod helpers;
mod library_lists;
mod playback_progress;

pub(crate) use artwork_diff::{
    continue_watching_card_fields_json, item_list_diff_json, item_list_new_entries_json,
    select_continue_watching_artwork_json, value_map_diff_json, watched_map_diff_json,
};
pub(crate) use continue_watching::{
    UP_NEXT_DURATION_SECONDS, UP_NEXT_POSITION_SECONDS, build_continue_watching_from_progress_json,
    compute_continue_watching_badges_json, continue_watching_source_plan_json,
    format_episode_line_json, is_episode_released, next_progress_info_plan_json,
    normalized_continue_watching_source, remember_last_watched_episodes_json,
    resolve_next_after_watched_json, resolve_next_episode_json,
};
pub(crate) use library_lists::{
    filter_home_continue_watching_json, is_up_next_continue_watching_item_json,
    library_continue_watching_items_json, library_watchlist_items_json,
    normalize_library_document_json, watched_video_ids_json,
};
pub(crate) use playback_progress::{
    clear_playback_progress_item_json, clear_playback_progress_plan_json,
    playback_progress_item_json, watched_state_items_json,
};
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};

    #[test]
    fn continue_watching_source_plan_selects_exactly_one_source() {
        let local: Value = serde_json::from_str(
            &continue_watching_source_plan_json(r#"{"source":"Fluxa"}"#).unwrap(),
        )
        .unwrap();
        assert_eq!(local["source"], "local");
        assert_eq!(local["provider"], Value::Null);
        assert_eq!(local["usesLocal"], true);

        let remote: Value = serde_json::from_str(
            &continue_watching_source_plan_json(r#"{"source":" SiMkL "}"#).unwrap(),
        )
        .unwrap();
        assert_eq!(remote["source"], "simkl");
        assert_eq!(remote["provider"], "simkl");
        assert_eq!(remote["usesLocal"], false);
    }

    #[test]
    fn continue_watching_keeps_resolved_up_next_placeholders() {
        let progress = json!({
            "tt0760437": {
                "meta": { "id": "tt0760437", "name": "Ben 10", "type": "series" },
                "timeOffset": 1,
                "duration": 1,
                "lastVideoId": "tt0760437:1:3",
                "lastEpisodeSeason": 1,
                "lastEpisodeNumber": 3,
                "continueWatchingBadge": "upNext",
                "continueWatchingEpisodeResolved": true,
                "savedAt": "2026-08-09T00:00:00Z"
            }
        });
        let result: Value = serde_json::from_str(
            &build_continue_watching_from_progress_json(&progress.to_string()).unwrap(),
        )
        .unwrap();
        assert_eq!(result.as_array().unwrap().len(), 1);
        assert_eq!(result[0]["id"], "tt0760437");
        assert_eq!(result[0]["continueWatchingBadge"], "upNext");
    }

    #[test]
    fn library_watchlist_items_excludes_removed_and_undated_entries() {
        let items = r#"[
            {"_id":"tt1","name":"Active","type":"movie","_mtime":"2026-01-01T00:00:00.000Z"},
            {"_id":"tt2","name":"Removed","type":"movie","removed":true,"_mtime":"2026-01-01T00:00:00.000Z"},
            {"_id":"tt3","name":"NoTimestamp","type":"movie"}
        ]"#;
        let result: Value =
            serde_json::from_str(&library_watchlist_items_json(items).unwrap()).unwrap();
        let ids: Vec<&str> = result
            .as_array()
            .unwrap()
            .iter()
            .map(|i| i["id"].as_str().unwrap())
            .collect();
        assert_eq!(ids, vec!["tt1"]);
        assert_eq!(result[0]["updatedAtMs"], json!(1767225600000i64));
    }

    #[test]
    fn continue_watching_items_exclude_removed_entries() {
        let items = r#"[
            {"_id":"tt1","name":"Active","type":"movie","state":{"timeOffset":100,"duration":1000,"flaggedWatched":0}},
            {"_id":"tt2","name":"Removed","type":"movie","removed":true,"state":{"timeOffset":100,"duration":1000,"flaggedWatched":0}}
        ]"#;
        let result: Value =
            serde_json::from_str(&library_continue_watching_items_json(items).unwrap()).unwrap();
        assert_eq!(result.as_array().unwrap().len(), 1);
        assert_eq!(result[0]["id"], "tt1");
    }

    #[test]
    fn watched_state_items_build_series_episode_payloads() {
        let items = watched_state_items_json(
            r#"{"id":"tt1","name":"Show","type":"series","poster":null,"background":"bg","logo":"logo"}"#,
            r#"[{"id":"tt1:1:2","name":null,"season":1,"number":2,"released":null,"thumbnail":"ep.jpg"}]"#,
            true,
            Some("2026-01-01T00:00:00.000Z"),
        )
        .and_then(|json| serde_json::from_str::<Value>(&json).ok())
        .expect("items");

        assert_eq!(
            items
                .get(0)
                .and_then(|item| item.get("_id"))
                .and_then(Value::as_str),
            Some("tt1:1:2")
        );
        assert_eq!(
            items
                .get(0)
                .and_then(|item| item.get("state"))
                .and_then(|state| state.get("flaggedWatched"))
                .and_then(Value::as_i64),
            Some(1)
        );
    }

    #[test]
    fn continue_watching_badges_advance_to_next_episode_and_drop_unconfirmed_finished_series() {
        let candidates = json!([{
            "id": "s1",
            "_id": "s1",
            "type": "series",
            "lastVideoId": "s1:1:2",
            "lastEpisodeSeason": 1,
            "lastEpisodeNumber": 2,
            "timeOffset": 1,
            "duration": 99999,
            "savedAt": "2020-02-01T00:00:00Z",
            "reason": "simkl",
        }]);
        let videos_by_series = json!({
            "s1": [
                { "id": "s1:1:2", "season": 1, "episode": 2, "released": "2020-01-01T00:00:00Z" },
                { "id": "s1:1:3", "season": 1, "episode": 3, "released": "2020-01-08T00:00:00Z" },
            ],
        });
        // s3 exists only via lastWatchedEpisodes (no real CW entry, no video data) —
        // it should be dropped rather than left as a zombie entry.
        let last_watched = json!({
            "s3": {
                "meta": { "type": "series", "name": "Only From Last Watched" },
                "lastVideoId": "s3:1:1",
                "lastEpisodeSeason": 1,
                "lastEpisodeNumber": 1,
                "watchedAt": "2020-01-01T00:00:00Z",
            },
        });
        let now_ms = chrono::DateTime::parse_from_rfc3339("2021-01-01T00:00:00Z")
            .unwrap()
            .timestamp_millis();

        let result = compute_continue_watching_badges_json(
            &candidates.to_string(),
            &videos_by_series.to_string(),
            &last_watched.to_string(),
            now_ms,
        )
        .and_then(|json| serde_json::from_str::<Value>(&json).ok())
        .expect("badges");
        let result = result.as_array().unwrap();

        assert_eq!(
            result.len(),
            1,
            "s3 (no video data, not from a real CW list) should be dropped"
        );
        assert_eq!(result[0]["id"], "s1");
        assert_eq!(result[0]["lastVideoId"], "s1:1:3");
        assert_eq!(result[0]["continueWatchingBadge"], "upNext");
    }

    #[test]
    fn continue_watching_badges_do_not_double_advance_resolved_up_next_entries() {
        let candidates = json!([{
            "id": "s1",
            "_id": "s1",
            "type": "series",
            "lastVideoId": "s1:2:3",
            "lastEpisodeSeason": 2,
            "lastEpisodeNumber": 3,
            "timeOffset": 1,
            "duration": 99999,
            "continueWatchingBadge": "upNext",
            "continueWatchingEpisodeResolved": true,
            "savedAt": "2020-02-01T00:00:00Z",
            "reason": "simkl",
        }]);
        let videos_by_series = json!({
            "s1": [
                { "id": "s1:2:3", "season": 2, "episode": 3, "released": "2020-01-01T00:00:00Z" },
                { "id": "s1:2:4", "season": 2, "episode": 4, "released": "2020-01-08T00:00:00Z" },
            ],
        });
        let now_ms = chrono::DateTime::parse_from_rfc3339("2021-01-01T00:00:00Z")
            .unwrap()
            .timestamp_millis();

        let result = compute_continue_watching_badges_json(
            &candidates.to_string(),
            &videos_by_series.to_string(),
            "{}",
            now_ms,
        )
        .and_then(|json| serde_json::from_str::<Value>(&json).ok())
        .expect("badges");
        let result = result.as_array().unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0]["lastVideoId"], "s1:2:3");
        assert_eq!(result[0]["lastEpisodeNumber"], 3);
        assert_eq!(result[0]["continueWatchingBadge"], "upNext");
        assert_eq!(result[0]["reason"], "simkl");
    }

    #[test]
    fn continue_watching_badges_count_only_released_episodes_ahead() {
        let candidates = json!([{
            "id": "s1",
            "_id": "s1",
            "type": "series",
            "lastVideoId": "s1:1:2",
            "lastEpisodeSeason": 1,
            "lastEpisodeNumber": 2,
            "timeOffset": 1,
            "duration": 99999,
            "savedAt": "2020-02-01T00:00:00Z",
            "reason": "simkl",
        }]);
        let videos_by_series = json!({
            "s1": [
                { "id": "s1:1:2", "season": 1, "episode": 2, "released": "2020-01-01T00:00:00Z" },
                { "id": "s1:1:3", "season": 1, "episode": 3, "released": "2020-01-08T00:00:00Z" },
                { "id": "s1:1:4", "season": 1, "episode": 4, "released": "2020-01-15T00:00:00Z" },
                { "id": "s1:1:5", "season": 1, "episode": 5, "released": "2099-01-01T00:00:00Z" },
            ],
        });
        let now_ms = chrono::DateTime::parse_from_rfc3339("2021-01-01T00:00:00Z")
            .unwrap()
            .timestamp_millis();

        let result = compute_continue_watching_badges_json(
            &candidates.to_string(),
            &videos_by_series.to_string(),
            "{}",
            now_ms,
        )
        .and_then(|json| serde_json::from_str::<Value>(&json).ok())
        .expect("badges");
        let result = result.as_array().unwrap();

        assert_eq!(result[0]["lastVideoId"], "s1:1:3");
        assert_eq!(result[0]["unwatchedAhead"], 1);
        assert_eq!(result[0]["reason"], "simkl");
    }
}
