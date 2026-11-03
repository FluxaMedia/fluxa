use super::*;

pub(crate) fn brand_lockup(
    painter: &Painter,
    center: Pos2,
    height: f32,
    alpha: u8,
    assets: &impl HomeAssets,
) {
    let Some((texture, aspect)) = assets.logo("wordmark") else {
        return;
    };
    painter.image(
        texture,
        Rect::from_center_size(center, Vec2::new(height * aspect.x, height)),
        full_uv(),
        Color32::WHITE.gamma_multiply_u8(alpha),
    );
}

pub(crate) fn toggle(
    context: &egui::Context,
    painter: &Painter,
    id: Id,
    track: Rect,
    on: bool,
    metrics: UiMetrics,
) {
    let seen = id.with("haptic");
    let before = context.data_mut(|data| {
        let before = data.get_temp::<bool>(seen);
        data.insert_temp(seen, on);
        before
    });
    if before.is_some_and(|before| before != on) {
        crate::buzz(if on {
            crate::Haptic::ToggleOn
        } else {
            crate::Haptic::ToggleOff
        });
    }
    let t = crate::motion::presence(context, id, on, crate::motion::normal());
    let off_track = metrics.ink(tone::strong());
    painter.rect_filled(
        track,
        track.height() * 0.5,
        lerp_color(off_track, metrics.accent, t),
    );
    painter.rect_stroke(
        track,
        track.height() * 0.5,
        egui::Stroke::new(
            stroke::hairline(),
            metrics.ink(tone::dim()).gamma_multiply(1.0 - t),
        ),
        egui::StrokeKind::Inside,
    );
    let radius = metrics
        .settings_toggle_knob_radius
        .min(track.height() * 0.5 - 2.0);
    let travel = track.width() - (radius + 2.0) * 2.0;
    let stretch = (t * (1.0 - t) * 4.0) * radius * 0.5;
    let x = track.left() + radius + 2.0 + travel * t;
    let knob = Rect::from_center_size(
        Pos2::new(x, track.center().y),
        Vec2::new(radius * 2.0 + stretch, radius * 2.0),
    );
    let knob_color = lerp_color(
        metrics.ink(tone::subtle()),
        if metrics.accent_foreground == metrics.text_primary {
            metrics.text_primary
        } else {
            metrics.background
        },
        t,
    );
    painter.rect_filled(knob, radius, knob_color);
}

pub(super) fn lerp_color(a: Color32, b: Color32, t: f32) -> Color32 {
    let mix = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * t).round() as u8;
    Color32::from_rgba_unmultiplied(
        mix(a.r(), b.r()),
        mix(a.g(), b.g()),
        mix(a.b(), b.b()),
        mix(a.a(), b.a()),
    )
}
