use super::*;

pub(crate) fn focus_ring(
    painter: &Painter,
    focusable: &[(u64, Rect)],
    focused: Option<u64>,
    viewport: Viewport,
    in_player: bool,
    scope: u64,
    metrics: UiMetrics,
) {
    use crate::*;
    let context = painter.ctx().clone();
    let state_id = Id::new("fluxa-focus-ring");
    let Some(node) = focused else {
        context.data_mut(|data| data.remove::<RingState>(state_id));
        return;
    };
    if viewport.is_compact() || is_text_node(node) || (in_player && node == NODE_PLAYER_SEEK) {
        context.data_mut(|data| data.remove::<RingState>(state_id));
        return;
    }
    if let Some((_, rect)) = focusable.iter().find(|(id, _)| *id == node) {
        let rect = rect.expand(metrics.focus_ring_expand);
        let radius = if in_player {
            if (NODE_PLAYER_SOURCE_BASE..NODE_PLAYER_PANEL_ROW_BASE).contains(&node) {
                metrics.card_radius + metrics.focus_ring_expand
            } else {
                rect.height().min(rect.width()) * 0.5
            }
        } else if is_pill_node(node) {
            rect.height() * 0.5
        } else {
            metrics.focus_ring_radius
        };
        let (rect, radius) = slide(&context, state_id, node, scope, rect, radius, metrics);
        painter.rect_stroke(
            rect,
            radius,
            crate::gfx::Stroke::new(metrics.focus_ring_width, accent()),
            crate::gfx::StrokeKind::Outside,
        );
    } else {
        context.data_mut(|data| data.remove::<RingState>(state_id));
    }
}

#[derive(Clone, Copy)]
struct RingState {
    node: u64,
    scope: u64,
    from: Rect,
    from_radius: f32,
    start: f64,
    shown: Rect,
    shown_radius: f32,
}

fn mix(from: Rect, to: Rect, t: f32) -> Rect {
    let at = |a: Pos2, b: Pos2| Pos2::new(a.x + (b.x - a.x) * t, a.y + (b.y - a.y) * t);
    Rect::from_min_max(at(from.min, to.min), at(from.max, to.max))
}

fn slide(
    context: &crate::gfx::Context,
    id: Id,
    node: u64,
    scope: u64,
    rect: Rect,
    radius: f32,
    metrics: UiMetrics,
) -> (Rect, f32) {
    let now = context.input(|input| input.time());
    let previous = context
        .data(|data| data.get_temp::<RingState>(id))
        .filter(|state| state.scope == scope);
    let seconds = f64::from(metrics.focus_ring_slide);
    let snap = seconds <= 0.0 || crate::motion::reduced();
    let state = match previous {
        Some(previous) if !snap => {
            let moved = previous.node != node;
            RingState {
                node,
                scope,
                from: if moved { previous.shown } else { previous.from },
                from_radius: if moved {
                    previous.shown_radius
                } else {
                    previous.from_radius
                },
                start: if moved { now } else { previous.start },
                ..previous
            }
        }
        _ => RingState {
            node,
            scope,
            from: rect,
            from_radius: radius,
            start: now - seconds,
            shown: rect,
            shown_radius: radius,
        },
    };
    let progress = if snap {
        1.0
    } else {
        ((now - state.start) / seconds).clamp(0.0, 1.0) as f32
    };
    if progress < 1.0 {
        context.request_repaint();
    }
    let eased = crate::gfx::emath::easing::cubic_out(progress);
    let shown = mix(state.from, rect, eased);
    let shown_radius = state.from_radius + (radius - state.from_radius) * eased;
    context.data_mut(|data| {
        data.insert_temp(
            id,
            RingState {
                shown,
                shown_radius,
                ..state
            },
        )
    });
    (shown, shown_radius)
}

