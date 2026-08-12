use aes_gcm::aead::{Aead, AeadCore, KeyInit, OsRng};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use rusqlite::Connection;
use rusqlite::params;
use serde_json::{Map, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::ops::{Deref, DerefMut};
use std::time::Duration;

#[cfg(target_os = "windows")]
use crate::DesktopState;

#[path = "library_storage.rs"]
mod library_storage;
#[path = "library_storage_migrations.rs"]
mod library_storage_migrations;

pub use library_storage::{
    library_continue_watching_delete, library_continue_watching_list,
    library_continue_watching_upsert, library_last_watched_delete, library_last_watched_list,
    library_last_watched_upsert, library_progress_delete, library_progress_list,
    library_progress_read, library_progress_upsert, library_progress_upsert_many,
    library_status_list, library_status_set,
    library_watched_list, library_watched_set, read_pref_field, storage_delete, storage_read,
    storage_write,
};
use library_storage_migrations::migrate_legacy_json_files;
fn decrypt_json(dir: &Path, bytes: Vec<u8>) -> Result<Value, String> {
    let text = decrypt_or_legacy(dir, &bytes)
        .ok_or_else(|| "failed to decrypt library value".to_string())?;
    serde_json::from_str(&text).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn library_snapshot(
    state: tauri::State<crate::DesktopState>,
    profile_key: String,
) -> Option<String> {
    let _storage_lock = state.storage_lock.lock().ok()?;
    let dir = state.data_dir.lock().ok()?.clone()?;
    let profile_key = sanitize_key(&profile_key);
    let mut database = open_database(&dir).ok()?;
    library_storage_migrations::ensure_progress_migrated(&database, &dir, &profile_key).ok()?;
    library_storage_migrations::ensure_items_migrated(&database, &dir, &profile_key).ok()?;
    library_storage_migrations::ensure_watched_migrated(&database, &dir, &profile_key).ok()?;
    library_storage_migrations::ensure_last_watched_migrated(&database, &dir, &profile_key).ok()?;
    library_storage_migrations::ensure_continue_watching_migrated(&database, &dir, &profile_key).ok()?;
    let transaction = database.transaction().ok()?;

    let mut progress = Map::new();
    let mut statement = transaction
        .prepare("SELECT media_id, value FROM library_progress WHERE profile_key = ?1")
        .ok()?;
    let rows = statement
        .query_map([&profile_key], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, Vec<u8>>(1)?))
        })
        .ok()?;
    for row in rows {
        let Ok((id, value)) = row else {
            log::warn!("library snapshot: failed to read progress row");
            continue;
        };
        match decrypt_json(&dir, value) {
            Ok(value) => {
                progress.insert(id, value);
            }
            Err(error) => log::warn!("library snapshot: skipping progress row {id}: {error}"),
        }
    }
    drop(statement);

    let mut statuses = Map::new();
    for status in ["watchlist", "completed", "dropped"] {
        let mut items = Vec::new();
        let mut statement = transaction
            .prepare("SELECT value FROM library_items WHERE profile_key = ?1 AND status = ?2 ORDER BY updated_at DESC")
            .ok()?;
        let rows = statement
            .query_map(params![profile_key, status], |row| row.get::<_, Vec<u8>>(0))
            .ok()?;
        for row in rows {
            let Ok(value) = row else {
                log::warn!("library snapshot: failed to read {status} row");
                continue;
            };
            match decrypt_json(&dir, value) {
                Ok(value) => items.push(value),
                Err(error) => log::warn!("library snapshot: skipping {status} row: {error}"),
            }
        }
        statuses.insert(status.to_string(), Value::Array(items));
    }

    let mut watched = Map::new();
    let mut statement = transaction
        .prepare("SELECT video_id FROM watched_videos WHERE profile_key = ?1")
        .ok()?;
    let rows = statement
        .query_map([&profile_key], |row| row.get::<_, String>(0))
        .ok()?;
    for row in rows {
        match row {
            Ok(id) => {
                watched.insert(id, Value::Bool(true));
            }
            Err(error) => {
                log::warn!("library snapshot: skipping watched row: {error}");
            }
        }
    }
    drop(statement);

    let mut last_watched = Map::new();
    let mut statement = transaction
        .prepare("SELECT series_id, value FROM library_last_watched WHERE profile_key = ?1")
        .ok()?;
    let rows = statement
        .query_map([&profile_key], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, Vec<u8>>(1)?))
        })
        .ok()?;
    for row in rows {
        let Ok((id, value)) = row else {
            log::warn!("library snapshot: failed to read last-watched row");
            continue;
        };
        match decrypt_json(&dir, value) {
            Ok(value) => {
                last_watched.insert(id, value);
            }
            Err(error) => log::warn!("library snapshot: skipping last-watched row {id}: {error}"),
        }
    }
    drop(statement);

    let mut continue_watching = Vec::new();
    let mut statement = transaction
        .prepare("SELECT value FROM library_continue_watching WHERE profile_key = ?1 ORDER BY updated_at DESC")
        .ok()?;
    let rows = statement
        .query_map([&profile_key], |row| row.get::<_, Vec<u8>>(0))
        .ok()?;
    for row in rows {
        let Ok(value) = row else {
            log::warn!("library snapshot: failed to read continue-watching row");
            continue;
        };
        match decrypt_json(&dir, value) {
            Ok(value) => continue_watching.push(value),
            Err(error) => log::warn!("library snapshot: skipping continue-watching row: {error}"),
        }
    }

    serde_json::to_string(&serde_json::json!({
        "progress": Value::Object(progress),
        "statuses": Value::Object(statuses),
        "watched": Value::Object(watched),
        "lastWatchedEpisodes": Value::Object(last_watched),
        "externalContinueWatching": Value::Array(continue_watching),
    }))
    .ok()
}

