use serde_json::Value;

const ANIME_LINK_HOSTS: &[&str] = &["anilist.co", "myanimelist.net", "kitsu.io", "anidb.net"];

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
}
