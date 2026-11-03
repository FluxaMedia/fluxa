use super::*;

pub(crate) fn icon_button(
    ui: &mut Ui,
    icon: Option<egui::TextureId>,
    size: f32,
    color: Color32,
    framed: bool,
    active: bool,
    enabled: bool,
) -> Response {
    icon_button_sized(ui, icon, Vec2::splat(size), color, framed, active, enabled)
}

pub(crate) fn icon_button_sized(
    ui: &mut Ui,
    icon: Option<egui::TextureId>,
    size: Vec2,
    color: Color32,
    framed: bool,
    active: bool,
    enabled: bool,
) -> Response {
    let radius = size.y * 0.5;
    let (rect, response) = ui.allocate_exact_size(
        size,
        if enabled {
            Sense::click()
        } else {
            Sense::hover()
        },
    );
    let hovered = enabled && response.hovered();
    if framed {
        let fill = if active {
            46
        } else if hovered {
            34
        } else {
            18
        };
        ui.painter().rect_filled(rect, radius, ink(fill));
        ui.painter().rect_stroke(
            rect.shrink(0.5),
            radius,
            egui::Stroke::new(stroke::hairline(), ink(if active { 90 } else { 36 })),
            egui::StrokeKind::Inside,
        );
    } else if hovered {
        ui.painter().rect_filled(rect, radius, ink(tone::soft()));
    }
    let tint = if !enabled {
        color.gamma_multiply(0.4)
    } else if hovered {
        text_primary()
    } else {
        color
    };
    if let Some(icon) = icon {
        let placed = crate::flex::stack(
            Align2::CENTER_CENTER,
            [crate::flex::leaf(Vec2::splat(size.y * crate::style::glyph_ratio())).id(0)],
        )
        .solve(rect);
        if let Some(glyph) = placed.find(0) {
            ui.painter().image(icon, glyph, full_uv(), tint);
        }
    }
    response
}

pub(crate) fn tap(
    ui: &Ui,
    id: Id,
    rect: Rect,
    radius: f32,
    node: u64,
    targets: Option<&mut HomeLayout>,
) -> Response {
    let response = ui.interact(rect, id, Sense::click());
    state_layer(ui, rect, radius, &response);
    if let Some(layout) = targets {
        layout.focusable.push((node, rect));
    }
    response
}

pub(crate) fn state_layer(ui: &Ui, rect: Rect, radius: f32, response: &Response) {
    let strength = crate::motion::hover(ui.ctx(), response);
    if strength > 0.0 {
        ui.painter()
            .rect_filled(rect, radius, ink(tone::raised()).gamma_multiply(strength));
    }
}

pub(crate) fn back_button(
    ui: &mut Ui,
    icon: Option<egui::TextureId>,
    metrics: UiMetrics,
) -> Response {
    let size = metrics.icon_button_size;
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(size), Sense::click());
    let painter = ui.painter();
    painter.circle_filled(rect.center(), size * 0.5, metrics.surface);
    state_layer(ui, rect, size * 0.5, &response);
    painter.circle_stroke(
        rect.center(),
        size * 0.5 - 0.5,
        egui::Stroke::new(stroke::hairline(), ink(tone::raised())),
    );
    if let Some(icon) = icon {
        painter.image(
            icon,
            Rect::from_center_size(rect.center(), Vec2::splat(size * 0.5)),
            full_uv(),
            text_primary(),
        );
    }
    response
}
