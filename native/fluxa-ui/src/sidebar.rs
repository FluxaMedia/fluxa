use super::*;
use crate::dp::Dp;
use crate::navigation::{
    NAV_ITEM_NODES, nav_entries, nav_text,
};
use std::sync::atomic::{AtomicU8, AtomicU32, AtomicU64, Ordering};


const OFF: u8 = 0;
const HOVER: u8 = 1;
const EXPANDED: u8 = 2;
const COLLAPSED: u8 = 3;

const ROUTE_CALENDAR: usize = 5;
const ROUTE_DOWNLOADS: usize = 6;
pub(crate) const ROUTE_LIVE: usize = 7;

static LIVE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
static MODE: AtomicU8 = AtomicU8::new(OFF);
static FOCUSED: AtomicU64 = AtomicU64::new(0);
static EXPAND: AtomicU32 = AtomicU32::new(0);
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

enum Kind {
    Brand,
    Search,
    Heading(String),
    Link {
        icon: &'static str,
        label: String,
        route: usize,
    },
    Update,
    Profile,
}

struct Slot {
    node: u64,
    rect: Rect,
    kind: Kind,
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
    let (platform, rail, expanded) = if form_factor == UiFormFactor::Tv {
        ("tv", 104.0, 300.0)
    } else {
        ("desktop", 76.0, 232.0)
    };
    let read = |key: &str, fallback: f32| {
        fluxa_theme::theme::shared_ui_tokens()
            .ok()
            .and_then(|tokens| {
                tokens
                    .layout
                    .get("platforms")?
                    .get(platform)?
                    .get("dp")?
                    .get(key)?
                    .as_f64()
            })
            .map_or(fallback, |value| value as f32)
    };
    (
        read("navigationRailWidth", rail),
        read("navigationRailExpandedWidth", expanded),
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

fn expand() -> f32 {
    f32::from_bits(EXPAND.load(Ordering::Relaxed))
}

fn panel_width(viewport: Viewport, t: f32) -> f32 {
    let (rail, expanded) = widths(viewport.form_factor);
    rail + (expanded - rail) * t
}

fn bounds(viewport: Viewport) -> Rect {
    crate::gfx::rect_at(
        Pos2::new(-viewport.side_inset, 0.0),
        Vec2::new(panel_width(viewport, expand()), viewport.height),
    )
}

fn slots(viewport: Viewport) -> Vec<Slot> {
    let metrics = UiMetrics::for_viewport(viewport);
    let height = metrics.sidebar_item_height;
    let card = height + Space::S12.px(&metrics);
    let item_gap = Space::S4.px(&metrics);
    let link = |node: u64, icon: &'static str, label: String, route: usize, lead: f32| {
        (node, Kind::Link { icon, label, route }, height, lead)
    };
    let mut items = vec![
        (0, Kind::Brand, card, 0.0),
        (NODE_SIDEBAR_SEARCH, Kind::Search, height, 0.0),
        (
            0,
            Kind::Heading(nav_text("nav.section_browse")),
            Dp::D28.px(),
            Space::S6.px(&metrics),
        ),
    ];
    for (index, label) in nav_entries() {
        let icon = ["Home", "Discover", "Library"][index];
        let lead = if index == 0 { 0.0 } else { item_gap };
        items.push(link(NAV_ITEM_NODES[index], icon, label, index, lead));
    }
    items.push((
        0,
        Kind::Heading(nav_text("nav.section_your_space")),
        Dp::D28.px(),
        item_gap,
    ));
    items.push(link(
        NODE_CALENDAR,
        "Calendar",
        nav_text("nav.calendar"),
        ROUTE_CALENDAR,
        0.0,
    ));
    if LIVE.load(Ordering::Relaxed) {
        items.push(link(
            NODE_LIVE_TV,
            "Live",
            nav_text("nav.live_tv"),
            ROUTE_LIVE,
            item_gap,
        ));
    }
    items.push(link(
        NODE_SIDEBAR_DOWNLOADS,
        "Downloads",
        nav_text("library.downloads"),
        ROUTE_DOWNLOADS,
        item_gap,
    ));
    let profile_id = items.len();
    let update_id = profile_id + 1;
    let mut children = Vec::new();
    for (index, (_, _, height, lead)) in items.iter().enumerate() {
        if *lead > 0.0 {
            children.push(crate::flex::leaf(Vec2::new(0.0, *lead)));
        }
        children.push(crate::flex::leaf(Vec2::ZERO).height(*height).id(index));
    }
    children.push(crate::flex::fill());
    let updating = crate::update::nav_status().is_some();
    if updating {
        children.push(crate::flex::leaf(Vec2::ZERO).height(card).id(update_id));
        children.push(crate::flex::leaf(Vec2::new(0.0, item_gap)));
    }
    children.push(crate::flex::leaf(Vec2::ZERO).height(card).id(profile_id));
    let inset = Space::S10.px(&metrics);
    let placed = crate::flex::column(children)
        .align(crate::flex::Align::Stretch)
        .padding(crate::flex::Insets {
            left: inset,
            top: Space::S20.px(&metrics),
            right: inset,
            bottom: Space::S16.px(&metrics) + viewport.safe_bottom,
        })
        .solve(bounds(viewport));
    let rect = |id: usize| placed.find(id).unwrap_or(Rect::NOTHING);
    let mut out: Vec<Slot> = items
        .into_iter()
        .enumerate()
        .map(|(index, (node, kind, _, _))| Slot {
            node,
            rect: rect(index),
            kind,
        })
        .collect();
    out.push(Slot {
        node: NODE_PROFILE,
        rect: rect(profile_id),
        kind: Kind::Profile,
    });
    if updating {
        out.push(Slot {
            node: NODE_UPDATE_OPEN,
            rect: rect(update_id),
            kind: Kind::Update,
        });
    }
    out
}

pub(crate) fn item_rects(viewport: Viewport) -> Vec<(u64, Rect)> {
    slots(viewport)
        .into_iter()
        .filter(|slot| slot.node != 0)
        .map(|slot| (slot.node, slot.rect))
        .collect()
}

