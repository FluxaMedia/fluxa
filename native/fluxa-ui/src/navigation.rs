use super::*;
use crate::components::{glass, glass_shape, liquid_glass};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

pub(crate) const NAV_BAR_TOP: f32 = 14.0;
pub(crate) const NAV_ITEM_HEIGHT: f32 = 44.0;
pub(crate) const NAV_BAR_PADDING: f32 = 6.0;
pub(crate) const NAV_ITEM_GAP: f32 = 2.0;
pub(crate) const NAV_AVATAR_RADIUS: f32 = 13.5;

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
    assets: &impl HomeAssets,
) -> Option<u64> {
    draw_navigation_bar_with_profile(
        context,
        viewport,
        active_route,
        assets,
        assets.active_profile_avatar_url(),
        assets.active_profile_name().unwrap_or("Profile"),
    )
}

pub(crate) fn draw_navigation_bar_with_profile(
    context: &egui::Context,
    viewport: Viewport,
    active_route: usize,
    assets: &impl HomeAssets,
    profile_avatar_url: Option<&str>,
    profile_name: &str,
) -> Option<u64> {
    let metrics = metrics_for_assets(viewport, assets);
    let compact = viewport.is_compact();
    let tv = viewport.is_tv();
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
    let label_size = metrics.navigation_label_size(tv);
    let icon_size = metrics.navigation_icon_size(tv);
    let profile_width = metrics.navigation_profile_width(profile_name, tv);
    let bar_width = metrics.navigation_bar_width(viewport, profile_name) + NAV_BAR_PADDING * 2.0;
    let bar_x = (viewport.width - bar_width).max(12.0) * 0.5;
    let font = FontId::proportional(label_size);
    let mut activated = None;
    egui::Area::new(Id::new("fluxa-shared-top-bar"))
        .fixed_pos(Pos2::new(bar_x, NAV_BAR_TOP))
        .order(egui::Order::Foreground)
        .show(context, |ui| {
            let fill = Color32::from_rgba_unmultiplied(22, 22, 24, 242);
            let backdrop = ui.painter().add(egui::Shape::Noop);
            let frame = egui::Frame::NONE
                .fill(if liquid_glass() {
                    Color32::TRANSPARENT
                } else {
                    fill
                })
                .stroke(egui::Stroke::new(1.0, Color32::from_white_alpha(20)))
                .corner_radius(NAV_ITEM_HEIGHT * 0.5 + NAV_BAR_PADDING)
                .inner_margin(egui::Margin::same(NAV_BAR_PADDING as i8))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = NAV_ITEM_GAP;
                        let pill = ui.painter().add(egui::Shape::Noop);
                        let mut slots = [Rect::NOTHING; 4];
                        let labels = nav_labels();
                        for (index, (label, icon)) in labels
                            .iter()
                            .zip(["Home", "Discover", "Library"])
                            .enumerate()
                        {
                            let active = index == active_route;
                            let width = metrics.navigation_item_width(label, tv);
                            let (rect, response) = ui.allocate_exact_size(
                                Vec2::new(width, NAV_ITEM_HEIGHT),
                                Sense::click(),
                            );
                            slots[index] = rect;
                            let fill = if response.hovered() && !active {
                                Color32::from_white_alpha(12)
                            } else {
                                Color32::TRANSPARENT
                            };
                            ui.painter().rect_filled(rect, NAV_ITEM_HEIGHT * 0.5, fill);
                            let color = if active || response.hovered() {
                                Color32::WHITE
                            } else {
                                Color32::from_white_alpha(160)
                            };
                            let galley =
                                ui.painter()
                                    .layout_no_wrap(label.to_owned(), font.clone(), color);
                            let content = icon_size + 8.0 + galley.size().x;
                            let left = rect.center().x - content * 0.5;
                            if let Some(icon_id) = assets.icon(icon) {
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
                            ui.painter().galley(
                                Pos2::new(
                                    left + icon_size + 8.0,
                                    rect.center().y - galley.size().y * 0.5,
                                ),
                                galley,
                                color,
                            );
                            if response.clicked() {
                                activated = Some([NODE_HOME, NODE_DISCOVER, NODE_LIBRARY][index]);
                            }
                        }
                        let (rect, response) = ui.allocate_exact_size(
                            Vec2::new(profile_width, NAV_ITEM_HEIGHT),
                            Sense::click(),
                        );
                        slots[3] = rect;
                        let profile_active = active_route == 3;
                        if !profile_active && response.hovered() {
                            ui.painter().rect_filled(
                                rect,
                                NAV_ITEM_HEIGHT * 0.5,
                                Color32::from_white_alpha(12),
                            );
                        }
                        let target = slots[active_route.min(3)];
                        let x = ui.ctx().animate_value_with_time(
                            Id::new("fluxa-top-bar-indicator-x"),
                            target.center().x,
                            0.22,
                        );
                        let width = ui.ctx().animate_value_with_time(
                            Id::new("fluxa-top-bar-indicator-w"),
                            target.width(),
                            0.22,
                        );
                        let stretch = (target.center().x - x).abs().min(target.width() * 0.5);
                        ui.painter().set(
                            pill,
                            glass_shape(
                                Rect::from_center_size(
                                    Pos2::new(x, target.center().y),
                                    Vec2::new(width + stretch, NAV_ITEM_HEIGHT),
                                ),
                                NAV_ITEM_HEIGHT * 0.5,
                                Color32::from_white_alpha(18),
                            ),
                        );
                        let center = rect.left_center() + Vec2::new(8.0 + NAV_AVATAR_RADIUS, 0.0);
                        if let Some(texture) = assets.cached_texture(profile_avatar_url) {
                            paint_circle_texture(ui.painter(), texture, center, NAV_AVATAR_RADIUS);
                        } else {
                            ui.painter().circle_filled(
                                center,
                                NAV_AVATAR_RADIUS,
                                Color32::from_white_alpha(30),
                            );
                            if let Some(icon) = assets.icon("Account") {
                                ui.painter().image(
                                    icon,
                                    Rect::from_center_size(center, Vec2::splat(16.0)),
                                    full_uv(),
                                    Color32::from_white_alpha(215),
                                );
                            }
                        }
                        let label_left = center.x + NAV_AVATAR_RADIUS + 8.0;
                        ui.painter().text(
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
                                Color32::WHITE
                            } else {
                                Color32::from_white_alpha(200)
                            },
                        );
                        if response.clicked() {
                            activated = Some(NODE_PROFILE);
                        }
                    });
                });
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

