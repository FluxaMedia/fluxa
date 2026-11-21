use super::*;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicU8, AtomicU32, Ordering};

static TOP_LABELS: AtomicU8 = AtomicU8::new(0);
static TOP_T: AtomicU32 = AtomicU32::new(0);

pub(crate) fn set_top_labels(behavior: &str) {
    let mode = match behavior {
        "expanded" => 1,
        "collapsed" => 2,
        _ => 0,
    };
    TOP_LABELS.store(mode, Ordering::Relaxed);
}

pub fn set_mobile_nav_labels(labels: bool) {
    NAV_LABELS.store(labels, Ordering::Relaxed);
}

pub(crate) fn top_label_t() -> f32 {
    f32::from_bits(TOP_T.load(Ordering::Relaxed))
}

pub(crate) fn labels_below() -> bool {
    NAV_LABELS.load(Ordering::Relaxed)
}

static DRAWN_PROFILE: Mutex<String> = Mutex::new(String::new());

pub(crate) fn drawn_profile() -> String {
    match DRAWN_PROFILE.lock() {
        Ok(name) if !name.is_empty() => name.clone(),
        _ => "Profile".to_owned(),
    }
}

pub(crate) fn nav_bar_top() -> f32 {
    crate::vars::px("nav-bar-top")
}
pub(crate) fn nav_item_height() -> f32 {
    crate::vars::px("nav-item-height")
}
pub(crate) fn nav_bar_padding() -> f32 {
    crate::vars::px("nav-bar-padding")
}

pub fn nav_bar_bottom(viewport: Viewport) -> f32 {
    if viewport.side_inset > 0.0 {
        return 0.0;
    }
    nav_bar_top() + viewport.nav_lead + nav_item_height() + nav_bar_padding() * 2.0
}
pub(crate) fn nav_pill_gap() -> f32 {
    crate::vars::px("nav-item-gap")
}
pub(crate) fn nav_avatar_radius() -> f32 {
    crate::vars::px("nav-avatar-radius")
}

pub(crate) fn estimated_navigation_text_width(text: &str, size: f32) -> f32 {
    text.chars()
        .map(|character| match character {
            'i' | 'l' | 'I' | '!' | '.' => 0.30,
            'm' | 'w' | 'M' | 'W' => 0.82,
            ' ' => 0.32,
            _ => 0.54,
        })
        .sum::<f32>()
        * size
        + 4.0
}

static NAV_LABELS: AtomicBool = AtomicBool::new(false);
static NAV_LANGUAGE: std::sync::RwLock<String> = std::sync::RwLock::new(String::new());

pub fn set_nav_language(language: &str) {
    if let Ok(mut current) = NAV_LANGUAGE.write()
        && *current != language
    {
        *current = language.to_owned();
    }
}

const NAV_KEYS: [&str; 3] = ["home", "discover", "library"];
pub(crate) const NAV_ITEM_NODES: [u64; 3] = [NODE_HOME, NODE_DISCOVER, NODE_LIBRARY];

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

pub(crate) fn nav_nodes() -> Vec<u64> {
    nav_entries()
        .iter()
        .map(|(index, _)| NAV_ITEM_NODES[*index])
        .chain([NODE_PROFILE])
        .collect()
}

pub fn mobile_nav_rect(viewport: Viewport) -> Rect {
    let labels = NAV_LABELS.load(Ordering::Relaxed);
    let (slot, item) = if labels {
        (68.0, 58.0)
    } else {
        (52.0, nav_item_height())
    };
    let slots = vec![crate::kit::Slot::sized(slot, item); nav_nodes().len()];
    let content = crate::kit::solve(
        crate::kit::Flow::Row,
        0.0,
        Pos2::ZERO,
        Vec2::new(1.0e7, item),
        &slots,
    )
    .last()
    .map_or(0.0, |last| last.right());
    let width = (content + nav_bar_padding() * 2.0).min(viewport.width - 16.0);
    let height = item + nav_bar_padding() * 2.0;
    let bottom = viewport.height - viewport.safe_bottom - nav_bar_top();
    crate::flex::anchored(
        crate::gfx::rect_at(Pos2::ZERO, Vec2::new(viewport.width, bottom.max(height))),
        crate::gfx::Align2::CENTER_BOTTOM,
        Vec2::new(width, height),
        0.0,
    )
}

pub fn mobile_nav_reserve(viewport: Viewport) -> f32 {
    viewport.height - mobile_nav_rect(viewport).top()
}

pub fn navigation_focus_rects(viewport: Viewport) -> Vec<(u64, Rect)> {
    if viewport.side_inset > 0.0 {
        return crate::sidebar::item_rects(viewport);
    }
    crate::navbar::focus_rects(viewport)
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
