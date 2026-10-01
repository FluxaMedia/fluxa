use super::*;
use serde_json::{Value, json};


#[test]
fn simkl_library_items_share_watchlist_metadata_mapping() {
    let items = simkl_library_to_items_json(
        r#"[{"show":{"title":"Example","poster":"poster","fanart":"fanart","ids":{"imdb":"tt42"}}}]"#,
        r#"[{"movie":{"title":"Film","ids":{"tmdb":7}}}]"#,
    )
    .expect("items");
    let items: Value = serde_json::from_str(&items).unwrap();
    assert_eq!(items[0]["id"], "tt42");
    assert_eq!(items[0]["type"], "series");
    assert_eq!(items[0]["poster"], "https://simkl.in/posters/poster_m.jpg");
    assert_eq!(items[1]["id"], "tmdb:7");
    assert_eq!(items[1]["type"], "movie");
}

#[test]
fn simkl_library_items_preserve_string_and_fallback_ids() {
    let items = simkl_library_to_items_json(
        r#"[{"show":{"title":"Slug show","ids":{"simkl":5,"slug":"slug-show"}}}]"#,
        r#"[{"movie":{"title":"String movie","ids":{"tmdb":"7"}}}]"#,
    )
    .expect("items");
    let items: Value = serde_json::from_str(&items).unwrap();
    assert_eq!(items[0]["id"], "simkl:5");
    assert_eq!(items[1]["id"], "tmdb:7");
}



#[test]
fn simkl_anime_seasons_get_distinct_anime_ids() {
    let items: Value = serde_json::from_str(
        &simkl_library_to_items_json(
            r#"{"shows":[
                {"anime_type":"tv","show":{"title":"Jujutsu Kaisen","ids":{"simkl":1,"slug":"jjk","mal":40748,"anilist":113415,"imdb":"tt12343534"}}},
                {"anime_type":"tv","show":{"title":"Jujutsu Kaisen S2","ids":{"simkl":2,"slug":"jjk-2","mal":51009,"imdb":"tt12343534"}}},
                {"show":{"title":"Plain","ids":{"simkl":3,"imdb":"tt1"}}}
            ]}"#,
            "[]",
        )
        .unwrap(),
    )
    .unwrap();

    assert_eq!(items[0]["id"], "mal:40748");
    assert_eq!(items[1]["id"], "mal:51009");
    assert_eq!(items[0]["type"], "series");
    assert_eq!(items[0]["isAnime"], true);
    assert_eq!(items[2]["id"], "tt1");
    assert_eq!(items[2]["type"], "series");
    assert_eq!(items[2]["isAnime"], false);
}

