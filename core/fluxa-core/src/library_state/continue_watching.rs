use serde_json::{Value, json};

pub(crate) const UP_NEXT_POSITION_SECONDS: i64 = 0;
pub(crate) const UP_NEXT_DURATION_SECONDS: i64 = 0;

const UNSTARTED_PLACEHOLDER_DURATION_SECONDS: f64 = 86_400.0;

pub(crate) fn normalized_continue_watching_source(value: Option<&str>) -> &'static str {
    match value.map(str::trim).map(str::to_ascii_lowercase).as_deref() {
        Some("stremio") => "stremio",
        Some("nuvio") => "nuvio",
        Some("trakt") => "trakt",
        Some("simkl") => "simkl",
        Some("anilist") => "anilist",
        _ => "local",
    }
}

pub(crate) fn continue_watching_source_plan_json(args_json: &str) -> Option<String> {
    let args: Value = serde_json::from_str(args_json).ok()?;
    let source = normalized_continue_watching_source(args.get("source").and_then(Value::as_str));
    let provider = match source {
        "local" => None,
        provider => Some(provider),
    };
    let uses_local = provider.is_none();
    serde_json::to_string(&json!({
        "source": source,
        "provider": provider,
        "usesLocal": uses_local,
        "requiresMetadataEnrichment": uses_local,
    }))
    .ok()
}

pub(crate) fn is_up_next_item(item: &Value) -> bool {
    let offset = item
        .get("timeOffset")
        .and_then(Value::as_f64)
        .unwrap_or(0.0);
    let duration = item.get("duration").and_then(Value::as_f64).unwrap_or(0.0);
    if offset <= 1.0 && duration > UNSTARTED_PLACEHOLDER_DURATION_SECONDS {
        return true;
    }
    if duration <= 0.0 {
        return offset <= 1.0;
    }
    offset / duration >= 0.995
}

pub(crate) fn build_continue_watching_from_progress_json(progress_json: &str) -> Option<String> {
    let progress: serde_json::Map<String, Value> = serde_json::from_str(progress_json).ok()?;
    let mut items: Vec<Value> = progress.values()
        .filter_map(|entry| {
            let offset = entry.get("timeOffset").and_then(Value::as_f64).unwrap_or(0.0);
            let duration = entry.get("duration").and_then(Value::as_f64).unwrap_or(0.0);
            let has_video_id = entry.get("lastVideoId").and_then(Value::as_str).filter(|s| !s.is_empty()).is_some();
            // Include: items with real progress OR up-next entries (offset=0 but has lastVideoId)
            let is_resolved_up_next = entry
                .get("continueWatchingBadge")
                .and_then(Value::as_str)
                == Some("upNext");
            let include = (offset > 0.0 && duration > 0.0 && offset / duration < 0.95)
                || (has_video_id && (offset == 0.0 || is_resolved_up_next));
            if !include { return None; }
            let meta = entry.get("meta")?;
            let id = meta.get("id").and_then(Value::as_str).unwrap_or("");
            if id.is_empty() { return None; }
            Some(json!({
                "id": id,
                "name": meta.get("name").and_then(Value::as_str).unwrap_or(""),
                "type": meta.get("type").and_then(Value::as_str).unwrap_or(""),
                "poster": meta.get("poster").cloned().unwrap_or(Value::Null),
                "background": meta.get("background").cloned().unwrap_or(Value::Null),
                "logo": meta.get("logo").cloned().unwrap_or(Value::Null),
                "timeOffset": offset as i64,
                "duration": duration as i64,
                "lastVideoId": entry.get("lastVideoId").cloned().unwrap_or(Value::Null),
                "lastEpisodeName": entry.get("lastEpisodeName").cloned().unwrap_or(Value::Null),
                "lastEpisodeSeason": entry.get("lastEpisodeSeason").cloned().unwrap_or(Value::Null),
                "lastEpisodeNumber": entry.get("lastEpisodeNumber").cloned().unwrap_or(Value::Null),
                "lastEpisodeThumbnail": entry.get("lastEpisodeThumbnail").cloned().unwrap_or(Value::Null),
                "lastStreamUrl": entry.get("lastStreamUrl").cloned().unwrap_or(Value::Null),
                "lastStreamTitle": entry.get("lastStreamTitle").cloned().unwrap_or(Value::Null),
                "lastStream": entry.get("lastStream").cloned().unwrap_or(Value::Null),
                "continueWatchingBadge": entry.get("continueWatchingBadge").cloned().unwrap_or(Value::Null),
                "continueWatchingEpisodeResolved": entry.get("continueWatchingEpisodeResolved").cloned().unwrap_or(Value::Null),
                "savedAt": entry.get("savedAt").cloned().unwrap_or(Value::Null),
                "source": entry.get("source").cloned().unwrap_or(Value::Null),
            }))
        })
        .collect();
    items.sort_by(|a, b| {
        let a = a.get("savedAt").and_then(Value::as_str).unwrap_or("");
        let b = b.get("savedAt").and_then(Value::as_str).unwrap_or("");
        b.cmp(a)
    });
    serde_json::to_string(&items).ok()
}

