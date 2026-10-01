use super::*;
use serde_json::{Value, json};

#[test]
fn continue_watching_source_plan_selects_exactly_one_source() {
    let local: Value =
        serde_json::from_str(&continue_watching_source_plan_json(r#"{"source":"Fluxa"}"#).unwrap())
            .unwrap();
    assert_eq!(local["source"], "local");
    assert_eq!(local["provider"], Value::Null);
    assert_eq!(local["usesLocal"], true);

    let remote: Value = serde_json::from_str(
        &continue_watching_source_plan_json(r#"{"source":" SiMkL "}"#).unwrap(),
    )
    .unwrap();
    assert_eq!(remote["source"], "simkl");
    assert_eq!(remote["provider"], "simkl");
    assert_eq!(remote["usesLocal"], false);
}

#[test]
fn a_nuvio_login_overrides_local_progress() {
    let plan: Value = serde_json::from_str(
        &continue_watching_source_plan_json(r#"{"source":"local","nuvioConnected":true}"#).unwrap(),
    )
    .unwrap();
    assert_eq!(plan["source"], "nuvio");
    assert_eq!(plan["usesLocal"], false);
}

#[test]
fn a_picked_tracker_beats_the_nuvio_login() {
    let plan: Value = serde_json::from_str(
        &continue_watching_source_plan_json(r#"{"source":"simkl","nuvioConnected":true}"#).unwrap(),
    )
    .unwrap();
    assert_eq!(plan["source"], "simkl");
}

#[test]
fn continue_watching_keeps_resolved_up_next_placeholders() {
    let progress = json!({
        "tt0760437": {
            "meta": { "id": "tt0760437", "name": "Ben 10", "type": "series" },
            "timeOffset": 1,
            "duration": 1,
            "lastVideoId": "tt0760437:1:3",
            "lastEpisodeSeason": 1,
            "lastEpisodeNumber": 3,
            "continueWatchingBadge": "upNext",
            "continueWatchingEpisodeResolved": true,
            "savedAt": "2026-08-09T00:00:00Z"
        }
    });
    let result: Value = serde_json::from_str(
        &build_continue_watching_from_progress_json(&progress.to_string()).unwrap(),
    )
    .unwrap();
    assert_eq!(result.as_array().unwrap().len(), 1);
    assert_eq!(result[0]["id"], "tt0760437");
    assert_eq!(result[0]["continueWatchingBadge"], "upNext");
}










