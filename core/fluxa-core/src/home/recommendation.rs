use serde_json::{Value, json};
use std::collections::HashSet;

fn candidate_id(value: &Value) -> Option<&str> {
    value
        .get("id")
        .or_else(|| value.get("imdbId"))
        .and_then(Value::as_str)
}

pub(crate) fn terminal_recommendation_eligibility_json(args_json: &str) -> Option<String> {
    let args: Value = serde_json::from_str(args_json).ok()?;
    if args.get("contentType").and_then(Value::as_str) != Some("series") {
        return Some(json!({"eligible": true, "hasNextEpisode": false}).to_string());
    }
    let videos = args.get("videos")?.as_array()?;
    let current_season = args.get("currentSeason")?.as_i64()?;
    let current_episode = args.get("currentEpisode")?.as_i64()?;
    let now_ms = args.get("nowMs")?.as_i64()?;
    let next = crate::library::state::resolve_next_episode_json(
        &Value::Array(videos.clone()).to_string(),
        current_season,
        current_episode,
        now_ms,
        true,
    )
    .is_some();
    Some(json!({"eligible": !next, "hasNextEpisode": next}).to_string())
}

pub(crate) fn recommendation_outro_plan_json(args_json: &str) -> Option<String> {
    let args: Value = serde_json::from_str(args_json).ok()?;
    let position = args.get("positionSeconds")?.as_f64()?;
    let duration = args.get("durationSeconds")?.as_f64()?;
    let threshold = args
        .get("thresholdPercent")
        .and_then(Value::as_f64)
        .unwrap_or(85.0)
        .clamp(0.0, 100.0);
    let outro_start = args
        .get("outroStartSeconds")
        .and_then(Value::as_f64)
        .filter(|value| value.is_finite() && *value >= 0.0);
    let valid = position.is_finite() && duration.is_finite() && duration > 0.0;
    let reached = valid
        && (position / duration * 100.0 >= threshold
            || outro_start.is_some_and(|start| position >= start));
    Some(json!({"shouldShow": reached && !args.get("alreadyShown").and_then(Value::as_bool).unwrap_or(false)}).to_string())
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
    use super::{
        recommendation_outro_plan_json, terminal_recommendation_eligibility_json,
        terminal_recommendation_plan_json,
    };
    use serde_json::Value;

    fn plan(input: Value) -> Value {
        serde_json::from_str(&terminal_recommendation_plan_json(&input.to_string()).unwrap())
            .unwrap()
    }

    #[test]
    fn eligibility_requires_terminal_series_episode() {
        let result: Value = serde_json::from_str(
            &terminal_recommendation_eligibility_json(
                &serde_json::json!({
                    "contentType": "series",
                    "videos": [
                        {"id": "s1e1", "season": 1, "episode": 1},
                        {"id": "s1e2", "season": 1, "episode": 2}
                    ],
                    "currentSeason": 1,
                    "currentEpisode": 1,
                    "nowMs": 1_800_000_000_000_i64
                })
                .to_string(),
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(result["eligible"], false);
        assert_eq!(result["hasNextEpisode"], true);
    }

    #[test]
    fn outro_plan_clamps_threshold_and_deduplicates_display() {
        let value: Value = serde_json::from_str(&recommendation_outro_plan_json(
            r#"{"positionSeconds":90,"durationSeconds":100,"thresholdPercent":85,"alreadyShown":false}"#,
        ).unwrap()).unwrap();
        assert_eq!(value["shouldShow"], true);
        let value: Value = serde_json::from_str(&recommendation_outro_plan_json(
            r#"{"positionSeconds":90,"durationSeconds":100,"thresholdPercent":85,"alreadyShown":true}"#,
        ).unwrap()).unwrap();
        assert_eq!(value["shouldShow"], false);
        let outro_value: Value = serde_json::from_str(&recommendation_outro_plan_json(
            r#"{"positionSeconds":600,"durationSeconds":1200,"thresholdPercent":95,"outroStartSeconds":590,"alreadyShown":false}"#,
        ).unwrap()).unwrap();
        assert_eq!(outro_value["shouldShow"], true);
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
