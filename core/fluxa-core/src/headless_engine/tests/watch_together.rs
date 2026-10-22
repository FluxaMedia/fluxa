use super::super::*;
use crate::runtime::EffectKind;
use serde_json::{Value, json};

fn act(engine: &mut HeadlessEngine, action: Value) -> Vec<crate::runtime::EffectEnvelope> {
    let action: AppAction = serde_json::from_value(action).unwrap();
    engine.apply(action).effects
}

fn media() -> Value {
    json!({"type": "movie", "meta_id": "tt1", "meta_type": "movie", "title": "Film"})
}

fn snapshot(own: &str, host: &str) -> Value {
    json!({
        "party": {
            "id": "p1", "code": "ABC234", "host_member_id": host, "host_only": false,
            "media": media(), "max_members": 3
        },
        "members": [
            {"id": "host", "display_name": "Host", "avatar_url": "https://x.test/h.png"},
            {"id": "ana", "display_name": "Ana"}
        ],
        "member_id": own,
        "server_time_ms": 1_000_000
    })
}

fn finish(engine: &mut HeadlessEngine, id: &str, value: Value) {
    let result: EffectResultInput =
        serde_json::from_value(json!({"effectId": id, "status": "ok", "value": value})).unwrap();
    engine.complete(result);
}

fn joined(engine: &mut HeadlessEngine, own: &str) {
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
        json!({"snapshot": snapshot(own, "host"), "sentAt": 1000, "receivedAt": 1040}),
    );
}

fn state_message(seq: i64, by: &str, paused: bool, position_ms: i64) -> Value {
    json!({
        "type": "watchTogetherMessageReceived", "event": "state", "nowMs": 2000,
        "payload": {
            "seq": seq, "member_id": by, "paused": paused, "position_ms": position_ms,
            "rate": 1.0, "at": 1_000_960, "media_key": "movie|tt1||", "waiting": false
        }
    })
}

#[test]
fn joining_exposes_members_with_avatars_and_the_current_content() {
    let mut engine = HeadlessEngine::new(Default::default());
    joined(&mut engine, "ana");
    let state = &engine.state.watch_together;
    assert_eq!(state.room_code.as_deref(), Some("ABC234"));
    assert_eq!(state.members.len(), 2);
    assert_eq!(state.members[0].avatar.as_deref(), Some("https://x.test/h.png"));
    assert!(state.members[0].is_host);
    assert_eq!(state.content.as_ref().map(|c| c.id.as_str()), Some("tt1"));
}

#[test]
fn creating_a_room_sends_the_playing_content() {
    let mut engine = HeadlessEngine::new(Default::default());
    let effects = act(
        &mut engine,
        json!({
            "type": "watchTogetherCreateRequested", "displayName": "Host",
            "content": {"id": "tt9", "contentType": "series", "videoId": "tt9:2:3", "title": "Show",
                "artwork": {"season": 2, "episode": 3, "episodeTitle": "Pilot"}}
        }),
    );
    let media = &effects[0].payload["media"];
    assert_eq!(effects[0].payload["op"], "create");
    assert_eq!(media["meta_id"], "tt9");
    assert_eq!(media["season"], 2);
    assert_eq!(media["episode_title"], "Pilot");
}

#[test]
fn newer_remote_state_becomes_a_remote_command_at_the_expected_position() {
    let mut engine = HeadlessEngine::new(Default::default());
    joined(&mut engine, "ana");
    let effects = act(&mut engine, state_message(1, "host", false, 10_000));
    let command = effects
        .iter()
        .find(|e| e.kind == EffectKind::WatchTogetherCommand)
        .expect("remote command");
    assert_eq!(command.payload["command"], "remote");
    assert_eq!(command.payload["playing"], true);
    assert!(command.payload["positionMs"].as_i64().unwrap() >= 10_000);

    let stale = act(&mut engine, state_message(1, "host", true, 0));
    assert!(stale.is_empty());
}

#[test]
fn any_member_can_publish_until_host_only_is_on() {
    let mut engine = HeadlessEngine::new(Default::default());
    joined(&mut engine, "ana");
    act(&mut engine, state_message(1, "host", false, 0));
    let effects = act(
        &mut engine,
        json!({"type": "watchTogetherControlRequested", "kind": "paused", "nowMs": 3000}),
    );
    assert_eq!(effects[0].kind, EffectKind::WatchTogetherBroadcast);
    assert_eq!(effects[0].payload["event"], "state");
    assert_eq!(effects[0].payload["payload"]["seq"], 2);
    assert_eq!(effects[0].payload["payload"]["paused"], true);

    act(
        &mut engine,
        json!({"type": "watchTogetherMessageReceived", "event": "control", "nowMs": 3100,
            "payload": {"host_only": true}}),
    );
    assert!(engine.state.watch_together.locked);
    let blocked = act(
        &mut engine,
        json!({"type": "watchTogetherControlRequested", "kind": "resumed", "nowMs": 3200}),
    );
    assert!(blocked.is_empty());
}

#[test]
fn host_changing_media_sends_guests_to_the_stream_picker() {
    let mut engine = HeadlessEngine::new(Default::default());
    joined(&mut engine, "ana");
    let effects = act(
        &mut engine,
        json!({"type": "watchTogetherMessageReceived", "event": "media", "nowMs": 3000,
            "payload": {"media": {"type": "movie", "meta_id": "tt2", "meta_type": "movie", "title": "Other"}}}),
    );
    let pick = effects
        .iter()
        .find(|e| e.payload["command"] == "pickStream")
        .expect("pick command");
    assert_eq!(pick.payload["content"]["id"], "tt2");
}

#[test]
fn drift_sample_turns_into_a_seek_command() {
    let mut engine = HeadlessEngine::new(Default::default());
    joined(&mut engine, "ana");
    act(&mut engine, state_message(1, "host", true, 10_000));
    let effects = act(
        &mut engine,
        json!({"type": "watchTogetherPositionSampled", "positionMs": 2000, "nowMs": 3000, "speedCorrectionActive": false}),
    );
    assert_eq!(effects[0].payload["command"], "seek");
    assert_eq!(effects[0].payload["positionMs"], 10_000);
}

#[test]
fn removal_ends_the_session_with_a_reason() {
    let mut engine = HeadlessEngine::new(Default::default());
    joined(&mut engine, "ana");
    act(
        &mut engine,
        json!({"type": "watchTogetherMessageReceived", "event": "member_left", "nowMs": 4000,
            "payload": {"member_id": "ana", "reason": "removed"}}),
    );
    assert!(engine.watch_session.is_none());
    assert_eq!(
        engine.state.watch_together.error_message.as_deref(),
        Some("removed")
    );
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
        "error": {"message": "party_not_found"}
    }))
    .unwrap();
    engine.complete(result);
    assert!(engine.watch_session.is_none());
    assert_eq!(
        engine.state.watch_together.error_message.as_deref(),
        Some("party_not_found")
    );
}
