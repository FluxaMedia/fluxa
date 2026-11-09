use super::*;
use crate::dp::Dp;
use crate::kit::UiKit as _;

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
        flow.gap(Space::S6);
        let origin = flow
            .allocate_space(Vec2::new(width, components::ticket_height(width)))
            .1
            .min;
        flow.gap(Space::S16);
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
    let painter = context.layer_painter(egui::LayerId::background());
    let hint = localized("settings.watch_party.host_help", language);
    let font = crate::fonts::regular(metrics.screen_card_subtitle_size + Dp::D2.px());
    let edge = Space::S4.px(&metrics);
    let slot = super::page::padded(
        Rect::from_min_size(
            Pos2::new(rect.left(), hint_top),
            Vec2::new(
                width + edge,
                crate::flex::text_size(&painter, &hint, &font).y,
            ),
        ),
        crate::flex::Insets {
            left: edge,
            ..crate::flex::Insets::default()
        },
    );
    crate::flex::paint_text(
        &painter,
        slot,
        &hint,
        &font,
        metrics.text_muted,
        Align2::LEFT_TOP,
    );
}
