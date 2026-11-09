use super::*;
use crate::components::{glass_shape, liquid_glass};
use crate::dp::Dp;
use crate::navigation::{
    NAV_ITEM_NODES, is_navigation_node, nav_avatar_radius, nav_entries, nav_text,
};
use std::sync::atomic::{AtomicU8, AtomicU32, AtomicU64, Ordering};

const ICON: usize = 1;
const TITLE: usize = 2;
const DETAIL: usize = 3;
const TRACK: usize = 4;
const CHEVRON: usize = 5;
const RULE: usize = 6;

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

pub(crate) fn nav_focused() -> bool {
    is_navigation_node(FOCUSED.load(Ordering::Relaxed))
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
    Rect::from_min_size(
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
        let icon = ["Home", "Discover", "Library", "Shorts"][index];
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

fn icon_at(
    ui: &crate::gfx::Ui,
    assets: &mut impl HomeAssets,
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
    context: &crate::gfx::Context,
    viewport: Viewport,
    active_route: usize,
    metrics: UiMetrics,
    assets: &mut impl HomeAssets,
    profile_avatar_url: Option<&str>,
    profile_name: &str,
) -> Option<u64> {
    let mode = MODE.load(Ordering::Relaxed);
    let (rail, _) = widths(viewport.form_factor);
    let origin = -viewport.side_inset;
    let hovered = context
        .pointer_hover_pos()
        .is_some_and(|pos| bounds(viewport).contains(pos));
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
    let panel = bounds(viewport);
    let label_alpha = ((t - 0.35) / 0.65).clamp(0.0, 1.0);
    let cell = rail - Space::S10.px(&metrics) * 2.0;
    let gap = Space::S14.px(&metrics);
    let inset = Space::S12.px(&metrics);
    let lines_gap = Space::S4.px(&metrics);
    let bar_width = Dp::D4.px();
    let leading = |size: f32| crate::flex::Insets {
        left: (cell - size) / 2.0,
        top: 0.0,
        right: inset,
        bottom: 0.0,
    };
    let icon_leaf = |size: f32| crate::flex::leaf(Vec2::splat(size)).width(size).id(ICON);
    let label_leaf =
        |height: f32, id: usize| crate::flex::leaf(Vec2::new(0.0, height)).grow(1.0).id(id);
    let two_lines = |first: f32, second: f32| {
        crate::flex::column([
            crate::flex::leaf(Vec2::new(0.0, first)).id(TITLE),
            crate::flex::leaf(Vec2::new(0.0, second)).id(DETAIL),
        ])
        .align(crate::flex::Align::Stretch)
        .gap(lines_gap)
        .grow(1.0)
    };
    let all = slots(viewport);
    let status = crate::update::nav_status();
    let mut activated = None;
    crate::gfx::Area::new(Id::new("fluxa-shared-side-bar"))
        .fixed_pos(panel.min)
        .order(crate::gfx::Order::Foreground)
        .show(context, |ui| {
            let fill = overlay_panel();
            if liquid_glass() {
                ui.painter().add(glass_shape(panel, 0.0, fill));
            } else {
                ui.painter().rect_filled(panel, 0.0, fill);
            }
            ui.painter().line_segment(
                [panel.right_top(), panel.right_bottom()],
                crate::gfx::Stroke::new(stroke::hairline(), metrics.border),
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
                ui.painter().add(glass_shape(
                    rect,
                    rect.height() * 0.5,
                    crate::stylesheet::color("track"),
                ));
                let bar = crate::flex::column([
                    crate::flex::fill(),
                    crate::flex::fill().grow(2.0).id(ICON),
                    crate::flex::fill(),
                ])
                .solve(Rect::from_min_size(
                    Pos2::new(origin, rect.top()),
                    Vec2::new(bar_width, rect.height()),
                ))
                .find(ICON)
                .unwrap_or(rect);
                ui.painter()
                    .rect_filled(bar, bar_width / 2.0, text_primary());
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
                let font_height = crate::flex::text_size(ui.painter(), "Ag", &font).y;
                let small_height = crate::flex::text_size(ui.painter(), "Ag", &small).y;
                let glyph = crate::stylesheet::color("glyph").gamma_multiply(label_alpha);
                match &slot.kind {
                    Kind::Brand => {
                        let mark_size = Dp::D40.px();
                        let brand = FontId::proportional(label_size + 4.0);
                        let brand_height = crate::flex::text_size(ui.painter(), "Fluxa", &brand).y;
                        let placed = crate::flex::row([
                            icon_leaf(mark_size),
                            label_leaf(brand_height, TITLE),
                        ])
                        .align(crate::flex::Align::Center)
                        .gap(inset)
                        .padding(leading(mark_size))
                        .solve(rect);
                        if let Some(texture) = assets.brand_mark() {
                            let mark = placed.find(ICON).unwrap_or(rect);
                            ui.painter().image(texture, mark, full_uv(), Color32::WHITE);
                        }
                        if label_alpha > 0.02 {
                            let title = placed.find(TITLE).unwrap_or(rect);
                            draw_text(
                                ui.painter(),
                                title.left_center(),
                                Align2::LEFT_CENTER,
                                "Fluxa".to_owned(),
                                brand,
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
                            .scope_builder(crate::gfx::UiBuilder::new().max_rect(rect), |ui| {
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
                            crate::stylesheet::color("track")
                        } else {
                            crate::stylesheet::color("well")
                        };
                        ui.painter().rect_filled(rect, radius, fill);
                        let placed = crate::flex::row([
                            icon_leaf(icon_size),
                            label_leaf(font_height, TITLE),
                        ])
                        .align(crate::flex::Align::Center)
                        .gap(gap)
                        .padding(leading(icon_size))
                        .solve(rect);
                        icon_at(
                            ui,
                            assets,
                            "Search",
                            placed.find(ICON).unwrap_or(rect).center(),
                            icon_size,
                            crate::stylesheet::color("text.subtle"),
                        );
                        if label_alpha > 0.02 {
                            crate::flex::paint_text(
                                ui.painter(),
                                placed.find(TITLE).unwrap_or(rect),
                                &nav_text("nav.search_placeholder"),
                                &font,
                                glyph,
                                Align2::LEFT_CENTER,
                            );
                        }
                    }
                    Kind::Heading(text) => {
                        let rule_length = Space::S20.px(&metrics);
                        let title_size = crate::flex::text_size(ui.painter(), text, &small);
                        let placed = crate::flex::stack(
                            Align2::LEFT_BOTTOM,
                            [
                                crate::flex::stack(
                                    Align2::LEFT_BOTTOM,
                                    [crate::flex::leaf(title_size).id(TITLE)],
                                )
                                .padding(crate::flex::Insets {
                                    left: inset,
                                    top: 0.0,
                                    right: 0.0,
                                    bottom: Space::S6.px(&metrics),
                                })
                                .grow(1.0),
                                crate::flex::stack(
                                    Align2::LEFT_BOTTOM,
                                    [crate::flex::leaf(Vec2::new(rule_length, 0.0)).id(RULE)],
                                )
                                .padding(crate::flex::Insets {
                                    left: (cell - rule_length) / 2.0,
                                    top: 0.0,
                                    right: 0.0,
                                    bottom: Space::S8.px(&metrics),
                                })
                                .grow(1.0),
                            ],
                        )
                        .solve(rect);
                        if label_alpha > 0.02 {
                            let title = placed.find(TITLE).unwrap_or(rect);
                            draw_text(
                                ui.painter(),
                                title.left_bottom(),
                                Align2::LEFT_BOTTOM,
                                text.clone(),
                                small.clone(),
                                glyph,
                                None,
                            );
                        }
                        let rule = 1.0 - label_alpha;
                        if rule > 0.02 {
                            let line = placed.find(RULE).unwrap_or(rect);
                            ui.painter().line_segment(
                                [line.left_bottom(), line.right_bottom()],
                                crate::gfx::Stroke::new(
                                    stroke::hairline(),
                                    crate::stylesheet::color("well").gamma_multiply(rule),
                                ),
                            );
                        }
                    }
                    Kind::Link { icon, label, route } => {
                        let is_active = *route == active_route;
                        if hover && !is_active {
                            ui.painter().rect_filled(
                                rect,
                                radius,
                                crate::stylesheet::color("well"),
                            );
                        }
                        let color = if is_active || hover {
                            text_primary()
                        } else {
                            crate::stylesheet::color("text.subtle")
                        };
                        let placed = crate::flex::row([
                            icon_leaf(icon_size),
                            label_leaf(font_height, TITLE),
                        ])
                        .align(crate::flex::Align::Center)
                        .gap(gap)
                        .padding(leading(icon_size))
                        .solve(rect);
                        icon_at(
                            ui,
                            assets,
                            icon,
                            placed.find(ICON).unwrap_or(rect).center(),
                            icon_size,
                            color,
                        );
                        if label_alpha > 0.02 {
                            crate::flex::paint_text(
                                ui.painter(),
                                placed.find(TITLE).unwrap_or(rect),
                                label,
                                &font,
                                color.gamma_multiply(label_alpha),
                                Align2::LEFT_CENTER,
                            );
                        }
                    }
                    Kind::Update => {
                        let Some(status) = &status else {
                            continue;
                        };
                        if hover {
                            ui.painter().rect_filled(
                                rect,
                                radius.min(Dp::D16.px()),
                                crate::stylesheet::color("well"),
                            );
                        }
                        let track_inset = Space::S14.px(&metrics);
                        let mut layers = vec![
                            crate::flex::row([
                                icon_leaf(icon_size),
                                two_lines(font_height, small_height),
                            ])
                            .align(crate::flex::Align::Center)
                            .gap(gap)
                            .padding(leading(icon_size))
                            .grow(1.0),
                        ];
                        if status.progress.is_some() {
                            layers.push(
                                crate::flex::column([
                                    crate::flex::fill(),
                                    crate::flex::leaf(Vec2::ZERO).height(Dp::D3.px()).id(TRACK),
                                ])
                                .align(crate::flex::Align::Stretch)
                                .padding(crate::flex::Insets {
                                    left: track_inset,
                                    top: 0.0,
                                    right: track_inset,
                                    bottom: Space::S4.px(&metrics),
                                })
                                .grow(1.0),
                            );
                        }
                        let placed = crate::flex::stack(Align2::LEFT_TOP, layers).solve(rect);
                        icon_at(
                            ui,
                            assets,
                            status.icon,
                            placed.find(ICON).unwrap_or(rect).center(),
                            icon_size,
                            accent(),
                        );
                        if let (Some(fraction), Some(track)) = (status.progress, placed.find(TRACK))
                        {
                            let corner = track.height() / 2.0;
                            ui.painter().rect_filled(
                                track,
                                corner,
                                crate::stylesheet::color("track"),
                            );
                            let filled = Rect::from_min_size(
                                track.min,
                                Vec2::new(track.width() * fraction, track.height()),
                            );
                            ui.painter().rect_filled(filled, corner, accent());
                        }
                        if label_alpha > 0.02 {
                            crate::flex::paint_text(
                                ui.painter(),
                                placed.find(TITLE).unwrap_or(rect),
                                &status.title,
                                &font,
                                text_primary().gamma_multiply(label_alpha),
                                Align2::LEFT_CENTER,
                            );
                            draw_text(
                                ui.painter(),
                                placed.find(DETAIL).unwrap_or(rect).left_center(),
                                Align2::LEFT_CENTER,
                                status.detail.clone(),
                                small.clone(),
                                glyph,
                                None,
                            );
                        }
                    }
                    Kind::Profile => {
                        let card = if active_route == ROUTE_SETTINGS {
                            crate::stylesheet::color("track")
                        } else if hover {
                            crate::stylesheet::color("well")
                        } else {
                            Color32::TRANSPARENT
                        };
                        ui.painter()
                            .rect_filled(rect, radius.min(Dp::D16.px()), card);
                        let chevron = Dp::D16.px();
                        let placed = crate::flex::row([
                            icon_leaf(icon_size),
                            two_lines(font_height, small_height),
                            crate::flex::leaf(Vec2::splat(chevron))
                                .width(chevron)
                                .id(CHEVRON),
                        ])
                        .align(crate::flex::Align::Center)
                        .gap(gap)
                        .padding(leading(icon_size))
                        .solve(rect);
                        let center = placed.find(ICON).unwrap_or(rect).center();
                        if crate::components::dom_avatar(
                            ui.painter(),
                            assets,
                            center,
                            nav_avatar_radius(),
                            profile_avatar_url,
                        ) {
                        } else if let Some(texture) = assets.texture_for(
                            profile_avatar_url,
                            [96, 96],
                            ArtworkPriority::Visible,
                        ) {
                            paint_circle_texture(
                                ui.painter(),
                                texture,
                                center,
                                nav_avatar_radius(),
                            );
                            crate::components::animated_avatar(
                                ui.painter(),
                                assets,
                                center,
                                nav_avatar_radius(),
                                nav_avatar_radius(),
                                profile_avatar_url,
                            );
                        } else {
                            ui.painter().circle_filled(
                                center,
                                nav_avatar_radius(),
                                crate::stylesheet::color("raised"),
                            );
                        }
                        if label_alpha > 0.02 {
                            crate::flex::paint_text(
                                ui.painter(),
                                placed.find(TITLE).unwrap_or(rect),
                                profile_name,
                                &font,
                                text_primary().gamma_multiply(label_alpha),
                                Align2::LEFT_CENTER,
                            );
                            draw_text(
                                ui.painter(),
                                placed.find(DETAIL).unwrap_or(rect).left_center(),
                                Align2::LEFT_CENTER,
                                nav_text("nav.settings"),
                                small.clone(),
                                glyph,
                                None,
                            );
                            icon_at(
                                ui,
                                assets,
                                "ChevronRight",
                                placed.find(CHEVRON).unwrap_or(rect).center(),
                                chevron,
                                glyph,
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
