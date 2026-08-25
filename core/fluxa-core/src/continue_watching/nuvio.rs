use serde_json::{Map, Value, json};
use std::collections::{HashMap, HashSet};

const COMPLETION_FRACTION: f64 = 0.90;
const COMPLETION_PERCENT: f64 = 90.0;
const PROGRESS_STORE_THRESHOLD_MS: f64 = 1_000.0;
const HOME_MAX_RECENT_PROGRESS_ITEMS: usize = 300;
const UPCOMING_NEXT_SEASON_WINDOW_DAYS: i64 = 7;
const DAY_MS: i64 = 86_400_000;

#[derive(Clone)]
struct Entry {
    content_id: String,
    content_type: String,
    video_id: String,
    season: Option<i64>,
    episode: Option<i64>,
    position_ms: f64,
    duration_ms: f64,
    last_updated_ms: i64,
    progress_key: String,
    completed: bool,
    percent: Option<f64>,
    source: String,
}

struct CompletedEpisode {
    season: i64,
    episode: i64,
    marked_at_ms: i64,
}

struct Seed {
    content_id: String,
    content_type: String,
    season: i64,
    episode: i64,
    marked_at_ms: i64,
}

fn normalize_season(season: Option<i64>) -> i64 {
    season.map(|value| value.max(0)).unwrap_or(0)
}

fn is_series_like(content_type: &str) -> bool {
    matches!(
        content_type.trim().to_ascii_lowercase().as_str(),
        "series" | "show" | "tv" | "tvshow"
    )
}

fn is_series_for_continue_watching(content_type: &str) -> bool {
    matches!(
        content_type.trim().to_ascii_lowercase().as_str(),
        "series" | "tv"
    )
}

fn is_malformed_seed_content_id(content_id: &str) -> bool {
    let trimmed = content_id.trim();
    if trimmed.is_empty() {
        return true;
    }
    matches!(
        trimmed.to_ascii_lowercase().as_str(),
        "tmdb" | "imdb" | "trakt" | "tmdb:" | "imdb:" | "trakt:"
    )
}

impl Entry {
    fn from_value(value: &Value) -> Option<Self> {
        let content_id = value.get("content_id")?.as_str()?.trim().to_string();
        if content_id.is_empty() {
            return None;
        }
        let content_type = value
            .get("content_type")
            .and_then(Value::as_str)
            .unwrap_or("")
            .trim()
            .to_string();
        let season = value.get("season").and_then(Value::as_i64);
        let episode = value.get("episode").and_then(Value::as_i64);
        let video_id = value
            .get("video_id")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|id| !id.is_empty())
            .map(str::to_string)
            .unwrap_or_else(|| match (season, episode) {
                (Some(season), Some(episode)) => format!("{content_id}:{season}:{episode}"),
                _ => content_id.clone(),
            });
        let progress_key = value
            .get("progress_key")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|key| !key.is_empty())
            .map(str::to_string)
            .unwrap_or_else(|| match (season, episode) {
                (Some(season), Some(episode)) => format!("{content_id}_s{season}e{episode}"),
                _ => content_id.clone(),
            });
        Some(Self {
            content_id,
            content_type,
            video_id,
            season,
            episode,
            position_ms: value
                .get("position")
                .and_then(Value::as_f64)
                .unwrap_or(0.0)
                .max(0.0),
            duration_ms: value.get("duration").and_then(Value::as_f64).unwrap_or(0.0),
            last_updated_ms: value
                .get("last_watched")
                .and_then(Value::as_i64)
                .unwrap_or(0),
            progress_key,
            completed: value
                .get("is_completed")
                .and_then(Value::as_bool)
                .unwrap_or(false),
            percent: value
                .get("progress_percent")
                .and_then(Value::as_f64)
                .map(|percent| percent.clamp(0.0, 100.0)),
            source: value
                .get("source")
                .and_then(Value::as_str)
                .unwrap_or("nuvio")
                .to_string(),
        })
    }

    fn is_effectively_completed(&self) -> bool {
        if self.completed {
            return true;
        }
        if self.percent.is_some_and(|percent| percent >= COMPLETION_PERCENT) {
            return true;
        }
        self.duration_ms > 0.0 && self.position_ms / self.duration_ms >= COMPLETION_FRACTION
    }

    fn has_started(&self) -> bool {
        self.position_ms > 0.0 || self.percent.is_some_and(|percent| percent > 0.0)
    }

    fn is_in_progress(&self) -> bool {
        if self.is_effectively_completed() || !self.has_started() {
            return false;
        }
        self.source != "trakt_history" && self.source != "trakt_show_progress"
    }

    fn should_store(&self) -> bool {
        self.position_ms >= PROGRESS_STORE_THRESHOLD_MS
    }

    fn fraction(&self) -> f64 {
        if let Some(percent) = self.percent {
            return (percent / 100.0).clamp(0.0, 1.0);
        }
        if self.duration_ms > 0.0 {
            (self.position_ms / self.duration_ms).clamp(0.0, 1.0)
        } else {
            0.0
        }
    }
}

