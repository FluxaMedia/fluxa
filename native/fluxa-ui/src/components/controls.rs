use super::*;
use crate::dp::Dp;
use crate::flex::{Align, Insets, Weight, column, fill, leaf, paint_text, row, text};
use crate::{IconSize, Radius, Space};

pub(crate) fn chip(ui: &mut Ui, label: &str, selected: bool) -> Response {
    let font = Weight::Medium.font(crate::vars::px("text-label"));
    let tree = row([text(ui.painter(), label, &font).id(0)])
        .align(Align::Center)
        .padding(Insets::xy(Space::S12.px(), Space::S6.px()))
        .min_height(
            crate::vars::px("icon-button-size") * crate::vars::px("control-chip-height-ratio"),
        );
    let (rect, response) = ui.allocate_exact_size(tree.outer_size(), Sense::click());
    let slot = tree.solve(rect).find(0).unwrap_or(rect);
    let style = crate::stylesheet::resolve_with(
        ui.ctx(),
        "chip",
        response.id,
        crate::stylesheet::State {
            selected: false,
            hovered: response.hovered(),
            pressed: response.is_pointer_button_down_on(),
            focused: response.has_focus(),
            enabled: true,
        },
        if selected { &["selected"] } else { &[] },
    );
    crate::motion::press_scale(ui.painter(), rect, || {
        let painter = ui.painter();
        painter.rect_filled(rect, style.radius.of(rect.height()), style.fill);
        paint_text(painter, slot, label, &font, style.text, Align2::LEFT_CENTER);
    });
    response
}

pub(crate) fn segmented(ui: &mut Ui, id: Id, labels: &[&str], selected: usize) -> Option<usize> {
    let font = Weight::Medium.font(crate::vars::px("text-label"));
    let height = crate::vars::px("screen-control-height");
    let width = ui
        .available_width()
        .min(labels.len() as f32 * crate::vars::px("control-segment-max-width"));
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, height), Sense::hover());
    let pad = Space::S2.px();
    let tree = row(labels.iter().enumerate().map(|(i, _)| fill().id(i)))
        .padding(Insets::all(pad))
        .align(Align::Stretch);
    let placed = tree.solve(rect);
    let painter = ui.painter();
    painter.rect_filled(
        rect,
        rect.height() * 0.5,
        crate::stylesheet::look(ui.ctx(), "segmented").fill,
    );
    let position = crate::motion::ease(
        ui.ctx(),
        id.with("indicator"),
        selected as f32,
        crate::motion::normal(),
        crate::motion::Ease::Token,
    );
    let first = placed.find(0).unwrap_or(rect);
    let slot = first.width();
    let indicator = first.translate(Vec2::new(slot * position, 0.0));
    painter.rect_filled(
        indicator,
        indicator.height() * 0.5,
        crate::stylesheet::look(ui.ctx(), "segmented.indicator").fill,
    );
    let mut chosen = None;
    for (index, label) in labels.iter().enumerate() {
        let cell = placed.find(index).unwrap_or(rect);
        let response = ui.interact(cell, id.with(index), Sense::click());
        paint_text(
            ui.painter(),
            cell,
            label,
            &font,
            crate::stylesheet::resolve(
                ui.ctx(),
                "segmented.label",
                response.id,
                crate::stylesheet::State {
                    hovered: response.hovered(),
                    selected: index == selected,
                    enabled: true,
                    ..Default::default()
                },
            )
            .text,
            Align2::CENTER_CENTER,
        );
        if response.clicked() && index != selected {
            chosen = Some(index);
        }
    }
    chosen
}

pub(crate) fn stepper(
    ui: &mut Ui,
    id: Id,
    value_label: &str,
    can_decrease: bool,
    can_increase: bool,
) -> Option<i32> {
    let font = Weight::Bold.font(crate::vars::px("text-body"));
    let button = crate::vars::px("screen-control-height");
    let value_width = crate::flex::text_size(ui.painter(), value_label, &font)
        .x
        .max(button);
    let tree = row([
        leaf(Vec2::splat(button)).id(0),
        leaf(Vec2::new(value_width, button)).id(1),
        leaf(Vec2::splat(button)).id(2),
    ])
    .gap(Space::S4.px())
    .align(Align::Center);
    let (rect, _) = ui.allocate_exact_size(tree.outer_size(), Sense::hover());
    let placed = tree.solve(rect);
    let mut delta = None;
    for (slot_id, sign, enabled) in [(0, -1, can_decrease), (2, 1, can_increase)] {
        let slot = placed.find(slot_id).unwrap_or(rect);
        let response = ui.interact(slot, id.with(slot_id), Sense::click());
        let look = crate::stylesheet::resolve(
            ui.ctx(),
            "stepper.button",
            response.id,
            crate::stylesheet::State {
                hovered: response.hovered(),
                enabled,
                ..Default::default()
            },
        );
        let painter = ui.painter();
        painter.rect_filled(slot, slot.height() * 0.5, look.fill);
        let arm = IconSize::Sm.px() * 0.4;
        let color = look.text;
        let line = crate::gfx::Stroke::new(stroke::emphasis(), color);
        let center = slot.center();
        painter.line_segment(
            [center - Vec2::new(arm, 0.0), center + Vec2::new(arm, 0.0)],
            line,
        );
        if sign > 0 {
            painter.line_segment(
                [center - Vec2::new(0.0, arm), center + Vec2::new(0.0, arm)],
                line,
            );
        }
        if enabled && response.clicked() {
            delta = Some(sign);
        }
    }
    paint_text(
        ui.painter(),
        placed.find(1).unwrap_or(rect),
        value_label,
        &font,
        crate::metrics::theme().text_primary,
        Align2::CENTER_CENTER,
    );
    delta
}

