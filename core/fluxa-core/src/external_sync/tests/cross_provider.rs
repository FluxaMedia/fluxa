use super::super::*;
use serde_json::Value;

#[test]
fn external_list_mappers_skip_invalid_records_and_keep_valid_ones() {
    let trakt: Vec<Value> = serde_json::from_str(
            &trakt_watchlist_to_items_json(
                r#"[{"movie":{"title":"Valid","ids":{"tmdb":7}}},{"movie":{"title":"Invalid","ids":{}}}]"#,
                "[]",
            )
            .expect("trakt items"),
        )
        .unwrap();
    assert_eq!(trakt.len(), 1);
    assert_eq!(trakt[0]["id"], "tmdb:7");

    let simkl: Vec<Value> = serde_json::from_str(
            &simkl_watchlist_to_items_json(
                r#"[{"show":{"title":"Valid","ids":{"tmdb":7}}},{"show":{"title":"Invalid","ids":{}}}]"#,
                "[]",
            )
            .expect("simkl items"),
        )
        .unwrap();
    assert_eq!(simkl.len(), 1);
    assert_eq!(simkl[0]["id"], "tmdb:7");
}

#[test]
fn watched_mappers_retain_tmdb_only_records() {
    let trakt: Value = serde_json::from_str(
        &trakt_watched_to_ids_json(r#"[{"movie":{"ids":{"tmdb":7}}}]"#, "[]")
            .expect("trakt watched"),
    )
    .unwrap();
    assert_eq!(trakt["tmdb:7"], Value::Bool(true));

    let simkl: Value = serde_json::from_str(
        &simkl_watched_to_ids_json("[]", r#"[{"movie":{"ids":{"tmdb":8}}}]"#)
            .expect("simkl watched"),
    )
    .unwrap();
    assert_eq!(simkl["tmdb:8"], Value::Bool(true));
}
