use serde_json::{Map, Value, json};

#[derive(serde::Deserialize)]
struct AnilistEntry {
    status: Option<String>,
    progress: Option<f64>,
    #[serde(rename = "updatedAt")]
    updated_at: Option<i64>,
    media: Option<AnilistMedia>,
}

#[derive(serde::Deserialize)]
struct AnilistMedia {
    id: Option<i64>,
    title: Option<AnilistTitle>,
    #[serde(rename = "coverImage")]
    cover_image: Option<AnilistCover>,
    #[serde(rename = "bannerImage")]
    banner_image: Option<String>,
    episodes: Option<i64>,
    #[serde(rename = "seasonYear")]
    season_year: Option<i64>,
    genres: Option<Vec<String>>,
}

#[derive(serde::Deserialize)]
struct AnilistTitle {
    english: Option<String>,
    romaji: Option<String>,
    native: Option<String>,
}

#[derive(serde::Deserialize)]
struct AnilistCover {
    #[serde(rename = "extraLarge")]
    extra_large: Option<String>,
    large: Option<String>,
}

fn anilist_media_to_item(media: &AnilistMedia, media_id: i64) -> Map<String, Value> {
    let title = media
        .title
        .as_ref()
        .and_then(|t| {
            [&t.english, &t.romaji, &t.native]
                .into_iter()
                .find_map(|s| s.as_deref().map(str::trim).filter(|s| !s.is_empty()))
        })
        .map(str::to_string)
        .unwrap_or_else(|| format!("AniList {media_id}"));

    let mut item = Map::new();
    item.insert("id".to_string(), json!(format!("anilist:{media_id}")));
    item.insert("name".to_string(), json!(title));
    item.insert("type".to_string(), json!("series"));
    let poster = media
        .cover_image
        .as_ref()
        .and_then(|c| c.extra_large.as_deref().or(c.large.as_deref()));
    if let Some(poster) = poster {
        item.insert("poster".to_string(), json!(poster));
    }
    if let Some(banner) = &media.banner_image {
        item.insert("background".to_string(), json!(banner));
    }
    if let Some(year) = media.season_year {
        item.insert("year".to_string(), json!(year));
    }
    if let Some(genres) = &media.genres {
        item.insert("genres".to_string(), json!(genres));
    }
    item.insert("anilistId".to_string(), json!(media_id));
    if let Some(episodes) = media.episodes {
        item.insert("totalEpisodes".to_string(), json!(episodes));
    }
    item
}

fn anilist_updated_at(updated_at_seconds: Option<i64>, now_ms: i64) -> String {
    let seconds = updated_at_seconds.unwrap_or(0);
    let ms = if seconds > 0 { seconds * 1000 } else { now_ms };
    chrono::DateTime::<chrono::Utc>::from_timestamp_millis(ms)
        .map(|dt| dt.to_rfc3339_opts(chrono::SecondsFormat::Millis, true))
        .unwrap_or_default()
}

fn mark_watched_through(
    watched: &mut serde_json::Map<String, Value>,
    watched_at_ms: &mut serde_json::Map<String, Value>,
    id: &str,
    through: i64,
    updated_at_ms: i64,
) {
    for ep in 1..=through.max(0) {
        let key = format!("{id}:1:{ep}");
        watched.insert(key.clone(), Value::Bool(true));
        watched_at_ms.insert(key, json!(updated_at_ms));
    }
}

pub(crate) fn anilist_media_id_from_content_id(content_id: &str) -> Option<i64> {
    content_id.strip_prefix("anilist:")?.parse::<i64>().ok()
}

const SAVE_MEDIA_LIST_ENTRY_MUTATION: &str = r#"
    mutation ($mediaId: Int, $status: MediaListStatus, $progress: Int) {
        SaveMediaListEntry(mediaId: $mediaId, status: $status, progress: $progress) { id }
    }
"#;

const MEDIA_LIST_ENTRY_LOOKUP_QUERY: &str = r#"
    query ($mediaId: Int) {
        Media(id: $mediaId) { mediaListEntry { id } }
    }
"#;

const DELETE_MEDIA_LIST_ENTRY_MUTATION: &str = r#"
    mutation ($id: Int) {
        DeleteMediaListEntry(id: $id) { deleted }
    }
"#;

pub(crate) fn anilist_graphql_queries_json() -> String {
    json!({
        "saveMediaListEntry": SAVE_MEDIA_LIST_ENTRY_MUTATION,
        "mediaListEntryLookup": MEDIA_LIST_ENTRY_LOOKUP_QUERY,
        "deleteMediaListEntry": DELETE_MEDIA_LIST_ENTRY_MUTATION,
    })
    .to_string()
}

pub(crate) fn anilist_save_media_list_entry_variables_json(
    content_id: &str,
    status: &str,
    progress: Option<i64>,
) -> Option<String> {
    let media_id = anilist_media_id_from_content_id(content_id)?;
    let mut variables = Map::new();
    variables.insert("mediaId".to_string(), json!(media_id));
    variables.insert("status".to_string(), json!(status));
    if let Some(progress) = progress {
        variables.insert("progress".to_string(), json!(progress.max(0)));
    }
    serde_json::to_string(&Value::Object(variables)).ok()
}

