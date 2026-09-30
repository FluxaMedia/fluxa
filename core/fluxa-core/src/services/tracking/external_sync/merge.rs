use serde_json::Value;

fn item_id(item: &Value) -> String {
    item.get("id")
        .or_else(|| item.get("_id"))
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string()
}

pub(crate) fn saved_at_ms(item: &Value) -> i64 {
    item.get("savedAt")
        .and_then(Value::as_str)
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
        .map(|dt: chrono::DateTime<chrono::FixedOffset>| dt.timestamp_millis())
        .unwrap_or(0)
}

fn episode_rank(item: &Value) -> Option<(i64, i64)> {
    let season = item.get("lastEpisodeSeason").and_then(Value::as_i64)?;
    let number = item.get("lastEpisodeNumber").and_then(Value::as_i64)?;
    Some((season, number))
}

pub(crate) fn ranked_winner(
    a: &Value,
    a_time: i64,
    b: &Value,
    b_time: i64,
    ranking_mode: Option<&str>,
) -> bool {
    if ranking_mode == Some("most_recent_episode")
        && let (Some(ra), Some(rb)) = (episode_rank(a), episode_rank(b))
        && ra != rb
    {
        return ra > rb;
    }
    a_time >= b_time
}

pub(crate) fn merge_continue_watching_lists_json(
    local_json: &str,
    external_json: &str,
    progress_json: &str,
    source_of_truth: Option<&str>,
    ranking_mode: Option<&str>,
) -> Option<String> {
    let local: Vec<Value> = serde_json::from_str(local_json).unwrap_or_default();
    let external: Vec<Value> = serde_json::from_str(external_json).unwrap_or_default();
    let progress: serde_json::Map<String, Value> =
        serde_json::from_str(progress_json).unwrap_or_default();

    let local_by_id: std::collections::HashMap<String, &Value> =
        local.iter().map(|item| (item_id(item), item)).collect();
    let external_by_id: std::collections::HashMap<String, &Value> =
        external.iter().map(|item| (item_id(item), item)).collect();

    fn local_saved_at_from_progress(progress: &serde_json::Map<String, Value>, id: &str) -> i64 {
        progress
            .get(id)
            .and_then(|entry| entry.get("savedAt"))
            .and_then(Value::as_str)
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map(|dt: chrono::DateTime<chrono::FixedOffset>| dt.timestamp_millis())
            .unwrap_or(0)
    }

    let mut merged: Vec<Value> = Vec::new();
    for ext_item in &external {
        let id = item_id(ext_item);
        let local_item = local_by_id.get(&id).copied();
        let local_time = local_saved_at_from_progress(&progress, &id);
        let ext_time = saved_at_ms(ext_item);

        let local_wins = if let Some(local_item) = local_item {
            let local_source = local_item
                .get("source")
                .and_then(Value::as_str)
                .unwrap_or("local");
            if source_of_truth.is_some() && source_of_truth == Some(local_source) {
                true
            } else if source_of_truth.is_some()
                && source_of_truth == ext_item.get("reason").and_then(Value::as_str)
            {
                false
            } else {
                ranked_winner(local_item, local_time, ext_item, ext_time, ranking_mode)
            }
        } else {
            false
        };

        if local_wins {
            if let Some(local_item) = local_item {
                merged.push(local_item.clone());
            } else {
                merged.push(ext_item.clone());
            }
        } else {
            merged.push(ext_item.clone());
        }
    }
    for local_item in &local {
        let id = item_id(local_item);
        if !external_by_id.contains_key(&id) {
            merged.push(local_item.clone());
        }
    }

    merged.sort_by_key(|item| std::cmp::Reverse(saved_at_ms(item)));

    serde_json::to_string(&merged).ok()
}