fn freshness_key(entry: &Entry) -> (i64, i64, i64, i64, bool, String, String) {
    (
        entry.last_updated_ms,
        entry.season.unwrap_or(0),
        entry.episode.unwrap_or(0),
        entry.position_ms as i64,
        entry.is_effectively_completed(),
        entry.progress_key.clone(),
        entry.video_id.clone(),
    )
}

fn newest_by_progress_key(entries: Vec<Entry>) -> Vec<Entry> {
    let mut newest: HashMap<String, Entry> = HashMap::new();
    for entry in entries {
        match newest.get(&entry.progress_key) {
            Some(existing) if freshness_key(existing) >= freshness_key(&entry) => {}
            _ => {
                newest.insert(entry.progress_key.clone(), entry);
            }
        }
    }
    let mut values: Vec<Entry> = newest.into_values().collect();
    values.sort_by(|a, b| freshness_key(b).cmp(&freshness_key(a)));
    values
}

fn continue_watching_entries(entries: &[Entry], limit: usize) -> Vec<Entry> {
    let selection: Vec<Entry> = entries
        .iter()
        .filter(|entry| entry.is_effectively_completed() || entry.is_in_progress())
        .cloned()
        .collect();
    let selection = newest_by_progress_key(selection);

    let (series, others): (Vec<&Entry>, Vec<&Entry>) = selection
        .iter()
        .partition(|entry| is_series_like(&entry.content_type) || entry.is_episode_like());

    let mut latest_per_series: HashMap<String, &Entry> = HashMap::new();
    for entry in series {
        match latest_per_series.get(entry.content_id.trim()) {
            Some(existing) if freshness_key(existing) >= freshness_key(entry) => {}
            _ => {
                latest_per_series.insert(entry.content_id.trim().to_string(), entry);
            }
        }
    }

    let mut kept: Vec<&Entry> = others
        .into_iter()
        .chain(latest_per_series.into_values())
        .filter(|entry| !entry.is_effectively_completed())
        .collect();
    kept.sort_by(|a, b| freshness_key(b).cmp(&freshness_key(a)));
    kept.into_iter()
        .take(limit)
        .filter(|entry| entry.is_in_progress())
        .cloned()
        .collect()
}

impl Entry {
    fn is_episode_like(&self) -> bool {
        self.season.is_some() && self.episode.is_some()
    }
}

fn latest_completed_by_series(
    entries: &[Entry],
    prefer_furthest: bool,
) -> HashMap<String, CompletedEpisode> {
    let mut best: HashMap<String, CompletedEpisode> = HashMap::new();
    for entry in entries {
        if !entry.is_effectively_completed() {
            continue;
        }
        let (Some(season), Some(episode)) = (entry.season, entry.episode) else {
            continue;
        };
        let candidate = CompletedEpisode {
            season,
            episode,
            marked_at_ms: entry.last_updated_ms,
        };
        let order = |value: &CompletedEpisode| {
            if prefer_furthest {
                (
                    normalize_season(Some(value.season)),
                    value.episode,
                    value.marked_at_ms,
                )
            } else {
                (
                    value.marked_at_ms,
                    normalize_season(Some(value.season)),
                    value.episode,
                )
            }
        };
        match best.get(&entry.content_id) {
            Some(existing) if order(existing) >= order(&candidate) => {}
            _ => {
                best.insert(entry.content_id.clone(), candidate);
            }
        }
    }
    best
}

