mod discussion;
mod helpers;
mod lists;
mod media_ratings;
mod scrobble;
mod user;
mod watchlist_sync;

pub(crate) use discussion::{
    mdblist_discussion_comment_delete_plan, mdblist_discussion_comment_like_plan,
    mdblist_discussion_comment_update_plan, mdblist_discussion_create_plan,
    mdblist_discussion_hot_url, mdblist_discussion_replies_url,
    mdblist_discussion_reply_create_plan, mdblist_discussion_reply_delete_plan,
    mdblist_discussion_reply_like_plan, mdblist_discussion_reply_update_plan,
    mdblist_discussion_summary_url, mdblist_discussion_url,
};
pub(crate) use helpers::{mdblist_bearer, mdblist_device_poll_outcome};
pub(crate) use lists::{
    mdblist_list_by_name_url, mdblist_list_changes_url, mdblist_list_create_plan,
    mdblist_list_delete_plan, mdblist_list_info_url, mdblist_list_items_mutate_plan,
    mdblist_list_items_response_to_metas_json, mdblist_list_items_url, mdblist_list_like_plan,
    mdblist_list_membership_url, mdblist_list_update_plan, mdblist_lists_curated_url,
    mdblist_lists_liked_url, mdblist_lists_official_url, mdblist_lists_recommended_url,
    mdblist_lists_search_url, mdblist_lists_top_url, mdblist_lists_user_url,
};
pub(crate) use media_ratings::{
    mdblist_catalog_url, mdblist_genres_url, mdblist_media_info_batch_plan, mdblist_media_info_url,
    mdblist_media_ratings_from_response_json, mdblist_ratings_batch_plan, mdblist_search_url,
    mdblist_watchprovider_links_url,
};
pub(crate) use scrobble::{mdblist_checkin_plan, mdblist_scrobble_plan};
pub(crate) use user::{
    mdblist_public_user_url, mdblist_user_follow_plan, mdblist_user_stats_url, mdblist_user_url,
};
pub(crate) use watchlist_sync::{
    mdblist_sync_get_url, mdblist_sync_mutate_plan, mdblist_upnext_url,
    mdblist_watchlist_items_url, mdblist_watchlist_mutate_plan,
};
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};

    #[test]
    fn builds_media_info_and_batch_requests() {
        assert_eq!(
            mdblist_media_info_url("imdb", "movie", "tt0111161", Some("ratings")),
            "https://api.mdblist.com/imdb/movie/tt0111161/?append_to_response=ratings"
        );
        let plan: Value = serde_json::from_str(
            &mdblist_media_info_batch_plan(
                "imdb",
                "movie",
                &["tt1".to_string(), "tt2".to_string()],
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(plan["method"], "POST");
        assert_eq!(plan["url"], "https://api.mdblist.com/imdb/movie/");
        assert_eq!(plan["body"]["ids"], json!(["tt1", "tt2"]));
        assert!(mdblist_media_info_batch_plan("imdb", "movie", &[]).is_none());
    }

    #[test]
    fn normalizes_ratings_response_into_a_flat_map() {
        let response = json!({
            "ratings": [
                { "source": "imdb", "value": 9.3 },
                { "source": "tmdb", "score": 88 },
                { "source": "myanimelist", "value": null, "score": null },
                { "source": "metacriticuser", "value": null, "score": 75 }
            ]
        })
        .to_string();
        let normalized: Value =
            serde_json::from_str(&mdblist_media_ratings_from_response_json(&response).unwrap())
                .unwrap();
        assert_eq!(normalized["imdb"], 9.3);
        assert_eq!(normalized["tmdb"], 88);
        assert!(normalized.get("myanimelist").is_none());
        assert_eq!(normalized["metacriticuser"], 75);
    }

    #[test]
    fn builds_catalog_url_with_repeated_genre_params() {
        let url = mdblist_catalog_url(
            "movie",
            r#"{"genre": ["action", "comedy"], "year_min": 2020, "sort": "score"}"#,
        )
        .unwrap();
        assert!(url.starts_with("https://api.mdblist.com/catalog/movie?"));
        assert!(url.contains("genre=action"));
        assert!(url.contains("genre=comedy"));
        assert!(url.contains("year_min=2020"));
        assert!(url.contains("sort=score"));
    }

    #[test]
    fn resolves_list_items_url_variants() {
        assert_eq!(
            mdblist_list_items_url(r#"{"listId": 42}"#, "{}").unwrap(),
            "https://api.mdblist.com/lists/42/items"
        );
        assert_eq!(
            mdblist_list_items_url(
                r#"{"username": "alice", "listName": "top-picks"}"#,
                r#"{"limit": 20}"#
            )
            .unwrap(),
            "https://api.mdblist.com/lists/alice/top-picks/items?limit=20"
        );
        assert_eq!(
            mdblist_list_items_url(r#"{"officialSlug": "oscar-winners"}"#, "{}").unwrap(),
            "https://api.mdblist.com/lists/official/oscar-winners/items"
        );
    }

    #[test]
    fn converts_list_items_response_to_metas() {
        let response = json!({
            "movies": [{ "id": 1, "title": "Movie A", "ids": { "imdb": "tt1" }, "release_year": 2020 }],
            "shows": [{ "id": 2, "title": "Show B", "ids": { "tmdb": 55 } }]
        })
        .to_string();
        let metas: Value =
            serde_json::from_str(&mdblist_list_items_response_to_metas_json(&response).unwrap())
                .unwrap();
        let metas = metas.as_array().unwrap();
        assert_eq!(metas.len(), 2);
        assert_eq!(metas[0]["id"], "tt1");
        assert_eq!(metas[0]["type"], "movie");
        assert_eq!(metas[1]["id"], "tmdb:55");
        assert_eq!(metas[1]["type"], "series");
    }

    #[test]
    fn builds_list_mutation_plans() {
        let items = json!([
            { "type": "movie", "imdbId": "tt1" },
            { "type": "series", "tmdbId": 99 }
        ])
        .to_string();
        let plan: Value =
            serde_json::from_str(&mdblist_list_items_mutate_plan(7, "add", &items).unwrap())
                .unwrap();
        assert_eq!(plan["url"], "https://api.mdblist.com/lists/7/items/add");
        assert_eq!(plan["body"]["movies"][0]["ids"]["imdb"], "tt1");
        assert_eq!(plan["body"]["shows"][0]["ids"]["tmdb"], 99);
    }

    #[test]
    fn builds_sync_and_watchlist_mutation_plans() {
        let items = json!([{ "type": "movie", "imdbId": "tt1" }]).to_string();
        let add: Value =
            serde_json::from_str(&mdblist_sync_mutate_plan("watched", false, &items).unwrap())
                .unwrap();
        assert_eq!(add["url"], "https://api.mdblist.com/sync/watched");
        let remove: Value =
            serde_json::from_str(&mdblist_sync_mutate_plan("watched", true, &items).unwrap())
                .unwrap();
        assert_eq!(remove["url"], "https://api.mdblist.com/sync/watched/remove");
        assert!(mdblist_sync_mutate_plan("bogus", false, &items).is_none());

        let watchlist_items = json!([{ "type": "movie", "imdbId": "tt1" }]).to_string();
        let watchlist: Value =
            serde_json::from_str(&mdblist_watchlist_mutate_plan("add", &watchlist_items).unwrap())
                .unwrap();
        assert_eq!(
            watchlist["url"],
            "https://api.mdblist.com/watchlist/items/add"
        );

        let rated_items = json!([{ "type": "movie", "imdbId": "tt1", "rating": 8 }]).to_string();
        let rate: Value = serde_json::from_str(
            &mdblist_sync_mutate_plan("ratings", false, &rated_items).unwrap(),
        )
        .unwrap();
        assert_eq!(rate["body"]["movies"][0]["rating"], 8);
    }

    #[test]
    fn builds_scrobble_and_checkin_plans() {
        let movie_args = json!({ "ids": {"imdb": "tt1"}, "progress": 42.5 }).to_string();
        let start: Value =
            serde_json::from_str(&mdblist_scrobble_plan("start", &movie_args).unwrap()).unwrap();
        assert_eq!(start["url"], "https://api.mdblist.com/scrobble/start");
        assert_eq!(start["body"]["progress"], 42.5);
        assert_eq!(start["body"]["movie"]["ids"]["imdb"], "tt1");
        assert!(mdblist_scrobble_plan("bogus", &movie_args).is_none());

        let episode_args = json!({
            "ids": {"imdb": "tt2"},
            "isEpisode": true,
            "season": 1,
            "episode": 3,
            "progress": 10
        })
        .to_string();
        let episode_start: Value =
            serde_json::from_str(&mdblist_scrobble_plan("start", &episode_args).unwrap()).unwrap();
        assert_eq!(episode_start["body"]["show"]["ids"]["imdb"], "tt2");
        assert_eq!(episode_start["body"]["show"]["season"], 1);
        assert_eq!(episode_start["body"]["show"]["episode"], 3);
        assert!(episode_start["body"].get("movie").is_none());

        let checkin_start: Value =
            serde_json::from_str(&mdblist_checkin_plan("POST", &movie_args).unwrap()).unwrap();
        assert_eq!(checkin_start["method"], "POST");
        assert_eq!(checkin_start["body"]["movie"]["ids"]["imdb"], "tt1");
        let checkin_stop: Value =
            serde_json::from_str(&mdblist_checkin_plan("DELETE", "{}").unwrap()).unwrap();
        assert_eq!(checkin_stop["body"], Value::Null);
    }

    #[test]
    fn builds_discussion_plans() {
        assert_eq!(
            mdblist_discussion_url("tmdb", "movie", 1),
            "https://api.mdblist.com/discussion/tmdb/movie/1"
        );
        let create: Value = serde_json::from_str(
            &mdblist_discussion_create_plan("tmdb", "movie", 1, "great film").unwrap(),
        )
        .unwrap();
        assert_eq!(create["body"]["content"], "great film");
        assert!(mdblist_discussion_create_plan("tmdb", "movie", 1, "  ").is_none());
    }

    #[test]
    fn device_poll_outcome_reads_oauth_error_field() {
        assert_eq!(
            mdblist_device_poll_outcome(r#"{"error":"authorization_pending"}"#),
            "pending"
        );
        assert_eq!(
            mdblist_device_poll_outcome(r#"{"error":"expired_token"}"#),
            "expired"
        );
        assert_eq!(
            mdblist_device_poll_outcome(r#"{"access_token":"tok"}"#),
            "success"
        );
    }
}
