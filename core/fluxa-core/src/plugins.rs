use crate::types::plugin::{PluginManifest, PluginStreamResult, PluginSubtitleResult};
use crate::types::resource::{Stream, SubtitleTrack};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

pub(crate) fn parse_plugin_manifest_json(payload: &str) -> Result<String, String> {
    let manifest: PluginManifest =
        serde_json::from_str(payload).map_err(|e| format!("invalid plugin manifest: {e}"))?;

    if manifest.name.trim().is_empty() {
        return Err("plugin manifest is missing a name".to_string());
    }
    if manifest.version.trim().is_empty() {
        return Err("plugin manifest is missing a version".to_string());
    }
    if manifest.scrapers.is_empty() {
        return Err("plugin manifest declares no providers".to_string());
    }

    serde_json::to_string(&manifest).map_err(|e| format!("failed to encode manifest: {e}"))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct PluginExecutionPlanRequest {
    scrapers: Vec<Value>,
    content_id: String,
    media_type: String,
    season: Option<i32>,
    episode: Option<i32>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct PluginExecutionPlan {
    content_id: String,
    media_type: String,
    season: Option<i32>,
    episode: Option<i32>,
    scrapers: Vec<Value>,
}

pub(crate) fn plugin_execution_plan_json(payload: &str) -> Option<String> {
    let request: PluginExecutionPlanRequest = serde_json::from_str(payload).ok()?;
    let segments: Vec<&str> = request.content_id.split(':').collect();
    let content_id = segments.first()?.trim().to_string();
    let media_type = match request.media_type.trim() {
        "series" | "show" => "tv".to_string(),
        value if !value.is_empty() => value.to_string(),
        _ => return None,
    };
    if content_id.is_empty() {
        return None;
    }
    let season = request.season.or_else(|| segments.get(1)?.parse().ok());
    let episode = request.episode.or_else(|| segments.get(2)?.parse().ok());
    let scrapers = request
        .scrapers
        .into_iter()
        .filter(|scraper| {
            scraper
                .get("enabled")
                .and_then(Value::as_bool)
                .unwrap_or(true)
                && scraper
                    .get("id")
                    .and_then(Value::as_str)
                    .is_some_and(|id| !id.trim().is_empty())
                && scraper
                    .get("filename")
                    .and_then(Value::as_str)
                    .is_some_and(|filename| !filename.trim().is_empty())
                && scraper
                    .get("supportedTypes")
                    .and_then(Value::as_array)
                    .map(|types| {
                        types
                            .iter()
                            .filter_map(Value::as_str)
                            .any(|kind| kind == media_type)
                    })
                    .unwrap_or(true)
        })
        .collect();
    serde_json::to_string(&PluginExecutionPlan {
        content_id,
        media_type,
        season,
        episode,
        scrapers,
    })
    .ok()
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawPluginStreamResult {
    title: Option<String>,
    name: Option<String>,
    url: Option<Value>,
    quality: Option<String>,
    size: Option<String>,
    language: Option<String>,
    provider: Option<String>,
    #[serde(rename = "type")]
    type_: Option<String>,
    seeders: Option<i64>,
    peers: Option<i64>,
    info_hash: Option<String>,
    headers: Option<HashMap<String, String>>,
    subtitles: Option<Vec<RawPluginSubtitleResult>>,
    unavailable_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawPluginSubtitleResult {
    url: Option<String>,
    language: Option<String>,
    name: Option<String>,
    headers: Option<HashMap<String, String>>,
}

fn raw_url(value: &Value) -> Option<String> {
    match value {
        Value::String(s) => Some(s.clone()),
        Value::Object(map) => map.get("url").and_then(Value::as_str).map(str::to_string),
        _ => None,
    }
}

fn non_blank(value: Option<String>) -> Option<String> {
    value.filter(|s| !s.trim().is_empty())
}

/// A plugin's `getStreams()` returns an untrusted, loosely-shaped JS array
/// (url can be a string or `{url}`, title commonly falls back to name).
/// This normalizes it into [`PluginStreamResult`], dropping entries without
/// a usable url — the same tolerance Nuvio's `parseJsonResults` applies.
pub(crate) fn parse_plugin_stream_results_json(raw_json: &str) -> String {
    let raw: Vec<RawPluginStreamResult> = match serde_json::from_str(raw_json) {
        Ok(items) => items,
        Err(_) => return "[]".to_string(),
    };

    let results: Vec<PluginStreamResult> = raw
        .into_iter()
        .filter_map(|item| {
            let url = non_blank(item.url.as_ref().and_then(raw_url))?;
            let title = non_blank(item.title)
                .or_else(|| item.name.clone())
                .unwrap_or_else(|| "Unknown".to_string());
            let headers = item.headers.filter(|h| !h.is_empty());
            let subtitles = item.subtitles.map(|subs| {
                subs.into_iter()
                    .filter_map(|sub| {
                        Some(PluginSubtitleResult {
                            url: non_blank(sub.url)?,
                            language: sub.language.unwrap_or_else(|| "Unknown".to_string()),
                            name: sub.name,
                            headers: sub.headers.filter(|h| !h.is_empty()),
                        })
                    })
                    .collect::<Vec<_>>()
            });

            Some(PluginStreamResult {
                title,
                name: item.name,
                url,
                quality: item.quality,
                size: item.size,
                language: item.language,
                provider: item.provider,
                type_: item.type_,
                seeders: item.seeders,
                peers: item.peers,
                info_hash: item.info_hash,
                headers,
                subtitles: subtitles.filter(|s| !s.is_empty()),
            })
        })
        .collect();

    serde_json::to_string(&results).unwrap_or_else(|_| "[]".to_string())
}

/// Maps a plugin's normalized stream results onto the same [`Stream`] shape
/// addon resources already produce, so the platform can hand plugin-sourced
/// streams straight to the existing `detailStreamsAppended` merge/ranking
/// path instead of needing a parallel one. Quality/size/provider/seeders/
/// peers don't have first-class `Stream` fields, so they ride along in
/// `extra` rather than being dropped.
pub(crate) fn plugin_stream_results_to_streams_json(raw_json: &str) -> String {
    let normalized = parse_plugin_stream_results_json(raw_json);
    let results: Vec<PluginStreamResult> = match serde_json::from_str(&normalized) {
        Ok(items) => items,
        Err(_) => return "[]".to_string(),
    };

    let streams: Vec<Stream> = if results.is_empty() {
        vec![plugin_unavailable_stream(plugin_unavailable_reason(
            raw_json,
        ))]
    } else {
        results.into_iter().map(plugin_result_to_stream).collect()
    };
    serde_json::to_string(&streams).unwrap_or_else(|_| "[]".to_string())
}

fn plugin_unavailable_reason(raw_json: &str) -> Option<String> {
    serde_json::from_str::<Vec<RawPluginStreamResult>>(raw_json)
        .ok()?
        .into_iter()
        .find_map(|result| non_blank(result.unavailable_reason))
}

fn plugin_unavailable_stream(reason: Option<String>) -> Stream {
    let mut extra = serde_json::Map::new();
    extra.insert("pluginUnavailable".to_string(), Value::Bool(true));
    extra.insert(
        "pluginUnavailableReason".to_string(),
        Value::String(reason.unwrap_or_else(|| "no_playable_stream".to_string())),
    );
    Stream {
        extra,
        ..Default::default()
    }
}

fn plugin_result_to_stream(result: PluginStreamResult) -> Stream {
    let mut extra = serde_json::Map::new();
    if let Some(quality) = result.quality {
        extra.insert("quality".to_string(), Value::String(quality));
    }
    if let Some(size) = result.size {
        extra.insert("size".to_string(), Value::String(size));
    }
    if let Some(provider) = &result.provider {
        extra.insert("provider".to_string(), Value::String(provider.clone()));
    }
    if let Some(seeders) = result.seeders {
        extra.insert("seeders".to_string(), Value::from(seeders));
    }
    if let Some(peers) = result.peers {
        extra.insert("peers".to_string(), Value::from(peers));
    }

    let subtitle_tracks = result.subtitles.map(|subs| {
        subs.into_iter()
            .enumerate()
            .map(|(index, sub)| SubtitleTrack {
                id: format!("plugin-sub-{index}"),
                url: sub.url,
                lang: sub.language,
                label: sub.name,
            })
            .collect()
    });

    Stream {
        url: Some(result.url),
        name: result.name.or(result.provider),
        title: Some(result.title),
        info_hash: result.info_hash,
        headers: result.headers,
        subtitles: subtitle_tracks.clone(),
        subtitle_tracks,
        extra,
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_requires_name_version_and_scrapers() {
        assert!(
            parse_plugin_manifest_json(r#"{"name":"","version":"1.0","scrapers":[]}"#).is_err()
        );
        assert!(
            parse_plugin_manifest_json(r#"{"name":"Repo","version":"","scrapers":[]}"#).is_err()
        );
        assert!(
            parse_plugin_manifest_json(r#"{"name":"Repo","version":"1.0","scrapers":[]}"#).is_err()
        );
    }

    #[test]
    fn manifest_accepts_a_well_formed_payload() {
        let payload = r#"{
            "name": "Phisher's Repo",
            "version": "1.0.0",
            "scrapers": [
                {"id":"MoviesDrive","name":"MoviesDrive","version":"1.1.1","filename":"src/providers/moviesdrive.js"}
            ]
        }"#;
        let result = parse_plugin_manifest_json(payload);
        assert!(result.is_ok());
        let manifest: PluginManifest = serde_json::from_str(&result.unwrap()).unwrap();
        assert_eq!(manifest.scrapers[0].supported_types, vec!["movie", "tv"]);
        assert!(manifest.scrapers[0].enabled);
    }

    #[test]
    fn execution_plan_filters_disabled_scrapers_and_parses_episode_locator() {
        let plan = plugin_execution_plan_json(
            r#"{"contentId":"tt0944947:2:3","mediaType":"series","scrapers":[{"id":"enabled","filename":"enabled.js","enabled":true},{"id":"disabled","filename":"disabled.js","enabled":false}]}"#,
        )
        .unwrap();
        let value: Value = serde_json::from_str(&plan).unwrap();
        assert_eq!(value["contentId"], "tt0944947");
        assert_eq!(value["season"], 2);
        assert_eq!(value["episode"], 3);
        assert_eq!(value["scrapers"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn stream_results_accept_string_and_object_url_shapes() {
        let raw = r#"[
            {"title":"1080p","url":"https://example.com/a.mp4"},
            {"name":"720p","url":{"url":"https://example.com/b.mp4"}}
        ]"#;
        let parsed = parse_plugin_stream_results_json(raw);
        let results: Vec<PluginStreamResult> = serde_json::from_str(&parsed).unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].title, "1080p");
        assert_eq!(results[1].title, "720p");
        assert_eq!(results[1].url, "https://example.com/b.mp4");
    }

    #[test]
    fn stream_results_drop_entries_without_a_usable_url() {
        let raw = r#"[{"title":"no url"},{"title":"blank","url":""}]"#;
        assert_eq!(parse_plugin_stream_results_json(raw), "[]");
    }

    #[test]
    fn empty_stream_results_produce_an_unavailable_stream_marker() {
        let parsed = plugin_stream_results_to_streams_json("[]");
        let streams: Vec<Stream> = serde_json::from_str(&parsed).unwrap();
        assert_eq!(streams.len(), 1);
        assert!(streams[0].url.is_none());
        assert_eq!(streams[0].extra["pluginUnavailable"], true);
        assert_eq!(
            streams[0].extra["pluginUnavailableReason"],
            "no_playable_stream"
        );
    }

    #[test]
    fn unavailable_reason_is_preserved_on_the_stream_marker() {
        let parsed = plugin_stream_results_to_streams_json(
            r#"[{"unavailableReason":"Content was removed."}]"#,
        );
        let streams: Vec<Stream> = serde_json::from_str(&parsed).unwrap();
        assert_eq!(
            streams[0].extra["pluginUnavailableReason"],
            "Content was removed."
        );
    }

    #[test]
    fn stream_results_map_onto_the_addon_stream_shape() {
        let raw = r#"[{
            "title": "1080p",
            "url": "https://example.com/a.mp4",
            "quality": "1080p",
            "provider": "MoviesDrive",
            "seeders": 12,
            "headers": {"Referer": "https://example.com"},
            "subtitles": [{"url": "https://example.com/sub.srt", "language": "en", "name": "English"}]
        }]"#;
        let parsed = plugin_stream_results_to_streams_json(raw);
        let streams: Vec<Stream> = serde_json::from_str(&parsed).unwrap();
        assert_eq!(streams.len(), 1);
        let stream = &streams[0];
        assert_eq!(stream.url.as_deref(), Some("https://example.com/a.mp4"));
        assert_eq!(stream.name.as_deref(), Some("MoviesDrive"));
        assert_eq!(stream.title.as_deref(), Some("1080p"));
        assert_eq!(stream.extra["quality"], "1080p");
        assert_eq!(stream.extra["seeders"], 12);
        assert_eq!(
            stream
                .headers
                .as_ref()
                .unwrap()
                .get("Referer")
                .map(String::as_str),
            Some("https://example.com")
        );
        let subs = stream.subtitle_tracks.as_ref().unwrap();
        assert_eq!(subs[0].lang, "en");
        assert_eq!(subs[0].url, "https://example.com/sub.srt");
        let subs_alias = stream.subtitles.as_ref().unwrap();
        assert_eq!(subs_alias[0].url, "https://example.com/sub.srt");
    }

    #[test]
    fn stream_results_tolerate_malformed_input() {
        assert_eq!(parse_plugin_stream_results_json("not json"), "[]");
        assert_eq!(parse_plugin_stream_results_json("{}"), "[]");
    }
}
