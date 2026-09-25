use std::path::{Path, PathBuf};

use serde_json::Value;

use super::sanitize_key;

#[derive(Clone)]
pub struct Storage {
    dir: PathBuf,
}

impl Storage {
    pub fn open(dir: PathBuf) -> Result<Self, String> {
        Ok(Self { dir })
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }

    pub fn read_json(&self, key: &str) -> Result<Option<Value>, String> {
        let Some(raw) = local_storage()?
            .get_item(&self.item_key(key))
            .map_err(|_| "localStorage read failed".to_owned())?
        else {
            return Ok(None);
        };
        serde_json::from_str(&raw)
            .map(Some)
            .map_err(|error| error.to_string())
    }

    pub fn write_json(&self, key: &str, value: &Value) -> Result<(), String> {
        local_storage()?
            .set_item(&self.item_key(key), &value.to_string())
            .map_err(|_| "localStorage write failed".to_owned())
    }

    fn item_key(&self, key: &str) -> String {
        format!("{}:{}", self.dir.display(), sanitize_key(key))
    }
}

fn local_storage() -> Result<web_sys::Storage, String> {
    web_sys::window()
        .and_then(|window| window.local_storage().ok().flatten())
        .ok_or_else(|| "localStorage is not available".to_owned())
}