fn build_seed_candidates(entries: &[Entry], prefer_furthest: bool) -> Vec<Seed> {
    let seeds: Vec<Entry> = entries
        .iter()
        .filter(|entry| is_series_for_continue_watching(&entry.content_type))
        .filter(|entry| {
            entry.season.is_some() && entry.episode.is_some() && entry.season != Some(0)
        })
        .filter(|entry| !is_malformed_seed_content_id(&entry.content_id))
        .filter(|entry| entry.is_effectively_completed())
        .cloned()
        .collect();
    let types: HashMap<String, String> = seeds
        .iter()
        .map(|entry| (entry.content_id.clone(), entry.content_type.clone()))
        .collect();

    let mut candidates: Vec<Seed> = latest_completed_by_series(&seeds, prefer_furthest)
        .into_iter()
        .filter(|(_, completed)| completed.season != 0)
        .map(|(content_id, completed)| Seed {
            content_type: types.get(&content_id).cloned().unwrap_or_default(),
            content_id,
            season: completed.season,
            episode: completed.episode,
            marked_at_ms: completed.marked_at_ms,
        })
        .collect();
    candidates.sort_by(|a, b| {
        (b.marked_at_ms, b.season, b.episode).cmp(&(a.marked_at_ms, a.season, a.episode))
    });
    candidates
}

fn episode_released_at(video: &Value) -> Option<i64> {
    video
        .get("released")
        .or_else(|| video.get("firstAired"))
        .and_then(Value::as_str)
        .and_then(|value| chrono::DateTime::parse_from_rfc3339(value).ok())
        .map(|value| value.timestamp_millis())
}

fn should_surface_next_episode(
    watched_season: i64,
    candidate: &Value,
    now_ms: i64,
    show_unaired: bool,
) -> bool {
    let candidate_season = candidate.get("season").and_then(Value::as_i64);
    let season_rollover = normalize_season(candidate_season) != normalize_season(Some(watched_season));
    let available = candidate
        .get("available")
        .and_then(Value::as_bool)
        .unwrap_or(true);
    let released_at = episode_released_at(candidate);
    let days_until = released_at.map(|released| (released - now_ms).div_euclid(DAY_MS));

    if !available {
        let Some(days) = days_until else {
            return false;
        };
        if days <= 0 {
            return true;
        }
        if !show_unaired {
            return false;
        }
        return !season_rollover || days <= UPCOMING_NEXT_SEASON_WINDOW_DAYS;
    }
    if !season_rollover {
        if show_unaired {
            return true;
        }
        return released_at.is_none_or(|released| released <= now_ms);
    }
    if released_at.is_some_and(|released| released <= now_ms) {
        return true;
    }
    if !show_unaired {
        return false;
    }
    days_until.is_some_and(|days| (0..=UPCOMING_NEXT_SEASON_WINDOW_DAYS).contains(&days))
}

fn sorted_episodes(videos: &[Value]) -> Vec<Value> {
    let mut sorted = videos.to_vec();
    sorted.sort_by_key(|video| {
        (
            normalize_season(video.get("season").and_then(Value::as_i64)),
            video
                .get("episode")
                .or_else(|| video.get("number"))
                .and_then(Value::as_i64)
                .unwrap_or(0),
        )
    });
    sorted
}

fn next_released_episode_after(
    content_id: &str,
    videos: &[Value],
    season: i64,
    episode: i64,
    now_ms: i64,
    show_unaired: bool,
) -> Option<Value> {
    let sorted = sorted_episodes(videos);
    let watched_video_id = format!("{content_id}:{season}:{episode}");
    let identity = |video: &Value| {
        match (
            video.get("season").and_then(Value::as_i64),
            video
                .get("episode")
                .or_else(|| video.get("number"))
                .and_then(Value::as_i64),
        ) {
            (Some(season), Some(episode)) => format!("{content_id}:{season}:{episode}"),
            _ => video
                .get("id")
                .or_else(|| video.get("_id"))
                .and_then(Value::as_str)
                .unwrap_or(content_id)
                .to_string(),
        }
    };
    let mut watched_index = sorted
        .iter()
        .position(|video| identity(video) == watched_video_id);

    if watched_index.is_none() && season == 1 && episode > 0 {
        let main: Vec<usize> = sorted
            .iter()
            .enumerate()
            .filter(|(_, video)| {
                normalize_season(video.get("season").and_then(Value::as_i64)) > 0
            })
            .map(|(index, _)| index)
            .collect();
        let seasons: HashSet<i64> = main
            .iter()
            .map(|index| normalize_season(sorted[*index].get("season").and_then(Value::as_i64)))
            .collect();
        if seasons.len() > 1 {
            let global_index = (episode - 1) as usize;
            if let Some(index) = main.get(global_index) {
                watched_index = Some(*index);
            }
        }
    }

    let watched_index = watched_index?;
    let watched_season = sorted[watched_index]
        .get("season")
        .and_then(Value::as_i64)
        .unwrap_or(season);
    sorted
        .iter()
        .skip(watched_index + 1)
        .filter(|video| should_surface_next_episode(watched_season, video, now_ms, show_unaired))
        .find(|video| normalize_season(video.get("season").and_then(Value::as_i64)) > 0)
        .cloned()
}

