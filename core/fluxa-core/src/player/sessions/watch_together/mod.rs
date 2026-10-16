use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WatchTogetherContent {
    pub id: String,
    pub content_type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub video_id: Option<String>,
    #[serde(default)]
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub artwork: Option<WatchTogetherArtwork>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WatchTogetherArtwork {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub backdrop: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub logo: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub episode_title: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub season: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub episode: Option<u32>,
}

impl WatchTogetherArtwork {
    fn from_wire(value: &Value) -> Option<Self> {
        let image = |key: &str| {
            value
                .get(key)
                .and_then(Value::as_str)
                .filter(|url| url.starts_with("https://") && url.len() <= 2048)
                .map(str::to_string)
        };
        let number = |key: &str| {
            value
                .get(key)
                .and_then(Value::as_u64)
                .and_then(|number| u32::try_from(number).ok())
        };
        let artwork = Self {
            backdrop: image("backdrop"),
            logo: image("logo"),
            episode_title: value
                .get("episodeTitle")
                .and_then(Value::as_str)
                .filter(|title| title.len() <= 300)
                .map(str::to_string),
            season: number("season"),
            episode: number("episode"),
        };
        (artwork != Self::default()).then_some(artwork)
    }
}

impl WatchTogetherContent {
    fn episode(&self) -> Option<(u32, u32)> {
        let mut parts = self.video_id.as_deref()?.rsplit(':');
        let episode = parts.next()?.parse().ok()?;
        let season = parts.next()?.parse().ok()?;
        parts.next()?;
        Some((season, episode))
    }

    pub fn is_same_episode(&self, other: &Self) -> bool {
        self.id == other.id && self.episode() == other.episode()
    }

    pub fn matches(&self, other: &Self) -> bool {
        self.id == other.id
            && (self.video_id.is_none()
                || other.video_id.is_none()
                || self.video_id == other.video_id)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct WatchTogetherPlaybackSnapshot {
    pub position_ms: i64,
    pub duration_ms: i64,
    pub is_playing: bool,
    #[serde(default)]
    pub is_buffering: bool,
    #[serde(default = "normal_rate")]
    pub rate: f64,
}

fn normal_rate() -> f64 {
    1.0
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum WatchTogetherConnectionState {
    Disconnected,
    Connecting,
    Connected,
    Error,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum WatchTogetherRole {
    None,
    Host,
    Guest,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WatchTogetherMember {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub is_host: bool,
    #[serde(default)]
    pub buffering: bool,
    #[serde(default)]
    pub ready: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub avatar: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(default)]
    pub status: WatchTogetherMemberStatus,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum WatchTogetherMemberStatus {
    #[default]
    Waiting,
    Picking,
    Ready,
    Watching,
    Paused,
    Buffering,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", rename_all_fields = "camelCase", tag = "kind")]
pub enum WatchTogetherEventKind {
    Paused,
    Resumed,
    Seeked { position_ms: i64 },
    Speed { rate: f64 },
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct WatchTogetherEvent {
    pub id: u64,
    pub by: String,
    #[serde(default)]
    pub name: String,
    #[serde(flatten)]
    pub kind: WatchTogetherEventKind,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WatchTogetherChatMessage {
    pub id: u64,
    pub by: String,
    #[serde(default)]
    pub name: String,
    pub text: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct WatchTogetherState {
    pub connection_state: WatchTogetherConnectionState,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub room_code: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub client_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub host_id: Option<String>,
    pub role: WatchTogetherRole,
    #[serde(default)]
    pub members: Vec<WatchTogetherMember>,
    #[serde(default)]
    pub phase: SessionPhase,
    #[serde(default)]
    pub locked: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<WatchTogetherContent>,
    #[serde(default)]
    pub content_mismatch: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_event: Option<WatchTogetherEvent>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub chat: Vec<WatchTogetherChatMessage>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
}

impl Default for WatchTogetherState {
    fn default() -> Self {
        Self {
            connection_state: WatchTogetherConnectionState::Disconnected,
            room_code: None,
            client_id: None,
            host_id: None,
            role: WatchTogetherRole::None,
            members: Vec::new(),
            phase: SessionPhase::Lobby,
            locked: false,
            content: None,
            content_mismatch: false,
            last_event: None,
            chat: Vec::new(),
            error_message: None,
        }
    }
}

impl WatchTogetherState {
    pub fn in_room(&self) -> bool {
        self.room_code.as_ref().is_some_and(|code| !code.is_empty())
    }

    pub fn is_host(&self) -> bool {
        self.role == WatchTogetherRole::Host
    }

    pub fn can_control_locally(&self) -> bool {
        !self.in_room() || self.is_host()
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum WatchTogetherIncomingKind {
    Room,
    Members,
    Content,
    Sync,
    Pong,
    Error,
    #[default]
    Unknown,
}

#[derive(Clone, Debug, Default, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct WatchTogetherIncoming {
    pub kind: WatchTogetherIncomingKind,
    pub room_code: Option<String>,
    pub client_id: Option<String>,
    pub host_id: Option<String>,
    pub members: Vec<WatchTogetherMember>,
    pub content: Option<WatchTogetherContent>,
    pub content_matches_local: bool,
    pub sequence: Option<i64>,
    pub position_ms: Option<i64>,
    pub playing: Option<bool>,
    pub server_time_ms: Option<i64>,
    pub client_time_ms: Option<i64>,
    pub error_message: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum WatchTogetherCorrection {
    None,
    Seek(i64),
    Speed(f32),
    ResetSpeed,
}

pub struct WatchTogetherDriftPolicy;

impl WatchTogetherDriftPolicy {
    pub const NORMAL_SPEED: f32 = 1.0;
    pub const SOFT_DRIFT_MS: i64 = 250;
    pub const HARD_SEEK_DRIFT_MS: i64 = 750;

    pub fn correction(
        local_position_ms: i64,
        expected_position_ms: i64,
        host_playing: bool,
        speed_correction_active: bool,
    ) -> WatchTogetherCorrection {
        let drift = expected_position_ms - local_position_ms;
        let magnitude = drift.abs();
        if magnitude > Self::HARD_SEEK_DRIFT_MS {
            WatchTogetherCorrection::Seek(expected_position_ms.max(0))
        } else if host_playing && magnitude > Self::SOFT_DRIFT_MS {
            let value = (Self::NORMAL_SPEED + (drift as f32 / 5_000.0)).clamp(0.97, 1.03);
            WatchTogetherCorrection::Speed(value)
        } else if speed_correction_active {
            WatchTogetherCorrection::ResetSpeed
        } else {
            WatchTogetherCorrection::None
        }
    }
}

pub struct WatchTogetherProtocol;

impl WatchTogetherProtocol {
    pub const ROOM_CODE_ALPHABET: &'static str = "ABCDEFGHJKLMNPQRSTUVWXYZ23456789";
    pub const ROOM_CODE_LENGTH: usize = 6;

    pub fn create(display_name: &str) -> Value {
        json!({"type": "create", "name": display_name})
    }

    pub fn join(room_code: &str, display_name: &str) -> Value {
        json!({"type": "join", "room": room_code, "name": display_name})
    }

    pub fn leave() -> Value {
        json!({"type": "leave"})
    }

    pub fn ping(client_time_ms: i64) -> Value {
        json!({"type": "ping", "clientTimeMs": client_time_ms})
    }

    pub fn buffering(buffering: bool) -> Value {
        json!({"type": "buffering", "buffering": buffering})
    }

    pub fn content(content: &WatchTogetherContent) -> Value {
        let mut value = content_fields(content);
        value.insert("type".to_string(), Value::String("content".to_string()));
        Value::Object(value)
    }

    pub fn playback_state(
        snapshot: &WatchTogetherPlaybackSnapshot,
        content: Option<&WatchTogetherContent>,
    ) -> Value {
        let mut value = Map::new();
        value.insert("type".to_string(), Value::String("state".to_string()));
        value.insert("positionMs".to_string(), json!(snapshot.position_ms));
        value.insert("durationMs".to_string(), json!(snapshot.duration_ms));
        value.insert("playing".to_string(), json!(snapshot.is_playing));
        value.insert("buffering".to_string(), json!(snapshot.is_buffering));
        if let Some(content) = content {
            value.extend(content_fields(content));
        }
        Value::Object(value)
    }

    pub fn message_type(value: &Value) -> Option<&str> {
        value.get("type").and_then(Value::as_str)
    }

    pub fn content_from(value: &Value) -> Option<WatchTogetherContent> {
        let id = value.get("contentId")?.as_str()?;
        let text = |key: &str| value.get(key).and_then(Value::as_str);
        let oversized = id.is_empty()
            || id.len() > 128
            || [text("contentType"), text("videoId")]
                .into_iter()
                .flatten()
                .any(|field| field.len() > 128)
            || text("title").is_some_and(|title| title.len() > 300);
        if oversized {
            return None;
        }
        Some(WatchTogetherContent {
            id: id.to_string(),
            content_type: value
                .get("contentType")
                .and_then(Value::as_str)
                .unwrap_or("movie")
                .to_string(),
            video_id: value
                .get("videoId")
                .and_then(Value::as_str)
                .map(str::to_string),
            title: value
                .get("title")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string(),
            artwork: value.get("artwork").and_then(WatchTogetherArtwork::from_wire),
        })
    }

    pub fn position_ms(value: &Value) -> Option<i64> {
        value.get("positionMs").and_then(Value::as_i64)
    }

    pub fn playing(value: &Value) -> Option<bool> {
        value.get("playing").and_then(Value::as_bool)
    }

    pub fn server_time_ms(value: &Value) -> Option<i64> {
        value.get("serverTimeMs").and_then(Value::as_i64)
    }

    pub fn client_time_ms(value: &Value) -> Option<i64> {
        value.get("clientTimeMs").and_then(Value::as_i64)
    }

    pub fn members_from(value: Option<&Value>, host_id: &str) -> Vec<WatchTogetherMember> {
        let Some(Value::Array(items)) = value else {
            return Vec::new();
        };
        items
            .iter()
            .filter_map(|item| {
                let id = item.get("id").and_then(Value::as_str).unwrap_or_default();
                if id.is_empty() {
                    return None;
                }
                Some(WatchTogetherMember {
                    id: id.to_string(),
                    name: item
                        .get("name")
                        .and_then(Value::as_str)
                        .unwrap_or("Guest")
                        .to_string(),
                    is_host: id == host_id,
                    buffering: item
                        .get("buffering")
                        .and_then(Value::as_bool)
                        .unwrap_or(false),
                    ready: false,
                    status: WatchTogetherMemberStatus::Waiting,
                    avatar: item
                        .get("avatar")
                        .and_then(Value::as_str)
                        .map(str::to_string),
                    color: item
                        .get("color")
                        .and_then(Value::as_str)
                        .map(str::to_string),
                })
            })
            .collect()
    }

    pub fn decode(
        value: &Value,
        local_content: Option<&WatchTogetherContent>,
    ) -> WatchTogetherIncoming {
        let kind = match Self::message_type(value) {
            Some("room") => WatchTogetherIncomingKind::Room,
            Some("members") => WatchTogetherIncomingKind::Members,
            Some("content") => WatchTogetherIncomingKind::Content,
            Some("sync") | Some("state") => WatchTogetherIncomingKind::Sync,
            Some("pong") => WatchTogetherIncomingKind::Pong,
            Some("error") => WatchTogetherIncomingKind::Error,
            _ => return WatchTogetherIncoming::default(),
        };
        let host_id = value.get("hostId").and_then(Value::as_str);
        let content = Self::content_from(value);
        let content_matches_local = match (content.as_ref(), local_content) {
            (Some(remote), Some(local)) => remote.matches(local),
            _ => true,
        };
        WatchTogetherIncoming {
            kind,
            room_code: value
                .get("room")
                .and_then(Value::as_str)
                .map(str::to_string),
            client_id: value
                .get("clientId")
                .and_then(Value::as_str)
                .map(str::to_string),
            host_id: host_id.map(str::to_string),
            members: Self::members_from(value.get("members"), host_id.unwrap_or_default()),
            content,
            content_matches_local,
            sequence: value.get("sequence").and_then(Value::as_i64),
            position_ms: Self::position_ms(value),
            playing: Self::playing(value),
            server_time_ms: Self::server_time_ms(value),
            client_time_ms: Self::client_time_ms(value),
            error_message: value
                .get("message")
                .and_then(Value::as_str)
                .map(str::to_string),
        }
    }
}

fn content_fields(content: &WatchTogetherContent) -> Map<String, Value> {
    let mut value = Map::new();
    value.insert("contentId".to_string(), json!(content.id));
    value.insert("contentType".to_string(), json!(content.content_type));
    if let Some(video_id) = &content.video_id {
        value.insert("videoId".to_string(), json!(video_id));
    }
    value.insert("title".to_string(), json!(content.title));
    if let Some(artwork) = &content.artwork {
        value.insert("artwork".to_string(), json!(artwork));
    }
    value
}

mod session;

pub use session::{SessionEffect, SessionPhase, SharedStream, WatchTogetherSession};

#[cfg(test)]
mod tests;
