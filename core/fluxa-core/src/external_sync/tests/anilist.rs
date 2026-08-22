use super::super::*;
use serde_json::{Value, json};

#[test]
fn extracts_anilist_id_from_link_url() {
    let meta = json!({"links": [{"name": "AniList", "category": "other", "url": "https://anilist.co/anime/1535"}]});
    assert_eq!(extract_anilist_id_from_links(&meta), Some(1535));
}

#[test]
fn extract_anilist_id_returns_none_without_matching_link() {
    let meta = json!({"links": [{"name": "IMDb", "category": "other", "url": "https://imdb.com/title/tt1"}]});
    assert_eq!(extract_anilist_id_from_links(&meta), None);
}

#[test]
fn extract_anilist_id_skips_malformed_link_and_uses_next_match() {
    let meta = json!({"links": [
        {"name": "Bad", "category": "other", "url": "https://anilist.co/anime/"},
        {"name": "Good", "category": "other", "url": "https://anilist.co/anime/1535"},
    ]});
    assert_eq!(extract_anilist_id_from_links(&meta), Some(1535));
}

#[test]
fn search_match_prefers_exact_title_within_year_tolerance() {
    let args = json!({
        "meta": {"name": "Attack on Titan", "year": 2013},
        "candidates": [
            {"id": 1, "seasonYear": 2020, "title": {"romaji": "Something Else"}},
            {"id": 2, "seasonYear": 2013, "title": {"english": "Attack on Titan"}},
        ],
    });
    let result = anilist_search_best_match_json(&args.to_string()).unwrap();
    let parsed: Value = serde_json::from_str(&result).unwrap();
    assert_eq!(parsed["anilistId"], 2);
    assert_eq!(parsed["confidence"], "title-year");
}

#[test]
fn search_match_falls_back_to_year_only_when_no_title_matches() {
    let args = json!({
        "meta": {"name": "Some Show", "year": 2013},
        "candidates": [
            {"id": 3, "seasonYear": 2013, "title": {"romaji": "Different Name"}},
        ],
    });
    let result = anilist_search_best_match_json(&args.to_string()).unwrap();
    let parsed: Value = serde_json::from_str(&result).unwrap();
    assert_eq!(parsed["anilistId"], 3);
}

#[test]
fn media_list_status_completes_when_progress_reaches_total() {
    assert_eq!(anilist_media_list_status(12, 12), "COMPLETED");
    assert_eq!(anilist_media_list_status(12, 5), "CURRENT");
    assert_eq!(anilist_media_list_status(0, 5), "CURRENT");
}

#[test]
fn anilist_current_entries_build_progress_and_watched_keys() {
    let entries = r#"[
            {"status":"CURRENT","progress":3,"updatedAt":1700000000,"media":{"id":5,"title":{"romaji":"Show"},"episodes":12}},
            {"status":"PLANNING","media":{"id":6,"title":{"english":"Other"}}}
        ]"#;
    let plan = anilist_entries_to_sync(
        serde_json::from_str::<Vec<Value>>(entries)
            .unwrap()
            .as_slice(),
        0,
        None,
        false,
    );
    assert_eq!(plan["watching"][0]["lastVideoId"], "anilist:5:1:3");
    assert_eq!(plan["watched"]["anilist:5:1:2"], Value::Bool(true));
    assert_eq!(
        plan["progress"]["anilist:5"]["savedAt"],
        "2023-11-14T22:13:20.000Z"
    );
    assert_eq!(plan["watchlist"][0]["inWatchlist"], Value::Bool(true));
    assert_eq!(plan["watchlist"][0]["name"], "Other");
    assert_eq!(plan["watchlist"][0]["updatedAtMs"], json!(0));
}

#[test]
fn anilist_planning_entry_carries_updated_at_ms_when_present() {
    let entries = r#"[
            {"status":"PLANNING","updatedAt":1700000000,"media":{"id":9,"title":{"romaji":"Planned"}}}
        ]"#;
    let plan = anilist_entries_to_sync(
        serde_json::from_str::<Vec<Value>>(entries)
            .unwrap()
            .as_slice(),
        0,
        None,
        false,
    );
    assert_eq!(plan["watchlist"][0]["updatedAtMs"], json!(1700000000000i64));
}

#[test]
fn merge_by_id_overlays_incoming_fields_onto_local_items() {
    let local: Vec<Value> =
        serde_json::from_str(r#"[{"id":"a","name":"Old","poster":"p"},{"id":"b","name":"Keep"}]"#)
            .unwrap();
    let incoming: Vec<Value> =
        serde_json::from_str(r#"[{"id":"a","name":"New"},{"id":"c","name":"Added"}]"#).unwrap();
    let merged = merge_library_items_by_id(&local, &incoming);
    assert_eq!(merged[0]["name"], "New");
    assert_eq!(merged[0]["poster"], "p");
    assert_eq!(merged[1]["name"], "Keep");
    assert_eq!(merged[2]["name"], "Added");
}

#[test]
fn anilist_save_media_list_entry_variables_parses_media_id() {
    let json = anilist_save_media_list_entry_variables_json("anilist:5", "PLANNING", None).unwrap();
    let value: Value = serde_json::from_str(&json).unwrap();
    assert_eq!(value["mediaId"], json!(5));
    assert_eq!(value["status"], json!("PLANNING"));
    assert!(value.get("progress").is_none());
}

#[test]
fn anilist_save_media_list_entry_variables_includes_progress_when_given() {
    let json =
        anilist_save_media_list_entry_variables_json("anilist:5", "COMPLETED", Some(12)).unwrap();
    let value: Value = serde_json::from_str(&json).unwrap();
    assert_eq!(value["progress"], json!(12));
}

#[test]
fn anilist_save_media_list_entry_variables_rejects_non_anilist_ids() {
    assert_eq!(
        anilist_save_media_list_entry_variables_json("tt1234567", "PLANNING", None),
        None
    );
}
