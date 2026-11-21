use super::*;
use std::sync::atomic::{AtomicU8, AtomicU64, Ordering};

const OFF: u8 = 0;
const HOVER: u8 = 1;
const EXPANDED: u8 = 2;
const COLLAPSED: u8 = 3;

static LIVE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
static MODE: AtomicU8 = AtomicU8::new(OFF);
static FOCUSED: AtomicU64 = AtomicU64::new(0);
static QUERY: std::sync::RwLock<String> = std::sync::RwLock::new(String::new());
static TYPED: std::sync::Mutex<Option<String>> = std::sync::Mutex::new(None);

pub fn set_nav_live(available: bool) {
    LIVE.store(available, Ordering::Relaxed);
}

pub fn set_nav_search(query: &str) {
    if let Ok(mut current) = QUERY.write()
        && *current != query
    {
        *current = query.to_owned();
    }
}

pub fn take_nav_search() -> Option<String> {
    TYPED.lock().ok()?.take()
}

pub fn set_nav_sidebar(layout: &str, behavior: &str) {
    crate::navigation::set_top_labels(behavior);
    let mode = if layout != "sidebar" {
        OFF
    } else {
        match behavior {
            "expanded" => EXPANDED,
            "collapsed" => COLLAPSED,
            _ => HOVER,
        }
    };
    MODE.store(mode, Ordering::Relaxed);
}

pub fn set_nav_focus(node: Option<u64>) {
    FOCUSED.store(node.unwrap_or(0), Ordering::Relaxed);
}

fn widths(form_factor: UiFormFactor) -> (f32, f32) {
    let form = if form_factor == UiFormFactor::Tv {
        "tv"
    } else {
        "desktop"
    };
    (
        crate::vars::px(&format!("navigation-rail-width-{form}")),
        crate::vars::px(&format!("navigation-rail-expanded-width-{form}")),
    )
}

pub(crate) fn side_inset(width: f32, form_factor: UiFormFactor) -> f32 {
    let mode = MODE.load(Ordering::Relaxed);
    if mode == OFF || form_factor == UiFormFactor::Mobile || width < 600.0 {
        return 0.0;
    }
    let (rail, expanded) = widths(form_factor);
    if mode == EXPANDED { expanded } else { rail }
}