fn extract_after_marker(text: &str, marker: &str) -> Option<i64> {
    let lower = text.to_ascii_lowercase();
    let mut search_from = 0;
    while let Some(offset) = lower[search_from..].find(marker) {
        let idx = search_from + offset;
        let rest = &text[idx + marker.len()..];
        let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
        if !digits.is_empty()
            && let Ok(id) = digits.parse()
        {
            return Some(id);
        }
        search_from = idx + marker.len();
    }
    None
}

pub(crate) fn extract_anilist_id_from_links(meta: &Value) -> Option<i64> {
    let links = meta.get("links")?.as_array()?;
    let text = links
        .iter()
        .map(|link| {
            format!(
                "{} {}",
                link.get("url").and_then(Value::as_str).unwrap_or(""),
                link.get("name").and_then(Value::as_str).unwrap_or(""),
            )
        })
        .collect::<Vec<_>>()
        .join(" ");
    extract_after_marker(&text, "anilist.co/anime/")
}

fn normalize_anime_title(value: &str) -> String {
    let mut out = String::new();
    let mut last_was_space = true;
    for c in value.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c.to_ascii_lowercase());
            last_was_space = false;
        } else if !last_was_space {
            out.push(' ');
            last_was_space = true;
        }
    }
    out.trim_end().to_string()
}

fn parse_year_from_text(value: &str) -> Option<i64> {
    let bytes = value.as_bytes();
    for i in 0..bytes.len().saturating_sub(3) {
        let Some(candidate) = bytes.get(i..i + 4) else {
            continue;
        };
        if !candidate.iter().all(|byte| byte.is_ascii_digit()) {
            continue;
        }
        if !matches!(candidate, [b'1', b'9', ..] | [b'2', b'0', ..]) {
            continue;
        }
        let before_ok = i == 0 || !bytes.get(i - 1).is_some_and(u8::is_ascii_alphanumeric);
        let after_ok =
            i + 4 == bytes.len() || !bytes.get(i + 4).is_some_and(u8::is_ascii_alphanumeric);
        if before_ok && after_ok {
            let [first, second, third, fourth] = *candidate else {
                continue;
            };
            return Some(
                i64::from(first - b'0') * 1000
                    + i64::from(second - b'0') * 100
                    + i64::from(third - b'0') * 10
                    + i64::from(fourth - b'0'),
            );
        }
    }
    None
}

pub(crate) fn anilist_search_best_match_json(args_json: &str) -> Option<String> {
    let args: Value = serde_json::from_str(args_json).ok()?;
    let meta = args.get("meta")?;
    let candidates = args.get("candidates")?.as_array()?;

    let search_name = meta.get("name").and_then(Value::as_str)?.trim();
    if search_name.is_empty() {
        return None;
    }
    let year = meta.get("year").and_then(Value::as_i64).or_else(|| {
        meta.get("releaseInfo")
            .and_then(Value::as_str)
            .and_then(parse_year_from_text)
    });
    let normalized_name = normalize_anime_title(search_name);

    let name_matches = |item: &Value| -> bool {
        item.get("title")
            .and_then(Value::as_object)
            .is_some_and(|title| {
                title.values().any(|t| {
                    t.as_str()
                        .is_some_and(|s| normalize_anime_title(s) == normalized_name)
                })
            })
    };
    let year_ok = |item: &Value| -> bool {
        let season_year = item.get("seasonYear").and_then(Value::as_i64);
        match (year, season_year) {
            (Some(y), Some(sy)) => (y - sy).abs() <= 1,
            _ => true,
        }
    };

    let best = candidates
        .iter()
        .find(|item| name_matches(item) && year_ok(item))
        .or_else(|| candidates.iter().find(|item| year_ok(item)))?;
    let id = best.get("id")?.as_i64()?;
    serde_json::to_string(&json!({ "anilistId": id, "confidence": "title-year" })).ok()
}

pub(crate) fn anilist_media_list_status(
    total_episodes: i64,
    progress_episode: i64,
) -> &'static str {
    if total_episodes > 0 && progress_episode >= total_episodes {
        "COMPLETED"
    } else {
        "CURRENT"
    }
}

