use serde_json::{Value, json};

pub(crate) fn detail_series_lookup_id(raw_id: &str) -> String {
    let trimmed = raw_id.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    if let Some(imdb) = extract_imdb_id(trimmed) {
        return imdb;
    }
    // Strip trailing season:episode parts (e.g. "kitsu:777:1:2" -> "kitsu:777", "base:1:2" -> "base")
    let mut parts = trimmed.rsplitn(3, ':');
    if let (Some(last), Some(second_last), Some(base)) = (parts.next(), parts.next(), parts.next())
        && last.parse::<i32>().is_ok()
        && second_last.parse::<i32>().is_ok()
    {
        return base.to_string();
    }
    trimmed.to_string()
}

fn extract_imdb_id(raw: &str) -> Option<String> {
    for (start, _) in raw.match_indices("tt") {
        let rest = raw.get(start + 2..)?;
        let digits = rest.bytes().take_while(u8::is_ascii_digit).count();
        if digits > 1 {
            return raw.get(start..start + 2 + digits).map(str::to_string);
        }
    }
    None
}

pub(crate) fn detail_season_load_plan_json(request_json: &str) -> Option<String> {
    let value: Value = serde_json::from_str(request_json).ok()?;
    let saved_video_id = value
        .get("savedVideoId")
        .and_then(Value::as_str)
        .unwrap_or("");
    let seasons_count = value
        .get("seasonsCount")
        .and_then(Value::as_i64)
        .unwrap_or(1)
        .max(1) as i32;

    let saved_season = saved_video_id
        .split(':')
        .nth(1)
        .and_then(|s| s.parse::<i32>().ok())
        .unwrap_or(0);

    let first_season = if saved_season > 0 && saved_season <= seasons_count {
        saved_season
    } else {
        1
    };

    serde_json::to_string(&json!({
        "firstSeasonToLoad": first_season,
        "savedSeason": if saved_season > 0 { json!(saved_season) } else { Value::Null }
    }))
    .ok()
}

pub(crate) fn detail_available_seasons_json(request_json: &str) -> Option<String> {
    let request: Value = serde_json::from_str(request_json).ok()?;
    let seasons_count = request
        .get("seasonsCount")
        .and_then(Value::as_i64)
        .unwrap_or(0)
        .max(0) as i32;
    let mut seasons: Vec<i32> = (1..=seasons_count).collect();
    let mut has_special_season = false;
    if let Some(values) = request.get("seasons").and_then(Value::as_array) {
        seasons.extend(values.iter().filter_map(Value::as_i64).filter_map(|value| {
            (value > 0 && value <= i32::MAX as i64).then_some(value as i32)
        }));
        has_special_season = values.iter().any(|value| value.as_i64() == Some(0));
    }
    seasons.sort_unstable();
    seasons.dedup();
    if has_special_season {
        seasons.push(0);
    }
    if seasons.is_empty() {
        seasons.push(1);
    }
    serde_json::to_string(&seasons).ok()
}

pub(crate) fn detail_load_plan_json(request_json: &str) -> Option<String> {
    let request: Value = serde_json::from_str(request_json).ok()?;
    let requested_type = request
        .get("requestedType")
        .and_then(Value::as_str)
        .unwrap_or("");
    let requested_id = request
        .get("requestedId")
        .and_then(Value::as_str)
        .unwrap_or("");
    let detail = request.get("detail").filter(|value| value.is_object());
    let resolved_id = detail
        .and_then(|value| value.get("id"))
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .unwrap_or(requested_id);
    let detail_type = detail
        .and_then(|value| value.get("type"))
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty());
    let effective_type = if requested_id.starts_with("cs3:") {
        detail_type.unwrap_or(requested_type)
    } else {
        requested_type
    };
    let series_lookup_id = detail_series_lookup_id(resolved_id);
    let videos = detail
        .and_then(|value| value.get("videos"))
        .and_then(Value::as_array);
    let has_detail_videos = videos.map(|items| !items.is_empty()).unwrap_or(false);
    let season = request
        .get("season")
        .and_then(Value::as_i64)
        .unwrap_or(1)
        .max(1);
    let stream_lookup_id = if effective_type == "series" {
        series_lookup_id.clone()
    } else {
        resolved_id.to_string()
    };

    serde_json::to_string(&serde_json::json!({
        "effectiveType": effective_type,
        "resolvedId": resolved_id,
        "seriesLookupId": series_lookup_id,
        "streamLookupId": stream_lookup_id,
        "season": season,
        "shouldReadSeasonFromDetail": effective_type == "series" && has_detail_videos,
        "shouldFetchSeason": effective_type == "series" && !has_detail_videos,
        "title": detail.and_then(|value| value.get("name")).cloned().unwrap_or(Value::Null),
        "originalName": detail.and_then(|value| value.get("originalName")).cloned().unwrap_or(Value::Null),
        "year": detail
            .and_then(|value| value.get("releaseInfo"))
            .and_then(Value::as_str)
            .and_then(|value| value.parse::<i32>().ok())
            .map(Value::from)
            .unwrap_or(Value::Null)
    }))
    .ok()
}

pub(crate) fn detail_season_videos_json(request_json: &str) -> Option<String> {
    let request: Value = serde_json::from_str(request_json).ok()?;
    let videos = request.get("videos").and_then(Value::as_array)?;
    let season = request
        .get("season")
        .and_then(Value::as_i64)
        .unwrap_or(1) as i32;
    let has_season_data = videos.iter().any(|video| {
        video
            .get("season")
            .and_then(Value::as_i64)
            .map(|value| value > 0)
            .unwrap_or(false)
    });
    if !has_season_data {
        return serde_json::to_string(&serde_json::json!({ "episodes": videos })).ok();
    }
    let selected: Vec<&Value> = videos
        .iter()
        .filter(|video| video.get("season").and_then(Value::as_i64) == Some(season as i64))
        .collect();
    if !selected.is_empty() {
        return serde_json::to_string(&serde_json::json!({ "episodes": selected })).ok();
    }
    let first_available = videos
        .iter()
        .filter_map(|video| video.get("season").and_then(Value::as_i64))
        .filter(|value| *value > 0)
        .min();
    let fallback: Vec<&Value> = match first_available {
        Some(first) => videos
            .iter()
            .filter(|video| video.get("season").and_then(Value::as_i64) == Some(first))
            .collect(),
        None => videos.iter().collect(),
    };
    serde_json::to_string(&serde_json::json!({ "episodes": fallback })).ok()
}

#[cfg(test)]
mod tests {
    use super::detail_available_seasons_json;

    #[test]
    fn available_seasons_merges_count_video_seasons_and_special_season_last() {
        assert_eq!(
            detail_available_seasons_json(
                r#"{"seasonsCount":2,"seasons":[3,0,2,3]}"#
            )
            .as_deref(),
            Some("[1,2,3,0]")
        );
        assert_eq!(
            detail_available_seasons_json(r#"{"seasonsCount":0,"seasons":[]}"#)
                .as_deref(),
            Some("[1]")
        );
    }
}
