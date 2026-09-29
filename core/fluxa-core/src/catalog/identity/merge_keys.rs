use super::helpers::{TMDB_ID_PREFIX, imdb_regex, meta_text, push_unique, year_regex};
use super::id::{base_content_id, imdb_id};
use super::text::collapse_whitespace;
use serde_json::Value;

pub(crate) fn normalized_billboard_title(value: &str) -> String {
    collapse_whitespace(
        &value
            .to_lowercase()
            .replace('ç', "c")
            .replace('ğ', "g")
            .replace('ı', "i")
            .replace('ö', "o")
            .replace('ş', "s")
            .replace('ü', "u")
            .chars()
            .map(|ch| {
                if ch.is_ascii_alphanumeric() || ch == ' ' {
                    ch
                } else {
                    ' '
                }
            })
            .collect::<String>(),
    )
}

pub(crate) fn content_trakt_key_value(meta: &Value) -> String {
    trakt_identity_key(meta)
}

pub(crate) fn content_merge_keys_value(meta: &Value) -> Vec<String> {
    let mut keys = Vec::new();
    let id = meta_text(meta, "id");
    push_unique(&mut keys, content_trakt_key_value(meta));
    push_unique(&mut keys, id.to_string());
    push_unique(&mut keys, base_content_id(id));
    if let Some(imdb) = imdb_id(id) {
        push_unique(&mut keys, imdb);
    }
    if let Some(key) = title_year_key(meta) {
        push_unique(&mut keys, key);
    }
    keys
}

pub(crate) fn content_watched_keys_value(meta: &Value) -> Vec<String> {
    let mut keys = Vec::new();
    push_unique(&mut keys, content_trakt_key_value(meta));
    push_unique(&mut keys, meta_text(meta, "id").to_string());
    if let Some(key) = title_year_key(meta) {
        push_unique(&mut keys, key);
    }
    keys
}

pub(crate) fn content_trakt_keys_batch(metas_json: &str) -> Option<String> {
    let metas: Vec<Value> = serde_json::from_str(metas_json).ok()?;
    let keys: Vec<String> = metas.iter().map(content_trakt_key_value).collect();
    serde_json::to_string(&keys).ok()
}

pub(crate) fn content_watched_keys_batch(metas_json: &str) -> Option<String> {
    let metas: Vec<Value> = serde_json::from_str(metas_json).ok()?;
    let keys: Vec<Vec<String>> = metas.iter().map(content_watched_keys_value).collect();
    serde_json::to_string(&keys).ok()
}

pub(crate) fn content_keys_json(meta_json: &str, watched: bool) -> Option<String> {
    let meta = serde_json::from_str::<Value>(meta_json).ok()?;
    let keys = if watched {
        content_watched_keys_value(&meta)
    } else {
        content_merge_keys_value(&meta)
    };
    serde_json::to_string(&keys).ok()
}
pub(crate) fn normalized_loose_title(value: &str) -> String {
    value
        .to_lowercase()
        .chars()
        .map(|ch| {
            if ch.is_ascii_lowercase() || ch.is_ascii_digit() {
                ch
            } else {
                ' '
            }
        })
        .collect::<String>()
        .trim()
        .to_string()
}

pub(crate) fn title_year_key(meta: &Value) -> Option<String> {
    let title = normalized_loose_title(meta_text(meta, "name"));
    if title.is_empty() {
        return None;
    }
    let year = year_regex()
        .find(meta_text(meta, "releaseInfo"))
        .map(|matched| matched.as_str())
        .unwrap_or("");
    if year.is_empty() {
        return None;
    }
    Some(format!("{}:{title}:{year}", meta_text(meta, "type")))
}

pub(crate) fn trakt_identity_key(meta: &Value) -> String {
    let id = meta_text(meta, "id");
    if let Some(value) = imdb_regex().find(id).map(|matched| matched.as_str()) {
        return value.to_string();
    }
    let tmdb = if id.to_ascii_lowercase().starts_with(TMDB_ID_PREFIX) {
        id.strip_prefix(TMDB_ID_PREFIX).unwrap_or(id)
    } else {
        ""
    };
    if !tmdb.is_empty() {
        return format!("tmdb:{tmdb}");
    }
    format!(
        "{}:{}:{}",
        meta_text(meta, "type"),
        normalized_loose_title(meta_text(meta, "name")),
        meta_text(meta, "releaseInfo")
    )
}
