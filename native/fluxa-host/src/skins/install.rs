use super::*;

pub fn activate(data_dir: &Path, id: Option<&str>) -> std::io::Result<()> {
    let file = data_dir.join("active-skin");
    match id {
        Some(id) => std::fs::write(&file, id)?,
        None => {
            let _ = std::fs::remove_file(&file);
        }
    }
    load(data_dir);
    if let Ok(mut cache) = CACHE.lock() {
        cache.clear();
    }
    if let Ok(mut data) = DATA.lock() {
        data.clear();
    }
    Ok(())
}

pub fn install(source: &Path, data_dir: &Path) -> Result<String, Vec<String>> {
    fluxa_skin::install(source, &data_dir.join("skins"))
}

pub(super) fn github_archive(url: &str) -> String {
    let trimmed = url.trim().trim_end_matches('/').trim_end_matches(".git");
    let Some(rest) = trimmed
        .strip_prefix("https://github.com/")
        .filter(|_| !trimmed.ends_with(".zip"))
    else {
        return trimmed.to_owned();
    };
    let mut parts = rest.split('/');
    let (Some(owner), Some(repo)) = (parts.next(), parts.next()) else {
        return trimmed.to_owned();
    };
    let branch = match (parts.next(), parts.next()) {
        (Some("tree"), Some(branch)) => branch,
        _ => "HEAD",
    };
    format!("https://github.com/{owner}/{repo}/archive/{branch}.zip")
}

pub(super) fn set_message(message: Option<String>, busy: bool) {
    if let Ok(mut manager) = MANAGER.lock() {
        manager.message = message;
        manager.busy = busy;
        manager.generation += 1;
    }
}

pub(crate) fn install_from(source: &str) {
    let source = source.trim().to_owned();
    let Some(data_dir) = MANAGER
        .lock()
        .ok()
        .and_then(|manager| manager.data_dir.clone())
    else {
        return;
    };
    if source.is_empty() {
        return;
    }
    if !source.starts_with("http://") && !source.starts_with("https://") {
        finish_install(&data_dir, Path::new(&source));
        return;
    }
    download_and_install(data_dir, source);
}

#[cfg(any(
    target_os = "linux",
    target_os = "macos",
    target_os = "windows",
    target_os = "android"
))]
pub(super) fn download_and_install(data_dir: PathBuf, source: String) {
    if MANAGER.lock().is_ok_and(|manager| manager.busy) {
        return;
    }
    set_message(Some("skins.installing".to_owned()), true);
    let url = github_archive(&source);
    std::thread::spawn(move || match fetch_skin(&data_dir, &url) {
        Ok(path) => {
            finish_install(&data_dir, &path);
            let _ = std::fs::remove_dir_all(&path).or_else(|_| std::fs::remove_file(&path));
        }
        Err(error) => set_message(Some(error), false),
    });
}

#[cfg(any(
    target_os = "linux",
    target_os = "macos",
    target_os = "windows",
    target_os = "android"
))]
pub(super) fn watch_remote(data_dir: PathBuf) {
    static STARTED: std::sync::Once = std::sync::Once::new();
    STARTED.call_once(|| {
        std::thread::spawn(move || {
            loop {
                if refresh_remote(&data_dir) {
                    let active = std::fs::read_to_string(data_dir.join("active-skin")).ok();
                    let _ = activate(&data_dir, active.as_deref().map(str::trim));
                }
                std::thread::sleep(std::time::Duration::from_secs(6 * 3600));
            }
        });
    });
}

#[cfg(not(any(
    target_os = "linux",
    target_os = "macos",
    target_os = "windows",
    target_os = "android"
)))]
pub(super) fn watch_remote(_data_dir: PathBuf) {}

#[cfg(any(
    target_os = "linux",
    target_os = "macos",
    target_os = "windows",
    target_os = "android"
))]
fn refresh_remote(data_dir: &Path) -> bool {
    let mut changed = false;
    let folders = std::fs::read_dir(data_dir.join("skins"))
        .into_iter()
        .flatten()
        .flatten();
    for folder in folders {
        let url = std::fs::read_to_string(folder.path().join("remote.json"))
            .ok()
            .and_then(|text| serde_json::from_str::<Value>(&text).ok())
            .and_then(|value| value["url"].as_str().map(str::to_owned));
        let Some(url) = url else { continue };
        let Ok(staged) = fetch_skin(data_dir, &url) else {
            continue;
        };
        let current = std::fs::read_to_string(folder.path().join("style.css")).ok();
        let fresh = std::fs::read_to_string(staged.join("style.css")).ok();
        let script = |dir: &Path| std::fs::read_to_string(dir.join("ext/script.js")).ok();
        if (current != fresh || script(&folder.path()) != script(&staged))
            && install(&staged, data_dir).is_ok()
        {
            changed = true;
        }
        let _ = std::fs::remove_dir_all(&staged);
    }
    changed
}

#[cfg(any(
    target_os = "linux",
    target_os = "macos",
    target_os = "windows",
    target_os = "android"
))]
fn get(url: &str) -> Result<Vec<u8>, String> {
    reqwest::blocking::Client::new()
        .get(url)
        .timeout(std::time::Duration::from_secs(60))
        .send()
        .and_then(|response| response.error_for_status())
        .and_then(|response| response.bytes())
        .map(|bytes| bytes.to_vec())
        .map_err(|error| error.to_string())
}

