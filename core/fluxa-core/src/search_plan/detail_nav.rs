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