fn meta_field(meta: Option<&Value>, name: &str) -> Value {
    meta.and_then(|meta| meta.get(name))
        .cloned()
        .unwrap_or(Value::Null)
}

fn episode_in(meta: Option<&Value>, season: Option<i64>, episode: Option<i64>) -> Option<Value> {
    let (season, episode) = (season?, episode?);
    meta?
        .get("videos")?
        .as_array()?
        .iter()
        .find(|video| {
            video.get("season").and_then(Value::as_i64) == Some(season)
                && video
                    .get("episode")
                    .or_else(|| video.get("number"))
                    .and_then(Value::as_i64)
                    == Some(episode)
        })
        .cloned()
}

fn episode_field(video: Option<&Value>, names: &[&str]) -> Value {
    names
        .iter()
        .find_map(|name| {
            video
                .and_then(|video| video.get(*name))
                .filter(|value| value.as_str().is_none_or(|text| !text.trim().is_empty()))
                .filter(|value| !value.is_null())
        })
        .cloned()
        .unwrap_or(Value::Null)
}

fn in_progress_item(entry: &Entry, meta: Option<&Value>) -> Value {
    let video = episode_in(meta, entry.season, entry.episode);
    json!({
        "id": entry.content_id,
        "_id": entry.content_id,
        "type": if entry.content_type.is_empty() { Value::String("series".into()) } else { Value::String(entry.content_type.clone()) },
        "name": meta_field(meta, "name"),
        "poster": meta_field(meta, "poster"),
        "background": meta_field(meta, "background"),
        "logo": meta_field(meta, "logo"),
        "timeOffset": (entry.position_ms / 1000.0).ceil() as i64,
        "duration": (entry.duration_ms / 1000.0).floor() as i64,
        "lastVideoId": entry.video_id,
        "lastEpisodeName": episode_field(video.as_ref(), &["name", "title"]),
        "lastEpisodeSeason": entry.season,
        "lastEpisodeNumber": entry.episode,
        "lastEpisodeThumbnail": episode_field(video.as_ref(), &["thumbnail"]),
        "progressFraction": entry.fraction(),
        "continueWatchingBadge": Value::Null,
        "continueWatchingEpisodeResolved": Value::Null,
        "savedAt": iso(entry.last_updated_ms),
        "source": "nuvio",
    })
}

fn next_up_item(seed: &Seed, next: &Value, meta: Option<&Value>) -> Value {
    let season = next.get("season").cloned().unwrap_or(Value::Null);
    let episode = next
        .get("episode")
        .or_else(|| next.get("number"))
        .cloned()
        .unwrap_or(Value::Null);
    let video_id = match (season.as_i64(), episode.as_i64()) {
        (Some(season), Some(episode)) => format!("{}:{season}:{episode}", seed.content_id),
        _ => next
            .get("id")
            .or_else(|| next.get("_id"))
            .and_then(Value::as_str)
            .unwrap_or(&seed.content_id)
            .to_string(),
    };
    json!({
        "id": seed.content_id,
        "_id": seed.content_id,
        "type": if seed.content_type.is_empty() { Value::String("series".into()) } else { Value::String(seed.content_type.clone()) },
        "name": meta_field(meta, "name"),
        "poster": meta_field(meta, "poster"),
        "background": meta_field(meta, "background"),
        "logo": meta_field(meta, "logo"),
        "timeOffset": 0,
        "duration": 0,
        "lastVideoId": video_id,
        "lastEpisodeName": next.get("name").or_else(|| next.get("title")).cloned().unwrap_or(Value::Null),
        "lastEpisodeSeason": season,
        "lastEpisodeNumber": episode,
        "lastEpisodeThumbnail": next.get("thumbnail").cloned().unwrap_or(Value::Null),
        "progressFraction": 0.0,
        "continueWatchingBadge": "upNext",
        "continueWatchingEpisodeResolved": true,
        "newEpisodeReleasedAt": next.get("released").cloned().unwrap_or(Value::Null),
        "savedAt": iso(seed.marked_at_ms),
        "source": "nuvio",
    })
}