pub(super) fn is_pill_node(node: u64) -> bool {
    use crate::*;
    crate::HomeCategory::from_node(node).is_some()
        || matches!(
            node,
            NODE_HOME
                | NODE_LIBRARY
                | NODE_DISCOVER
                | NODE_CALENDAR
                | NODE_PROFILE
                | NODE_SHORTS
                | NODE_PLAY
                | NODE_HERO_WATCHLIST
                | NODE_HOME_ADD_ADDONS
                | NODE_DETAIL_BACK
                | NODE_DETAIL_PLAY
                | NODE_DETAIL_WATCHLIST
                | NODE_DETAIL_COMPLETED
                | NODE_DETAIL_DROPPED
                | NODE_DETAIL_FAVORITE
                | NODE_DETAIL_SHUFFLE
                | NODE_SHORTS_PLAY
                | NODE_SHORTS_WATCHLIST
                | NODE_SHORTS_INFO
                | NODE_SETTINGS_BACK
                | NODE_SETTINGS_SWITCH_PROFILE
                | NODE_SETTINGS_LOG_OUT
                | NODE_FOLDER_BACK
        )
        || (NODE_PROFILES_GEAR..NODE_PROFILES_CELL_BASE + 200).contains(&node)
}

pub fn is_text_node(node: u64) -> bool {
    matches!(
        node,
        crate::NODE_LIBRARY_SEARCH
            | crate::NODE_DISCOVER_SEARCH
            | crate::NODE_SIDEBAR_SEARCH
            | crate::NODE_SETTINGS_SEARCH
            | crate::NODE_SETTINGS_ADDON_URL
            | crate::NODE_SETTINGS_PLUGIN_URL
            | crate::NODE_SETTINGS_STREAMKIT_ADDON_URL
            | crate::NODE_SETTINGS_WATCH_CODE
            | crate::NODE_REPORT_TITLE
            | crate::NODE_ADDON_EDIT_NAME
            | crate::NODE_REPORT_BODY
    ) || crate::poster_field(node).is_some()
        || crate::server_input(node).is_some()
        || crate::badge_input(node).is_some()
        || crate::plugin_config_input(node).is_some()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{UiFormFactor, Viewport};

    fn shown_after(
        context: &crate::gfx::Context,
        time: f64,
        focusable: &[(u64, Rect)],
        node: u64,
        scope: u64,
        viewport: Viewport,
    ) -> Rect {
        let output = context.run_ui(
            crate::gfx::RawInput {
                time: Some(time),
                screen_rect: Some(Rect::from_min_size(
                    Pos2::ZERO,
                    Vec2::new(viewport.width, viewport.height),
                )),
                ..Default::default()
            },
            |ui| {
                let layout = crate::HomeLayout {
                    focusable: focusable.to_vec(),
                    ..Default::default()
                };
                crate::draw_focus(ui.ctx(), viewport, &layout, Some(node), false, scope);
            },
        );
        output.drop_without_applying_deltas();
        context
            .data(|data| data.get_temp::<RingState>(Id::new("fluxa-focus-ring")))
            .map(|state| state.shown)
            .unwrap()
    }

    #[test]
    fn ring_slides_between_items_and_snaps_across_screens() {
        let viewport = Viewport::new(1920, 1080, UiFormFactor::Tv);
        let context = crate::gfx::Context::default();
        let first = Rect::from_min_size(Pos2::new(100.0, 400.0), Vec2::new(200.0, 300.0));
        let second = Rect::from_min_size(Pos2::new(700.0, 400.0), Vec2::new(200.0, 300.0));
        let targets = [(1, first), (2, second)];
        let expand = crate::UiMetrics::for_viewport(viewport).focus_ring_expand;
        let at_first = shown_after(&context, 1.0, &targets, 1, 7, viewport);
        assert_eq!(at_first, first.expand(expand));
        let started = shown_after(&context, 2.0, &targets, 2, 7, viewport);
        assert_eq!(started, first.expand(expand));
        let midway = shown_after(&context, 2.05, &targets, 2, 7, viewport);
        assert!(midway.min.x > first.min.x && midway.min.x < second.min.x);
        let done = shown_after(&context, 3.0, &targets, 2, 7, viewport);
        assert_eq!(done, second.expand(expand));
        let other_screen = shown_after(&context, 4.0, &targets, 1, 8, viewport);
        assert_eq!(other_screen, first.expand(expand));
    }
}