pub(crate) fn compute_continue_watching_badges_json(
    candidates_json: &str,
    videos_by_series_json: &str,
    last_watched_json: &str,
    now_ms: i64,
) -> Option<String> {
    let mut by_id: std::collections::HashMap<String, Value> = {
        let items: Vec<Value> = serde_json::from_str(candidates_json).unwrap_or_default();
        items
            .into_iter()
            .filter_map(|item| {
                let id = item
                    .get("id")
                    .or_else(|| item.get("_id"))
                    .and_then(Value::as_str)
                    .map(str::to_string)?;
                Some((id, item))
            })
            .collect()
    };
    let videos_by_series: serde_json::Map<String, Value> =
        serde_json::from_str(videos_by_series_json).unwrap_or_default();
    let last_watched: serde_json::Map<String, Value> =
        serde_json::from_str(last_watched_json).unwrap_or_default();

    // Track which IDs came from the real CW lists vs only from lastWatchedEpisodes.
    // Candidates added only from lastWatchedEpisodes are removed when no video data
    // is available to confirm a next episode exists, preventing phantom CW entries.
    let cw_list_ids: std::collections::HashSet<String> = by_id.keys().cloned().collect();

    seed_candidates_from_last_watched(&mut by_id, &last_watched);

    let mut finished_series: Vec<String> = Vec::new();
    for (series_id, candidate) in by_id.iter_mut() {
        let next =
            match next_episode_for_candidate(series_id, candidate, &videos_by_series, &cw_list_ids)
            {
                NextEpisodeOutcome::Skip => continue,
                NextEpisodeOutcome::MarkFinished => {
                    finished_series.push(series_id.clone());
                    continue;
                }
                NextEpisodeOutcome::Found(next) => next,
            };
        let videos = videos_by_series
            .get(series_id)
            .and_then(Value::as_array)
            .map(Vec::as_slice)
            .unwrap_or(&[]);
        apply_next_episode_badge(series_id, candidate, &next, videos, now_ms);
    }

    for id in &finished_series {
        by_id.remove(id);
    }
    let mut result: Vec<Value> = by_id.into_values().collect();
    result.sort_by(|a, b| {
        let a_new = a.get("continueWatchingBadge").and_then(Value::as_str) == Some("newEpisode");
        let b_new = b.get("continueWatchingBadge").and_then(Value::as_str) == Some("newEpisode");
        if a_new != b_new {
            return if a_new {
                std::cmp::Ordering::Less
            } else {
                std::cmp::Ordering::Greater
            };
        }
        let a_time = a
            .get("savedAt")
            .or_else(|| a.get("newEpisodeReleasedAt"))
            .and_then(Value::as_str)
            .unwrap_or("");
        let b_time = b
            .get("savedAt")
            .or_else(|| b.get("newEpisodeReleasedAt"))
            .and_then(Value::as_str)
            .unwrap_or("");
        b_time.cmp(a_time)
    });
    serde_json::to_string(&result).ok()
}

