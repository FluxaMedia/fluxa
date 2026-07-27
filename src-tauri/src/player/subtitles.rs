use super::*;

pub(super) fn selected_external_subtitle_source(renderer: &dyn PlaybackEngine) -> Option<String> {
    let count = renderer
        .query_property("track-list/count")?
        .parse::<usize>()
        .ok()?;
    for index in 0..count {
        if renderer
            .query_property(&format!("track-list/{index}/type"))
            .as_deref()
            != Some("sub")
            || renderer
                .query_property(&format!("track-list/{index}/selected"))
                .as_deref()
                != Some("yes")
        {
            continue;
        }
        if let Some(source) =
            renderer.query_property(&format!("track-list/{index}/external-filename"))
        {
            if !source.trim().is_empty() {
                return Some(source);
            }
        }
    }
    None
}

pub(super) async fn load_subtitle_text(source: &str) -> Result<String, String> {
    if source.starts_with("http://") || source.starts_with("https://") {
        return reqwest::get(source)
            .await
            .map_err(|error| error.to_string())?
            .text()
            .await
            .map_err(|error| error.to_string());
    }
    let path = source.strip_prefix("file://").unwrap_or(source);
    std::fs::read_to_string(path).map_err(|error| error.to_string())
}

pub(super) fn speech_intervals_from_media(
    source: &str,
    analysis_seconds: f64,
) -> Result<Vec<Value>, String> {
    let output = Command::new("ffmpeg")
        .args([
            "-hide_banner",
            "-nostats",
            "-t",
            &analysis_seconds.to_string(),
            "-i",
            source,
            "-map",
            "0:a:0",
            "-af",
            "silencedetect=n=-35dB:d=0.2",
            "-f",
            "null",
            "-",
        ])
        .output()
        .map_err(|error| format!("could not run ffmpeg: {error}"))?;
    let log = String::from_utf8_lossy(&output.stderr);
    let mut speech_start = 0.0;
    let mut intervals = Vec::new();
    for line in log.lines() {
        if let Some(value) = line
            .split("silence_start:")
            .nth(1)
            .and_then(|value| value.trim().parse::<f64>().ok())
        {
            if value - speech_start >= 0.2 {
                intervals.push(json!({ "start": speech_start, "end": value }));
            }
        }
        if let Some(value) = line
            .split("silence_end:")
            .nth(1)
            .and_then(|value| value.trim().split_whitespace().next()?.parse::<f64>().ok())
        {
            speech_start = value;
        }
    }
    if analysis_seconds - speech_start >= 0.2 {
        intervals.push(json!({ "start": speech_start, "end": analysis_seconds }));
    }
    if intervals.is_empty() {
        return Err("no speech activity could be detected".to_string());
    }
    Ok(intervals)
}

#[tauri::command]
pub(crate) async fn player_auto_sync_subtitles(
    state: State<'_, DesktopState>,
) -> Result<Value, String> {
    let (media_source, subtitle_source, duration) = with_renderer_retry(&state, 60, |renderer| {
        Ok((
            renderer.query_property("path"),
            selected_external_subtitle_source(renderer),
            renderer.query_property("duration"),
        ))
    })?
    .ok_or_else(|| "player renderer is not initialized".to_string())?;
    let media_source = media_source.ok_or_else(|| "media source is unavailable".to_string())?;
    let subtitle_source = subtitle_source
        .ok_or_else(|| "automatic sync requires an external subtitle track".to_string())?;
    let subtitle_text = load_subtitle_text(&subtitle_source).await?;
    let analysis_seconds = duration
        .and_then(|value| value.parse::<f64>().ok())
        .filter(|value| *value > 0.0)
        .unwrap_or(600.0)
        .min(600.0);
    let speech_intervals = tauri::async_runtime::spawn_blocking(move || {
        speech_intervals_from_media(&media_source, analysis_seconds)
    })
    .await
    .map_err(|error| error.to_string())??;
    let request = json!({ "subtitleText": subtitle_text, "speechIntervals": speech_intervals });
    FluxaCore::subtitle_sync_estimate_json(&request.to_string())
        .and_then(|result| serde_json::from_str(&result).ok())
        .ok_or_else(|| "could not determine a reliable subtitle delay".to_string())
}

#[tauri::command]
pub(crate) async fn player_capture_subtitle_cues(
    state: State<'_, DesktopState>,
) -> Result<Value, String> {
    let (subtitle_source, current_time) = with_renderer_retry(&state, 60, |renderer| {
        Ok((
            selected_external_subtitle_source(renderer),
            renderer.query_property("time-pos"),
        ))
    })?
    .ok_or_else(|| "player renderer is not initialized".to_string())?;
    let subtitle_source =
        subtitle_source.ok_or_else(|| "capture requires an external subtitle track".to_string())?;
    let current_time = current_time
        .and_then(|value| value.parse::<f64>().ok())
        .filter(|value| value.is_finite() && *value >= 0.0)
        .ok_or_else(|| "playback time is unavailable".to_string())?;
    let subtitle_text = load_subtitle_text(&subtitle_source).await?;
    let request = json!({ "subtitleText": subtitle_text, "currentTime": current_time });
    FluxaCore::subtitle_cues_around_time_json(&request.to_string())
        .and_then(|result| serde_json::from_str(&result).ok())
        .map(|mut result: Value| {
            result["capturedTime"] = json!(current_time);
            result
        })
        .ok_or_else(|| "could not read subtitle cues".to_string())
}
