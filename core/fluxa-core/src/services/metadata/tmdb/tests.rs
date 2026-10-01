use super::*;
use serde_json::{Value, json};



#[test]
fn builtin_manifest_declares_no_stream_resource() {
    let manifest: Value = serde_json::from_str(&tmdb_builtin_manifest_json()).unwrap();
    let resources: Vec<&str> = manifest["resources"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect();
    assert!(resources.contains(&"catalog"));
    assert!(resources.contains(&"meta"));
    assert!(!resources.contains(&"stream"));
}





#[test]
fn catalog_url_maps_genre_name_to_id() {
    let url = tmdb_builtin_catalog_url("movie", &json!({"genre": "Horror"}), "KEY", "en");
    assert!(url.contains("3/discover/movie"));
    assert!(url.contains("with_genres=27"));
}

#[test]
fn catalog_url_maps_skip_to_page() {
    let url = tmdb_builtin_catalog_url("movie", &json!({"skip": 40}), "KEY", "en");
    assert!(url.contains("page=3"));
}








