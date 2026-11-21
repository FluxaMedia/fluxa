use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ButtonKind {
    Primary,
    Secondary,
    Selected,
    Danger,
}

pub(crate) fn overlay_chip(painter: &Painter, rect: Rect, outline: Option<Color32>) {
    painter.rect_filled(rect, radius::chip(), badge_fill());
    if let Some(color) = outline {
        painter.rect_stroke(
            rect,
            radius::chip(),
            crate::gfx::Stroke::new(stroke::hairline(), color),
            crate::gfx::StrokeKind::Inside,
        );
    }
}

pub(crate) fn placed_front<R>(
    context: &crate::gfx::Context,
    id: Id,
    pos: Pos2,
    add: impl FnOnce(&mut Ui) -> R,
) -> crate::gfx::InnerResponse<R> {
    crate::gfx::Area::new(id)
        .constrain(false)
        .fixed_pos(pos)
        .order(crate::gfx::Order::Foreground)
        .show(context, add)
}

impl ButtonKind {
    fn rule(self) -> &'static str {
        match self {
            ButtonKind::Primary => "button.primary",
            ButtonKind::Secondary => "button.secondary",
            ButtonKind::Selected => "button.selected",
            ButtonKind::Danger => "button.danger",
        }
    }
}

pub(crate) fn button_core(
    ui: &mut Ui,
    label: &str,
    width: f32,
    height: f32,
    kind: ButtonKind,
    text_size: f32,
    enabled: bool,
) -> Response {
    let (rect, response) = ui.allocate_exact_size(
        Vec2::new(width, height),
        if enabled {
            Sense::click()
        } else {
            Sense::hover()
        },
    );
    let style = crate::stylesheet::resolve(
        ui.ctx(),
        kind.rule(),
        response.id,
        crate::stylesheet::State {
            selected: false,
            hovered: response.hovered(),
            pressed: response.is_pointer_button_down_on(),
            focused: response.has_focus(),
            enabled,
        },
    );
    let (fill, text) = (style.fill, style.text);
    let painter = ui.painter();
    crate::motion::press_scale(painter, rect, || {
        painter.rect_filled(rect, height * 0.5, fill);
        draw_text(
            painter,
            rect.center(),
            Align2::CENTER_CENTER,
            label,
            crate::fonts::bold(text_size),
            text,
            None,
        );
    });
    if enabled && response.has_focus() {
        let (expand, width) = (
            crate::vars::px("focus-ring-expand"),
            crate::vars::px("focus-ring-width"),
        );
        painter.rect_stroke(
            rect.expand(expand),
            height * 0.5 + expand,
            crate::gfx::Stroke::new(width, accent()),
            crate::gfx::StrokeKind::Outside,
        );
    }
    response
}
