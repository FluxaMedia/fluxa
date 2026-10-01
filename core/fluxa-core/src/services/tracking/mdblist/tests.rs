use super::*;
use serde_json::{Value, json};





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
fn builds_sync_and_watchlist_mutation_plans() {
    let items = json!([{ "type": "movie", "imdbId": "tt1" }]).to_string();
    let add: Value =
        serde_json::from_str(&mdblist_sync_mutate_plan("watched", false, &items).unwrap()).unwrap();
    assert_eq!(add["url"], "https://api.mdblist.com/sync/watched");
    let remove: Value =
        serde_json::from_str(&mdblist_sync_mutate_plan("watched", true, &items).unwrap()).unwrap();
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
    let rate: Value =
        serde_json::from_str(&mdblist_sync_mutate_plan("ratings", false, &rated_items).unwrap())
            .unwrap();
    assert_eq!(rate["body"]["movies"][0]["rating"], 8);
}



