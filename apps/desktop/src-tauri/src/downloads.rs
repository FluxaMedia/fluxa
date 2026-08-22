use crate::DesktopState;
use fluxa_core::FluxaCore;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, State};

fn sanitize_file_name(name: &str) -> String {
    let sanitized = name
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-') {
                c
            } else {
                '_'
            }
        })
        .collect::<String>();
    let trimmed = sanitized.trim_matches('.').trim_matches('_');
    if trimmed.is_empty() {
        "download.mp4".to_string()
    } else {
        trimmed.chars().take(180).collect()
    }
}

pub fn resolve_offline_dir(state: &DesktopState) -> Option<PathBuf> {
    if let Some(dir) = state.download_dir.lock().unwrap().clone() {
        return Some(dir);
    }
    let data_dir = state.data_dir.lock().unwrap().clone()?;
    Some(data_dir.join("offline"))
}

#[tauri::command]
pub fn set_download_dir(state: State<DesktopState>, path: Option<String>) -> Result<(), String> {
    *state.download_dir.lock().unwrap() = path.filter(|p| !p.is_empty()).map(PathBuf::from);
    Ok(())
}

#[derive(Clone, Serialize, Deserialize)]
struct ManifestEntry {
    id: String,
    #[serde(rename = "requestJson")]
    request_json: String,
    #[serde(rename = "videoFileName")]
    video_file_name: String,
    title: Option<String>,
    #[serde(rename = "downloadedBytes")]
    downloaded_bytes: u64,
    #[serde(rename = "totalBytes")]
    total_bytes: Option<u64>,
    status: String,
    error: Option<String>,
}

fn manifest_path(offline_dir: &Path) -> PathBuf {
    offline_dir.join("downloads_manifest.json")
}

fn load_manifest(offline_dir: &Path) -> Vec<ManifestEntry> {
    fs::read_to_string(manifest_path(offline_dir))
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

fn save_manifest(offline_dir: &Path, entries: &[ManifestEntry]) {
    if let Ok(json) = serde_json::to_string(entries) {
        let _ = fs::write(manifest_path(offline_dir), json);
    }
}

fn upsert_manifest(offline_dir: &Path, entry: ManifestEntry) {
    let mut entries = load_manifest(offline_dir);
    if let Some(existing) = entries.iter_mut().find(|e| e.id == entry.id) {
        *existing = entry;
    } else {
        entries.push(entry);
    }
    save_manifest(offline_dir, &entries);
}

fn remove_manifest_entry(offline_dir: &Path, id: &str) {
    let mut entries = load_manifest(offline_dir);
    entries.retain(|e| e.id != id);
    save_manifest(offline_dir, &entries);
}

#[derive(Default)]
pub struct DownloadsState {
    active: Mutex<HashMap<String, Arc<AtomicBool>>>,
}

fn extract_title(request_json: &str) -> Option<String> {
    let value: Value = serde_json::from_str(request_json).ok()?;
    value
        .get("meta")
        .and_then(|m| m.get("name"))
        .and_then(Value::as_str)
        .map(str::to_string)
}

fn build_plan(request_json: &str) -> Result<(Value, String, String), String> {
    let plan_json = FluxaCore::offline_download_plan_json(request_json)
        .ok_or_else(|| "offline download plan could not be created".to_string())?;
    let plan: Value = serde_json::from_str(&plan_json)
        .map_err(|e| format!("invalid offline download plan: {e}"))?;
    if plan.get("supported").and_then(Value::as_bool) != Some(true) {
        return Err(plan_json);
    }
    let playback_url = plan
        .get("playbackUrl")
        .and_then(Value::as_str)
        .ok_or_else(|| "offline plan has no playback url".to_string())?
        .to_string();
    let video_file_name = plan
        .get("videoFileName")
        .and_then(Value::as_str)
        .map(sanitize_file_name)
        .ok_or_else(|| "offline plan has no video file name".to_string())?;
    Ok((plan, playback_url, video_file_name))
}

async fn run_download(
    app: AppHandle,
    offline_dir: PathBuf,
    id: String,
    request_json: String,
    playback_url: String,
    video_file_name: String,
    is_local_source: bool,
    cancel: Arc<AtomicBool>,
) {
    let temp_path = offline_dir.join(format!("{video_file_name}.part"));
    let target_path = offline_dir.join(&video_file_name);
    let mut resume_from = fs::metadata(&temp_path).map(|m| m.len()).unwrap_or(0);

    let title = extract_title(&request_json);
    let emit = |downloaded: u64, total: Option<u64>, status: &str, error: Option<&str>| {
        let _ = app.emit(
            "download-progress",
            json!({ "id": id, "title": title, "downloadedBytes": downloaded, "totalBytes": total, "status": status, "error": error }),
        );
    };
    let fail = |offline_dir: &Path, downloaded: u64, total: Option<u64>, message: String| {
        upsert_manifest(
            offline_dir,
            ManifestEntry {
                id: id.clone(),
                request_json: request_json.clone(),
                video_file_name: video_file_name.clone(),
                title: extract_title(&request_json),
                downloaded_bytes: downloaded,
                total_bytes: total,
                status: "failed".to_string(),
                error: Some(message.clone()),
            },
        );
        emit(downloaded, total, "failed", Some(&message));
    };

    let timeout = std::time::Duration::from_secs(60 * 60);
    let client = if is_local_source {
        reqwest::Client::builder()
            .timeout(timeout)
            .build()
            .map_err(|e| e.to_string())
    } else {
        crate::net_guard::vetted_client(&playback_url, timeout).await
    };
    let client = match client {
        Ok(c) => c,
        Err(e) => {
            fail(&offline_dir, resume_from, None, e);
            return;
        }
    };
    let mut req = client.get(&playback_url).header("User-Agent", "Fluxa/1.0");
    if resume_from > 0 {
        req = req.header("Range", format!("bytes={resume_from}-"));
    }
    let mut response = match req.send().await {
        Ok(r) => r,
        Err(e) => {
            fail(&offline_dir, resume_from, None, e.to_string());
            return;
        }
    };
    if !response.status().is_success() {
        fail(
            &offline_dir,
            resume_from,
            None,
            format!("download failed: HTTP {}", response.status()),
        );
        return;
    }
    if resume_from > 0 && response.status() != reqwest::StatusCode::PARTIAL_CONTENT {
        let _ = fs::remove_file(&temp_path);
        resume_from = 0;
    }
    let total = response.content_length().map(|n| n + resume_from);

    let mut file = match fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&temp_path)
    {
        Ok(f) => f,
        Err(e) => {
            fail(&offline_dir, resume_from, total, e.to_string());
            return;
        }
    };
    let mut downloaded = resume_from;
    let mut last_emit = std::time::Instant::now();

    loop {
        if cancel.load(Ordering::Relaxed) {
            let _ = file.flush();
            upsert_manifest(
                &offline_dir,
                ManifestEntry {
                    id: id.clone(),
                    request_json: request_json.clone(),
                    video_file_name: video_file_name.clone(),
                    title: extract_title(&request_json),
                    downloaded_bytes: downloaded,
                    total_bytes: total,
                    status: "paused".to_string(),
                    error: None,
                },
            );
            emit(downloaded, total, "paused", None);
            return;
        }
        let chunk = match response.chunk().await {
            Ok(Some(c)) => c,
            Ok(None) => break,
            Err(e) => {
                fail(&offline_dir, downloaded, total, e.to_string());
                return;
            }
        };
        if let Err(e) = file.write_all(&chunk) {
            fail(&offline_dir, downloaded, total, e.to_string());
            return;
        }
        downloaded += chunk.len() as u64;
        if last_emit.elapsed() > std::time::Duration::from_millis(250) {
            emit(downloaded, total, "downloading", None);
            last_emit = std::time::Instant::now();
        }
    }

    if let Err(e) = file.flush() {
        fail(&offline_dir, downloaded, total, e.to_string());
        return;
    }
    drop(file);
    if target_path.exists() {
        let _ = fs::remove_file(&target_path);
    }
    if let Err(e) = fs::rename(&temp_path, &target_path) {
        fail(&offline_dir, downloaded, total, e.to_string());
        return;
    }
    remove_manifest_entry(&offline_dir, &id);
    emit(downloaded, total, "downloaded", None);
}

