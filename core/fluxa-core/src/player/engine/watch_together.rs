use crate::headless_engine::helpers::normalize_error;
use crate::headless_engine::state::GenerationKey;
use crate::headless_engine::{EffectResultInput, HeadlessEngine};
use crate::player::party::{
    ActiveParty, PartyAction, PartyCorrection, PartyEffect, PartyEvent, PartyMedia, PartyMember,
    PartyMemberStatus, PartyNotice, PartyPlayback, PartyPresence, PartySession, PartySnapshot,
    Time, active_party, expected_position_ms,
};
use crate::player::watch_together::{
    SharedStream, WatchTogetherArtwork, WatchTogetherChatMessage, WatchTogetherConnectionState,
    WatchTogetherContent, WatchTogetherEvent, WatchTogetherEventKind, WatchTogetherMember,
    WatchTogetherMemberStatus, WatchTogetherRole, WatchTogetherState, SessionPhase,
};
use crate::runtime::{EffectEnvelope, EffectKind};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

const CHAT_HISTORY: usize = 50;
const CHAT_LENGTH: usize = 300;

#[derive(Serialize, Default)]
#[serde(rename_all = "camelCase")]
struct RoomPayload {
    op: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    party_id: Option<String>,
    #[serde(skip_serializing_if = "String::is_empty")]
    display_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    profile_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    avatar: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    profile_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    member_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    host_only: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    media: Option<PartyMedia>,
    #[serde(skip_serializing_if = "Option::is_none")]
    playback: Option<PartyPlayback>,
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    announce: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RoomJoined {
    snapshot: PartySnapshot,
    #[serde(default)]
    sent_at: i64,
    #[serde(default)]
    received_at: i64,
}

fn t(now_ms: i64) -> Time {
    Time {
        local_ms: now_ms,
        wall_ms: now_ms,
    }
}

pub(crate) fn media_of(content: &WatchTogetherContent, source: Option<&SharedStream>) -> PartyMedia {
    let artwork = content.artwork.as_ref();
    let _ = source;
    PartyMedia {
        kind: content.content_type.clone(),
        meta_id: content.id.clone(),
        meta_type: content.content_type.clone(),
        video_id: content.video_id.clone(),
        season: artwork.and_then(|a| a.season).map(|v| v as i32),
        episode: artwork.and_then(|a| a.episode).map(|v| v as i32),
        title: content.title.clone(),
        episode_title: artwork.and_then(|a| a.episode_title.clone()),
        description: None,
        poster: None,
        background: artwork.and_then(|a| a.backdrop.clone()),
        logo: artwork.and_then(|a| a.logo.clone()),
        source: None,
    }
}

pub(crate) fn content_of(media: &PartyMedia) -> WatchTogetherContent {
    let artwork = WatchTogetherArtwork {
        backdrop: media.background.clone(),
        logo: media.logo.clone(),
        episode_title: media.episode_title.clone(),
        season: media.season.map(|v| v as u32),
        episode: media.episode.map(|v| v as u32),
    };
    WatchTogetherContent {
        id: media.meta_id.clone(),
        content_type: media.meta_type.clone(),
        video_id: media.video_id.clone(),
        title: media.title.clone(),
        artwork: (artwork != WatchTogetherArtwork::default()).then_some(artwork),
    }
}

fn room_effect(engine: &mut HeadlessEngine, payload: RoomPayload) -> EffectEnvelope {
    let generation = engine.state.runtime.get(GenerationKey::WatchTogether);
    engine.effect(EffectKind::WatchTogetherRoom, generation, payload)
}

fn party_id(engine: &HeadlessEngine) -> Option<String> {
    engine.watch_session.as_ref().map(|s| s.party().id.clone())
}

pub(crate) fn dispatch_create(
    engine: &mut HeadlessEngine,
    display_name: String,
    profile_id: Option<String>,
    avatar: Option<String>,
    profile_key: Option<String>,
    content: WatchTogetherContent,
) -> Vec<EffectEnvelope> {
    let media = media_of(&content, None);
    request_room(
        engine,
        RoomPayload {
            op: "create",
            display_name,
            profile_id,
            avatar,
            profile_key,
            media: Some(media),
            ..Default::default()
        },
    )
}

pub(crate) fn dispatch_join(
    engine: &mut HeadlessEngine,
    code: String,
    display_name: String,
    profile_id: Option<String>,
    avatar: Option<String>,
    profile_key: Option<String>,
) -> Vec<EffectEnvelope> {
    request_room(
        engine,
        RoomPayload {
            op: "join",
            code: Some(crate::player::party::normalize_party_code(&code)),
            display_name,
            profile_id,
            avatar,
            profile_key,
            ..Default::default()
        },
    )
}

fn request_room(engine: &mut HeadlessEngine, payload: RoomPayload) -> Vec<EffectEnvelope> {
    let generation = engine.bump_generation(GenerationKey::WatchTogether);
    let previous = engine.watch_session.take().map(|s| s.party().id.clone());
    let mut effects = Vec::new();
    if let Some(id) = previous {
        effects.push(engine.effect(
            EffectKind::WatchTogetherRoom,
            generation,
            RoomPayload {
                op: "leave",
                party_id: Some(id),
                ..Default::default()
            },
        ));
    }
    let state = &mut *engine.state.watch_together;
    *state = WatchTogetherState::default();
    state.connection_state = WatchTogetherConnectionState::Connecting;
    effects.push(engine.effect(EffectKind::WatchTogetherRoom, generation, payload));
    effects
}

pub(crate) fn dispatch_leave(engine: &mut HeadlessEngine) -> Vec<EffectEnvelope> {
    let id = engine.watch_session.take().map(|s| s.party().id.clone());
    *engine.state.watch_together = WatchTogetherState::default();
    let generation = engine.bump_generation(GenerationKey::WatchTogether);
    let Some(id) = id else {
        return Vec::new();
    };
    vec![engine.effect(
        EffectKind::WatchTogetherRoom,
        generation,
        RoomPayload {
            op: "leave",
            party_id: Some(id),
            ..Default::default()
        },
    )]
}

pub(crate) fn dispatch_end(engine: &mut HeadlessEngine) -> Vec<EffectEnvelope> {
    let Some(session) = engine.watch_session.as_ref() else {
        return Vec::new();
    };
    if !session.party().is_host() {
        return dispatch_leave(engine);
    }
    let id = session.party().id.clone();
    let generation = engine.state.runtime.get(GenerationKey::WatchTogether);
    let end = engine.effect(
        EffectKind::WatchTogetherRoom,
        generation,
        RoomPayload {
            op: "end",
            party_id: Some(id),
            ..Default::default()
        },
    );
    engine.watch_session = None;
    *engine.state.watch_together = WatchTogetherState::default();
    engine.bump_generation(GenerationKey::WatchTogether);
    vec![end]
}

pub(crate) fn dispatch_kick(engine: &mut HeadlessEngine, member_id: String) -> Vec<EffectEnvelope> {
    let Some(session) = engine.watch_session.as_ref() else {
        return Vec::new();
    };
    if !session.party().is_host() || member_id == session.party().member_id {
        return Vec::new();
    }
    let id = session.party().id.clone();
    vec![room_effect(
        engine,
        RoomPayload {
            op: "kick",
            party_id: Some(id),
            member_id: Some(member_id),
            ..Default::default()
        },
    )]
}

pub(crate) fn dispatch_lock(engine: &mut HeadlessEngine, locked: bool) -> Vec<EffectEnvelope> {
    let Some(session) = engine.watch_session.as_mut() else {
        return Vec::new();
    };
    let Some(host_only) = session.set_host_only(locked) else {
        return Vec::new();
    };
    let id = session.party().id.clone();
    sync_state(engine);
    vec![room_effect(
        engine,
        RoomPayload {
            op: "hostOnly",
            party_id: Some(id),
            host_only: Some(host_only),
            ..Default::default()
        },
    )]
}

pub(crate) fn dispatch_content(
    engine: &mut HeadlessEngine,
    content: WatchTogetherContent,
    stream: Option<SharedStream>,
    start_position_ms: i64,
    now_ms: i64,
) -> Vec<EffectEnvelope> {
    let media = media_of(&content, stream.as_ref());
    let mut effects = run(engine, now_ms, |session| {
        session.publish_media(media, start_position_ms, t(now_ms))
    });
    if let (Some(stream), Some(session)) = (stream, engine.watch_session.as_ref()) {
        let payload = json!({
            "member_id": session.party().member_id,
            "stream": {
                "media_key": session.party().media.key(),
                "url": stream.url,
                "headers": stream.headers,
                "name": stream.title,
            },
        });
        let room = session.party().id.clone();
        effects.push(broadcast(engine, &room, "stream", payload));
    }
    effects
}

pub(crate) fn dispatch_status(
    engine: &mut HeadlessEngine,
    status: PartyMemberStatus,
    media_key: Option<String>,
    behind_ms: Option<i64>,
    now_ms: i64,
) -> Vec<EffectEnvelope> {
    run(engine, now_ms, |session| {
        session.set_local_status(status, media_key, behind_ms, t(now_ms));
        Vec::new()
    })
}

pub(crate) fn dispatch_control(
    engine: &mut HeadlessEngine,
    kind: WatchTogetherEventKind,
    now_ms: i64,
) -> Vec<EffectEnvelope> {
    run(engine, now_ms, |session| {
        let Some(current) = session.playback().cloned() else {
            return Vec::new();
        };
        let now = session.now_ms(t(now_ms));
        let position = expected_position_ms(&current, now, 0);
        let (paused, position, rate) = match kind {
            WatchTogetherEventKind::Paused => (true, position, current.rate),
            WatchTogetherEventKind::Resumed => (false, position, current.rate),
            WatchTogetherEventKind::Seeked { position_ms } => {
                (current.paused, position_ms.max(0), current.rate)
            }
            WatchTogetherEventKind::Speed { rate } => (current.paused, position, rate as f32),
        };
        session.publish(paused, position, rate, false, now, t(now_ms))
    })
}

pub(crate) fn dispatch_chat(
    engine: &mut HeadlessEngine,
    text: String,
    now_ms: i64,
) -> Vec<EffectEnvelope> {
    let _ = now_ms;
    let text: String = text
        .trim()
        .chars()
        .filter(|c| !c.is_control())
        .take(CHAT_LENGTH)
        .collect();
    let Some(id) = party_id(engine).filter(|_| !text.is_empty()) else {
        return Vec::new();
    };
    push_own_chat(engine, text.clone());
    vec![room_effect(
        engine,
        RoomPayload {
            op: "chat",
            party_id: Some(id),
            text: Some(text),
            ..Default::default()
        },
    )]
}

pub(crate) fn dispatch_message(
    engine: &mut HeadlessEngine,
    event: String,
    payload: Value,
    now_ms: i64,
) -> Vec<EffectEnvelope> {
    run(engine, now_ms, |session| {
        session.on_message(&event, &payload, t(now_ms))
    })
}

pub(crate) fn dispatch_presence(
    engine: &mut HeadlessEngine,
    joins: Vec<(String, Value)>,
    leaves: Vec<String>,
    now_ms: i64,
) -> Vec<EffectEnvelope> {
    run(engine, now_ms, |session| session.on_presence(&joins, &leaves))
}

pub(crate) fn dispatch_connection(
    engine: &mut HeadlessEngine,
    connected: bool,
    now_ms: i64,
) -> Vec<EffectEnvelope> {
    run(engine, now_ms, |session| session.on_connection(connected))
}

pub(crate) fn dispatch_snapshot(
    engine: &mut HeadlessEngine,
    snapshot: PartySnapshot,
    announce: Option<String>,
    now_ms: i64,
) -> Vec<EffectEnvelope> {
    run(engine, now_ms, |session| {
        session.apply_snapshot(&snapshot, announce.as_deref(), t(now_ms))
    })
}

pub(crate) fn dispatch_rejoined(
    engine: &mut HeadlessEngine,
    snapshot: Option<PartySnapshot>,
    error: Option<String>,
    now_ms: i64,
) -> Vec<EffectEnvelope> {
    let Some(snapshot) = snapshot.filter(|s| s.error.is_none()) else {
        let error = crate::player::party::PartyError::from_code(error.as_deref());
        return run(engine, now_ms, |session| session.rejoin_failed(error));
    };
    let Some(session) = engine.watch_session.as_mut() else {
        return Vec::new();
    };
    let Some(replacement) = session.rejoined(&snapshot) else {
        sync_state(engine);
        return Vec::new();
    };
    install(engine, replacement, snapshot, 0, now_ms)
}

pub(crate) fn dispatch_clock(
    engine: &mut HeadlessEngine,
    server_ms: i64,
    sent_at: i64,
    received_at: i64,
) -> Vec<EffectEnvelope> {
    if let Some(session) = engine.watch_session.as_mut() {
        session.clock_mut().add_sample(server_ms, sent_at, received_at);
    }
    Vec::new()
}

pub(crate) fn dispatch_tick(engine: &mut HeadlessEngine, now_ms: i64) -> Vec<EffectEnvelope> {
    run(engine, now_ms, |session| session.tick(t(now_ms)))
}

pub(crate) fn dispatch_sample(
    engine: &mut HeadlessEngine,
    position_ms: i64,
    duration_ms: i64,
    now_ms: i64,
    nudging: bool,
) -> Vec<EffectEnvelope> {
    let Some(session) = engine.watch_session.as_ref() else {
        return Vec::new();
    };
    let Some(correction) = session.correction_for(t(now_ms), position_ms, duration_ms, nudging)
    else {
        return Vec::new();
    };
    let command = match correction {
        PartyCorrection::InSync if nudging => json!({"command": "resetSpeed"}),
        PartyCorrection::InSync => return Vec::new(),
        PartyCorrection::Seek(position_ms) => json!({"command": "seek", "positionMs": position_ms}),
        PartyCorrection::Nudge(rate) => {
            let base = session.playback().map_or(1.0, |p| p.rate);
            json!({"command": "speed", "value": rate / base})
        }
    };
    let generation = engine.state.runtime.get(GenerationKey::WatchTogether);
    vec![engine.effect(EffectKind::WatchTogetherCommand, generation, command)]
}

fn install(
    engine: &mut HeadlessEngine,
    party: ActiveParty,
    snapshot: PartySnapshot,
    sent_at: i64,
    received_at: i64,
) -> Vec<EffectEnvelope> {
    let playback = snapshot.party.as_ref().and_then(|p| p.playback.clone());
    let timing = snapshot
        .server_time_ms
        .map(|server_ms| (server_ms, sent_at, received_at));
    let mut session = PartySession::new(
        party,
        playback,
        crate::player::party::PartyClock::default(),
        timing,
        received_at,
    );
    let _ = &mut session;
    engine.watch_session = Some(session);
    let state = &mut *engine.state.watch_together;
    state.connection_state = WatchTogetherConnectionState::Connected;
    state.error_message = None;
    sync_state(engine);
    remote_effect(engine, received_at).into_iter().collect()
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
    if !matches!(payload["op"].as_str(), Some("create" | "join")) {
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
    let Some(party) = active_party(&joined.snapshot) else {
        let state = &mut *engine.state.watch_together;
        state.connection_state = WatchTogetherConnectionState::Error;
        state.error_message = Some("generic".to_string());
        return Vec::new();
    };
    install(engine, party, joined.snapshot, joined.sent_at, joined.received_at)
}

fn broadcast(engine: &mut HeadlessEngine, room: &str, event: &str, payload: Value) -> EffectEnvelope {
    let generation = engine.state.runtime.get(GenerationKey::WatchTogether);
    engine.effect(
        EffectKind::WatchTogetherBroadcast,
        generation,
        json!({"room": room, "event": event, "payload": payload}),
    )
}

fn run(
    engine: &mut HeadlessEngine,
    now_ms: i64,
    action: impl FnOnce(&mut PartySession) -> Vec<PartyEffect>,
) -> Vec<EffectEnvelope> {
    let Some(session) = engine.watch_session.as_mut() else {
        return Vec::new();
    };
    let before = session.playback().cloned();
    let room = session.party().id.clone();
    let effects = action(session);
    let mut envelopes = Vec::new();
    let mut finish = None;
    for effect in effects {
        match effect {
            PartyEffect::Send { event, payload } => {
                envelopes.push(broadcast(engine, &room, event, payload));
            }
            PartyEffect::Track(payload) => {
                let generation = engine.state.runtime.get(GenerationKey::WatchTogether);
                envelopes.push(engine.effect(
                    EffectKind::WatchTogetherBroadcast,
                    generation,
                    json!({"room": room, "track": payload}),
                ));
            }
            PartyEffect::Persist { media, playback } => {
                envelopes.push(room_effect(
                    engine,
                    RoomPayload {
                        op: "update",
                        party_id: Some(room.clone()),
                        media,
                        playback,
                        ..Default::default()
                    },
                ));
            }
            PartyEffect::Heartbeat { announce } => {
                envelopes.push(room_effect(
                    engine,
                    RoomPayload {
                        op: "heartbeat",
                        party_id: Some(room.clone()),
                        announce,
                        ..Default::default()
                    },
                ));
            }
            PartyEffect::Rejoin => {
                let code = engine.watch_session.as_ref().map(|s| s.party().code.clone());
                envelopes.push(room_effect(
                    engine,
                    RoomPayload {
                        op: "rejoin",
                        party_id: Some(room.clone()),
                        code,
                        ..Default::default()
                    },
                ));
            }
            PartyEffect::Chat(payload) => push_chat(engine, &payload),
            PartyEffect::Stream(stream) => {
                let content = engine
                    .watch_session
                    .as_ref()
                    .map(|s| content_of(&s.party().media));
                if let (Some(content), false) = (content, stream.is_null()) {
                    let generation = engine.state.runtime.get(GenerationKey::WatchTogether);
                    envelopes.push(engine.effect(
                        EffectKind::WatchTogetherCommand,
                        generation,
                        json!({"command": "playShared", "content": content, "stream": stream}),
                    ));
                }
            }
            PartyEffect::Notice(notice) => record_notice(engine, notice),
            PartyEffect::Event(PartyEvent::MediaChanged(media)) => {
                let generation = engine.state.runtime.get(GenerationKey::WatchTogether);
                envelopes.push(engine.effect(
                    EffectKind::WatchTogetherCommand,
                    generation,
                    json!({"command": "pickStream", "content": content_of(&media)}),
                ));
            }
            PartyEffect::Event(_) => {}
            PartyEffect::Finish(event) => finish = Some(event),
        }
    }
    sync_state(engine);
    if let Some(event) = finish {
        let reason = match event {
            PartyEvent::Removed => "removed",
            _ => "ended",
        };
        envelopes.extend(dispatch_leave(engine));
        let state = &mut *engine.state.watch_together;
        state.connection_state = WatchTogetherConnectionState::Error;
        state.error_message = Some(reason.to_owned());
        return envelopes;
    }
    let changed = engine.watch_session.as_ref().and_then(|s| s.playback()) != before.as_ref();
    if changed {
        envelopes.extend(remote_effect(engine, now_ms));
    }
    envelopes
}

fn remote_effect(engine: &mut HeadlessEngine, now_ms: i64) -> Option<EffectEnvelope> {
    let session = engine.watch_session.as_ref()?;
    let playback = session.playback()?;
    if playback.member_id.as_deref() == Some(session.party().member_id.as_str()) {
        return None;
    }
    let position = expected_position_ms(playback, session.now_ms(t(now_ms)), 0);
    let command = json!({
        "command": "remote",
        "playing": !playback.paused,
        "positionMs": position,
        "rate": playback.rate,
    });
    let generation = engine.state.runtime.get(GenerationKey::WatchTogether);
    Some(engine.effect(EffectKind::WatchTogetherCommand, generation, command))
}

fn record_notice(engine: &mut HeadlessEngine, notice: PartyNotice) {
    let kind = match notice.action {
        PartyAction::Paused => WatchTogetherEventKind::Paused,
        PartyAction::Resumed => WatchTogetherEventKind::Resumed,
        PartyAction::Seeked => WatchTogetherEventKind::Seeked {
            position_ms: notice.position_ms,
        },
        PartyAction::Speed => WatchTogetherEventKind::Speed {
            rate: notice.rate as f64,
        },
    };
    let state = &mut *engine.state.watch_together;
    let id = state.last_event.as_ref().map_or(0, |e| e.id) + 1;
    state.last_event = Some(WatchTogetherEvent {
        id,
        by: String::new(),
        name: notice.name,
        kind,
    });
}

fn push_chat(engine: &mut HeadlessEngine, payload: &Value) {
    let Some(session) = engine.watch_session.as_ref() else {
        return;
    };
    let by = payload["member_id"].as_str().unwrap_or_default().to_owned();
    if by == session.party().member_id {
        return;
    }
    let text = payload["body"].as_str().unwrap_or_default().to_owned();
    if text.is_empty() {
        return;
    }
    let name = payload["name"]
        .as_str()
        .map(str::to_owned)
        .or_else(|| session.party().member(&by).map(|m| m.display_name.clone()))
        .unwrap_or_default();
    let state = &mut *engine.state.watch_together;
    let id = state.chat.last().map_or(0, |m| m.id) + 1;
    state.chat.push(WatchTogetherChatMessage { id, by, name, text });
    let overflow = state.chat.len().saturating_sub(CHAT_HISTORY);
    state.chat.drain(..overflow);
}

pub(crate) fn push_own_chat(engine: &mut HeadlessEngine, text: String) {
    let Some(me) = engine.watch_session.as_ref().map(|s| s.party().member_id.clone()) else {
        return;
    };
    let state = &mut *engine.state.watch_together;
    let id = state.chat.last().map_or(0, |m| m.id) + 1;
    state.chat.push(WatchTogetherChatMessage {
        id,
        by: me,
        name: String::new(),
        text,
    });
    let overflow = state.chat.len().saturating_sub(CHAT_HISTORY);
    state.chat.drain(..overflow);
}

fn member_status(party: &ActiveParty, member: &PartyMember, paused: bool) -> (WatchTogetherMemberStatus, bool, bool) {
    let presence = party
        .presence
        .get(&member.id)
        .cloned()
        .or_else(|| (member.id == party.member_id).then(|| PartyPresence::browsing(&member.id)));
    match presence.map(|p| p.status) {
        Some(PartyMemberStatus::Loading) => (WatchTogetherMemberStatus::Picking, true, false),
        Some(PartyMemberStatus::Buffering) => (WatchTogetherMemberStatus::Buffering, true, false),
        Some(PartyMemberStatus::Ready) => (
            if paused {
                WatchTogetherMemberStatus::Paused
            } else {
                WatchTogetherMemberStatus::Watching
            },
            false,
            true,
        ),
        Some(PartyMemberStatus::Behind) => (WatchTogetherMemberStatus::Paused, false, true),
        _ => (WatchTogetherMemberStatus::Waiting, false, false),
    }
}

fn sync_state(engine: &mut HeadlessEngine) {
    let Some(session) = engine.watch_session.as_ref() else {
        return;
    };
    let party = session.party();
    let paused = session.playback().is_none_or(|p| p.paused);
    let members: Vec<WatchTogetherMember> = party
        .members
        .iter()
        .map(|member| {
            let (status, buffering, ready) = member_status(party, member, paused);
            WatchTogetherMember {
                id: member.id.clone(),
                name: member.display_name.clone(),
                is_host: party.host_member_id.as_deref() == Some(member.id.as_str()),
                buffering,
                ready,
                avatar: member.avatar_url.clone(),
                color: member.avatar_color.clone(),
                status,
            }
        })
        .collect();
    let state = &mut *engine.state.watch_together;
    state.room_code = Some(party.code.clone());
    state.client_id = Some(party.member_id.clone());
    state.host_id = party.host_member_id.clone();
    state.role = if party.is_host() {
        WatchTogetherRole::Host
    } else {
        WatchTogetherRole::Guest
    };
    state.members = members;
    state.phase = SessionPhase::Playing;
    state.locked = party.host_only;
    state.max_members = party.max_members;
    state.content = Some(content_of(&party.media));
    state.content_mismatch = false;
}

fn error_message(error: &Value) -> String {
    error["message"]
        .as_str()
        .or_else(|| error["code"].as_str())
        .or_else(|| error.as_str())
        .unwrap_or("generic")
        .to_string()
}
