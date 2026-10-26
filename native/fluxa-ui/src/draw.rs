use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Edge {
    Top,
    Left,
}

pub(crate) fn corners(radius: u8, edge: Edge) -> egui::CornerRadius {
    let on = |hit: bool| if hit { radius } else { 0 };
    egui::CornerRadius {
        nw: on(matches!(edge, Edge::Top | Edge::Left)),
        ne: on(edge == Edge::Top),
        sw: on(edge == Edge::Left),
        se: 0,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Cast {
    Up,
    Left,
}

pub(crate) fn shadow(metrics: &UiMetrics, cast: Cast) -> egui::Shadow {
    let reach = metrics.shadow_offset;
    let offset = match cast {
        Cast::Up => [0, -reach],
        Cast::Left => [-reach, 0],
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
        .stroke(egui::Stroke::new(stroke::hairline(), metrics.border))
        .corner_radius(corners(radius.corner(metrics), edge));
    match cast {
        Some(cast) => frame.shadow(shadow(metrics, cast)),
        None => frame,
    }
}

pub(crate) fn paint_shadow(painter: &egui::Painter, rect: Rect, radius: u8, shadow: egui::Shadow) {
    painter.add(shadow.as_shape(rect, radius));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn top_edge_rounds_only_the_upper_corners() {
        let c = corners(24, Edge::Top);
        assert_eq!((c.nw, c.ne, c.sw, c.se), (24, 24, 0, 0));
    }
}