// Adds a synthetic CW candidate for any series that only has a lastWatchedEpisodes
// record (no real continue-watching entry yet) so its next-episode badge still gets
// computed below; `by_id.entry(...).or_insert_with` leaves real entries untouched.
fn seed_candidates_from_last_watched(
    by_id: &mut std::collections::HashMap<String, Value>,
    last_watched: &serde_json::Map<String, Value>,
) {
    for (series_id, raw) in last_watched {
        let meta = match raw.get("meta") {
            Some(m) if m.get("type").and_then(Value::as_str) == Some("series") => m,
            _ => continue,
        };
        let record = raw;
        by_id.entry(series_id.clone()).or_insert_with(|| json!({
            "id": series_id,
            "_id": series_id,
            "type": "series",
            "name": meta.get("name").cloned().unwrap_or(Value::Null),
            "poster": meta.get("poster").cloned().unwrap_or(Value::Null),
            "background": meta.get("background").cloned().unwrap_or(Value::Null),
            "logo": meta.get("logo").cloned().unwrap_or(Value::Null),
            "lastVideoId": record.get("lastVideoId").cloned().unwrap_or(Value::Null),
            "lastEpisodeName": record.get("lastEpisodeName").cloned().unwrap_or(Value::Null),
            "lastEpisodeSeason": record.get("lastEpisodeSeason").cloned().unwrap_or(Value::Null),
            "lastEpisodeNumber": record.get("lastEpisodeNumber").cloned().unwrap_or(Value::Null),
            "lastEpisodeThumbnail": record.get("lastEpisodeThumbnail").cloned().unwrap_or(Value::Null),
            "timeOffset": UP_NEXT_POSITION_SECONDS,
            "duration": UP_NEXT_DURATION_SECONDS,
            "savedAt": record.get("watchedAt").cloned().unwrap_or(Value::Null),
        }));
    }
}

enum NextEpisodeOutcome {
    Skip,
    MarkFinished,
    Found(Value),
}

// Decides what's next for one candidate: skip it untouched, mark its series as
// finished (to be dropped from the result), or hand back the episode to advance to.
fn next_episode_for_candidate(
    series_id: &str,
    candidate: &Value,
    videos_by_series: &serde_json::Map<String, Value>,
    cw_list_ids: &std::collections::HashSet<String>,
) -> NextEpisodeOutcome {
    if candidate.get("type").and_then(Value::as_str) != Some("series") {
        return NextEpisodeOutcome::Skip;
    }
    if !is_up_next_item(candidate) {
        return NextEpisodeOutcome::Skip;
    }
    let Some(season) = candidate.get("lastEpisodeSeason").and_then(Value::as_i64) else {
        return NextEpisodeOutcome::Skip;
    };
    let Some(episode) = candidate.get("lastEpisodeNumber").and_then(Value::as_i64) else {
        return NextEpisodeOutcome::Skip;
    };
    let Some(videos) = videos_by_series.get(series_id).and_then(Value::as_array) else {
        // No video data available. If this entry exists only because of
        // lastWatchedEpisodes (not from any real CW list), conservatively
        // remove it — we cannot confirm a next episode exists. It will
        // reappear on the next home load once the addon responds.
        return if !cw_list_ids.contains(series_id) {
            NextEpisodeOutcome::MarkFinished
        } else {
            NextEpisodeOutcome::Skip
        };
    };
    let stored_badge = candidate
        .get("continueWatchingBadge")
        .and_then(Value::as_str);
    if stored_badge == Some("upNext")
        && candidate
            .get("continueWatchingEpisodeResolved")
            .and_then(Value::as_bool)
            .unwrap_or(false)
    {
        return NextEpisodeOutcome::Skip;
    }
    let stored_video_id = candidate
        .get("lastVideoId")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();

    // When the stored badge is scheduledEpisode, lastEpisodeNumber already points to the
    // scheduled episode itself. Re-check that same episode rather than advancing past it.
    let next = if stored_badge == Some("scheduledEpisode") {
        videos
            .iter()
            .find(|v| {
                let vid = v
                    .get("id")
                    .or_else(|| v.get("_id"))
                    .and_then(Value::as_str)
                    .unwrap_or("");
                vid == stored_video_id
            })
            .cloned()
            .or_else(|| first_episode_after(videos, season, episode))
    } else {
        first_episode_after(videos, season, episode)
    };

    // No next episode and we have real video data — the series is fully watched.
    // Remove it from Continue Watching instead of leaving a zombie entry.
    match next {
        Some(v) => NextEpisodeOutcome::Found(v),
        None => NextEpisodeOutcome::MarkFinished,
    }
}

