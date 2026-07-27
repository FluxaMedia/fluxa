use crate::DesktopState;
use aes_gcm::aead::{Aead, AeadCore, KeyInit, OsRng};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use fluxa_core::FluxaCore;
use rusqlite::{params, Connection, OptionalExtension};
use serde::Deserialize;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tauri::State;

pub fn sanitize_key(key: &str) -> String {
    key.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '_' || c == '-' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

// Files written before this existed are plain JSON with no magic prefix -- storage_read
// falls back to reading those as-is so upgrading doesn't wipe existing profiles/library.
const MAGIC: &[u8] = b"FXE1";
const DATABASE_FILE: &str = "fluxa-storage.sqlite3";
const LEGACY_MIGRATION_KEY: &str = "legacy_json_migration_v1";

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct MigrationEntry {
    key: String,
    value: Value,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct LibraryItemMigrationEntry {
    media_id: String,
    status: String,
    value: Value,
}

fn key_file_path(dir: &Path) -> PathBuf {
    dir.join(".storage_key")
}

fn load_or_create_key(dir: &Path) -> Result<Key<Aes256Gcm>, String> {
    let path = key_file_path(dir);
    if let Ok(bytes) = fs::read(&path) {
        if bytes.len() == 32 {
            return Ok(*Key::<Aes256Gcm>::from_slice(&bytes));
        }
    }
    fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    let key = Aes256Gcm::generate_key(&mut OsRng);
    let mut options = fs::OpenOptions::new();
    options.write(true).create(true).truncate(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    use std::io::Write;
    let mut file = options.open(&path).map_err(|e| e.to_string())?;
    file.write_all(key.as_slice()).map_err(|e| e.to_string())?;
    Ok(key)
}

fn encrypt(dir: &Path, plaintext: &[u8]) -> Result<Vec<u8>, String> {
    let key = load_or_create_key(dir)?;
    let cipher = Aes256Gcm::new(&key);
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let ciphertext = cipher
        .encrypt(&nonce, plaintext)
        .map_err(|e| e.to_string())?;
    let mut out = Vec::with_capacity(MAGIC.len() + nonce.len() + ciphertext.len());
    out.extend_from_slice(MAGIC);
    out.extend_from_slice(&nonce);
    out.extend_from_slice(&ciphertext);
    Ok(out)
}

fn decrypt_or_legacy(dir: &Path, bytes: &[u8]) -> Option<String> {
    if !bytes.starts_with(MAGIC) {
        return String::from_utf8(bytes.to_vec()).ok();
    }
    let key = load_or_create_key(dir).ok()?;
    let cipher = Aes256Gcm::new(&key);
    let rest = &bytes[MAGIC.len()..];
    if rest.len() < 12 {
        return None;
    }
    let (nonce_bytes, ciphertext) = rest.split_at(12);
    let nonce = Nonce::from_slice(nonce_bytes);
    let plaintext = cipher.decrypt(nonce, ciphertext).ok()?;
    String::from_utf8(plaintext).ok()
}

fn database_path(dir: &Path) -> PathBuf {
    dir.join(DATABASE_FILE)
}

fn open_database(dir: &Path) -> Result<Connection, String> {
    fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    let connection = Connection::open(database_path(dir)).map_err(|e| e.to_string())?;
    connection
        .busy_timeout(Duration::from_secs(5))
        .map_err(|e| e.to_string())?;
    connection
        .execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA synchronous = FULL;
             PRAGMA foreign_keys = ON;
             CREATE TABLE IF NOT EXISTS storage_meta (
               key TEXT PRIMARY KEY NOT NULL,
               value TEXT NOT NULL
             ) STRICT;
             CREATE TABLE IF NOT EXISTS kv_store (
               key TEXT PRIMARY KEY NOT NULL,
               value BLOB NOT NULL,
               updated_at INTEGER NOT NULL DEFAULT (unixepoch())
             ) STRICT;
             CREATE TABLE IF NOT EXISTS library_progress (
               profile_key TEXT NOT NULL,
               media_id TEXT NOT NULL,
               value BLOB NOT NULL,
               updated_at INTEGER NOT NULL DEFAULT (unixepoch()),
               PRIMARY KEY (profile_key, media_id)
             ) STRICT;
             CREATE INDEX IF NOT EXISTS library_progress_updated_idx
               ON library_progress (profile_key, updated_at DESC);
             CREATE TABLE IF NOT EXISTS library_items (
               profile_key TEXT NOT NULL,
               media_id TEXT NOT NULL,
               status TEXT NOT NULL CHECK(status IN ('watchlist', 'completed', 'dropped')),
               value BLOB NOT NULL,
               updated_at INTEGER NOT NULL DEFAULT (unixepoch()),
               PRIMARY KEY (profile_key, media_id)
             ) STRICT;
             CREATE INDEX IF NOT EXISTS library_items_status_idx
               ON library_items (profile_key, status, updated_at DESC);
             CREATE TABLE IF NOT EXISTS watched_videos (
               profile_key TEXT NOT NULL,
               video_id TEXT NOT NULL,
               watched_at INTEGER NOT NULL DEFAULT (unixepoch()),
               PRIMARY KEY (profile_key, video_id)
             ) STRICT;
             CREATE TABLE IF NOT EXISTS library_last_watched (
               profile_key TEXT NOT NULL,
               series_id TEXT NOT NULL,
               value BLOB NOT NULL,
               updated_at INTEGER NOT NULL DEFAULT (unixepoch()),
               PRIMARY KEY (profile_key, series_id)
             ) STRICT;
             CREATE TABLE IF NOT EXISTS library_continue_watching (
               profile_key TEXT NOT NULL,
               media_id TEXT NOT NULL,
               value BLOB NOT NULL,
               updated_at INTEGER NOT NULL DEFAULT (unixepoch()),
               PRIMARY KEY (profile_key, media_id)
             ) STRICT;
             CREATE INDEX IF NOT EXISTS library_continue_watching_updated_idx
               ON library_continue_watching (profile_key, updated_at DESC);
             CREATE TABLE IF NOT EXISTS library_domain_migrations (
               profile_key TEXT NOT NULL,
               domain TEXT NOT NULL,
               migrated_at INTEGER NOT NULL DEFAULT (unixepoch()),
               PRIMARY KEY (profile_key, domain)
             ) STRICT;
             CREATE TABLE IF NOT EXISTS library_migrations (
               profile_key TEXT PRIMARY KEY NOT NULL,
               progress_imported_at INTEGER NOT NULL DEFAULT (unixepoch())
             ) STRICT;",
        )
        .map_err(|e| e.to_string())?;
    migrate_legacy_json_files(&connection, dir)?;
    Ok(connection)
}

fn ensure_progress_migrated(
    connection: &Connection,
    dir: &Path,
    profile_key: &str,
) -> Result<(), String> {
    let migrated: Option<i64> = connection
        .query_row(
            "SELECT 1 FROM library_migrations WHERE profile_key = ?1",
            [profile_key],
            |row| row.get(0),
        )
        .optional()
        .map_err(|e| e.to_string())?;
    if migrated.is_some() {
        return Ok(());
    }

    let document: Option<Vec<u8>> = connection
        .query_row(
            "SELECT value FROM kv_store WHERE key = ?1",
            [profile_key],
            |row| row.get(0),
        )
        .optional()
        .map_err(|e| e.to_string())?;
    let document = document
        .as_deref()
        .and_then(|bytes| decrypt_or_legacy(dir, bytes))
        .unwrap_or_else(|| "{}".to_string());
    let entries: Vec<MigrationEntry> =
        serde_json::from_str(&FluxaCore::library_progress_entries_json(&document))
            .map_err(|e| e.to_string())?;
    let entries = entries
        .into_iter()
        .map(|entry| {
            let value = serde_json::to_vec(&entry.value).map_err(|e| e.to_string())?;
            Ok((entry.key, encrypt(dir, &value)?))
        })
        .collect::<Result<Vec<_>, String>>()?;
    let tx = connection
        .unchecked_transaction()
        .map_err(|e| e.to_string())?;
    for (media_id, value) in entries {
        tx.execute(
            "INSERT INTO library_progress (profile_key, media_id, value)
             VALUES (?1, ?2, ?3)",
            params![profile_key, media_id, value],
        )
        .map_err(|e| e.to_string())?;
    }
    tx.execute(
        "INSERT INTO library_migrations (profile_key) VALUES (?1)",
        [profile_key],
    )
    .map_err(|e| e.to_string())?;
    tx.commit().map_err(|e| e.to_string())
}

fn library_document(
    connection: &Connection,
    dir: &Path,
    profile_key: &str,
) -> Result<String, String> {
    let document: Option<Vec<u8>> = connection
        .query_row(
            "SELECT value FROM kv_store WHERE key = ?1",
            [profile_key],
            |row| row.get(0),
        )
        .optional()
        .map_err(|e| e.to_string())?;
    Ok(document
        .as_deref()
        .and_then(|bytes| decrypt_or_legacy(dir, bytes))
        .unwrap_or_else(|| "{}".to_string()))
}

fn ensure_items_migrated(
    connection: &Connection,
    dir: &Path,
    profile_key: &str,
) -> Result<(), String> {
    let migrated: Option<i64> = connection
        .query_row(
            "SELECT 1 FROM library_domain_migrations WHERE profile_key = ?1 AND domain = 'items'",
            [profile_key],
            |row| row.get(0),
        )
        .optional()
        .map_err(|e| e.to_string())?;
    if migrated.is_some() {
        return Ok(());
    }
    let document = library_document(connection, dir, profile_key)?;
    let entries: Vec<LibraryItemMigrationEntry> =
        serde_json::from_str(&FluxaCore::library_items_json(&document))
            .map_err(|e| e.to_string())?;
    let tx = connection
        .unchecked_transaction()
        .map_err(|e| e.to_string())?;
    for entry in entries {
        let value = encrypt(
            dir,
            &serde_json::to_vec(&entry.value).map_err(|e| e.to_string())?,
        )?;
        tx.execute("INSERT INTO library_items (profile_key, media_id, status, value) VALUES (?1, ?2, ?3, ?4)", params![profile_key, entry.media_id, entry.status, value]).map_err(|e| e.to_string())?;
    }
    tx.execute(
        "INSERT INTO library_domain_migrations (profile_key, domain) VALUES (?1, 'items')",
        [profile_key],
    )
    .map_err(|e| e.to_string())?;
    tx.commit().map_err(|e| e.to_string())
}

fn ensure_watched_migrated(
    connection: &Connection,
    dir: &Path,
    profile_key: &str,
) -> Result<(), String> {
    let migrated: Option<i64> = connection
        .query_row(
            "SELECT 1 FROM library_domain_migrations WHERE profile_key = ?1 AND domain = 'watched'",
            [profile_key],
            |row| row.get(0),
        )
        .optional()
        .map_err(|e| e.to_string())?;
    if migrated.is_some() {
        return Ok(());
    }
    let document = library_document(connection, dir, profile_key)?;
    let tx = connection
        .unchecked_transaction()
        .map_err(|e| e.to_string())?;
    let video_ids: Vec<String> =
        serde_json::from_str(&FluxaCore::library_watched_video_ids_json(&document))
            .map_err(|e| e.to_string())?;
    for video_id in video_ids {
        tx.execute(
            "INSERT INTO watched_videos (profile_key, video_id) VALUES (?1, ?2)",
            params![profile_key, video_id],
        )
        .map_err(|e| e.to_string())?;
    }
    tx.execute(
        "INSERT INTO library_domain_migrations (profile_key, domain) VALUES (?1, 'watched')",
        [profile_key],
    )
    .map_err(|e| e.to_string())?;
    tx.commit().map_err(|e| e.to_string())
}

fn ensure_last_watched_migrated(
    connection: &Connection,
    dir: &Path,
    profile_key: &str,
) -> Result<(), String> {
    let migrated: Option<i64> = connection
        .query_row(
            "SELECT 1 FROM library_domain_migrations WHERE profile_key = ?1 AND domain = 'last_watched'",
            [profile_key],
            |row| row.get(0),
        )
        .optional()
        .map_err(|e| e.to_string())?;
    if migrated.is_some() {
        return Ok(());
    }
    let document = library_document(connection, dir, profile_key)?;
    let entries: Vec<MigrationEntry> =
        serde_json::from_str(&FluxaCore::library_last_watched_entries_json(&document))
            .map_err(|e| e.to_string())?;
    let tx = connection
        .unchecked_transaction()
        .map_err(|e| e.to_string())?;
    for entry in entries {
        let value = encrypt(
            dir,
            &serde_json::to_vec(&entry.value).map_err(|e| e.to_string())?,
        )?;
        tx.execute(
            "INSERT INTO library_last_watched (profile_key, series_id, value) VALUES (?1, ?2, ?3)",
            params![profile_key, entry.key, value],
        )
        .map_err(|e| e.to_string())?;
    }
    tx.execute(
        "INSERT INTO library_domain_migrations (profile_key, domain) VALUES (?1, 'last_watched')",
        [profile_key],
    )
    .map_err(|e| e.to_string())?;
    tx.commit().map_err(|e| e.to_string())
}

fn ensure_continue_watching_migrated(
    connection: &Connection,
    dir: &Path,
    profile_key: &str,
) -> Result<(), String> {
    let migrated: Option<i64> = connection
        .query_row(
            "SELECT 1 FROM library_domain_migrations WHERE profile_key = ?1 AND domain = 'continue_watching'",
            [profile_key],
            |row| row.get(0),
        )
        .optional()
        .map_err(|e| e.to_string())?;
    if migrated.is_some() {
        return Ok(());
    }
    let document = library_document(connection, dir, profile_key)?;
    let entries: Vec<MigrationEntry> = serde_json::from_str(
        &FluxaCore::library_continue_watching_entries_json(&document),
    )
    .map_err(|e| e.to_string())?;
    let tx = connection
        .unchecked_transaction()
        .map_err(|e| e.to_string())?;
    for entry in entries {
        let value = encrypt(
            dir,
            &serde_json::to_vec(&entry.value).map_err(|e| e.to_string())?,
        )?;
        tx.execute(
            "INSERT INTO library_continue_watching (profile_key, media_id, value) VALUES (?1, ?2, ?3)",
            params![profile_key, entry.key, value],
        )
        .map_err(|e| e.to_string())?;
    }
    tx.execute(
        "INSERT INTO library_domain_migrations (profile_key, domain) VALUES (?1, 'continue_watching')",
        [profile_key],
    )
    .map_err(|e| e.to_string())?;
    tx.commit().map_err(|e| e.to_string())
}

/// Imports every legacy root-level storage document in a single transaction. The old
/// files are renamed only after that transaction commits, so an interrupted upgrade
/// always has a complete source to retry from.
fn migrate_legacy_json_files(connection: &Connection, dir: &Path) -> Result<(), String> {
    let migrated: Option<String> = connection
        .query_row(
            "SELECT value FROM storage_meta WHERE key = ?1",
            [LEGACY_MIGRATION_KEY],
            |row| row.get(0),
        )
        .optional()
        .map_err(|e| e.to_string())?;
    if migrated.as_deref() == Some("complete") {
        return Ok(());
    }

    let mut legacy_files = Vec::<(String, Vec<u8>, PathBuf)>::new();
    for entry in fs::read_dir(dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if !entry.file_type().map_err(|e| e.to_string())?.is_file()
            || path.extension().and_then(|ext| ext.to_str()) != Some("json")
        {
            continue;
        }
        let Some(stem) = path.file_stem().and_then(|name| name.to_str()) else {
            continue;
        };
        legacy_files.push((
            stem.to_owned(),
            fs::read(&path).map_err(|e| e.to_string())?,
            path,
        ));
    }

    let tx = connection
        .unchecked_transaction()
        .map_err(|e| e.to_string())?;
    for (key, bytes, _) in &legacy_files {
        tx.execute(
            "INSERT INTO kv_store (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO NOTHING",
            params![key, bytes],
        )
        .map_err(|e| e.to_string())?;
    }
    tx.execute(
        "INSERT INTO storage_meta (key, value) VALUES (?1, 'complete')
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        [LEGACY_MIGRATION_KEY],
    )
    .map_err(|e| e.to_string())?;
    tx.commit().map_err(|e| e.to_string())?;

    for (_, _, path) in legacy_files {
        let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        let backup = path.with_file_name(format!("{file_name}.migrated-backup"));
        // Keep the original if a prior successful migration already made a backup.
        // The database takes precedence, so this does not affect normal reads.
        if !backup.exists() {
            let _ = fs::rename(&path, backup);
        }
    }
    Ok(())
}

fn read_legacy_file(dir: &Path, key: &str) -> Option<String> {
    let path = dir.join(format!("{}.json", sanitize_key(key)));
    let bytes = fs::read(path).ok()?;
    decrypt_or_legacy(dir, &bytes)
}

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

pub fn read_pref_bool(state: State<DesktopState>, field: &str) -> Option<bool> {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp_dir() -> PathBuf {
        static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let n = COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let dir =
            std::env::temp_dir().join(format!("fluxa-storage-test-{}-{n}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn round_trips_through_encryption() {
        let dir = tmp_dir();
        let encrypted = encrypt(&dir, b"{\"token\":\"secret\"}").unwrap();
        assert!(encrypted.starts_with(MAGIC));
        assert_eq!(
            decrypt_or_legacy(&dir, &encrypted).unwrap(),
            "{\"token\":\"secret\"}"
        );
    }

    #[test]
    fn falls_back_to_legacy_plaintext_without_magic_prefix() {
        let dir = tmp_dir();
        let legacy = b"{\"token\":\"secret\"}".to_vec();
        assert_eq!(
            decrypt_or_legacy(&dir, &legacy).unwrap(),
            "{\"token\":\"secret\"}"
        );
    }

    #[test]
    fn reuses_the_same_key_across_calls() {
        let dir = tmp_dir();
        let a = encrypt(&dir, b"first").unwrap();
        let b = encrypt(&dir, b"second").unwrap();
        assert_eq!(decrypt_or_legacy(&dir, &a).unwrap(), "first");
        assert_eq!(decrypt_or_legacy(&dir, &b).unwrap(), "second");
    }

    #[test]
    fn migrates_legacy_json_to_sqlite_before_retiring_the_source_file() {
        let dir = tmp_dir();
        let legacy_path = dir.join("library_guest.json");
        let legacy_value = br#"{"progress":{"movie":{"timeOffset":42}}}"#;
        fs::write(&legacy_path, encrypt(&dir, legacy_value).unwrap()).unwrap();

        let database = open_database(&dir).unwrap();
        let imported: Vec<u8> = database
            .query_row(
                "SELECT value FROM kv_store WHERE key = 'library_guest'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(
            decrypt_or_legacy(&dir, &imported).unwrap(),
            String::from_utf8(legacy_value.to_vec()).unwrap()
        );
        assert!(!legacy_path.exists());
        assert!(dir.join("library_guest.json.migrated-backup").exists());

        // A normal write after migration remains authoritative; a later app start
        // must never import the backup over it.
        database
            .execute(
                "UPDATE kv_store SET value = ?1 WHERE key = 'library_guest'",
                [encrypt(&dir, br#"{"progress":{"movie":{"timeOffset":84}}}"#).unwrap()],
            )
            .unwrap();
        drop(database);
        let database = open_database(&dir).unwrap();
        let current: Vec<u8> = database
            .query_row(
                "SELECT value FROM kv_store WHERE key = 'library_guest'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert!(decrypt_or_legacy(&dir, &current).unwrap().contains("84"));
    }

    #[test]
    fn migrates_profile_progress_into_independent_rows() {
        let dir = tmp_dir();
        let database = open_database(&dir).unwrap();
        let library = br#"{"progress":{"movie-a":{"timeOffset":15},"series-b":{"timeOffset":30}}}"#;
        database
            .execute(
                "INSERT INTO kv_store (key, value) VALUES ('library_guest', ?1)",
                [encrypt(&dir, library).unwrap()],
            )
            .unwrap();

        ensure_progress_migrated(&database, &dir, "library_guest").unwrap();
        let count: i64 = database
            .query_row(
                "SELECT COUNT(*) FROM library_progress WHERE profile_key = 'library_guest'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 2);

        let stored: Vec<u8> = database
            .query_row(
                "SELECT value FROM library_progress WHERE profile_key = 'library_guest' AND media_id = 'movie-a'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(
            decrypt_or_legacy(&dir, &stored).unwrap(),
            r#"{"timeOffset":15}"#
        );
    }

    #[test]
    fn migrates_last_watched_episodes_into_independent_rows() {
        let dir = tmp_dir();
        let database = open_database(&dir).unwrap();
        let library = br#"{"lastWatchedEpisodes":{"series-a":{"lastVideoId":"series-a:1:2"}}}"#;
        database
            .execute(
                "INSERT INTO kv_store (key, value) VALUES ('library_guest', ?1)",
                [encrypt(&dir, library).unwrap()],
            )
            .unwrap();

        ensure_last_watched_migrated(&database, &dir, "library_guest").unwrap();
        let stored: Vec<u8> = database
            .query_row(
                "SELECT value FROM library_last_watched WHERE profile_key = 'library_guest' AND series_id = 'series-a'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(
            decrypt_or_legacy(&dir, &stored).unwrap(),
            r#"{"lastVideoId":"series-a:1:2"}"#
        );

        // Re-running the migration must not duplicate rows.
        ensure_last_watched_migrated(&database, &dir, "library_guest").unwrap();
        let count: i64 = database
            .query_row(
                "SELECT COUNT(*) FROM library_last_watched WHERE profile_key = 'library_guest'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn migrates_external_continue_watching_into_independent_rows() {
        let dir = tmp_dir();
        let database = open_database(&dir).unwrap();
        let library = br#"{"externalContinueWatching":[{"id":"series-a","name":"Show A"}]}"#;
        database
            .execute(
                "INSERT INTO kv_store (key, value) VALUES ('library_guest', ?1)",
                [encrypt(&dir, library).unwrap()],
            )
            .unwrap();

        ensure_continue_watching_migrated(&database, &dir, "library_guest").unwrap();
        let stored: Vec<u8> = database
            .query_row(
                "SELECT value FROM library_continue_watching WHERE profile_key = 'library_guest' AND media_id = 'series-a'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(
            decrypt_or_legacy(&dir, &stored).unwrap(),
            r#"{"id":"series-a","name":"Show A"}"#
        );

        // Re-running the migration must not duplicate rows.
        ensure_continue_watching_migrated(&database, &dir, "library_guest").unwrap();
        let count: i64 = database
            .query_row(
                "SELECT COUNT(*) FROM library_continue_watching WHERE profile_key = 'library_guest'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn migrates_watchlist_completed_dropped_into_independent_rows() {
        let dir = tmp_dir();
        let database = open_database(&dir).unwrap();
        let library = br#"{"watchlist":[{"id":"movie-a"}],"completed":[{"id":"movie-b"}],"dropped":[{"id":"movie-c"}]}"#;
        database
            .execute(
                "INSERT INTO kv_store (key, value) VALUES ('library_guest', ?1)",
                [encrypt(&dir, library).unwrap()],
            )
            .unwrap();

        ensure_items_migrated(&database, &dir, "library_guest").unwrap();
        let status: String = database
            .query_row(
                "SELECT status FROM library_items WHERE profile_key = 'library_guest' AND media_id = 'movie-b'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(status, "completed");

        // Re-running the migration must not duplicate rows.
        ensure_items_migrated(&database, &dir, "library_guest").unwrap();
        let count: i64 = database
            .query_row(
                "SELECT COUNT(*) FROM library_items WHERE profile_key = 'library_guest'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 3);
    }

    #[test]
    fn migrates_watched_videos_into_independent_rows() {
        let dir = tmp_dir();
        let database = open_database(&dir).unwrap();
        let library = br#"{"watched":{"video-a":true,"video-b":false}}"#;
        database
            .execute(
                "INSERT INTO kv_store (key, value) VALUES ('library_guest', ?1)",
                [encrypt(&dir, library).unwrap()],
            )
            .unwrap();

        ensure_watched_migrated(&database, &dir, "library_guest").unwrap();
        let count: i64 = database
            .query_row(
                "SELECT COUNT(*) FROM watched_videos WHERE profile_key = 'library_guest'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        // Only entries explicitly marked true are imported.
        assert_eq!(count, 1);

        // Re-running the migration must not duplicate rows.
        ensure_watched_migrated(&database, &dir, "library_guest").unwrap();
        let count: i64 = database
            .query_row(
                "SELECT COUNT(*) FROM watched_videos WHERE profile_key = 'library_guest'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }
}
