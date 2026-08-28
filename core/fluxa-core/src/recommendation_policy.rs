use serde_json::{json, Value};
use std::collections::HashSet;

fn candidate_id(value: &Value) -> Option<&str> {
    value
        .get("id")
        .or_else(|| value.get("imdbId"))
        .and_then(Value::as_str)
}

/// Curates recommendations only after terminal playback. The network provider
/// owns relevance and ordering; this policy only owns playback safety and the
/// user's local watch state.
pub(crate) fn terminal_recommendation_plan_json(args_json: &str) -> Option<String> {
    let args: Value = serde_json::from_str(args_json).ok()?;
    let current = args.get("current")?;
    let candidates = args.get("candidates")?.as_array()?;
    let has_next_episode = args
        .get("hasNextEpisode")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let limit = args
        .get("limit")
        .and_then(Value::as_u64)
        .unwrap_or(20)
        .clamp(1, 50) as usize;
    let current_id = candidate_id(current).unwrap_or("");
    let watched: HashSet<&str> = args
        .get("watchedIds")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .collect();
    let mut seen = HashSet::new();
    let items = if has_next_episode {
        Vec::new()
    } else {
        candidates
            .iter()
            .filter(|candidate| {
                let Some(id) = candidate_id(candidate) else {
                    return false;
                };
                !id.is_empty() && id != current_id && !watched.contains(id) && seen.insert(id)
            })
            .take(limit)
            .cloned()
            .collect()
    };
    Some(
        json!({
            "showRecommendations": !items.is_empty(),
            "items": items,
        })
        .to_string(),
    )
}

#[cfg(test)]
mod tests {
    use super::terminal_recommendation_plan_json;
    use serde_json::Value;

    fn plan(input: Value) -> Value {
        serde_json::from_str(&terminal_recommendation_plan_json(&input.to_string()).unwrap())
            .unwrap()
    }

    #[test]
    fn ongoing_series_does_not_show_recommendations() {
        let result = plan(serde_json::json!({
            "current": {"id": "show-1", "type": "series"},
            "hasNextEpisode": true,
            "candidates": [{"id": "show-2", "type": "series"}]
        }));
        assert_eq!(result["showRecommendations"], false);
        assert!(result["items"].as_array().unwrap().is_empty());
    }

    #[test]
    fn recommendations_preserve_provider_types_but_exclude_watched_and_current() {
        let result = plan(serde_json::json!({
            "current": {"id": "movie-1", "type": "movie"},
            "watchedIds": ["movie-2"],
            "candidates": [
                {"id": "movie-1", "type": "movie"},
                {"id": "movie-2", "type": "movie"},
                {"id": "show-1", "type": "series"},
                {"id": "movie-3", "type": "movie"}
            ]
        }));
        assert_eq!(result["items"].as_array().unwrap().len(), 2);
        assert_eq!(result["items"][0]["id"], "show-1");
        assert_eq!(result["items"][1]["id"], "movie-3");
    }
}
