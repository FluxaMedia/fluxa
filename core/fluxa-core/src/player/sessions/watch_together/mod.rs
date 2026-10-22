use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

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
    #[serde(default)]
    pub max_members: u32,
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
            max_members: 0,
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

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SharedStream {
    pub url: String,
    #[serde(default)]
    pub title: String,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub headers: BTreeMap<String, String>,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum SessionPhase {
    #[default]
    Lobby,
    Selecting,
    Playing,
}
