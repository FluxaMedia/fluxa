use crate::external_sync::{ranked_winner, saved_at_ms};
use serde_json::Value;

pub(crate) fn replace_external_continue_watching_json(
    existing_json: &str,
    provider: Option<&str>,
    items_json: &str,
    source_of_truth: Option<&str>,
    ranking_mode: Option<&str>,
    continue_watching_days: Option<i64>,
) -> String {
    let existing: Vec<Value> = serde_json::from_str(existing_json).unwrap_or_default();
    let incoming: Vec<Value> = serde_json::from_str(items_json).unwrap_or_default();

    let incoming_filtered: Vec<Value> = incoming
        .into_iter()
        .filter(|item| {
            let id = item.get("id").and_then(Value::as_str).unwrap_or("").trim();
            let offset = item
                .get("timeOffset")
                .and_then(Value::as_f64)
                .unwrap_or(0.0);
            let duration = item.get("duration").and_then(Value::as_f64).unwrap_or(0.0);
            let has_progress_percent = item
                .get("resumeProgressPercent")
                .and_then(Value::as_f64)
                .is_some_and(|percent| percent > 0.0);
            let has_watch_next_badge = item
                .get("continueWatchingBadge")
                .is_some_and(|value| !value.is_null());
            let within_window =
                continue_watching_days
                    .filter(|days| *days > 0)
                    .is_none_or(|days| {
                        saved_at_ms(item)
                            >= chrono::Utc::now().timestamp_millis() - days * 86_400_000
                    });
            !id.is_empty()
                && (offset > 0.0 && duration > 0.0 || has_progress_percent || has_watch_next_badge)
                && within_window
        })
        .collect();

    let base: Vec<Value> = if let Some(prov) = provider {
        existing
            .into_iter()
            .filter(|item| item.get("reason").and_then(Value::as_str) != Some(prov))
            .collect()
    } else {
        Vec::new()
    };

    let combined = base.into_iter().chain(incoming_filtered);
    let mut by_id: std::collections::HashMap<String, Value> = std::collections::HashMap::new();
    for item in combined {
        let id = item
            .get("id")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        if id.is_empty() {
            continue;
        }
        match by_id.get(&id) {
            Some(prev) => {
                let item_reason = item.get("reason").and_then(Value::as_str);
                let prev_reason = prev.get("reason").and_then(Value::as_str);
                let item_wins = if source_of_truth.is_some() && source_of_truth == item_reason {
                    true
                } else if source_of_truth.is_some() && source_of_truth == prev_reason {
                    false
                } else {
                    ranked_winner(
                        &item,
                        saved_at_ms(&item),
                        prev,
                        saved_at_ms(prev),
                        ranking_mode,
                    )
                };
                if item_wins {
                    by_id.insert(id, item);
                }
            }
            None => {
                by_id.insert(id, item);
            }
        }
    }

    let mut result: Vec<Value> = by_id.into_values().collect();
    result.sort_by_key(|item| std::cmp::Reverse(saved_at_ms(item)));
    serde_json::to_string(&result).unwrap_or_else(|_| "[]".to_string())
}

pub(crate) fn trakt_playback_items_dedup_json(items_json: &str) -> Option<String> {
    let items: Vec<Value> = serde_json::from_str(items_json).ok()?;

    fn saved_at_str(item: &Value) -> &str {
        item.get("savedAt").and_then(Value::as_str).unwrap_or("")
    }

    fn episode_rank(item: &Value) -> Option<(i64, i64)> {
        Some((
            item.get("lastEpisodeSeason").and_then(Value::as_i64)?,
            item.get("lastEpisodeNumber").and_then(Value::as_i64)?,
        ))
    }

    let mut best: std::collections::HashMap<String, Value> = std::collections::HashMap::new();
    for item in items {
        let id = item
            .get("id")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        if id.is_empty() {
            continue;
        }
        let cur = saved_at_str(&item).to_string();
        match best.get(&id) {
            None => {
                best.insert(id, item);
            }
            Some(existing) => {
                let incoming_rank = episode_rank(&item);
                let existing_rank = episode_rank(existing);
                let incoming_is_watched =
                    item.get("continueWatchingBadge").and_then(Value::as_str) == Some("upNext");
                let existing_is_watched = existing
                    .get("continueWatchingBadge")
                    .and_then(Value::as_str)
                    == Some("upNext");
                let incoming_wins = match (incoming_rank, existing_rank) {
                    (Some(incoming_rank), Some(existing_rank))
                        if incoming_is_watched || existing_is_watched =>
                    {
                        incoming_rank > existing_rank
                    }
                    _ => cur.as_str() > saved_at_str(existing),
                };
                if incoming_wins {
                    best.insert(id, item);
                }
            }
        }
    }

    let mut deduped: Vec<Value> = best.into_values().collect();
    deduped.sort_by(|a, b| saved_at_str(b).cmp(saved_at_str(a)));
    serde_json::to_string(&deduped).ok()
}
