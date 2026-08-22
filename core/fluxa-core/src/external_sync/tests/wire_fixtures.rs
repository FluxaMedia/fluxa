use super::super::*;
use crate::player_scrobble;
use serde_json::Value;

#[test]
fn external_sync_wire_fixtures_preserve_provider_contracts() {
    let trakt_input: Value = serde_json::from_str(include_str!(
        "../../../tests/fixtures/external_sync/trakt_scrobble_plan_input.json"
    ))
    .unwrap();
    let trakt_expected: Value = serde_json::from_str(include_str!(
        "../../../tests/fixtures/external_sync/trakt_scrobble_plan_expected.json"
    ))
    .unwrap();
    let trakt_actual: Value = serde_json::from_str(
        &player_scrobble::trakt_scrobble_plan_json(
            &trakt_input["ids"].to_string(),
            trakt_input["isEpisode"].as_bool().unwrap(),
            None,
            None,
            trakt_input["timePosSec"].as_f64().unwrap(),
            trakt_input["durationSec"].as_f64().unwrap(),
            None,
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(trakt_actual, trakt_expected);

    let simkl_input =
        include_str!("../../../tests/fixtures/external_sync/simkl_mark_watched_input.json");
    let simkl_expected: Value = serde_json::from_str(include_str!(
        "../../../tests/fixtures/external_sync/simkl_mark_watched_expected.json"
    ))
    .unwrap();
    let simkl_actual: Value =
        serde_json::from_str(&simkl_mark_watched_body_json(simkl_input).unwrap()).unwrap();
    assert_eq!(simkl_actual, simkl_expected);

    let trakt_playback_expected: Value = serde_json::from_str(include_str!(
        "../../../tests/fixtures/external_sync/trakt_playback_expected.json"
    ))
    .unwrap();
    let trakt_playback_actual: Value = serde_json::from_str(
        &trakt_playback_items_to_library_json(include_str!(
            "../../../tests/fixtures/external_sync/trakt_playback_response.json"
        ))
        .unwrap(),
    )
    .unwrap();
    assert_eq!(trakt_playback_actual, trakt_playback_expected);

    let simkl_response: Value = serde_json::from_str(include_str!(
        "../../../tests/fixtures/external_sync/simkl_watched_response.json"
    ))
    .unwrap();
    let simkl_watched_expected: Value = serde_json::from_str(include_str!(
        "../../../tests/fixtures/external_sync/simkl_watched_expected.json"
    ))
    .unwrap();
    let simkl_watched_actual: Value = serde_json::from_str(
        &simkl_watched_to_ids_json(
            &simkl_response["shows"].to_string(),
            &simkl_response["movies"].to_string(),
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(simkl_watched_actual, simkl_watched_expected);
}