#[test]
fn simkl_writes_accept_native_anime_ids() {
    let watchlist: Value = serde_json::from_str(
        &simkl_watchlist_body_json(
            &json!({"id": "mal:52991", "contentType": "series", "command": "add"}).to_string(),
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(watchlist["shows"][0]["ids"], json!({"mal": 52991}));
    assert_eq!(watchlist["shows"][0]["to"], "plantowatch");

    let by_simkl_id: Value = serde_json::from_str(
        &simkl_watchlist_body_json(
            &json!({"id": "simkl:11121", "contentType": "series", "command": "remove"}).to_string(),
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(by_simkl_id["shows"][0]["ids"], json!({"simkl": 11121}));

    let watched: Value = serde_json::from_str(
        &simkl_mark_watched_body_json(
            &json!({"videoIds": ["kitsu:46474:5"], "meta": {"type": "series"}}).to_string(),
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(watched["shows"][0]["ids"], json!({"kitsu": 46474}));
    assert_eq!(watched["shows"][0]["use_tvdb_anime_seasons"], false);
    assert_eq!(
        watched["shows"][0]["seasons"][0]["episodes"][0]["number"],
        5
    );
}

#[test]
fn simkl_anime_playback_lines_up_with_the_anime_library_id() {
    let watching = r#"{"shows":[{"anime":true,"show":{"title":"Frieren","ids":{"simkl":7,"mal":52991,"imdb":"tt22248376"}},"last_watched":"S01E04","next_to_watch":"S01E05"}]}"#;
    let items = simkl_watching_to_items_json(watching, "[]").unwrap();
    let merged: Value = serde_json::from_str(
        &simkl_merge_playback_progress_json(
            &items,
            r#"[{"id":9,"progress":30,"type":"episode","episode":{"season":1,"number":5},"anime":{"title":"Frieren","ids":{"simkl":7,"mal":52991,"imdb":"tt22248376"}}}]"#,
        )
        .unwrap(),
    )
    .unwrap();

    assert_eq!(merged[0]["id"], "mal:52991");
    assert_eq!(merged[0]["simklId"], 7);
}

#[test]
fn simkl_writes_carry_every_id_key_simkl_documents() {
    for (id, expected) in [
        ("simkl:49108", json!({"simkl": 49108})),
        ("tvdb:153021", json!({"tvdb": 153021})),
        ("tvdb:the-walking-dead", json!({"tvdb": "the-walking-dead"})),
        ("anisearch:2227", json!({"anisearch": 2227})),
        ("livechart:321", json!({"livechart": 321})),
        ("animeplanet:one-piece", json!({"animeplanet": "one-piece"})),
        (
            "letterboxd:the-truman-show",
            json!({"letterboxd": "the-truman-show"}),
        ),
        ("netflix:70210890", json!({"netflix": 70210890})),
        (
            "traktslug:john-wick-chapter-4-2023",
            json!({"traktslug": "john-wick-chapter-4-2023"}),
        ),
    ] {
        let body: Value = serde_json::from_str(
            &simkl_watchlist_body_json(&json!({"id": id, "contentType": "movie"}).to_string())
                .unwrap(),
        )
        .unwrap();
        assert_eq!(body["movies"][0]["ids"], expected, "{id}");
    }
}

#[test]
fn simkl_library_id_falls_back_to_tvdb_then_simkl_number() {
    let items: Value = serde_json::from_str(
        &simkl_library_to_items_json(
            r#"{"shows":[
                {"show":{"title":"A","ids":{"simkl":1,"slug":"a","tvdb":99}}},
                {"show":{"title":"B","ids":{"simkl":2,"slug":"b"}}}
            ]}"#,
            "[]",
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(items[0]["id"], "tvdb:99");
    assert_eq!(items[1]["id"], "simkl:2");
}

#[test]
fn simkl_writes_prefer_the_items_full_id_set() {
    let watchlist: Value = serde_json::from_str(
        &simkl_watchlist_body_json(
            &json!({
                "id": "tt1",
                "contentType": "series",
                "command": "add",
                "providerIds": {"simkl": 5, "imdb": "tt1", "slug": "x", "mal": 9, "empty": ""}
            })
            .to_string(),
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(
        watchlist["shows"][0]["ids"],
        json!({"simkl": 5, "imdb": "tt1", "mal": 9})
    );
}

#[test]
fn anime_video_ids_with_season_and_episode_parse_the_episode() {
    let target = simkl_target("mal:52991:1:5").unwrap();
    assert_eq!((target.season, target.episode), (1, Some(5)));
    let target = simkl_target("kitsu:46474:5").unwrap();
    assert_eq!((target.season, target.episode), (1, Some(5)));
}

#[test]
fn rewatch_marks_the_item_and_pins_the_session() {
    let body: Value = serde_json::from_str(
        &simkl_mark_watched_body_json(
            &json!({
                "videoIds": ["tt1"],
                "meta": {"type": "movie"},
                "rewatch": true,
                "rewatchId": 77
            })
            .to_string(),
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(body["movies"][0]["is_rewatch"], true);
    assert_eq!(body["movies"][0]["rewatch_id"], 77);

    let plain: Value = serde_json::from_str(
        &simkl_mark_watched_body_json(
            &json!({"videoIds": ["tt1"], "meta": {"type": "movie"}}).to_string(),
        )
        .unwrap(),
    )
    .unwrap();
    assert!(plain["movies"][0].get("is_rewatch").is_none());
}
