use crate::player::subtitle_sync::parse_subtitle_cues_with_text;
use regex::Regex;
use serde_json::{Value, json};
use std::sync::OnceLock;
use url::Url;

pub(crate) fn trailer_youtube_video_ids_json(input: &str) -> Option<String> {
    let value: Value = serde_json::from_str(input).ok()?;
    let urls = value.get("urls")?.as_array()?;
    let mut ids = Vec::new();
    for url in urls.iter().filter_map(Value::as_str) {
        if let Some(id) = youtube_video_id(url)
            && !ids.contains(&id)
        {
            ids.push(id);
        }
    }
    serde_json::to_string(&ids).ok()
}

pub(crate) fn trailer_direct_selection_json(input: &str) -> Option<String> {
    let value: Value = serde_json::from_str(input).ok()?;
    let trailers = value.get("trailers")?.as_array()?;
    let max_height = value
        .get("maxHeight")
        .and_then(Value::as_i64)
        .filter(|height| *height > 0)
        .unwrap_or(i64::MAX);
    let mut best_declared: Option<(String, &'static str, i64)> = None;
    let mut first_unknown: Option<(String, &'static str)> = None;

    for trailer in trailers {
        let Some(url) = trailer.get("url").and_then(Value::as_str) else {
            continue;
        };
        let Some(mime_type) = direct_trailer_mime_type(url) else {
            continue;
        };
        let quality = direct_trailer_quality(trailer, url);
        if quality <= 0 {
            if first_unknown.is_none() {
                first_unknown = Some((url.to_string(), mime_type));
            }
            continue;
        }
        if quality > max_height {
            continue;
        }
        if best_declared
            .as_ref()
            .is_none_or(|(_, _, best_quality)| quality > *best_quality)
        {
            best_declared = Some((url.to_string(), mime_type, quality));
        }
    }

    let selected = best_declared
        .map(|(url, mime_type, _)| (url, mime_type))
        .or(first_unknown);
    serde_json::to_string(
        &selected.map(|(url, mime_type)| json!({ "url": url, "mimeType": mime_type })),
    )
    .ok()
}

fn youtube_video_id(raw: &str) -> Option<String> {
    let parsed = Url::parse(raw.trim()).ok()?;
    let host = parsed
        .host_str()?
        .trim_end_matches('.')
        .to_ascii_lowercase();
    let candidate = if host == "youtu.be" || host.ends_with(".youtu.be") {
        parsed
            .path_segments()?
            .find(|segment| !segment.is_empty())?
            .to_string()
    } else if host == "youtube.com" || host.ends_with(".youtube.com") {
        parsed
            .query_pairs()
            .find_map(|(key, value)| (key == "v").then_some(value.into_owned()))
            .or_else(|| {
                parsed
                    .path_segments()?
                    .filter(|segment| !segment.is_empty())
                    .next_back()
                    .map(str::to_string)
            })?
    } else {
        return None;
    };
    is_youtube_video_id(&candidate).then_some(candidate)
}

fn is_youtube_video_id(value: &str) -> bool {
    value.len() == 11
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
}

fn direct_trailer_mime_type(raw: &str) -> Option<&'static str> {
    let parsed = Url::parse(raw.trim()).ok()?;
    if !matches!(parsed.scheme(), "http" | "https") {
        return None;
    }
    match parsed
        .path()
        .rsplit('.')
        .next()?
        .to_ascii_lowercase()
        .as_str()
    {
        "m3u8" => Some("application/x-mpegURL"),
        "mpd" => Some("application/dash+xml"),
        "webm" => Some("video/webm"),
        "mp4" | "m4v" => Some("video/mp4"),
        "mkv" => Some("video/x-matroska"),
        "mov" => Some("video/quicktime"),
        "ogv" => Some("video/ogg"),
        _ => None,
    }
}

fn direct_trailer_quality(trailer: &Value, url: &str) -> i64 {
    let description = format!(
        "{} {} {url}",
        trailer.get("title").and_then(Value::as_str).unwrap_or(""),
        trailer.get("source").and_then(Value::as_str).unwrap_or(""),
    );
    if ultra_high_definition_pattern().is_match(&description) {
        return 2160;
    }
    resolution_pattern()
        .captures(&description)
        .and_then(|captures| captures.get(1))
        .and_then(|capture| capture.as_str().parse().ok())
        .unwrap_or_default()
}

fn ultra_high_definition_pattern() -> &'static Regex {
    static PATTERN: OnceLock<Regex> = OnceLock::new();
    PATTERN.get_or_init(|| Regex::new(r"(?i)\b(?:4k|uhd)\b").expect("valid UHD regex"))
}

