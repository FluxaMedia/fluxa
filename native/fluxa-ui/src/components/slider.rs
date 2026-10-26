use super::*;

pub(crate) struct Slider {
    pub id: Id,
    pub track: Rect,
    pub ratio: f32,
    pub fill: Color32,
    pub overdrive: Option<f32>,
    pub amount: f32,
    pub height: f32,
    pub knob: f32,
    pub bubble: Option<String>,
}

pub(crate) struct SliderOutput {
    pub response: Response,
    pub value: Option<f32>,
}

pub(crate) fn slider(ui: &mut Ui, painter: &Painter, slider: Slider) -> SliderOutput {
    let Slider {
        id,
        track,
        ratio,
        fill,
        overdrive,
        amount,
        height,
        knob: knob_radius,
        bubble,
    } = slider;
    let response = ui.interact(track, id, Sense::click_and_drag());
    let value = if response.clicked() || response.dragged() {
        response
            .interact_pointer_pos()
            .map(|pointer| ((pointer.x - track.left()) / track.width()).clamp(0.0, 1.0))
    } else {
        None
    };
    let alpha = (255.0 * amount) as u8;
    let ratio = ratio.clamp(0.0, 1.0);
    let bar = Rect::from_center_size(track.center(), Vec2::new(track.width(), height));
    painter.rect_filled(bar, radius::BAR, ink(alpha / 3));
    let filled = Rect::from_min_size(bar.min, Vec2::new(bar.width() * ratio, bar.height()));
    let split = overdrive.unwrap_or(1.0);
    let base = Rect::from_min_size(
        bar.min,
        Vec2::new(bar.width() * ratio.min(split), bar.height()),
    );
    painter.rect_filled(base, radius::BAR, fill.gamma_multiply(amount));
    let mut knob = fill;
    if ratio > split {
        let span = 1.0 - split;
        let start = bar.left() + bar.width() * split;
        let end_t = ((ratio - split) / span).clamp(0.0, 1.0);
        gradient_bar(
            painter,
            Rect::from_min_max(
                Pos2::new(start, bar.top()),
                Pos2::new(filled.right(), bar.bottom()),
            ),
            overdrive_color(0.0).gamma_multiply(amount),
            overdrive_color(end_t).gamma_multiply(amount),
        );
        knob = overdrive_color(end_t);
    }
    if overdrive.is_some() {
        let mark = bar.left() + bar.width() * split;
        painter.line_segment(
            [
                Pos2::new(mark, bar.center().y - crate::style::slider_mark()),
                Pos2::new(mark, bar.center().y + crate::style::slider_mark()),
            ],
            egui::Stroke::new(stroke::icon(), ink(alpha / 2)),
        );
    }
    let center = Pos2::new(filled.right(), bar.center().y);
    let lit = bubble.is_some() && (response.hovered() || response.dragged());
    if lit {
        painter.circle_filled(center, knob_radius + 7.0, Color32::from_white_alpha(40));
    }
    painter.circle_filled(center, knob_radius * amount, knob);
    if let (true, Some(text)) = (lit, bubble) {
        let galley = painter.layout_no_wrap(text, crate::fonts::bold(13.0), Color32::BLACK);
        let size = galley.size() + Vec2::new(20.0, 8.0);
        let pill = Rect::from_center_size(
            center - Vec2::new(0.0, knob_radius + 9.0 + size.y * 0.5 + 6.0),
            size,
        );
        painter.rect_filled(pill, size.y * 0.5, Color32::WHITE);
        painter.add(egui::Shape::convex_polygon(
            vec![
                Pos2::new(center.x - 5.0, pill.bottom() - 0.5),
                Pos2::new(center.x + 5.0, pill.bottom() - 0.5),
                Pos2::new(center.x, pill.bottom() + 5.0),
            ],
            Color32::WHITE,
            egui::Stroke::NONE,
        ));
        painter.galley(pill.center() - galley.size() * 0.5, galley, Color32::BLACK);
    }
    SliderOutput { response, value }
}

pub(crate) fn gradient_bar(painter: &Painter, rect: Rect, left: Color32, right: Color32) {
    let mut mesh = egui::Mesh::default();
    let vertex = |pos: Pos2, color: Color32| egui::epaint::Vertex {
        pos,
        uv: egui::epaint::WHITE_UV,
        color,
    };
    mesh.vertices.extend([
        vertex(rect.left_top(), left),
        vertex(rect.right_top(), right),
        vertex(rect.right_bottom(), right),
        vertex(rect.left_bottom(), left),
    ]);
    mesh.indices.extend([0, 1, 2, 0, 2, 3]);
    painter.add(egui::Shape::mesh(mesh));
}

fn overdrive_color(t: f32) -> Color32 {
    let mix = |a: u8, b: u8| (a as f32 + (b as f32 - a as f32) * t) as u8;
    Color32::from_rgb(mix(250, 239), mix(204, 68), mix(21, 68))
}
