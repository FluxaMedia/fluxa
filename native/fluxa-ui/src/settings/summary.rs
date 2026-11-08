use super::about::page_flow;
use super::*;

pub(super) fn summary_height(sections: &[crate::StatsSection], metrics: UiMetrics) -> f32 {
    let groups = sections.len().max(1);
    let rows: f32 = if sections.is_empty() {
        settings_group_card_height(1, metrics)
    } else {
        sections
            .iter()
            .map(|section| settings_group_card_height(section.rows.len(), metrics))
            .sum()
    };
    rows + (metrics.settings_group_heading_height + metrics.settings_group_gap) * groups as f32
}

fn group_block_height(rows: usize, metrics: UiMetrics) -> f32 {
    metrics.settings_group_heading_height + settings_group_card_height(rows, metrics)
}

pub(super) fn draw_summary(
    painter: &egui::Painter,
    settings: &SettingsModel,
    language: &str,
    rect: Rect,
    metrics: UiMetrics,
) {
    let top = rect.top() + metrics.settings_appearance_header_height;
    page_flow(painter.ctx(), "fluxa-summary-page", rect, top, |flow| {
        if settings.playback_summary.is_empty() {
            let block = flow
                .allocate_space(Vec2::new(rect.width(), group_block_height(1, metrics)))
                .1;
            let card = account_group(
                painter,
                rect,
                block.top(),
                settings_group_card_height(1, metrics),
                &localized("settings.summary_title", language),
                metrics,
            );
            draw_text(
                painter,
                account_row(card, 0, metrics).left_center() + Vec2::new(4.0, 0.0),
                Align2::LEFT_CENTER,
                localized("settings.summary_empty", language),
                crate::fonts::regular(metrics.settings_row_value_size_desktop),
                metrics.text_secondary,
                None,
            );
            return;
        }
        for section in &settings.playback_summary {
            let rows: Vec<(&str, &str)> = section
                .rows
                .iter()
                .map(|(label, value)| (label.as_str(), value.as_str()))
                .collect();
            let block = flow
                .allocate_space(Vec2::new(
                    rect.width(),
                    group_block_height(rows.len(), metrics),
                ))
                .1;
            draw_binding_group(
                painter,
                rect,
                block.top(),
                &localized(&section.title, language),
                &rows,
                language,
                metrics,
            );
            flow.add_space(metrics.settings_group_gap);
        }
    });
}
