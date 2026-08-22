use serde_json::{Value, json};
use std::sync::OnceLock;

fn cleaned_url(raw: Option<&str>) -> Option<String> {
    raw.map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
}

fn github_blob_url_regex() -> &'static regex::Regex {
    static REGEX: OnceLock<regex::Regex> = OnceLock::new();
    REGEX.get_or_init(|| {
        #[expect(
            clippy::expect_used,
            reason = "static literal regex is reviewed at build time"
        )]
        regex::Regex::new(r"^https://github\.com/([^/]+)/([^/]+)/blob/([^/]+)/(.+)$")
            .expect("valid github blob url regex")
    })
}

fn cleaned_artwork_url(raw: Option<&str>) -> Option<String> {
    let s = raw?.trim().trim_matches('\'').trim_matches('"').trim();
    if s.is_empty() {
        return None;
    }
    let with_scheme = if s.starts_with("//") {
        format!("https:{s}")
    } else {
        s.to_string()
    };
    let normalized = if let Some(caps) = github_blob_url_regex().captures(&with_scheme) {
        format!(
            "https://raw.githubusercontent.com/{}/{}/{}/{}",
            caps.get(1)?.as_str(),
            caps.get(2)?.as_str(),
            caps.get(3)?.as_str(),
            caps.get(4)?.as_str()
        )
    } else {
        with_scheme
    };
    Some(normalized.replace(' ', "%20"))
}

fn pick_str<'a>(obj: &'a serde_json::Map<String, Value>, keys: &[&str]) -> Option<&'a str> {
    for k in keys {
        if let Some(Value::String(s)) = obj.get(*k) {
            return Some(s.as_str());
        }
    }
    None
}

fn normalize_shape(value: Option<&str>) -> &'static str {
    match value.map(|s| s.trim().to_uppercase()).as_deref() {
        Some("LANDSCAPE") | Some("WIDE") => "wide",
        Some("SQUARE") => "square",
        _ => "poster",
    }
}

fn export_shape(value: Option<&str>) -> &'static str {
    match value.map(str::to_lowercase).as_deref() {
        Some("wide") | Some("landscape") => "LANDSCAPE",
        Some("square") => "SQUARE",
        _ => "POSTER",
    }
}

