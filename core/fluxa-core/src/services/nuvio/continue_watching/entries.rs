use super::*;

pub(super) fn freshness_key(entry: &Entry) -> (i64, i64, i64, i64, bool, String, String) {
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

pub(super) fn newest_by_progress_key(entries: Vec<Entry>) -> Vec<Entry> {
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

pub(super) fn continue_watching_entries(entries: &[Entry], limit: usize) -> Vec<Entry> {
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

pub(super) fn latest_completed_by_series(
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

pub(super) fn build_seed_candidates(entries: &[Entry], prefer_furthest: bool) -> Vec<Seed> {
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
        (b.marked_at_ms, b.season, b.episode)
            .cmp(&(a.marked_at_ms, a.season, a.episode))
            .then_with(|| a.content_id.cmp(&b.content_id))
    });
    candidates
}