pub(crate) fn switch(ui: &mut Ui, id: Id, on: bool) -> Response {
    let size = Vec2::new(
        crate::vars::px("settings-toggle-width"),
        crate::vars::px("settings-toggle-height"),
    );
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());
    toggle(ui.ctx(), ui.painter(), id, rect, on);
    response
}

pub(crate) enum Trailing<'a> {
    Value(&'a str),
    Chevron,
    Switch(bool),
}

pub(crate) struct ListRow<'a> {
    pub icon: Option<crate::gfx::TextureId>,
    pub title: &'a str,
    pub subtitle: Option<&'a str>,
    pub trailing: Trailing<'a>,
}

pub(crate) fn list_row(ui: &mut Ui, row_data: ListRow) -> Response {
    let title_font = Weight::Medium.font(crate::vars::px("text-body"));
    let sub_font = Weight::Regular.font(crate::vars::px("text-label"));
    let value_font = Weight::Regular.font(crate::vars::px("text-body"));
    let icon = IconSize::Md.px();
    let trailing_size = match &row_data.trailing {
        Trailing::Value(value) => {
            let width = crate::flex::text_size(ui.painter(), value, &value_font).x;
            Vec2::new(width + IconSize::Sm.px() + Space::S4.px(), Dp::D20.px())
        }
        Trailing::Chevron => Vec2::splat(IconSize::Sm.px()),
        Trailing::Switch(_) => Vec2::new(
            crate::vars::px("settings-toggle-width"),
            crate::vars::px("settings-toggle-height"),
        ),
    };
    let mut lines = vec![
        leaf(Vec2::new(
            0.0,
            crate::vars::px("text-body") * crate::vars::px("control-row-line-ratio"),
        ))
        .id(1),
    ];
    if row_data.subtitle.is_some() {
        lines.push(
            leaf(Vec2::new(
                0.0,
                crate::vars::px("text-label") * crate::vars::px("control-row-line-ratio"),
            ))
            .id(2),
        );
    }
    let tree = row([
        leaf(if row_data.icon.is_some() {
            Vec2::splat(icon)
        } else {
            Vec2::ZERO
        })
        .id(0),
        column(lines)
            .grow(1.0)
            .align(Align::Stretch)
            .min_width(0.0)
            .gap(Space::S2.px()),
        leaf(trailing_size).id(3),
    ])
    .gap(Space::S12.px())
    .align(Align::Center)
    .padding(Insets::xy(Space::S16.px(), Space::S8.px()))
    .min_height(crate::vars::px("settings-row-height"));
    let size = Vec2::new(ui.available_width(), tree.outer_size().y);
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());
    let placed = tree.solve(rect);
    let hover = crate::motion::hover(ui.ctx(), &response);
    let painter = ui.painter();
    painter.rect_filled(
        rect,
        Radius::Md.px(),
        crate::metrics::ink((tone::faint() as f32 * hover) as u8),
    );
    if let (Some(texture), Some(slot)) = (row_data.icon, placed.find(0)) {
        painter.image(texture, slot, full_uv(), crate::metrics::ink(tone::body()));
    }
    if let Some(slot) = placed.find(1) {
        paint_text(
            painter,
            slot,
            row_data.title,
            &title_font,
            crate::metrics::theme().text_primary,
            Align2::LEFT_CENTER,
        );
    }
    if let (Some(subtitle), Some(slot)) = (row_data.subtitle, placed.find(2)) {
        paint_text(
            painter,
            slot,
            subtitle,
            &sub_font,
            crate::metrics::theme().text_secondary,
            Align2::LEFT_CENTER,
        );
    }
    if let Some(slot) = placed.find(3) {
        match row_data.trailing {
            Trailing::Value(value) => {
                let parts = row([fill().id(4), leaf(Vec2::splat(IconSize::Sm.px())).id(5)])
                    .gap(Space::S4.px())
                    .align(Align::Center)
                    .solve(slot);
                let chevron = parts.find(5).unwrap_or(slot);
                paint_text(
                    painter,
                    parts.find(4).unwrap_or(slot),
                    value,
                    &value_font,
                    crate::metrics::theme().text_muted,
                    Align2::RIGHT_CENTER,
                );
                paint_chevron(painter, chevron);
            }
            Trailing::Chevron => paint_chevron(painter, slot),
            Trailing::Switch(on) => {
                toggle(ui.ctx(), ui.painter(), response.id.with("switch"), slot, on);
            }
        }
    }
    response
}

