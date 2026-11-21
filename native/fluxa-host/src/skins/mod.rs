use crate::route::{NativeAction, Route};
use fluxa_skin::{FormFactor, Resolved, ResolvedAction, Skin};
use fluxa_ui::{HomeLayout, HomeModel, SettingsModel, SkinFrame, SkinTarget, UiFormFactorJson};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::sync::{Arc, LazyLock, Mutex, RwLock};

mod bundled;
mod consent;
mod ext;
mod fingerprint;
mod install;
mod nodes;
#[cfg(test)]
mod tests;
mod theme;
mod tree;
mod watch;

use self::bundled::*;
pub(crate) use self::ext::{act as script_act, observe as script_observe};
pub(crate) use self::fingerprint::Extra;
use self::fingerprint::*;
use self::install::*;
pub use self::install::{activate, install};
pub(crate) use self::install::{install_from, set_scripts, use_skin};
use self::nodes::*;
pub(crate) use self::nodes::{
    Local, action_for_node, detail_node, edit_field, field_text, field_value, fill_for_node,
    is_field, item_for_node, local_for_node, remember, remember_more, set_field, take_field,
};
use self::theme::*;
pub(crate) use self::theme::{clear_custom_css, load};
pub(crate) use self::tree::{TREE_NANOS, begin_frame, touch, tree};
use self::watch::*;
pub(crate) use self::watch::{next_watch, poll_dev};
static DEFAULT: RwLock<Option<Arc<Skin>>> = RwLock::new(None);
static ACTIVE: RwLock<Option<Arc<Skin>>> = RwLock::new(None);
static PATCHED: RwLock<Option<Arc<Skin>>> = RwLock::new(None);
static PAGE: Mutex<Option<String>> = Mutex::new(None);
static TARGETS: Mutex<Vec<(u64, SkinTarget)>> = Mutex::new(Vec::new());
static RESOLVE: LazyLock<Mutex<HashMap<String, (fluxa_skin::ResolveCache, u64)>>> =
    LazyLock::new(Default::default);
static DATA: LazyLock<Mutex<HashMap<String, (u64, Arc<Value>)>>> = LazyLock::new(Default::default);
static CACHE: LazyLock<Mutex<HashMap<String, (u64, Arc<Vec<Resolved>>)>>> =
    LazyLock::new(Default::default);
static STAMPS: LazyLock<Mutex<HashMap<String, u64>>> = LazyLock::new(Default::default);
static IDLE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
static IDLE_FRAMES: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
const IDLE_REFRESH: u32 = 6;
static FIELDS: Mutex<Option<HashMap<String, String>>> = Mutex::new(None);
static FAILED: Mutex<Option<String>> = Mutex::new(None);
static MANAGER: Mutex<Manager> = Mutex::new(Manager {
    data_dir: None,
    installed: Vec::new(),
    allowed: Vec::new(),
    message: None,
    busy: false,
    generation: 0,
});

struct Manager {
    data_dir: Option<std::path::PathBuf>,
    installed: Vec<(String, String, String, String)>,
    allowed: Vec<String>,
    message: Option<String>,
    busy: bool,
    generation: u64,
}

fn skin_for(screen: &str) -> Option<Arc<Skin>> {
    let active = ACTIVE.read().ok()?.clone();
    let default = PATCHED
        .read()
        .ok()?
        .clone()
        .or_else(|| DEFAULT.read().ok()?.clone());
    [active, default]
        .into_iter()
        .flatten()
        .find(|skin| skin.replaces(screen))
}

pub(crate) fn open_page(name: &str) {
    let screen = format!("page.{name}");
    if skin_for(&screen).is_some()
        && let Ok(mut page) = PAGE.lock()
    {
        *page = Some(screen);
    }
}

pub(crate) fn close_page() -> bool {
    PAGE.lock().is_ok_and(|mut page| page.take().is_some())
}

pub(crate) fn page_open() -> bool {
    PAGE.lock().is_ok_and(|page| page.is_some())
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
        "short": items.len() <= 9,
        "long": items.len() > 9,
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

fn decorate(card: &mut Value, source: &fluxa_ui::HomeCard, library: &fluxa_ui::LibraryModel) {
    let personal = source
        .id
        .as_deref()
        .and_then(|id| library.personal.get(id))
        .unwrap_or_default();
    let still = (source.row_kind == fluxa_ui::HomeRowKind::Continue)
        .then(|| {
            source
                .raw
                .get("lastEpisodeThumbnail")
                .and_then(Value::as_str)
                .filter(|url| !url.is_empty())
        })
        .flatten();
    card["saved"] = json!(personal.saved);
    card["watched"] = json!(personal.watched);
    card["previewArt"] = json!(
        still
            .or(source.backdrop_url.as_deref())
            .or(source.artwork_url.as_deref())
    );
    card["percent"] = json!(source.progress.max(personal.progress));
    card["hasProgress"] = json!(source.progress > 0.0 || personal.progress > 0.0);
}

fn truthy_arg(value: &Value) -> bool {
    match value {
        Value::Bool(flag) => *flag,
        Value::String(text) => text == "true",
        _ => false,
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
        "mark_watched" => {
            let id = text(action, "id")?;
            let watched = action.args.get("watched").is_some_and(truthy_arg);
            let meta = fluxa_ui::card_raw(&id)?;
            json!({"type": "coreCommand", "command": {
                "type": "markWatchedRequested",
                "seriesId": id,
                "videoIds": [id],
                "watched": !watched,
                "meta": meta,
            }})
        }
        "clear_progress" => {
            let meta = fluxa_ui::card_raw(&text(action, "id")?)?;
            json!({"type": "coreCommand", "command": {"type": "clearPlaybackProgressRequested", "meta": meta}})
        }
        "watchlist_toggle" => {
            json!({"type": "toggleWatchlist", "item": {"id": text(action, "id")?, "type": text(action, "type")?}})
        }
        _ => return None,
    };
    serde_json::from_value(value).ok()
}
