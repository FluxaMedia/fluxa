use crate::route::{NativeAction, Route};
use fluxa_skin::{FormFactor, Resolved, ResolvedAction, Skin};
use fluxa_ui::{HomeLayout, HomeModel, SettingsModel, SkinFrame, SkinTarget, UiFormFactorJson};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::path::Path;
use std::sync::{Arc, Mutex, RwLock};

static DEFAULT: RwLock<Option<Arc<Skin>>> = RwLock::new(None);
static ACTIVE: RwLock<Option<Arc<Skin>>> = RwLock::new(None);
static TARGETS: Mutex<Vec<(u64, SkinTarget)>> = Mutex::new(Vec::new());
static RESOLVE: Mutex<(fluxa_skin::ResolveCache, u64)> =
    Mutex::new((fluxa_skin::ResolveCache::new(), 0));
static DATA: Mutex<Option<(u64, Arc<Value>)>> = Mutex::new(None);
static CACHE: Mutex<Option<(u64, Arc<Vec<Resolved>>)>> = Mutex::new(None);
static FIELDS: Mutex<Option<HashMap<String, String>>> = Mutex::new(None);
static FAILED: Mutex<Option<String>> = Mutex::new(None);
static MANAGER: Mutex<Manager> = Mutex::new(Manager {
    data_dir: None,
    installed: Vec::new(),
    message: None,
    busy: false,
    generation: 0,
});

struct Manager {
    data_dir: Option<std::path::PathBuf>,
    installed: Vec<(String, String)>,
    message: Option<String>,
    busy: bool,
    generation: u64,
}

const DEFAULT_FILES: &[(&str, &str)] = &[
    (
        "manifest.json",
        include_str!("../../../shared/skins/default/manifest.json"),
    ),
    (
        "languages/english_us.json",
        include_str!("../../../shared/skins/default/languages/english_us.json"),
    ),
    (
        "languages/tr_tr.json",
        include_str!("../../../shared/skins/default/languages/tr_tr.json"),
    ),
    (
        "languages/ar.json",
        include_str!("../../../shared/skins/default/languages/ar.json"),
    ),
    (
        "languages/he.json",
        include_str!("../../../shared/skins/default/languages/he.json"),
    ),
    (
        "constants.json",
        include_str!("../../../shared/skins/default/constants.json"),
    ),
    (
        "includes.json",
        include_str!("../../../shared/skins/default/includes.json"),
    ),
    (
        "screens/settings.json",
        include_str!("../../../shared/skins/default/screens/settings.json"),
    ),
    (
        "screens/detail.json",
        include_str!("../../../shared/skins/default/screens/detail.json"),
    ),
    (
        "screens/profiles.json",
        include_str!("../../../shared/skins/default/screens/profiles.json"),
    ),
    (
        "screens/player.json",
        include_str!("../../../shared/skins/default/screens/player.json"),
    ),
    (
        "screens/menu.json",
        include_str!("../../../shared/skins/default/screens/menu.json"),
    ),
    (
        "screens/toast.json",
        include_str!("../../../shared/skins/default/screens/toast.json"),
    ),
    (
        "screens/detail_modal.json",
        include_str!("../../../shared/skins/default/screens/detail_modal.json"),
    ),
    (
        "screens/mini.json",
        include_str!("../../../shared/skins/default/screens/mini.json"),
    ),
    (
        "screens/watch.json",
        include_str!("../../../shared/skins/default/screens/watch.json"),
    ),
    (
        "screens/update.json",
        include_str!("../../../shared/skins/default/screens/update.json"),
    ),
    (
        "screens/window.json",
        include_str!("../../../shared/skins/default/screens/window.json"),
    ),
    (
        "screens/tip.json",
        include_str!("../../../shared/skins/default/screens/tip.json"),
    ),
    (
        "screens/welcome.json",
        include_str!("../../../shared/skins/default/screens/welcome.json"),
    ),
    (
        "screens/live.json",
        include_str!("../../../shared/skins/default/screens/live.json"),
    ),
    (
        "screens/folder.json",
        include_str!("../../../shared/skins/default/screens/folder.json"),
    ),
    (
        "screens/home.json",
        include_str!("../../../shared/skins/default/screens/home.json"),
    ),
    (
        "screens/library.json",
        include_str!("../../../shared/skins/default/screens/library.json"),
    ),
    (
        "screens/discover.json",
        include_str!("../../../shared/skins/default/screens/discover.json"),
    ),
    (
        "screens/calendar.json",
        include_str!("../../../shared/skins/default/screens/calendar.json"),
    ),
    (
        "style.css",
        include_str!("../../../shared/skins/default/style.css"),
    ),
];

fn load_default() {
    match fluxa_skin::load_embedded(DEFAULT_FILES) {
        Ok(skin) => {
            if let Ok(mut slot) = DEFAULT.write() {
                *slot = Some(Arc::new(skin));
            }
        }
        Err(problems) => problems
            .iter()
            .for_each(|problem| crate::host_log(format!("default skin: {problem}"))),
    }
}

const DEFAULT_READY: &[&str] = &[];

fn skin_for(screen: &str) -> Option<Arc<Skin>> {
    let active = ACTIVE.read().ok()?.clone();
    let default = DEFAULT
        .read()
        .ok()?
        .clone()
        .filter(|_| DEFAULT_READY.contains(&screen));
    [active, default]
        .into_iter()
        .flatten()
        .find(|skin| skin.replaces(screen))
}

fn menu_data(view: &crate::card_menu::MenuView) -> Value {
    let items: Vec<_> = view
        .items
        .iter()
        .enumerate()
        .map(|(index, item)| {
            json!({
                "index": index,
                "label": item.label,
                "icon": item.icon,
                "appIcon": item.app_icon,
                "selected": view.selected == Some(index),
            })
        })
        .collect();
    json!({
        "title": view.title,
        "items": items,
        "x": view.anchor.x,
        "y": view.anchor.y,
    })
}

pub(crate) fn menu_skinned() -> bool {
    skin_for("menu").is_some()
}

