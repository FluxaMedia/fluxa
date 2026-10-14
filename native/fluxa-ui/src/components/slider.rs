use super::*;

pub(crate) struct Slider {
    pub id: Id,
    pub track: Rect,
    pub ratio: f32,
    pub fill: Color32,
    pub overdrive: Option<f32>,
    pub amount: f32,
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
    let bar = Rect::from_center_size(track.center(), Vec2::new(track.width(), 4.0));
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
                Pos2::new(mark, bar.center().y - 5.0),
                Pos2::new(mark, bar.center().y + 5.0),
            ],
            egui::Stroke::new(stroke::ICON, ink(alpha / 2)),
        );
    }
    painter.circle_filled(
        Pos2::new(filled.right(), bar.center().y),
        6.0 * amount,
        knob,
    );
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
