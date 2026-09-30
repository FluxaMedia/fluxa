use crate::services::tracking::external_sync::{ranked_winner, saved_at_ms};
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
