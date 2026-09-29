use super::the_introdb_canonical_type;
use serde_json::{Value, json};
use std::collections::HashMap;

pub(crate) fn parse_intro_db_segments_json(data_json: &str) -> Option<String> {
    let data: Value = serde_json::from_str(data_json).ok()?;
    let segments = collect_segments(&data);
    serde_json::to_string(&segments).ok()
}

pub(crate) fn parse_skipdb_segments_json(data_json: &str) -> Option<String> {
    let data: Value = serde_json::from_str(data_json).ok()?;
    let segments = collect_segments(&data);
    serde_json::to_string(&segments).ok()
}

// PublicMetaDB's `/api/external/skips` response wraps entries in "items" and
// uses "credits_start_ms"/"credits_end_ms" for the outro (collect_segments
// already covers "items" as a bucket and "intro_start_ms"/"intro_end_ms").
pub(crate) fn parse_publicmetadb_segments_json(data_json: &str) -> Option<String> {
    let data: Value = serde_json::from_str(data_json).ok()?;
    let segments = collect_segments(&data);
    serde_json::to_string(&segments).ok()
}

pub(crate) fn anilist_mal_id_json(data_json: &str) -> Option<String> {
    let data: Value = serde_json::from_str(data_json).ok()?;
    let mal_id = data
        .pointer("/data/Media/idMal")
        .and_then(Value::as_i64)
        .filter(|id| *id > 0)?;
    serde_json::to_string(&mal_id).ok()
}

pub(crate) fn anilist_id_json(data_json: &str) -> Option<String> {
    let data: Value = serde_json::from_str(data_json).ok()?;
    let id = data
        .pointer("/data/Media/id")
        .and_then(Value::as_i64)
        .filter(|id| *id > 0)?;
    serde_json::to_string(&id).ok()
}

pub(crate) fn anime_skip_show_id_json(data_json: &str) -> Option<String> {
    let data: Value = serde_json::from_str(data_json).ok()?;
    let id = data.pointer("/data/findShowsByExternalId/0/id")?.as_str()?;
    serde_json::to_string(id).ok()
}

fn collect_segments(data: &Value) -> Vec<Value> {
    match data {
        Value::Array(arr) => arr.iter().flat_map(collect_segments).collect(),
        Value::Object(obj) => {
            let mut result = Vec::new();
            for key in &["segments", "results", "data", "items"] {
                if let Some(child) = obj.get(*key) {
                    result.extend(collect_segments(child));
                }
            }
            for (seg_type, key_prefix) in &[
                ("intro", "intro"),
                ("outro", "outro"),
                ("outro", "credits"),
                ("recap", "recap"),
                ("preview", "preview"),
            ] {
                if let Some(child) = obj.get(*key_prefix)
                    && let Some(seg) = segment_from_object_with_type(child, seg_type)
                {
                    result.push(seg);
                }
                let start = number_from_keys(
                    obj,
                    &[
                        &format!("{key_prefix}Start"),
                        &format!("{key_prefix}_start"),
                        &format!("{key_prefix}StartTime"),
                        &format!("{key_prefix}_start_time"),
                        &format!("{key_prefix}StartMs"),
                        &format!("{key_prefix}_start_ms"),
                    ],
                );
                let end = number_from_keys(
                    obj,
                    &[
                        &format!("{key_prefix}End"),
                        &format!("{key_prefix}_end"),
                        &format!("{key_prefix}EndTime"),
                        &format!("{key_prefix}_end_time"),
                        &format!("{key_prefix}EndMs"),
                        &format!("{key_prefix}_end_ms"),
                    ],
                );
                if let (Some(s), Some(e)) = (start, end) {
                    let start_ms = normalize_time(s);
                    let end_ms = normalize_time(e);
                    if end_ms > start_ms {
                        result.push(make_segment(seg_type, start_ms, end_ms));
                    }
                }
            }
            if let Some(seg) = segment_from_object(obj)
                && !result.iter().any(|r| r == &seg)
            {
                result.push(seg);
            }
            result
                .into_iter()
                .filter(|s| {
                    let st = s.get("startTime").and_then(Value::as_i64).unwrap_or(0);
                    let et = s.get("endTime").and_then(Value::as_i64).unwrap_or(0);
                    et > st
                })
                .collect()
        }
        _ => vec![],
    }
}

fn segment_from_object(obj: &serde_json::Map<String, Value>) -> Option<Value> {
    let start = number_from_keys(
        obj,
        &[
            "startTime",
            "start",
            "from",
            "start_sec",
            "start_time",
            "startTimeMs",
            "start_ms",
            "startOffset",
        ],
    )?;
    let end = number_from_keys(
        obj,
        &[
            "endTime",
            "end",
            "to",
            "end_sec",
            "end_time",
            "endTimeMs",
            "end_ms",
            "endOffset",
        ],
    )?;
    let raw_type = string_from_keys(
        obj,
        &["segment_type", "skip_type", "category", "name", "type"],
    )
    .unwrap_or_else(|| "intro".to_string());
    let seg_type = normalize_skip_type(&raw_type);
    let start_ms = normalize_time(start);
    let end_ms = normalize_time(end);
    if end_ms <= start_ms {
        return None;
    }
    Some(make_segment(seg_type, start_ms, end_ms))
}

