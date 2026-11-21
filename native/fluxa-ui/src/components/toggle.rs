use super::*;

pub(crate) fn toggle(
    context: &crate::gfx::Context,
    painter: &Painter,
    id: Id,
    track: Rect,
    on: bool,
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
    let state = crate::stylesheet::State {
        selected: on,
        enabled: true,
        ..Default::default()
    };
    let look = crate::stylesheet::resolve(context, "toggle", id, state);
    let t = look.selected;
    painter.rect_filled(track, look.radius.of(track.height()), look.fill);
    painter.rect_stroke(
        track,
        track.height() * 0.5,
        crate::gfx::Stroke::new(look.stroke, look.border),
        crate::gfx::StrokeKind::Inside,
    );
    let radius = crate::vars::px("settings-toggle-knob-radius")
        .min(track.height() * 0.5 - crate::vars::px("toggle-knob-inset"));
    let inset = crate::vars::px("toggle-knob-inset");
    let travel = track.width() - (radius + inset) * 2.0;
    let stretch = (t * (1.0 - t) * 4.0) * radius * 0.5;
    let x = track.left() + radius + inset + travel * t;
    let knob = crate::flex::centered(track, Vec2::new(radius * 2.0 + stretch, radius * 2.0))
        .translate(Vec2::new(x - track.center().x, 0.0));
    let knob_color =
        crate::stylesheet::resolve(context, "toggle.knob", id.with("knob"), state).fill;
    painter.rect_filled(knob, radius, knob_color);
}
