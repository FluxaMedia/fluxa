use super::*;
use crate::components::{glass_shape, liquid_glass};
use crate::navigation::{
    NAV_AVATAR_RADIUS, NAV_ITEM_NODES, is_navigation_node, nav_entries, nav_text,
};
use std::sync::atomic::{AtomicU8, AtomicU32, AtomicU64, Ordering};

const ITEM_INSET: f32 = 10.0;
const ITEM_GAP: f32 = 4.0;
const HEADING_HEIGHT: f32 = 30.0;
const TOP: f32 = 20.0;
const BOTTOM: f32 = 16.0;

const OFF: u8 = 0;
const HOVER: u8 = 1;
const EXPANDED: u8 = 2;
const COLLAPSED: u8 = 3;

const ROUTE_SETTINGS: usize = 4;
const ROUTE_CALENDAR: usize = 5;
const ROUTE_DOWNLOADS: usize = 6;

static MODE: AtomicU8 = AtomicU8::new(OFF);
static FOCUSED: AtomicU64 = AtomicU64::new(0);
static EXPAND: AtomicU32 = AtomicU32::new(0);
static QUERY: std::sync::RwLock<String> = std::sync::RwLock::new(String::new());
static TYPED: std::sync::Mutex<Option<String>> = std::sync::Mutex::new(None);

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
    Profile,
}

struct Slot {
    node: u64,
    rect: Rect,
    kind: Kind,
}

