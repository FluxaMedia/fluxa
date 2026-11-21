use super::*;
use crate::flex::Insets;
use crate::navigation::{
    drawn_profile, nav_avatar_radius, nav_bar_padding, nav_bar_top, nav_entries, nav_item_height,
    nav_nodes, nav_pill_gap, top_label_t,
};

pub(crate) struct Slot {
    pub node: u64,
    pub rect: Rect,
}

pub(crate) struct Bar {
    pub rect: Rect,
    pub slots: Vec<Slot>,
    pub badge: Option<Rect>,
}

fn avatar_center(slot: Rect) -> Pos2 {
    let near = Space::S8.px();
    let shift = (slot.left() + near + nav_avatar_radius() - slot.center().x) * top_label_t();
    Pos2::new(slot.center().x + shift, slot.center().y)
}

pub(crate) fn layout(viewport: Viewport, profile_name: &str) -> Bar {
    let nodes = nav_nodes();
    let pad = Insets::all(nav_bar_padding());
    let cells: Vec<crate::flex::Node> = nav_entries()
        .iter()
        .map(|(_, label)| crate::metrics::navigation_item_width(label))
        .chain([crate::metrics::navigation_profile_width(profile_name)])
        .enumerate()
        .map(|(index, width)| crate::flex::leaf(Vec2::new(width, nav_item_height())).id(index))
        .collect();
    let row = crate::flex::row(cells)
        .gap(nav_pill_gap())
        .padding(pad)
        .align(crate::flex::Align::Start);
    let size = row.outer_size();
    let margin = Space::S12.px();
    let lane = crate::gfx::rect_at(
        Pos2::new(0.0, nav_bar_top() + viewport.nav_lead),
        Vec2::new(viewport.width.max(size.x + margin), size.y),
    );
    let rect = crate::flex::anchored(lane, Align2::CENTER_TOP, size, 0.0);
    let placed = row.solve(rect);
    let boxes = (0..nodes.len()).map(|index| placed.find(index).unwrap_or(rect));
    let slots: Vec<Slot> = nodes
        .iter()
        .zip(boxes)
        .map(|(&node, rect)| Slot { node, rect })
        .collect();
    let badge = crate::update::nav_status().and_then(|_| {
        let profile: &Slot = slots.last()?;
        Some(crate::update::badge_rect(
            avatar_center(profile.rect),
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

pub(crate) fn focus_rects(viewport: Viewport) -> Vec<(u64, Rect)> {
    if viewport.is_compact() {
        return Vec::new();
    }
    let bar = layout(viewport, &drawn_profile());
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
    if viewport.side_inset > 0.0 || viewport.is_compact() {
        return None;
    }
    crate::metrics::enter(viewport);
    Some(layout(viewport, &drawn_profile()).rect)
}

pub fn nav_node_at(viewport: Viewport, position: Pos2) -> u64 {
    crate::metrics::enter(viewport);
    node_at(&layout(viewport, &drawn_profile()), position)
}

pub fn nav_slot_center(viewport: Viewport, x: f32) -> f32 {
    crate::metrics::enter(viewport);
    slot_center(&layout(viewport, &drawn_profile()), x)
}
