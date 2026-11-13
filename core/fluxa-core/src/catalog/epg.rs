use chrono::{DateTime, Duration, NaiveDate, TimeZone, Utc};
use serde_json::{Map, Value, json};
use std::collections::BTreeMap;

const GUIDE_REFRESH_MS: i64 = 15 * 60 * 1000;
const FAILED_REFRESH_MS: i64 = 60 * 1000;

pub(crate) fn guide_catalogs(manifest: &Value) -> Vec<Value> {
    let manifest = manifest.get("manifest").unwrap_or(manifest);
    let provider = manifest
        .pointer("/behaviorHints/epgProvider")
        .and_then(Value::as_bool)
        == Some(true);
    if !provider {
        return Vec::new();
    }
    manifest
        .get("catalogs")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter(|catalog| catalog.get("type").and_then(Value::as_str) == Some("tv"))
        .filter(|catalog| {
            catalog
                .get("extra")
                .and_then(Value::as_array)
                .is_some_and(|extra| {
                    extra
                        .iter()
                        .any(|item| item.get("name").and_then(Value::as_str) == Some("date"))
                })
        })
        .cloned()
        .collect()
}

pub(crate) fn utc_days(from_ms: i64, to_ms: i64) -> Vec<String> {
    let day = |ms: i64| {
        Utc.timestamp_millis_opt(ms)
            .single()
            .map(|time| time.date_naive())
    };
    let (Some(first), Some(last)) = (day(from_ms), day(to_ms.max(from_ms))) else {
        return Vec::new();
    };
    let mut days = Vec::new();
    let mut current: NaiveDate = first;
    while current <= last && days.len() < 8 {
        days.push(current.format("%Y-%m-%d").to_string());
        current += Duration::days(1);
    }
    days
}

fn timestamp_ms(video: &Value, key: &str) -> Option<i64> {
    DateTime::parse_from_rfc3339(video.get(key)?.as_str()?)
        .ok()
        .map(|time| time.timestamp_millis())
}

pub(crate) fn programme(video: &Value) -> Option<Value> {
    let start = timestamp_ms(video, "startTime")?;
    let end = timestamp_ms(video, "endTime")?;
    if end <= start {
        return None;
    }
    let mut out = video.as_object().cloned().unwrap_or_default();
    out.insert("startMs".into(), json!(start));
    out.insert("endMs".into(), json!(end));
    Some(Value::Object(out))
}

pub(crate) fn build_guide(pages: &[Value], from_ms: i64, to_ms: i64, now_ms: i64) -> Value {
    let mut channels: BTreeMap<String, (Map<String, Value>, BTreeMap<String, Value>)> =
        BTreeMap::new();
    let mut order: Vec<String> = Vec::new();
    for page in pages {
        let metas = page
            .get("metasDetailed")
            .or_else(|| page.get("metas"))
            .and_then(Value::as_array)
            .into_iter()
            .flatten();
        for meta in metas {
            let Some(id) = meta.get("id").and_then(Value::as_str) else {
                continue;
            };
            if !channels.contains_key(id) {
                order.push(id.to_owned());
            }
            let (info, programmes) = channels.entry(id.to_owned()).or_default();
            for key in [
                "id",
                "type",
                "name",
                "poster",
                "posterShape",
                "logo",
                "behaviorHints",
            ] {
                if let Some(value) = meta.get(key) {
                    info.insert(key.to_owned(), value.clone());
                }
            }
            for video in meta
                .get("videos")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
            {
                let Some(entry) = programme(video) else {
                    continue;
                };
                let (start, end) = (entry["startMs"].as_i64(), entry["endMs"].as_i64());
                let overlaps = end.is_some_and(|end| end > from_ms)
                    && start.is_some_and(|start| start < to_ms);
                let key = video.get("id").and_then(Value::as_str).unwrap_or_default();
                if overlaps && !key.is_empty() {
                    programmes.insert(key.to_owned(), entry);
                }
            }
        }
    }
    let rows: Vec<Value> = order
        .iter()
        .filter_map(|id| channels.remove(id))
        .map(|(info, programmes)| {
            let mut list: Vec<Value> = programmes.into_values().collect();
            list.sort_by_key(|entry| entry["startMs"].as_i64().unwrap_or_default());
            let now_index = list.iter().position(|entry| {
                entry["startMs"]
                    .as_i64()
                    .is_some_and(|start| start <= now_ms)
                    && entry["endMs"].as_i64().is_some_and(|end| end > now_ms)
            });
            let mut row = info;
            row.insert("programmes".into(), Value::Array(list));
            row.insert("nowIndex".into(), json!(now_index));
            Value::Object(row)
        })
        .collect();
    json!({ "channels": rows })
}