#[cfg(any(
    target_os = "linux",
    target_os = "macos",
    target_os = "windows",
    target_os = "android"
))]
fn fetch_skin(data_dir: &Path, url: &str) -> Result<PathBuf, String> {
    let bytes = get(url)?;
    if bytes.starts_with(b"PK") {
        let staging = data_dir.join("skins-download.zip");
        std::fs::write(&staging, bytes).map_err(|error| error.to_string())?;
        return Ok(staging);
    }
    let css = String::from_utf8(bytes).map_err(|error| error.to_string())?;
    let base = reqwest::Url::parse(url).map_err(|error| error.to_string())?;
    let fetch = |script: &str| {
        let target = base.join(script).map_err(|error| error.to_string())?;
        String::from_utf8(get(target.as_str())?).map_err(|error| error.to_string())
    };
    let (_, files) =
        fluxa_skin::files_from_css(&css, Some(url), &fetch).map_err(|errors| errors.join("; "))?;
    let staging = data_dir.join("skin-staging");
    let _ = std::fs::remove_dir_all(&staging);
    for (name, text) in files {
        let path = staging.join(name);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        }
        std::fs::write(path, text).map_err(|error| error.to_string())?;
    }
    Ok(staging)
}

#[cfg(not(any(
    target_os = "linux",
    target_os = "macos",
    target_os = "windows",
    target_os = "android"
)))]
pub(super) fn download_and_install(_data_dir: PathBuf, _source: String) {}

pub(super) fn finish_install(data_dir: &Path, source: &Path) {
    match install(source, data_dir) {
        Ok(id) => {
            let _ = activate(data_dir, Some(&id));
            set_message(None, false);
        }
        Err(problems) => set_message(Some(problems.join("; ")), false),
    }
}

pub(crate) fn use_skin(id: &str) {
    let Some(data_dir) = MANAGER
        .lock()
        .ok()
        .and_then(|manager| manager.data_dir.clone())
    else {
        return;
    };
    let id = Some(id).filter(|id| !id.is_empty());
    if let Err(error) = activate(&data_dir, id) {
        set_message(Some(error.to_string()), false);
    }
}

pub(super) fn manager_data(language: &str) -> Value {
    let active = ACTIVE
        .read()
        .ok()
        .and_then(|slot| slot.as_ref().map(|skin| skin.manifest.id.clone()))
        .unwrap_or_default();
    let bundled = DEFAULT
        .read()
        .ok()
        .and_then(|slot| slot.as_ref().map(|skin| skin.manifest.id.clone()))
        .unwrap_or_default();
    let active = if active == bundled {
        String::new()
    } else {
        active
    };
    let app = ext::app();
    let manager = MANAGER.lock();
    let Ok(manager) = manager else {
        return Value::Null;
    };
    let mut items = vec![json!({"id": "", "name": "Fluxa", "active": active.is_empty()})];
    items.extend(manager.installed.iter().map(|(id, name, hosts, _)| {
        let scripts = if hosts.is_empty() {
            String::new()
        } else {
            format!(
                "{} {hosts}",
                fluxa_ui::localized("skins.scripts_note", language)
            )
        };
        json!({
            "id": id,
            "name": name,
            "active": *id == active,
            "scripts": scripts,
            "allowed": manager.allowed.contains(id),
        })
    }));
    let trackers: Vec<Value> = app["trackers"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|tracker| {
            let auth = &tracker["results"]["auth"];
            let status = if auth["ok"] == true {
                fluxa_ui::localized("skins.tracker_connected", language).to_owned()
            } else if let Some(error) = auth["error"].as_str() {
                format!(
                    "{} {error}",
                    fluxa_ui::localized("skins.tracker_failed", language)
                )
            } else {
                String::new()
            };
            json!({
                "extension": tracker["extension"],
                "title": tracker["title"],
                "status": status,
                "watched": tracker["results"]["watched"]["value"].as_array().map_or(0, Vec::len),
                "library": tracker["results"]["library"]["value"].as_array().map_or(0, Vec::len),
            })
        })
        .collect();
    let extensions: Vec<Value> = app["extensions"]
        .as_object()
        .map(|map| {
            map.iter()
                .filter(|(_, extension)| {
                    extension["settings"].as_array().is_some_and(|list| !list.is_empty())
                })
                .map(|(id, extension)| {
                    let mut settings = extension["settings"].clone();
                    for setting in settings.as_array_mut().into_iter().flatten() {
                        if setting["kind"] == "toggle" {
                            setting["choices"] = json!([
                                {"value": "true", "label": fluxa_ui::localized("skins.on", language)},
                                {"value": "false", "label": fluxa_ui::localized("skins.off", language)},
                            ]);
                        }
                    }
                    json!({"id": id, "name": extension["name"], "settings": settings})
                })
                .collect()
        })
        .unwrap_or_default();
    json!({
        "extensions": extensions,
        "trackers": trackers,
        "items": items,
        "message": manager.message,
        "busy": manager.busy,
        "placeholder": fluxa_ui::localized("skins.source_placeholder", language),
        "cssPlaceholder": fluxa_ui::localized("skins.css_placeholder", language),
    })
}

pub(crate) fn set_scripts(id: &str, allow: bool) {
    let Some(data_dir) = MANAGER
        .lock()
        .ok()
        .and_then(|manager| manager.data_dir.clone())
    else {
        return;
    };
    let (skins, _) = fluxa_skin::load_skins(&data_dir.join("skins"));
    let Some(skin) = skins.iter().find(|skin| skin.manifest.id == id) else {
        return;
    };
    consent::record(&data_dir, skin, allow);
    if let Ok(mut manager) = MANAGER.lock() {
        manager.allowed = consent::granted(&data_dir, &skins);
        manager.generation += 1;
    }
    ext::stop();
}
