use super::super::*;
use serde_json::{Value, json};

#[test]
fn calendar_completion_plans_os_side_effects_in_core() {
    let handle = create_headless_engine("{}");
    let requested: Value = serde_json::from_str(
            &headless_engine_dispatch_json(
                handle,
                r#"{"type":"calendarMonthRequested","profile":{"id":"p1","language":"tr"},"year":2026,"month":5}"#,
            )
            .unwrap(),
        )
        .unwrap();

    let completed: Value = serde_json::from_str(
        &headless_engine_complete_effect_json(
            handle,
            &json!({
                "effectId": requested["effects"][0]["id"].as_str().unwrap(),
                "status": "ok",
                "value": {
                    "items": [{ "dateIso": "2026-05-20", "title": "Episode" }],
                    "localItems": [{ "id": "tt1" }],
                    "externalItems": [{ "id": "tt2" }]
                }
            })
            .to_string(),
        )
        .unwrap(),
    )
    .unwrap();

    assert_eq!(completed["state"]["calendar"]["isLoading"], false);
    assert_eq!(
        completed["state"]["calendar"]["items"][0]["title"],
        "Episode"
    );
    assert_eq!(
        completed["effects"]
            .as_array()
            .unwrap()
            .iter()
            .map(|effect| effect["type"].as_str().unwrap())
            .collect::<Vec<_>>(),
        vec![
            "updateCalendarWidget",
            "notifyReleasedEpisodes",
            "replaceExternalContinueWatching"
        ]
    );
    assert_eq!(completed["effects"][0]["payload"]["profile"]["id"], "p1");
    assert_eq!(completed["effects"][2]["payload"]["items"][0]["id"], "tt2");
    assert!(destroy_headless_engine(handle));
}