pub(crate) fn refresh_delay_ms(last_end_ms: Option<i64>, now_ms: i64, failed: bool) -> i64 {
    if failed || last_end_ms.is_some_and(|end| end <= now_ms) {
        FAILED_REFRESH_MS
    } else {
        GUIDE_REFRESH_MS
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ms(text: &str) -> i64 {
        DateTime::parse_from_rfc3339(text)
            .unwrap()
            .timestamp_millis()
    }

    fn guide_manifest(provider: bool) -> Value {
        json!({
            "behaviorHints": {"epgProvider": provider},
            "catalogs": [
                {"type": "tv", "id": "channels", "extra": [{"name": "skip"}, {"name": "date"}]},
                {"type": "tv", "id": "plain"},
                {"type": "movie", "id": "m", "extra": [{"name": "date"}]}
            ]
        })
    }

    #[test]
    fn only_tv_catalogs_with_date_are_guides() {
        let found = guide_catalogs(&guide_manifest(true));
        assert_eq!(found.len(), 1);
        assert_eq!(found[0]["id"], "channels");
        assert!(guide_catalogs(&guide_manifest(false)).is_empty());
    }

    #[test]
    fn local_day_spanning_midnight_requests_both_utc_days() {
        let days = utc_days(ms("2026-09-12T22:00:00Z"), ms("2026-09-13T02:00:00Z"));
        assert_eq!(days, ["2026-09-12", "2026-09-13"]);
    }

    #[test]
    fn pages_merge_per_channel_and_drop_invalid_programmes() {
        let page = |videos: Value| json!({"metasDetailed": [{"id": "ch:1", "type": "tv", "name": "One", "videos": videos}]});
        let first = page(json!([
            {"id": "a", "title": "A", "startTime": "2026-09-12T18:00:00Z", "endTime": "2026-09-12T19:00:00Z"},
            {"id": "bad", "title": "B", "startTime": "2026-09-12T20:00:00Z", "endTime": "2026-09-12T20:00:00Z"},
            {"id": "noend", "title": "C", "startTime": "2026-09-12T20:00:00Z"}
        ]));
        let second = page(json!([
            {"id": "b", "title": "B", "startTime": "2026-09-12T19:00:00Z", "endTime": "2026-09-12T20:00:00Z"},
            {"id": "a", "title": "A", "startTime": "2026-09-12T18:00:00Z", "endTime": "2026-09-12T19:00:00Z"}
        ]));
        let guide = build_guide(
            &[first, second],
            ms("2026-09-12T00:00:00Z"),
            ms("2026-09-13T00:00:00Z"),
            ms("2026-09-12T19:30:00Z"),
        );
        let channel = &guide["channels"][0];
        let ids: Vec<_> = channel["programmes"]
            .as_array()
            .unwrap()
            .iter()
            .map(|entry| entry["id"].as_str().unwrap())
            .collect();
        assert_eq!(ids, ["a", "b"]);
        assert_eq!(channel["nowIndex"], 1);
    }

    #[test]
    fn programmes_outside_the_window_are_dropped() {
        let page = json!({"metasDetailed": [{"id": "ch:1", "videos": [
            {"id": "old", "startTime": "2026-09-11T10:00:00Z", "endTime": "2026-09-11T11:00:00Z"}
        ]}]});
        let guide = build_guide(
            &[page],
            ms("2026-09-12T00:00:00Z"),
            ms("2026-09-13T00:00:00Z"),
            ms("2026-09-12T12:00:00Z"),
        );
        assert!(
            guide["channels"][0]["programmes"]
                .as_array()
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn refresh_is_fast_after_last_programme_ended_or_failure() {
        let now = ms("2026-09-12T12:00:00Z");
        assert_eq!(
            refresh_delay_ms(Some(now + 1), now, false),
            GUIDE_REFRESH_MS
        );
        assert_eq!(refresh_delay_ms(Some(now), now, false), FAILED_REFRESH_MS);
        assert_eq!(refresh_delay_ms(None, now, true), FAILED_REFRESH_MS);
    }
}
