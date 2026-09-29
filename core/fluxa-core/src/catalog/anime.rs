use serde_json::{Value, json};

const HIGH_CONFIDENCE_THRESHOLD: i64 = 80;

const ANIME_PROVIDER_PATTERNS: &[&str] = &[
    "anime",
    "anilist",
    "ani-list",
    "myanimelist",
    "mal",
    "kitsu",
    "anidb",
    "jikan",
    "aniskip",
];

const ANIME_RELEASE_PATTERNS: &[&str] = &[
    "subsplease",
    "erai-raws",
    "horriblesubs",
    "commie",
    "judas",
    "ember",
    "anime time",
    "animepahe",
    "nyaa",
];

const ANIME_LINK_HOSTS: &[&str] = &["anilist.co", "myanimelist.net", "kitsu.io", "anidb.net"];

const JAPANESE_SIGNALS: &[&str] = &["japanese", "japan", "jpn", "dual audio", "japanese audio"];

fn normalize(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for c in value.chars().flat_map(char::to_lowercase) {
        let is_separator = c.is_whitespace()
            || matches!(
                c,
                '_' | '.' | '/' | ':' | '[' | ']' | '(' | ')' | '{' | '}' | '-'
            );
        if is_separator {
            if !out.is_empty() && !out.ends_with(' ') {
                out.push(' ');
            }
        } else {
            out.push(c);
        }
    }
    while out.ends_with(' ') {
        out.pop();
    }
    out
}

fn matches_any(value: &str, needles: &[&str]) -> bool {
    let normalized = normalize(value);
    needles
        .iter()
        .any(|needle| normalized.contains(&normalize(needle)))
}

fn push_text(parts: &mut Vec<String>, value: Option<&Value>) {
    if let Some(s) = value.and_then(Value::as_str)
        && !s.trim().is_empty()
    {
        parts.push(s.to_string());
    }
}