fn segment_from_object_with_type(value: &Value, fallback_type: &str) -> Option<Value> {
    let obj = value.as_object()?;
    let start = number_from_keys(
        obj,
        &[
            "startTime",
            "start",
            "from",
            "start_time",
            "start_sec",
            "startTimeMs",
            "start_ms",
        ],
    )?;
    let end = number_from_keys(
        obj,
        &[
            "endTime",
            "end",
            "to",
            "end_time",
            "end_sec",
            "endTimeMs",
            "end_ms",
        ],
    )?;
    let raw_type = string_from_keys(obj, &["type", "segment_type"])
        .unwrap_or_else(|| fallback_type.to_string());
    let seg_type = normalize_skip_type(&raw_type);
    let start_ms = normalize_time(start);
    let end_ms = normalize_time(end);
    if end_ms <= start_ms {
        return None;
    }
    Some(make_segment(seg_type, start_ms, end_ms))
}

fn make_segment(seg_type: &str, start_ms: i64, end_ms: i64) -> Value {
    json!({ "type": seg_type, "startTime": start_ms, "endTime": end_ms })
}

fn number_from_keys(obj: &serde_json::Map<String, Value>, keys: &[&str]) -> Option<f64> {
    for key in keys {
        match obj.get(*key) {
            Some(Value::Number(n)) => {
                if let Some(f) = n.as_f64() {
                    return Some(f);
                }
            }
            Some(Value::String(s)) => {
                if let Ok(f) = s.trim().parse::<f64>() {
                    return Some(f);
                }
            }
            _ => {}
        }
    }
    None
}

fn string_from_keys(obj: &serde_json::Map<String, Value>, keys: &[&str]) -> Option<String> {
    for key in keys {
        if let Some(Value::String(s)) = obj.get(*key) {
            let t = s.trim();
            if !t.is_empty() {
                return Some(t.to_string());
            }
        }
    }
    None
}

fn normalize_time(value: f64) -> i64 {
    if value < 10_000.0 {
        (value * 1000.0).round() as i64
    } else {
        value.round() as i64
    }
}

pub(crate) fn normalize_skip_type(raw: &str) -> &'static str {
    match raw.to_lowercase().as_str() {
        "op" | "opening" | "intro" | "mixed-intro" => "intro",
        "ed" | "ending" | "outro" | "credits" => "outro",
        "recap" | "previously" => "recap",
        "preview" | "next-time" | "nexttime" => "preview",
        _ => "intro",
    }
}

pub(crate) fn parse_aniskip_results_json(results_json: &str) -> Option<String> {
    let results: Value = serde_json::from_str(results_json).ok()?;
    let items = results.get("results").and_then(Value::as_array)?;
    let segments: Vec<Value> = items
        .iter()
        .filter_map(|item| {
            let skip_type = item
                .get("skipType")
                .or_else(|| item.get("skip_type"))
                .and_then(Value::as_str)?;
            let interval = item.get("interval")?;
            let start = interval
                .get("startTime")
                .or_else(|| interval.get("start_time"))
                .and_then(Value::as_f64)?;
            let end = interval
                .get("endTime")
                .or_else(|| interval.get("end_time"))
                .and_then(Value::as_f64)?;
            let start_ms = normalize_time(start);
            let end_ms = normalize_time(end);
            if end_ms <= start_ms {
                return None;
            }
            Some(make_segment(
                normalize_skip_type(skip_type),
                start_ms,
                end_ms,
            ))
        })
        .collect();
    serde_json::to_string(&segments).ok()
}

pub(crate) fn parse_anime_skip_results_json(results_json: &str) -> Option<String> {
    let data: Value = serde_json::from_str(results_json).ok()?;
    let timestamps = data
        .pointer("/data/findTimestampsByEpisodeId")
        .and_then(Value::as_array)
        .or_else(|| data.as_array())?;

    let mut points: Vec<(i64, &str)> = timestamps
        .iter()
        .filter_map(|t| {
            let at = t.get("at").and_then(Value::as_f64)?;
            let ty = t
                .get("type")
                .and_then(|v| v.get("name"))
                .and_then(Value::as_str)
                .or_else(|| t.get("type").and_then(Value::as_str))?;
            Some((normalize_time(at), ty))
        })
        .collect();
    points.sort_by_key(|(at, _)| *at);

    let segments: Vec<Value> = points
        .windows(2)
        .filter_map(|pair| {
            let [(start_ms, raw_type), (end_ms, _)] = pair else {
                return None;
            };
            let seg_type = animeskip_type_to_skip_type(raw_type)?;
            if end_ms <= start_ms {
                return None;
            }
            Some(make_segment(seg_type, *start_ms, *end_ms))
        })
        .collect();
    serde_json::to_string(&segments).ok()
}

