use super::*;
use crate::flex::{Weight, paint_text};
use crate::kit::{self, view};
use crate::{IconSize, Radius, Space};

pub(crate) fn chip(ui: &mut Ui, label: &str, selected: bool, metrics: UiMetrics) -> Response {
    let font = Weight::Medium.font(metrics.text.label);
    let text_size = crate::flex::text_size(ui.painter(), label, &font);
    let shown = view()
        .row()
        .height(metrics.icon_button_size * 0.8)
        .max_width(text_size.x + Space::S12.px(&metrics) * 2.0)
        .clickable()
        .class("control-chip")
        .show(ui, |ui| {
            kit::space(ui, Space::S12);
            ui.allocate_exact_size(text_size, Sense::hover()).0
        });
    let rect = shown.response.rect;
    let slot = shown.inner;
    let response = shown.response;
    let style = crate::stylesheet::resolve(
        ui.ctx(),
        if selected { "chip.selected" } else { "chip" },
        response.id,
        crate::stylesheet::State {
            selected: false,
            hovered: response.hovered(),
            pressed: response.is_pointer_button_down_on(),
            focused: response.has_focus(),
            enabled: true,
        },
        Some(&metrics),
    );
    crate::motion::press_scale(ui.painter(), rect, || {
        let painter = ui.painter();
        painter.rect_filled(rect, style.radius.of(rect.height()), style.fill);
        paint_text(painter, slot, label, &font, style.text, Align2::LEFT_CENTER);
    });
    response
}

pub(crate) fn segmented_width(painter: &Painter, labels: &[String], metrics: UiMetrics) -> f32 {
    let font = Weight::Medium.font(metrics.text.label);
    let widest = labels
        .iter()
        .map(|label| crate::flex::text_size(painter, label, &font).x)
        .fold(0.0, f32::max);
    labels.len() as f32 * (widest + 28.0) + Space::S2.px(&metrics) * 2.0
}

pub(crate) fn segmented(
    ui: &mut Ui,
    id: Id,
    labels: &[&str],
    selected: usize,
    metrics: UiMetrics,
) -> Option<usize> {
    let font = Weight::Medium.font(metrics.text.label);
    let height = metrics.screen_control_height;
    let width = ui.available_width().min(labels.len() as f32 * 120.0);
    let pad = Space::S2.px(&metrics);
    let count = labels.len().max(1) as f32;
    let shown = view()
        .row()
        .height(height)
        .max_width(width)
        .padding(Space::S2)
        .show(ui, |ui| {
            let cell = Vec2::new((width - pad * 2.0) / count, height - pad * 2.0);
            labels
                .iter()
                .map(|_| ui.allocate_exact_size(cell, Sense::hover()).0)
                .collect::<Vec<Rect>>()
        });
    let rect = shown.response.rect;
    let cells = shown.inner;
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
    let first = cells.first().copied().unwrap_or(rect);
    let slot = first.width();
    let indicator = Rect::from_min_size(
        Pos2::new(first.left() + slot * position, first.top()),
        first.size(),
    );
    painter.rect_filled(
        indicator,
        indicator.height() * 0.5,
        crate::stylesheet::look(ui.ctx(), "segmented.indicator").fill,
    );
    let mut chosen = None;
    for (index, label) in labels.iter().enumerate() {
        let cell = cells.get(index).copied().unwrap_or(rect);
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
                None,
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
    metrics: UiMetrics,
) -> Option<i32> {
    let font = Weight::Bold.font(metrics.text.body);
    let button = metrics.screen_control_height;
    let value_width = crate::flex::text_size(ui.painter(), value_label, &font)
        .x
        .max(button);
    let width = button * 2.0 + value_width + Space::S4.px(&metrics) * 2.0;
    let shown = view()
        .row()
        .gap(Space::S4)
        .height(button)
        .max_width(width)
        .class("control-stepper")
        .show(ui, |ui| {
            let minus = ui
                .allocate_exact_size(Vec2::splat(button), Sense::hover())
                .0;
            let value = ui
                .allocate_exact_size(Vec2::new(value_width, button), Sense::hover())
                .0;
            let plus = ui
                .allocate_exact_size(Vec2::splat(button), Sense::hover())
                .0;
            [minus, value, plus]
        });
    let placed = shown.inner;
    let mut delta = None;
    for (index, sign, enabled) in [(0, -1, can_decrease), (2, 1, can_increase)] {
        let slot = placed[index];
        let response = ui.interact(slot, id.with(index), Sense::click());
        let look = crate::stylesheet::resolve(
            ui.ctx(),
            "stepper.button",
            response.id,
            crate::stylesheet::State {
                hovered: response.hovered(),
                enabled,
                ..Default::default()
            },
            None,
        );
        let painter = ui.painter();
        painter.rect_filled(slot, slot.height() * 0.5, look.fill);
        let arm = IconSize::Sm.px(&metrics) * 0.4;
        let color = look.text;
        let line = egui::Stroke::new(stroke::emphasis(), color);
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
        placed[1],
        value_label,
        &font,
        metrics.text_primary,
        Align2::CENTER_CENTER,
    );
    delta
}

pub(crate) fn switch(ui: &mut Ui, id: Id, on: bool, metrics: UiMetrics) -> Response {
    let size = Vec2::new(
        metrics.settings_toggle_width,
        metrics.settings_toggle_height,
    );
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());
    toggle(ui.ctx(), ui.painter(), id, rect, on, metrics);
    response
}

