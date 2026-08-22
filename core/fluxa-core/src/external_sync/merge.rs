use serde_json::{Value, json};

pub(crate) fn merge_external_watchlist_json(local_json: &str, external_json: &str) -> String {
    let mut local: Vec<Value> = serde_json::from_str(local_json).unwrap_or_default();
    let external: Vec<Value> = serde_json::from_str(external_json).unwrap_or_default();
    let local_ids: std::collections::HashSet<String> = local
        .iter()
        .filter_map(|i| i.get("id").and_then(Value::as_str).map(str::to_string))
        .collect();
    for item in external {
        if let Some(id) = item.get("id").and_then(Value::as_str)
            && !local_ids.contains(id)
        {
            local.push(item);
        }
    }
    serde_json::to_string(&local).unwrap_or_else(|_| "[]".to_string())
}

pub(crate) fn merge_external_watched_json(local_json: &str, external_json: &str) -> String {
    let mut local: serde_json::Map<String, Value> =
        serde_json::from_str(local_json).unwrap_or_default();
    let external: serde_json::Map<String, Value> =
        serde_json::from_str(external_json).unwrap_or_default();
    for (id, val) in external {
        if val.as_bool() == Some(true) && !local.contains_key(&id) {
            local.insert(id, Value::Bool(true));
        }
    }
    serde_json::to_string(&Value::Object(local)).unwrap_or_else(|_| "{}".to_string())
}

#[derive(serde::Deserialize)]
struct TimestampedLocalItem {
    id: String,
    #[serde(default)]
    active: bool,
    #[serde(rename = "updatedAt", default)]
    updated_at: i64,
}

#[derive(serde::Deserialize)]
struct TimestampedRemoteItem {
    id: String,
    #[serde(rename = "updatedAt", default)]
    updated_at: i64,
}

fn merge_timestamped_membership(local_json: &str, remote_json: &str) -> String {
    let local: Vec<TimestampedLocalItem> = serde_json::from_str(local_json).unwrap_or_default();
    let remote: Vec<TimestampedRemoteItem> = serde_json::from_str(remote_json).unwrap_or_default();

    let local_by_id: std::collections::HashMap<&str, &TimestampedLocalItem> =
        local.iter().map(|item| (item.id.as_str(), item)).collect();
    let remote_ids: std::collections::HashSet<&str> =
        remote.iter().map(|item| item.id.as_str()).collect();

    let mut apply_local_add: Vec<String> = Vec::new();
    let mut push_remote_add: Vec<String> = Vec::new();
    let mut push_remote_remove: Vec<String> = Vec::new();

    for remote_item in &remote {
        match local_by_id.get(remote_item.id.as_str()) {
            None => apply_local_add.push(remote_item.id.clone()),
            Some(local_item) if !local_item.active => {
                if local_item.updated_at >= remote_item.updated_at {
                    push_remote_remove.push(remote_item.id.clone());
                } else {
                    apply_local_add.push(remote_item.id.clone());
                }
            }
            Some(_) => {}
        }
    }
    for local_item in &local {
        if local_item.active && !remote_ids.contains(local_item.id.as_str()) {
            push_remote_add.push(local_item.id.clone());
        }
    }

    json!({
        "toApplyLocal": { "add": apply_local_add },
        "toPushRemote": { "add": push_remote_add, "remove": push_remote_remove }
    })
    .to_string()
}

pub(crate) fn merge_watchlist_timestamped_json(local_json: &str, remote_json: &str) -> String {
    merge_timestamped_membership(local_json, remote_json)
}

pub(crate) fn merge_watched_timestamped_json(local_json: &str, remote_json: &str) -> String {
    merge_timestamped_membership(local_json, remote_json)
}

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
