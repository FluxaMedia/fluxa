use super::*;

pub(super) fn episode_released_at(video: &Value) -> Option<i64> {
    video
        .get("released")
        .or_else(|| video.get("firstAired"))
        .and_then(Value::as_str)
        .and_then(|value| chrono::DateTime::parse_from_rfc3339(value).ok())
        .map(|value| value.timestamp_millis())
}

pub(super) fn should_surface_next_episode(
    watched_season: i64,
    candidate: &Value,
    now_ms: i64,
    show_unaired: bool,
) -> bool {
    let candidate_season = candidate.get("season").and_then(Value::as_i64);
    let season_rollover =
        normalize_season(candidate_season) != normalize_season(Some(watched_season));
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

pub(super) fn sorted_episodes(videos: &[Value]) -> Vec<Value> {
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

pub(super) fn next_released_episode_after(
    content_id: &str,
    videos: &[Value],
    season: i64,
    episode: i64,
    now_ms: i64,
    show_unaired: bool,
) -> Option<Value> {
    let sorted = sorted_episodes(videos);
    let watched_video_id = format!("{content_id}:{season}:{episode}");
    let identity = |video: &Value| match (
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
    };
    let mut watched_index = sorted
        .iter()
        .position(|video| identity(video) == watched_video_id);

    if watched_index.is_none() && season == 1 && episode > 0 {
        let main: Vec<usize> = sorted
            .iter()
            .enumerate()
            .filter(|(_, video)| normalize_season(video.get("season").and_then(Value::as_i64)) > 0)
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