fn iso(ms: i64) -> Value {
    chrono::DateTime::from_timestamp_millis(ms)
        .map(|value| Value::String(value.to_rfc3339()))
        .unwrap_or(Value::Null)
}

pub(super) fn continue_watching_json(args_json: &str) -> Option<String> {
    let args: Value = serde_json::from_str(args_json).ok()?;
    let now_ms = args.get("nowMs").and_then(Value::as_i64).unwrap_or(0);
    let prefs = args.get("prefs").cloned().unwrap_or_else(|| json!({}));
    let prefer_furthest = prefs
        .get("upNextFromFurthestEpisode")
        .and_then(Value::as_bool)
        .unwrap_or(true);
    let show_unaired = prefs
        .get("showUnairedNextUp")
        .and_then(Value::as_bool)
        .unwrap_or(true);
    let days_cap = prefs.get("continueWatchingDaysCap").and_then(Value::as_i64);
    let cutoff_ms = days_cap.filter(|days| *days > 0).map(|days| now_ms - days * DAY_MS);
    let hidden: HashSet<String> = args
        .get("hiddenContentIds")
        .and_then(Value::as_array)
        .map(|ids| {
            ids.iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default();

    let metas: Map<String, Value> = args
        .get("metaById")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();

    let all: Vec<Entry> = args
        .get("watchProgress")
        .and_then(Value::as_array)
        .map(|entries| entries.iter().filter_map(Entry::from_value).collect())
        .unwrap_or_default();
    let all: Vec<Entry> = all
        .into_iter()
        .filter(|entry| !hidden.contains(&entry.content_id))
        .filter(Entry::should_store)
        .collect();

    let windowed: Vec<Entry> = match cutoff_ms {
        Some(cutoff) => all
            .iter()
            .filter(|entry| entry.last_updated_ms >= cutoff)
            .cloned()
            .collect(),
        None => all.clone(),
    };

    let visible = continue_watching_entries(&windowed, HOME_MAX_RECENT_PROGRESS_ITEMS);
    let seeds = build_seed_candidates(&all, prefer_furthest);
    let recent_seeds: Vec<&Seed> = match cutoff_ms {
        Some(cutoff) => seeds
            .iter()
            .filter(|seed| seed.marked_at_ms >= cutoff)
            .collect(),
        None => seeds.iter().collect(),
    };

    let latest_completed_at: HashMap<&str, i64> = seeds
        .iter()
        .map(|seed| (seed.content_id.as_str(), seed.marked_at_ms))
        .collect();
    let suppressed: HashSet<&str> = visible
        .iter()
        .filter(|entry| is_series_for_continue_watching(&entry.content_type))
        .filter(|entry| match latest_completed_at.get(entry.content_id.as_str()) {
            Some(completed_at) => entry.last_updated_ms >= *completed_at,
            None => true,
        })
        .map(|entry| entry.content_id.as_str())
        .collect();

    let videos_for = |content_id: &str| -> Vec<Value> {
        metas
            .get(content_id)
            .and_then(|meta| meta.get("videos"))
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default()
    };

    let mut candidates: Vec<(i64, bool, Value)> = visible
        .iter()
        .map(|entry| {
            (
                entry.last_updated_ms,
                true,
                in_progress_item(entry, metas.get(&entry.content_id)),
            )
        })
        .collect();

    for seed in recent_seeds {
        if suppressed.contains(seed.content_id.as_str()) {
            continue;
        }
        let videos = videos_for(&seed.content_id);
        let Some(next) = next_released_episode_after(
            &seed.content_id,
            &videos,
            seed.season,
            seed.episode,
            now_ms,
            show_unaired,
        ) else {
            continue;
        };
        candidates.push((
            seed.marked_at_ms,
            false,
            next_up_item(seed, &next, metas.get(&seed.content_id)),
        ));
    }

    candidates.sort_by(|a, b| (b.0, b.1).cmp(&(a.0, a.1)));

    let mut seen: HashSet<String> = HashSet::new();
    let items: Vec<Value> = candidates
        .into_iter()
        .filter_map(|(_, _, item)| {
            let key = item
                .get("id")
                .and_then(Value::as_str)
                .filter(|id| !id.trim().is_empty())
                .or_else(|| item.get("lastVideoId").and_then(Value::as_str))
                .unwrap_or_default()
                .to_string();
            seen.insert(key).then_some(item)
        })
        .collect();

    serde_json::to_string(&items).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(video: &str, season: i64, episode: i64, position: f64, duration: f64, at: i64) -> Value {
        json!({
            "content_id": "tt1",
            "content_type": "series",
            "video_id": video,
            "season": season,
            "episode": episode,
            "position": position,
            "duration": duration,
            "last_watched": at,
        })
    }

    fn videos() -> Value {
        json!([
            { "id": "tt1:2:9", "season": 2, "episode": 9, "name": "Three Ghosts", "released": "2020-01-01T00:00:00Z" },
            { "id": "tt1:2:10", "season": 2, "episode": 10, "released": "2020-01-08T00:00:00Z" },
            { "id": "tt1:2:11", "season": 2, "episode": 11, "released": "2020-01-15T00:00:00Z" },
        ])
    }

    fn run(progress: Value, now: &str) -> Vec<Value> {
        let now_ms = chrono::DateTime::parse_from_rfc3339(now)
            .unwrap()
            .timestamp_millis();
        let args = json!({
            "watchProgress": progress,
            "metaById": { "tt1": { "name": "Show", "type": "series", "videos": videos() } },
            "nowMs": now_ms,
        });
        serde_json::from_str(&continue_watching_json(&args.to_string()).unwrap()).unwrap()
    }

    #[test]
    fn a_half_watched_episode_stays_a_resume_row() {
        let items = run(
            json!([entry("tt1:2:9", 2, 9, 600_000.0, 2_545_000.0, 1_600_000_000_000)]),
            "2021-01-01T00:00:00Z",
        );
        assert_eq!(items.len(), 1);
        assert_eq!(items[0]["lastVideoId"], "tt1:2:9");
        assert_eq!(items[0]["lastEpisodeName"], "Three Ghosts");
        assert_eq!(items[0]["continueWatchingBadge"], Value::Null);
    }

    #[test]
    fn a_finished_episode_becomes_the_next_one_and_stays_there() {
        let progress = json!([entry("tt1:2:9", 2, 9, 2_500_000.0, 2_545_000.0, 1_600_000_000_000)]);
        let items = run(progress.clone(), "2021-01-01T00:00:00Z");
        assert_eq!(items.len(), 1);
        assert_eq!(items[0]["lastEpisodeNumber"], 10);
        assert_eq!(items[0]["continueWatchingBadge"], "upNext");

        let again = run(progress, "2021-01-02T00:00:00Z");
        assert_eq!(again[0]["lastEpisodeNumber"], 10);
    }

    #[test]
    fn resuming_a_later_episode_suppresses_the_next_up_row() {
        let items = run(
            json!([
                entry("tt1:2:9", 2, 9, 2_500_000.0, 2_545_000.0, 1_600_000_000_000),
                entry("tt1:2:10", 2, 10, 300_000.0, 2_545_000.0, 1_600_000_100_000),
            ]),
            "2021-01-01T00:00:00Z",
        );
        assert_eq!(items.len(), 1);
        assert_eq!(items[0]["lastEpisodeNumber"], 10);
        assert_eq!(items[0]["continueWatchingBadge"], Value::Null);
    }

    #[test]
    fn a_series_with_no_episode_left_drops_out() {
        let items = run(
            json!([entry("tt1:2:11", 2, 11, 2_500_000.0, 2_545_000.0, 1_600_000_000_000)]),
            "2021-01-01T00:00:00Z",
        );
        assert!(items.is_empty());
    }

    #[test]
    fn an_unaired_next_episode_still_surfaces_by_default() {
        let items = run(
            json!([entry("tt1:2:9", 2, 9, 2_500_000.0, 2_545_000.0, 1_577_836_800_000)]),
            "2020-01-05T00:00:00Z",
        );
        assert_eq!(items[0]["lastEpisodeNumber"], 10);
    }

    #[test]
    fn the_remaining_minute_matches_a_millisecond_precise_countdown() {
        let items = run(
            json!([entry("tt1:2:9", 2, 9, 1_400.0, 1_321_000.0, 1_600_000_000_000)]),
            "2021-01-01T00:00:00Z",
        );
        let time_offset = items[0]["timeOffset"].as_i64().unwrap();
        let duration = items[0]["duration"].as_i64().unwrap();
        assert_eq!((duration - time_offset) / 60, 21);
    }

    #[test]
    fn a_barely_started_episode_is_not_stored_as_progress() {
        let items = run(
            json!([entry("tt1:2:9", 2, 9, 500.0, 2_545_000.0, 1_600_000_000_000)]),
            "2021-01-01T00:00:00Z",
        );
        assert!(items.is_empty());
    }
}
