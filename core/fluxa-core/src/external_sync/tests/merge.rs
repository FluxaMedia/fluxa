use super::super::*;
use serde_json::{Value, json};

#[test]
fn replace_external_continue_watching_sorts_by_saved_at_descending() {
    let items = json!([
        {"id": "tt1", "reason": "Nuvio", "timeOffset": 100, "duration": 1000, "savedAt": "2026-07-16T16:18:15Z"},
        {"id": "tt2", "reason": "Nuvio", "timeOffset": 100, "duration": 1000, "savedAt": "2026-07-18T22:15:23Z"},
        {"id": "tt3", "reason": "Nuvio", "timeOffset": 100, "duration": 1000, "savedAt": "2026-07-17T19:11:51Z"},
    ]);
    let result = replace_external_continue_watching_json(
        "[]",
        Some("Nuvio"),
        &items.to_string(),
        None,
        None,
        None,
    );
    let parsed: Vec<Value> = serde_json::from_str(&result).unwrap();
    let ids: Vec<&str> = parsed.iter().map(|v| v["id"].as_str().unwrap()).collect();
    assert_eq!(ids, vec!["tt2", "tt3", "tt1"]);
}

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

#[test]
fn timestamped_merge_pushes_removal_when_local_is_newer() {
    let local = json!([{"id": "a", "active": false, "updatedAt": 2000}]).to_string();
    let remote = json!([{"id": "a", "updatedAt": 1000}]).to_string();
    let result: Value =
        serde_json::from_str(&merge_watchlist_timestamped_json(&local, &remote)).unwrap();
    assert_eq!(result["toPushRemote"]["remove"], json!(["a"]));
    assert_eq!(result["toApplyLocal"]["add"], json!([]));
}

#[test]
fn timestamped_merge_reapplies_locally_when_remote_is_newer() {
    let local = json!([{"id": "a", "active": false, "updatedAt": 1000}]).to_string();
    let remote = json!([{"id": "a", "updatedAt": 2000}]).to_string();
    let result: Value =
        serde_json::from_str(&merge_watchlist_timestamped_json(&local, &remote)).unwrap();
    assert_eq!(result["toApplyLocal"]["add"], json!(["a"]));
    assert_eq!(result["toPushRemote"]["remove"], json!([]));
}

#[test]
fn timestamped_merge_pushes_local_only_additions() {
    let local = json!([{"id": "a", "active": true, "updatedAt": 1000}]).to_string();
    let remote = json!([]).to_string();
    let result: Value =
        serde_json::from_str(&merge_watchlist_timestamped_json(&local, &remote)).unwrap();
    assert_eq!(result["toPushRemote"]["add"], json!(["a"]));
}

#[test]
fn timestamped_merge_imports_remote_only_additions() {
    let local = json!([]).to_string();
    let remote = json!([{"id": "a", "updatedAt": 1000}]).to_string();
    let result: Value =
        serde_json::from_str(&merge_watchlist_timestamped_json(&local, &remote)).unwrap();
    assert_eq!(result["toApplyLocal"]["add"], json!(["a"]));
}
