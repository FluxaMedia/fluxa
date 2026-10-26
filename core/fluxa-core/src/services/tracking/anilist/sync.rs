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
    #[serde(rename = "nextAiringEpisode")]
    next_airing: Option<AnilistAiring>,
}

#[derive(serde::Deserialize)]
struct AnilistAiring {
    episode: Option<i64>,
    #[serde(rename = "airingAt")]
    airing_at: Option<i64>,
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
    let mut on_hold: Vec<Value> = Vec::new();
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
                if status == "PAUSED" {
                    on_hold.push(Value::Object(it));
                } else {
                    dropped.push(Value::Object(it));
                }
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
                let finished = media
                    .episodes
                    .is_some_and(|total| total > 0 && progress_episode >= total);
                let airing = media.next_airing.as_ref().and_then(|airing| {
                    let upcoming = airing.episode?;
                    (progress_episode + 1 >= upcoming).then_some(airing.airing_at)
                });
                if !finished {
                    let next_episode = progress_episode + 1;
                    let mut it = item.clone();
                    it.insert(
                        "lastVideoId".to_string(),
                        json!(format!("{id}:1:{next_episode}")),
                    );
                    it.insert("lastEpisodeSeason".to_string(), json!(1));
                    it.insert("lastEpisodeNumber".to_string(), json!(next_episode));
                    it.insert(
                        "lastEpisodeName".to_string(),
                        json!(format!("Episode {next_episode}")),
                    );
                    it.insert("continueWatchingBadge".to_string(), json!("upNext"));
                    it.insert("savedAt".to_string(), json!(updated_at));
                    if let Some(airing_at) = airing {
                        it.insert("upcoming".to_string(), json!(true));
                        it.insert("airsAt".to_string(), json!(airing_at.map(|at| at * 1000)));
                    }
                    watching.push(Value::Object(it));
                }
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
        "onHold": if watched_apply { json!(on_hold) } else { Value::Null },
        "watching": if continue_watching_apply { json!(watching) } else { Value::Null },
        "watchingCount": watching_count,
        "watched": if watched_apply { Value::Object(watched) } else { Value::Null },
        "watchedUpdatedAtMs": if watched_apply { Value::Object(watched_at_ms) } else { Value::Null },
        "progress": if continue_watching_apply { Value::Object(progress) } else { Value::Null },
    })
}
