use serde_json::{Value, json};

pub(crate) const UP_NEXT_POSITION_SECONDS: i64 = 0;
pub(crate) const UP_NEXT_DURATION_SECONDS: i64 = 0;

pub(crate) fn normalized_continue_watching_source(value: Option<&str>) -> &'static str {
    match value.map(str::trim).map(str::to_ascii_lowercase).as_deref() {
        Some("stremio") => "stremio",
        Some("nuvio") => "nuvio",
        Some("trakt") => "trakt",
        Some("simkl") => "simkl",
        Some("anilist") => "anilist",
        Some("mdblist") => "mdblist",
        _ => "local",
    }
}

pub(crate) fn continue_watching_source_plan_json(args_json: &str) -> Option<String> {
    let args: Value = serde_json::from_str(args_json).ok()?;
    let source =
        match normalized_continue_watching_source(args.get("source").and_then(Value::as_str)) {
            "local" if args.get("nuvioConnected").and_then(Value::as_bool) == Some(true) => "nuvio",
            source => source,
        };
    let provider = match source {
        "local" => None,
        provider => Some(provider),
    };
    let uses_local = provider.is_none();
    serde_json::to_string(&json!({
        "source": source,
        "provider": provider,
        "usesLocal": uses_local,
        "requiresMetadataEnrichment": uses_local,
    }))
    .ok()
}

pub(crate) fn build_continue_watching_from_progress_json(progress_json: &str) -> Option<String> {
    let progress = serde_json::from_str::<Value>(progress_json).ok()?;
    serde_json::to_string(&build_continue_watching_from_progress(&progress)?).ok()
}

pub(crate) fn build_continue_watching_from_progress(progress: &Value) -> Option<Value> {
    let progress = progress.as_object()?;
    let mut items: Vec<Value> = progress.values()
        .filter_map(|entry| {
            let offset = entry.get("timeOffset").and_then(Value::as_f64).unwrap_or(0.0);
            let duration = entry.get("duration").and_then(Value::as_f64).unwrap_or(0.0);
            let has_video_id = entry.get("lastVideoId").and_then(Value::as_str).filter(|s| !s.is_empty()).is_some();
            // Include: items with real progress OR up-next entries (offset=0 but has lastVideoId)
            let is_resolved_up_next = entry
                .get("continueWatchingBadge")
                .and_then(Value::as_str)
                == Some("upNext");
            let include = (offset > 0.0 && duration > 0.0 && offset / duration < 0.95)
                || (has_video_id && (offset == 0.0 || is_resolved_up_next));
            if !include { return None; }
            let meta = entry.get("meta")?;
            let id = meta.get("id").and_then(Value::as_str).unwrap_or("");
            if id.is_empty() { return None; }
            Some(json!({
                "id": id,
                "name": meta.get("name").and_then(Value::as_str).unwrap_or(""),
                "type": meta.get("type").and_then(Value::as_str).unwrap_or(""),
                "poster": meta.get("poster").cloned().unwrap_or(Value::Null),
                "background": meta.get("background").cloned().unwrap_or(Value::Null),
                "logo": meta.get("logo").cloned().unwrap_or(Value::Null),
                "timeOffset": offset as i64,
                "duration": duration as i64,
                "lastVideoId": entry.get("lastVideoId").cloned().unwrap_or(Value::Null),
                "lastEpisodeName": entry.get("lastEpisodeName").cloned().unwrap_or(Value::Null),
                "lastEpisodeSeason": entry.get("lastEpisodeSeason").cloned().unwrap_or(Value::Null),
                "lastEpisodeNumber": entry.get("lastEpisodeNumber").cloned().unwrap_or(Value::Null),
                "lastEpisodeThumbnail": entry.get("lastEpisodeThumbnail").cloned().unwrap_or(Value::Null),
                "lastStreamUrl": entry.get("lastStreamUrl").cloned().unwrap_or(Value::Null),
                "lastStreamTitle": entry.get("lastStreamTitle").cloned().unwrap_or(Value::Null),
                "lastStream": entry.get("lastStream").cloned().unwrap_or(Value::Null),
                "continueWatchingBadge": entry.get("continueWatchingBadge").cloned().unwrap_or(Value::Null),
                "continueWatchingEpisodeResolved": entry.get("continueWatchingEpisodeResolved").cloned().unwrap_or(Value::Null),
                "savedAt": entry.get("savedAt").cloned().unwrap_or(Value::Null),
                "source": entry.get("source").cloned().unwrap_or(Value::Null),
            }))
        })
        .collect();
    items.sort_by(|a, b| {
        let a = a.get("savedAt").and_then(Value::as_str).unwrap_or("");
        let b = b.get("savedAt").and_then(Value::as_str).unwrap_or("");
        b.cmp(a)
    });
    Some(Value::Array(items))
}