pub(crate) fn canvas(form: UiFormFactorJson) -> Option<f32> {
    let skin = skin_for("home")?;
    skin.canvas(match form {
        UiFormFactorJson::Mobile => FormFactor::Mobile,
        UiFormFactorJson::Desktop => FormFactor::Desktop,
        UiFormFactorJson::Tv => FormFactor::Tv,
    })
}

fn localize(tree: &mut [Resolved], language: &str) {
    for node in tree {
        if let Some(key) = &node.i18n {
            node.text = Some(fluxa_ui::localized(key, language));
        }
        localize(&mut node.children, language);
    }
}

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
        *cache = None;
    }
    if let Ok(mut data) = DATA.lock() {
        *data = None;
    }
    Ok(())
}

pub fn install(source: &Path, data_dir: &Path) -> Result<String, Vec<String>> {
    fluxa_skin::install(source, &data_dir.join("skins"))
}

fn github_archive(url: &str) -> String {
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

fn set_message(message: Option<String>, busy: bool) {
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

fn finish_install(data_dir: &Path, source: &Path) {
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

fn manager_data(language: &str) -> Value {
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
    })
}

pub(crate) fn load(data_dir: &Path) {
    load_default();
    let (skins, problems) = fluxa_skin::load_skins(&data_dir.join("skins"));
    if let Ok(mut manager) = MANAGER.lock() {
        manager.data_dir = Some(data_dir.to_owned());
        manager.installed = skins
            .iter()
            .map(|skin| (skin.manifest.id.clone(), skin.manifest.name.clone()))
            .collect();
        manager.generation += 1;
    }
    for problem in problems {
        crate::host_log(format!("skins/{problem}"));
    }
    let wanted = crate::preview::skin_override()
        .or_else(|| std::fs::read_to_string(data_dir.join("active-skin")).ok())
        .map(|id| id.trim().to_owned())
        .filter(|id| !id.is_empty());
    let active = wanted.and_then(|id| {
        let folder = Path::new(&id);
        if cfg!(debug_assertions)
            && (folder.join("manifest.json").is_file() || folder.join("skin.json").is_file())
        {
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
    if let Ok(mut slot) = ACTIVE.write() {
        *slot = active;
    }
}

fn apply_theme(active: Option<&Skin>) {
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
    let css = [default.as_deref(), active]
        .into_iter()
        .flatten()
        .filter_map(Skin::theme)
        .collect::<Vec<_>>()
        .join("\n");
    if css.is_empty() {
        return;
    }
    if let Err(error) = fluxa_ui::reload_styles_with(&css) {
        crate::host_log(format!("skin theme: {error}"));
    }
}

const FINGERPRINT_CARDS: usize = 60;

fn fingerprint(
    screen: &str,
    skin: &Skin,
    home: &HomeModel,
    settings: &SettingsModel,
    fields: &HashMap<String, String>,
    nav: &Value,
    extra: &(String, String, String),
) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    (screen, &skin.manifest.id, &skin.manifest.version).hash(&mut hasher);
    (
        &home.title,
        &home.eyebrow,
        &home.description,
        &home.background_url,
        &home.logo_url,
        &home.item_id,
        &home.profile_name,
        &home.profile_avatar_url,
        home.is_loading,
        home.show_hero_section,
    )
        .hash(&mut hasher);
    std::mem::discriminant(&home.form_factor).hash(&mut hasher);
    extra.hash(&mut hasher);
    for row in &home.rows {
        (&row.id, &row.title, row.cards.len()).hash(&mut hasher);
        for card in row.cards.iter().take(FINGERPRINT_CARDS) {
            (
                &card.id,
                &card.title,
                &card.subtitle,
                &card.artwork_url,
                card.progress.to_bits(),
            )
                .hash(&mut hasher);
        }
    }
    settings.values.to_string().hash(&mut hasher);
    (settings.active_section, settings.page_open).hash(&mut hasher);
    nav.to_string().hash(&mut hasher);
    manager_data("").to_string().hash(&mut hasher);
    let mut names: Vec<_> = fields.iter().collect();
    names.sort();
    names.hash(&mut hasher);
    hasher.finish()
}

#[derive(Clone, Copy)]
pub(crate) enum Extra<'a> {
    None,
    Profiles(&'a fluxa_ui::ProfilesModel),
    Player(&'a fluxa_ui::PlayerModel),
    Folder(&'a fluxa_ui::FolderModel, usize),
    Guide(&'a fluxa_ui::GuideModel),
    Welcome(&'a fluxa_ui::WelcomeModel),
    Menu(&'a crate::card_menu::MenuView),
    Toast(&'a str),
    Tip(&'a fluxa_ui::HoverTip),
    Update(&'a str),
    Window(bool),
    Mini(&'a fluxa_ui::PlayerModel),
    Watch(&'a fluxa_ui::WatchTogetherModel),
    Modal,
}

impl<'a> Extra<'a> {
    pub(crate) fn for_route(
        route: Route,
        folder: &'a fluxa_ui::FolderModel,
        tab: usize,
        guide: &'a fluxa_ui::GuideModel,
    ) -> Self {
        if route == Route::Folder {
            Self::Folder(folder, tab)
        } else if route == Route::Live {
            Self::Guide(guide)
        } else {
            Self::None
        }
    }
}

pub(crate) fn tree(
    route: Route,
    home: &HomeModel,
    settings: &SettingsModel,
    library: &fluxa_ui::LibraryModel,
    library_tab: fluxa_ui::LibraryTab,
    discover: &fluxa_ui::DiscoverModel,
    calendar: &fluxa_ui::CalendarModel,
    detail: &fluxa_ui::DetailModel,
    extra: Extra<'_>,
) -> Option<Arc<Vec<Resolved>>> {
    let screen = match extra {
        Extra::Profiles(_) => "profiles",
        Extra::Player(_) => "player",
        Extra::Folder(..) => "folder",
        Extra::Guide(_) => "live",
        Extra::Welcome(_) => "welcome",
        Extra::Menu(_) => "menu",
        Extra::Toast(_) => "toast",
        Extra::Tip(_) => "tip",
        Extra::Update(_) => "update",
        Extra::Window(_) => "window",
        Extra::Mini(_) => "mini",
        Extra::Watch(_) => "watch",
        Extra::Modal => "detail_modal",
        Extra::None => route.as_str(),
    };
    let skin = skin_for(screen)?;
    let fields = FIELDS
        .lock()
        .ok()
        .and_then(|fields| fields.clone())
        .unwrap_or_default();
    let mut nav = fluxa_ui::nav_data(if route == Route::Calendar {
        "library"
    } else {
        screen
    });

    let behavior = settings
        .values
        .get("navSidebarMode")
        .and_then(Value::as_str)
        .unwrap_or("hover");
    nav["expanded"] = json!(behavior == "expanded");
    nav["hover"] = json!(behavior != "expanded" && behavior != "collapsed");
    let slides = fluxa_ui::hero_slides_data(home);
    let windows = fluxa_ui::grid_windows();
    let screen_data = || match route {
        Route::Library => fluxa_ui::library_data(library, library_tab),
        Route::Discover => fluxa_ui::discover_data(discover),
        Route::Calendar => fluxa_ui::calendar_data(calendar),
        Route::Detail => fluxa_ui::detail_data(detail),
        _ => Value::Null,
    };
    let screen_key = if route == Route::Discover {
        format!(
            "{}|{:?}|{:?}|{}|{}|{}|{}|{}|{}|{}",
            discover.results.len(),
            discover.results.last().and_then(|card| card.id.as_deref()),
            discover.next_page,
            discover.generation,
            discover.query,
            discover.content_type,
            discover.selected_catalog_key,
            discover.selected_extra_value,
            discover.is_loading || discover.catalogs_loading,
            discover.error.is_some(),
        )
    } else {
        screen_data().to_string()
    };
    let extra_data = match extra {
        Extra::Profiles(model) => fluxa_ui::profiles_data(model, home.form_factor.into()),
        Extra::Player(model) => fluxa_ui::player_data(model),
        Extra::Folder(model, tab) => fluxa_ui::folder_data(model, tab),
        Extra::Guide(model) => fluxa_ui::guide_data(model),
        Extra::Welcome(model) => fluxa_ui::welcome_data(model),
        Extra::Menu(view) => menu_data(view),
        Extra::Toast(text) => json!({"text": text}),
        Extra::Modal => fluxa_ui::detail_data(detail),
        Extra::Mini(player) => json!({
            "playIcon": if player.paused { "PlayFilled" } else { "PauseFilled" },
            "ratio": if player.duration > 0.0 { (player.position / player.duration).clamp(0.0, 1.0) } else { 0.0 },
        }),
        Extra::Watch(model) => fluxa_ui::watch_data(model),
        Extra::Update(version) => json!({"version": version}),
        Extra::Window(full) => json!({
            "icon": if full { "CornersIn" } else { "CornersOut" },
        }),
        Extra::Tip(tip) => json!({
            "text": tip.text,
            "x": tip.rect.center().x,
            "y": tip.rect.bottom() + 8.0,
        }),
        Extra::None => Value::Null,
    };
    let base = fingerprint(
        screen,
        &skin,
        home,
        settings,
        &fields,
        &nav,
        &(slides.to_string(), screen_key, extra_data.to_string()),
    );
    let key = {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        (base, format!("{windows:?}")).hash(&mut hasher);
        hasher.finish()
    };
    if let Ok(cache) = CACHE.lock()
        && let Some((cached, tree)) = cache.as_ref()
        && *cached == key
    {
        return Some(tree.clone());
    }
    let language = settings
        .values
        .get("language")
        .and_then(Value::as_str)
        .unwrap_or("en")
        .to_owned();
    let cached = DATA.lock().ok().and_then(|data| {
        data.as_ref()
            .filter(|(cached, _)| *cached == base)
            .map(|(_, data)| data.clone())
    });
    let data = match cached {
        Some(data) => data,
        None => {
            let active = match route {
                Route::Calendar => 1,
                Route::Library if library.downloads_open => 2,
                _ => 0,
            };
            let sections: Vec<_> = [
                ("library", "nav.library"),
                ("calendar", "nav.calendar"),
                ("downloads", "library.downloads"),
            ]
            .into_iter()
            .enumerate()
            .map(|(index, (id, label))| {
                json!({
                    "id": id,
                    "key": "section",
                    "label": fluxa_ui::localized(label, &language),
                    "selected": index == active,
                })
            })
            .collect();
            let mut home_data = serde_json::to_value(home).ok()?;
            home_data["heroSlides"] = slides;
            home_data["addonsPage"] = json!(fluxa_ui::addons_page());
            let screen_data = screen_data();
            let data = Arc::new(json!({
                "home": home_data,
                "library": if route == Route::Library { screen_data.clone() } else { Value::Null },
                "discover": if route == Route::Discover { screen_data.clone() } else { Value::Null },
                "calendar": if route == Route::Calendar { screen_data.clone() } else { Value::Null },
                "detail": if matches!(extra, Extra::Modal) { extra_data.clone() } else if route == Route::Detail { screen_data } else { Value::Null },
                "profiles": if matches!(extra, Extra::Profiles(_)) { extra_data.clone() } else { Value::Null },
                "player": if matches!(extra, Extra::Player(_)) { extra_data.clone() } else { Value::Null },
                "folder": if matches!(extra, Extra::Folder(..)) { extra_data.clone() } else { Value::Null },
                "guide": if matches!(extra, Extra::Guide(_)) { extra_data.clone() } else { Value::Null },
                "welcome": if matches!(extra, Extra::Welcome(_)) { extra_data.clone() } else { Value::Null },
                "menu": if matches!(extra, Extra::Menu(_)) { extra_data.clone() } else { Value::Null },
                "toast": if matches!(extra, Extra::Toast(_)) { extra_data.clone() } else { Value::Null },
                "tip": if matches!(extra, Extra::Tip(_)) { extra_data.clone() } else { Value::Null },
                "update": if matches!(extra, Extra::Update(_)) { extra_data.clone() } else { Value::Null },
                "window": if matches!(extra, Extra::Window(_)) { extra_data.clone() } else { Value::Null },
                "mini": if matches!(extra, Extra::Mini(_)) { extra_data.clone() } else { Value::Null },
                "watch": if matches!(extra, Extra::Watch(_)) { extra_data } else { Value::Null },
                "const": skin.constants(),
                "sections": sections,
                "settings": settings.values,
                "prefs": if route == Route::Settings { prefs_data(settings, home.form_factor.into(), &fields) } else { Value::Null },
                "fields": fields,
                "nav": nav,
            }));
            if let Ok(mut cache) = DATA.lock() {
                *cache = Some((base, data.clone()));
            }
            data
        }
    };
    let form = match home.form_factor {
        UiFormFactorJson::Mobile => FormFactor::Mobile,
        UiFormFactorJson::Desktop => FormFactor::Desktop,
        UiFormFactorJson::Tv => FormFactor::Tv,
    };
    let resolved = {
        let mut cache = RESOLVE.lock().ok()?;
        if cache.1 != base {
            cache.0.clear_items();
            cache.1 = base;
        }
        cache.0.windows = windows;
        skin.resolve(screen, &data, form, &mut cache.0)?
    };
    match resolved {
        Ok(mut tree) => {
            localize(&mut tree, &language);
            let tree = Arc::new(tree);
            if let Ok(mut cache) = CACHE.lock() {
                *cache = Some((key, tree.clone()));
            }
            Some(tree)
        }
        Err(error) => {
            let message = format!("skin {}: {screen}: {error}", skin.manifest.id);
            if let Ok(mut last) = FAILED.lock()
                && last.as_deref() != Some(&message)
            {
                crate::host_log(&message);
                *last = Some(message);
            }
            None
        }
    }
}

fn prefs_data(
    settings: &SettingsModel,
    form: fluxa_ui::UiFormFactor,
    fields: &HashMap<String, String>,
) -> Value {
    let mut data = fluxa_ui::settings_data(settings, form);
    if fluxa_ui::SETTINGS_SECTIONS
        .get(settings.active_section)
        .is_some_and(|section| section.title == "Theme")
    {
        data["skins"] = manager_data(settings.language());
    }
    if let Some(inputs) = data["inputs"].as_array_mut() {
        for input in inputs {
            let typed = input["key"].as_str().and_then(|key| fields.get(key));
            if let Some(typed) = typed {
                input["value"] = json!(typed);
            }
        }
    }
    if let Some(blocks) = data["blocks"].as_array_mut() {
        for row in blocks
            .iter_mut()
            .filter_map(|group| group["rows"].as_array_mut())
            .flatten()
        {
            let typed = row["key"]
                .as_str()
                .filter(|_| row["kind"] == "form")
                .and_then(|key| fields.get(key));
            if let Some(typed) = typed {
                row["value"] = json!(typed);
            }
        }
    }
    data
}

pub(crate) fn remember(frame: SkinFrame) -> HomeLayout {
    if let Ok(mut slot) = TARGETS.lock() {
        *slot = frame.targets;
    }
    frame.layout
}

pub(crate) fn remember_more(frame: SkinFrame) -> HomeLayout {
    if let Ok(mut slot) = TARGETS.lock() {
        slot.extend(frame.targets);
    }
    frame.layout
}

pub(crate) fn action_for_node(node: u64) -> Option<NativeAction> {
    let targets = TARGETS.lock().ok()?;
    let (_, target) = targets.iter().find(|(id, _)| *id == node)?;
    match target {
        SkinTarget::Action(action) => native_action(action),
        SkinTarget::Field(_) | SkinTarget::Control => None,
    }
}

pub(crate) enum Local {
    Choose(String, String),
    CalendarDay(u32),
    CalendarMonth(i32),
    Downloads(String, String),
    Detail(String, usize),
    Profiles(String, usize),
    Player(String),
    PlayerRow(usize),
    Update(String),
    Window(String),
    Modal(String, usize),
    Folder(String, usize),
    Guide(u64),
    Welcome(String),
    Watch(String, usize),
    Skins(String, String),
    Node(u64, String),
    MenuPick(usize),
    MenuDismiss,
}

pub(crate) fn local_for_node(node: u64) -> Option<Local> {
    let targets = TARGETS.lock().ok()?;
    let (_, target) = targets.iter().find(|(id, _)| *id == node)?;
    let SkinTarget::Action(action) = target else {
        return None;
    };
    match action.name.as_str() {
        "node" => Some(Local::Node(
            text(action, "id")?.parse().ok()?,
            text(action, "fill").unwrap_or_default(),
        )),
        "menu_pick" => Some(Local::MenuPick(text(action, "index")?.parse().ok()?)),
        "menu_dismiss" => Some(Local::MenuDismiss),
        "choose" => Some(Local::Choose(text(action, "key")?, text(action, "value")?)),
        "calendar_day" => Some(Local::CalendarDay(text(action, "day")?.parse().ok()?)),
        "calendar_month" => Some(Local::CalendarMonth(text(action, "delta")?.parse().ok()?)),
        "downloads" => Some(Local::Downloads(text(action, "op")?, text(action, "id")?)),
        "folder" => Some(Local::Folder(
            text(action, "op")?,
            text(action, "index")
                .and_then(|index| index.parse().ok())
                .unwrap_or(0),
        )),
        "welcome" => Some(Local::Welcome(text(action, "op")?)),
        "watch" => Some(Local::Watch(
            text(action, "op")?,
            text(action, "index")
                .and_then(|index| index.parse().ok())
                .unwrap_or(0),
        )),
        "skins" => Some(Local::Skins(
            text(action, "op")?,
            text(action, "id").unwrap_or_default(),
        )),
        "guide" => Some(Local::Guide(text(action, "node")?.parse().ok()?)),
        "player" => Some(Local::Player(text(action, "op")?)),
        "update" => Some(Local::Update(text(action, "op")?)),
        "window" => Some(Local::Window(text(action, "op")?)),
        "modal" => Some(Local::Modal(
            text(action, "op")?,
            text(action, "index")
                .and_then(|index| index.parse().ok())
                .unwrap_or(0),
        )),
        "player_row" => Some(Local::PlayerRow(text(action, "index")?.parse().ok()?)),
        "profiles" => Some(Local::Profiles(
            text(action, "op")?,
            text(action, "index")
                .and_then(|index| index.parse().ok())
                .unwrap_or(0),
        )),
        "detail" => Some(Local::Detail(
            text(action, "op")?,
            text(action, "index")
                .and_then(|index| index.parse().ok())
                .unwrap_or(0),
        )),
        _ => None,
    }
}

pub(crate) fn fill_for_node(node: u64) -> Vec<(u64, String)> {
    let Some(Local::Node(_, fill)) = local_for_node(node) else {
        return Vec::new();
    };
    fill.split(';')
        .filter_map(|pair| {
            let (target, field) = pair.split_once('=')?;
            Some((target.parse().ok()?, field_value(field).unwrap_or_default()))
        })
        .collect()
}

pub(crate) fn detail_node(node: u64) -> Option<u64> {
    use fluxa_ui::*;
    let local = local_for_node(node)?;
    if let Local::Profiles(op, index) = &local {
        let index = *index as u64;
        return match op.as_str() {
            "select" | "edit" | "add" => Some(NODE_PROFILES_CELL_BASE + index),
            "manage" => Some(NODE_PROFILES_MANAGE),
            "gear" => Some(NODE_PROFILES_GEAR),
            _ => None,
        };
    }
    if let Local::Guide(node) | Local::Node(node, _) = &local {
        return Some(*node);
    }
    if let Local::Folder(op, index) = &local {
        return match op.as_str() {
            "back" => Some(NODE_FOLDER_BACK),
            "tab" => Some(NODE_FOLDER_TAB_BASE + *index as u64),
            _ => None,
        };
    }
    if let Local::Update(op) = &local {
        return match op.as_str() {
            "later" => Some(NODE_UPDATE_LATER),
            "download" => Some(NODE_UPDATE_DOWNLOAD),
            "install" => Some(NODE_UPDATE_INSTALL),
            _ => None,
        };
    }
    if matches!(local, Local::Window(_)) {
        return Some(NODE_WINDOW_FULLSCREEN);
    }
    if let Local::PlayerRow(index) = &local {
        return Some(NODE_PLAYER_PANEL_ROW_BASE + *index as u64);
    }
    if let Local::Player(op) = &local {
        return Some(match op.as_str() {
            "close" => NODE_PLAYER_CLOSE,
            "toggle" => NODE_PLAYER_TOGGLE,
            "mute" => NODE_PLAYER_MUTE,
            "fullscreen" => NODE_PLAYER_FULLSCREEN,
            "previous" => NODE_PLAYER_PREVIOUS,
            "next" => NODE_PLAYER_NEXT,
            "tracks" => NODE_PLAYER_TRACKS,
            "speed" => NODE_PLAYER_SPEED,
            "episodes" => NODE_PLAYER_EPISODES,
            "settings" => NODE_PLAYER_SETTINGS,
            "info" => NODE_PLAYER_INFO,
            "cast" => NODE_PLAYER_CAST,
            "show" => NODE_PLAYER_CONTROLS,
            "hide" => NODE_PLAYER_HIDE_CONTROLS,
            "panel_close" => NODE_PLAYER_PANEL_CLOSE,
            "expand" => NODE_PLAYER_EXPAND,
            "skip" => NODE_PLAYER_SKIP,
            "next_play" => NODE_PLAYER_NEXT_PLAY,
            "next_dismiss" => NODE_PLAYER_NEXT_DISMISS,
            _ => return None,
        });
    }
    let Local::Detail(op, index) = local else {
        return None;
    };
    let index = index as u64;
    Some(match op.as_str() {
        "back" => NODE_DETAIL_BACK,
        "play" => NODE_DETAIL_PLAY,
        "watchlist" => NODE_DETAIL_WATCHLIST,
        "completed" => NODE_DETAIL_COMPLETED,
        "dropped" => NODE_DETAIL_DROPPED,
        "favorite" => NODE_DETAIL_FAVORITE,
        "shuffle" => NODE_DETAIL_SHUFFLE,
        "episode" => NODE_DETAIL_EPISODE_BASE + index,
        "cast" => NODE_DETAIL_CAST_BASE + index,
        "trailer" => NODE_DETAIL_TRAILER_BASE + index,
        "rating" => NODE_DETAIL_RATING_BASE + index,
        _ => return None,
    })
}

pub(crate) fn item_for_node(node: u64) -> Option<String> {
    let targets = TARGETS.lock().ok()?;
    let (_, target) = targets.iter().find(|(id, _)| *id == node)?;
    match target {
        SkinTarget::Action(action) if matches!(action.name.as_str(), "open_detail" | "play") => {
            text(action, "id")
        }
        _ => None,
    }
}

fn field_name(node: u64) -> Option<String> {
    let targets = TARGETS.lock().ok()?;
    targets.iter().find_map(|(id, target)| match target {
        SkinTarget::Field(name) if *id == node => Some(name.clone()),
        _ => None,
    })
}

pub(crate) fn field_value(name: &str) -> Option<String> {
    let fields = FIELDS.lock().ok()?;
    fields.as_ref()?.get(name).cloned()
}

pub(crate) fn is_field(node: u64) -> bool {
    field_name(node).is_some()
}

pub(crate) fn field_text(node: u64) -> Option<String> {
    let name = field_name(node)?;
    let fields = FIELDS.lock().ok()?;
    Some(
        fields
            .as_ref()
            .and_then(|fields| fields.get(&name).cloned())
            .unwrap_or_default(),
    )
}

pub(crate) fn set_field(name: &str, value: String) {
    if let Ok(mut fields) = FIELDS.lock() {
        fields
            .get_or_insert_default()
            .insert(name.to_owned(), value);
    }
}

pub(crate) fn take_field(name: &str) -> String {
    FIELDS
        .lock()
        .ok()
        .and_then(|mut fields| fields.as_mut()?.remove(name))
        .unwrap_or_default()
}

pub(crate) fn edit_field(node: u64, edit: impl FnOnce(&mut String)) -> bool {
    let Some(name) = field_name(node) else {
        return false;
    };
    let Ok(mut fields) = FIELDS.lock() else {
        return false;
    };
    edit(fields.get_or_insert_default().entry(name).or_default());
    true
}

fn text(action: &ResolvedAction, key: &str) -> Option<String> {
    match action.args.get(key)? {
        Value::String(text) => Some(text.clone()),
        Value::Number(number) => Some(number.to_string()),
        _ => None,
    }
}

pub(crate) fn native_action(action: &ResolvedAction) -> Option<NativeAction> {
    let value = match action.name.as_str() {
        "play" => {
            json!({"type": "play", "id": text(action, "id")?, "itemType": text(action, "type")?})
        }
        "open_detail" => {
            json!({"type": "detail", "id": text(action, "id")?, "itemType": text(action, "type")?})
        }
        "navigate" => json!({"type": "navigate", "destination": text(action, "screen")?}),
        "back" => json!({"type": "back"}),
        "search" => json!({"type": "search", "query": text(action, "query")?}),
        "load_more" => json!({"type": "loadMore", "rowId": text(action, "row")?}),
        "set_setting" => {
            json!({"type": "settingsChange", "key": text(action, "key")?, "value": action.args.get("value")?})
        }
        "save_field" => {
            json!({"type": "settingsChange", "key": text(action, "key")?, "value": field_value(&text(action, "key")?).unwrap_or_default()})
        }
        "open_settings" => match text(action, "page").and_then(|page| page.parse::<usize>().ok()) {
            Some(index) => json!({"type": "settingsSection", "index": index}),
            None => json!({"type": "navigate", "destination": "settings"}),
        },
        "watchlist_toggle" => {
            json!({"type": "toggleWatchlist", "item": {"id": text(action, "id")?, "type": text(action, "type")?}})
        }
        _ => return None,
    };
    serde_json::from_value(value).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    fn action(name: &str, args: &[(&str, Value)]) -> ResolvedAction {
        ResolvedAction {
            name: name.to_owned(),
            args: args
                .iter()
                .map(|(key, value)| ((*key).to_owned(), value.clone()))
                .collect::<BTreeMap<_, _>>(),
        }
    }

    #[test]
    fn open_detail_carries_id_and_type() {
        let native = native_action(&action(
            "open_detail",
            &[("id", json!("tt1")), ("type", json!("movie"))],
        ));
        assert!(
            matches!(native, Some(NativeAction::Detail { id, item_type, .. }) if id == "tt1" && item_type == "movie")
        );
    }

    #[test]
    fn navigate_rejects_unknown_screens() {
        assert!(native_action(&action("navigate", &[("screen", json!("library"))])).is_some());
        assert!(native_action(&action("navigate", &[("screen", json!("nowhere"))])).is_none());
    }

    #[test]
    fn missing_arguments_produce_no_action() {
        assert!(native_action(&action("play", &[("id", json!("tt1"))])).is_none());
    }

    #[test]
    fn setting_values_keep_their_type() {
        let native = native_action(&action(
            "set_setting",
            &[("key", json!("autoplay")), ("value", json!(true))],
        ));
        assert!(
            matches!(native, Some(NativeAction::SettingsChange { value, .. }) if value == json!(true))
        );
    }

    #[test]
    fn extension_actions_are_not_native() {
        assert!(native_action(&action("ext.example.sync", &[])).is_none());
    }

    #[test]
    fn field_edits_land_in_the_named_field() {
        remember(SkinFrame {
            targets: vec![(40_000_001, SkinTarget::Field("query".to_owned()))],
            layout: HomeLayout::default(),
        });
        assert!(is_field(40_000_001));
        assert!(edit_field(40_000_001, |text| text.push_str("dune")));
        assert_eq!(field_text(40_000_001).as_deref(), Some("dune"));
        assert!(!edit_field(40_000_002, |_| {}));
    }

    fn count_actions(nodes: &[fluxa_skin::Resolved], name: &str, found: &mut Vec<Value>) {
        for node in nodes {
            if let Some(action) = node.action.as_ref().filter(|action| action.name == name) {
                found.push(json!(action.args));
            }
            count_actions(&node.children, name, found);
        }
    }

    #[test]
    fn default_menu_lists_every_item_on_each_form() {
        let skin = fluxa_skin::load_embedded(DEFAULT_FILES).unwrap();
        let data = json!({
            "const": skin.constants(),
            "menu": {
                "title": "Language",
                "x": 300.0,
                "y": 200.0,
                "items": [
                    {"index": 0, "label": "English", "icon": null, "appIcon": null, "selected": true},
                    {"index": 1, "label": "Turkish", "icon": null, "appIcon": null, "selected": false},
                    {"index": 2, "label": "Arabic", "icon": null, "appIcon": null, "selected": false},
                ],
            },
        });
        for form in [FormFactor::Desktop, FormFactor::Mobile, FormFactor::Tv] {
            let tree = skin
                .resolve("menu", &data, form, &mut Default::default())
                .unwrap()
                .unwrap();
            let mut picks = Vec::new();
            count_actions(&tree, "menu_pick", &mut picks);
            assert_eq!(picks.len(), 3, "{form:?}");
            let mut dismiss = Vec::new();
            count_actions(&tree, "menu_dismiss", &mut dismiss);
            assert!(!dismiss.is_empty(), "{form:?}");
        }
    }

    #[test]
    fn player_panel_rows_activate_by_index() {
        use fluxa_ui::{PanelRow, PanelStep, PlayerModel, PlayerPanel};
        let skin = fluxa_skin::load_embedded(DEFAULT_FILES).unwrap();
        let row = |label: &str| PanelRow {
            label: label.to_owned(),
            ..Default::default()
        };
        let step = |label: &str, step: PanelStep| PanelRow {
            step: Some(step),
            ..row(label)
        };
        let player = PlayerModel {
            panel: Some(PlayerPanel {
                title: "Audio".to_owned(),
                rows: vec![
                    PanelRow {
                        heading: true,
                        ..row("Group")
                    },
                    row("Plain"),
                    step("Size", PanelStep::Value),
                    step("-", PanelStep::Minus),
                    step("+", PanelStep::Plus),
                    step("Color", PanelStep::Swatches(2)),
                    PanelRow {
                        value: Some("#ff0000".to_owned()),
                        ..step("", PanelStep::Swatch)
                    },
                    PanelRow {
                        value: Some("#00ff00".to_owned()),
                        ..step("", PanelStep::Swatch)
                    },
                ],
                list: true,
                ..Default::default()
            }),
            ..Default::default()
        };
        let data = json!({
            "const": skin.constants(),
            "player": fluxa_ui::player_data(&player),
        });
        for form in [FormFactor::Desktop, FormFactor::Mobile, FormFactor::Tv] {
            let tree = skin
                .resolve("player", &data, form, &mut Default::default())
                .unwrap()
                .unwrap();
            let mut rows = Vec::new();
            count_actions(&tree, "player_row", &mut rows);
            assert_eq!(rows.len(), 5, "{form:?}");
        }
    }

    #[test]
    fn account_page_routes_service_buttons_to_legacy_nodes() {
        let skin = fluxa_skin::load_embedded(DEFAULT_FILES).unwrap();
        let settings = SettingsModel::default();
        let data = json!({
            "const": skin.constants(),
            "nav": fluxa_ui::nav_data("settings"),
            "settings": settings.values,
            "fields": {},
            "prefs": fluxa_ui::settings_data(&settings, fluxa_ui::UiFormFactor::Desktop),
        });
        let tree = skin
            .resolve(
                "settings",
                &data,
                FormFactor::Desktop,
                &mut Default::default(),
            )
            .unwrap()
            .unwrap();
        let mut nodes = Vec::new();
        count_actions(&tree, "node", &mut nodes);
        let dump = format!("{nodes:?}");
        for id in [
            fluxa_ui::NODE_SETTINGS_SWITCH_PROFILE,
            fluxa_ui::NODE_SETTINGS_LOG_OUT,
            fluxa_ui::NODE_SETTINGS_ACCOUNT_BASE,
        ] {
            assert!(
                dump.contains(&format!("\"{id}\"")),
                "{id} missing in {dump}"
            );
        }
    }

    #[test]
    fn addon_cards_expose_row_actions_and_install_fill() {
        let skin = fluxa_skin::load_embedded(DEFAULT_FILES).unwrap();
        let mut settings = SettingsModel::default();
        settings.active_section = fluxa_ui::addons_page();
        settings.addons = vec![json!({
            "transportUrl": "https://example.com/manifest.json",
            "manifest": {"name": "ExampleAddon", "version": "1.0.0", "resources": ["catalog"], "types": ["movie"]},
        })];
        let data = json!({
            "const": skin.constants(),
            "nav": fluxa_ui::nav_data("settings"),
            "settings": settings.values,
            "fields": {},
            "prefs": fluxa_ui::settings_data(&settings, fluxa_ui::UiFormFactor::Desktop),
        });
        let tree = skin
            .resolve(
                "settings",
                &data,
                FormFactor::Desktop,
                &mut Default::default(),
            )
            .unwrap()
            .unwrap();
        let mut nodes = Vec::new();
        count_actions(&tree, "node", &mut nodes);
        let dump = format!("{nodes:?}");
        let remove = fluxa_ui::addon_action_node(0, 3);
        assert!(dump.contains(&format!("\"{remove}\"")), "{dump}");
        assert!(
            dump.contains(&format!("{}=addonUrl", fluxa_ui::NODE_SETTINGS_ADDON_URL)),
            "{dump}"
        );
    }

    #[test]
    fn empty_home_offers_addons() {
        let skin = fluxa_skin::load_embedded(DEFAULT_FILES).unwrap();
        let data = json!({
            "const": skin.constants(),
            "nav": fluxa_ui::nav_data("home"),
            "home": {"heroSlides": [], "rows": [], "cards": [], "isLoading": false, "addonsPage": 5},
        });
        let tree = skin
            .resolve("home", &data, FormFactor::Desktop, &mut Default::default())
            .unwrap()
            .unwrap();
        let mut pages = Vec::new();
        count_actions(&tree, "open_settings", &mut pages);
        assert!(pages.iter().any(|args| args["page"] == 5), "{pages:?}");
    }

    #[test]
    fn skip_next_and_warning_layers_resolve() {
        use fluxa_ui::{NextEpisodeCard, PlayerModel, SkipCard, SkipKind};
        let skin = fluxa_skin::load_embedded(DEFAULT_FILES).unwrap();
        let player = PlayerModel {
            skip: Some(SkipCard {
                kind: SkipKind::Intro,
                seek_to: 90.0,
                progress: 0.4,
            }),
            next_episode: Some(NextEpisodeCard {
                label: "S1E2".to_owned(),
                title: "Second".to_owned(),
                thumbnail: Some("https://example.com/t.jpg".to_owned()),
                countdown: Some(5),
            }),
            warnings: vec![("Violence".to_owned(), "Moderate".to_owned())],
            warnings_elapsed: Some(1.0),
            ..Default::default()
        };
        let data = json!({
            "const": skin.constants(),
            "player": fluxa_ui::player_data(&player),
        });
        let tree = skin
            .resolve(
                "player",
                &data,
                FormFactor::Desktop,
                &mut Default::default(),
            )
            .unwrap()
            .unwrap();
        let mut ops = Vec::new();
        count_actions(&tree, "player", &mut ops);
        let dump = format!("{ops:?}");
        for op in ["skip", "next_play", "next_dismiss"] {
            assert!(dump.contains(op), "{op} missing in {dump}");
        }
    }

    #[test]
    fn github_links_become_archive_urls() {
        assert_eq!(
            github_archive("https://github.com/example/skin"),
            "https://github.com/example/skin/archive/HEAD.zip"
        );
        assert_eq!(
            github_archive("https://github.com/example/skin/tree/dev/"),
            "https://github.com/example/skin/archive/dev.zip"
        );
        assert_eq!(
            github_archive("https://example.com/skin.zip"),
            "https://example.com/skin.zip"
        );
    }

    #[test]
    fn toast_and_tip_screens_resolve() {
        let skin = fluxa_skin::load_embedded(DEFAULT_FILES).unwrap();
        let data = json!({
            "toast": {"text": "Saved"},
            "tip": {"text": "Hint", "x": 100.0, "y": 80.0},
        });
        let data = json!({
            "toast": {"text": "Saved"},
            "tip": {"text": "Hint", "x": 100.0, "y": 80.0},
            "update": {"version": "1.2.3"},
            "window": {"icon": "CornersOut"},
            "mini": {"playIcon": "PauseFilled", "ratio": 0.4},
            "const": skin.constants(),
        });
        for screen in ["toast", "tip", "update", "window", "mini"] {
            let tree = skin
                .resolve(screen, &data, FormFactor::Desktop, &mut Default::default())
                .unwrap()
                .unwrap();
            assert!(!tree.is_empty(), "{screen}");
        }
    }

    #[test]
    fn bundled_theme_parses() {
        let skin = fluxa_skin::load_embedded(DEFAULT_FILES).unwrap();
        let css = skin.theme().unwrap();
        fluxa_ui::reload_styles_with(css).unwrap();
    }

    #[test]
    fn every_profile_mode_resolves() {
        use fluxa_ui::{ProfileEntry, ProfilesMode, ProfilesModel};
        let skin = fluxa_skin::load_embedded(DEFAULT_FILES).unwrap();
        let entry = |id: &str| ProfileEntry {
            id: id.to_owned(),
            name: id.to_owned(),
            locked: id == "b",
            ..Default::default()
        };
        let mut model = ProfilesModel {
            profiles: vec![entry("a"), entry("b")],
            ..Default::default()
        };
        let mut seen = Vec::new();
        for step in 0..5 {
            match step {
                1 => model.open_form(None),
                2 => model.mode = ProfilesMode::Avatars,
                3 => {
                    model.mode = ProfilesMode::Select;
                    model.confirm_delete = Some("a".to_owned());
                }
                4 => {
                    model.confirm_delete = None;
                    model.request_delete(1);
                }
                _ => {}
            }
            let data = json!({
                "const": skin.constants(),
                "settings": {},
                "fields": {},
                "profiles": fluxa_ui::profiles_data(&model, fluxa_ui::UiFormFactor::Desktop),
            });
            let tree = skin
                .resolve(
                    "profiles",
                    &data,
                    FormFactor::Desktop,
                    &mut Default::default(),
                )
                .unwrap()
                .unwrap();
            assert!(!tree.is_empty(), "step {step}");
            seen.push(format!("{tree:?}").len());
        }
        assert!(seen.windows(2).any(|pair| pair[0] != pair[1]));
    }

    #[test]
    fn watch_room_and_entry_resolve() {
        let skin = fluxa_skin::load_embedded(DEFAULT_FILES).unwrap();
        let model = fluxa_ui::WatchTogetherModel {
            room_code: Some("ABC123".to_owned()),
            host: true,
            members: vec![
                fluxa_ui::WatchMemberView {
                    id: "a".to_owned(),
                    name: "Anna".to_owned(),
                    you: true,
                    ..Default::default()
                },
                fluxa_ui::WatchMemberView {
                    id: "b".to_owned(),
                    name: "Marcus".to_owned(),
                    ..Default::default()
                },
            ],
            chat: vec![fluxa_ui::WatchChatView {
                name: "Marcus".to_owned(),
                text: "Hi".to_owned(),
                ..Default::default()
            }],
            ..Default::default()
        };
        for model in [model, Default::default()] {
            let data = json!({
                "const": skin.constants(),
                "settings": {},
                "fields": {},
                "watch": fluxa_ui::watch_data(&model),
            });
            for form in [FormFactor::Desktop, FormFactor::Mobile, FormFactor::Tv] {
                let tree = skin
                    .resolve("watch", &data, form, &mut Default::default())
                    .unwrap()
                    .unwrap();
                assert!(!tree.is_empty(), "{form:?}");
            }
        }
    }

    #[test]
    fn detail_modal_episodes_and_similar_carry_their_index() {
        let skin = fluxa_skin::load_embedded(DEFAULT_FILES).unwrap();
        let data = json!({
            "const": skin.constants(),
            "settings": {},
            "modal": {
                "title": "T", "backdrop": null, "meta": "", "description": "", "playLabel": "Play",
                "watchlistIcon": "Plus", "multiSeason": false, "seasons": [],
                "titles": {"episodes": "Episodes", "similar": "More"},
                "show": {"episodes": true, "similar": true},
                "episodes": [
                    {"index": "0", "label": "S1E1", "title": "A", "overview": "", "thumb": null},
                    {"index": "1", "label": "S1E2", "title": "B", "overview": "", "thumb": null},
                ],
                "similar": [{"title": "X", "artworkUrl": null}],
            },
        });
        let data = json!({"const": data["const"].clone(), "settings": {}, "detail": data["modal"].clone()});
        let tree = skin
            .resolve(
                "detail_modal",
                &data,
                FormFactor::Desktop,
                &mut Default::default(),
            )
            .unwrap()
            .unwrap();
        let mut found = Vec::new();
        count_actions(&tree, "modal", &mut found);
        assert!(found.len() >= 6, "{found:?}");
    }

    #[test]
    fn bundled_default_skin_loads() {
        let skin = fluxa_skin::load_embedded(DEFAULT_FILES).unwrap();
        assert_eq!(skin.manifest.id, "default");
    }
}