// Computes the badge (upNext / newEpisode / scheduledEpisode) for advancing `candidate`
// to `next`, and rewrites `candidate` in place to point at that episode.
fn apply_next_episode_badge(
    series_id: &str,
    candidate: &mut Value,
    next: &Value,
    videos: &[Value],
    now_ms: i64,
) {
    let existing_video_id = candidate
        .get("lastVideoId")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    let next_id = next
        .get("id")
        .or_else(|| next.get("_id"))
        .and_then(Value::as_str)
        .unwrap_or(&existing_video_id)
        .to_string();
    if !is_up_next_item(candidate) && existing_video_id != next_id {
        return;
    }
    let is_new_target = existing_video_id != next_id;
    let is_released = is_episode_released(next, now_ms);
    let existing_badge = if !is_new_target {
        candidate
            .get("continueWatchingBadge")
            .and_then(Value::as_str)
            .map(str::to_string)
    } else {
        None
    };

    let badge = if !is_released {
        "scheduledEpisode"
    } else if existing_badge.as_deref() == Some("scheduledEpisode") {
        "newEpisode"
    } else if let Some(b) = existing_badge.as_deref() {
        b
    } else {
        let watched_at = candidate
            .get("savedAt")
            .and_then(Value::as_str)
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.timestamp_millis())
            .unwrap_or(now_ms);
        let next_released_at = next
            .get("released")
            .and_then(Value::as_str)
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.timestamp_millis())
            .unwrap_or(0);
        let was_released_when_watched =
            next.get("released").is_none() || next_released_at <= watched_at;
        if was_released_when_watched {
            "upNext"
        } else {
            "newEpisode"
        }
    }
    .to_string();

    let released_str = next
        .get("released")
        .and_then(Value::as_str)
        .map(str::to_string)
        .unwrap_or_else(|| chrono::Utc::now().to_rfc3339());
    let saved_at_new = if is_new_target && badge == "newEpisode" {
        Value::String(chrono::Utc::now().to_rfc3339())
    } else {
        candidate.get("savedAt").cloned().unwrap_or(Value::Null)
    };

    let unwatched_ahead = if badge == "scheduledEpisode" {
        0
    } else {
        let next_season = next.get("season").and_then(Value::as_i64).unwrap_or(0);
        let next_episode = next
            .get("episode")
            .or_else(|| next.get("number"))
            .and_then(Value::as_i64)
            .unwrap_or(0);
        count_released_after(videos, next_season, next_episode, now_ms)
    };

    *candidate = json!({
        "id": series_id,
        "_id": series_id,
        "type": "series",
        "name": candidate.get("name").cloned().unwrap_or(Value::Null),
        "poster": candidate.get("poster").cloned().unwrap_or(Value::Null),
        "background": candidate.get("background").cloned().unwrap_or(Value::Null),
        "logo": candidate.get("logo").cloned().unwrap_or(Value::Null),
        "timeOffset": UP_NEXT_POSITION_SECONDS,
        "duration": UP_NEXT_DURATION_SECONDS,
        "lastVideoId": next_id,
        "lastEpisodeName": next.get("name").or_else(|| next.get("title")).cloned().unwrap_or(Value::Null),
        "lastEpisodeSeason": next.get("season").cloned().unwrap_or(Value::Null),
        "lastEpisodeNumber": next.get("episode").or_else(|| next.get("number")).cloned().unwrap_or(Value::Null),
        "lastEpisodeThumbnail": next.get("thumbnail")
            .filter(|v| v.as_str().map_or(!v.is_null(), |s| !s.trim().is_empty()))
            .cloned()
            .or_else(|| if !is_new_target {
                candidate.get("lastEpisodeThumbnail").cloned().filter(|v| v.as_str().map_or(!v.is_null(), |s| !s.trim().is_empty()))
            } else {
                None
            })
            .unwrap_or(Value::Null),
        "continueWatchingBadge": badge,
        "newEpisodeReleasedAt": released_str,
        "savedAt": saved_at_new,
        "source": candidate.get("source").cloned().unwrap_or(Value::Null),
        "reason": candidate.get("reason").cloned().unwrap_or(Value::Null),
        "unwatchedAhead": unwatched_ahead,
    });
}

