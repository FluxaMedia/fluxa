use super::*;
use serde_json::Map;

pub(super) const LIMIT: usize = 64 * 1024;

static CACHE: LazyLock<Mutex<HashMap<String, Map<String, Value>>>> =
    LazyLock::new(Default::default);

fn path(kind: &str, id: &str) -> Option<PathBuf> {
    let dir = MANAGER.lock().ok()?.data_dir.clone()?;
    Some(dir.join("skin-data").join(kind).join(format!("{id}.json")))
}

fn load(kind: &str, id: &str) -> Map<String, Value> {
    let key = format!("{kind}/{id}");
    if let Some(found) = CACHE.lock().ok().and_then(|cache| cache.get(&key).cloned()) {
        return found;
    }
    let read: Map<String, Value> = path(kind, id)
        .and_then(|file| std::fs::read_to_string(file).ok())
        .and_then(|raw| serde_json::from_str(&raw).ok())
        .unwrap_or_default();
    if let Ok(mut cache) = CACHE.lock() {
        cache.insert(key, read.clone());
    }
    read
}

fn save(kind: &str, id: &str, map: Map<String, Value>) {
    if let Some(file) = path(kind, id) {
        if let Some(parent) = file.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(raw) = serde_json::to_string(&map) {
            let _ = std::fs::write(file, raw);
        }
    }
    if let Ok(mut cache) = CACHE.lock() {
        cache.insert(format!("{kind}/{id}"), map);
    }
}

pub(super) fn storage(id: &str) -> Map<String, Value> {
    load("storage", id)
}

pub(super) fn apply(id: &str, stored: Map<String, Value>, removed: &[String]) -> bool {
    if stored.is_empty() && removed.is_empty() {
        return true;
    }
    let mut map = load("storage", id);
    for key in removed {
        map.remove(key);
    }
    map.extend(stored);
    if serde_json::to_string(&map).is_ok_and(|raw| raw.len() > LIMIT) {
        return false;
    }
    save("storage", id, map);
    true
}

pub(super) fn settings(extension: &fluxa_skin::Extension) -> Value {
    let saved = load("settings", &extension.id);
    let mut values = Map::new();
    for setting in &extension.settings {
        let value = saved
            .get(&setting.key)
            .filter(|value| setting.accepts(value))
            .cloned()
            .unwrap_or_else(|| setting.fallback());
        values.insert(setting.key.clone(), value);
    }
    Value::Object(values)
}

pub(super) fn set_setting(extension: &fluxa_skin::Extension, key: &str, value: Value) -> bool {
    let Some(setting) = extension.settings.iter().find(|setting| setting.key == key) else {
        return false;
    };
    if !setting.accepts(&value) {
        return false;
    }
    let mut saved = load("settings", &extension.id);
    saved.insert(key.to_owned(), value);
    save("settings", &extension.id, saved);
    true
}
