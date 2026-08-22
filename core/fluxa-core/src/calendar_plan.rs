mod helpers;
mod meta_dates;
mod plans;
mod release_rows;
mod widget_notifications;

pub(crate) use meta_dates::{
    calendar_item_matches_month_json, calendar_items_from_meta_json, next_unaired_episode_json,
    partition_this_week_json,
};
pub(crate) use plans::{
    calendar_candidate_plan_json, calendar_content_plan_json, calendar_visibility_plan_json,
    desktop_calendar_read_plan_json,
};
pub(crate) use release_rows::{calendar_release_rows_json, calendar_season_candidates_json};
pub(crate) use widget_notifications::{
    calendar_notification_content_json, calendar_release_detection_json, calendar_widget_rows_json,
};
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};

    #[test]
    fn visibility_keeps_new_episodes_for_completed_series() {
        let request = json!({
            "items": [
                {"contentId":"tt2861424","title":"Rick and Morty","dateIso":"2026-07-25T03:00:00Z"},
                {"contentId":"tt2861424","title":"Rick and Morty","dateIso":"2026-07-27T03:00:00Z"}
            ],
            "completedItems": [{"id":"tt2861424","name":"Rick and Morty"}],
            "showCompleted": false,
            "todayIso": "2026-07-26"
        });
        let result: Value =
            serde_json::from_str(&calendar_visibility_plan_json(&request.to_string()).unwrap())
                .unwrap();
        let items = result.as_array().unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0]["dateIso"], "2026-07-27T03:00:00Z");
    }

    #[test]
    fn visibility_deduplicates_provider_copies_of_the_same_episode() {
        let request = json!({
            "items": [
                {"id":"tt2861424:episode-10:2026-07-27","title":"Rick and Morty","dateIso":"2026-07-27T03:00:00Z","seasonNumber":9,"episodeNumber":10,"poster":"poster.jpg"},
                {"contentId":"tt2861424","title":"Rick and Morty","dateIso":"2026-07-27T00:00:00Z","seasonNumber":9,"episodeNumber":10,"episodeTitle":"Field of Dreams"},
                {"contentId":"tt2861424","title":"Rick and Morty","dateIso":"2026-07-27T00:00:00Z","seasonNumber":9,"episodeNumber":10,"poster":"poster.jpg","episodeTitle":"Field of Dreams"}
            ],
            "completedItems": [],
            "showCompleted": true,
            "todayIso": "2026-07-26"
        });
        let result: Value =
            serde_json::from_str(&calendar_visibility_plan_json(&request.to_string()).unwrap())
                .unwrap();
        let items = result.as_array().unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0]["episodeTitle"], "Field of Dreams");
    }

    #[test]
    fn items_from_meta_keep_calendar_identity_for_deduplication() {
        let result: Value = serde_json::from_str(
            &calendar_items_from_meta_json(
                r#"{"id":"tt2861424","type":"series","name":"Rick and Morty","videos":[{"released":"2026-08-02T00:00:00Z","season":9,"episode":11,"name":"Episode"}]}"#,
                "2026-08",
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(result[0]["contentId"], "tt2861424");
        assert_eq!(result[0]["seriesId"], "tt2861424");
        assert_eq!(result[0]["metaType"], "series");
    }

    #[test]
    fn candidate_plan_merges_groups_and_deduplicates_content() {
        let result: Value = serde_json::from_str(
            &calendar_candidate_plan_json(
                r#"{"groups":[[{"id":"tt1","type":"series","name":"Library"}],[{"id":"tt1","type":"series","name":"Progress"},{"id":"tt2","type":"anime","name":"Provider"}]]}"#,
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(result.as_array().unwrap().len(), 2);
        assert_eq!(result[0]["name"], json!("Library"));
        assert_eq!(result[1]["id"], json!("tt2"));
    }

    #[test]
    fn release_rows_build_series_episodes_and_movie_releases() {
        let series: Value = serde_json::from_str(
            &calendar_release_rows_json(
                r#"{"meta":{"id":"tt1","type":"tv","name":"Show","poster":"poster.jpg"},"videos":[{"released":"2026-07-20T00:00:00Z","season":2,"number":1,"name":"Premiere"}],"monthPrefix":"2026-07","movieLabel":"Movie"}"#,
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(series[0]["subtitle"], json!("S2:E1 Premiere"));
        assert_eq!(series[0]["meta"]["id"], json!("tt1"));

        let movie: Value = serde_json::from_str(
            &calendar_release_rows_json(
                r#"{"meta":{"id":"tt2","type":"film","name":"Film","released":"2026-07-21"},"videos":[],"monthPrefix":"2026-07","movieLabel":"Movie"}"#,
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(movie[0]["subtitle"], json!("Movie"));
    }

    #[test]
    fn content_plan_filters_deduplicates_and_sorts_by_date_then_title() {
        let result: Value = serde_json::from_str(
            &calendar_content_plan_json(
                r#"{"monthPrefix":"2026-06","items":[
                    {"dateIso":"2026-06-15","metaId":"tt1","metaType":"series","title":"B","subtitle":"E2"},
                    {"dateIso":"2026-06-10","metaId":"tt2","metaType":"movie","title":"A"},
                    {"dateIso":"2026-06-15","metaId":"tt1","metaType":"series","title":"B","subtitle":"E2"},
                    {"dateIso":"2026-05-01","metaId":"tt3","metaType":"movie","title":"Old"}
                ]}"#,
            )
            .unwrap(),
        )
        .unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0]["metaId"], "tt2");
        assert_eq!(arr[1]["metaId"], "tt1");
    }

    #[test]
    fn season_candidates_covers_watched_next_and_last_season() {
        let result: Value = serde_json::from_str(
            &calendar_season_candidates_json(r#"{"seasonsCount":5,"lastVideoId":"tt1:2:3"}"#)
                .unwrap(),
        )
        .unwrap();
        let seasons: Vec<i64> = result
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_i64().unwrap())
            .collect();
        assert!(seasons.contains(&2));
        assert!(seasons.contains(&3));
        assert!(seasons.contains(&5));
    }

    #[test]
    fn widget_rows_truncates_to_max_rows() {
        let items = (0..6)
            .map(|i| {
                json!({
                    "dateIso": format!("2026-06-{:02}", i + 1),
                    "title": format!("Show {}", i),
                    "subtitle": "",
                    "seasonNumber": 1,
                    "episodeNumber": i + 1
                })
            })
            .collect::<Vec<_>>();
        let request = json!({"items": items, "maxRows": 4});
        let result: Value =
            serde_json::from_str(&calendar_widget_rows_json(&request.to_string()).unwrap())
                .unwrap();
        assert_eq!(result.as_array().unwrap().len(), 4);
    }

    #[test]
    fn notification_content_skips_already_notified_and_non_today_items() {
        let request = json!({
            "items": [
                {"dateIso":"2026-06-10","metaId":"tt1","metaType":"series","title":"Show","subtitle":"E1","seasonNumber":1,"episodeNumber":1},
                {"dateIso":"2026-06-11","metaId":"tt2","metaType":"series","title":"Show2","subtitle":"E1","seasonNumber":1,"episodeNumber":1}
            ],
            "todayIso": "2026-06-10",
            "alreadyNotifiedKeys": [":2026-06-10:tt1:E1"],
            "notificationsEnabled": true,
            "alertNewEpisodes": true
        });
        let result: Value = serde_json::from_str(
            &calendar_notification_content_json(&request.to_string()).unwrap(),
        )
        .unwrap();
        assert_eq!(result["items"].as_array().unwrap().len(), 0);
    }

    #[test]
    fn notification_content_returns_render_fields_and_bounded_stored_keys() {
        let request = json!({
            "items": [{"dateIso":"2026-06-10","metaId":"tt2","metaType":"series","title":"Show","subtitle":"Episode","episodeTitle":"Pilot"}],
            "todayIso": "2026-06-10",
            "alreadyNotifiedKeys": ["old-1", "old-2"],
            "maxStoredKeys": 2
        });
        let result: Value = serde_json::from_str(
            &calendar_notification_content_json(&request.to_string()).unwrap(),
        )
        .unwrap();
        assert_eq!(result["items"][0]["episodeTitle"], "Pilot");
        assert_eq!(result["storedKeys"].as_array().unwrap().len(), 2);
        assert_eq!(result["storedKeys"][0], "old-2");
    }

    #[test]
    fn next_unaired_episode_picks_earliest_future_date() {
        let now_ms = chrono::DateTime::parse_from_rfc3339("2026-06-16T00:00:00Z")
            .unwrap()
            .timestamp_millis();
        let videos = json!([
            {"id": "v1", "released": "2026-06-01T00:00:00Z"},
            {"id": "v2", "released": "2026-07-10T00:00:00Z"},
            {"id": "v3", "released": "2026-06-20T00:00:00Z"},
            {"id": "v4"}
        ]);
        let result: Value =
            serde_json::from_str(&next_unaired_episode_json(&videos.to_string(), now_ms).unwrap())
                .unwrap();
        assert_eq!(result["id"], "v3");
    }

    #[test]
    fn next_unaired_episode_returns_none_when_nothing_upcoming() {
        let now_ms = chrono::DateTime::parse_from_rfc3339("2026-06-16T00:00:00Z")
            .unwrap()
            .timestamp_millis();
        let videos = json!([
            {"id": "v1", "released": "2026-06-01T00:00:00Z"},
            {"id": "v2"}
        ]);
        assert!(next_unaired_episode_json(&videos.to_string(), now_ms).is_none());
    }

    #[test]
    fn release_detection_returns_only_today_items() {
        let request = json!({
            "todayIso": "2026-06-10",
            "items": [
                {"dateIso":"2026-06-10","metaId":"tt1"},
                {"dateIso":"2026-06-11","metaId":"tt2"},
                {"dateIso":"2026-06-10","metaId":"tt3"}
            ]
        });
        let result: Value =
            serde_json::from_str(&calendar_release_detection_json(&request.to_string()).unwrap())
                .unwrap();
        assert_eq!(result.as_array().unwrap().len(), 2);
    }
}
