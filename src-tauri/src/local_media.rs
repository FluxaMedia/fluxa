use std::fs;
use std::path::{Path, PathBuf};

use serde::Serialize;
use tauri::command;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalMediaFile {
    pub path: String,
    pub file_name: String,
    pub relative_path: String,
    pub size_bytes: u64,
    pub modified_at_ms: Option<u128>,
}

const MAX_FILES: usize = 50_000;
const MAX_DEPTH: usize = 24;

fn walk(root: &Path, current: &Path, depth: usize, files: &mut Vec<LocalMediaFile>) {
    if depth > MAX_DEPTH || files.len() >= MAX_FILES {
        return;
    }
    let Ok(entries) = fs::read_dir(current) else { return };
    for entry in entries.flatten() {
        if files.len() >= MAX_FILES { return; }
        let path = entry.path();
        let Ok(file_type) = entry.file_type() else { continue };
        if file_type.is_dir() {
            if entry.file_name().to_string_lossy().starts_with('.') { continue; }
            walk(root, &path, depth + 1, files);
            continue;
        }
        if !file_type.is_file() { continue; }
        let file_name = entry.file_name().to_string_lossy().into_owned();
        let args = serde_json::json!({ "name": file_name });
        let response = fluxa_core::ffi::core_invoke("localMediaIsVideoFile", &args.to_string());
        let is_video = serde_json::from_str::<serde_json::Value>(&response)
            .ok()
            .and_then(|v| v.get("value").and_then(serde_json::Value::as_bool))
            .unwrap_or(false);
        if !is_video { continue; }
        let metadata = entry.metadata().ok();
        let relative_path = path.strip_prefix(root).unwrap_or(&path).to_string_lossy().into_owned();
        files.push(LocalMediaFile {
            path: path.to_string_lossy().into_owned(),
            file_name,
            relative_path,
            size_bytes: metadata.as_ref().map(|m| m.len()).unwrap_or(0),
            modified_at_ms: metadata.and_then(|m| m.modified().ok()).and_then(|time| {
                time.duration_since(std::time::UNIX_EPOCH).ok().map(|d| d.as_millis())
            }),
        });
    }
}

#[command]
pub fn local_media_scan(root: String) -> Result<Vec<LocalMediaFile>, String> {
    let root_path = PathBuf::from(root.trim());
    if !root_path.is_dir() { return Err("Local media folder does not exist".into()); }
    let mut files = Vec::new();
    walk(&root_path, &root_path, 0, &mut files);
    files.sort_by_cached_key(|file| file.relative_path.to_lowercase());
    Ok(files)
}
