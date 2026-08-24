use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::HashMap;

fn string_or_vec<'de, D>(deserializer: D) -> Result<Option<Vec<String>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrVec {
        Text(String),
        List(Vec<String>),
    }

    Ok(match Option::<StringOrVec>::deserialize(deserializer)? {
        None => None,
        Some(StringOrVec::List(items)) => Some(items),
        Some(StringOrVec::Text(text)) => Some(
            text.split(',')
                .map(str::trim)
                .filter(|part| !part.is_empty())
                .map(str::to_string)
                .collect(),
        ),
    })
}

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
    #[serde(deserialize_with = "string_or_vec")]
    pub genres: Option<Vec<String>>,
    #[serde(deserialize_with = "string_or_vec")]
    pub director: Option<Vec<String>>,
    #[serde(deserialize_with = "string_or_vec")]
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
    pub audio_tracks: Option<Vec<AudioTrack>>,
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
pub struct AudioTrack {
    pub id: String,
    pub url: String,
    pub lang: String,
    pub label: Option<String>,
    pub headers: Option<HashMap<String, String>>,
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

#[cfg(test)]
mod meta_item_tests {
    use super::MetaItem;
    use serde_json::json;

    fn director_of(value: serde_json::Value) -> Option<Vec<String>> {
        serde_json::from_value::<MetaItem>(json!({
            "id": "tt11198330",
            "type": "series",
            "name": "House of the Dragon",
            "director": value,
        }))
        .expect("meta with a string director must still decode")
        .director
    }

    #[test]
    fn empty_string_director_decodes_instead_of_rejecting_the_meta() {
        assert_eq!(director_of(json!("")), Some(vec![]));
    }

    #[test]
    fn comma_joined_director_splits_into_entries() {
        assert_eq!(
            director_of(json!("Ryan Condal, Miguel Sapochnik")),
            Some(vec!["Ryan Condal".to_string(), "Miguel Sapochnik".to_string()]),
        );
    }

    #[test]
    fn list_director_is_preserved() {
        assert_eq!(director_of(json!(["Ryan Condal"])), Some(vec!["Ryan Condal".to_string()]));
    }
}
