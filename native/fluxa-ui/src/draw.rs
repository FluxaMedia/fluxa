use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Edge {
    Top,
    Bottom,
    Left,
    Right,
    All,
}

pub(crate) fn corners(radius: u8, edge: Edge) -> egui::CornerRadius {
    let on = |hit: bool| if hit { radius } else { 0 };
    egui::CornerRadius {
        nw: on(matches!(edge, Edge::Top | Edge::Left | Edge::All)),
        ne: on(matches!(edge, Edge::Top | Edge::Right | Edge::All)),
        sw: on(matches!(edge, Edge::Bottom | Edge::Left | Edge::All)),
        se: on(matches!(edge, Edge::Bottom | Edge::Right | Edge::All)),
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Cast {
    Up,
    Down,
    Left,
    Right,
    Around,
}

pub(crate) fn shadow(metrics: &UiMetrics, cast: Cast) -> egui::Shadow {
    let reach = metrics.shadow_offset;
    let offset = match cast {
        Cast::Up => [0, -reach],
        Cast::Down => [0, reach],
        Cast::Left => [-reach, 0],
        Cast::Right => [reach, 0],
        Cast::Around => [0, 0],
    };
    egui::Shadow {
        offset,
        blur: metrics.shadow_blur,
        spread: 0,
        color: Color32::from_black_alpha(metrics.shadow_alpha),
    }
}

pub(crate) fn surface_frame(
    metrics: &UiMetrics,
    radius: Radius,
    edge: Edge,
    cast: Option<Cast>,
) -> egui::Frame {
    let frame = egui::Frame::NONE
        .fill(metrics.surface_raised)
        .stroke(egui::Stroke::new(stroke::HAIRLINE, metrics.border))
        .corner_radius(corners(radius.corner(metrics), edge));
    match cast {
        Some(cast) => frame.shadow(shadow(metrics, cast)),
        None => frame,
    }
}

pub(crate) fn paint_shadow(painter: &egui::Painter, rect: Rect, radius: u8, shadow: egui::Shadow) {
    painter.add(shadow.as_shape(rect, radius));
}

pub(crate) fn fade(ui: &mut egui::Ui, alpha: f32, add: impl FnOnce(&mut egui::Ui)) {
    ui.scope(|ui| {
        ui.set_opacity(alpha.clamp(0.0, 1.0));
        add(ui);
    });
}

pub(crate) fn clipped(painter: &egui::Painter, rect: Rect, draw: impl FnOnce(&egui::Painter)) {
    draw(&painter.with_clip_rect(rect.intersect(painter.clip_rect())));
}

pub(crate) fn linear_gradient(
    painter: &egui::Painter,
    rect: Rect,
    vertical: bool,
    stops: &[(f32, Color32)],
) {
    for pair in stops.windows(2) {
        let [(from, start), (to, end)] = [pair[0], pair[1]];
        let slice = if vertical {
            Rect::from_min_max(
                Pos2::new(rect.left(), rect.top() + rect.height() * from),
                Pos2::new(rect.right(), rect.top() + rect.height() * to),
            )
        } else {
            Rect::from_min_max(
                Pos2::new(rect.left() + rect.width() * from, rect.top()),
                Pos2::new(rect.left() + rect.width() * to, rect.bottom()),
            )
        };
        if vertical {
            paint_vertical_gradient(painter, slice, start, end);
        } else {
            paint_horizontal_gradient(painter, slice, start, end);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn top_edge_rounds_only_the_upper_corners() {
        let c = corners(24, Edge::Top);
        assert_eq!((c.nw, c.ne, c.sw, c.se), (24, 24, 0, 0));
    }

    #[test]
    fn right_edge_rounds_only_the_right_corners() {
        let c = corners(12, Edge::Right);
        assert_eq!((c.nw, c.ne, c.sw, c.se), (0, 12, 0, 12));
    }
}
