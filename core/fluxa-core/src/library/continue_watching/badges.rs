use serde_json::Value;

// Adds a synthetic CW candidate for any series that only has a lastWatchedEpisodes
// record (no real continue-watching entry yet) so its next-episode badge still gets
// computed below; `by_id.entry(...).or_insert_with` leaves real entries untouched.

// Decides what's next for one candidate: skip it untouched, mark its series as
// finished (to be dropped from the result), or hand back the episode to advance to.

// Computes the badge (upNext / newEpisode / scheduledEpisode) for advancing `candidate`
// to `next`, and rewrites `candidate` in place to point at that episode.

pub(super) fn first_episode_after(videos: &[Value], season: i64, episode: i64) -> Option<Value> {
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