pub(crate) fn detect_anime_playback(
    meta: &Value,
    episode: &Value,
    stream: &Value,
    addons: &[Value],
) -> Value {
    let mut confidence: i64 = 0;
    let mut reasons: Vec<&str> = Vec::new();

    let mut text_fields: Vec<String> = Vec::new();
    for value in [
        meta.get("id"),
        meta.get("name"),
        episode.get("id"),
        episode.get("title"),
        episode.get("name"),
        stream.get("addonName"),
        stream.get("name"),
        stream.get("title"),
        stream.get("description"),
        stream.get("behaviorHints").and_then(|b| b.get("filename")),
    ] {
        push_text(&mut text_fields, value);
    }
    if let Some(sources) = stream.get("sources").and_then(Value::as_array) {
        for source in sources {
            push_text(&mut text_fields, Some(source));
        }
    }

    let link_text = meta
        .get("links")
        .and_then(Value::as_array)
        .map(|links| {
            links
                .iter()
                .map(|link| {
                    format!(
                        "{} {} {}",
                        link.get("name").and_then(Value::as_str).unwrap_or(""),
                        link.get("category").and_then(Value::as_str).unwrap_or(""),
                        link.get("url").and_then(Value::as_str).unwrap_or("")
                    )
                })
                .collect::<Vec<_>>()
                .join(" ")
        })
        .unwrap_or_default();
    if matches_any(&link_text, ANIME_LINK_HOSTS) {
        confidence += 100;
        reasons.push("anime external link");
    }

    let genres: Vec<String> = meta
        .get("genres")
        .and_then(Value::as_array)
        .map(|genres| {
            genres
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default();
    if genres.iter().any(|g| normalize(g) == "anime") {
        confidence += 65;
        reasons.push("anime genre");
    }

    let all_text = text_fields.join(" ");
    if matches_any(&all_text, ANIME_PROVIDER_PATTERNS) {
        confidence += 85;
        reasons.push("anime provider or source text");
    }
    if matches_any(&all_text, ANIME_RELEASE_PATTERNS) {
        confidence += 30;
        reasons.push("anime release group or filename");
    }

    let mut addon_parts: Vec<String> = Vec::new();
    for addon in addons {
        for value in [
            addon.get("id"),
            addon.get("name"),
            addon.get("manifest").and_then(|m| m.get("id")),
            addon.get("manifest").and_then(|m| m.get("name")),
            addon.get("transportUrl"),
        ] {
            push_text(&mut addon_parts, value);
        }
        if let Some(types) = addon.get("types").and_then(Value::as_array) {
            for t in types {
                push_text(&mut addon_parts, Some(t));
            }
        }
        if let Some(catalogs) = addon.get("catalogs").and_then(Value::as_array) {
            for catalog in catalogs {
                for value in [catalog.get("id"), catalog.get("name"), catalog.get("type")] {
                    push_text(&mut addon_parts, value);
                }
            }
        }
    }
    let addon_text = addon_parts.join(" ");
    let stream_addon_name = stream
        .get("addonName")
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty());
    if let Some(addon_name) = stream_addon_name
        && matches_any(&addon_text, ANIME_PROVIDER_PATTERNS)
        && matches_any(addon_name, ANIME_PROVIDER_PATTERNS)
    {
        confidence += 85;
        reasons.push("anime addon");
    }

    if genres.iter().any(|g| normalize(g) == "animation")
        && matches_any(&all_text, JAPANESE_SIGNALS)
    {
        confidence += 45;
        reasons.push("animation with Japanese signal");
    }

    let clamped = confidence.min(100);
    json!({
        "isAnime": clamped >= HIGH_CONFIDENCE_THRESHOLD,
        "confidence": clamped,
        "reasons": reasons,
    })
}

pub(crate) fn should_attempt_anime_tracking(meta: &Value) -> bool {
    if meta.get("type").and_then(Value::as_str) != Some("series") {
        return false;
    }

    let link_text = meta
        .get("links")
        .and_then(Value::as_array)
        .map(|links| {
            links
                .iter()
                .map(|link| {
                    format!(
                        "{} {} {}",
                        link.get("name").and_then(Value::as_str).unwrap_or(""),
                        link.get("category").and_then(Value::as_str).unwrap_or(""),
                        link.get("url").and_then(Value::as_str).unwrap_or("")
                    )
                })
                .collect::<Vec<_>>()
                .join(" ")
        })
        .unwrap_or_default();
    if matches_any(&link_text, ANIME_LINK_HOSTS) {
        return true;
    }

    let has_anime_genre = meta
        .get("genres")
        .and_then(Value::as_array)
        .is_some_and(|genres| {
            genres
                .iter()
                .filter_map(Value::as_str)
                .any(|g| normalize(g) == "anime")
        });
    if has_anime_genre {
        return true;
    }

    let mut text_fields = Vec::new();
    push_text(&mut text_fields, meta.get("id"));
    push_text(&mut text_fields, meta.get("name"));
    push_text(&mut text_fields, meta.get("description"));
    matches_any(&text_fields.join(" "), &["anime", "anilist"])
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    fn detect(meta: &str, stream: &str) -> Value {
        detect_anime_playback(
            &serde_json::from_str(meta).unwrap(),
            &Value::Null,
            &serde_json::from_str(stream).unwrap(),
            &[],
        )
    }

    #[test]
    fn tracking_gate_ignores_movies() {
        let meta: Value = serde_json::from_str(
            r#"{"id":"tt1","name":"Anime Movie","type":"movie","genres":["anime"]}"#,
        )
        .unwrap();
        assert!(!should_attempt_anime_tracking(&meta));
    }

    #[test]
    fn tracking_gate_accepts_anime_genre_series() {
        let meta: Value = serde_json::from_str(
            r#"{"id":"tt1","name":"Some Show","type":"series","genres":["Anime"]}"#,
        )
        .unwrap();
        assert!(should_attempt_anime_tracking(&meta));
    }

    #[test]
    fn tracking_gate_rejects_unrelated_series() {
        let meta: Value = serde_json::from_str(
            r#"{"id":"tt1","name":"Some Drama","type":"series","genres":["Drama"]}"#,
        )
        .unwrap();
        assert!(!should_attempt_anime_tracking(&meta));
    }

    #[test]
    fn release_group_alone_stays_below_threshold() {
        let result = detect(
            r#"{"id":"tt1","name":"Some Show","type":"series"}"#,
            r#"{"name":"[SubsPlease] Some Show - 01"}"#,
        );
        assert_eq!(result["isAnime"], Value::Bool(false));
        assert_eq!(result["confidence"], 30);
    }

    #[test]
    fn anilist_link_is_decisive() {
        let result = detect(
            r#"{"id":"tt1","name":"Show","links":[{"name":"AniList","category":"other","url":"https://anilist.co/anime/1"}]}"#,
            "null",
        );
        assert_eq!(result["isAnime"], Value::Bool(true));
        assert_eq!(result["confidence"], 100);
    }
}
