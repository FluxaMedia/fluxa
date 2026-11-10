use super::*;
use crate::components::{glass, glass_shape, liquid_glass};
use crate::dp::Dp;
use crate::flex::Insets;
use crate::navigation::{
    drawn_profile, labels_below, mobile_nav_drag, mobile_nav_pending, mobile_nav_rect,
    nav_avatar_radius, nav_bar_padding, nav_bar_top, nav_entries, nav_item_height, nav_nodes,
    nav_pill_gap, nav_slot, set_mobile_nav_pending, set_top_label_t, top_label_t, top_labels_mode,
};
use std::sync::atomic::{AtomicU32, Ordering};

static MOUSE_DRAG: AtomicU32 = AtomicU32::new(u32::MAX);

pub(crate) struct Slot {
    pub node: u64,
    pub rect: Rect,
}

pub(crate) struct Bar {
    pub rect: Rect,
    pub slots: Vec<Slot>,
    pub badge: Option<Rect>,
}

fn avatar_center(viewport: Viewport, slot: Rect) -> Pos2 {
    let near = Space::S8.px(&crate::metrics::active());
    if viewport.is_compact() {
        if labels_below() {
            slot.center() - Vec2::new(0.0, near)
        } else {
            slot.center()
        }
    } else {
        let shift = (slot.left() + near + nav_avatar_radius() - slot.center().x) * top_label_t();
        Pos2::new(slot.center().x + shift, slot.center().y)
    }
}

pub(crate) fn layout(viewport: Viewport, metrics: UiMetrics, profile_name: &str) -> Bar {
    let nodes = nav_nodes();
    let pad = Insets::all(nav_bar_padding());
    let (rect, placed) = if viewport.is_compact() {
        let rect = mobile_nav_rect(viewport);
        let cells = (0..nodes.len()).map(|index| crate::flex::fill().id(index));
        (
            rect,
            crate::flex::row(cells)
                .padding(pad)
                .align(crate::flex::Align::Start)
                .solve(rect),
        )
    } else {
        let cells: Vec<crate::flex::Node> = nav_entries()
            .iter()
            .map(|(_, label)| metrics.navigation_item_width(label))
            .chain([metrics.navigation_profile_width(profile_name)])
            .enumerate()
            .map(|(index, width)| crate::flex::leaf(Vec2::new(width, nav_item_height())).id(index))
            .collect();
        let row = crate::flex::row(cells)
            .gap(nav_pill_gap())
            .padding(pad)
            .align(crate::flex::Align::Start);
        let size = row.outer_size();
        let margin = Space::S12.px(&metrics);
        let lane = crate::gfx::rect_at(
            Pos2::new(0.0, nav_bar_top() + viewport.nav_lead),
            Vec2::new(viewport.width.max(size.x + margin), size.y),
        );
        let rect = crate::flex::anchored(lane, Align2::CENTER_TOP, size, 0.0);
        (rect, row.solve(rect))
    };
    let boxes = (0..nodes.len()).map(|index| placed.find(index).unwrap_or(rect));
    let slots: Vec<Slot> = nodes
        .iter()
        .zip(boxes)
        .map(|(&node, rect)| Slot { node, rect })
        .collect();
    let badge = crate::update::nav_status().and_then(|_| {
        let profile: &Slot = slots.last()?;
        Some(crate::update::badge_rect(
            avatar_center(viewport, profile.rect),
            nav_avatar_radius(),
        ))
    });
    Bar { rect, slots, badge }
}

pub(crate) fn node_at(bar: &Bar, position: Pos2) -> u64 {
    if let Some(badge) = bar.badge
        && badge.contains(position)
    {
        return NODE_UPDATE_OPEN;
    }
    nearest(bar, position.x).node
}

fn nearest(bar: &Bar, x: f32) -> &Slot {
    bar.slots
        .iter()
        .find(|slot| x < slot.rect.right() + nav_pill_gap() * 0.5)
        .unwrap_or_else(|| &bar.slots[bar.slots.len() - 1])
}

pub(crate) fn slot_center(bar: &Bar, x: f32) -> f32 {
    nearest(bar, x).rect.center().x
}