pub(crate) fn anilist_entries_to_sync(
    entries: &[Value],
    now_ms: i64,
    categories: Option<&[&str]>,
    dry_run: bool,
) -> Value {
    let wants = |c: &str| categories.is_none_or(|cats| cats.contains(&c));
    let mut watchlist: Vec<Value> = Vec::new();
    let mut completed: Vec<Value> = Vec::new();
    let mut dropped: Vec<Value> = Vec::new();
    let mut watching: Vec<Value> = Vec::new();
    let mut watched = Map::new();
    let mut watched_at_ms = Map::new();
    let mut progress = Map::new();

    for raw in entries {
        let Ok(entry) = serde_json::from_value::<AnilistEntry>(raw.clone()) else {
            continue;
        };
        let Some(media) = &entry.media else {
            continue;
        };
        let Some(media_id) = media.id else {
            continue;
        };
        let item = anilist_media_to_item(media, media_id);
        let id = format!("anilist:{media_id}");
        let status = entry.status.as_deref().unwrap_or("").to_uppercase();
        let progress_episode = entry
            .progress
            .map(|p| p.floor().max(0.0) as i64)
            .unwrap_or(0);
        let updated_at = anilist_updated_at(entry.updated_at, now_ms);
        let updated_at_ms = entry.updated_at.map(|s| s * 1000).unwrap_or(now_ms);

        match status.as_str() {
            "PLANNING" => {
                let mut it = item.clone();
                it.insert("inWatchlist".to_string(), Value::Bool(true));
                it.insert("updatedAtMs".to_string(), json!(updated_at_ms));
                watchlist.push(Value::Object(it));
            }
            "COMPLETED" => {
                let mut it = item.clone();
                it.insert("statusChangedAt".to_string(), json!(updated_at));
                completed.push(Value::Object(it));
                let through = if progress_episode > 0 {
                    progress_episode
                } else {
                    media.episodes.unwrap_or(0)
                };
                mark_watched_through(
                    &mut watched,
                    &mut watched_at_ms,
                    &id,
                    through,
                    updated_at_ms,
                );
            }
            "DROPPED" | "PAUSED" => {
                let mut it = item.clone();
                it.insert("statusChangedAt".to_string(), json!(updated_at));
                dropped.push(Value::Object(it));
                mark_watched_through(
                    &mut watched,
                    &mut watched_at_ms,
                    &id,
                    progress_episode,
                    updated_at_ms,
                );
            }
            "CURRENT" | "REPEATING" if progress_episode > 0 => {
                let last_video_id = format!("{id}:1:{progress_episode}");
                let episode_name = format!("Episode {progress_episode}");
                let mut it = item.clone();
                it.insert("lastVideoId".to_string(), json!(last_video_id));
                it.insert("lastEpisodeSeason".to_string(), json!(1));
                it.insert("lastEpisodeNumber".to_string(), json!(progress_episode));
                it.insert("lastEpisodeName".to_string(), json!(episode_name));
                it.insert("timeOffset".to_string(), json!(1));
                it.insert("duration".to_string(), json!(1));
                watching.push(Value::Object(it));
                progress.insert(
                    id.clone(),
                    json!({
                        "meta": Value::Object(item.clone()),
                        "lastVideoId": last_video_id,
                        "lastEpisodeSeason": 1,
                        "lastEpisodeNumber": progress_episode,
                        "lastEpisodeName": episode_name,
                        "timeOffset": 1,
                        "duration": 1,
                        "savedAt": updated_at,
                    }),
                );
                mark_watched_through(
                    &mut watched,
                    &mut watched_at_ms,
                    &id,
                    progress_episode,
                    updated_at_ms,
                );
            }
            _ => {}
        }
    }

    let watchlist_count = watchlist.len();
    let completed_count = completed.len();
    let dropped_count = dropped.len();
    let watching_count = watching.len();
    let watched_apply = wants("watched") && !dry_run;
    let continue_watching_apply = wants("continueWatching") && !dry_run;

    json!({
        "watchlist": if wants("watchlist") && !dry_run { json!(watchlist) } else { Value::Null },
        "watchlistCount": watchlist_count,
        "completed": if watched_apply { json!(completed) } else { Value::Null },
        "completedCount": completed_count,
        "dropped": if watched_apply { json!(dropped) } else { Value::Null },
        "droppedCount": dropped_count,
        "watching": if continue_watching_apply { json!(watching) } else { Value::Null },
        "watchingCount": watching_count,
        "watched": if watched_apply { Value::Object(watched) } else { Value::Null },
        "watchedUpdatedAtMs": if watched_apply { Value::Object(watched_at_ms) } else { Value::Null },
        "progress": if continue_watching_apply { Value::Object(progress) } else { Value::Null },
    })
}

pub(crate) fn merge_library_items_by_id(local: &[Value], incoming: &[Value]) -> Value {
    fn item_id(item: &Value) -> String {
        match item.get("id") {
            Some(Value::String(s)) => s.clone(),
            Some(Value::Number(n)) => n.to_string(),
            _ => String::new(),
        }
    }

    let mut order: Vec<String> = Vec::new();
    let mut by_id: std::collections::HashMap<String, Value> = std::collections::HashMap::new();
    for item in local {
        let id = item_id(item);
        if !by_id.contains_key(&id) {
            order.push(id.clone());
        }
        by_id.insert(id, item.clone());
    }
    for item in incoming {
        let id = item_id(item);
        let merged = match (by_id.get(&id), item) {
            (Some(Value::Object(existing)), Value::Object(inc)) => {
                let mut m = existing.clone();
                for (k, v) in inc {
                    m.insert(k.clone(), v.clone());
                }
                Value::Object(m)
            }
            _ => item.clone(),
        };
        if !by_id.contains_key(&id) {
            order.push(id.clone());
        }
        by_id.insert(id, merged);
    }

    let merged: Vec<Value> = order.iter().filter_map(|id| by_id.remove(id)).collect();
    Value::Array(merged)
}