pub fn set_nav_sidebar(layout: &str, behavior: &str) {
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

fn item_height(viewport: Viewport) -> f32 {
    if viewport.is_tv() { 56.0 } else { 44.0 }
}

fn panel_width(viewport: Viewport, t: f32) -> f32 {
    let (rail, expanded) = widths(viewport.form_factor);
    rail + (expanded - rail) * t
}

fn slots(viewport: Viewport) -> Vec<Slot> {
    let origin = -viewport.side_inset;
    let width = panel_width(viewport, expand()) - ITEM_INSET * 2.0;
    let height = item_height(viewport);
    let x = origin + ITEM_INSET;
    let mut y = TOP;
    let mut out = Vec::new();
    let mut place = |node: u64, kind: Kind, h: f32, y: &mut f32| {
        out.push(Slot {
            node,
            rect: Rect::from_min_size(Pos2::new(x, *y), Vec2::new(width, h)),
            kind,
        });
        *y += h;
    };
    place(0, Kind::Brand, height + 12.0, &mut y);
    place(NODE_SIDEBAR_SEARCH, Kind::Search, height, &mut y);
    y += 6.0;
    place(
        0,
        Kind::Heading(nav_text("nav.section_browse")),
        HEADING_HEIGHT,
        &mut y,
    );
    for (index, label) in nav_entries() {
        let icon = ["Home", "Discover", "Library", "Shorts"][index];
        place(
            NAV_ITEM_NODES[index],
            Kind::Link {
                icon,
                label,
                route: index,
            },
            height,
            &mut y,
        );
        y += ITEM_GAP;
    }
    place(
        0,
        Kind::Heading(nav_text("nav.section_your_space")),
        HEADING_HEIGHT,
        &mut y,
    );
    place(
        NODE_CALENDAR,
        Kind::Link {
            icon: "Calendar",
            label: nav_text("nav.calendar"),
            route: ROUTE_CALENDAR,
        },
        height,
        &mut y,
    );
    y += ITEM_GAP;
    place(
        NODE_SIDEBAR_DOWNLOADS,
        Kind::Link {
            icon: "Downloads",
            label: nav_text("library.downloads"),
            route: ROUTE_DOWNLOADS,
        },
        height,
        &mut y,
    );
    let card = height + 12.0;
    let mut bottom = viewport.height - viewport.safe_bottom - BOTTOM - card;
    place(NODE_PROFILE, Kind::Profile, card, &mut bottom);
    out
}

pub(crate) fn item_rects(viewport: Viewport) -> Vec<(u64, Rect)> {
    slots(viewport)
        .into_iter()
        .filter(|slot| slot.node != 0)
        .map(|slot| (slot.node, slot.rect))
        .collect()
}

fn icon_at(
    ui: &egui::Ui,
    assets: &impl HomeAssets,
    name: &str,
    center: Pos2,
    size: f32,
    color: Color32,
) {
    if let Some(icon) = assets.icon(name) {
        ui.painter().image(
            icon,
            Rect::from_center_size(center, Vec2::splat(size)),
            full_uv(),
            color,
        );
    }
}

pub(crate) fn draw_sidebar(
    context: &egui::Context,
    viewport: Viewport,
    active_route: usize,
    metrics: UiMetrics,
    assets: &impl HomeAssets,
    profile_avatar_url: Option<&str>,
    profile_name: &str,
) -> Option<u64> {
    let mode = MODE.load(Ordering::Relaxed);
    let (rail, _) = widths(viewport.form_factor);
    let origin = -viewport.side_inset;
    let current = panel_width(viewport, expand());
    let hovered = context.pointer_hover_pos().is_some_and(|pos| {
        Rect::from_min_size(Pos2::new(origin, 0.0), Vec2::new(current, viewport.height))
            .contains(pos)
    });
    let focused = is_navigation_node(FOCUSED.load(Ordering::Relaxed));
    let target = match mode {
        EXPANDED => 1.0,
        COLLAPSED => 0.0,
        _ => f32::from(hovered || focused),
    };
    let t = crate::motion::ease(
        context,
        Id::new("fluxa-side-bar-expand"),
        target,
        crate::motion::normal(),
        crate::motion::Ease::Out,
    );
    EXPAND.store(t.to_bits(), Ordering::Relaxed);

    let label_size = metrics.navigation_label_size();
    let icon_size = metrics.navigation_icon_size();
    let font = FontId::proportional(label_size);
    let small = FontId::proportional((label_size - 3.0).max(11.0));
    let panel = Rect::from_min_size(
        Pos2::new(origin, 0.0),
        Vec2::new(panel_width(viewport, t), viewport.height),
    );
    let label_alpha = ((t - 0.35) / 0.65).clamp(0.0, 1.0);
    let icon_x = origin + rail * 0.5;
    let label_left = icon_x + icon_size * 0.5 + 14.0;
    let all = slots(viewport);
    let mut activated = None;
    egui::Area::new(Id::new("fluxa-shared-side-bar"))
        .fixed_pos(panel.min)
        .order(egui::Order::Foreground)
        .show(context, |ui| {
            let fill = overlay_panel();
            if liquid_glass() {
                ui.painter().add(glass_shape(panel, 0.0, fill));
            } else {
                ui.painter().rect_filled(panel, 0.0, fill);
            }
            ui.painter().line_segment(
                [panel.right_top(), panel.right_bottom()],
                egui::Stroke::new(stroke::hairline(), metrics.border),
            );
            ui.interact(panel, Id::new("fluxa-side-bar-panel"), Sense::click());

            let active = all.iter().find(|slot| match slot.kind {
                Kind::Link { route, .. } => route == active_route,
                Kind::Profile => active_route == ROUTE_SETTINGS,
                _ => false,
            });
            if let Some(slot) = active {
                let y = crate::motion::ease(
                    ui.ctx(),
                    Id::new("fluxa-side-bar-indicator-y"),
                    slot.rect.center().y,
                    crate::motion::normal(),
                    crate::motion::Ease::Linear,
                );
                let rect =
                    Rect::from_center_size(Pos2::new(slot.rect.center().x, y), slot.rect.size());
                ui.painter()
                    .add(glass_shape(rect, rect.height() * 0.5, ink(tone::soft())));
                let bar = Rect::from_center_size(
                    Pos2::new(origin + 2.0, y),
                    Vec2::new(4.0, rect.height() * 0.5),
                );
                ui.painter().rect_filled(bar, 2.0, text_primary());
            }

            for slot in &all {
                let rect = slot.rect;
                let radius = rect.height() * 0.5;
                let mut hover = false;
                let field = matches!(slot.kind, Kind::Search) && t > 0.6;
                if slot.node != 0 && !field {
                    let response = ui.interact(
                        rect,
                        Id::new(("fluxa-side-bar-item", slot.node)),
                        Sense::click(),
                    );
                    hover = response.hovered();
                    if response.clicked() {
                        activated = Some(slot.node);
                    }
                }
                let center = Pos2::new(icon_x, rect.center().y);
                match &slot.kind {
                    Kind::Brand => {
                        let mark = Rect::from_center_size(center, Vec2::splat(30.0));
                        if let Some(texture) = assets.brand_mark() {
                            ui.painter().image(texture, mark, full_uv(), Color32::WHITE);
                        }
                        if label_alpha > 0.02 {
                            draw_text(
                                ui.painter(),
                                Pos2::new(mark.right() + 12.0, rect.center().y),
                                Align2::LEFT_CENTER,
                                "Fluxa".to_owned(),
                                FontId::proportional(label_size + 4.0),
                                text_primary().gamma_multiply(label_alpha),
                                None,
                            );
                        }
                    }
                    Kind::Search if t > 0.6 => {
                        let field_id = Id::new("fluxa-side-bar-search-focus");
                        let had_focus = ui
                            .data(|data| data.get_temp::<bool>(field_id))
                            .unwrap_or(false);
                        let mut query = if had_focus {
                            QUERY.read().map(|q| q.clone()).unwrap_or_default()
                        } else {
                            String::new()
                        };
                        let response = ui
                            .scope_builder(egui::UiBuilder::new().max_rect(rect), |ui| {
                                crate::components::search_field(
                                    ui,
                                    &mut query,
                                    &nav_text("nav.search_placeholder"),
                                    rect.width(),
                                    rect.height(),
                                    metrics,
                                )
                            })
                            .inner;
                        let focused_now = response.has_focus();
                        ui.data_mut(|data| data.insert_temp(field_id, focused_now));
                        if response.changed()
                            && let Ok(mut typed) = TYPED.lock()
                        {
                            *typed = Some(query);
                        }
                    }
                    Kind::Search => {
                        let fill = if hover {
                            ink(tone::soft())
                        } else {
                            ink(tone::faint())
                        };
                        ui.painter().rect_filled(rect, radius, fill);
                        icon_at(ui, assets, "Search", center, icon_size, ink(tone::subtle()));
                        if label_alpha > 0.02 {
                            draw_text(
                                ui.painter(),
                                Pos2::new(label_left, rect.center().y),
                                Align2::LEFT_CENTER,
                                truncate_to_width(
                                    ui.painter(),
                                    &nav_text("nav.search_placeholder"),
                                    &font,
                                    rect.right() - 12.0 - label_left,
                                ),
                                font.clone(),
                                ink(tone::muted()).gamma_multiply(label_alpha),
                                None,
                            );
                        }
                    }
                    Kind::Heading(text) => {
                        if label_alpha > 0.02 {
                            draw_text(
                                ui.painter(),
                                Pos2::new(rect.left() + 12.0, rect.bottom() - 6.0),
                                Align2::LEFT_BOTTOM,
                                text.clone(),
                                small.clone(),
                                ink(tone::muted()).gamma_multiply(label_alpha),
                                None,
                            );
                        }
                        let rule = 1.0 - label_alpha;
                        if rule > 0.02 {
                            ui.painter().line_segment(
                                [
                                    Pos2::new(icon_x - 10.0, rect.bottom() - 8.0),
                                    Pos2::new(icon_x + 10.0, rect.bottom() - 8.0),
                                ],
                                egui::Stroke::new(
                                    stroke::hairline(),
                                    ink(tone::faint()).gamma_multiply(rule),
                                ),
                            );
                        }
                    }
                    Kind::Link { icon, label, route } => {
                        let is_active = *route == active_route;
                        if hover && !is_active {
                            ui.painter().rect_filled(rect, radius, ink(tone::faint()));
                        }
                        let color = if is_active || hover {
                            text_primary()
                        } else {
                            ink(tone::subtle())
                        };
                        icon_at(ui, assets, icon, center, icon_size, color);
                        if label_alpha > 0.02 {
                            draw_text(
                                ui.painter(),
                                Pos2::new(label_left, rect.center().y),
                                Align2::LEFT_CENTER,
                                truncate_to_width(
                                    ui.painter(),
                                    label,
                                    &font,
                                    rect.right() - 12.0 - label_left,
                                ),
                                font.clone(),
                                color.gamma_multiply(label_alpha),
                                None,
                            );
                        }
                    }
                    Kind::Profile => {
                        let card = if active_route == ROUTE_SETTINGS {
                            ink(tone::soft())
                        } else if hover {
                            ink(tone::faint())
                        } else {
                            Color32::TRANSPARENT
                        };
                        ui.painter().rect_filled(rect, radius.min(18.0), card);
                        if let Some(texture) = assets.cached_texture(profile_avatar_url) {
                            paint_circle_texture(ui.painter(), texture, center, NAV_AVATAR_RADIUS);
                        } else {
                            ui.painter().circle_filled(
                                center,
                                NAV_AVATAR_RADIUS,
                                ink(tone::raised()),
                            );
                            icon_at(ui, assets, "Account", center, 16.0, ink(tone::body()));
                        }
                        if label_alpha > 0.02 {
                            let text_width = rect.right() - 40.0 - label_left;
                            draw_text(
                                ui.painter(),
                                Pos2::new(label_left, rect.center().y - 8.0),
                                Align2::LEFT_CENTER,
                                truncate_to_width(ui.painter(), profile_name, &font, text_width),
                                font.clone(),
                                text_primary().gamma_multiply(label_alpha),
                                None,
                            );
                            draw_text(
                                ui.painter(),
                                Pos2::new(label_left, rect.center().y + 9.0),
                                Align2::LEFT_CENTER,
                                nav_text("nav.settings"),
                                small.clone(),
                                ink(tone::muted()).gamma_multiply(label_alpha),
                                None,
                            );
                            icon_at(
                                ui,
                                assets,
                                "ChevronRight",
                                Pos2::new(rect.right() - 20.0, rect.center().y),
                                16.0,
                                ink(tone::muted()).gamma_multiply(label_alpha),
                            );
                        }
                    }
                }
            }
        });
    if (t - target).abs() > 0.002 {
        context.request_repaint();
    }
    activated
}