fn resolution_pattern() -> &'static Regex {
    static PATTERN: OnceLock<Regex> = OnceLock::new();
    PATTERN.get_or_init(|| {
        Regex::new(r"(?i)\b(2160|1440|1080|720|576|540|480|360)\s*p\b")
            .expect("valid resolution regex")
    })
}

pub(crate) fn trailer_playback_policy_json(input: &str) -> Option<String> {
    let value: Value = serde_json::from_str(input).unwrap_or_default();
    let is_autoplay = value
        .get("autoplay")
        .and_then(Value::as_bool)
        .unwrap_or(true);
    Some(
        json!({
            "stallTimeoutMs": if is_autoplay { 7_000 } else { 10_000 },
            "maxRetries": 1,
            "retryDelayMs": 250,
        })
        .to_string(),
    )
}

fn language(value: Option<&str>) -> Option<String> {
    let value = value?.trim().to_lowercase();
    if value.is_empty() || value == "none" {
        return None;
    }
    value.split(['-', '_']).next().map(str::to_string)
}

pub(crate) fn trailer_subtitle_selection_plan_json(input: &str) -> Option<String> {
    let value: Value = serde_json::from_str(input).ok()?;
    let tracks = value.get("tracks")?.as_array()?;
    if tracks.is_empty() {
        return Some("null".to_string());
    }
    let mut real_wanted = Vec::new();
    for candidate in ["preferred", "secondary", "systemLanguage"] {
        if let Some(language) = language(value.get(candidate).and_then(Value::as_str))
            && !real_wanted.contains(&language)
        {
            real_wanted.push(language);
        }
    }
    let mut wanted = real_wanted.clone();
    if !wanted.iter().any(|value| value == "en") {
        wanted.push("en".to_string());
    }
    let selected = tracks.iter().enumerate().max_by_key(|(index, track)| {
        let track_language = language(track.get("languageTag").and_then(Value::as_str));
        let wanted_index =
            track_language.and_then(|language| wanted.iter().position(|value| value == &language));
        let preferred = wanted_index
            .map(|position| 1000_i64 - position as i64 * 100)
            .unwrap_or_default();
        let english_label = (wanted_index.is_none()
            && track
                .get("label")
                .and_then(Value::as_str)
                .is_some_and(|label| label.to_lowercase().contains("english")))
            as i64
            * 250;
        let human = (!track
            .get("isAuto")
            .and_then(Value::as_bool)
            .unwrap_or(false)) as i64
            * 25;
        (preferred + english_label + human, std::cmp::Reverse(*index))
    })?;
    let (_, best_track) = selected;

    let best_language = language(best_track.get("languageTag").and_then(Value::as_str));
    let translation_target = real_wanted
        .into_iter()
        .find(|language| Some(language.clone()) != best_language);

    if let Some(target) = translation_target {
        let base_url = best_track.get("url").and_then(Value::as_str)?;
        let separator = if base_url.contains('?') { "&" } else { "?" };
        let source_label = best_track
            .get("label")
            .and_then(Value::as_str)
            .unwrap_or("");
        return serde_json::to_string(&json!({
            "languageTag": target,
            "label": format!("{target} (auto-translated from {source_label})"),
            "url": format!("{base_url}{separator}tlang={target}"),
            "mimeType": "text/vtt",
            "isAuto": true,
        }))
        .ok();
    }

    Some(best_track.to_string())
}

pub(crate) fn normalize_trailer_subtitle_url_json(input: &str) -> Option<String> {
    let value: Value = serde_json::from_str(input).ok()?;
    let raw = value.get("url")?.as_str()?;
    let normalized = set_query_parameter(raw, "fmt", "vtt");
    serde_json::to_string(&normalized).ok()
}

fn set_query_parameter(raw: &str, name: &str, value: &str) -> String {
    if !(raw.starts_with("http://") || raw.starts_with("https://")) {
        return raw.to_string();
    }
    let (without_fragment, fragment) = raw
        .split_once('#')
        .map_or((raw, None), |(base, fragment)| (base, Some(fragment)));
    let (base, query) = without_fragment
        .split_once('?')
        .map_or((without_fragment, ""), |(base, query)| (base, query));
    let mut pairs = query
        .split('&')
        .filter(|pair| !pair.is_empty() && pair.split('=').next() != Some(name))
        .map(str::to_string)
        .collect::<Vec<_>>();
    pairs.push(format!("{name}={value}"));
    let mut output = format!("{base}?{}", pairs.join("&"));
    if let Some(fragment) = fragment {
        output.push('#');
        output.push_str(fragment);
    }
    output
}

