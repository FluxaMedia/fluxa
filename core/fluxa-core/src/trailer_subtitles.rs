use crate::subtitle_sync::parse_subtitle_cues_with_text;
use serde_json::{Value, json};

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
