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

pub(crate) fn set_top_label_t(t: f32) {
    TOP_T.store(t.to_bits(), Ordering::Relaxed);
}

pub(crate) fn top_labels_mode() -> u8 {
    TOP_LABELS.load(Ordering::Relaxed)
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
    crate::metrics::active().nav_bar_top
}
pub(crate) fn nav_item_height() -> f32 {
    crate::metrics::active().nav_item_height
}
pub(crate) fn nav_bar_padding() -> f32 {
    crate::metrics::active().nav_bar_padding
}

pub fn nav_bar_bottom(viewport: Viewport) -> f32 {
    if viewport.side_inset > 0.0 {
        return 0.0;
    }
    nav_bar_top() + viewport.nav_lead + nav_item_height() + nav_bar_padding() * 2.0
}
pub(crate) fn nav_pill_gap() -> f32 {
    crate::metrics::active().nav_item_gap
}
pub(crate) fn nav_avatar_radius() -> f32 {
    crate::metrics::active().nav_avatar_radius
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

/// Draws the navigation shared by every native screen. The active route is an
/// index into Home/Discover/Library and the returned node id is
/// consumed by the platform event adapters.
pub fn draw_navigation_bar(
    context: &egui::Context,
    viewport: Viewport,
    active_route: usize,
    assets: &mut impl HomeAssets,
) -> Option<u64> {
    let avatar = assets.active_profile_avatar_url().map(ToOwned::to_owned);
    let name = assets.active_profile_name().unwrap_or("Profile").to_owned();
    draw_navigation_bar_with_profile(
        context,
        viewport,
        active_route,
        assets,
        avatar.as_deref(),
        &name,
    )
}

pub(crate) fn draw_navigation_bar_with_profile(
    context: &egui::Context,
    viewport: Viewport,
    active_route: usize,
    assets: &mut impl HomeAssets,
    profile_avatar_url: Option<&str>,
    profile_name: &str,
) -> Option<u64> {
    let metrics = metrics_for_assets(viewport, assets);
    if let Ok(mut name) = DRAWN_PROFILE.lock()
        && *name != profile_name
    {
        *name = profile_name.to_owned();
    }
    if viewport.side_inset > 0.0 {
        return crate::sidebar::draw_sidebar(
            context,
            viewport,
            active_route,
            metrics,
            assets,
            profile_avatar_url,
            profile_name,
        );
    }
    let active_route = if active_route > 4 { 2 } else { active_route };
    crate::navbar::draw(
        context,
        viewport,
        active_route,
        metrics,
        assets,
        profile_avatar_url,
        profile_name,
    )
}

static NAV_LABELS: AtomicBool = AtomicBool::new(false);
static NAV_LANGUAGE: std::sync::RwLock<String> = std::sync::RwLock::new(String::new());
static NAV_DRAG: AtomicU32 = AtomicU32::new(u32::MAX);

pub fn set_nav_language(language: &str) {
    if let Ok(mut current) = NAV_LANGUAGE.write()
        && *current != language
    {
        *current = language.to_owned();
    }
}

const NAV_KEYS: [&str; 4] = ["home", "discover", "library", "shorts"];
pub(crate) const NAV_ITEM_NODES: [u64; 4] = [NODE_HOME, NODE_DISCOVER, NODE_LIBRARY, NODE_SHORTS];

pub(crate) fn nav_text(key: &str) -> String {
    let language = NAV_LANGUAGE.read().map(|l| l.clone()).unwrap_or_default();
    localized(key, &language)
}

pub(crate) fn nav_labels() -> [String; 4] {
    let language = NAV_LANGUAGE.read().map(|l| l.clone()).unwrap_or_default();
    ["nav.home", "nav.discover", "nav.library", "nav.shorts"].map(|key| localized(key, &language))
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

pub(crate) fn nav_nodes() -> Vec<u64> {
    nav_entries()
        .iter()
        .map(|(index, _)| NAV_ITEM_NODES[*index])
        .chain([NODE_PROFILE])
        .collect()
}

pub(crate) fn nav_slot(active_route: usize) -> usize {
    let entries = nav_entries();
    entries
        .iter()
        .position(|(index, _)| *index == active_route)
        .unwrap_or(entries.len())
}

pub fn set_mobile_nav_drag(x: Option<f32>) {
    NAV_DRAG.store(x.map_or(u32::MAX, f32::to_bits), Ordering::Relaxed);
}

static NAV_PENDING: AtomicU32 = AtomicU32::new(u32::MAX);

pub fn set_mobile_nav_pending(x: f32) {
    NAV_PENDING.store(60, Ordering::Relaxed);
    NAV_DRAG.store(u32::MAX, Ordering::Relaxed);
    PENDING_X.store(x.to_bits(), Ordering::Relaxed);
}

static PENDING_X: AtomicU32 = AtomicU32::new(0);

pub(crate) fn mobile_nav_pending(active_x: f32) -> Option<f32> {
    let frames = NAV_PENDING.load(Ordering::Relaxed);
    if frames == u32::MAX {
        return None;
    }
    let x = f32::from_bits(PENDING_X.load(Ordering::Relaxed));
    if frames == 0 || (x - active_x).abs() < 1.0 {
        NAV_PENDING.store(u32::MAX, Ordering::Relaxed);
        return None;
    }
    NAV_PENDING.store(frames - 1, Ordering::Relaxed);
    Some(x)
}

pub(crate) fn mobile_nav_drag() -> Option<f32> {
    let bits = NAV_DRAG.load(Ordering::Relaxed);
    (bits != u32::MAX).then(|| f32::from_bits(bits))
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
    Rect::from_min_size(
        Pos2::new((viewport.width - width) * 0.5, (bottom - height).max(0.0)),
        Vec2::new(width, height),
    )
}

pub fn mobile_nav_reserve(viewport: Viewport) -> f32 {
    viewport.height - mobile_nav_rect(viewport).top()
}

pub fn navigation_focus_rects(viewport: Viewport, metrics: UiMetrics) -> Vec<(u64, Rect)> {
    if viewport.side_inset > 0.0 {
        return crate::sidebar::item_rects(viewport);
    }
    crate::navbar::focus_rects(viewport, metrics)
}

pub fn is_navigation_node(node: u64) -> bool {
    matches!(
        node,
        NODE_HOME
            | NODE_LIBRARY
            | NODE_DISCOVER
            | NODE_CALENDAR
            | NODE_SHORTS
            | NODE_PROFILE
            | NODE_SIDEBAR_SEARCH
            | NODE_SIDEBAR_DOWNLOADS
            | NODE_UPDATE_OPEN
    )
}
