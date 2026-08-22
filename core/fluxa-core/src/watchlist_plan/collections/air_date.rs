use super::helpers::timestamp;
use serde_json::{Value, json};

const AIR_DATE_COOLDOWN_MS: i64 = 12 * 60 * 60 * 1000;

pub(crate) fn air_date_refresh_candidates_json(args_json: &str) -> Option<String> {
    let args: Value = serde_json::from_str(args_json).ok()?;
    let now_ms = args.get("nowMs").and_then(Value::as_i64)?;
    let items = args.get("items").and_then(Value::as_array)?;

    let parse_ms = |value: Option<&Value>| {
        value
            .and_then(Value::as_str)
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.timestamp_millis())
    };

    let mut seen: Vec<&str> = Vec::new();
    let mut due: Vec<Value> = Vec::new();
    for item in items {
        let Some(id) = item.get("id").and_then(Value::as_str) else {
            continue;
        };
        if seen.contains(&id) {
            continue;
        }
        seen.push(id);
        if item.get("type").and_then(Value::as_str) != Some("series") {
            continue;
        }
        let next_air = parse_ms(item.get("nextEpisodeAirDate"));
        let missing_or_past = match next_air {
            Some(ms) => ms <= now_ms,
            None => true,
        };
        let missing_episode_details =
            ["nextEpisodeSeason", "nextEpisodeNumber", "nextEpisodeTitle"]
                .iter()
                .any(|key| item.get(*key).is_none() || item.get(*key) == Some(&Value::Null));
        if !missing_or_past && !missing_episode_details {
            continue;
        }
        let last_checked = parse_ms(item.get("lastAirDateCheckedAt")).unwrap_or(0);
        if missing_episode_details || now_ms - last_checked >= AIR_DATE_COOLDOWN_MS {
            due.push(Value::String(id.to_string()));
        }
    }
    Some(Value::Array(due).to_string())
}

pub(super) fn air_time(item: &Value) -> i64 {
    let value = timestamp(item, "nextEpisodeAirDate").max(timestamp(item, "newEpisodeReleasedAt"));
    if value > 0 { value } else { i64::MAX }
}

pub(crate) fn air_date_refresh_plan_json(args_json: &str) -> Option<String> {
    let args: Value = serde_json::from_str(args_json).ok()?;
    let items = args.get("items")?.as_array()?;
    let due_ids: Vec<String> =
        serde_json::from_str(&air_date_refresh_candidates_json(args_json)?).ok()?;
    let due: std::collections::HashSet<&str> = due_ids.iter().map(String::as_str).collect();
    let mut seen = std::collections::HashSet::new();
    let candidates: Vec<&Value> = items
        .iter()
        .filter(|item| {
            item.get("id")
                .and_then(Value::as_str)
                .is_some_and(|id| due.contains(id) && seen.insert(id))
        })
        .collect();
    serde_json::to_string(&candidates).ok()
}

pub(crate) fn apply_air_date_updates_json(args_json: &str) -> Option<String> {
    let args: Value = serde_json::from_str(args_json).ok()?;
    let updates = args.get("updates")?.as_array()?;
    let apply = |items: &Vec<Value>| {
        items
            .iter()
            .map(|item| {
                let Some(update) = updates
                    .iter()
                    .find(|update| update.get("id") == item.get("id"))
                else {
                    return item.clone();
                };
                let mut merged = item.as_object().cloned().unwrap_or_default();
                for key in [
                    "nextEpisodeAirDate",
                    "nextEpisodeSeason",
                    "nextEpisodeNumber",
                    "nextEpisodeTitle",
                    "nextEpisodePoster",
                    "lastAirDateCheckedAt",
                ] {
                    merged.insert(
                        key.to_string(),
                        update.get(key).cloned().unwrap_or(Value::Null),
                    );
                }
                Value::Object(merged)
            })
            .collect::<Vec<_>>()
    };
    serde_json::to_string(&json!({
        "watchlist": apply(args.get("watchlist")?.as_array()?),
        "continueWatching": apply(args.get("continueWatching")?.as_array()?),
    }))
    .ok()
}
