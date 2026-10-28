use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum StreamType {
    P2p,
    Debrid,
    Usenet,
    StremioUsenet,
    Archive,
    Http,
    Live,
    External,
    Youtube,
    Error,
    Statistic,
    Info,
}

impl StreamType {
    pub fn as_str(self) -> &'static str {
        match self {
            StreamType::P2p => "p2p",
            StreamType::Debrid => "debrid",
            StreamType::Usenet => "usenet",
            StreamType::StremioUsenet => "stremio-usenet",
            StreamType::Archive => "archive",
            StreamType::Http => "http",
            StreamType::Live => "live",
            StreamType::External => "external",
            StreamType::Youtube => "youtube",
            StreamType::Error => "error",
            StreamType::Statistic => "statistic",
            StreamType::Info => "info",
        }
    }

    pub fn is_usenet(self) -> bool {
        matches!(self, StreamType::Usenet | StreamType::StremioUsenet)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MediaType {
    Movie,
    Series,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Subtitle {
    pub id: Option<String>,
    pub url: String,
    pub lang: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct BehaviorHints {
    pub filename: Option<String>,
    pub folder_name: Option<String>,
    pub video_size: Option<f64>,
    pub folder_size: Option<Value>,
    pub video_hash: Option<String>,
    pub not_web_ready: bool,
    pub proxy_headers: Option<ProxyHeaders>,
    pub seeders: Option<f64>,
    pub indexer: Option<String>,
    pub service: Option<String>,
    pub cached: Option<bool>,
    pub duration: Option<f64>,
    pub publish_date: Option<String>,
    #[serde(flatten)]
    pub other: HashMap<String, Value>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct ProxyHeaders {
    pub request: HashMap<String, String>,
    pub response: HashMap<String, String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct RawStream {
    pub name: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub url: Option<String>,
    pub info_hash: Option<String>,
    pub file_idx: Option<u32>,
    pub sources: Vec<String>,
    pub external_url: Option<String>,
    pub yt_id: Option<String>,
    pub nzb_url: Option<String>,
    pub servers: Vec<String>,
    pub subtitles: Vec<Subtitle>,
    pub size: Option<f64>,
    pub size_bytes: Option<f64>,
    pub behavior_hints: BehaviorHints,
    #[serde(flatten)]
    pub other: HashMap<String, Value>,
}

const ARCHIVE_KEYS: [&str; 5] = ["rarUrls", "zipUrls", "7zipUrls", "tgzUrls", "tarUrls"];

impl RawStream {
    pub fn has_archive(&self) -> bool {
        ARCHIVE_KEYS.iter().any(|key| {
            self.other
                .get(*key)
                .and_then(Value::as_array)
                .is_some_and(|list| !list.is_empty())
        })
    }

    pub fn is_playable(&self) -> bool {
        self.url.is_some()
            || self.external_url.is_some()
            || self.info_hash.is_some()
            || self.yt_id.is_some()
            || self.nzb_url.is_some()
            || self.has_archive()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PinPosition {
    Top,
    Bottom,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct AddonRef {
    pub instance_id: String,
    pub preset_id: String,
    pub name: String,
    pub pin: Option<PinPosition>,
    pub passthrough: bool,
    pub library: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ServiceRef {
    pub id: String,
    pub cached: bool,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct TorrentInfo {
    pub info_hash: String,
    pub file_idx: Option<u32>,
    pub seeders: Option<u32>,
    pub sources: Vec<String>,
    pub private: bool,
    pub freeleech: bool,
    pub file: Option<String>,
    pub title: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ParsedFile {
    pub title: Option<String>,
    pub year: Option<String>,
    pub country: Option<String>,
    pub episode_title: Option<String>,
    pub seasons: Vec<u32>,
    pub episodes: Vec<u32>,
    pub volumes: Vec<u32>,
    pub date: Option<String>,
    pub resolution: Option<String>,
    pub quality: Option<String>,
    pub encode: Option<String>,
    pub release_group: Option<String>,
    pub visual_tags: Vec<String>,
    pub audio_tags: Vec<String>,
    pub audio_channels: Vec<String>,
    pub languages: Vec<String>,
    pub subtitles: Vec<String>,
    pub editions: Vec<String>,
    pub network: Option<String>,
    pub site: Option<String>,
    pub container: Option<String>,
    pub extension: Option<String>,
    pub regraded: bool,
    pub repack: bool,
    pub proper: bool,
    pub uncensored: bool,
    pub unrated: bool,
    pub upscaled: bool,
    pub subbed: bool,
    pub dubbed: bool,
    pub season_pack: bool,
    pub folder_seasons: Vec<u32>,
    pub folder_episodes: Vec<u32>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RegexMatch {
    pub name: Option<String>,
    pub pattern: String,
    pub index: usize,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ExprMatch {
    pub name: Option<String>,
    pub index: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SeadexMethod {
    Hash,
    Group,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Seadex {
    pub is_best: bool,
    pub is_seadex: bool,
    pub method: SeadexMethod,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Passthrough {
    pub all: bool,
    pub stages: Vec<String>,
}

impl Passthrough {
    pub fn covers(&self, stage: &str) -> bool {
        self.all || self.stages.iter().any(|s| s == stage)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParsedStream {
    pub id: String,
    pub addon: AddonRef,
    #[serde(rename = "type")]
    pub stream_type: StreamType,
    pub url: Option<String>,
    pub nzb_url: Option<String>,
    pub external_url: Option<String>,
    pub yt_id: Option<String>,
    pub subtitles: Vec<Subtitle>,
    pub request_headers: HashMap<String, String>,
    pub response_headers: HashMap<String, String>,
    pub not_web_ready: bool,
    pub video_hash: Option<String>,
    pub filename: Option<String>,
    pub folder_name: Option<String>,
    pub size: Option<u64>,
    pub folder_size: Option<u64>,
    pub indexer: Option<String>,
    pub service: Option<ServiceRef>,
    pub duration: Option<u64>,
    pub age: Option<f64>,
    pub bitrate: Option<u64>,
    pub message: Option<String>,
    pub library: bool,
    pub id_matched: bool,
    pub torrent: Option<TorrentInfo>,
    pub parsed: ParsedFile,
    pub passthrough: Passthrough,
    pub keyword_matched: bool,
    pub regex_matched: Option<RegexMatch>,
    pub ranked_regexes_matched: Vec<RegexMatch>,
    pub regex_score: f64,
    pub stream_expression_matched: Option<ExprMatch>,
    pub ranked_stream_expressions_matched: Vec<ExprMatch>,
    pub stream_expression_score: f64,
    pub seadex: Option<Seadex>,
}

impl ParsedStream {
    pub fn blank(id: impl Into<String>, addon: AddonRef, stream_type: StreamType) -> Self {
        Self {
            id: id.into(),
            addon,
            stream_type,
            url: None,
            nzb_url: None,
            external_url: None,
            yt_id: None,
            subtitles: Vec::new(),
            request_headers: HashMap::new(),
            response_headers: HashMap::new(),
            not_web_ready: false,
            video_hash: None,
            filename: None,
            folder_name: None,
            size: None,
            folder_size: None,
            indexer: None,
            service: None,
            duration: None,
            age: None,
            bitrate: None,
            message: None,
            library: false,
            id_matched: false,
            torrent: None,
            parsed: ParsedFile::default(),
            passthrough: Passthrough::default(),
            keyword_matched: false,
            regex_matched: None,
            ranked_regexes_matched: Vec::new(),
            regex_score: 0.0,
            stream_expression_matched: None,
            ranked_stream_expressions_matched: Vec::new(),
            stream_expression_score: 0.0,
            seadex: None,
        }
    }

    pub fn seeders(&self) -> Option<u32> {
        self.torrent.as_ref().and_then(|t| t.seeders)
    }

    pub fn is_cached(&self) -> Option<bool> {
        self.service.as_ref().map(|s| s.cached)
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct TitleEntry {
    pub title: String,
    pub language: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct TitleConflict {
    pub title: String,
    pub year: Option<i32>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct SeasonInfo {
    pub season_number: u32,
    pub episode_count: u32,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct EpisodeTitle {
    pub title: String,
    pub language: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct RequestContext {
    pub media_type: MediaType,
    pub is_anime: bool,
    pub season: Option<u32>,
    pub episode: Option<u32>,
    pub titles: Vec<TitleEntry>,
    pub title_conflicts: Vec<TitleConflict>,
    pub year: Option<i32>,
    pub year_end: Option<i32>,
    pub release_years: Vec<i32>,
    pub season_year: Option<i32>,
    pub country: Option<String>,
    pub episode_titles: Vec<EpisodeTitle>,
    pub episode_air_dates: Vec<String>,
    pub tvdb_season: Option<u32>,
    pub tvdb_episode: Option<u32>,
    pub absolute_episode: Option<u32>,
    pub relative_absolute_episode: Option<u32>,
    pub seasons: Vec<SeasonInfo>,
    pub original_language: Option<String>,
    pub runtime: Option<u32>,
    pub genres: Vec<String>,
    pub days_since_release: Option<i64>,
}

impl Default for RequestContext {
    fn default() -> Self {
        Self {
            media_type: MediaType::Movie,
            is_anime: false,
            season: None,
            episode: None,
            titles: Vec::new(),
            title_conflicts: Vec::new(),
            year: None,
            year_end: None,
            release_years: Vec::new(),
            season_year: None,
            country: None,
            episode_titles: Vec::new(),
            episode_air_dates: Vec::new(),
            tvdb_season: None,
            tvdb_episode: None,
            absolute_episode: None,
            relative_absolute_episode: None,
            seasons: Vec::new(),
            original_language: None,
            runtime: None,
            genres: Vec::new(),
            days_since_release: None,
        }
    }
}