pub(crate) enum Trailing<'a> {
    Value(&'a str),
    Chevron,
    Switch(bool),
}

pub(crate) struct ListRow<'a> {
    pub icon: Option<egui::TextureId>,
    pub title: &'a str,
    pub subtitle: Option<&'a str>,
    pub trailing: Trailing<'a>,
}

pub(crate) fn list_row(ui: &mut Ui, row_data: ListRow, metrics: UiMetrics) -> Response {
    let title_font = Weight::Medium.font(metrics.text.body);
    let sub_font = Weight::Regular.font(metrics.text.label);
    let value_font = Weight::Regular.font(metrics.text.body);
    let icon = IconSize::Md.px(&metrics);
    let trailing_size = match &row_data.trailing {
        Trailing::Value(value) => {
            let width = crate::flex::text_size(ui.painter(), value, &value_font).x;
            Vec2::new(
                width + IconSize::Sm.px(&metrics) + Space::S4.px(&metrics),
                20.0,
            )
        }
        Trailing::Chevron => Vec2::splat(IconSize::Sm.px(&metrics)),
        Trailing::Switch(_) => Vec2::new(
            metrics.settings_toggle_width,
            metrics.settings_toggle_height,
        ),
    };
    let title_height = metrics.text.body * 1.3;
    let mut text_height = title_height;
    if row_data.subtitle.is_some() {
        text_height += Space::S2.px(&metrics) + metrics.text.label * 1.3;
    }
    let icon_size = if row_data.icon.is_some() {
        Vec2::splat(icon)
    } else {
        Vec2::ZERO
    };
    let shown = view()
        .row()
        .gap(Space::S12)
        .height(metrics.settings_row_height)
        .clickable()
        .class("control-list-row")
        .show(ui, |ui| {
            kit::space(ui, Space::S16);
            let leading = ui.allocate_exact_size(icon_size, Sense::hover()).0;
            let text_width = (ui.available_width()
                - Space::S12.px(&metrics)
                - trailing_size.x
                - Space::S16.px(&metrics))
            .max(0.0);
            let (title, subtitle) = view()
                .column()
                .gap(Space::S2)
                .height(text_height)
                .max_width(text_width)
                .show(ui, |ui| {
                    let title = ui
                        .allocate_exact_size(Vec2::new(text_width, title_height), Sense::hover())
                        .0;
                    let subtitle = row_data.subtitle.map(|_| {
                        ui.allocate_exact_size(
                            Vec2::new(text_width, metrics.text.label * 1.3),
                            Sense::hover(),
                        )
                        .0
                    });
                    (title, subtitle)
                })
                .inner;
            let trailing = ui.allocate_exact_size(trailing_size, Sense::hover()).0;
            (leading, title, subtitle, trailing)
        });
    let (leading, title, subtitle, slot) = shown.inner;
    let response = shown.response;
    let hover = crate::motion::hover(ui.ctx(), &response);
    let painter = ui.painter();
    painter.rect_filled(
        response.rect,
        Radius::Md.px(&metrics),
        metrics.ink((tone::faint() as f32 * hover) as u8),
    );
    if let Some(texture) = row_data.icon {
        painter.image(texture, leading, full_uv(), metrics.ink(tone::body()));
    }
    paint_text(
        painter,
        title,
        row_data.title,
        &title_font,
        metrics.text_primary,
        Align2::LEFT_CENTER,
    );
    if let (Some(subtitle_text), Some(line)) = (row_data.subtitle, subtitle) {
        paint_text(
            painter,
            line,
            subtitle_text,
            &sub_font,
            metrics.text_secondary,
            Align2::LEFT_CENTER,
        );
    }
    match row_data.trailing {
        Trailing::Value(value) => {
            let chevron = Rect::from_center_size(
                Pos2::new(
                    slot.right() - IconSize::Sm.px(&metrics) * 0.5,
                    slot.center().y,
                ),
                Vec2::splat(IconSize::Sm.px(&metrics)),
            );
            paint_text(
                painter,
                Rect::from_min_max(
                    slot.min,
                    Pos2::new(chevron.left() - Space::S4.px(&metrics), slot.max.y),
                ),
                value,
                &value_font,
                metrics.text_muted,
                Align2::RIGHT_CENTER,
            );
            paint_chevron(painter, chevron, &metrics);
        }
        Trailing::Chevron => paint_chevron(painter, slot, &metrics),
        Trailing::Switch(on) => {
            toggle(
                ui.ctx(),
                ui.painter(),
                response.id.with("switch"),
                slot,
                on,
                metrics,
            );
        }
    }
    response
}