fn width_at(slots: &[Slot], x: f32) -> f32 {
    let first = &slots[0];
    if x <= first.rect.center().x {
        return first.rect.width();
    }
    for pair in slots.windows(2) {
        let (a, b) = (pair[0].rect, pair[1].rect);
        if x <= b.center().x {
            let t = (x - a.center().x) / (b.center().x - a.center().x);
            return a.width() + (b.width() - a.width()) * t;
        }
    }
    slots[slots.len() - 1].rect.width()
}

fn lens(x: f32, slot: Rect, span: f32) -> f32 {
    let t = 1.0 - ((slot.center().x - x).abs() / (slot.width() * span)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn draw_avatar(
    ui: &crate::gfx::Ui,
    assets: &mut impl HomeAssets,
    center: Pos2,
    radius: f32,
    url: Option<&str>,
) {
    if crate::components::dom_avatar(ui.painter(), assets, center, radius, url) {
    } else if let Some(texture) = assets.texture_for(url, [96, 96], ArtworkPriority::Visible) {
        paint_circle_texture(ui.painter(), texture, center, radius);
        crate::components::animated_avatar(ui.painter(), assets, center, radius, radius, url);
    } else {
        ui.painter()
            .circle_filled(center, radius, crate::stylesheet::color("raised"));
    }
}

pub(crate) fn draw(
    context: &crate::gfx::Context,
    viewport: Viewport,
    active_route: usize,
    metrics: UiMetrics,
    assets: &mut impl HomeAssets,
    profile_avatar_url: Option<&str>,
    profile_name: &str,
) -> Option<u64> {
    let compact = viewport.is_compact();
    let below = compact && labels_below();
    let bar_id = Id::new("fluxa-top-bar-rect");
    if !compact {
        let hovered = context
            .data(|data| data.get_temp::<Rect>(bar_id))
            .zip(context.pointer_hover_pos())
            .is_some_and(|(rect, pos)| rect.expand(Space::S6.px(&metrics)).contains(pos));
        let target = match top_labels_mode() {
            1 => 1.0,
            2 => 0.0,
            _ => f32::from(hovered || crate::sidebar::nav_focused()),
        };
        let label_t = crate::motion::ease(
            context,
            Id::new("fluxa-top-bar-labels"),
            target,
            crate::motion::normal(),
            crate::motion::Ease::Out,
        );
        set_top_label_t(label_t);
    }
    let label_t = if compact { 0.0 } else { top_label_t() };
    let label_alpha = ((label_t - 0.35) / 0.65).clamp(0.0, 1.0);
    let bar = layout(viewport, metrics, profile_name);
    context.data_mut(|data| data.insert_temp(bar_id, bar.rect));
    let status = crate::update::nav_status();
    let icon_size = metrics.navigation_icon_size();
    let label_size = metrics.navigation_label_size();
    let font = FontId::proportional(label_size);
    let items: Vec<(String, &str)> = nav_entries()
        .into_iter()
        .map(|(original, label)| {
            (
                label,
                ["NavHome", "NavSearch", "NavLibrary", "NavShorts"][original],
            )
        })
        .chain([(profile_name.to_owned(), "")])
        .collect();
    let first = &bar.slots[0].rect;
    let inner = Rect::from_min_max(
        first.left_top(),
        bar.slots[bar.slots.len() - 1].rect.right_bottom(),
    );
    let mut activated = None;
    let area = if compact {
        ("fluxa-shared-bottom-bar", crate::gfx::Order::Tooltip)
    } else {
        ("fluxa-shared-top-bar", crate::gfx::Order::Foreground)
    };
    let mut moving = false;
    crate::gfx::Area::new(Id::new(area.0))
        .fixed_pos(bar.rect.min)
        .order(area.1)
        .show(context, |ui| {
            let response = ui.interact(
                bar.rect,
                Id::new("fluxa-nav-bar-input"),
                Sense::click_and_drag(),
            );
            let pointer = response.interact_pointer_pos();
            let mouse = if response.dragged() {
                pointer.map(|pos| pos.x)
            } else {
                None
            };
            MOUSE_DRAG.store(mouse.map_or(u32::MAX, f32::to_bits), Ordering::Relaxed);
            if response.drag_stopped() {
                let at = pointer
                    .or(ui.ctx().pointer_latest_pos())
                    .unwrap_or_default();
                set_mobile_nav_pending(slot_center(&bar, at.x));
                activated = Some(node_at(&bar, at));
            } else if response.clicked() {
                let at = pointer.unwrap_or_default();
                set_mobile_nav_pending(slot_center(&bar, at.x));
                activated = Some(node_at(&bar, at));
            }
            let hover = if mouse.is_some() {
                None
            } else {
                ui.ctx().pointer_hover_pos()
            };

            let painter = ui.painter().clone().with_clip_rect(Rect::EVERYTHING);
            let radius = bar.rect.height() * 0.5;
            if liquid_glass() {
                glass(&painter, bar.rect, radius, overlay_panel());
            } else {
                painter.rect_filled(bar.rect, radius, overlay_panel());
                painter.rect_stroke(
                    bar.rect,
                    radius,
                    crate::gfx::Stroke::new(stroke::hairline(), metrics.border),
                    crate::gfx::StrokeKind::Inside,
                );
            }

            let active_slot = nav_slot(active_route);
            let active_x = bar.slots[active_slot.min(bar.slots.len() - 1)]
                .rect
                .center()
                .x;
            let drag = mouse.or_else(mobile_nav_drag);
            let target = drag
                .map(|x| {
                    x.clamp(
                        bar.slots[0].rect.center().x,
                        bar.slots[bar.slots.len() - 1].rect.center().x,
                    )
                })
                .or_else(|| mobile_nav_pending(active_x))
                .unwrap_or(active_x);
            let x = crate::motion::ease(
                ui.ctx(),
                Id::new("fluxa-nav-indicator-x"),
                target,
                crate::motion::normal(),
                crate::motion::Ease::Linear,
            );
            let width = width_at(&bar.slots, x);
            let reach = width * metrics.screen_nav_stretch;
            let stretch = (target - x).abs().min(reach);
            let squash = (stretch / reach).clamp(0.0, 1.0);
            moving = drag.is_some() || stretch > 0.5;
            let pill = crate::gfx::rect_around(
                Pos2::new(x, inner.center().y),
                Vec2::new(
                    width + stretch,
                    inner.height() * (1.0 - metrics.screen_nav_jelly * squash),
                ),
            );
            painter.add(glass_shape(
                pill.intersect(inner),
                pill.height() * 0.5,
                crate::stylesheet::color("strong"),
            ));

            let highlighted = drag
                .map(|at| {
                    bar.slots
                        .iter()
                        .position(|slot| slot.node == nearest(&bar, at).node)
                        .unwrap_or(active_slot)
                })
                .unwrap_or(active_slot);
            for (index, (slot, (label, icon))) in bar.slots.iter().zip(&items).enumerate() {
                let rect = slot.rect;
                let active = index == highlighted;
                let hovered = hover.is_some_and(|pos| rect.contains(pos));
                if hovered && !active && !compact {
                    painter.rect_filled(
                        rect,
                        nav_item_height() * 0.5,
                        crate::stylesheet::color("well"),
                    );
                }
                let color = if active || hovered {
                    text_primary()
                } else {
                    crate::stylesheet::color("text.subtle")
                };
                let scale =
                    1.0 + metrics.screen_nav_magnify * lens(x, rect, metrics.screen_nav_lens_span);
                let galley = if below {
                    painter.layout_job(crate::gfx::text::LayoutJob {
                        wrap: crate::gfx::text::TextWrapping::truncate_at_width(
                            rect.width() - Space::S6.px(&metrics),
                        ),
                        ..crate::gfx::text::LayoutJob::simple_singleline(
                            label.clone(),
                            FontId::proportional((metrics.text.label - 1.0) * scale),
                            color,
                        )
                    })
                } else {
                    painter.layout_no_wrap(
                        label.clone(),
                        FontId::proportional(label_size * scale),
                        color,
                    )
                };
                let icon_px = icon_size * scale;
                let beside = !compact;
                let near = Space::S8.px(&metrics);
                let reveal = (near + galley.size().x) * label_t;
                let icon_rect = crate::flex::row([
                    crate::flex::leaf(Vec2::splat(icon_px)).width(icon_px).id(0),
                    crate::flex::leaf(Vec2::ZERO).width(reveal),
                ])
                .justify(crate::flex::Justify::Center)
                .align(crate::flex::Align::Center)
                .solve(rect)
                .find(0)
                .unwrap_or(rect);
                let icon_center = if beside && slot.node != NODE_PROFILE {
                    icon_rect.center()
                } else if below {
                    rect.center() - Vec2::new(0.0, near)
                } else {
                    rect.center()
                };
                if slot.node == NODE_PROFILE {
                    let center = avatar_center(viewport, rect);
                    draw_avatar(
                        ui,
                        assets,
                        center,
                        nav_avatar_radius() * scale,
                        profile_avatar_url,
                    );
                    if let Some(status) = &status {
                        crate::update::draw_badge(
                            &painter,
                            &*assets,
                            status,
                            center,
                            nav_avatar_radius(),
                        );
                    }
                    if beside && label_alpha > 0.02 {
                        let left = center.x + nav_avatar_radius() * scale + near;
                        let name = crate::flex::row([crate::flex::fill().id(0)])
                            .padding(Insets {
                                left: left - rect.left(),
                                top: 0.0,
                                right: Space::S12.px(&metrics),
                                bottom: 0.0,
                            })
                            .align(crate::flex::Align::Start)
                            .solve(rect)
                            .find(0)
                            .unwrap_or(rect);
                        crate::flex::paint_text(
                            &painter,
                            name,
                            label,
                            &font,
                            color.gamma_multiply(label_alpha),
                            Align2::LEFT_CENTER,
                        );
                    }
                } else {
                    let filled = format!("{icon}Fill");
                    let texture = if active {
                        assets.icon(&filled).or_else(|| assets.icon(icon))
                    } else {
                        assets.icon(icon)
                    };
                    if let Some(texture) = texture {
                        painter.image(
                            texture,
                            crate::gfx::square_around(icon_center, icon_px),
                            full_uv(),
                            color,
                        );
                    }
                    if beside && label_alpha > 0.02 {
                        let text = crate::flex::row([
                            crate::flex::leaf(Vec2::new(icon_px + near, 0.0)),
                            crate::flex::leaf(galley.size()).id(0),
                        ])
                        .align(crate::flex::Align::Center)
                        .solve(Rect::from_min_max(
                            Pos2::new(icon_rect.left(), rect.top()),
                            rect.right_bottom(),
                        ))
                        .find(0)
                        .unwrap_or(rect);
                        painter.galley(text.min, galley.clone(), color.gamma_multiply(label_alpha));
                    }
                }
                if below {
                    let text =
                        crate::flex::anchored(rect, Align2::CENTER_CENTER, galley.size(), 0.0)
                            .translate(Vec2::new(0.0, Dp::D16.px()));
                    painter.galley(text.min, galley, color);
                }
            }
        });
    if moving {
        context.request_repaint();
    }
    activated
}

pub(crate) fn focus_rects(viewport: Viewport, metrics: UiMetrics) -> Vec<(u64, Rect)> {
    let bar = layout(viewport, metrics, &drawn_profile());
    let mut rects: Vec<(u64, Rect)> = bar
        .slots
        .iter()
        .map(|slot| (slot.node, slot.rect))
        .collect();
    if let Some(badge) = bar.badge {
        rects.push((NODE_UPDATE_OPEN, badge));
    }
    rects
}

pub fn nav_bar_rect(viewport: Viewport) -> Option<Rect> {
    if viewport.side_inset > 0.0 {
        return None;
    }
    let metrics = UiMetrics::for_viewport(viewport);
    Some(layout(viewport, metrics, &drawn_profile()).rect)
}

pub fn nav_node_at(viewport: Viewport, position: Pos2) -> u64 {
    let metrics = UiMetrics::for_viewport(viewport);
    node_at(&layout(viewport, metrics, &drawn_profile()), position)
}

pub fn nav_slot_center(viewport: Viewport, x: f32) -> f32 {
    let metrics = UiMetrics::for_viewport(viewport);
    slot_center(&layout(viewport, metrics, &drawn_profile()), x)
}
