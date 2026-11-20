use super::*;

pub(super) fn rank_lead(node: &Resolved, width: Option<f32>) -> f32 {
    let rank = prop(node, "rank").parse::<usize>().ok();
    match (rank, width) {
        (Some(rank), Some(width)) => crate::poster_overlay::lead_width(width * 1.5, Some(rank)),
        _ => 0.0,
    }
}

pub(super) fn card_width<A: HomeAssets>(node: &Resolved, env: &Env<'_, A>) -> Option<f32> {
    let width = card_width_base(node, env);
    width.map(|width| width + rank_lead(node, Some(width)))
}

pub(super) fn card_width_base<A: HomeAssets>(node: &Resolved, env: &Env<'_, A>) -> Option<f32> {
    node.width
        .or(node.width_vw.map(|share| share * env.view.get().x))
        .or_else(|| {
            class_parts(node, None)
                .and_then(|(rule, modifiers)| crate::stylesheet::base_with(&rule, &modifiers).width)
        })
}

pub(super) fn far_card<'a, A: HomeAssets>(
    node: &Resolved,
    width: f32,
    height: Option<f32>,
    env: &'a Env<'a, A>,
) -> kit::Item<'a> {
    let target = env.claim(&node.key, node.action.as_ref(), false);
    kit::shared(Vec2::new(width, height.unwrap_or(0.0)), move |ui, rect| {
        activate(ui, rect, target)
    })
}

pub(super) fn fixed<'a, A: HomeAssets>(
    ui: &mut Ui,
    node: &Resolved,
    env: &'a Env<'a, A>,
) -> kit::Item<'a> {
    let number = |name: &str| {
        node.props.get(name).and_then(|value| match value {
            Value::Number(number) => number.as_f64(),
            Value::String(text) => text.parse().ok(),
            _ => None,
        })
    };
    if let (Some(x), Some(y)) = (number("x"), number("y")) {
        let content = layout_items::column(children(ui, node, env));
        let bottom = prop(node, "anchor") == "bottom";
        return kit::item(Vec2::ZERO, move |ui, _| {
            let view = ui.clip_rect();
            let point = if bottom {
                Pos2::new(view.left(), view.bottom())
            } else {
                Pos2::new(x as f32, y as f32)
            };
            kit::lay_point(ui, view, point, content);
        });
    }
    if prop(node, "snap") == "corners" {
        let content = layout_items::column(children(ui, node, env));
        let margin = margins(&prop(node, "margin"));
        let start = match &*prop(node, "corner") {
            "top-left" => 0,
            "top-right" => 1,
            "bottom-left" => 2,
            _ => 3,
        };
        let key = node.key.clone();
        return kit::item(Vec2::ZERO, move |ui, _| {
            let view = ui.clip_rect();
            let id = Id::new(("skin-snap", &key));
            let mut state = ui
                .ctx()
                .data(|data| data.get_temp::<SnapState>(id))
                .unwrap_or(SnapState {
                    corner: start,
                    pos: None,
                    size: Vec2::new(1.0, 1.0),
                });
            let corners = snap_corners(view, state.size, margin);
            let mut pos = state.pos.unwrap_or(corners[state.corner]);
            let response = ui.interact(Rect::from_min_size(pos, state.size), id, Sense::drag());
            if response.dragged() {
                pos += response.drag_delta();
                pos = pos.clamp(view.min, view.max - state.size);
                state.corner = nearest_corner(&corners, pos);
            } else {
                let target = corners[state.corner];
                let step = 1.0 - (-ui.ctx().input(|input| input.stable_dt()) * 18.0).exp();
                let next = pos + (target - pos) * step;
                if (target - next).length() < 0.5 {
                    pos = target;
                } else {
                    pos = next;
                    ui.ctx().request_repaint();
                }
            }
            let placed = kit::lay_point(ui, view, pos, content);
            state.pos = Some(pos);
            state.size = placed.size();
            crate::components::mark_mini_player(ui.ctx(), placed);
            ui.ctx().data_mut(|data| data.insert_temp(id, state));
        });
    }
    let safe = env.safe;
    let full = env.view.get();
    env.view
        .set(full - Vec2::new(safe.left + safe.right, safe.top + safe.bottom));
    let content = layout_items::column(children(ui, node, env))
        .align(FlexAlign::Center)
        .padding(Insets::all(12.0));
    env.view.set(full);
    kit::item(Vec2::ZERO, move |ui, _| {
        let view = ui.clip_rect();
        let view = Rect::from_min_max(
            view.min + Vec2::new(safe.left, safe.top),
            view.max - Vec2::new(safe.right, safe.bottom),
        );
        kit::lay_in(ui, view, content);
    })
}

#[derive(Clone, Copy)]
pub(super) struct SnapState {
    pub(super) corner: usize,
    pub(super) pos: Option<Pos2>,
    pub(super) size: Vec2,
}

pub(super) fn margins(text: &str) -> [f32; 4] {
    let values: Vec<f32> = text
        .split_whitespace()
        .filter_map(|part| part.parse().ok())
        .collect();
    match values.as_slice() {
        [all] => [*all; 4],
        [top, right, bottom, left] => [*top, *right, *bottom, *left],
        _ => [24.0; 4],
    }
}

pub(super) fn snap_corners(
    view: Rect,
    size: Vec2,
    [top, right, bottom, left]: [f32; 4],
) -> [Pos2; 4] {
    let right_x = view.right() - right - size.x;
    let bottom_y = view.bottom() - bottom - size.y;
    [
        Pos2::new(view.left() + left, view.top() + top),
        Pos2::new(right_x, view.top() + top),
        Pos2::new(view.left() + left, bottom_y),
        Pos2::new(right_x, bottom_y),
    ]
}

pub(super) fn nearest_corner(corners: &[Pos2; 4], pos: Pos2) -> usize {
    (0..4)
        .min_by(|a, b| {
            (corners[*a] - pos)
                .length_sq()
                .total_cmp(&(corners[*b] - pos).length_sq())
        })
        .unwrap_or(3)
}
