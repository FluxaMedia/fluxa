use crate::{torrent_transport, DesktopState};
use fluxa_core::FluxaCore;
use serde_json::{json, Value};
use std::fs;
use tauri::State;

fn start_torrent_stream_inner(
    data_dir: std::path::PathBuf,
    stream_json: String,
    title: Option<String>,
    preferences: Option<Value>,
    existing_base_url: Option<String>,
) -> Result<(String, String, String, Option<u64>, Option<usize>), String> {
    let (base_url, generation) = if let Some(base_url) = existing_base_url {
        torrent_transport::apply_preferences(&base_url, preferences.as_ref());
        (base_url, None)
    } else {
        let cache_dir = data_dir.join("torrent-cache");
        let server_json =
            fluxa_streaming_engine::start_torrent_server(&cache_dir.to_string_lossy(), 0, "")
                .ok_or_else(|| "failed to start torrent server".to_string())?;
        let server: Value = serde_json::from_str(&server_json)
            .map_err(|e| format!("invalid torrent server response: {e}"))?;
        let base_url = server
            .get("url")
            .and_then(Value::as_str)
            .map(str::to_string)
            .ok_or_else(|| "torrent server did not return url".to_string())?;
        let generation = server.get("generation").and_then(Value::as_u64);
        torrent_transport::apply_preferences(&base_url, preferences.as_ref());
        (base_url, generation)
    };

    let stream: Value =
        serde_json::from_str(&stream_json).map_err(|e| format!("invalid stream json: {e}"))?;
    let playback_json = FluxaCore::stream_playback_info_json(&stream_json)
        .ok_or_else(|| "stream playback info could not be resolved".to_string())?;
    let playback: Value =
        serde_json::from_str(&playback_json).map_err(|e| format!("invalid playback info: {e}"))?;
    let link = playback
        .get("playableUrl")
        .and_then(Value::as_str)
        .ok_or_else(|| "torrent stream has no playable link".to_string())?;
    let requested_file_idx = stream
        .get("fileIdx")
        .and_then(Value::as_i64)
        .map(|v| v as i32);
    let preferred_filename = stream
        .get("behaviorHints")
        .and_then(|hints| hints.get("filename"))
        .and_then(Value::as_str)
        .or_else(|| stream.get("filename").and_then(Value::as_str));
    let sources = stream
        .get("sources")
        .and_then(Value::as_array)
        .map(|items| items.iter().filter_map(Value::as_str).collect::<Vec<_>>())
        .unwrap_or_default();

    let runtime_request = json!({
        "link": link,
        "title": title
            .or_else(|| stream.get("title").and_then(Value::as_str).map(str::to_string))
            .or_else(|| stream.get("name").and_then(Value::as_str).map(str::to_string))
            .unwrap_or_else(|| "Fluxa stream".to_string()),
        "requestedFileIdx": requested_file_idx,
        "preferredFilename": preferred_filename,
        "sources": sources,
        "fileStats": [],
        "rejectedIndex": Value::Null,
        "baseUrl": base_url,
        "play": true,
        "stat": false
    });
    let runtime_json = FluxaCore::torrent_runtime_info_json(&runtime_request.to_string())
        .ok_or_else(|| "torrent runtime info could not be resolved".to_string())?;
    let runtime: Value = serde_json::from_str(&runtime_json)
        .map_err(|e| format!("invalid torrent runtime response: {e}"))?;
    let stream_url = runtime
        .get("streamUrl")
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| "torrent runtime did not return streamUrl".to_string())?;
    let stats_link = runtime
        .get("normalizedLink")
        .and_then(Value::as_str)
        .map(str::to_string)
        .unwrap_or_else(|| link.to_string());
    let selected_file_id = runtime
        .get("selectedFileIdx")
        .and_then(Value::as_i64)
        .map(|v| v as usize);
    torrent_transport::add(&base_url, &stats_link, selected_file_id);
    Ok((
        stream_url,
        base_url,
        stats_link,
        generation,
        selected_file_id,
    ))
}

#[tauri::command]
pub fn stream_magnet_link(stream_json: String) -> Option<String> {
    FluxaCore::stream_magnet_link_json(&stream_json)
}

