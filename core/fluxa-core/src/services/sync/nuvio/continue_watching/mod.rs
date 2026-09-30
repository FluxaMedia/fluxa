use serde_json::{Map, Value, json};
use std::collections::{HashMap, HashSet};

mod entries;
mod episodes;
mod items;

pub(crate) use entries::*;
pub(crate) use episodes::*;
pub(crate) use items::*;

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
        if self
            .percent
            .is_some_and(|percent| percent >= COMPLETION_PERCENT)
        {
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

pub(crate) fn continue_watching_json(args_json: &str) -> Option<String> {
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
    let cutoff_ms = days_cap
        .filter(|days| *days > 0)
        .map(|days| now_ms - days * DAY_MS);
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
        .filter(
            |entry| match latest_completed_at.get(entry.content_id.as_str()) {
                Some(completed_at) => entry.last_updated_ms >= *completed_at,
                None => true,
            },
        )
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

    let mut candidates: Vec<(i64, bool, String, Value)> = visible
        .iter()
        .map(|entry| {
            (
                entry.last_updated_ms,
                true,
                entry.content_id.clone(),
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
            seed.content_id.clone(),
            next_up_item(seed, &next, metas.get(&seed.content_id)),
        ));
    }

    candidates.sort_by(|a, b| (b.0, b.1).cmp(&(a.0, a.1)).then_with(|| a.2.cmp(&b.2)));

    let mut seen: HashSet<String> = HashSet::new();
    let items: Vec<Value> = candidates
        .into_iter()
        .filter_map(|(_, _, _, item)| {
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
mod tests;
