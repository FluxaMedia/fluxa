use crate::headless_engine::helpers::normalize_error;
use crate::headless_engine::state::GenerationKey;
use crate::headless_engine::{EffectResultInput, HeadlessEngine};
use crate::player::watch_together::{
    SessionEffect, SharedStream, WatchTogetherConnectionState, WatchTogetherContent,
    WatchTogetherCorrection, WatchTogetherMember, WatchTogetherPlaybackSnapshot, WatchTogetherRole,
    WatchTogetherSession, WatchTogetherState,
};
use crate::runtime::{EffectEnvelope, EffectKind};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct RoomPayload {
    op: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    code: Option<String>,
    display_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    profile_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    avatar: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    member_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    locked: Option<bool>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RoomJoined {
    room: String,
    client_id: String,
    host_id: String,
    #[serde(default)]
    members: Vec<WatchTogetherMember>,
}

pub(crate) fn dispatch_create(
    engine: &mut HeadlessEngine,
    display_name: String,
    profile_id: Option<String>,
    avatar: Option<String>,
) -> Vec<EffectEnvelope> {
    request_room(engine, "create", None, display_name, profile_id, avatar)
}

pub(crate) fn dispatch_join(
    engine: &mut HeadlessEngine,
    code: String,
    display_name: String,
    profile_id: Option<String>,
    avatar: Option<String>,
) -> Vec<EffectEnvelope> {
    request_room(
        engine,
        "join",
        Some(code.trim().to_uppercase()),
        display_name,
        profile_id,
        avatar,
    )
}

pub(crate) fn dispatch_leave(engine: &mut HeadlessEngine) -> Vec<EffectEnvelope> {
    let code = engine.watch_session.take().map(|s| s.room().to_string());
    *engine.state.watch_together = WatchTogetherState::default();
    let generation = engine.bump_generation(GenerationKey::WatchTogether);
    let Some(code) = code else {
        return Vec::new();
    };
    vec![engine.effect(
        EffectKind::WatchTogetherRoom,
        generation,
        RoomPayload {
            op: "leave",
            code: Some(code),
            display_name: String::new(),
            profile_id: None,
            avatar: None,
            member_id: None,
            locked: None,
        },
    )]
}

pub(crate) fn dispatch_kick(engine: &mut HeadlessEngine, member_id: String) -> Vec<EffectEnvelope> {
    let Some(session) = engine.watch_session.as_ref() else {
        return Vec::new();
    };
    if !session.is_host() || member_id == session.client_id() {
        return Vec::new();
    }
    admin_effect(engine, "kick", Some(member_id), None)
}

pub(crate) fn dispatch_lock(engine: &mut HeadlessEngine, locked: bool) -> Vec<EffectEnvelope> {
    let Some(session) = engine.watch_session.as_ref() else {
        return Vec::new();
    };
    if !session.is_host() {
        return Vec::new();
    }
    engine.state.watch_together.locked = locked;
    admin_effect(engine, "lock", None, Some(locked))
}

fn admin_effect(
    engine: &mut HeadlessEngine,
    op: &'static str,
    member_id: Option<String>,
    locked: Option<bool>,
) -> Vec<EffectEnvelope> {
    let Some(code) = engine.watch_session.as_ref().map(|s| s.room().to_string()) else {
        return Vec::new();
    };
    let generation = engine.state.runtime.get(GenerationKey::WatchTogether);
    vec![engine.effect(
        EffectKind::WatchTogetherRoom,
        generation,
        RoomPayload {
            op,
            code: Some(code),
            display_name: String::new(),
            profile_id: None,
            avatar: None,
            member_id,
            locked,
        },
    )]
}

pub(crate) fn dispatch_content(
    engine: &mut HeadlessEngine,
    content: WatchTogetherContent,
    stream: Option<SharedStream>,
    start_position_ms: i64,
) -> Vec<EffectEnvelope> {
    run(engine, |session| {
        session.set_content(content, stream, start_position_ms)
    })
}

pub(crate) fn dispatch_ready(
    engine: &mut HeadlessEngine,
    ready: bool,
    content: Option<WatchTogetherContent>,
    now_ms: i64,
) -> Vec<EffectEnvelope> {
    run(engine, |session| {
        session.local_ready(ready, content.as_ref(), now_ms)
    })
}

pub(crate) fn dispatch_buffering(
    engine: &mut HeadlessEngine,
    buffering: bool,
    now_ms: i64,
) -> Vec<EffectEnvelope> {
    run(engine, |session| session.local_buffering(buffering, now_ms))
}

pub(crate) fn dispatch_playback(
    engine: &mut HeadlessEngine,
    position_ms: i64,
    duration_ms: i64,
    playing: bool,
    buffering: bool,
    now_ms: i64,
) -> Vec<EffectEnvelope> {
    let snapshot = WatchTogetherPlaybackSnapshot {
        position_ms,
        duration_ms,
        is_playing: playing,
        is_buffering: buffering,
    };
    run(engine, |session| session.host_playback(&snapshot, now_ms))
}

pub(crate) fn dispatch_message(
    engine: &mut HeadlessEngine,
    message: Value,
    now_ms: i64,
) -> Vec<EffectEnvelope> {
    run(engine, |session| session.handle(&message, now_ms))
}

pub(crate) fn dispatch_members(
    engine: &mut HeadlessEngine,
    host_id: String,
    members: Vec<WatchTogetherMember>,
    locked: bool,
    now_ms: i64,
) -> Vec<EffectEnvelope> {
    let effects = run(engine, |session| {
        session.members_changed(host_id, members, now_ms)
    });
    if engine.watch_session.is_some() {
        engine.state.watch_together.locked = locked;
    }
    effects
}

pub(crate) fn dispatch_sample(
    engine: &mut HeadlessEngine,
    position_ms: i64,
    now_ms: i64,
    speed_correction_active: bool,
) -> Vec<EffectEnvelope> {
    let Some(session) = engine.watch_session.as_ref() else {
        return Vec::new();
    };
    let command = match session.correction(position_ms, now_ms, speed_correction_active) {
        WatchTogetherCorrection::None => return Vec::new(),
        WatchTogetherCorrection::Seek(position_ms) => {
            json!({"command": "seek", "positionMs": position_ms})
        }
        WatchTogetherCorrection::Speed(value) => json!({"command": "speed", "value": value}),
        WatchTogetherCorrection::ResetSpeed => json!({"command": "resetSpeed"}),
    };
    let generation = engine.state.runtime.get(GenerationKey::WatchTogether);
    vec![engine.effect(EffectKind::WatchTogetherCommand, generation, command)]
}

pub(crate) fn complete(
    engine: &mut HeadlessEngine,
    generation: u64,
    result: &EffectResultInput,
    payload: &Value,
) -> Vec<EffectEnvelope> {
    if generation != engine.state.runtime.get(GenerationKey::WatchTogether) {
        return Vec::new();
    }
    if matches!(payload["op"].as_str(), Some("leave" | "kick" | "lock")) {
        return Vec::new();
    }
    if !result.status.is_ok() {
        engine.watch_session = None;
        let state = &mut *engine.state.watch_together;
        *state = WatchTogetherState::default();
        state.connection_state = WatchTogetherConnectionState::Error;
        state.error_message = Some(error_message(&normalize_error(result.error.clone())));
        return Vec::new();
    }
    let Ok(joined) = serde_json::from_value::<RoomJoined>(result.value.clone()) else {
        let state = &mut *engine.state.watch_together;
        state.connection_state = WatchTogetherConnectionState::Error;
        state.error_message = Some("generic".to_string());
        return Vec::new();
    };
    let session = WatchTogetherSession::new(
        joined.room,
        joined.client_id,
        joined.host_id,
        joined.members,
    );
    engine.watch_session = Some(session);
    sync_state(engine);
    engine.state.watch_together.connection_state = WatchTogetherConnectionState::Connected;
    engine.state.watch_together.error_message = None;
    Vec::new()
}

fn request_room(
    engine: &mut HeadlessEngine,
    op: &'static str,
    code: Option<String>,
    display_name: String,
    profile_id: Option<String>,
    avatar: Option<String>,
) -> Vec<EffectEnvelope> {
    let generation = engine.bump_generation(GenerationKey::WatchTogether);
    let previous = engine.watch_session.take().map(|s| s.room().to_string());
    let mut effects = Vec::new();
    if let Some(room) = previous {
        effects.push(engine.effect(
            EffectKind::WatchTogetherRoom,
            generation,
            RoomPayload {
                op: "leave",
                code: Some(room),
                display_name: String::new(),
                profile_id: None,
                avatar: None,
                member_id: None,
                locked: None,
            },
        ));
    }
    let state = &mut *engine.state.watch_together;
    *state = WatchTogetherState::default();
    state.connection_state = WatchTogetherConnectionState::Connecting;
    effects.push(engine.effect(
        EffectKind::WatchTogetherRoom,
        generation,
        RoomPayload {
            op,
            code,
            display_name,
            profile_id,
            avatar,
            member_id: None,
            locked: None,
        },
    ));
    effects
}

fn run(
    engine: &mut HeadlessEngine,
    action: impl FnOnce(&mut WatchTogetherSession) -> Vec<SessionEffect>,
) -> Vec<EffectEnvelope> {
    let Some(session) = engine.watch_session.as_mut() else {
        return Vec::new();
    };
    let room = session.room().to_string();
    let effects = action(session);
    sync_state(engine);
    let generation = engine.state.runtime.get(GenerationKey::WatchTogether);
    effects
        .into_iter()
        .map(|effect| {
            let (kind, payload) = wire(&room, effect);
            engine.effect(kind, generation, payload)
        })
        .collect()
}

fn wire(room: &str, effect: SessionEffect) -> (EffectKind, Value) {
    match effect {
        SessionEffect::Broadcast(message) => (
            EffectKind::WatchTogetherBroadcast,
            json!({"room": room, "message": message}),
        ),
        SessionEffect::PickStream(content) => (
            EffectKind::WatchTogetherCommand,
            json!({"command": "pickStream", "content": content}),
        ),
        SessionEffect::PlaySharedStream { content, stream } => (
            EffectKind::WatchTogetherCommand,
            json!({"command": "playShared", "content": content, "stream": stream}),
        ),
        SessionEffect::Start { position_ms } => (
            EffectKind::WatchTogetherCommand,
            json!({"command": "start", "positionMs": position_ms}),
        ),
        SessionEffect::Remote {
            playing,
            position_ms,
        } => (
            EffectKind::WatchTogetherCommand,
            json!({"command": "remote", "playing": playing, "positionMs": position_ms}),
        ),
    }
}

fn sync_state(engine: &mut HeadlessEngine) {
    let Some(session) = engine.watch_session.as_ref() else {
        return;
    };
    let state = &mut *engine.state.watch_together;
    state.room_code = Some(session.room().to_string());
    state.client_id = Some(session.client_id().to_string());
    state.host_id = Some(session.host_id().to_string());
    state.role = if session.is_host() {
        WatchTogetherRole::Host
    } else {
        WatchTogetherRole::Guest
    };
    state.members = session.roster();
    state.phase = session.phase();
    state.content = session.content().cloned();
    state.content_mismatch = session.content_mismatch();
}

fn error_message(error: &Value) -> String {
    error["message"]
        .as_str()
        .or_else(|| error["code"].as_str())
        .or_else(|| error.as_str())
        .unwrap_or("generic")
        .to_string()
}