fn count_released_after(videos: &[Value], season: i64, episode: i64, now_ms: i64) -> i64 {
    videos
        .iter()
        .filter(|v| {
            let vs = v.get("season").and_then(Value::as_i64).unwrap_or(0);
            let ve = v
                .get("episode")
                .or_else(|| v.get("number"))
                .and_then(Value::as_i64)
                .unwrap_or(0);
            (vs > season || (vs == season && ve > episode)) && is_episode_released(v, now_ms)
        })
        .count() as i64
}

fn first_episode_after(videos: &[Value], season: i64, episode: i64) -> Option<Value> {
    let mut candidates: Vec<&Value> = videos
        .iter()
        .filter(|v| {
            let vs = v.get("season").and_then(Value::as_i64).unwrap_or(0);
            let ve = v
                .get("episode")
                .or_else(|| v.get("number"))
                .and_then(Value::as_i64)
                .unwrap_or(0);
            vs > season || (vs == season && ve > episode)
        })
        .collect();
    candidates.sort_by(|a, b| {
        let as_ = a.get("season").and_then(Value::as_i64).unwrap_or(0);
        let bs = b.get("season").and_then(Value::as_i64).unwrap_or(0);
        if as_ != bs {
            return as_.cmp(&bs);
        }
        let ae = a
            .get("episode")
            .or_else(|| a.get("number"))
            .and_then(Value::as_i64)
            .unwrap_or(0);
        let be = b
            .get("episode")
            .or_else(|| b.get("number"))
            .and_then(Value::as_i64)
            .unwrap_or(0);
        ae.cmp(&be)
    });
    candidates.first().map(|v| (*v).clone())
}

pub(crate) fn is_episode_released(video: &Value, now_ms: i64) -> bool {
    let released = match video.get("released").and_then(Value::as_str) {
        Some(s) => s,
        None => return true,
    };
    match chrono::DateTime::parse_from_rfc3339(released) {
        Ok(dt) => dt.timestamp_millis() <= now_ms,
        Err(_) => true,
    }
}

/// Given a library JSON and a set of just-watched video IDs, update `lastWatchedEpisodes`.
/// Returns the updated library as JSON.
pub(crate) fn remember_last_watched_episodes_json(
    lib_json: &str,
    watched_ids_json: &str,
) -> String {
    let mut lib: Value = serde_json::from_str(lib_json).unwrap_or(json!({}));
    let watched_ids: std::collections::HashSet<String> = serde_json::from_str(watched_ids_json)
        .ok()
        .and_then(|v: Value| {
            v.as_array().map(|arr| {
                arr.iter()
                    .filter_map(|s| s.as_str().map(str::to_string))
                    .collect()
            })
        })
        .unwrap_or_default();
    let progress = lib
        .get("progress")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();
    let mut last_watched = lib
        .get("lastWatchedEpisodes")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();
    for (series_id, raw) in &progress {
        let video_id = raw.get("lastVideoId").and_then(Value::as_str).unwrap_or("");
        if video_id.is_empty() || !watched_ids.contains(video_id) {
            continue;
        }
        let meta = match raw.get("meta") {
            Some(m) if m.get("type").and_then(Value::as_str) == Some("series") => m,
            _ => continue,
        };
        last_watched.insert(series_id.clone(), json!({
            "meta": meta,
            "lastVideoId": video_id,
            "lastEpisodeName": raw.get("lastEpisodeName").cloned().unwrap_or(Value::Null),
            "lastEpisodeSeason": raw.get("lastEpisodeSeason").cloned().unwrap_or(Value::Null),
            "lastEpisodeNumber": raw.get("lastEpisodeNumber").cloned().unwrap_or(Value::Null),
            "lastEpisodeThumbnail": raw.get("lastEpisodeThumbnail").cloned().unwrap_or(Value::Null),
            "watchedAt": chrono::Utc::now().to_rfc3339(),
        }));
    }
    if let Some(obj) = lib.as_object_mut() {
        obj.insert(
            "lastWatchedEpisodes".to_string(),
            Value::Object(last_watched),
        );
    }
    serde_json::to_string(&lib).unwrap_or_else(|_| lib_json.to_string())
}

