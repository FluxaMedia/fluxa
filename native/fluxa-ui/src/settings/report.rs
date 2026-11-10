use super::*;
use crate::kit::UiKit as _;

pub const REPORT_TITLE_LIMIT: usize = 100;
pub const REPORT_BODY_LIMIT: usize = 1000;
const REPORT_BODY_LINES: usize = 6;

pub(super) fn draw_report_modal(
    context: &crate::gfx::Context,
    viewport: Viewport,
    settings: &SettingsModel,
    layout: &mut HomeLayout,
) {
    if !settings.report_open {
        return;
    }
    let language = settings
        .values
        .get("language")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("en");
    let metrics = UiMetrics::for_viewport(viewport);
    layout.focusable.clear();
    layout.activated = None;
    layout.text_input = None;
    layout.text_input_node = None;
    let title = std::cell::RefCell::new(settings.report_title.clone());
    let body = std::cell::RefCell::new(settings.report_body.clone());
    crate::kit::dialog_with(context, "fluxa-report-problem", &["wide"], |ui| {
        use crate::kit::layout_items::gap;
        let title_field = std::cell::RefCell::new(None);
        let body_field = std::cell::RefCell::new(None);
        let cancel = std::cell::RefCell::new(None);
        let send = std::cell::RefCell::new(None);
        let field_width = ui.available_width();
        let heading = crate::kit::widget(ui, |ui| {
            ui.label(
                RichText::new(localized("settings.report.title", language))
                    .size(metrics.text.subtitle)
                    .strong()
                    .color(text_primary()),
            );
        });
        let description = crate::kit::widget(ui, |ui| {
            ui.label(
                RichText::new(localized("settings.report.description", language))
                    .size(metrics.text.meta)
                    .color(metrics.text_secondary),
            );
        });
        let title_item = crate::kit::widget(ui, |ui| {
            let mut value = title.borrow_mut();
            let field = components::text_input(
                ui,
                &mut value,
                &localized("settings.report.title_hint", language),
                field_width,
                components::InputKind::Text,
                metrics,
            );
            *value = value.chars().take(REPORT_TITLE_LIMIT).collect();
            *title_field.borrow_mut() = Some(field);
        });
        let body_item = crate::kit::widget(ui, |ui| {
            let mut value = body.borrow_mut();
            let field = components::text_area(
                ui,
                &mut value,
                &localized("settings.report.body_hint", language),
                field_width,
                REPORT_BODY_LINES,
                REPORT_BODY_LIMIT,
                metrics,
            );
            *body_field.borrow_mut() = Some(field);
        });
        let counter = crate::kit::widget(ui, |ui| {
            ui.label(
                RichText::new(format!(
                    "{}/{}",
                    body.borrow().chars().count(),
                    REPORT_BODY_LIMIT
                ))
                .size(metrics.text.meta)
                .color(metrics.text_muted),
            );
        });
        let actions = crate::kit::widget(ui, |ui| {
            let ready = !title.borrow().trim().is_empty() && !body.borrow().trim().is_empty();
            crate::kit::flex("fluxa-report-actions")
                .gap(Space::S8)
                .grow(|ui| {
                    *cancel.borrow_mut() = Some(components::button_with_text_size(
                        ui,
                        &localized("common.cancel", language),
                        ui.available_width(),
                        crate::metrics::button_height(),
                        components::ButtonKind::Secondary,
                        crate::style::size::BUTTON_TEXT,
                        metrics,
                    ));
                })
                .grow(|ui| {
                    *send.borrow_mut() = Some(components::primary_button(
                        ui,
                        &localized("settings.report.send", language),
                        ui.available_width(),
                        ready,
                    ));
                })
                .show(ui);
        });
        crate::kit::lay(
            ui,
            vec![
                heading,
                gap(Space::S6.px(&metrics)),
                description,
                gap(Space::S16.px(&metrics)),
                title_item,
                gap(Space::S10.px(&metrics)),
                body_item,
                gap(Space::S6.px(&metrics)),
                counter,
                gap(Space::S14.px(&metrics)),
                actions,
            ],
        );
        let ready = !title.borrow().trim().is_empty() && !body.borrow().trim().is_empty();
        if let Some(field) = title_field.take() {
            layout.focusable.push((NODE_REPORT_TITLE, field.rect));
            if field.changed() {
                layout.text_input = Some(title.borrow().clone());
                layout.text_input_node = Some(NODE_REPORT_TITLE);
            }
        }
        if let Some(field) = body_field.take() {
            layout.focusable.push((NODE_REPORT_BODY, field.rect));
            if field.changed() {
                layout.text_input = Some(body.borrow().clone());
                layout.text_input_node = Some(NODE_REPORT_BODY);
            }
        }
        if let Some(cancel) = cancel.take() {
            components::focus_action(layout, NODE_REPORT_CANCEL, &cancel);
        }
        if let Some(send) = send.take() {
            layout.focusable.push((NODE_REPORT_SEND, send.rect));
            if ready && send.clicked() {
                layout.activated = Some(NODE_REPORT_SEND);
            }
        }
        if ui.input(|input| input.key_pressed(crate::gfx::Key::Escape)) {
            layout.activated = Some(NODE_REPORT_CANCEL);
        }
    });
}
