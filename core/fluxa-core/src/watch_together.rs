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
}

impl WatchTogetherContent {
    pub fn matches(&self, other: &Self) -> bool {
        self.id == other.id
            && (self.video_id.is_none()
                || other.video_id.is_none()
                || self.video_id == other.video_id)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WatchTogetherPlaybackSnapshot {
    pub position_ms: i64,
    pub duration_ms: i64,
    pub is_playing: bool,
    #[serde(default)]
    pub is_buffering: bool,
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
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<WatchTogetherContent>,
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
            content: None,
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
        Some(WatchTogetherContent {
            id: value.get("contentId")?.as_str()?.to_string(),
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
    value
}

#[cfg(test)]
mod tests {
    use super::*;

    fn content() -> WatchTogetherContent {
        WatchTogetherContent {
            id: "tt123".to_string(),
            content_type: "series".to_string(),
            video_id: Some("tt123:1:2".to_string()),
            title: "Episode 2".to_string(),
        }
    }

    #[test]
    fn protocol_matches_existing_server_wire_shape() {
        assert_eq!(WatchTogetherProtocol::create("Alice"), json!({"type": "create", "name": "Alice"}));
        assert_eq!(WatchTogetherProtocol::join("ABC234", "Alice"), json!({"type": "join", "room": "ABC234", "name": "Alice"}));
        assert_eq!(WatchTogetherProtocol::content(&content())["contentId"], "tt123");
    }

    #[test]
    fn playback_state_round_trips_content_and_snapshot() {
        let snapshot = WatchTogetherPlaybackSnapshot {
            position_ms: 12_000,
            duration_ms: 30_000,
            is_playing: true,
            is_buffering: false,
        };
        let value = WatchTogetherProtocol::playback_state(&snapshot, Some(&content()));
        assert_eq!(WatchTogetherProtocol::message_type(&value), Some("state"));
        assert_eq!(WatchTogetherProtocol::position_ms(&value), Some(12_000));
        assert_eq!(WatchTogetherProtocol::playing(&value), Some(true));
        assert_eq!(WatchTogetherProtocol::content_from(&value), Some(content()));
    }

    #[test]
    fn drift_policy_is_shared_across_platforms() {
        assert_eq!(
            WatchTogetherDriftPolicy::correction(1_000, 1_100, true, false),
            WatchTogetherCorrection::None
        );
        assert_eq!(
            WatchTogetherDriftPolicy::correction(1_000, 1_600, true, false),
            WatchTogetherCorrection::Speed(1.03)
        );
        assert_eq!(
            WatchTogetherDriftPolicy::correction(1_000, 2_500, true, true),
            WatchTogetherCorrection::Seek(2_500)
        );
        assert_eq!(
            WatchTogetherDriftPolicy::correction(1_000, 1_100, true, true),
            WatchTogetherCorrection::ResetSpeed
        );
    }

    #[test]
    fn relayed_sync_and_peer_state_decode_the_same() {
        let snapshot = WatchTogetherPlaybackSnapshot {
            position_ms: 12_000,
            duration_ms: 30_000,
            is_playing: true,
            is_buffering: false,
        };
        let peer = WatchTogetherProtocol::playback_state(&snapshot, Some(&content()));
        let relayed = json!({
            "type": "sync", "sequence": 4, "senderId": "abc", "serverTimeMs": 1_700_000_000_000i64,
            "positionMs": 12_000, "durationMs": 30_000, "playing": true, "buffering": false,
            "contentId": "tt123", "contentType": "series", "videoId": "tt123:1:2", "title": "Episode 2",
        });

        let from_peer = WatchTogetherProtocol::decode(&peer, Some(&content()));
        let from_relay = WatchTogetherProtocol::decode(&relayed, Some(&content()));

        assert_eq!(from_peer.kind, WatchTogetherIncomingKind::Sync);
        assert_eq!(from_relay.kind, WatchTogetherIncomingKind::Sync);
        assert_eq!(from_peer.position_ms, from_relay.position_ms);
        assert_eq!(from_peer.playing, from_relay.playing);
        assert_eq!(from_peer.content, from_relay.content);
        assert!(from_peer.content_matches_local);
        assert_eq!(from_peer.sequence, None);
        assert_eq!(from_relay.sequence, Some(4));
    }

    #[test]
    fn room_message_flags_only_the_host_member() {
        let decoded = WatchTogetherProtocol::decode(
            &json!({
                "type": "room", "room": "ABC234", "clientId": "b2", "hostId": "a1",
                "members": [
                    {"id": "a1", "name": "Alice", "buffering": false},
                    {"id": "b2", "name": "Bob", "buffering": true},
                    {"name": "ghost"},
                ],
            }),
            None,
        );

        assert_eq!(decoded.kind, WatchTogetherIncomingKind::Room);
        assert_eq!(decoded.room_code.as_deref(), Some("ABC234"));
        assert_eq!(decoded.members.len(), 2);
        assert!(decoded.members.iter().filter(|m| m.is_host).count() == 1);
        assert!(decoded.members.iter().any(|m| m.id == "a1" && m.is_host));
        assert!(decoded.members.iter().any(|m| m.id == "b2" && m.buffering));
    }

    #[test]
    fn sync_for_a_different_episode_is_flagged_as_mismatched() {
        let decoded = WatchTogetherProtocol::decode(
            &json!({"type": "sync", "contentId": "tt123", "videoId": "tt123:1:9"}),
            Some(&content()),
        );
        assert!(!decoded.content_matches_local);
    }

    #[test]
    fn content_matching_preserves_episode_identity() {
        let same = content();
        assert!(same.matches(&content()));
        assert!(!same.matches(&WatchTogetherContent {
            video_id: Some("tt123:1:3".to_string()),
            ..content()
        }));
    }
}
