use super::{
    WatchTogetherContent, WatchTogetherCorrection, WatchTogetherDriftPolicy, WatchTogetherMember,
    WatchTogetherPlaybackSnapshot, WatchTogetherProtocol,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SharedStream {
    pub url: String,
    #[serde(default)]
    pub title: String,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub headers: BTreeMap<String, String>,
}

impl SharedStream {
    fn is_playable(&self) -> bool {
        let web = self.url.starts_with("https://") || self.url.starts_with("http://");
        web && self.url.len() <= 4096
            && self.title.len() <= 300
            && self.headers.len() <= 16
            && self
                .headers
                .iter()
                .all(|(name, value)| name.len() <= 64 && value.len() <= 2048)
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum SessionPhase {
    #[default]
    Lobby,
    Selecting,
    Playing,
}

#[derive(Clone, Debug, PartialEq)]
pub enum SessionEffect {
    Broadcast(Value),
    PickStream(WatchTogetherContent),
    PlaySharedStream {
        content: WatchTogetherContent,
        stream: SharedStream,
    },
    Start {
        position_ms: i64,
    },
    Remote {
        playing: bool,
        position_ms: i64,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct HostPlayback {
    position_ms: i64,
    playing: bool,
    at_ms: i64,
}

impl HostPlayback {
    fn position_at(&self, now_ms: i64) -> i64 {
        if self.playing {
            self.position_ms + (now_ms - self.at_ms).max(0)
        } else {
            self.position_ms
        }
    }
}

#[derive(Clone, Debug)]
pub struct WatchTogetherSession {
    room: String,
    client_id: String,
    host_id: String,
    phase: SessionPhase,
    members: Vec<WatchTogetherMember>,
    content: Option<WatchTogetherContent>,
    shared_stream: Option<SharedStream>,
    start_position_ms: i64,
    ready: BTreeSet<String>,
    buffering: BTreeSet<String>,
    playback: Option<HostPlayback>,
    held: bool,
    resume_after_hold: bool,
    sequence: i64,
}

impl WatchTogetherSession {
    pub fn new(
        room: String,
        client_id: String,
        host_id: String,
        members: Vec<WatchTogetherMember>,
    ) -> Self {
        Self {
            room,
            client_id,
            host_id,
            phase: SessionPhase::Lobby,
            members,
            content: None,
            shared_stream: None,
            start_position_ms: 0,
            ready: BTreeSet::new(),
            buffering: BTreeSet::new(),
            playback: None,
            held: false,
            resume_after_hold: false,
            sequence: 0,
        }
    }

    pub fn room(&self) -> &str {
        &self.room
    }

    pub fn phase(&self) -> SessionPhase {
        self.phase
    }

    pub fn content(&self) -> Option<&WatchTogetherContent> {
        self.content.as_ref()
    }

    pub fn host_id(&self) -> &str {
        &self.host_id
    }

    pub fn client_id(&self) -> &str {
        &self.client_id
    }

    pub fn is_host(&self) -> bool {
        self.client_id == self.host_id
    }

    pub fn roster(&self) -> Vec<WatchTogetherMember> {
        self.members
            .iter()
            .map(|member| WatchTogetherMember {
                is_host: member.id == self.host_id,
                ready: self.ready.contains(&member.id),
                buffering: self.buffering.contains(&member.id),
                ..member.clone()
            })
            .collect()
    }

    pub fn ready_count(&self) -> usize {
        self.members
            .iter()
            .filter(|member| self.ready.contains(&member.id))
            .count()
    }

    pub fn member_count(&self) -> usize {
        self.members.len()
    }

    pub fn is_ready(&self, id: &str) -> bool {
        self.ready.contains(id)
    }

    pub fn set_content(
        &mut self,
        content: WatchTogetherContent,
        shared_stream: Option<SharedStream>,
        start_position_ms: i64,
    ) -> Vec<SessionEffect> {
        if !self.is_host() {
            return Vec::new();
        }
        self.phase = SessionPhase::Selecting;
        self.ready.clear();
        self.buffering.clear();
        self.held = false;
        self.playback = None;
        self.start_position_ms = start_position_ms.max(0);
        let message = self.content_message(&content, shared_stream.as_ref());
        self.content = Some(content);
        self.shared_stream = shared_stream;
        vec![SessionEffect::Broadcast(message)]
    }

    pub fn local_ready(&mut self, ready: bool, now_ms: i64) -> Vec<SessionEffect> {
        if self.phase != SessionPhase::Selecting {
            return Vec::new();
        }
        let changed = if ready {
            self.ready.insert(self.client_id.clone())
        } else {
            self.ready.remove(&self.client_id)
        };
        if !changed {
            return Vec::new();
        }
        let mut effects = vec![SessionEffect::Broadcast(self.tagged(json!({
            "type": "ready",
            "ready": ready,
        })))];
        effects.extend(self.try_start(now_ms));
        effects
    }

    pub fn local_buffering(&mut self, buffering: bool, now_ms: i64) -> Vec<SessionEffect> {
        if self.phase != SessionPhase::Playing {
            return Vec::new();
        }
        let changed = if buffering {
            self.buffering.insert(self.client_id.clone())
        } else {
            self.buffering.remove(&self.client_id)
        };
        if !changed {
            return Vec::new();
        }
        let mut effects = vec![SessionEffect::Broadcast(self.tagged(json!({
            "type": "buffering",
            "buffering": buffering,
        })))];
        effects.extend(self.settle_hold(now_ms));
        effects
    }

    pub fn host_playback(
        &mut self,
        snapshot: &WatchTogetherPlaybackSnapshot,
        now_ms: i64,
    ) -> Vec<SessionEffect> {
        if !self.is_host() || self.phase != SessionPhase::Playing {
            return Vec::new();
        }
        self.resume_after_hold = snapshot.is_playing;
        self.playback = Some(HostPlayback {
            position_ms: snapshot.position_ms.max(0),
            playing: snapshot.is_playing && !self.held,
            at_ms: now_ms,
        });
        vec![self.state_message(now_ms)]
    }

    pub fn members_changed(
        &mut self,
        host_id: String,
        members: Vec<WatchTogetherMember>,
        now_ms: i64,
    ) -> Vec<SessionEffect> {
        let known: BTreeSet<String> = self.members.iter().map(|m| m.id.clone()).collect();
        let present: BTreeSet<String> = members.iter().map(|m| m.id.clone()).collect();
        let joined = present.difference(&known).count();
        self.ready.retain(|id| present.contains(id));
        self.buffering.retain(|id| present.contains(id));
        self.host_id = host_id;
        self.members = members;

        let mut effects = Vec::new();
        if self.is_host() {
            if joined > 0 {
                if let Some(content) = self.content.clone() {
                    effects.push(SessionEffect::Broadcast(
                        self.content_message(&content, self.shared_stream.as_ref()),
                    ));
                }
            }
            effects.extend(self.try_start(now_ms));
            effects.extend(self.settle_hold(now_ms));
        }
        effects
    }

    pub fn handle(&mut self, message: &Value, now_ms: i64) -> Vec<SessionEffect> {
        let from = message
            .get("from")
            .and_then(Value::as_str)
            .unwrap_or_default();
        if from.is_empty() || from == self.client_id {
            return Vec::new();
        }
        let from_host = from == self.host_id;
        match WatchTogetherProtocol::message_type(message) {
            Some("content") if from_host => self.on_content(message),
            Some("start") if from_host => self.on_start(message, now_ms),
            Some("state") if from_host => self.on_state(message, now_ms),
            Some("ready") if self.has_member(from) => self.on_ready(from, message, now_ms),
            Some("buffering") if self.has_member(from) => self.on_buffering(from, message, now_ms),
            Some("hello") if self.is_host() && self.has_member(from) => self.on_hello(now_ms),
            _ => Vec::new(),
        }
    }

    pub fn correction(
        &self,
        local_position_ms: i64,
        now_ms: i64,
        speed_correction_active: bool,
    ) -> WatchTogetherCorrection {
        match (self.is_host(), self.playback) {
            (false, Some(playback)) => WatchTogetherDriftPolicy::correction(
                local_position_ms,
                playback.position_at(now_ms),
                playback.playing,
                speed_correction_active,
            ),
            _ => WatchTogetherCorrection::None,
        }
    }

    fn has_member(&self, id: &str) -> bool {
        self.members.iter().any(|member| member.id == id)
    }

    fn tagged(&self, mut value: Value) -> Value {
        value["from"] = json!(self.client_id);
        value
    }

    fn content_message(
        &self,
        content: &WatchTogetherContent,
        shared_stream: Option<&SharedStream>,
    ) -> Value {
        let mut message = WatchTogetherProtocol::content(content);
        message["startPositionMs"] = json!(self.start_position_ms);
        if let Some(stream) = shared_stream {
            message["stream"] = json!(stream);
        }
        self.tagged(message)
    }

    fn state_message(&mut self, now_ms: i64) -> SessionEffect {
        self.sequence += 1;
        let playback = self.playback.unwrap_or(HostPlayback {
            position_ms: self.start_position_ms,
            playing: false,
            at_ms: now_ms,
        });
        SessionEffect::Broadcast(self.tagged(json!({
            "type": "state",
            "sequence": self.sequence,
            "positionMs": playback.position_at(now_ms),
            "playing": playback.playing,
        })))
    }

    fn try_start(&mut self, now_ms: i64) -> Vec<SessionEffect> {
        let everyone_ready = !self.members.is_empty()
            && self
                .members
                .iter()
                .all(|member| self.ready.contains(&member.id));
        if !self.is_host() || self.phase != SessionPhase::Selecting || !everyone_ready {
            return Vec::new();
        }
        self.phase = SessionPhase::Playing;
        self.resume_after_hold = true;
        self.playback = Some(HostPlayback {
            position_ms: self.start_position_ms,
            playing: true,
            at_ms: now_ms,
        });
        vec![
            SessionEffect::Broadcast(self.tagged(json!({
                "type": "start",
                "positionMs": self.start_position_ms,
            }))),
            SessionEffect::Start {
                position_ms: self.start_position_ms,
            },
        ]
    }

    fn settle_hold(&mut self, now_ms: i64) -> Vec<SessionEffect> {
        if !self.is_host() || self.phase != SessionPhase::Playing {
            return Vec::new();
        }
        let Some(playback) = self.playback else {
            return Vec::new();
        };
        let position_ms = playback.position_at(now_ms);
        if !self.buffering.is_empty() && !self.held {
            self.held = true;
            self.playback = Some(HostPlayback {
                position_ms,
                playing: false,
                at_ms: now_ms,
            });
            return vec![
                self.state_message(now_ms),
                SessionEffect::Remote {
                    playing: false,
                    position_ms,
                },
            ];
        }
        if self.buffering.is_empty() && self.held {
            self.held = false;
            let playing = self.resume_after_hold;
            self.playback = Some(HostPlayback {
                position_ms,
                playing,
                at_ms: now_ms,
            });
            return vec![
                self.state_message(now_ms),
                SessionEffect::Remote {
                    playing,
                    position_ms,
                },
            ];
        }
        Vec::new()
    }

    fn on_content(&mut self, message: &Value) -> Vec<SessionEffect> {
        let Some(content) = WatchTogetherProtocol::content_from(message) else {
            return Vec::new();
        };
        self.phase = SessionPhase::Selecting;
        self.ready.clear();
        self.buffering.clear();
        self.playback = None;
        self.start_position_ms = message
            .get("startPositionMs")
            .and_then(Value::as_i64)
            .unwrap_or(0)
            .max(0);
        self.shared_stream = message
            .get("stream")
            .and_then(|value| serde_json::from_value::<SharedStream>(value.clone()).ok())
            .filter(SharedStream::is_playable);
        self.content = Some(content.clone());
        match self.shared_stream.clone() {
            Some(stream) => vec![SessionEffect::PlaySharedStream { content, stream }],
            None => vec![SessionEffect::PickStream(content)],
        }
    }

    fn on_start(&mut self, message: &Value, now_ms: i64) -> Vec<SessionEffect> {
        if self.phase != SessionPhase::Selecting || !self.ready.contains(&self.client_id) {
            return Vec::new();
        }
        let position_ms = WatchTogetherProtocol::position_ms(message).unwrap_or(0);
        self.phase = SessionPhase::Playing;
        self.playback = Some(HostPlayback {
            position_ms,
            playing: true,
            at_ms: now_ms,
        });
        vec![SessionEffect::Start { position_ms }]
    }

    fn on_state(&mut self, message: &Value, now_ms: i64) -> Vec<SessionEffect> {
        let (Some(position_ms), Some(playing)) = (
            WatchTogetherProtocol::position_ms(message),
            WatchTogetherProtocol::playing(message),
        ) else {
            return Vec::new();
        };
        let sequence = message.get("sequence").and_then(Value::as_i64).unwrap_or(0);
        if sequence != 0 && sequence <= self.sequence {
            return Vec::new();
        }
        self.sequence = sequence.max(self.sequence);
        let joining = match self.phase {
            SessionPhase::Selecting if self.ready.contains(&self.client_id) => true,
            SessionPhase::Playing => false,
            _ => return Vec::new(),
        };
        self.phase = SessionPhase::Playing;
        self.playback = Some(HostPlayback {
            position_ms,
            playing,
            at_ms: now_ms,
        });
        if joining {
            vec![SessionEffect::Start { position_ms }]
        } else {
            vec![SessionEffect::Remote {
                playing,
                position_ms,
            }]
        }
    }

    fn on_hello(&mut self, now_ms: i64) -> Vec<SessionEffect> {
        let mut effects = Vec::new();
        if let Some(content) = self.content.clone() {
            effects.push(SessionEffect::Broadcast(
                self.content_message(&content, self.shared_stream.as_ref()),
            ));
        }
        if self.phase == SessionPhase::Playing {
            effects.push(self.state_message(now_ms));
        }
        effects
    }

    fn on_ready(&mut self, from: &str, message: &Value, now_ms: i64) -> Vec<SessionEffect> {
        let ready = message
            .get("ready")
            .and_then(Value::as_bool)
            .unwrap_or(true);
        if ready {
            self.ready.insert(from.to_string());
        } else {
            self.ready.remove(from);
        }
        if !self.is_host() || !ready {
            return Vec::new();
        }
        match self.phase {
            SessionPhase::Selecting => self.try_start(now_ms),
            SessionPhase::Playing => vec![self.state_message(now_ms)],
            SessionPhase::Lobby => Vec::new(),
        }
    }

    fn on_buffering(&mut self, from: &str, message: &Value, now_ms: i64) -> Vec<SessionEffect> {
        let buffering = message
            .get("buffering")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        if buffering {
            self.buffering.insert(from.to_string());
        } else {
            self.buffering.remove(from);
        }
        self.settle_hold(now_ms)
    }
}

#[cfg(test)]
mod tests;