/// Returns the next episode after (current_season, current_episode).
/// If released_only is true, episodes whose `released` date is in the future
/// (relative to now_ms) are excluded.
pub(crate) fn resolve_next_episode_json(
    videos_json: &str,
    current_season: i64,
    current_episode: i64,
    now_ms: i64,
    released_only: bool,
) -> Option<String> {
    let videos: Vec<Value> = serde_json::from_str(videos_json).ok()?;
    let filtered: Vec<&Value> = videos
        .iter()
        .filter(|v| !released_only || is_episode_released(v, now_ms))
        .collect();
    let next = first_episode_after(
        &filtered.into_iter().cloned().collect::<Vec<_>>(),
        current_season,
        current_episode,
    )?;
    serde_json::to_string(&next).ok()
}

pub(crate) fn resolve_next_after_watched_json(request_json: &str) -> Option<String> {
    let request: Value = serde_json::from_str(request_json).ok()?;
    let watched = request.get("watchedEpisodes")?.as_array()?;
    let last = watched.iter().max_by_key(|episode| {
        episode.get("season").and_then(Value::as_i64).unwrap_or(1) * 10_000
            + episode
                .get("episode")
                .or_else(|| episode.get("number"))
                .and_then(Value::as_i64)
                .unwrap_or(0)
    })?;
    resolve_next_episode_json(
        &request.get("videos")?.to_string(),
        last.get("season").and_then(Value::as_i64).unwrap_or(1),
        last.get("episode")
            .or_else(|| last.get("number"))
            .and_then(Value::as_i64)
            .unwrap_or(0),
        request.get("nowMs").and_then(Value::as_i64).unwrap_or(0),
        false,
    )
}

pub(crate) fn next_progress_info_plan_json(request_json: &str) -> Option<String> {
    let request: Value = serde_json::from_str(request_json).ok()?;
    let next: Value = serde_json::from_str(&resolve_next_after_watched_json(request_json)?).ok()?;
    let content_id = request.get("contentId")?.as_str()?.trim();
    if content_id.is_empty() {
        return None;
    }
    serde_json::to_string(&json!({
        "contentId": content_id,
        "contentType": request.get("contentType").and_then(Value::as_str).unwrap_or("series"),
        "videoId": next.get("id")?,
        "positionSeconds": UP_NEXT_POSITION_SECONDS,
        "durationSeconds": UP_NEXT_DURATION_SECONDS,
        "lastWatched": request.get("nowMs").and_then(Value::as_i64).unwrap_or(0),
        "season": next.get("season"),
        "episode": next.get("episode").or_else(|| next.get("number")),
    }))
    .ok()
}

/// Formats a "S1:E2 Episode Name" line from the episode progress fields.
/// Falls back to parsing season/episode from lastVideoId when the explicit
/// season/episode numbers are absent.
pub(crate) fn format_episode_line_json(
    last_episode_name: Option<&str>,
    last_episode_season: Option<i64>,
    last_episode_number: Option<i64>,
    last_video_id: Option<&str>,
) -> String {
    let mut season = last_episode_season;
    let mut episode = last_episode_number;

    if (season.is_none() || episode.is_none())
        && let Some(id) = last_video_id.filter(|id| !id.is_empty())
    {
        let mut parts = id.rsplitn(3, ':');
        if let (Some(episode_part), Some(season_part), Some(_)) =
            (parts.next(), parts.next(), parts.next())
            && let (Ok(s), Ok(e)) = (season_part.parse::<i64>(), episode_part.parse::<i64>())
            && s > 0
            && e > 0
        {
            if season.is_none() {
                season = Some(s);
            }
            if episode.is_none() {
                episode = Some(e);
            }
        }
    }

    let code = match (season, episode) {
        (Some(s), Some(e)) => format!("S{s}:E{e}"),
        _ => String::new(),
    };
    let name = last_episode_name.map(str::trim).unwrap_or("").to_string();
    [code, name]
        .into_iter()
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}
