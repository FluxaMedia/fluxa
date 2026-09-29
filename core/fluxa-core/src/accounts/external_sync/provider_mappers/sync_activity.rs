use serde_json::{Value, json};

fn activity_at(activities: Option<&Value>, group: &str, field: &str) -> Option<String> {
    activities?
        .get(group)?
        .get(field)
        .and_then(Value::as_str)
        .map(str::to_string)
}

pub(crate) fn trakt_activity_diff_json(request_json: &str) -> Option<String> {
    let request: Value = serde_json::from_str(request_json).ok()?;
    let current = request.get("current").filter(|v| !v.is_null());
    let previous = request.get("previous").filter(|v| !v.is_null());
    let has = |key: &str| request.get(key).and_then(Value::as_bool).unwrap_or(false);
    let changed = |group: &str, field: &str| {
        current.is_none()
            || activity_at(current, group, field) != activity_at(previous, group, field)
    };

    let result = json!({
        "playbackChanged": !has("hasPlayback") || changed("movies", "paused_at") || changed("episodes", "paused_at"),
        "watchlistMoviesChanged": !has("hasWatchlistMovies") || changed("movies", "watchlisted_at"),
        "watchlistShowsChanged": !has("hasWatchlistShows") || changed("shows", "watchlisted_at"),
        "watchedMoviesChanged": !has("hasWatchedMovies") || changed("movies", "watched_at"),
        "watchedShowsChanged": !has("hasWatchedShows") || changed("episodes", "watched_at"),
    });
    serde_json::to_string(&result).ok()
}
