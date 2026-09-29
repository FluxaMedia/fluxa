use super::*;

pub(super) fn trakt_progress_number(entry: &Value, key: &str) -> i64 {
    entry
        .pointer(&format!("/progress/{key}"))
        .and_then(Value::as_i64)
        .unwrap_or(0)
}

pub(super) fn trakt_show_finished(entry: &Value) -> bool {
    let aired = trakt_progress_number(entry, "aired");
    aired > 0 && trakt_progress_number(entry, "completed") >= aired
}

pub(super) fn trakt_next_episode_aired(entry: &Value, now: i64) -> bool {
    entry
        .pointer("/progress/next_episode/first_aired")
        .and_then(Value::as_str)
        .and_then(|value| chrono::DateTime::parse_from_rfc3339(value).ok())
        .is_some_and(|aired| aired.timestamp() <= now)
}

pub(super) fn trakt_continue_watching(playback: &Value, up_next: &Value) -> Value {
    let mut items: Vec<Value> = Vec::new();
    for item in playback.as_array().into_iter().flatten() {
        let id = str_field(item, "id");
        let saved_at = str_field(item, "savedAt");
        match items.iter_mut().find(|kept| str_field(kept, "id") == id) {
            Some(kept) if saved_at > str_field(kept, "savedAt") => *kept = item.clone(),
            Some(_) => {}
            None => items.push(item.clone()),
        }
    }
    for item in up_next.as_array().into_iter().flatten() {
        if !items
            .iter()
            .any(|kept| str_field(kept, "id") == str_field(item, "id"))
        {
            items.push(item.clone());
        }
    }
    items.sort_by(|a, b| str_field(b, "savedAt").cmp(str_field(a, "savedAt")));
    Value::Array(items)
}


pub(crate) fn trakt_calendar_plan_json(args_json: &str) -> Option<String> {
    let args: Value = serde_json::from_str(args_json).ok()?;
    let year = args.get("year")?.as_i64()? as i32;
    let month = args.get("month")?.as_u64()? as u32;
    let start = chrono::NaiveDate::from_ymd_opt(year, month, 1)?;
    let next = if month == 12 {
        chrono::NaiveDate::from_ymd_opt(year + 1, 1, 1)?
    } else {
        chrono::NaiveDate::from_ymd_opt(year, month + 1, 1)?
    };
    let days = (next - start).num_days();
    let requests: Vec<Value> = ["shows", "movies"]
        .into_iter()
        .map(|kind| {
            let mut plan = request(
                "trakt",
                &args,
                "GET",
                format!("{TRAKT_API}/calendars/my/{kind}/{start}/{days}?extended=full,images"),
                Value::Null,
            );
            plan["key"] = json!(kind);
            plan
        })
        .collect();
    serde_json::to_string(&requests).ok()
}


