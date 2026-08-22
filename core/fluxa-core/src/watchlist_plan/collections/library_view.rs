use super::air_date::air_time;
use super::helpers::timestamp;
use serde_json::{Value, json};

pub(crate) fn library_view_plan_json(args_json: &str) -> Option<String> {
    let args: Value = serde_json::from_str(args_json).ok()?;
    let list = |name: &str| {
        args.get(name)
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default()
    };
    let watchlist = list("watchlist");
    let watching = list("watching");
    let favorites = list("favorites");
    let mut completed = list("completed");
    let mut dropped = list("dropped");
    completed.sort_by(|a, b| status_changed_at(b).cmp(status_changed_at(a)));
    dropped.sort_by(|a, b| status_changed_at(b).cmp(status_changed_at(a)));
    let progress: Vec<Value> = args
        .get("progress")
        .and_then(Value::as_object)
        .map(|values| values.values().cloned().collect())
        .unwrap_or_default();
    let all = unique_items(
        watchlist
            .iter()
            .chain(&watching)
            .chain(&completed)
            .chain(&dropped)
            .chain(&progress),
    );
    let mut airing = unique_items(watching.iter().chain(&watchlist));
    airing.retain(|item| {
        item.get("nextEpisodeAirDate")
            .is_some_and(|value| !value.is_null())
            || item
                .get("newEpisodeReleasedAt")
                .is_some_and(|value| !value.is_null())
            || matches!(
                item.get("continueWatchingBadge").and_then(Value::as_str),
                Some("newEpisode" | "scheduledEpisode")
            )
    });
    airing.sort_by_key(air_time);
    let mut rated = all.clone();
    rated.retain(|item| rating(item) >= 7.5);
    rated.sort_by(|a, b| {
        rating(b)
            .partial_cmp(&rating(a))
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let watching_ids: std::collections::HashSet<&str> = watching
        .iter()
        .filter_map(|item| item.get("id").and_then(Value::as_str))
        .collect();
    let mut history = all;
    history.retain(|item| {
        item.get("id")
            .and_then(Value::as_str)
            .is_none_or(|id| !watching_ids.contains(id))
            && playback_time(item) > 0
    });
    history.sort_by_key(|item| std::cmp::Reverse(playback_time(item)));
    let tab = args.get("tab").and_then(Value::as_str).unwrap_or("");
    let mut items = match tab {
        "watchlist" => watchlist.clone(),
        "watching" => watching.clone(),
        "completed" => completed.clone(),
        "dropped" => dropped.clone(),
        "airing" => airing.clone(),
        "rated" => rated.clone(),
        "history" => history.clone(),
        "favorites" => favorites.clone(),
        _ => Vec::new(),
    };
    let tab_items = items.clone();
    let query = args
        .get("query")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim()
        .to_ascii_lowercase();
    if !query.is_empty() {
        items.retain(|item| {
            item.get("name")
                .and_then(Value::as_str)
                .is_some_and(|name| name.to_ascii_lowercase().contains(&query))
        });
    }
    match args
        .get("sortBy")
        .and_then(Value::as_str)
        .unwrap_or("default")
    {
        "title" => items.sort_by(|a, b| name(a).cmp(name(b))),
        "rating" => items.sort_by(|a, b| {
            rating(b)
                .partial_cmp(&rating(a))
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| name(a).cmp(name(b)))
        }),
        _ => {}
    }
    serde_json::to_string(&json!({"completed": completed, "dropped": dropped, "smartLists": {"airing": airing, "rated": rated, "history": history}, "tabItems": tab_items, "items": items})).ok()
}

fn unique_items<'a>(items: impl Iterator<Item = &'a Value>) -> Vec<Value> {
    let mut seen = std::collections::HashSet::new();
    items
        .filter(|item| {
            item.get("id")
                .and_then(Value::as_str)
                .is_some_and(|id| !id.is_empty() && seen.insert(id))
        })
        .cloned()
        .collect()
}

fn name(item: &Value) -> &str {
    item.get("name").and_then(Value::as_str).unwrap_or("")
}
fn rating(item: &Value) -> f64 {
    item.get("imdbRating")
        .and_then(Value::as_f64)
        .unwrap_or(0.0)
}
fn status_changed_at(item: &Value) -> &str {
    item.get("statusChangedAt")
        .and_then(Value::as_str)
        .unwrap_or("")
}
fn playback_time(item: &Value) -> i64 {
    let last_watched = timestamp(item, "lastWatchedAt");
    if last_watched > 0 {
        return last_watched;
    }
    let has_playback_state = item
        .get("timeOffset")
        .and_then(Value::as_i64)
        .is_some_and(|value| value > 0)
        || item
            .get("lastVideoId")
            .and_then(Value::as_str)
            .is_some_and(|value| !value.is_empty());
    if has_playback_state {
        timestamp(item, "savedAt")
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn history_excludes_active_watching_and_non_playback_changes() {
        let plan = library_view_plan_json(
            &json!({
                "watchlist": [
                    {"id": "saved", "savedAt": "2026-07-01T00:00:00Z"},
                    {"id": "played", "lastVideoId": "played:1:1", "savedAt": "2026-07-02T00:00:00Z"}
                ],
                "watching": [
                    {"id": "active", "lastVideoId": "active:1:1", "savedAt": "2026-07-03T00:00:00Z"}
                ],
                "completed": [],
                "dropped": [],
                "progress": {},
                "tab": "history"
            })
            .to_string(),
        )
        .unwrap();
        let items = serde_json::from_str::<Value>(&plan).unwrap()["items"]
            .as_array()
            .unwrap()
            .clone();

        assert_eq!(
            items,
            vec![
                json!({"id": "played", "lastVideoId": "played:1:1", "savedAt": "2026-07-02T00:00:00Z"})
            ]
        );
    }
}
