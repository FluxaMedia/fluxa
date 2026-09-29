use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct CalendarItemInput {
    pub(super) date_iso: String,
    #[serde(default)]
    pub(super) meta_id: String,
    #[serde(default)]
    pub(super) meta_type: String,
    #[serde(default)]
    pub(super) title: String,
    #[serde(default)]
    pub(super) subtitle: Option<String>,
    #[serde(default)]
    pub(super) season_number: Option<i32>,
    #[serde(default)]
    pub(super) episode_number: Option<i32>,
    #[serde(default)]
    pub(super) episode_title: Option<String>,
    #[serde(default)]
    pub(super) artwork_url: Option<String>,
    #[serde(default)]
    pub(super) meta: Value,
    #[serde(default)]
    pub(super) poster: Option<String>,
    #[serde(default)]
    pub(super) episode_poster: Option<String>,
}

pub(super) fn calendar_item_identity(item: &Value) -> String {
    let date = item
        .get("dateIso")
        .and_then(Value::as_str)
        .map(|value| value.get(..10).unwrap_or(value))
        .unwrap_or("");
    let content = item
        .get("title")
        .and_then(Value::as_str)
        .or_else(|| item.get("name").and_then(Value::as_str))
        .or_else(|| {
            ["contentId", "seriesId"]
                .iter()
                .find_map(|key| item.get(*key).and_then(Value::as_str))
        })
        .or_else(|| item.get("id").and_then(Value::as_str))
        .unwrap_or("");
    let season = ["seasonNumber", "season"]
        .iter()
        .find_map(|key| item.get(*key).and_then(Value::as_i64))
        .unwrap_or_default();
    let episode = ["episodeNumber", "episode", "number"]
        .iter()
        .find_map(|key| item.get(*key).and_then(Value::as_i64))
        .unwrap_or_default();
    format!("{date}:{content}:{season}:{episode}")
}

pub(super) fn calendar_item_detail_score(item: &Value) -> usize {
    [
        "poster",
        "seriesPoster",
        "episodePoster",
        "episodeTitle",
        "airTime",
        "releaseTime",
    ]
    .iter()
    .filter(|key| {
        item.get(**key)
            .and_then(Value::as_str)
            .is_some_and(|value| !value.is_empty())
    })
    .count()
}

pub(super) fn usable_artwork(url: Option<&str>) -> Option<&str> {
    url.filter(|value| {
        let normalized = value.trim().to_ascii_lowercase();
        !normalized.is_empty()
            && normalized != "null"
            && !normalized.contains("default-poster")
            && !normalized.contains("placeholder")
            && !normalized.contains("no-image")
            && !normalized.contains("no_image")
    })
}

pub(super) fn resolve_calendar_artwork(item: &CalendarItemInput) -> Option<String> {
    [
        item.episode_poster.as_deref(),
        item.poster.as_deref(),
        item.meta.get("poster").and_then(Value::as_str),
        item.meta
            .get("continueWatchingPoster")
            .and_then(Value::as_str),
        item.meta.get("background").and_then(Value::as_str),
        item.meta
            .get("continueWatchingBackground")
            .and_then(Value::as_str),
    ]
    .into_iter()
    .find_map(usable_artwork)
    .map(str::to_string)
}

pub(super) fn end_of_current_week_ms(now_ms: i64) -> i64 {
    use chrono::{Datelike, Local, TimeZone};
    let now = Local
        .timestamp_millis_opt(now_ms)
        .single()
        .unwrap_or_else(chrono::Local::now);
    let days_until_sunday = (7 - now.weekday().num_days_from_sunday() as i64) % 7;
    let end_date = now.date_naive() + chrono::Duration::days(days_until_sunday);
    let Some(end) = end_date.and_hms_milli_opt(23, 59, 59, 999) else {
        return now_ms;
    };
    Local
        .from_local_datetime(&end)
        .single()
        .map(|d| d.timestamp_millis())
        .unwrap_or(now_ms)
}

pub(super) fn parse_date_ms(raw: &str) -> Option<i64> {
    if let Ok(timestamp) = chrono::DateTime::parse_from_rfc3339(raw) {
        return Some(timestamp.timestamp_millis());
    }
    chrono::NaiveDate::parse_from_str(raw, "%Y-%m-%d")
        .ok()?
        .and_hms_opt(0, 0, 0)
        .map(|time| time.and_utc().timestamp_millis())
}
