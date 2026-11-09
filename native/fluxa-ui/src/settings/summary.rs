use super::*;

pub(super) fn draw_summary(
    context: &egui::Context,
    settings: &SettingsModel,
    language: &str,
    rect: Rect,
    metrics: UiMetrics,
) {
    let top = rect.top() + metrics.settings_appearance_header_height;
    flow_area(context, "fluxa-summary-page", rect, top, |flow| {
        if settings.playback_summary.is_empty() {
            account_group(
                flow,
                &localized("settings.summary_title", language),
                metrics,
                |card| {
                    let painter = card.painter().clone();
                    let row = account_row(card, &painter, metrics, false);
                    crate::flex::paint_text(
                        &painter,
                        super::page::padded(
                            row,
                            crate::flex::Insets::xy(Space::S4.px(&metrics), 0.0),
                        ),
                        &localized("settings.summary_empty", language),
                        &crate::fonts::regular(metrics.settings_row_value_size_desktop),
                        metrics.text_secondary,
                        Align2::LEFT_CENTER,
                    );
                },
            );
            return;
        }
        for section in &settings.playback_summary {
            let rows: Vec<(&str, &str)> = section
                .rows
                .iter()
                .map(|(label, value)| (label.as_str(), value.as_str()))
                .collect();
            draw_binding_group(
                flow,
                &localized(&section.title, language),
                &rows,
                language,
                metrics,
            );
            flow.add_space(metrics.settings_group_gap);
        }
    });
}