fn spawn_download(
    app: AppHandle,
    state: &DesktopState,
    offline_dir: PathBuf,
    id: String,
    request_json: String,
    playback_url: String,
    video_file_name: String,
    is_local_source: bool,
) {
    let cancel = Arc::new(AtomicBool::new(false));
    state
        .downloads
        .active
        .lock()
        .unwrap()
        .insert(id.clone(), cancel.clone());
    tauri::async_runtime::spawn(run_download(
        app,
        offline_dir,
        id,
        request_json,
        playback_url,
        video_file_name,
        is_local_source,
        cancel,
    ));
}

async fn resolve_download_source(
    state: &State<'_, DesktopState>,
    plan: &Value,
    request_json: &str,
    fallback_url: String,
) -> Result<(String, bool), String> {
    if plan.get("isTorrent").and_then(Value::as_bool) != Some(true) {
        return Ok((fallback_url, false));
    }
    let stream_json = serde_json::from_str::<Value>(request_json)
        .ok()
        .and_then(|request| request.get("stream").cloned())
        .ok_or_else(|| "offline download request has no stream".to_string())?
        .to_string();
    let url = crate::resolve_torrent_download_url(state, stream_json).await?;
    Ok((url, true))
}

#[tauri::command]
pub async fn enqueue_offline_download(
    app: AppHandle,
    state: State<'_, DesktopState>,
    request_json: String,
) -> Result<Option<String>, String> {
    let (plan, playback_url, video_file_name) = match build_plan(&request_json) {
        Ok(v) => v,
        Err(plan_json) => return Ok(Some(plan_json)),
    };
    let (download_url, is_local_source) =
        resolve_download_source(&state, &plan, &request_json, playback_url).await?;
    let offline_dir =
        resolve_offline_dir(&state).ok_or_else(|| "app data dir is not ready".to_string())?;
    fs::create_dir_all(&offline_dir).map_err(|e| e.to_string())?;

    let id = video_file_name.clone();
    upsert_manifest(
        &offline_dir,
        ManifestEntry {
            id: id.clone(),
            request_json: request_json.clone(),
            video_file_name: video_file_name.clone(),
            title: extract_title(&request_json),
            downloaded_bytes: 0,
            total_bytes: None,
            status: "downloading".to_string(),
            error: None,
        },
    );
    spawn_download(
        app,
        &state,
        offline_dir,
        id,
        request_json,
        download_url,
        video_file_name.clone(),
        is_local_source,
    );

    let mut queued = plan;
    queued["status"] = json!("downloading");
    queued["videoFileName"] = json!(video_file_name);
    serde_json::to_string(&queued)
        .map(Some)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn pause_offline_download(state: State<DesktopState>, id: String) -> Result<(), String> {
    if let Some(flag) = state.downloads.active.lock().unwrap().get(&id) {
        flag.store(true, Ordering::Relaxed);
    }
    Ok(())
}

#[tauri::command]
pub fn cancel_offline_download(state: State<DesktopState>, id: String) -> Result<(), String> {
    if let Some(flag) = state.downloads.active.lock().unwrap().remove(&id) {
        flag.store(true, Ordering::Relaxed);
    }
    let offline_dir =
        resolve_offline_dir(&state).ok_or_else(|| "app data dir is not ready".to_string())?;
    let temp_path = offline_dir.join(format!("{id}.part"));
    if temp_path.exists() {
        let _ = fs::remove_file(&temp_path);
    }
    remove_manifest_entry(&offline_dir, &id);
    Ok(())
}

#[tauri::command]
pub async fn resume_offline_download(
    app: AppHandle,
    state: State<'_, DesktopState>,
    id: String,
) -> Result<(), String> {
    let offline_dir =
        resolve_offline_dir(&state).ok_or_else(|| "app data dir is not ready".to_string())?;
    let entries = load_manifest(&offline_dir);
    let entry = entries
        .into_iter()
        .find(|e| e.id == id)
        .ok_or_else(|| "no download to resume".to_string())?;
    let (plan, playback_url, video_file_name) = build_plan(&entry.request_json)?;
    let request_json = entry.request_json.clone();
    let (download_url, is_local_source) =
        resolve_download_source(&state, &plan, &request_json, playback_url).await?;
    upsert_manifest(
        &offline_dir,
        ManifestEntry {
            status: "downloading".to_string(),
            error: None,
            ..entry
        },
    );
    spawn_download(
        app,
        &state,
        offline_dir,
        id,
        request_json,
        download_url,
        video_file_name,
        is_local_source,
    );
    Ok(())
}

#[tauri::command]
pub fn list_offline_downloads(state: State<DesktopState>) -> Vec<Value> {
    let offline_dir = match resolve_offline_dir(&state) {
        Some(d) => d,
        None => return vec![],
    };
    if !offline_dir.exists() {
        return vec![];
    }
    let entries = match fs::read_dir(&offline_dir) {
        Ok(e) => e,
        Err(_) => return vec![],
    };
    let mut items: Vec<Value> = entries
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("part") {
                return None;
            }
            if path.file_name().and_then(|n| n.to_str()) == Some("downloads_manifest.json") {
                return None;
            }
            let metadata = fs::metadata(&path).ok()?;
            let size = metadata.len();
            let name = path.file_name()?.to_string_lossy().to_string();
            Some(json!({
                "id": name.clone(),
                "videoFileName": name,
                "path": path.to_string_lossy().to_string(),
                "sizeBytes": size,
                "status": "downloaded",
            }))
        })
        .collect();
    for entry in load_manifest(&offline_dir) {
        items.push(json!({
            "id": entry.id,
            "videoFileName": entry.video_file_name,
            "title": entry.title,
            "downloadedBytes": entry.downloaded_bytes,
            "totalBytes": entry.total_bytes,
            "status": entry.status,
            "error": entry.error,
        }));
    }
    items.sort_by(|a, b| {
        a["videoFileName"]
            .as_str()
            .unwrap_or("")
            .cmp(b["videoFileName"].as_str().unwrap_or(""))
    });
    items
}

#[tauri::command]
pub fn delete_offline_download(
    state: State<DesktopState>,
    file_name: String,
) -> Result<(), String> {
    let offline_dir =
        resolve_offline_dir(&state).ok_or_else(|| "app data dir not ready".to_string())?;
    if let Some(flag) = state.downloads.active.lock().unwrap().remove(&file_name) {
        flag.store(true, Ordering::Relaxed);
    }
    let safe_name = sanitize_file_name(&file_name);
    let path = offline_dir.join(&safe_name);
    if path.exists() {
        fs::remove_file(&path).map_err(|e| e.to_string())?;
    }
    let temp_path = offline_dir.join(format!("{safe_name}.part"));
    if temp_path.exists() {
        let _ = fs::remove_file(&temp_path);
    }
    remove_manifest_entry(&offline_dir, &file_name);
    Ok(())
}
