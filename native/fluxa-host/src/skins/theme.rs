use super::*;

pub(super) static CUSTOM_CSS: Mutex<String> = Mutex::new(String::new());
pub(super) static BUILTIN_CSS: Mutex<String> = Mutex::new(String::new());

pub(super) fn builtin_css() -> String {
    BUILTIN_CSS
        .lock()
        .map(|css| css.clone())
        .unwrap_or_default()
}

pub(super) fn custom_css_path() -> Option<PathBuf> {
    let manager = MANAGER.lock().ok()?;
    Some(manager.data_dir.as_ref()?.join("custom.css"))
}

pub(super) fn apply_custom_css(css: String) {
    if let Ok(mut slot) = CUSTOM_CSS.lock() {
        *slot = css;
    }
    let active = ACTIVE.read().ok().and_then(|slot| slot.clone());
    apply_theme(active.as_deref());
    if let Ok(mut cache) = CACHE.lock() {
        cache.clear();
    }
    touch();
}

pub(crate) fn clear_custom_css() {
    if let Some(path) = custom_css_path() {
        let _ = std::fs::remove_file(path);
    }
    apply_custom_css(String::new());
    set_field("customCss", builtin_css());
}

pub(crate) fn load(data_dir: &Path) {
    let custom = std::fs::read_to_string(data_dir.join("custom.css")).unwrap_or_default();
    if let Ok(mut slot) = CUSTOM_CSS.lock() {
        *slot = custom.clone();
    }
    load_default();
    let (skins, problems) = fluxa_skin::load_skins(&data_dir.join("skins"));
    if let Ok(mut manager) = MANAGER.lock() {
        manager.data_dir = Some(data_dir.to_owned());
        manager.installed = skins
            .iter()
            .map(|skin| {
                (
                    skin.manifest.id.clone(),
                    skin.manifest.name.clone(),
                    skin.script_hosts().join(", "),
                    skin.script_digest().unwrap_or_default(),
                )
            })
            .collect();
        manager.allowed = consent::granted(data_dir, &skins);
        manager.generation += 1;
    }
    for problem in problems {
        crate::host_log(format!("skins/{problem}"));
    }
    let wanted = dev_folder()
        .or_else(|| std::fs::read_to_string(data_dir.join("active-skin")).ok())
        .map(|id| id.trim().to_owned())
        .filter(|id| !id.is_empty());
    let active = wanted.and_then(|id| {
        let folder = Path::new(&id);
        if folder.join("manifest.json").is_file() {
            return match fluxa_skin::load_skin(folder) {
                Ok(skin) => Some(Arc::new(skin)),
                Err(problems) => {
                    problems
                        .iter()
                        .for_each(|problem| crate::host_log(format!("skin {id}: {problem}")));
                    None
                }
            };
        }
        skins
            .into_iter()
            .find(|skin| skin.manifest.id == id)
            .map(Arc::new)
    });
    apply_theme(active.as_deref());
    set_field(
        "customCss",
        if custom.trim().is_empty() {
            builtin_css()
        } else {
            custom
        },
    );
    let default = DEFAULT.read().ok().and_then(|slot| slot.clone());
    let (own, base) = match (active.as_deref(), default.as_deref()) {
        (Some(skin), Some(default)) if skin.has_elements() => (
            Some(Arc::new(skin.grafted(skin))),
            Some(Arc::new(skin.grafted(default))),
        ),
        _ => (active, None),
    };
    if let Ok(mut slot) = PATCHED.write() {
        *slot = base;
    }
    if let Ok(mut slot) = ACTIVE.write() {
        *slot = own;
    }
}

pub(super) fn apply_theme(active: Option<&Skin>) {
    fluxa_ui::set_skin_strings(
        active
            .into_iter()
            .flat_map(|skin| skin.strings().iter())
            .map(|(code, table)| {
                (
                    code.clone(),
                    table
                        .iter()
                        .map(|(key, text)| (key.clone(), text.clone()))
                        .collect(),
                )
            }),
    );
    if let Err(error) = fluxa_theme::theme::set_live_tokens(active.and_then(Skin::tokens)) {
        crate::host_log(format!("skin tokens: {error}"));
    }
    let default = DEFAULT.read().ok().and_then(|slot| slot.clone());
    let custom = CUSTOM_CSS.lock().map(|css| css.clone()).unwrap_or_default();
    let builtin = [default.as_deref(), active]
        .into_iter()
        .flatten()
        .filter_map(Skin::theme)
        .collect::<Vec<_>>()
        .join("\n");
    if let Ok(mut slot) = BUILTIN_CSS.lock() {
        slot.clone_from(&builtin);
    }
    let css = if custom.trim().is_empty() {
        builtin
    } else {
        custom
    };
    if css.is_empty() {
        return;
    }
    if let Err(error) = fluxa_ui::reload_styles_with(&css) {
        crate::host_log(format!("skin theme: {error}"));
    }
}