fn animeskip_type_to_skip_type(raw: &str) -> Option<&'static str> {
    match raw.to_lowercase().as_str() {
        "intro" | "mixed intro" | "new intro" => Some("intro"),
        "credits" | "mixed credits" | "new credits" => Some("outro"),
        "recap" => Some("recap"),
        _ => None,
    }
}

pub(crate) fn match_anime_skip_episode_id(
    episodes_json: &str,
    season: i64,
    episode: i64,
) -> Option<String> {
    let data: Value = serde_json::from_str(episodes_json).ok()?;
    let episodes: Vec<Value> = data
        .pointer("/data/findEpisodesByShowId")
        .and_then(Value::as_array)
        .cloned()
        .or_else(|| data.as_array().cloned())?;

    let by_season_and_number = episodes.iter().find(|ep| {
        let ep_season = ep
            .get("season")
            .and_then(Value::as_str)
            .and_then(|s| s.parse::<i64>().ok());
        let ep_number = ep
            .get("number")
            .and_then(Value::as_str)
            .and_then(|s| s.parse::<i64>().ok());
        (season <= 0 || ep_season.is_none() || ep_season == Some(season))
            && ep_number == Some(episode)
    });

    let matched = by_season_and_number.or_else(|| {
        episodes.iter().find(|ep| {
            ep.get("absoluteNumber")
                .and_then(Value::as_str)
                .and_then(|s| s.parse::<i64>().ok())
                == Some(episode)
        })
    })?;

    matched
        .get("id")
        .and_then(Value::as_str)
        .map(|s| s.to_string())
}

pub(crate) fn unique_intro_segments_json(
    segments_a_json: &str,
    segments_b_json: &str,
) -> Option<String> {
    let a: Vec<Value> = serde_json::from_str(segments_a_json).unwrap_or_default();
    let b: Vec<Value> = serde_json::from_str(segments_b_json).unwrap_or_default();
    dedup_and_sort(a.into_iter().chain(b).collect())
}

pub(crate) fn merge_intro_segments_json(sources_json: &str) -> Option<String> {
    let sources: Vec<Value> = serde_json::from_str(sources_json).ok()?;
    let all: Vec<Value> = sources
        .into_iter()
        .flat_map(|s| s.as_array().cloned().unwrap_or_default())
        .collect();
    dedup_and_sort(all)
}

fn dedup_and_sort(segments: Vec<Value>) -> Option<String> {
    let mut seen: HashMap<String, bool> = HashMap::new();
    let mut result: Vec<Value> = Vec::new();
    for seg in segments {
        let key = format!(
            "{}:{}:{}",
            seg.get("type").and_then(Value::as_str).unwrap_or(""),
            seg.get("startTime").and_then(Value::as_i64).unwrap_or(0),
            seg.get("endTime").and_then(Value::as_i64).unwrap_or(0),
        );
        let end = seg.get("endTime").and_then(Value::as_i64).unwrap_or(0);
        let start = seg.get("startTime").and_then(Value::as_i64).unwrap_or(0);
        if end <= start {
            continue;
        }
        if seen.insert(key, true).is_none() {
            result.push(seg);
        }
    }
    result.sort_by_key(|s| s.get("startTime").and_then(Value::as_i64).unwrap_or(0));
    serde_json::to_string(&result).ok()
}

pub(crate) fn parse_the_introdb_segments_json(args_json: &str) -> Option<String> {
    let args: Value = serde_json::from_str(args_json).ok()?;
    let response: Value =
        serde_json::from_str(args.get("responseJson").and_then(Value::as_str)?).ok()?;
    let duration_ms = args
        .get("durationMs")
        .and_then(Value::as_i64)
        .filter(|d| *d > 0);

    let mut segments = Vec::new();
    for wire_type in &["intro", "recap", "credits", "preview"] {
        let Some(items) = response.get(*wire_type).and_then(Value::as_array) else {
            continue;
        };
        let canonical = the_introdb_canonical_type(wire_type);
        for item in items {
            let start_ms = item.get("start_ms").and_then(Value::as_i64).unwrap_or(0);
            let end_ms = match item.get("end_ms").and_then(Value::as_i64) {
                Some(ms) => ms,
                None => match duration_ms {
                    Some(d) => d,
                    None => continue,
                },
            };
            if end_ms <= start_ms {
                continue;
            }
            segments.push(make_segment(canonical, start_ms, end_ms));
        }
    }
    serde_json::to_string(&segments).ok()
}
