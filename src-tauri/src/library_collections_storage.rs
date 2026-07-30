use super::super::{decrypt_or_legacy, encrypt, open_database, sanitize_key};
use super::super::library_storage_migrations::{ensure_items_migrated, ensure_watched_migrated};
use crate::DesktopState;
use rusqlite::params;
use serde_json::Value;
use tauri::State;

#[tauri::command]
pub fn library_status_set(
    state: State<DesktopState>,
    profile_key: String,
    media_id: String,
    status: Option<String>,
    item_json: Option<String>,
) -> bool {
    let _storage_lock = state.storage_lock.lock().unwrap();
    let Some(dir) = state.data_dir.lock().unwrap().clone() else {
        return false;
    };
    let profile_key = sanitize_key(&profile_key);
    let Ok(database) = open_database(&dir) else {
        return false;
    };
    if ensure_items_migrated(&database, &dir, &profile_key).is_err() {
        return false;
    }
    match (status, item_json) {
        (Some(status), Some(item_json))
            if matches!(status.as_str(), "watchlist" | "completed" | "dropped") =>
        {
            let Ok(item) = serde_json::from_str::<Value>(&item_json) else {
                return false;
            };
            let Ok(value) = serde_json::to_vec(&item)
                .map_err(|e| e.to_string())
                .and_then(|v| encrypt(&dir, &v))
            else {
                return false;
            };
            database.execute("INSERT INTO library_items (profile_key, media_id, status, value, updated_at) VALUES (?1, ?2, ?3, ?4, unixepoch()) ON CONFLICT(profile_key, media_id) DO UPDATE SET status=excluded.status, value=excluded.value, updated_at=excluded.updated_at", params![profile_key, media_id, status, value]).is_ok()
        }
        (None, _) => database
            .execute(
                "DELETE FROM library_items WHERE profile_key = ?1 AND media_id = ?2",
                params![profile_key, media_id],
            )
            .is_ok(),
        _ => false,
    }
}

#[tauri::command]
pub fn library_status_list(state: State<DesktopState>, profile_key: String) -> Option<String> {
    let _storage_lock = state.storage_lock.lock().unwrap();
    let dir = state.data_dir.lock().unwrap().clone()?;
    let profile_key = sanitize_key(&profile_key);
    let database = open_database(&dir).ok()?;
    ensure_items_migrated(&database, &dir, &profile_key).ok()?;
    let mut lists = serde_json::Map::new();
    for status in ["watchlist", "completed", "dropped"] {
        let mut statement = database.prepare("SELECT value FROM library_items WHERE profile_key = ?1 AND status = ?2 ORDER BY updated_at DESC").ok()?;
        let rows = statement
            .query_map(params![profile_key, status], |row| row.get::<_, Vec<u8>>(0))
            .ok()?;
        let mut items = Vec::new();
        for row in rows {
            items.push(serde_json::from_str::<Value>(&decrypt_or_legacy(&dir, &row.ok()?)?).ok()?);
        }
        lists.insert(status.to_owned(), Value::Array(items));
    }
    serde_json::to_string(&Value::Object(lists)).ok()
}

#[tauri::command]
pub fn library_watched_set(
    state: State<DesktopState>,
    profile_key: String,
    video_id: String,
    watched: bool,
) -> bool {
    let _storage_lock = state.storage_lock.lock().unwrap();
    let Some(dir) = state.data_dir.lock().unwrap().clone() else {
        return false;
    };
    let profile_key = sanitize_key(&profile_key);
    let Ok(database) = open_database(&dir) else {
        return false;
    };
    if ensure_watched_migrated(&database, &dir, &profile_key).is_err() {
        return false;
    }
    if watched {
        database.execute("INSERT INTO watched_videos (profile_key, video_id) VALUES (?1, ?2) ON CONFLICT(profile_key, video_id) DO UPDATE SET watched_at=unixepoch()", params![profile_key, video_id]).is_ok()
    } else {
        database
            .execute(
                "DELETE FROM watched_videos WHERE profile_key = ?1 AND video_id = ?2",
                params![profile_key, video_id],
            )
            .is_ok()
    }
}

#[tauri::command]
pub fn library_watched_list(state: State<DesktopState>, profile_key: String) -> Option<String> {
    let _storage_lock = state.storage_lock.lock().unwrap();
    let dir = state.data_dir.lock().unwrap().clone()?;
    let profile_key = sanitize_key(&profile_key);
    let database = open_database(&dir).ok()?;
    ensure_watched_migrated(&database, &dir, &profile_key).ok()?;
    let mut statement = database
        .prepare("SELECT video_id FROM watched_videos WHERE profile_key = ?1")
        .ok()?;
    let rows = statement
        .query_map([profile_key], |row| row.get::<_, String>(0))
        .ok()?;
    let mut watched = serde_json::Map::new();
    for row in rows {
        watched.insert(row.ok()?, Value::Bool(true));
    }
    serde_json::to_string(&Value::Object(watched)).ok()
}
