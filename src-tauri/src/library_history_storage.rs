use super::super::{decrypt_or_legacy, encrypt, open_database, sanitize_key};
use super::super::library_storage_migrations::{
    ensure_continue_watching_migrated, ensure_last_watched_migrated,
};
use crate::DesktopState;
use rusqlite::params;
use serde_json::Value;
use tauri::State;

#[tauri::command]
pub fn library_last_watched_list(
    state: State<DesktopState>,
    profile_key: String,
) -> Option<String> {
    let _storage_lock = state.storage_lock.lock().unwrap();
    let dir = state.data_dir.lock().unwrap().clone()?;
    let profile_key = sanitize_key(&profile_key);
    let database = open_database(&dir).ok()?;
    ensure_last_watched_migrated(&database, &dir, &profile_key).ok()?;
    let mut statement = database
        .prepare("SELECT series_id, value FROM library_last_watched WHERE profile_key = ?1")
        .ok()?;
    let rows = statement
        .query_map([profile_key], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, Vec<u8>>(1)?))
        })
        .ok()?;
    let mut entries = serde_json::Map::new();
    for row in rows {
        let (series_id, value) = row.ok()?;
        let value = decrypt_or_legacy(&dir, &value)?;
        entries.insert(series_id, serde_json::from_str(&value).ok()?);
    }
    serde_json::to_string(&Value::Object(entries)).ok()
}

#[tauri::command]
pub fn library_last_watched_upsert(
    state: State<DesktopState>,
    profile_key: String,
    series_id: String,
    entry_json: String,
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
    if ensure_last_watched_migrated(&database, &dir, &profile_key).is_err() {
        return false;
    }
    let Ok(value) = serde_json::from_str::<Value>(&entry_json) else {
        return false;
    };
    let Ok(value) = serde_json::to_vec(&value).and_then(|value| {
        encrypt(&dir, &value).map_err(|e| serde_json::Error::io(std::io::Error::other(e)))
    }) else {
        return false;
    };
    database
        .execute(
            "INSERT INTO library_last_watched (profile_key, series_id, value, updated_at)
             VALUES (?1, ?2, ?3, unixepoch())
             ON CONFLICT(profile_key, series_id) DO UPDATE SET
               value = excluded.value, updated_at = excluded.updated_at",
            params![profile_key, series_id, value],
        )
        .is_ok()
}

#[tauri::command]
pub fn library_last_watched_delete(
    state: State<DesktopState>,
    profile_key: String,
    series_id: String,
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
    if ensure_last_watched_migrated(&database, &dir, &profile_key).is_err() {
        return false;
    }
    database
        .execute(
            "DELETE FROM library_last_watched WHERE profile_key = ?1 AND series_id = ?2",
            params![profile_key, series_id],
        )
        .is_ok()
}

#[tauri::command]
pub fn library_continue_watching_list(
    state: State<DesktopState>,
    profile_key: String,
) -> Option<String> {
    let _storage_lock = state.storage_lock.lock().unwrap();
    let dir = state.data_dir.lock().unwrap().clone()?;
    let profile_key = sanitize_key(&profile_key);
    let database = open_database(&dir).ok()?;
    ensure_continue_watching_migrated(&database, &dir, &profile_key).ok()?;
    let mut statement = database
        .prepare("SELECT value FROM library_continue_watching WHERE profile_key = ?1 ORDER BY updated_at DESC")
        .ok()?;
    let rows = statement
        .query_map([profile_key], |row| row.get::<_, Vec<u8>>(0))
        .ok()?;
    let mut items = Vec::new();
    for row in rows {
        items.push(serde_json::from_str::<Value>(&decrypt_or_legacy(&dir, &row.ok()?)?).ok()?);
    }
    serde_json::to_string(&Value::Array(items)).ok()
}

#[tauri::command]
pub fn library_continue_watching_upsert(
    state: State<DesktopState>,
    profile_key: String,
    media_id: String,
    item_json: String,
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
    if ensure_continue_watching_migrated(&database, &dir, &profile_key).is_err() {
        return false;
    }
    let Ok(value) = serde_json::from_str::<Value>(&item_json) else {
        return false;
    };
    let Ok(value) = serde_json::to_vec(&value).and_then(|value| {
        encrypt(&dir, &value).map_err(|e| serde_json::Error::io(std::io::Error::other(e)))
    }) else {
        return false;
    };
    database
        .execute(
            "INSERT INTO library_continue_watching (profile_key, media_id, value, updated_at)
             VALUES (?1, ?2, ?3, unixepoch())
             ON CONFLICT(profile_key, media_id) DO UPDATE SET
               value = excluded.value, updated_at = excluded.updated_at",
            params![profile_key, media_id, value],
        )
        .is_ok()
}

#[tauri::command]
pub fn library_continue_watching_delete(
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
    if ensure_continue_watching_migrated(&database, &dir, &profile_key).is_err() {
        return false;
    }
    database
        .execute(
            "DELETE FROM library_continue_watching WHERE profile_key = ?1 AND media_id = ?2",
            params![profile_key, media_id],
        )
        .is_ok()
}
