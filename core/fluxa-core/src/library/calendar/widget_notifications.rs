use super::helpers::CalendarItemInput;
use serde::Deserialize;
use serde_json::{Value, json};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WidgetRowsRequest {
    #[serde(default)]
    items: Vec<Value>,
    #[serde(default = "default_max_rows")]
    max_rows: usize,
}

fn default_max_rows() -> usize {
    4
}

pub(crate) fn calendar_widget_rows_json(request_json: &str) -> Option<String> {
    let request = serde_json::from_str::<WidgetRowsRequest>(request_json).ok()?;
    let rows: Vec<Value> = request
        .items
        .iter()
        .take(request.max_rows)
        .map(|item| {
            let date_iso = item.get("dateIso").and_then(Value::as_str).unwrap_or("");
            let title = item.get("title").and_then(Value::as_str).unwrap_or("");
            let subtitle = item
                .get("episodeTitle")
                .and_then(Value::as_str)
                .filter(|s| !s.is_empty())
                .or_else(|| item.get("subtitle").and_then(Value::as_str))
                .unwrap_or("");
            let season = item.get("seasonNumber").and_then(Value::as_i64);
            let episode = item.get("episodeNumber").and_then(Value::as_i64);
            let episode_text = match (season, episode) {
                (Some(s), Some(e)) => format!("S{}E{}", s, e),
                (Some(s), None) => format!("S{}", s),
                (None, Some(e)) => format!("E{}", e),
                _ => String::new(),
            };
            json!({
                "dateIso": date_iso,
                "title": title,
                "subtitle": subtitle,
                "episodeText": episode_text
            })
        })
        .collect();
    serde_json::to_string(&rows).ok()
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct NotificationContentRequest {
    #[serde(default)]
    items: Vec<CalendarItemInput>,
    today_iso: String,
    #[serde(default)]
    already_notified_keys: Vec<String>,
    #[serde(default)]
    profile_id: Option<String>,
    notifications_enabled: Option<bool>,
    alert_new_episodes: Option<bool>,
    #[serde(default = "default_notification_key_limit")]
    max_stored_keys: usize,
}

fn default_notification_key_limit() -> usize {
    500
}

pub(crate) fn calendar_notification_content_json(request_json: &str) -> Option<String> {
    let request = serde_json::from_str::<NotificationContentRequest>(request_json).ok()?;
    if request.notifications_enabled == Some(false) || request.alert_new_episodes == Some(false) {
        return serde_json::to_string(&json!({"items": [], "keys": []})).ok();
    }
    let profile_id = request.profile_id.as_deref().unwrap_or("");
    let mut items_out = Vec::new();
    let mut keys_out = Vec::new();
    for item in &request.items {
        if item.date_iso != request.today_iso || item.meta_type != "series" {
            continue;
        }
        let key = format!(
            "{}:{}:{}:{}",
            profile_id,
            item.date_iso,
            item.meta_id,
            item.subtitle.as_deref().unwrap_or("")
        );
        if request.already_notified_keys.contains(&key) {
            continue;
        }
        let title_key = if item.episode_number == Some(1) {
            "notification.new_season_released"
        } else {
            "notification.new_episode_released"
        };
        let body_text = match (item.season_number, item.episode_number) {
            (Some(s), Some(e)) => format!("{}:season:{}:episode:{}", item.title, s, e),
            _ => [Some(item.title.as_str()), item.subtitle.as_deref()]
                .into_iter()
                .flatten()
                .filter(|s| !s.is_empty())
                .collect::<Vec<_>>()
                .join(" - "),
        };
        items_out.push(json!({
            "key": key,
            "titleKey": title_key,
            "bodyText": body_text,
            "metaId": item.meta_id,
            "dateIso": item.date_iso,
            "artworkUrl": item.artwork_url,
            "seasonNumber": item.season_number,
            "episodeNumber": item.episode_number,
            "title": item.title,
            "subtitle": item.subtitle,
            "episodeTitle": item.episode_title
        }));
        keys_out.push(key);
    }
    let mut stored_keys = request.already_notified_keys;
    stored_keys.extend(keys_out.iter().cloned());
    let start = stored_keys.len().saturating_sub(request.max_stored_keys);
    serde_json::to_string(
        &json!({"items": items_out, "keys": keys_out, "storedKeys": stored_keys.get(start..).unwrap_or_default()}),
    )
    .ok()
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ReleaseDetectionRequest {
    #[serde(default)]
    items: Vec<Value>,
    today_iso: String,
}

pub(crate) fn calendar_release_detection_json(request_json: &str) -> Option<String> {
    let request = serde_json::from_str::<ReleaseDetectionRequest>(request_json).ok()?;
    let today = request.today_iso.trim();
    let released: Vec<&Value> = request
        .items
        .iter()
        .filter(|item| {
            item.get("dateIso")
                .and_then(Value::as_str)
                .is_some_and(|d| d == today)
        })
        .collect();
    serde_json::to_string(&released).ok()
}