fn stable_suffix(seed: &str) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in seed.as_bytes() {
        h ^= *b as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

fn merge_object(raw: &serde_json::Map<String, Value>, normalized: Value) -> Value {
    let mut merged = raw.clone();
    if let Some(normalized) = normalized.as_object() {
        for (key, value) in normalized {
            merged.insert(key.clone(), value.clone());
        }
    }
    Value::Object(merged)
}

pub(crate) fn import_collections_json(raw_json: &str) -> Option<String> {
    let parsed: Value = serde_json::from_str(raw_json).ok()?;
    let arr: Vec<&Value> = match parsed.as_array() {
        Some(a) => a.iter().collect(),
        None => vec![&parsed],
    };

    let collections: Vec<Value> = arr.iter().enumerate().filter_map(|(i, col)| {
        let col = col.as_object()?;
        let title = col.get("title")?.as_str()?.trim().to_string();
        if title.is_empty() { return None; }
        let id = col.get("id").and_then(Value::as_str).filter(|s| !s.trim().is_empty())
            .map(str::to_string)
            .unwrap_or_else(|| format!("imported_{}_{i}", stable_suffix(&title)));

        let raw_folders = col.get("folders").and_then(Value::as_array).map(Vec::as_slice).unwrap_or(&[]);
        let folders: Vec<Value> = raw_folders.iter().enumerate().filter_map(|(fi, f)| {
            let folder = f.as_object()?;
            let folder_title = folder.get("title")?.as_str()?.trim().to_string();
            if folder_title.is_empty() { return None; }
            let fid = folder.get("id").and_then(Value::as_str).filter(|s| !s.trim().is_empty())
                .map(str::to_string)
                .unwrap_or_else(|| format!("folder_{}_{fi}", stable_suffix(&folder_title)));

            let raw_sources = folder.get("catalogSources").and_then(Value::as_array).map(Vec::as_slice).unwrap_or(&[]);
            let mut sources: Vec<Value> = raw_sources.iter().filter_map(|s| {
                let o = s.as_object()?;
                let catalog_id = o.get("catalogId")?.as_str().filter(|s| !s.is_empty())?;
                Some(json!({
                    "catalogId": catalog_id,
                    "type": o.get("type").and_then(Value::as_str).unwrap_or("movie"),
                    "addonId": o.get("addonId").and_then(Value::as_str),
                    "genre": o.get("genre").and_then(Value::as_str),
                }))
            }).collect();

            if sources.is_empty()
                && let Some(fallback_id) = folder.get("catalogId").and_then(Value::as_str).filter(|s| !s.is_empty()) {
                    sources.push(json!({ "catalogId": fallback_id, "type": "movie" }));
                }
            let nuvio_sources: Vec<Value> = folder.get("sources")
                .and_then(Value::as_array)
                .map(Vec::as_slice)
                .unwrap_or(&[])
                .iter()
                .filter_map(|source| {
                    let provider = source.get("provider")?.as_str()?.to_ascii_lowercase();
                    match provider.as_str() {
                        "trakt" if source.get("traktListId").and_then(Value::as_i64).is_some() => Some(json!({
                            "provider": "trakt",
                            "title": source.get("title").and_then(Value::as_str),
                            "mediaType": source.get("mediaType").and_then(Value::as_str).unwrap_or("MOVIE"),
                            "traktListId": source.get("traktListId"),
                            "sortBy": source.get("sortBy").and_then(Value::as_str).unwrap_or("rank"),
                        "sortHow": source.get("sortHow").and_then(Value::as_str).unwrap_or("asc"),
                    })),
                        "tmdb" if source.get("tmdbSourceType").and_then(Value::as_str).is_some() => Some(json!({
                            "provider": "tmdb",
                            "title": source.get("title").and_then(Value::as_str),
                            "mediaType": source.get("mediaType").and_then(Value::as_str).unwrap_or("MOVIE"),
                            "tmdbSourceType": source.get("tmdbSourceType"),
                            "tmdbId": source.get("tmdbId"),
                            "sortBy": source.get("sortBy").and_then(Value::as_str),
                            "sortHow": source.get("sortHow").and_then(Value::as_str),
                            "filters": source.get("filters").cloned().unwrap_or(Value::Null),
                        })),
                        _ if source.get("addonId").and_then(Value::as_str).is_some()
                            && source.get("type").and_then(Value::as_str).is_some()
                            && source.get("catalogId").and_then(Value::as_str).is_some() => Some(json!({
                            "provider": "addon",
                            "addonId": source.get("addonId"),
                            "type": source.get("type"),
                            "catalogId": source.get("catalogId"),
                            "genre": source.get("genre").and_then(Value::as_str),
                        })),
                        _ => None,
                    }
                })
                .collect();

            let cover_image_url = cleaned_artwork_url(pick_str(folder, &["coverImageUrl","coverUrl","coverImage","cover","poster","thumbnail","thumb"]));
            let image_url = cleaned_artwork_url(pick_str(folder, &["imageUrl","image","image_url","posterUrl","poster_url"]));
            let effective_cover = cover_image_url.or(image_url);
            let hero_backdrop_url = cleaned_url(pick_str(folder, &["heroBackdropUrl","background","backdrop","backgroundUrl","backdropUrl"]));
            let shape = normalize_shape(folder.get("tileShape").or(folder.get("shape")).and_then(Value::as_str));

            Some(merge_object(folder, json!({
                "id": fid,
                "title": folder_title,
                "catalogTitle": folder.get("catalogTitle").and_then(Value::as_str).unwrap_or(&folder_title),
                "catalogId": sources.first().and_then(|s| s.get("catalogId")).and_then(Value::as_str),
                "genre": folder.get("genre").and_then(Value::as_str),
                "shape": shape,
                "hideTitle": folder.get("hideTitle").and_then(Value::as_bool).unwrap_or(false),
                "focusGifEnabled": folder.get("focusGifEnabled").and_then(Value::as_bool).unwrap_or(true),
                "catalogSources": folder.get("catalogSources").cloned().unwrap_or_else(|| if sources.is_empty() { Value::Null } else { json!(sources) }),
                "sources": folder.get("sources").cloned().unwrap_or_else(|| if nuvio_sources.is_empty() { Value::Null } else { json!(nuvio_sources) }),
                "coverEmoji": folder.get("coverEmoji").and_then(Value::as_str),
                "imageUrl": effective_cover,
                "coverImageUrl": effective_cover,
                "focusGifUrl": cleaned_url(folder.get("focusGifUrl").and_then(Value::as_str)),
                "titleLogoUrl": cleaned_url(folder.get("titleLogoUrl").and_then(Value::as_str)),
                "heroBackdropUrl": hero_backdrop_url,
                "heroVideoUrl": cleaned_url(folder.get("heroVideoUrl").and_then(Value::as_str)),
            })))
        }).collect();

        let first_folder_cover = raw_folders.first()
            .and_then(|f| f.as_object())
            .and_then(|f| cleaned_artwork_url(pick_str(f, &["coverImageUrl","coverUrl","coverImage","cover","poster","thumbnail","thumb"]))
                .or_else(|| cleaned_artwork_url(pick_str(f, &["imageUrl","image","image_url","posterUrl","poster_url"]))));

        Some(merge_object(col, json!({
            "id": id,
            "title": title,
            "backdropImageUrl": cleaned_url(col.get("backdropImageUrl").and_then(Value::as_str)),
            "imageUrl": first_folder_cover,
            "showOnHome": col.get("showOnHome").and_then(Value::as_bool).unwrap_or(true),
            "itemIds": [],
            "folders": folders,
            "showAllTab": col.get("showAllTab").and_then(Value::as_bool).unwrap_or(true),
            "viewMode": col.get("viewMode").and_then(Value::as_str).unwrap_or("FOLLOW_LAYOUT"),
            "pinToTop": col.get("pinToTop").and_then(Value::as_bool).unwrap_or(false),
            "focusGlowEnabled": col.get("focusGlowEnabled").and_then(Value::as_bool).unwrap_or(true),
        })))
    }).collect();

    serde_json::to_string(&collections).ok()
}

pub(crate) fn export_collections_json(collections_json: &str) -> Option<String> {
    let collections: Vec<Value> = serde_json::from_str(collections_json).ok()?;
    let data: Vec<Value> = collections
        .iter()
        .filter_map(|collection| {
            let mut collection = collection.as_object()?.clone();
            let folders = collection
                .get("folders")
                .and_then(Value::as_array)?
                .iter()
                .filter_map(|folder| {
                    let mut folder = folder.as_object()?.clone();
                    let tile_shape = folder.get("tileShape").cloned().unwrap_or_else(|| {
                        Value::String(
                            export_shape(folder.get("shape").and_then(Value::as_str)).to_string(),
                        )
                    });
                    folder.insert("tileShape".to_string(), tile_shape);
                    folder
                        .entry("hideTitle".to_string())
                        .or_insert_with(|| Value::Bool(false));
                    folder
                        .entry("focusGifEnabled".to_string())
                        .or_insert_with(|| Value::Bool(true));
                    folder
                        .entry("catalogSources".to_string())
                        .or_insert_with(|| Value::Array(Vec::new()));
                    folder
                        .entry("sources".to_string())
                        .or_insert_with(|| Value::Array(Vec::new()));
                    Some(Value::Object(folder))
                })
                .collect();
            collection.insert("folders".to_string(), Value::Array(folders));
            collection
                .entry("showAllTab".to_string())
                .or_insert_with(|| Value::Bool(true));
            collection
                .entry("viewMode".to_string())
                .or_insert_with(|| Value::String("FOLLOW_LAYOUT".to_string()));
            collection
                .entry("pinToTop".to_string())
                .or_insert_with(|| Value::Bool(false));
            Some(Value::Object(collection))
        })
        .collect();
    serde_json::to_string(&data).ok()
}
