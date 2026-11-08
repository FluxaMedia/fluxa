use super::*;

pub(super) fn watch_ticket_width(rect_width: f32, metrics: UiMetrics) -> f32 {
    rect_width.min(metrics.settings_watch_ticket_max)
}

pub(super) fn draw_watch_party(
    context: &egui::Context,
    settings: &SettingsModel,
    language: &str,
    rect: Rect,
    metrics: UiMetrics,
    layout: &mut HomeLayout,
    assets: &mut impl HomeAssets,
) {
    let top = page_top(rect, metrics);
    let width = watch_ticket_width(rect.width(), metrics);
    let (origin, hint_top) = flow_area(context, "fluxa-watch-page", rect, top, |flow| {
        flow.add_space(6.0);
        let origin = flow
            .allocate_space(Vec2::new(width, components::ticket_height(width)))
            .1
            .min;
        flow.add_space(18.0);
        (origin, flow.next_widget_position().y)
    });
    let mut code = settings.watch_code.clone();
    let ticket = &settings.watch_ticket;
    components::placed_front(context, Id::new("fluxa-watch-ticket"), origin, |ui| {
        let output =
            components::watch_ticket(ui, ticket, &mut code, width, language, assets, metrics);
        if let Some(input) = output.code {
            layout
                .focusable
                .push((NODE_SETTINGS_WATCH_CODE, input.rect));
            layout
                .focusable
                .push((NODE_SETTINGS_WATCH_JOIN, input.button));
            if input.response.changed() {
                layout.text_input = Some(code.clone());
                layout.text_input_node = Some(NODE_SETTINGS_WATCH_CODE);
            }
        }
        layout.activated = match output.action {
            Some(components::TicketAction::Join) => Some(NODE_SETTINGS_WATCH_JOIN),
            None => layout.activated,
        };
    });
    draw_text(
        &context.layer_painter(egui::LayerId::background()),
        Pos2::new(rect.left() + 4.0, hint_top),
        Align2::LEFT_TOP,
        localized("settings.watch_party.host_help", language),
        crate::fonts::regular(metrics.screen_card_subtitle_size + 2.0),
        metrics.text_muted,
        Some(width),
    );
}
