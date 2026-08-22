use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::HashMap;

// `extra` catches any field an addon sends that isn't modeled above. Effect
// payloads echo these objects back to the platform verbatim, so a field this
// struct doesn't know about must still survive a decode/encode round trip
// instead of silently vanishing.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct MetaItem {
    pub id: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub name: String,
    pub poster: Option<String>,
    pub poster_shape: Option<String>,
    pub background: Option<String>,
    pub logo: Option<String>,
    pub description: Option<String>,
    pub release_info: Option<String>,
    pub imdb_rating: Option<String>,
    pub released: Option<String>,
    pub genres: Option<Vec<String>>,
    pub director: Option<Vec<String>>,
    pub cast: Option<Vec<String>>,
    pub year: Option<String>,
    pub runtime: Option<String>,
    pub language: Option<String>,
    pub country: Option<String>,
    pub awards: Option<String>,
    pub website: Option<String>,
    pub trailers: Option<Vec<serde_json::Value>>,
    pub videos: Option<Vec<Video>>,
    pub links: Option<Vec<Link>>,
    pub behavior_hints: Option<MetaBehaviorHints>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Video {
    pub id: String,
    pub title: Option<String>,
    pub released: Option<String>,
    pub season: Option<i32>,
    pub episode: Option<i32>,
    pub thumbnail: Option<String>,
    pub streams: Option<Vec<Stream>>,
    pub available: Option<bool>,
    pub trailer: Option<String>,
    pub trailers: Option<Vec<Stream>>,
    pub overview: Option<String>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Stream {
    pub url: Option<String>,
    pub yt_id: Option<String>,
    pub info_hash: Option<String>,
    pub file_idx: Option<i32>,
    pub file_must_include: Option<String>,
    pub nzb_url: Option<String>,
    pub servers: Option<Vec<String>>,
    pub rar_urls: Option<Vec<SourceObject>>,
    pub zip_urls: Option<Vec<SourceObject>>,
    #[serde(rename = "7zipUrls")]
    pub seven_zip_urls: Option<Vec<SourceObject>>,
    pub tgz_urls: Option<Vec<SourceObject>>,
    pub tar_urls: Option<Vec<SourceObject>>,
    pub external_url: Option<String>,
    pub name: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub sources: Option<Vec<String>>,
    #[serde(rename = "subtitles")]
    pub subtitles: Option<Vec<SubtitleTrack>>,
    pub subtitle_tracks: Option<Vec<SubtitleTrack>>,
    pub headers: Option<HashMap<String, String>>,
    pub behavior_hints: Option<StreamBehaviorHints>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceObject {
    pub url: String,
    pub bytes: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubtitleTrack {
    pub id: String,
    pub url: String,
    pub lang: String,
    pub label: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Link {
    pub name: String,
    pub category: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MetaBehaviorHints {
    pub default_video_id: Option<String>,
    pub featured_video_id: Option<String>,
    pub has_scheduled_videos: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamBehaviorHints {
    pub country_whitelist: Option<Vec<String>>,
    pub not_web_ready: Option<bool>,
    pub video_hash: Option<String>,
    pub video_size: Option<i64>,
    pub filename: Option<String>,
    pub binge_group: Option<String>,
    pub proxy_headers: Option<ProxyHeaders>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProxyHeaders {
    pub request: Option<HashMap<String, String>>,
    pub response: Option<HashMap<String, String>>,
}
