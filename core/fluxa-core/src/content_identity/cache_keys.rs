use super::episode_matching::text_matches_episode;
use super::id::parse_episode_locator;
use super::text::form_decode;
use serde_json::{Map, Value};

pub(crate) fn episode_filename_candidate(stream_json: &str, video_id: &str) -> Option<String> {
    let (_, season, episode) = parse_episode_locator(video_id)?;
    let stream = serde_json::from_str::<Value>(stream_json).ok()?;
    for value in ["title", "description", "name"] {
        if let Some(text) = stream.get(value).and_then(Value::as_str) {
            for line in text.lines().map(str::trim) {
                if is_likely_video_file(line) && text_matches_episode(line, season, episode) {
                    return Some(line.to_string());
                }
            }
        }
    }
    None
}

pub(crate) fn is_likely_video_file(path: &str) -> bool {
    let path = path.to_ascii_lowercase();
    [".mkv", ".mp4", ".avi", ".webm", ".m4v", ".mov", ".ts"]
        .iter()
        .any(|extension| path.ends_with(extension))
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct StreamDiscoveryCacheKeyRequest {
    #[serde(rename = "type")]
    content_type: String,
    id: String,
    language: String,
    cs3_search_query: Option<String>,
    cs3_year: Option<i64>,
    cs3_original_name: Option<String>,
    addon_signatures: Vec<String>,
    cs3_plugin_names: Vec<String>,
}

pub(crate) fn java_string_hash(value: &str) -> i32 {
    let mut hash = 0i32;
    for unit in value.encode_utf16() {
        hash = hash.wrapping_mul(31).wrapping_add(unit as i32);
    }
    hash
}

pub(crate) fn stream_discovery_cache_key(request_json: &str) -> Option<String> {
    let mut request = serde_json::from_str::<StreamDiscoveryCacheKeyRequest>(request_json).ok()?;
    let mut addon_signatures = request.addon_signatures;
    addon_signatures.sort();
    let mut cs3_plugin_names = request.cs3_plugin_names;
    cs3_plugin_names.sort();
    let search_query = request.cs3_search_query.unwrap_or_default();
    let original_name_hash = request
        .cs3_original_name
        .take()
        .filter(|value| value != &search_query)
        .map(|value| java_string_hash(&value).to_string())
        .unwrap_or_default();
    Some(
        [
            request.content_type,
            request.id,
            request.language,
            search_query,
            request
                .cs3_year
                .map(|value| value.to_string())
                .unwrap_or_default(),
            original_name_hash,
            addon_signatures.join("|"),
            cs3_plugin_names.join("|"),
        ]
        .join("|"),
    )
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DiscoverCatalogCacheKeyRequest {
    #[serde(rename = "type")]
    content_type: String,
    catalog_key: Option<String>,
    genre: Option<String>,
    year: Option<String>,
    rating: Option<f32>,
    provider: Option<String>,
    region: Option<String>,
    catalog_signatures: Vec<String>,
}

pub(crate) fn discover_catalog_cache_key(request_json: &str) -> Option<String> {
    let request = serde_json::from_str::<DiscoverCatalogCacheKeyRequest>(request_json).ok()?;
    Some(
        [
            request.content_type,
            request.catalog_key.unwrap_or_default(),
            request.genre.unwrap_or_default(),
            request.year.unwrap_or_default(),
            request
                .rating
                .map(|value| value.to_string())
                .unwrap_or_default(),
            request.provider.unwrap_or_default(),
            request.region.unwrap_or_default(),
            request.catalog_signatures.join(","),
        ]
        .join("|"),
    )
}

pub(crate) fn parse_extra_args_json(extra: &str) -> Option<String> {
    let mut map = Map::new();
    for part in extra.split('&') {
        let key = part.split_once('=').map(|(key, _)| key).unwrap_or(part);
        if key.is_empty() {
            continue;
        }
        let value = part.split_once('=').map(|(_, value)| value).unwrap_or("");
        map.insert(form_decode(key), Value::String(form_decode(value)));
    }
    serde_json::to_string(&Value::Object(map)).ok()
}
