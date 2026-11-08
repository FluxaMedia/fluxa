use super::*;

pub const REPORT_TITLE_LIMIT: usize = 100;
pub const REPORT_BODY_LIMIT: usize = 1000;
const REPORT_BODY_LINES: usize = 6;

pub(super) fn draw_report_modal(
    context: &egui::Context,
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
    let mut title = settings.report_title.clone();
    let mut body = settings.report_body.clone();
    crate::kit::dialog_with(context, "fluxa-report-problem", &["wide"], |ui| {
        ui.label(
            RichText::new(localized("settings.report.title", language))
                .size(metrics.text.subtitle)
                .strong()
                .color(text_primary()),
        );
        ui.add_space(6.0);
        ui.label(
            RichText::new(localized("settings.report.description", language))
                .size(metrics.text.meta)
                .color(metrics.text_secondary),
        );
        ui.add_space(16.0);
        let field_width = ui.available_width();
        let title_field = components::text_input(
            ui,
            &mut title,
            &localized("settings.report.title_hint", language),
            field_width,
            components::InputKind::Text,
            metrics,
        );
        title = title.chars().take(REPORT_TITLE_LIMIT).collect();
        layout.focusable.push((NODE_REPORT_TITLE, title_field.rect));
        if title_field.changed() {
            layout.text_input = Some(title.clone());
            layout.text_input_node = Some(NODE_REPORT_TITLE);
        }
        ui.add_space(10.0);
        let body_field = components::text_area(
            ui,
            &mut body,
            &localized("settings.report.body_hint", language),
            field_width,
            REPORT_BODY_LINES,
            REPORT_BODY_LIMIT,
            metrics,
        );
        layout.focusable.push((NODE_REPORT_BODY, body_field.rect));
        if body_field.changed() {
            layout.text_input = Some(body.clone());
            layout.text_input_node = Some(NODE_REPORT_BODY);
        }
        ui.add_space(6.0);
        ui.label(
            RichText::new(format!("{}/{}", body.chars().count(), REPORT_BODY_LIMIT))
                .size(metrics.text.meta)
                .color(metrics.text_muted),
        );
        ui.add_space(14.0);
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 8.0;
            let half = (field_width - 8.0) * 0.5;
            let cancel = components::button_with_text_size(
                ui,
                &localized("common.cancel", language),
                half,
                crate::metrics::button_height(),
                components::ButtonKind::Secondary,
                crate::style::size::BUTTON_TEXT,
                metrics,
            );
            components::focus_action(layout, NODE_REPORT_CANCEL, &cancel);
            let ready = !title.trim().is_empty() && !body.trim().is_empty();
            let send = components::primary_button(
                ui,
                &localized("settings.report.send", language),
                half,
                ready,
            );
            layout.focusable.push((NODE_REPORT_SEND, send.rect));
            if ready && send.clicked() {
                layout.activated = Some(NODE_REPORT_SEND);
            }
        });
        if ui.input(|input| input.key_pressed(egui::Key::Escape)) {
            layout.activated = Some(NODE_REPORT_CANCEL);
        }
    });
}
