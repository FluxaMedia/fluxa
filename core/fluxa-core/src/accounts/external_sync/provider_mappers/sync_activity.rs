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

pub(crate) fn simkl_resource_sync_plan_json(request_json: &str) -> Option<String> {
    let request: Value = serde_json::from_str(request_json).ok()?;
    let current = request.get("current").filter(|v| !v.is_null());
    let previous = request.get("previous").filter(|v| !v.is_null());
    let resources = request.get("resources")?.as_array()?;

    let plans: Vec<Value> = resources
        .iter()
        .map(|resource| {
            let key = resource.get("key").and_then(Value::as_str).unwrap_or("");
            let res_type = resource.get("type").and_then(Value::as_str).unwrap_or("");
            let status = resource.get("status").and_then(Value::as_str).unwrap_or("");
            let has_cached = resource
                .get("hasCached")
                .and_then(Value::as_bool)
                .unwrap_or(false);

            let previous_activity = activity_at(previous, res_type, status);
            let current_activity = activity_at(current, res_type, status);
            let force_full = !has_cached
                || activity_at(previous, res_type, "removed_from_list")
                    != activity_at(current, res_type, "removed_from_list");

            let action = if !force_full && previous_activity == current_activity {
                "unchanged"
            } else if force_full {
                "full"
            } else {
                "delta"
            };
            let date_from = if action == "delta" {
                previous_activity.clone()
            } else {
                None
            };

            json!({ "key": key, "action": action, "dateFrom": date_from })
        })
        .collect();

    serde_json::to_string(&plans).ok()
}
