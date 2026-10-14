use super::super::*;
use crate::runtime::EffectKind;
use serde_json::{Value, json};

fn act(engine: &mut HeadlessEngine, action: Value) -> Vec<crate::runtime::EffectEnvelope> {
    let action: AppAction = serde_json::from_value(action).unwrap();
    engine.apply(action).effects
}

fn finish(engine: &mut HeadlessEngine, id: &str, value: Value) {
    let result: EffectResultInput =
        serde_json::from_value(json!({"effectId": id, "status": "ok", "value": value})).unwrap();
    engine.complete(result);
}

fn joined(engine: &mut HeadlessEngine, client: &str) {
    let effects = act(
        engine,
        json!({"type": "watchTogetherJoinRequested", "code": " abc234 ", "displayName": "Ana"}),
    );
    assert_eq!(effects[0].kind, EffectKind::WatchTogetherRoom);
    assert_eq!(effects[0].payload["op"], "join");
    assert_eq!(effects[0].payload["code"], "ABC234");
    finish(
        engine,
        &effects[0].id,
        json!({
            "room": "ABC234",
            "clientId": client,
            "hostId": "host",
            "members": [
                {"id": "host", "name": "Host", "avatar": "https://x.test/h.png"},
                {"id": client, "name": "Ana"}
            ]
        }),
    );
}

#[test]
fn joining_a_room_exposes_members_with_avatars() {
    let mut engine = HeadlessEngine::new(Default::default());
    joined(&mut engine, "ana");
    let state = &engine.state.watch_together;
    assert_eq!(state.room_code.as_deref(), Some("ABC234"));
    assert_eq!(state.members.len(), 2);
    assert_eq!(
        state.members[0].avatar.as_deref(),
        Some("https://x.test/h.png")
    );
    assert!(state.members[0].is_host);
}

#[test]
fn guest_is_sent_to_the_stream_picker_then_starts_on_the_hosts_signal() {
    let mut engine = HeadlessEngine::new(Default::default());
    joined(&mut engine, "ana");

    let effects = act(
        &mut engine,
        json!({"type": "watchTogetherMessageReceived", "nowMs": 0, "message": {
            "type": "content", "from": "host", "contentId": "tt1", "contentType": "movie", "title": "Film"
        }}),
    );
    assert_eq!(effects[0].kind, EffectKind::WatchTogetherCommand);
    assert_eq!(effects[0].payload["command"], "pickStream");

    let effects = act(
        &mut engine,
        json!({"type": "watchTogetherReadyChanged", "ready": true, "nowMs": 5}),
    );
    assert_eq!(effects[0].kind, EffectKind::WatchTogetherBroadcast);
    assert_eq!(effects[0].payload["message"]["type"], "ready");
    assert!(
        engine
            .state
            .watch_together
            .members
            .iter()
            .any(|m| m.id == "ana" && m.ready)
    );

    let effects = act(
        &mut engine,
        json!({"type": "watchTogetherMessageReceived", "nowMs": 10, "message": {
            "type": "start", "from": "host", "positionMs": 42000
        }}),
    );
    assert_eq!(effects[0].payload["command"], "start");
    assert_eq!(effects[0].payload["positionMs"], 42000);
}

#[test]
fn drift_sample_turns_into_a_seek_command() {
    let mut engine = HeadlessEngine::new(Default::default());
    joined(&mut engine, "ana");
    act(
        &mut engine,
        json!({"type": "watchTogetherMessageReceived", "nowMs": 0, "message": {
            "type": "content", "from": "host", "contentId": "tt1", "contentType": "movie"
        }}),
    );
    act(
        &mut engine,
        json!({"type": "watchTogetherReadyChanged", "ready": true, "nowMs": 0}),
    );
    act(
        &mut engine,
        json!({"type": "watchTogetherMessageReceived", "nowMs": 1000, "message": {
            "type": "start", "from": "host", "positionMs": 10000
        }}),
    );
    let effects = act(
        &mut engine,
        json!({"type": "watchTogetherPositionSampled", "positionMs": 10000, "nowMs": 3000, "speedCorrectionActive": false}),
    );
    assert_eq!(effects[0].payload["command"], "seek");
    assert_eq!(effects[0].payload["positionMs"], 12000);
}

#[test]
fn failed_join_surfaces_the_error_and_leaves_no_session() {
    let mut engine = HeadlessEngine::new(Default::default());
    let effects = act(
        &mut engine,
        json!({"type": "watchTogetherJoinRequested", "code": "ZZZZZZ", "displayName": "Ana"}),
    );
    let result: EffectResultInput = serde_json::from_value(json!({
        "effectId": effects[0].id,
        "status": "error",
        "error": {"message": "room not found"}
    }))
    .unwrap();
    engine.complete(result);
    assert!(engine.watch_session.is_none());
    assert_eq!(
        engine.state.watch_together.error_message.as_deref(),
        Some("room not found")
    );
}

#[test]
fn leaving_clears_the_room_and_asks_the_platform_to_unsubscribe() {
    let mut engine = HeadlessEngine::new(Default::default());
    joined(&mut engine, "ana");
    let effects = act(&mut engine, json!({"type": "watchTogetherLeaveRequested"}));
    assert_eq!(effects[0].payload["op"], "leave");
    assert_eq!(effects[0].payload["code"], "ABC234");
    assert!(engine.state.watch_together.room_code.is_none());
}
