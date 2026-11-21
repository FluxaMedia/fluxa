use super::*;
use std::sync::atomic::{AtomicU8, AtomicU32, Ordering};

static TOP_LABELS: AtomicU8 = AtomicU8::new(0);
pub(crate) fn set_top_labels(behavior: &str) {
    let mode = match behavior {
        "expanded" => 1,
        "collapsed" => 2,
        _ => 0,
    };
    TOP_LABELS.store(mode, Ordering::Relaxed);
}

static NAV_BOTTOM: AtomicU32 = AtomicU32::new(0);

pub(crate) fn set_nav_bottom(bottom: f32) {
    NAV_BOTTOM.store(bottom.to_bits(), Ordering::Relaxed);
}

pub fn nav_bar_bottom() -> f32 {
    f32::from_bits(NAV_BOTTOM.load(Ordering::Relaxed))
}
static NAV_LANGUAGE: std::sync::RwLock<String> = std::sync::RwLock::new(String::new());

pub fn set_nav_language(language: &str) {
    if let Ok(mut current) = NAV_LANGUAGE.write()
        && *current != language
    {
        *current = language.to_owned();
    }
}

const NAV_KEYS: [&str; 3] = ["home", "discover", "library"];
pub(crate) fn nav_text(key: &str) -> String {
    let language = NAV_LANGUAGE.read().map(|l| l.clone()).unwrap_or_default();
    localized(key, &language)
}

pub(crate) fn nav_labels() -> [String; 3] {
    let language = NAV_LANGUAGE.read().map(|l| l.clone()).unwrap_or_default();
    ["nav.home", "nav.discover", "nav.library"].map(|key| localized(key, &language))
}

pub(crate) fn nav_entries() -> Vec<(usize, String)> {
    let labels = nav_labels();
    let order =
        fluxa_theme::user::nav_items().unwrap_or_else(|| NAV_KEYS.map(str::to_owned).to_vec());
    order
        .iter()
        .filter_map(|key| NAV_KEYS.iter().position(|known| known == key))
        .map(|index| (index, labels[index].clone()))
        .collect()
}

pub fn nav_data(active: &str) -> serde_json::Value {
    let screens = ["home", "discover", "library"];
    let icons = ["NavHome", "NavSearch", "NavLibrary"];
    let items: Vec<_> = nav_entries()
        .into_iter()
        .map(|(index, label)| {
            let class = if screens[index] == active {
                "navbar-item-active"
            } else {
                "navbar-item"
            };
            serde_json::json!({
                "screen": screens[index],
                "label": label,
                "icon": icons[index],
                "class": class,
                "active": screens[index] == active,
            })
        })
        .collect();
    let on_item = items.iter().any(|item| item["active"] == true);
    serde_json::json!({
        "items": items,
        "profile": { "active": !on_item },
        "update": crate::update::update_data(),
    })
}

static NAV_RESERVE: AtomicU32 = AtomicU32::new(0);

pub(crate) fn set_nav_reserve(height: f32) {
    NAV_RESERVE.store(height.to_bits(), Ordering::Relaxed);
}

pub(crate) fn nav_reserve() -> f32 {
    f32::from_bits(NAV_RESERVE.load(Ordering::Relaxed))
}

pub fn is_navigation_node(node: u64) -> bool {
    matches!(
        node,
        NODE_HOME
            | NODE_LIBRARY
            | NODE_DISCOVER
            | NODE_CALENDAR
            | NODE_LIVE_TV
            | NODE_PROFILE
            | NODE_SIDEBAR_SEARCH
            | NODE_SIDEBAR_DOWNLOADS
            | NODE_UPDATE_OPEN
    )
}
