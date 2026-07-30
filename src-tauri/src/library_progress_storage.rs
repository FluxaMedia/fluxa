use super::super::{decrypt_or_legacy, encrypt, open_database, sanitize_key};
use super::super::library_storage_migrations::ensure_progress_migrated;
use crate::DesktopState;
use rusqlite::{params, OptionalExtension};
use serde_json::Value;
use tauri::State;

#[tauri::command]
pub fn library_progress_read(
    state: State<DesktopState>,
    profile_key: String,
    media_id: String,
) -> Option<String> {
    let _storage_lock = state.storage_lock.lock().unwrap();
    let dir = state.data_dir.lock().unwrap().clone()?;
    let profile_key = sanitize_key(&profile_key);
    let database = open_database(&dir).ok()?;
    ensure_progress_migrated(&database, &dir, &profile_key).ok()?;
    let value: Option<Vec<u8>> = database
        .query_row(
            "SELECT value FROM library_progress WHERE profile_key = ?1 AND media_id = ?2",
            params![profile_key, media_id],
            |row| row.get(0),
        )
        .optional()
        .ok()?;
    value.and_then(|value| decrypt_or_legacy(&dir, &value))
}

#[tauri::command]
pub fn library_progress_list(state: State<DesktopState>, profile_key: String) -> Option<String> {
    let _storage_lock = state.storage_lock.lock().unwrap();
    let dir = state.data_dir.lock().unwrap().clone()?;
    let profile_key = sanitize_key(&profile_key);
    let database = open_database(&dir).ok()?;
    ensure_progress_migrated(&database, &dir, &profile_key).ok()?;
    let mut statement = database
        .prepare("SELECT media_id, value FROM library_progress WHERE profile_key = ?1")
        .ok()?;
    let rows = statement
        .query_map([profile_key], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, Vec<u8>>(1)?))
        })
        .ok()?;
    let mut progress = serde_json::Map::new();
    for row in rows {
        let (media_id, value) = row.ok()?;
        let value = decrypt_or_legacy(&dir, &value)?;
        progress.insert(media_id, serde_json::from_str(&value).ok()?);
    }
    serde_json::to_string(&Value::Object(progress)).ok()
}

#[tauri::command]
pub fn library_progress_upsert(
    state: State<DesktopState>,
    profile_key: String,
    media_id: String,
    progress_json: String,
) -> bool {
    let _storage_lock = state.storage_lock.lock().unwrap();
    let dir = match state.data_dir.lock().unwrap().clone() {
        Some(dir) => dir,
        None => return false,
    };
    let profile_key = sanitize_key(&profile_key);
    let Ok(database) = open_database(&dir) else {
        return false;
    };
    if ensure_progress_migrated(&database, &dir, &profile_key).is_err() {
        return false;
    }
    let Ok(value) = serde_json::from_str::<Value>(&progress_json) else {
        return false;
    };
    let Ok(value) = serde_json::to_vec(&value).and_then(|value| {
        encrypt(&dir, &value).map_err(|e| serde_json::Error::io(std::io::Error::other(e)))
    }) else {
        return false;
    };
    database
        .execute(
            "INSERT INTO library_progress (profile_key, media_id, value, updated_at)
             VALUES (?1, ?2, ?3, unixepoch())
             ON CONFLICT(profile_key, media_id) DO UPDATE SET
               value = excluded.value, updated_at = excluded.updated_at",
            params![profile_key, media_id, value],
        )
        .is_ok()
}

#[tauri::command]
pub fn library_progress_delete(
    state: State<DesktopState>,
    profile_key: String,
    media_id: String,
) -> bool {
    let _storage_lock = state.storage_lock.lock().unwrap();
    let dir = match state.data_dir.lock().unwrap().clone() {
        Some(dir) => dir,
        None => return false,
    };
    let profile_key = sanitize_key(&profile_key);
    let Ok(database) = open_database(&dir) else {
        return false;
    };
    if ensure_progress_migrated(&database, &dir, &profile_key).is_err() {
        return false;
    }
    database
        .execute(
            "DELETE FROM library_progress WHERE profile_key = ?1 AND media_id = ?2",
            params![profile_key, media_id],
        )
        .is_ok()
}