async fn ensure_healthy_torrent_base_url(
    state: &State<'_, DesktopState>,
    data_dir: &std::path::Path,
) -> Option<String> {
    let base_url = state.torrent_server_base_url.lock().unwrap().clone()?;
    let healthy = tauri::async_runtime::spawn_blocking({
        let base_url = base_url.clone();
        move || torrent_transport::healthy(&base_url)
    })
    .await
    .unwrap_or(false);
    if healthy {
        return Some(base_url);
    }
    let previous_generation = state.torrent_generation.lock().unwrap().take();
    *state.torrent_server_base_url.lock().unwrap() = None;
    *state.torrent_stream_link.lock().unwrap() = None;
    *state.torrent_stream_file_id.lock().unwrap() = None;
    let cleanup_dir = data_dir.to_path_buf();
    let _ = tauri::async_runtime::spawn_blocking(move || {
        let stopped = fluxa_streaming_engine::stop_torrent_server(previous_generation);
        let _ = fs::remove_dir_all(cleanup_dir.join("torrent-cache"));
        stopped
    })
    .await;
    None
}

#[tauri::command]
pub async fn start_torrent_stream(
    state: State<'_, DesktopState>,
    stream_json: String,
    title: Option<String>,
    preferences: Option<Value>,
) -> Result<String, String> {
    let data_dir = state
        .data_dir
        .lock()
        .unwrap()
        .clone()
        .ok_or_else(|| "app data dir is not ready".to_string())?;
    let info_hash = serde_json::from_str::<Value>(&stream_json)
        .ok()
        .and_then(|stream| {
            stream
                .get("infoHash")
                .and_then(Value::as_str)
                .map(str::to_ascii_lowercase)
        });
    let existing_link = state.torrent_stream_link.lock().unwrap().clone();
    let existing_base_url = ensure_healthy_torrent_base_url(&state, &data_dir).await;
    let reuse_existing_server = existing_base_url.is_some();
    let same_torrent = info_hash.as_ref().is_some_and(|hash| {
        existing_link
            .as_ref()
            .is_some_and(|link| link.to_ascii_lowercase().contains(hash))
    });
    if reuse_existing_server && !same_torrent {
        if let (Some(base_url), Some(old_link)) = (existing_base_url.clone(), existing_link) {
            torrent_transport::remove(&base_url, &old_link);
        }
        *state.torrent_stream_link.lock().unwrap() = None;
        *state.torrent_stream_file_id.lock().unwrap() = None;
    }
    let (stream_url, base_url, link, generation, file_id) =
        tauri::async_runtime::spawn_blocking(move || {
            start_torrent_stream_inner(data_dir, stream_json, title, preferences, existing_base_url)
        })
        .await
        .map_err(|e| e.to_string())??;
    *state.torrent_server_base_url.lock().unwrap() = Some(base_url);
    *state.torrent_stream_link.lock().unwrap() = Some(link);
    *state.torrent_stream_file_id.lock().unwrap() = file_id;
    if let Some(generation) = generation {
        *state.torrent_generation.lock().unwrap() = Some(generation);
    }
    Ok(stream_url)
}

pub(crate) async fn resolve_torrent_download_url(
    state: &State<'_, DesktopState>,
    stream_json: String,
) -> Result<String, String> {
    let data_dir = state
        .data_dir
        .lock()
        .unwrap()
        .clone()
        .ok_or_else(|| "app data dir is not ready".to_string())?;
    let existing_base_url = ensure_healthy_torrent_base_url(state, &data_dir).await;
    let (stream_url, base_url, _link, generation, _file_id) =
        tauri::async_runtime::spawn_blocking(move || {
            start_torrent_stream_inner(data_dir, stream_json, None, None, existing_base_url)
        })
        .await
        .map_err(|e| e.to_string())??;
    *state.torrent_server_base_url.lock().unwrap() = Some(base_url);
    if let Some(generation) = generation {
        *state.torrent_generation.lock().unwrap() = Some(generation);
    }
    Ok(stream_url)
}

#[tauri::command]
pub async fn stop_torrent_stream(state: State<'_, DesktopState>) -> Result<bool, String> {
    let was_playing = state.torrent_stream_link.lock().unwrap().take().is_some();
    *state.torrent_stream_file_id.lock().unwrap() = None;
    Ok(was_playing)
}

#[tauri::command]
pub async fn player_torrent_stats(state: State<'_, DesktopState>) -> Result<Option<Value>, String> {
    let base_url = state.torrent_server_base_url.lock().unwrap().clone();
    let link = state.torrent_stream_link.lock().unwrap().clone();
    let file_id = *state.torrent_stream_file_id.lock().unwrap();
    let (Some(base_url), Some(link)) = (base_url, link) else {
        return Ok(None);
    };
    let url = format!("{}/torrents", base_url.trim_end_matches('/'));
    let client = reqwest::Client::new();
    let response = client
        .post(&url)
        .json(&serde_json::json!({ "action": "get", "link": link, "file_id": file_id }))
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await
        .map_err(|e| e.to_string())?;
    let json_val = response.json::<Value>().await.map_err(|e| e.to_string())?;
    Ok(Some(json_val))
}