pub(crate) fn parse_trailer_subtitle_cues_json(input: &str) -> Option<String> {
    let value: Value = serde_json::from_str(input).ok()?;
    let body = value.get("body")?.as_str()?;
    let cues = parse_subtitle_cues_with_text(body)
        .into_iter()
        .map(|cue| json!({ "start": cue.start, "end": cue.end, "text": cue.text }))
        .collect::<Vec<_>>();
    serde_json::to_string(&cues).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_youtube_ids_in_order_without_duplicates() {
        let result = trailer_youtube_video_ids_json(
            r#"{"urls":["https://youtu.be/abcdefghijk","https://youtube.com/watch?v=abcdefghijk","https://www.youtube.com/embed/zyxwvutsrqp","https://example.com/watch?v=abcdefghijk"]}"#,
        )
        .unwrap();
        assert_eq!(
            serde_json::from_str::<Value>(&result).unwrap(),
            json!(["abcdefghijk", "zyxwvutsrqp"])
        );
    }

    #[test]
    fn selects_best_direct_trailer_with_quality_cap_and_mime_type() {
        let result = trailer_direct_selection_json(
            r#"{"maxHeight":1080,"trailers":[{"title":"IMDb 4K","url":"https://video.example/imdb.m3u8"},{"title":"Trailerio 1080p","url":"https://video.example/trailerio-1080.mp4"},{"title":"Trailerio 1080p","url":"https://video.example/second-1080.mp4"}]}"#,
        )
        .unwrap();
        let selected: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(selected["url"], "https://video.example/trailerio-1080.mp4");
        assert_eq!(selected["mimeType"], "video/mp4");
    }

    #[test]
    fn selects_preferred_human_subtitle() {
        let result = trailer_subtitle_selection_plan_json(r#"{"tracks":[{"languageTag":"en","label":"English","isAuto":true},{"languageTag":"tr-TR","label":"Türkçe","isAuto":false}],"preferred":"tr"}"#).unwrap();
        assert_eq!(
            serde_json::from_str::<Value>(&result).unwrap()["languageTag"],
            "tr-TR"
        );
    }

    #[test]
    fn synthesizes_youtube_auto_translation_when_no_native_track_matches() {
        let result = trailer_subtitle_selection_plan_json(
            r#"{"tracks":[{"languageTag":"ja","label":"日本語","url":"https://youtube.com/timedtext?lang=ja","isAuto":false}],"preferred":"tr"}"#,
        )
        .unwrap();
        let value: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(value["languageTag"], "tr");
        assert_eq!(value["isAuto"], true);
        assert_eq!(
            value["url"],
            "https://youtube.com/timedtext?lang=ja&tlang=tr"
        );
    }

    #[test]
    fn falls_back_to_system_language_translation_when_no_explicit_preference() {
        let result = trailer_subtitle_selection_plan_json(
            r#"{"tracks":[{"languageTag":"en","label":"English","url":"https://youtube.com/timedtext?lang=en","isAuto":false}],"systemLanguage":"tr-TR"}"#,
        )
        .unwrap();
        let value: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(value["languageTag"], "tr");
        assert_eq!(
            value["url"],
            "https://youtube.com/timedtext?lang=en&tlang=tr"
        );
    }

    #[test]
    fn parses_vtt_and_timed_text() {
        let vtt = parse_trailer_subtitle_cues_json(
            r#"{"body":"WEBVTT\n\n00:00:01.000 --> 00:00:02.500\nHello &amp; world"}"#,
        )
        .unwrap();
        assert_eq!(
            serde_json::from_str::<Value>(&vtt).unwrap()[0]["text"],
            "Hello & world"
        );
        let xml = parse_trailer_subtitle_cues_json(r#"{"body":"<?xml version=\"1.0\"?><timedtext><p t=\"1000\" d=\"500\">Hi</p></timedtext>"}"#).unwrap();
        assert_eq!(serde_json::from_str::<Value>(&xml).unwrap()[0]["end"], 1.5);
    }
}