fn paint_chevron(painter: &Painter, rect: Rect, metrics: &UiMetrics) {
    let arm = rect.width() * 0.25;
    let center = rect.center();
    let stroke = egui::Stroke::new(stroke::icon(), metrics.ink(tone::muted()));
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

pub(crate) fn swatch(ui: &mut Ui, color: Color32, selected: bool, metrics: UiMetrics) -> Response {
    let size = IconSize::Xl.px(&metrics);
    let outer = size + Space::S8.px(&metrics);
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(outer), Sense::click());
    let hover = crate::motion::hover(ui.ctx(), &response);
    crate::motion::press_scale(ui.painter(), rect, || {
        let painter = ui.painter();
        let radius = size * 0.5 * (1.0 + 0.06 * hover);
        painter.circle_filled(rect.center(), radius, color);
        painter.circle_stroke(
            rect.center(),
            radius,
            egui::Stroke::new(stroke::hairline(), metrics.border),
        );
        if selected {
            painter.circle_stroke(
                rect.center(),
                outer * 0.5 - 1.0,
                egui::Stroke::new(stroke::emphasis(), metrics.text_primary),
            );
        }
    });
    response
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{UiFormFactor, Viewport};

    fn click_at(context: &egui::Context, point: Pos2, mut run: impl FnMut(&mut Ui)) {
        context.set_fonts(crate::fonts::definitions());
        crate::kit::provide(context, metrics());
        let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(400.0, 300.0));
        let frames = [
            vec![egui::Event::PointerMoved(point)],
            vec![egui::Event::PointerButton {
                pos: point,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: Default::default(),
            }],
            vec![egui::Event::PointerButton {
                pos: point,
                button: egui::PointerButton::Primary,
                pressed: false,
                modifiers: Default::default(),
            }],
        ];
        for (tick, events) in frames.into_iter().enumerate() {
            let input = egui::RawInput {
                time: Some(tick as f64 / 60.0),
                predicted_dt: 1.0 / 60.0,
                screen_rect: Some(screen),
                events,
                ..Default::default()
            };
            context
                .run_ui(input, |root| {
                    egui::CentralPanel::default()
                        .frame(egui::Frame::NONE)
                        .show(root, |ui| run(ui));
                })
                .drop_without_applying_deltas();
        }
    }

    fn metrics() -> UiMetrics {
        UiMetrics::for_viewport(Viewport::new(400, 300, UiFormFactor::Desktop))
    }

    #[test]
    fn segmented_reports_the_clicked_segment() {
        let context = egui::Context::default();
        let metrics = metrics();
        let mut chosen = None;
        click_at(&context, Pos2::new(350.0, 12.0), |ui| {
            chosen = chosen.or(segmented(ui, Id::new("seg"), &["a", "b", "c"], 0, metrics));
        });
        assert_eq!(chosen, Some(2));
    }

    #[test]
    fn stepper_ignores_a_disabled_direction() {
        let context = egui::Context::default();
        let metrics = metrics();
        let mut delta = None;
        click_at(&context, Pos2::new(10.0, 12.0), |ui| {
            delta = delta.or(stepper(ui, Id::new("step"), "3", false, true, metrics));
        });
        assert_eq!(delta, None);
    }

    #[test]
    fn stepper_increases_from_the_plus_button() {
        let context = egui::Context::default();
        let metrics = metrics();
        let mut delta = None;
        let plus =
            metrics.screen_control_height * 2.0 + 4.0 * 2.0 + metrics.screen_control_height * 0.5;
        click_at(&context, Pos2::new(plus + 8.0, 12.0), |ui| {
            delta = delta.or(stepper(ui, Id::new("step"), "3", true, true, metrics));
        });
        assert_eq!(delta, Some(1));
    }
}
