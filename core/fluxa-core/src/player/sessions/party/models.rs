use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct PartySource {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub info_hash: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_idx: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub filename: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub video_size: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub binge_group: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub addon_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct PartyMedia {
    #[serde(rename = "type")]
    pub kind: String,
    pub meta_id: String,
    pub meta_type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub video_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub season: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub episode: Option<i32>,
    #[serde(default)]
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub episode_title: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub poster: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub background: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub logo: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<PartySource>,
}

impl PartyMedia {
    pub fn key(&self) -> String {
        media_key(&self.meta_type, &self.meta_id, self.season, self.episode)
    }
}

pub fn media_key(meta_type: &str, meta_id: &str, season: Option<i32>, episode: Option<i32>) -> String {
    format!(
        "{}|{}|{}|{}",
        meta_type.to_lowercase(),
        meta_id,
        season.map(|v| v.to_string()).unwrap_or_default(),
        episode.map(|v| v.to_string()).unwrap_or_default()
    )
}

fn normal_rate() -> f32 {
    1.0
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct PartyPlayback {
    pub seq: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub member_id: Option<String>,
    pub paused: bool,
    pub position_ms: i64,
    #[serde(default = "normal_rate")]
    pub rate: f32,
    pub at: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub media_key: Option<String>,
    #[serde(default)]
    pub waiting: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct PartyMember {
    pub id: String,
    pub display_name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub avatar_color: Option<String>,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PartyMemberStatus {
    #[default]
    Browsing,
    Loading,
    Buffering,
    Ready,
    Behind,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct PartyPresence {
    pub member_id: String,
    #[serde(default)]
    pub status: PartyMemberStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub media_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub behind_ms: Option<i64>,
}

impl PartyPresence {
    pub fn browsing(member_id: &str) -> Self {
        Self {
            member_id: member_id.to_owned(),
            status: PartyMemberStatus::Browsing,
            media_key: None,
            behind_ms: None,
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct PartyPreview {
    #[serde(default)]
    pub media: Option<PartyMedia>,
    #[serde(default)]
    pub host: Option<PartyMember>,
    #[serde(default)]
    pub members: u32,
    #[serde(default)]
    pub max_members: u32,
    #[serde(default)]
    pub error: Option<String>,
}

impl PartyPreview {
    pub fn is_full(&self) -> bool {
        self.max_members > 0 && self.max_members <= self.members
    }
}

fn default_max_members() -> u32 {
    3
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct PartyInfo {
    pub id: String,
    pub code: String,
    #[serde(default)]
    pub host_member_id: Option<String>,
    #[serde(default)]
    pub host_only: bool,
    pub media: PartyMedia,
    #[serde(default)]
    pub playback: Option<PartyPlayback>,
    #[serde(default = "default_max_members")]
    pub max_members: u32,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq)]
pub struct PartySnapshot {
    #[serde(default)]
    pub party: Option<PartyInfo>,
    #[serde(default)]
    pub members: Vec<PartyMember>,
    #[serde(default)]
    pub member_id: Option<String>,
    #[serde(default)]
    pub server_time_ms: Option<i64>,
    #[serde(default)]
    pub error: Option<String>,
    #[serde(default)]
    pub ok: Option<bool>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PartyError {
    NotFound,
    Full,
    Removed,
    Ended,
    RateLimited,
    SignInRequired,
    Unavailable,
    Network,
}

impl PartyError {
    pub fn from_code(code: Option<&str>) -> Option<Self> {
        Some(match code? {
            "party_not_found" => Self::NotFound,
            "party_full" => Self::Full,
            "removed" | "not_member" => Self::Removed,
            "party_ended" => Self::Ended,
            "rate_limited" => Self::RateLimited,
            "unavailable" => Self::Unavailable,
            _ => Self::Network,
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ActiveParty {
    pub id: String,
    pub code: String,
    pub member_id: String,
    pub host_member_id: Option<String>,
    pub members: Vec<PartyMember>,
    pub presence: HashMap<String, PartyPresence>,
    pub media: PartyMedia,
    pub max_members: u32,
    pub connected: bool,
    pub host_only: bool,
}

impl ActiveParty {
    pub fn is_host(&self) -> bool {
        self.host_member_id.as_deref() == Some(self.member_id.as_str())
    }

    pub fn can_control(&self) -> bool {
        !self.host_only || self.is_host()
    }

    pub fn member(&self, id: &str) -> Option<&PartyMember> {
        self.members.iter().find(|member| member.id == id)
    }
}

pub const PARTY_CODE_LENGTH: usize = 6;
const PARTY_CODE_ALPHABET: &str = "ABCDEFGHJKLMNPQRSTUVWXYZ23456789";

pub fn normalize_party_code(input: &str) -> String {
    input
        .to_uppercase()
        .chars()
        .filter(|c| PARTY_CODE_ALPHABET.contains(*c))
        .collect()
}

pub fn is_valid_party_code(input: &str) -> bool {
    normalize_party_code(input).len() == PARTY_CODE_LENGTH
}