#[cfg(test)]
use library_storage_migrations::{
    ensure_continue_watching_migrated, ensure_items_migrated, ensure_last_watched_migrated,
    ensure_progress_migrated, ensure_watched_migrated,
};

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

const MAGIC: &[u8] = b"FXE1";
const DATABASE_FILE: &str = "fluxa-storage.sqlite3";
const LEGACY_MIGRATION_KEY: &str = "legacy_json_migration_v1";

static STORAGE_KEYS: OnceLock<Mutex<HashMap<PathBuf, Key<Aes256Gcm>>>> = OnceLock::new();
static STORAGE_DATABASE: OnceLock<Mutex<Option<(PathBuf, Connection)>>> = OnceLock::new();

pub struct StorageDatabaseGuard(std::sync::MutexGuard<'static, Option<(PathBuf, Connection)>>);

impl Deref for StorageDatabaseGuard {
    type Target = Connection;

    fn deref(&self) -> &Self::Target {
        &self.0.as_ref().expect("storage database is initialized").1
    }
}

impl DerefMut for StorageDatabaseGuard {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0.as_mut().expect("storage database is initialized").1
    }
}

#[cfg(target_os = "windows")]
pub(crate) fn read_pref_bool(
    state: tauri::State<DesktopState>,
    field: &str,
) -> Option<bool> {
    library_storage::read_pref_bool(state, field)
}

fn key_file_path(dir: &Path) -> PathBuf {
    dir.join(".storage_key")
}

fn load_or_create_key(dir: &Path) -> Result<Key<Aes256Gcm>, String> {
    let keys = STORAGE_KEYS.get_or_init(|| Mutex::new(HashMap::new()));
    if let Some(key) = keys
        .lock()
        .map_err(|_| "storage key cache poisoned".to_string())?
        .get(dir)
        .copied()
    {
        return Ok(key);
    }
    let path = key_file_path(dir);
    if let Ok(bytes) = fs::read(&path) {
        if bytes.len() == 32 {
            let key = *Key::<Aes256Gcm>::from_slice(&bytes);
            keys
                .lock()
                .map_err(|_| "storage key cache poisoned".to_string())?
                .insert(dir.to_path_buf(), key);
            return Ok(key);
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
    keys
        .lock()
        .map_err(|_| "storage key cache poisoned".to_string())?
        .insert(dir.to_path_buf(), key);
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

fn initialize_database(dir: &Path) -> Result<Connection, String> {
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

fn open_database(dir: &Path) -> Result<StorageDatabaseGuard, String> {
    let cache = STORAGE_DATABASE.get_or_init(|| Mutex::new(None));
    let mut guard = cache
        .lock()
        .map_err(|_| "storage database cache poisoned".to_string())?;
    if guard.as_ref().is_none_or(|(path, _)| path != dir) {
        *guard = Some((dir.to_path_buf(), initialize_database(dir)?));
    }
    Ok(StorageDatabaseGuard(guard))
}

pub fn initialize_storage(dir: &Path) -> Result<(), String> {
    match open_database(dir) {
        Ok(_) => Ok(()),
        Err(error) => {
            let database = database_path(dir);
            if database.exists() {
                let stamp = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|value| value.as_secs())
                    .unwrap_or(0);
                let quarantine = dir.join(format!("{DATABASE_FILE}.corrupt.{stamp}"));
                fs::rename(&database, &quarantine).map_err(|rename_error| {
                    format!("storage initialization failed: {error}; quarantine failed: {rename_error}")
                })?;
                for suffix in ["-wal", "-shm"] {
                    let sidecar = PathBuf::from(format!("{}{suffix}", database.display()));
                    if sidecar.exists() {
                        let sidecar_quarantine = PathBuf::from(format!("{}{suffix}", quarantine.display()));
                        let _ = fs::rename(sidecar, sidecar_quarantine);
                    }
                }
                log::error!("storage database quarantined at {} after initialization failure: {error}", quarantine.display());
            }
            let _database = open_database(dir).map_err(|retry_error| {
                format!("storage recovery failed after initialization error: {error}; retry failed: {retry_error}")
            })?;
            Ok(())
        }
    }
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
