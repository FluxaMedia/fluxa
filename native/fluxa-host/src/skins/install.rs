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
    std::thread::spawn(move || {
        let staging = data_dir.join("skins-download.zip");
        let fetched = reqwest::blocking::Client::new()
            .get(&url)
            .timeout(std::time::Duration::from_secs(60))
            .send()
            .and_then(|response| response.error_for_status())
            .and_then(|response| response.bytes())
            .map_err(|error| error.to_string())
            .and_then(|bytes| std::fs::write(&staging, bytes).map_err(|error| error.to_string()));
        match fetched {
            Ok(()) => finish_install(&data_dir, &staging),
            Err(error) => set_message(Some(error), false),
        }
        let _ = std::fs::remove_file(&staging);
    });
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
    let manager = MANAGER.lock();
    let Ok(manager) = manager else {
        return Value::Null;
    };
    let mut items = vec![json!({"id": "", "name": "Fluxa", "active": active.is_empty()})];
    items.extend(
        manager
            .installed
            .iter()
            .map(|(id, name)| json!({"id": id, "name": name, "active": *id == active})),
    );
    json!({
        "items": items,
        "message": manager.message,
        "busy": manager.busy,
        "placeholder": fluxa_ui::localized("skins.source_placeholder", language),
        "cssPlaceholder": fluxa_ui::localized("skins.css_placeholder", language),
    })
}