static NAV_FLOATING: AtomicBool = AtomicBool::new(true);
static NAV_LABELS: AtomicBool = AtomicBool::new(true);
static NAV_LANGUAGE: std::sync::RwLock<String> = std::sync::RwLock::new(String::new());
static NAV_DRAG: AtomicU32 = AtomicU32::new(u32::MAX);

pub fn set_mobile_nav_style(floating: bool, labels: bool) {
    NAV_FLOATING.store(floating, Ordering::Relaxed);
    NAV_LABELS.store(labels, Ordering::Relaxed);
}

pub fn set_nav_language(language: &str) {
    if let Ok(mut current) = NAV_LANGUAGE.write()
        && *current != language
    {
        *current = language.to_owned();
    }
}

pub(crate) fn nav_labels() -> [String; 3] {
    let language = NAV_LANGUAGE.read().map(|l| l.clone()).unwrap_or_default();
    ["nav.home", "nav.discover", "nav.library"].map(|key| localized(key, &language))
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

fn nav_accent(assets: &impl HomeAssets) -> (Color32, Color32) {
    match assets.accent_color() {
        Some(accent) if !accent_is_neutral(assets) => {
            let [r, g, b, _] = accent.to_array();
            (accent, Color32::from_rgba_unmultiplied(r, g, b, 46))
        }
        _ => (Color32::WHITE, Color32::from_white_alpha(18)),
    }
}

const MOBILE_NODES: [u64; 4] = [NODE_HOME, NODE_DISCOVER, NODE_LIBRARY, NODE_PROFILE];

pub fn mobile_nav_rect(viewport: Viewport) -> Rect {
    let height = 64.0;
    if NAV_FLOATING.load(Ordering::Relaxed) {
        let width = (viewport.width - 20.0).min(560.0);
        let bottom = viewport.height - viewport.safe_bottom - 10.0;
        Rect::from_min_size(
            Pos2::new((viewport.width - width) * 0.5, (bottom - height).max(0.0)),
            Vec2::new(width, height),
        )
    } else {
        let y = (viewport.height - viewport.safe_bottom - height).max(0.0);
        Rect::from_min_size(Pos2::new(8.0, y), Vec2::new(viewport.width - 16.0, height))
    }
}

pub fn mobile_nav_reserve(viewport: Viewport) -> f32 {
    viewport.height - mobile_nav_rect(viewport).top()
}

pub fn mobile_nav_node_at(viewport: Viewport, x: f32) -> u64 {
    let rect = mobile_nav_rect(viewport);
    let slot = rect.width() / 4.0;
    let index = ((x - rect.left()) / slot).floor().clamp(0.0, 3.0) as usize;
    MOBILE_NODES[index]
}

pub(crate) fn draw_mobile_navigation_bar(
    context: &egui::Context,
    viewport: Viewport,
    active_route: usize,
    metrics: UiMetrics,
    assets: &impl HomeAssets,
    profile_avatar_url: Option<&str>,
    profile_name: &str,
) -> Option<u64> {
    let floating = NAV_FLOATING.load(Ordering::Relaxed);
    let labels = NAV_LABELS.load(Ordering::Relaxed);
    let bar = mobile_nav_rect(viewport);
    let slot_width = bar.width() / 4.0;
    let (accent, indicator_fill) = nav_accent(assets);
    let [r, g, b, _] = metrics.background.to_array();
    let solid = Color32::from_rgb(r, g, b);
    let drag = mobile_nav_drag();
    let active_x = bar.left() + slot_width * (active_route.min(3) as f32 + 0.5);
    let target = drag
        .map(|x| {
            x.clamp(
                bar.left() + slot_width * 0.5,
                bar.right() - slot_width * 0.5,
            )
        })
        .or_else(|| mobile_nav_pending(active_x))
        .unwrap_or(active_x);
    let indicator_x =
        context.animate_value_with_time(Id::new("fluxa-bottom-bar-indicator"), target, 0.22);
    let stretch = (target - indicator_x).abs().min(slot_width * 0.6);
    let highlighted = drag
        .map(|x| {
            MOBILE_NODES
                .iter()
                .position(|node| *node == mobile_nav_node_at(viewport, x))
        })
        .flatten()
        .unwrap_or(active_route);
    let mut activated = None;
    egui::Area::new(Id::new("fluxa-shared-bottom-bar"))
        .fixed_pos(bar.min)
        .order(egui::Order::Tooltip)
        .show(context, |ui| {
            let painter = ui.painter().clone().with_clip_rect(Rect::EVERYTHING);
            if floating && liquid_glass() {
                glass(
                    &painter,
                    bar,
                    bar.height() * 0.5,
                    Color32::from_rgba_unmultiplied(22, 22, 24, 242),
                );
            } else if floating {
                paint_vertical_gradient(
                    &painter,
                    Rect::from_min_max(
                        Pos2::new(0.0, bar.top() - 64.0),
                        Pos2::new(viewport.width, bar.top() + 4.0),
                    ),
                    Color32::TRANSPARENT,
                    solid,
                );
                painter.rect_filled(
                    Rect::from_min_max(
                        Pos2::new(0.0, bar.top() + 4.0),
                        Pos2::new(viewport.width, viewport.height),
                    ),
                    0.0,
                    solid,
                );
                painter.rect_filled(
                    bar,
                    bar.height() * 0.5,
                    Color32::from_rgba_unmultiplied(22, 22, 24, 242),
                );
                painter.rect_stroke(
                    bar,
                    bar.height() * 0.5,
                    egui::Stroke::new(1.0, Color32::from_white_alpha(20)),
                    egui::StrokeKind::Inside,
                );
            } else {
                paint_vertical_gradient(
                    &painter,
                    Rect::from_min_max(
                        Pos2::new(0.0, bar.top() - 48.0),
                        Pos2::new(viewport.width, bar.top() + 8.0),
                    ),
                    Color32::TRANSPARENT,
                    solid,
                );
                painter.rect_filled(
                    Rect::from_min_max(
                        Pos2::new(0.0, bar.top() + 8.0),
                        Pos2::new(viewport.width, viewport.height),
                    ),
                    0.0,
                    solid,
                );
            }
            let indicator_height = bar.height() - 8.0;
            let indicator = Rect::from_center_size(
                Pos2::new(indicator_x, bar.center().y),
                Vec2::new(
                    slot_width * 1.2 + stretch,
                    indicator_height - stretch * 0.12,
                ),
            )
            .intersect(bar.shrink(4.0));
            if floating {
                glass(&painter, indicator, indicator.height() * 0.5, indicator_fill);
            } else {
                painter.rect_filled(indicator, indicator.height() * 0.5, indicator_fill);
            }
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                let [home, discover, library] = nav_labels();
                for (index, (label, icon, node)) in [
                    (home.as_str(), "Home", NODE_HOME),
                    (discover.as_str(), "Discover", NODE_DISCOVER),
                    (library.as_str(), "Library", NODE_LIBRARY),
                    (profile_name, "Account", NODE_PROFILE),
                ]
                .into_iter()
                .enumerate()
                {
                    let active = index == highlighted;
                    let (rect, response) =
                        ui.allocate_exact_size(Vec2::new(slot_width, bar.height()), Sense::click());
                    let color = if active {
                        accent
                    } else {
                        Color32::from_white_alpha(130)
                    };
                    let center = if labels {
                        rect.center() - Vec2::new(0.0, 9.0)
                    } else {
                        rect.center()
                    };
                    let avatar = (index == 3)
                        .then(|| assets.cached_texture(profile_avatar_url))
                        .flatten();
                    if let Some(avatar) = avatar {
                        paint_circle_texture(ui.painter(), avatar, center, 13.0);
                        if active {
                            ui.painter().circle_stroke(
                                center,
                                15.0,
                                egui::Stroke::new(1.5, accent),
                            );
                        }
                    } else if let Some(icon) = assets.icon(icon) {
                        ui.painter().image(
                            icon,
                            Rect::from_center_size(center, Vec2::splat(26.0)),
                            full_uv(),
                            color,
                        );
                    }
                    if labels {
                        let galley = ui.painter().layout_job(egui::text::LayoutJob {
                            wrap: egui::text::TextWrapping::truncate_at_width(slot_width - 8.0),
                            ..egui::text::LayoutJob::simple_singleline(
                                label.to_owned(),
                                FontId::proportional(metrics.text.label - 1.0),
                                color,
                            )
                        });
                        let width = galley.size().x;
                        ui.painter().galley(
                            Pos2::new(
                                rect.center().x - width * 0.5,
                                rect.center().y + 15.0 - galley.size().y * 0.5,
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
        });
    if drag.is_some() || stretch > 0.5 {
        context.request_repaint();
    }
    activated
}

pub fn navigation_focus_rects(viewport: Viewport, metrics: UiMetrics) -> Vec<(u64, Rect)> {
    if viewport.is_compact() {
        let bar = mobile_nav_rect(viewport);
        let slot = bar.width() / 4.0;
        MOBILE_NODES
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
        let tv = viewport.is_tv();
        let widths = nav_labels().map(|label| metrics.navigation_item_width(&label, tv));
        let bar_width = metrics.navigation_bar_width(viewport, "Profile") + NAV_BAR_PADDING * 2.0;
        let x = (viewport.width - bar_width).max(12.0) * 0.5 + NAV_BAR_PADDING;
        let y = NAV_BAR_TOP + NAV_BAR_PADDING;
        let mut left = x;
        let mut rects = Vec::with_capacity(4);
        for (index, node) in [NODE_HOME, NODE_DISCOVER, NODE_LIBRARY]
            .into_iter()
            .enumerate()
        {
            let item_width = widths[index];
            rects.push((
                node,
                Rect::from_min_size(Pos2::new(left, y), Vec2::new(item_width, NAV_ITEM_HEIGHT)),
            ));
            left += item_width + NAV_ITEM_GAP;
        }
        let profile_width = metrics.navigation_profile_width("Profile", tv);
        rects.push((
            NODE_PROFILE,
            Rect::from_min_size(
                Pos2::new(left, y),
                Vec2::new(profile_width, NAV_ITEM_HEIGHT),
            ),
        ));
        rects
    }
}

pub(crate) fn is_navigation_node(node: u64) -> bool {
    matches!(
        node,
        NODE_HOME | NODE_LIBRARY | NODE_DISCOVER | NODE_PROFILE
    )
}
