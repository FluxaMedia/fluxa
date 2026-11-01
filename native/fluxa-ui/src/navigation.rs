use super::*;
use crate::components::{glass, glass_shape, liquid_glass};
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

static DRAWN_PROFILE: Mutex<String> = Mutex::new(String::new());

fn drawn_profile() -> String {
    match DRAWN_PROFILE.lock() {
        Ok(name) if !name.is_empty() => name.clone(),
        _ => "Profile".to_owned(),
    }
}

pub(crate) const NAV_BAR_TOP: f32 = 14.0;
pub(crate) const NAV_ITEM_HEIGHT: f32 = 44.0;
pub(crate) const NAV_BAR_PADDING: f32 = 6.0;

pub fn nav_bar_bottom(viewport: Viewport) -> f32 {
    if viewport.side_inset > 0.0 {
        return 0.0;
    }
    NAV_BAR_TOP + viewport.nav_lead + NAV_ITEM_HEIGHT + NAV_BAR_PADDING * 2.0
}
pub(crate) const NAV_ITEM_GAP: f32 = 2.0;
pub(crate) const NAV_AVATAR_RADIUS: f32 = 15.0;

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
    let compact = viewport.is_compact();
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
    if compact {
        return draw_mobile_navigation_bar(
            context,
            viewport,
            active_route,
            metrics,
            assets,
            profile_avatar_url,
            profile_name,
        );
    }
    let bar_id = Id::new("fluxa-top-bar-rect");
    let hovered = context
        .data(|data| data.get_temp::<Rect>(bar_id))
        .zip(context.pointer_hover_pos())
        .is_some_and(|(rect, pos)| rect.expand(6.0).contains(pos));
    let focused = crate::sidebar::nav_focused();
    let target = match TOP_LABELS.load(Ordering::Relaxed) {
        1 => 1.0,
        2 => 0.0,
        _ => f32::from(hovered || focused),
    };
    let label_t = crate::motion::ease(
        context,
        Id::new("fluxa-top-bar-labels"),
        target,
        crate::motion::normal(),
        crate::motion::Ease::Out,
    );
    TOP_T.store(label_t.to_bits(), Ordering::Relaxed);
    let label_alpha = ((label_t - 0.35) / 0.65).clamp(0.0, 1.0);
    let label_size = metrics.navigation_label_size();
    let icon_size = metrics.navigation_icon_size();
    let profile_width = metrics.navigation_profile_width(profile_name);
    let bar_width = metrics.navigation_bar_width(viewport, profile_name) + NAV_BAR_PADDING * 2.0;
    let bar_x = (viewport.width - bar_width).max(12.0) * 0.5;
    let font = FontId::proportional(label_size);
    let mut activated = None;
    egui::Area::new(Id::new("fluxa-shared-top-bar"))
        .fixed_pos(Pos2::new(bar_x, NAV_BAR_TOP + viewport.nav_lead))
        .order(egui::Order::Foreground)
        .show(context, |ui| {
            let fill = overlay_panel();
            let backdrop = ui.painter().add(egui::Shape::Noop);
            let frame = egui::Frame::NONE
                .fill(if liquid_glass() {
                    Color32::TRANSPARENT
                } else {
                    fill
                })
                .stroke(egui::Stroke::new(stroke::hairline(), metrics.border))
                .corner_radius(NAV_ITEM_HEIGHT * 0.5 + NAV_BAR_PADDING)
                .inner_margin(egui::Margin::same(NAV_BAR_PADDING as i8))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = NAV_ITEM_GAP;
                        let pill = ui.painter().add(egui::Shape::Noop);
                        let entries = nav_entries();
                        let mut slots = vec![Rect::NOTHING; entries.len() + 1];
                        for (position, (original, label)) in entries.iter().enumerate() {
                            let icon =
                                ["NavHome", "NavSearch", "NavLibrary", "NavShorts"][*original];
                            let active = *original == active_route;
                            let width = metrics.navigation_item_width(label);
                            let (rect, response) = ui.allocate_exact_size(
                                Vec2::new(width, NAV_ITEM_HEIGHT),
                                Sense::click(),
                            );
                            slots[position] = rect;
                            let fill = if response.hovered() && !active {
                                ink(tone::faint())
                            } else {
                                Color32::TRANSPARENT
                            };
                            ui.painter().rect_filled(rect, NAV_ITEM_HEIGHT * 0.5, fill);
                            let color = if active || response.hovered() {
                                text_primary()
                            } else {
                                ink(tone::subtle())
                            };
                            let galley =
                                ui.painter()
                                    .layout_no_wrap(label.to_owned(), font.clone(), color);
                            let content = icon_size + (8.0 + galley.size().x) * label_t;
                            let left = rect.center().x - content * 0.5;
                            let filled = format!("{icon}Fill");
                            let icon_id = if active {
                                assets.icon(&filled).or_else(|| assets.icon(icon))
                            } else {
                                assets.icon(icon)
                            };
                            if let Some(icon_id) = icon_id {
                                ui.painter().image(
                                    icon_id,
                                    Rect::from_min_size(
                                        Pos2::new(left, rect.center().y - icon_size * 0.5),
                                        Vec2::splat(icon_size),
                                    ),
                                    full_uv(),
                                    color,
                                );
                            }
                            if label_alpha > 0.02 {
                                ui.painter().galley(
                                    Pos2::new(
                                        left + icon_size + 8.0,
                                        rect.center().y - galley.size().y * 0.5,
                                    ),
                                    galley,
                                    color.gamma_multiply(label_alpha),
                                );
                            }
                            if response.clicked() {
                                activated = Some(NAV_ITEM_NODES[*original]);
                            }
                        }
                        let (rect, response) = ui.allocate_exact_size(
                            Vec2::new(profile_width, NAV_ITEM_HEIGHT),
                            Sense::click(),
                        );
                        let profile_slot = entries.len();
                        slots[profile_slot] = rect;
                        let profile_active = active_route == 4;
                        if !profile_active && response.hovered() {
                            ui.painter().rect_filled(
                                rect,
                                NAV_ITEM_HEIGHT * 0.5,
                                ink(tone::faint()),
                            );
                        }
                        let target_slot = nav_slot(active_route);
                        let target = slots[target_slot];
                        let position = crate::motion::ease(
                            ui.ctx(),
                            Id::new("fluxa-top-bar-indicator-slot"),
                            target_slot as f32,
                            crate::motion::normal(),
                            crate::motion::Ease::Linear,
                        );
                        let last = slots.len() - 1;
                        let from = (position.floor().max(0.0) as usize).min(last);
                        let to = (from + 1).min(last);
                        let blend = position - from as f32;
                        let x = slots[from].center().x
                            + (slots[to].center().x - slots[from].center().x) * blend;
                        let width =
                            slots[from].width() + (slots[to].width() - slots[from].width()) * blend;
                        let stretch = (target.center().x - x).abs().min(target.width() * 0.5);
                        ui.painter().set(
                            pill,
                            glass_shape(
                                Rect::from_center_size(
                                    Pos2::new(x, target.center().y),
                                    Vec2::new(width + stretch, NAV_ITEM_HEIGHT),
                                ),
                                NAV_ITEM_HEIGHT * 0.5,
                                ink(tone::strong()),
                            ),
                        );
                        let center = Pos2::new(
                            rect.center().x
                                + (rect.left() + 8.0 + NAV_AVATAR_RADIUS - rect.center().x)
                                    * label_t,
                            rect.center().y,
                        );
                        if crate::components::dom_avatar(
                            ui.painter(),
                            assets,
                            center,
                            NAV_AVATAR_RADIUS,
                            profile_avatar_url,
                        ) {
                        } else if let Some(texture) = assets.texture_for(
                            profile_avatar_url,
                            [96, 96],
                            ArtworkPriority::Visible,
                        ) {
                            paint_circle_texture(ui.painter(), texture, center, NAV_AVATAR_RADIUS);
                            crate::components::animated_avatar(
                                ui.painter(),
                                assets,
                                center,
                                NAV_AVATAR_RADIUS,
                                NAV_AVATAR_RADIUS,
                                profile_avatar_url,
                            );
                        } else {
                            ui.painter().circle_filled(
                                center,
                                NAV_AVATAR_RADIUS,
                                ink(tone::raised()),
                            );
                        }
                        let label_left = center.x + NAV_AVATAR_RADIUS + 8.0;
                        if label_alpha > 0.02 {
                            draw_text(
                                ui.painter(),
                                Pos2::new(label_left, rect.center().y),
                                Align2::LEFT_CENTER,
                                truncate_to_width(
                                    ui.painter(),
                                    profile_name,
                                    &font,
                                    rect.right() - 12.0 - label_left,
                                ),
                                font.clone(),
                                if profile_active || response.hovered() {
                                    text_primary()
                                } else {
                                    ink(tone::body())
                                }
                                .gamma_multiply(label_alpha),
                                None,
                            );
                        }
                        if response.clicked() {
                            activated = Some(NODE_PROFILE);
                        }
                    });
                });
            ui.ctx()
                .data_mut(|data| data.insert_temp(bar_id, frame.response.rect));
            if liquid_glass() {
                ui.painter().set(
                    backdrop,
                    glass_shape(
                        frame.response.rect,
                        NAV_ITEM_HEIGHT * 0.5 + NAV_BAR_PADDING,
                        fill,
                    ),
                );
            }
        });
    activated
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

