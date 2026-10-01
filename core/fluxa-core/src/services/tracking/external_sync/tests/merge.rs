use super::super::*;
use serde_json::{Value, json};


#[test]
fn merge_continue_watching_sorts_the_combined_result_by_saved_at_descending() {
    let local = json!([
        {"id": "tt1", "savedAt": "2026-07-16T16:18:15Z"},
        {"id": "tt3", "savedAt": "2026-07-17T19:11:51Z"},
    ]);
    let external = json!([
        {"id": "tt2", "savedAt": "2026-07-18T22:15:23Z"},
    ]);
    let result = merge_continue_watching_lists_json(
        &local.to_string(),
        &external.to_string(),
        "{}",
        None,
        None,
    )
    .unwrap();
    let parsed: Vec<Value> = serde_json::from_str(&result).unwrap();
    let ids: Vec<&str> = parsed.iter().map(|v| v["id"].as_str().unwrap()).collect();
    assert_eq!(ids, vec!["tt2", "tt3", "tt1"]);
}
