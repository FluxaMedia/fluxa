mod anime_seasons;
mod catalogs;
mod episode_ratings;
mod helpers;
mod highlights;
mod lists;
mod mappings;
mod ratings;
mod resume;
mod skips;
mod votes;
mod watched;

pub(crate) use anime_seasons::{
    publicmetadb_anime_seasons_delete_chunk_plan, publicmetadb_anime_seasons_delete_mapping_plan,
    publicmetadb_anime_seasons_submit_plan, publicmetadb_anime_seasons_url,
};
pub(crate) use catalogs::{publicmetadb_catalog_items_url, publicmetadb_catalogs_url};
pub(crate) use episode_ratings::{
    publicmetadb_episode_ratings_batch_create_plan, publicmetadb_episode_ratings_batch_delete_plan,
    publicmetadb_episode_ratings_batch_url, publicmetadb_episode_ratings_create_plan,
    publicmetadb_episode_ratings_delete_plan, publicmetadb_episode_ratings_url,
};
pub(crate) use helpers::publicmetadb_bearer;
pub(crate) use highlights::{
    publicmetadb_highlights_create_plan, publicmetadb_highlights_delete_plan,
    publicmetadb_highlights_url,
};
pub(crate) use lists::{
    publicmetadb_list_items_add_plan, publicmetadb_list_items_remove_plan,
    publicmetadb_list_items_url, publicmetadb_lists_create_plan, publicmetadb_lists_delete_plan,
    publicmetadb_lists_url,
};
pub(crate) use mappings::{
    publicmetadb_mappings_create_plan, publicmetadb_mappings_delete_plan,
    publicmetadb_mappings_lookup_url, publicmetadb_mappings_url,
};
pub(crate) use ratings::{
    publicmetadb_ratings_create_plan, publicmetadb_ratings_delete_plan, publicmetadb_ratings_url,
};
pub(crate) use resume::{
    publicmetadb_resume_batch_plan, publicmetadb_resume_delete_plan, publicmetadb_resume_save_plan,
    publicmetadb_resume_url,
};
pub(crate) use skips::{
    publicmetadb_skips_create_plan, publicmetadb_skips_delete_plan, publicmetadb_skips_url,
};
pub(crate) use votes::{
    publicmetadb_votes_create_plan, publicmetadb_votes_delete_plan, publicmetadb_votes_url,
};
pub(crate) use watched::{
    publicmetadb_watched_bulk_delete_plan, publicmetadb_watched_delete_plan,
    publicmetadb_watched_edit_date_plan, publicmetadb_watched_mark_plan, publicmetadb_watched_url,
};

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};

    #[test]
    fn builds_bearer_header() {
        assert_eq!(publicmetadb_bearer("pm-abc"), "Bearer pm-abc");
    }

    #[test]
    fn builds_resume_requests() {
        assert_eq!(
            publicmetadb_resume_url(r#"{"tmdb_id":1399,"media_type":"tv"}"#),
            "https://publicmetadb.com/api/external/resume?tmdb_id=1399&media_type=tv"
        );
        let plan: Value = serde_json::from_str(
            &publicmetadb_resume_save_plan(
                r#"{"media_type":"movie","position_ms":1000,"runtime_ms":6000,"tmdb_id":42}"#,
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(plan["method"], "POST");
        assert_eq!(plan["url"], "https://publicmetadb.com/api/external/resume");
        assert_eq!(plan["body"]["tmdb_id"], 42);
        assert!(publicmetadb_resume_save_plan(r#"{"media_type":"movie"}"#).is_none());

        let items = json!({ "items": [{ "tmdb_id": 1, "media_type": "movie", "position_ms": 1, "runtime_ms": 2 }] })
            .to_string();
        assert!(publicmetadb_resume_batch_plan(&items).is_some());
        assert!(publicmetadb_resume_batch_plan(r#"{"items":[]}"#).is_none());
        assert_eq!(
            publicmetadb_resume_delete_plan("abc123")
                .and_then(|p| serde_json::from_str::<Value>(&p).ok())
                .map(|p| p["url"].clone()),
            Some(json!("https://publicmetadb.com/api/external/resume/abc123"))
        );
    }

    #[test]
    fn builds_watched_requests() {
        let mark: Value = serde_json::from_str(
            &publicmetadb_watched_mark_plan(
                r#"{"tmdb_id":1,"media_type":"movie","watched_at":null}"#,
                false,
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(mark["body"]["watched_at"], Value::Null);
        let deduped: Value = serde_json::from_str(
            &publicmetadb_watched_mark_plan(r#"{"tmdb_id":1,"media_type":"movie"}"#, true).unwrap(),
        )
        .unwrap();
        assert!(deduped["url"].as_str().unwrap().contains("dedupe=true"));

        assert!(
            publicmetadb_watched_bulk_delete_plan(r#"{"tmdb_id":1,"media_type":"movie"}"#)
                .is_some()
        );
        assert!(publicmetadb_watched_bulk_delete_plan(r#"{"tmdb_id":1}"#).is_none());
    }

    #[test]
    fn builds_skips_requests() {
        assert_eq!(
            publicmetadb_skips_url(r#"{"tmdb_id":1399,"media_type":"tv","season":1,"episode":1}"#),
            Some(
                "https://publicmetadb.com/api/external/skips?tmdb_id=1399&media_type=tv&season=1&episode=1"
                    .to_string()
            )
        );
        assert!(publicmetadb_skips_url(r#"{"tmdb_id":1399}"#).is_none());
        let create: Value = serde_json::from_str(
            &publicmetadb_skips_create_plan(
                r#"{"tmdb_id":1,"media_type":"tv","season":1,"episode":1,"intro_end_ms":60000}"#,
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(create["body"]["intro_end_ms"], 60000);
    }

    #[test]
    fn builds_episode_ratings_batch_requests() {
        let ratings = json!({
            "tmdb_id": 1,
            "media_type": "tv",
            "season": 1,
            "ratings": [{ "episode": 1, "score": 90 }]
        })
        .to_string();
        assert!(publicmetadb_episode_ratings_batch_create_plan(&ratings).is_some());

        let too_many = json!({
            "tmdb_id": 1,
            "media_type": "tv",
            "season": 1,
            "ratings": (0..51).map(|i| json!({ "episode": i, "score": 50 })).collect::<Vec<_>>()
        })
        .to_string();
        assert!(publicmetadb_episode_ratings_batch_create_plan(&too_many).is_none());

        assert!(publicmetadb_episode_ratings_batch_delete_plan(r#"{"ids":["a","b"]}"#).is_some());
        assert!(publicmetadb_episode_ratings_batch_delete_plan(r#"{"ids":[]}"#).is_none());
    }

    #[test]
    fn builds_anime_seasons_requests() {
        let submit = json!({
            "tmdb_id": 1,
            "season_number": 1,
            "chunks": [{ "tmdb_season": 1 }]
        })
        .to_string();
        assert!(publicmetadb_anime_seasons_submit_plan(&submit).is_some());
        assert!(
            publicmetadb_anime_seasons_submit_plan(
                &json!({ "tmdb_id": 1, "season_number": 1, "chunks": [] }).to_string()
            )
            .is_none()
        );
        assert!(
            publicmetadb_anime_seasons_delete_mapping_plan(r#"{"tmdb_id":1,"season_number":1}"#)
                .is_some()
        );
    }

    #[test]
    fn builds_votes_requests() {
        assert_eq!(
            publicmetadb_votes_url("skips", "skip123", true),
            Some("https://publicmetadb.com/api/external/skips/skip123/votes?all=true".to_string())
        );
        assert!(publicmetadb_votes_url("bogus", "id", false).is_none());
        assert!(publicmetadb_votes_create_plan("ratings", "r1", 1).is_some());
        assert!(publicmetadb_votes_create_plan("ratings", "r1", 2).is_none());
    }

    #[test]
    fn builds_lists_and_catalogs_requests() {
        assert_eq!(
            publicmetadb_catalogs_url(),
            "https://publicmetadb.com/api/external/catalogs"
        );
        assert_eq!(
            publicmetadb_catalog_items_url("pick1", r#"{"page":2}"#),
            Some("https://publicmetadb.com/api/external/catalogs/pick1/items?page=2".to_string())
        );
        assert!(
            publicmetadb_list_items_add_plan("", r#"{"tmdb_id":1,"media_type":"movie"}"#).is_none()
        );
        let add: Value = serde_json::from_str(
            &publicmetadb_list_items_add_plan("list1", r#"{"tmdb_id":1,"media_type":"movie"}"#)
                .unwrap(),
        )
        .unwrap();
        assert_eq!(
            add["url"],
            "https://publicmetadb.com/api/external/lists/list1/items"
        );
    }
}