fn nav_nodes() -> Vec<u64> {
    nav_entries()
        .iter()
        .map(|(index, _)| NAV_ITEM_NODES[*index])
        .chain([NODE_PROFILE])
        .collect()
}

fn nav_slot(active_route: usize) -> usize {
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

fn mobile_nav_pending(active_x: f32) -> Option<f32> {
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

fn mobile_nav_drag() -> Option<f32> {
    let bits = NAV_DRAG.load(Ordering::Relaxed);
    (bits != u32::MAX).then(|| f32::from_bits(bits))
}

pub fn mobile_nav_rect(viewport: Viewport) -> Rect {
    let labels = NAV_LABELS.load(Ordering::Relaxed);
    let (slot, item) = if labels {
        (68.0, 58.0)
    } else {
        (52.0, NAV_ITEM_HEIGHT)
    };
    let width =
        (slot * nav_nodes().len() as f32 + NAV_BAR_PADDING * 2.0).min(viewport.width - 16.0);
    let height = item + NAV_BAR_PADDING * 2.0;
    let bottom = viewport.height - viewport.safe_bottom - NAV_BAR_TOP;
    Rect::from_min_size(
        Pos2::new((viewport.width - width) * 0.5, (bottom - height).max(0.0)),
        Vec2::new(width, height),
    )
}

pub fn mobile_nav_reserve(viewport: Viewport) -> f32 {
    viewport.height - mobile_nav_rect(viewport).top()
}

fn mobile_nav_index(viewport: Viewport, x: f32) -> (usize, f32, Rect) {
    let inner = mobile_nav_rect(viewport).shrink(NAV_BAR_PADDING);
    let count = nav_nodes().len();
    let slot = inner.width() / count as f32;
    let index = ((x - inner.left()) / slot)
        .floor()
        .clamp(0.0, count.saturating_sub(1) as f32) as usize;
    (index, slot, inner)
}

pub fn mobile_nav_node_at(viewport: Viewport, x: f32) -> u64 {
    nav_nodes()[mobile_nav_index(viewport, x).0]
}

pub fn mobile_nav_slot_center(viewport: Viewport, x: f32) -> f32 {
    let (index, slot, inner) = mobile_nav_index(viewport, x);
    inner.left() + slot * (index as f32 + 0.5)
}

pub(crate) fn draw_mobile_navigation_bar(
    context: &egui::Context,
    viewport: Viewport,
    active_route: usize,
    metrics: UiMetrics,
    assets: &mut impl HomeAssets,
    profile_avatar_url: Option<&str>,
    profile_name: &str,
) -> Option<u64> {
    let labels = NAV_LABELS.load(Ordering::Relaxed);
    let bar = mobile_nav_rect(viewport);
    let inner = bar.shrink(NAV_BAR_PADDING);
    let nodes = nav_nodes();
    let slot_width = inner.width() / nodes.len() as f32;
    let drag = mobile_nav_drag();
    let active_x = inner.left() + slot_width * (nav_slot(active_route) as f32 + 0.5);
    let target = drag
        .map(|x| {
            x.clamp(
                inner.left() + slot_width * 0.5,
                inner.right() - slot_width * 0.5,
            )
        })
        .or_else(|| mobile_nav_pending(active_x))
        .unwrap_or(active_x);
    let indicator_x = crate::motion::ease(
        context,
        Id::new("fluxa-bottom-bar-indicator"),
        target,
        crate::motion::normal(),
        crate::motion::Ease::Linear,
    );
    let stretch = (target - indicator_x).abs().min(slot_width * 0.6);
    let highlighted = drag
        .and_then(|x| {
            nodes
                .iter()
                .position(|node| *node == mobile_nav_node_at(viewport, x))
        })
        .unwrap_or_else(|| nav_slot(active_route));
    let icon_size = metrics.navigation_icon_size();
    let mut activated = None;
    egui::Area::new(Id::new("fluxa-shared-bottom-bar"))
        .fixed_pos(bar.min)
        .order(egui::Order::Tooltip)
        .show(context, |ui| {
            let painter = ui.painter().clone().with_clip_rect(Rect::EVERYTHING);
            let radius = bar.height() * 0.5;
            if liquid_glass() {
                glass(&painter, bar, radius, overlay_panel());
            } else {
                painter.rect_filled(bar, radius, overlay_panel());
                painter.rect_stroke(
                    bar,
                    radius,
                    egui::Stroke::new(stroke::hairline(), metrics.border),
                    egui::StrokeKind::Inside,
                );
            }
            painter.add(glass_shape(
                Rect::from_center_size(
                    Pos2::new(indicator_x, inner.center().y),
                    Vec2::new(slot_width + stretch, inner.height()),
                )
                .intersect(inner),
                inner.height() * 0.5,
                ink(tone::strong()),
            ));
            let items: Vec<(String, &str, u64)> = nav_entries()
                .into_iter()
                .map(|(original, label)| {
                    (
                        label,
                        ["NavHome", "NavSearch", "NavLibrary", "NavShorts"][original],
                        NAV_ITEM_NODES[original],
                    )
                })
                .chain([(profile_name.to_owned(), "", NODE_PROFILE)])
                .collect();
            for (index, (label, icon, node)) in items.into_iter().enumerate() {
                let rect = Rect::from_min_size(
                    Pos2::new(inner.left() + slot_width * index as f32, inner.top()),
                    Vec2::new(slot_width, inner.height()),
                );
                let response = ui.interact(
                    rect,
                    Id::new(("fluxa-bottom-bar-item", node)),
                    Sense::click(),
                );
                let active = index == highlighted;
                let color = if active {
                    text_primary()
                } else {
                    ink(tone::subtle())
                };
                let center = if labels {
                    rect.center() - Vec2::new(0.0, 8.0)
                } else {
                    rect.center()
                };
                if node == NODE_PROFILE {
                    if crate::components::dom_avatar(
                        ui.painter(),
                        assets,
                        center,
                        NAV_AVATAR_RADIUS,
                        profile_avatar_url,
                    ) {
                    } else if let Some(texture) =
                        assets.texture_for(profile_avatar_url, [96, 96], ArtworkPriority::Visible)
                    {
                        paint_circle_texture(ui.painter(), texture, center, NAV_AVATAR_RADIUS);
                        crate::components::animated_avatar(
                            ui.painter(),
                            assets,
                            center,
                            NAV_AVATAR_RADIUS,
                            NAV_AVATAR_RADIUS,
                            profile_avatar_url,
                        );
                    } else {
                        ui.painter()
                            .circle_filled(center, NAV_AVATAR_RADIUS, ink(tone::raised()));
                    }
                } else {
                    let filled = format!("{icon}Fill");
                    let texture = if active {
                        assets.icon(&filled).or_else(|| assets.icon(icon))
                    } else {
                        assets.icon(icon)
                    };
                    if let Some(texture) = texture {
                        ui.painter().image(
                            texture,
                            Rect::from_center_size(center, Vec2::splat(icon_size)),
                            full_uv(),
                            color,
                        );
                    }
                }
                if labels {
                    let galley = ui.painter().layout_job(egui::text::LayoutJob {
                        wrap: egui::text::TextWrapping::truncate_at_width(slot_width - 6.0),
                        ..egui::text::LayoutJob::simple_singleline(
                            label,
                            FontId::proportional(metrics.text.label - 1.0),
                            color,
                        )
                    });
                    let width = galley.size().x;
                    ui.painter().galley(
                        Pos2::new(
                            rect.center().x - width * 0.5,
                            rect.center().y + 17.0 - galley.size().y * 0.5,
                        ),
                        galley,
                        color,
                    );
                }
                if response.clicked() {
                    activated = Some(node);
                }
            }
        });
    if drag.is_some() || stretch > 0.5 {
        context.request_repaint();
    }
    activated
}

pub fn navigation_focus_rects(viewport: Viewport, metrics: UiMetrics) -> Vec<(u64, Rect)> {
    if viewport.side_inset > 0.0 {
        return crate::sidebar::item_rects(viewport);
    }
    if viewport.is_compact() {
        let bar = mobile_nav_rect(viewport).shrink(NAV_BAR_PADDING);
        let nodes = nav_nodes();
        let slot = bar.width() / nodes.len() as f32;
        nodes
            .into_iter()
            .enumerate()
            .map(|(index, node)| {
                (
                    node,
                    Rect::from_min_size(
                        Pos2::new(bar.left() + slot * index as f32, bar.top()),
                        Vec2::new(slot, bar.height()),
                    ),
                )
            })
            .collect()
    } else {
        let entries = nav_entries();
        let profile = drawn_profile();
        let widths: Vec<f32> = entries
            .iter()
            .map(|(_, label)| metrics.navigation_item_width(label))
            .chain([metrics.navigation_profile_width(&profile)])
            .collect();
        let nodes = entries
            .iter()
            .map(|(original, _)| NAV_ITEM_NODES[*original])
            .chain([NODE_PROFILE]);
        let bar_width = metrics.navigation_bar_width(viewport, &profile) + NAV_BAR_PADDING * 2.0;
        let x = (viewport.width - bar_width).max(12.0) * 0.5 + NAV_BAR_PADDING;
        let y = NAV_BAR_TOP + viewport.nav_lead + NAV_BAR_PADDING;
        let bar = Rect::from_min_size(Pos2::new(x, y), Vec2::new(bar_width, NAV_ITEM_HEIGHT));
        let slots = crate::flex::row(widths.iter().map(|width| crate::flex::slot(*width)))
            .gap(NAV_ITEM_GAP)
            .align(crate::flex::Align::Stretch)
            .solve(bar)
            .children
            .into_iter()
            .map(|slot| slot.rect);
        nodes.zip(slots).collect()
    }
}

pub(crate) fn is_navigation_node(node: u64) -> bool {
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
    )
}
