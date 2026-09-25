use aes_gcm::aead::{Aead, Generate, KeyInit};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use rusqlite::{Connection, OptionalExtension, params};
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

const MAGIC: &[u8] = b"FXE1";
const DATABASE_FILE: &str = "fluxa-storage.sqlite3";

#[derive(Clone)]
pub struct Storage {
    dir: PathBuf,
    key: Key<Aes256Gcm>,
}

impl Storage {
    pub fn open_default() -> Result<Self, String> {
        let dir = data_dir()?;
        Self::open(dir)
    }

    pub fn open(dir: PathBuf) -> Result<Self, String> {
        fs::create_dir_all(&dir).map_err(|error| error.to_string())?;
        let key = load_or_create_key(&dir)?;
        let storage = Self { dir, key };
        storage.initialize_database()?;
        Ok(storage)
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }

    pub fn read_json(&self, key: &str) -> Result<Option<Value>, String> {
        let connection = self.connection()?;
        let storage_key = sanitize_key(key);
        let bytes: Option<Vec<u8>> = connection
            .query_row(
                "SELECT value FROM kv_store WHERE key = ?1",
                [storage_key],
                |row| row.get(0),
            )
            .optional()
            .map_err(|error| error.to_string())?;
        let Some(bytes) = bytes else {
            return Ok(None);
        };
        let text = self.decrypt(&bytes)?;
        serde_json::from_str(&text)
            .map(Some)
            .map_err(|error| error.to_string())
    }

    pub fn write_json(&self, key: &str, value: &Value) -> Result<(), String> {
        let connection = self.connection()?;
        let encrypted = self.encrypt(value.to_string().as_bytes())?;
        connection
            .execute(
                "INSERT INTO kv_store (key, value, updated_at) VALUES (?1, ?2, unixepoch())
                 ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at",
                params![sanitize_key(key), encrypted],
            )
            .map_err(|error| error.to_string())?;
        Ok(())
    }

    pub fn library_key(profile_id: &str) -> String {
        format!("library_{}", sanitize_key(profile_id))
    }

    pub fn prefs_key(profile_id: &str) -> String {
        format!("prefs_{}", sanitize_key(profile_id))
    }

    pub fn addons_key(owner_id: &str) -> String {
        format!("addons_{}", sanitize_key(owner_id))
    }

    fn connection(&self) -> Result<Connection, String> {
        let connection =
            Connection::open(self.dir.join(DATABASE_FILE)).map_err(|error| error.to_string())?;
        connection
            .busy_timeout(std::time::Duration::from_secs(5))
            .map_err(|error| error.to_string())?;
        Ok(connection)
    }

    fn initialize_database(&self) -> Result<(), String> {
        let connection = self.connection()?;
        connection
            .execute_batch(
                "PRAGMA journal_mode = WAL;
                 PRAGMA synchronous = FULL;
                 CREATE TABLE IF NOT EXISTS kv_store (
                   key TEXT PRIMARY KEY NOT NULL,
                   value BLOB NOT NULL,
                   updated_at INTEGER NOT NULL DEFAULT (unixepoch())
                 ) STRICT;",
            )
            .map_err(|error| error.to_string())
    }

    fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>, String> {
        let cipher = Aes256Gcm::new(&self.key);
        let nonce = Nonce::generate();
        let ciphertext = cipher
            .encrypt(&nonce, plaintext)
            .map_err(|error| error.to_string())?;
        let mut output = Vec::with_capacity(MAGIC.len() + nonce.len() + ciphertext.len());
        output.extend_from_slice(MAGIC);
        output.extend_from_slice(&nonce);
        output.extend_from_slice(&ciphertext);
        Ok(output)
    }

    fn decrypt(&self, bytes: &[u8]) -> Result<String, String> {
        if !bytes.starts_with(MAGIC) {
            return String::from_utf8(bytes.to_vec()).map_err(|error| error.to_string());
        }
        let rest = &bytes[MAGIC.len()..];
        if rest.len() < 12 {
            return Err("stored value has an invalid nonce".to_owned());
        }
        let (nonce_bytes, ciphertext) = rest.split_at(12);
        let nonce = Nonce::try_from(nonce_bytes).map_err(|error| error.to_string())?;
        let plaintext = Aes256Gcm::new(&self.key)
            .decrypt(&nonce, ciphertext)
            .map_err(|error| error.to_string())?;
        String::from_utf8(plaintext).map_err(|error| error.to_string())
    }
}

pub fn data_dir() -> Result<PathBuf, String> {
    if let Some(path) = std::env::var_os("FLUXA_DATA_DIR") {
        return Ok(PathBuf::from(path));
    }
    #[cfg(target_os = "windows")]
    let base = std::env::var_os("APPDATA")
        .map(PathBuf::from)
        .ok_or_else(|| "APPDATA is not available".to_owned())?;
    #[cfg(target_os = "macos")]
    let base = std::env::var_os("HOME")
        .map(PathBuf::from)
        .ok_or_else(|| "HOME is not available".to_owned())?
        .join("Library/Application Support");
    #[cfg(all(unix, not(target_os = "macos")))]
    let base = std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".local/share")))
        .ok_or_else(|| "no desktop data directory is available".to_owned())?;
    Ok(base.join("com.fluxa.desktop").join("fluxa"))
}

fn load_or_create_key(dir: &Path) -> Result<Key<Aes256Gcm>, String> {
    let path = dir.join(".storage_key");
    if let Ok(bytes) = fs::read(&path) {
        if bytes.len() == 32 {
            return Key::<Aes256Gcm>::try_from(bytes.as_slice()).map_err(|error| error.to_string());
        }
    }
    let key = Key::<Aes256Gcm>::generate();
    let mut options = fs::OpenOptions::new();
    options.write(true).create(true).truncate(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    use std::io::Write;
    let mut file = options.open(&path).map_err(|error| error.to_string())?;
    file.write_all(key.as_slice())
        .map_err(|error| error.to_string())?;
    Ok(key)
}

pub fn sanitize_key(key: &str) -> String {
    key.chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || character == '_' || character == '-' {
                character
            } else {
                '_'
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temporary_directory() -> PathBuf {
        static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let counter = COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        std::env::temp_dir().join(format!(
            "fluxa-native-storage-{}-{counter}",
            std::process::id()
        ))
    }

    #[test]
    fn encrypted_storage_round_trip() {
        let directory = temporary_directory();
        let storage = Storage::open(directory.clone()).expect("open storage");
        let value = serde_json::json!({"name": "Fluxa", "count": 3});
        storage
            .write_json("round_trip", &value)
            .expect("write value");
        assert_eq!(
            storage.read_json("round_trip").expect("read value"),
            Some(value)
        );
        let _ = fs::remove_dir_all(directory);
    }
}
