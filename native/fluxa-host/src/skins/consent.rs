use super::*;

fn file(data_dir: &Path) -> PathBuf {
    data_dir.join("skin-scripts.json")
}

fn read(data_dir: &Path) -> HashMap<String, String> {
    std::fs::read_to_string(file(data_dir))
        .ok()
        .and_then(|raw| serde_json::from_str(&raw).ok())
        .unwrap_or_default()
}

pub(super) fn granted(data_dir: &Path, skins: &[Skin]) -> Vec<String> {
    let approved = read(data_dir);
    skins
        .iter()
        .filter(|skin| {
            skin.script_digest()
                .is_some_and(|digest| approved.get(&skin.manifest.id) == Some(&digest))
        })
        .map(|skin| skin.manifest.id.clone())
        .collect()
}

pub(super) fn record(data_dir: &Path, skin: &Skin, allow: bool) {
    let mut approved = read(data_dir);
    match skin.script_digest().filter(|_| allow) {
        Some(digest) => approved.insert(skin.manifest.id.clone(), digest),
        None => approved.remove(&skin.manifest.id),
    };
    if let Ok(raw) = serde_json::to_string(&approved) {
        let _ = std::fs::write(file(data_dir), raw);
    }
}

pub(super) fn allowed(skin: &Skin) -> bool {
    let bundled = DEFAULT
        .read()
        .ok()
        .and_then(|slot| slot.as_ref().map(|default| default.manifest.id.clone()));
    if bundled.as_deref() == Some(skin.manifest.id.as_str()) {
        return true;
    }
    MANAGER
        .lock()
        .is_ok_and(|manager| manager.allowed.contains(&skin.manifest.id))
}
