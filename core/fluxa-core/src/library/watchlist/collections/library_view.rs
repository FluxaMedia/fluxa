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
    let on_hold = list("onHold");
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
            .chain(&on_hold)
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
    let source = args
        .get("source")
        .and_then(Value::as_str)
        .unwrap_or("local");
    let capabilities = source_capabilities(source);
    let requested = args.get("tab").and_then(Value::as_str).unwrap_or("");
    let tab = if capabilities.statuses.is_empty() {
        "watchlist"
    } else if capabilities.statuses.contains(&requested) {
        requested
    } else {
        capabilities.statuses[0]
    };
    let mut items = match tab {
        "watchlist" => watchlist.clone(),
        "watching" => watching.clone(),
        "completed" => completed.clone(),
        "dropped" => dropped.clone(),
        "hold" => on_hold.clone(),
        "airing" => airing.clone(),
        "rated" => rated.clone(),
        "history" => history.clone(),
        "favorites" => favorites.clone(),
        _ => Vec::new(),
    };
    let tab_items = items.clone();
    let types: Vec<&str> = capabilities
        .types
        .iter()
        .copied()
        .filter(|kind| tab_items.iter().any(|item| content_kind(item) == *kind))
        .collect();
    let kind = args.get("type").and_then(Value::as_str).unwrap_or("all");
    if matches!(kind, "movie" | "series" | "anime") {
        items.retain(|item| content_kind(item) == kind);
    }
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
        "title_desc" => items.sort_by(|a, b| name(b).cmp(name(a))),
        "oldest" => items.reverse(),
        "rating" => items.sort_by(|a, b| {
            rating(b)
                .partial_cmp(&rating(a))
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| name(a).cmp(name(b)))
        }),
        _ => {}
    }
    serde_json::to_string(&json!({"completed": completed, "dropped": dropped, "smartLists": {"airing": airing, "rated": rated, "history": history}, "tabItems": tab_items, "types": types, "statuses": capabilities.statuses, "sorts": capabilities.sorts, "items": items})).ok()
}

struct SourceCapabilities {
    statuses: &'static [&'static str],
    types: &'static [&'static str],
    sorts: &'static [&'static str],
}

fn source_capabilities(source: &str) -> SourceCapabilities {
    match source {
        "trakt" => SourceCapabilities {
            statuses: &["watchlist", "watching", "completed", "favorites"],
            types: &["movie", "series"],
            sorts: &["tracker", "title", "rating"],
        },
        "simkl" => SourceCapabilities {
            statuses: &["watchlist", "watching", "completed", "hold", "dropped"],
            types: &["movie", "series", "anime"],
            sorts: &["tracker", "title", "rating"],
        },
        "mdblist" => SourceCapabilities {
            statuses: &["watchlist", "watching", "completed", "dropped"],
            types: &["movie", "series"],
            sorts: &["tracker", "title", "rating"],
        },
        "anilist" => SourceCapabilities {
            statuses: &["watchlist", "watching", "completed", "dropped"],
            types: &["anime"],
            sorts: &["tracker", "title", "rating"],
        },
        "stremio" => SourceCapabilities {
            statuses: &["watchlist", "watching", "completed"],
            types: &["movie", "series", "anime"],
            sorts: &["tracker", "title"],
        },
        _ => SourceCapabilities {
            statuses: &[],
            types: &["movie", "series", "anime"],
            sorts: &["recent", "oldest", "title", "title_desc"],
        },
    }
}

fn content_kind(item: &Value) -> &'static str {
    let kind = item.get("type").and_then(Value::as_str).unwrap_or("");
    let id = item.get("id").and_then(Value::as_str).unwrap_or("");
    if item.get("source").and_then(Value::as_str) == Some("simkl") {
        return if item.get("isAnime").and_then(Value::as_bool) == Some(true) {
            "anime"
        } else if kind == "movie" {
            "movie"
        } else {
            "series"
        };
    }
    if kind == "anime"
        || item.get("isAnime").and_then(Value::as_bool) == Some(true)
        || crate::catalog::anime::should_attempt_anime_tracking(item)
        || ["kitsu:", "mal:", "anilist:", "anidb:"]
            .iter()
            .any(|prefix| id.starts_with(prefix))
    {
        "anime"
    } else if kind == "movie" {
        "movie"
    } else {
        "series"
    }
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
    fn simkl_items_are_filtered_by_simkl_classification() {
        let plan = library_view_plan_json(
            &json!({
                "source": "simkl",
                "watchlist": [
                    {"id": "tt1", "type": "series", "source": "simkl", "isAnime": false, "genres": ["Anime"]},
                    {"id": "mal:1", "type": "series", "source": "simkl", "isAnime": true}
                ],
                "tab": "watchlist",
                "type": "anime"
            })
            .to_string(),
        )
        .unwrap();
        let plan = serde_json::from_str::<Value>(&plan).unwrap();

        assert_eq!(plan["items"].as_array().unwrap().len(), 1);
        assert_eq!(plan["items"][0]["id"], "mal:1");
    }

    #[test]
    fn type_filter_keeps_anime_apart_and_lists_available_types() {
        let plan = library_view_plan_json(
            &json!({
                "watchlist": [
                    {"id": "tt1", "type": "movie"},
                    {"id": "tt2", "type": "series"},
                    {"id": "kitsu:9", "type": "series"},
                    {"id": "tt3", "type": "series", "genres": ["Animation", "Anime"]}
                ],
                "tab": "watchlist",
                "type": "anime"
            })
            .to_string(),
        )
        .unwrap();
        let plan = serde_json::from_str::<Value>(&plan).unwrap();

        assert_eq!(plan["items"].as_array().unwrap().len(), 2);
        assert_eq!(plan["types"], json!(["movie", "series", "anime"]));
    }
}
