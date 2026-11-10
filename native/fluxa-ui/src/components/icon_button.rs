use super::*;

pub(crate) fn icon_button(
    ui: &mut Ui,
    icon: Option<crate::gfx::TextureId>,
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
    icon: Option<crate::gfx::TextureId>,
    size: Vec2,
    color: Color32,
    framed: bool,
    active: bool,
    enabled: bool,
) -> Response {
    let (rect, response) = ui.allocate_exact_size(
        size,
        if enabled {
            Sense::click()
        } else {
            Sense::hover()
        },
    );
    let hovered = enabled && response.hovered();
    let rule = match (framed, active) {
        (true, true) => "icon.framed.active",
        (true, false) => "icon.framed",
        (false, _) => "icon.plain",
    };
    let style = crate::stylesheet::resolve(
        ui.ctx(),
        rule,
        response.id,
        crate::stylesheet::State {
            hovered,
            pressed: enabled && response.is_pointer_button_down_on(),
            focused: response.has_focus(),
            selected: false,
            enabled: true,
        },
        None,
    );
    let radius = style.radius.of(size.y);
    ui.painter().rect_filled(rect, radius, style.fill);
    if style.border.a() > 0 {
        ui.painter().rect_stroke(
            rect.shrink(0.5),
            radius,
            crate::gfx::Stroke::new(stroke::hairline(), style.border),
            crate::gfx::StrokeKind::Inside,
        );
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
        ui.painter().rect_filled(
            rect,
            radius,
            crate::stylesheet::color("raised").gamma_multiply(strength),
        );
    }
}

pub(crate) fn back_button(
    ui: &mut Ui,
    icon: Option<crate::gfx::TextureId>,
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
        crate::gfx::Stroke::new(stroke::hairline(), crate::stylesheet::color("raised")),
    );
    if let Some(icon) = icon {
        painter.image(
            icon,
            crate::flex::centered(rect, Vec2::splat(size * 0.5)),
            full_uv(),
            text_primary(),
        );
    }
    response
}