fn paint_chevron(painter: &Painter, rect: Rect) {
    let arm = rect.width() * 0.25;
    let center = rect.center();
    let stroke = crate::gfx::Stroke::new(stroke::icon(), crate::metrics::ink(tone::muted()));
    painter.line_segment(
        [
            center + Vec2::new(-arm * 0.5, -arm),
            center + Vec2::new(arm * 0.5, 0.0),
        ],
        stroke,
    );
    painter.line_segment(
        [
            center + Vec2::new(arm * 0.5, 0.0),
            center + Vec2::new(-arm * 0.5, arm),
        ],
        stroke,
    );
}

pub(crate) fn swatch(ui: &mut Ui, color: Color32, selected: bool) -> Response {
    let size = IconSize::Xl.px();
    let outer = size + Space::S8.px();
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(outer), Sense::click());
    let hover = crate::motion::hover(ui.ctx(), &response);
    crate::motion::press_scale(ui.painter(), rect, || {
        let painter = ui.painter();
        let radius = size * 0.5 * (1.0 + crate::vars::px("control-swatch-hover-grow") * hover);
        painter.circle_filled(rect.center(), radius, color);
        painter.circle_stroke(
            rect.center(),
            radius,
            crate::gfx::Stroke::new(stroke::hairline(), crate::metrics::theme().border),
        );
        if selected {
            painter.circle_stroke(
                rect.center(),
                outer * 0.5 - Dp::D1.px(),
                crate::gfx::Stroke::new(stroke::emphasis(), crate::metrics::theme().text_primary),
            );
        }
    });
    response
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{UiFormFactor, Viewport};

    fn click_at(context: &crate::gfx::Context, point: Pos2, mut run: impl FnMut(&mut Ui)) {
        context.set_fonts(crate::fonts::definitions());
        let screen = crate::gfx::rect_at(Pos2::ZERO, Vec2::new(400.0, 300.0));
        let frames = [
            vec![crate::gfx::Event::PointerMoved(point)],
            vec![crate::gfx::Event::PointerButton {
                pos: point,
                button: crate::gfx::PointerButton::Primary,
                pressed: true,
                modifiers: Default::default(),
            }],
            vec![crate::gfx::Event::PointerButton {
                pos: point,
                button: crate::gfx::PointerButton::Primary,
                pressed: false,
                modifiers: Default::default(),
            }],
        ];
        for (tick, events) in frames.into_iter().enumerate() {
            let input = crate::gfx::RawInput {
                time: Some(tick as f64 / 60.0),
                predicted_dt: 1.0 / 60.0,
                screen_rect: Some(screen),
                events,
                ..Default::default()
            };
            context
                .run_ui(input, |root| {
                    crate::gfx::CentralPanel::default()
                        .frame(crate::gfx::Frame::NONE)
                        .show(root, |ui| run(ui));
                })
                .drop_without_applying_deltas();
        }
    }

    #[test]
    fn segmented_reports_the_clicked_segment() {
        let context = crate::gfx::Context::default();
        let mut chosen = None;
        click_at(&context, Pos2::new(350.0, 12.0), |ui| {
            chosen = chosen.or(segmented(ui, Id::new("seg"), &["a", "b", "c"], 0));
        });
        assert_eq!(chosen, Some(2));
    }

    #[test]
    fn stepper_ignores_a_disabled_direction() {
        let context = crate::gfx::Context::default();
        let mut delta = None;
        click_at(&context, Pos2::new(10.0, 12.0), |ui| {
            delta = delta.or(stepper(ui, Id::new("step"), "3", false, true));
        });
        assert_eq!(delta, None);
    }

    #[test]
    fn stepper_increases_from_the_plus_button() {
        let context = crate::gfx::Context::default();
        let mut delta = None;
        let plus = crate::vars::px("screen-control-height") * 2.0
            + 4.0 * 2.0
            + crate::vars::px("screen-control-height") * 0.5;
        click_at(&context, Pos2::new(plus + 8.0, 12.0), |ui| {
            delta = delta.or(stepper(ui, Id::new("step"), "3", true, true));
        });
        assert_eq!(delta, Some(1));
    }
}
