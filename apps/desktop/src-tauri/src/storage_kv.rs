use super::super::{decrypt_or_legacy, encrypt, open_database, sanitize_key};
use super::super::library_storage_migrations::read_legacy_file;
use crate::DesktopState;
use rusqlite::{params, OptionalExtension};
use serde_json::Value;
use std::fs;
use tauri::State;

#[tauri::command]
pub fn storage_read(state: State<DesktopState>, key: String) -> Option<String> {
    let _storage_lock = state.storage_lock.lock().unwrap();
    let dir = state.data_dir.lock().unwrap().clone()?;
    let storage_key = sanitize_key(&key);
    if let Ok(database) = open_database(&dir) {
        if let Ok(Some(bytes)) = database
            .query_row(
                "SELECT value FROM kv_store WHERE key = ?1",
                [storage_key],
                |row| row.get::<_, Vec<u8>>(0),
            )
            .optional()
        {
            return decrypt_or_legacy(&dir, &bytes);
        }
    }
    read_legacy_file(&dir, &key)
}

#[cfg(target_os = "windows")]
pub(crate) fn read_pref_bool(state: State<DesktopState>, field: &str) -> Option<bool> {
    let raw = storage_read(state, "prefs".to_string())?;
    let value: Value = serde_json::from_str(&raw).ok()?;
    value.get(field)?.as_bool()
}

pub fn read_pref_field(state: State<DesktopState>, field: &str) -> Option<String> {
    let raw = storage_read(state, "prefs".to_string())?;
    let value: Value = serde_json::from_str(&raw).ok()?;
    value.get(field)?.as_str().map(str::to_string)
}

#[tauri::command]
pub fn storage_write(state: State<DesktopState>, key: String, value: String) -> bool {
    let _storage_lock = state.storage_lock.lock().unwrap();
    let dir = match state.data_dir.lock().unwrap().clone() {
        Some(d) => d,
        None => return false,
    };
    if fs::create_dir_all(&dir).is_err() {
        return false;
    }
    let encrypted = match encrypt(&dir, value.as_bytes()) {
        Ok(bytes) => bytes,
        Err(_) => return false,
    };
    let Ok(database) = open_database(&dir) else {
        return false;
    };
    database
        .execute(
            "INSERT INTO kv_store (key, value, updated_at) VALUES (?1, ?2, unixepoch())
         ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at",
            params![sanitize_key(&key), encrypted],
        )
        .is_ok()
}

#[tauri::command]
pub fn storage_delete(state: State<DesktopState>, key: String) -> bool {
    let _storage_lock = state.storage_lock.lock().unwrap();
    let dir = match state.data_dir.lock().unwrap().clone() {
        Some(d) => d,
        None => return false,
    };
    let Ok(database) = open_database(&dir) else {
        return false;
    };
    database
        .execute("DELETE FROM kv_store WHERE key = ?1", [sanitize_key(&key)])
        .is_ok()
}
