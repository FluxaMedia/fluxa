use super::collections::{Group, Line, draw_groups, groups_height};
use super::*;

fn groups(settings: &SettingsModel, language: &str) -> Vec<Group> {
    let tr = |key: &str| localized(key, language);
    let count = settings.detail_sections.len();
    let mut lines = vec![Line::Note(tr("settings.detail_sections_hint"))];
    for (index, row) in settings.detail_sections.iter().enumerate() {
        lines.push(Line::Section {
            label: tr(row.section.title_key()),
            node: NODE_SETTINGS_DETAIL_SECTION_BASE + row.section as u64,
            on: row.enabled,
            up: (index > 0).then_some(NODE_SETTINGS_DETAIL_SECTION_UP_BASE + row.section as u64),
            down: (index + 1 < count)
                .then_some(NODE_SETTINGS_DETAIL_SECTION_DOWN_BASE + row.section as u64),
        });
    }
    vec![Group {
        title: tr("settings.detail_sections"),
        lines,
    }]
}

pub(super) fn detail_sections_height(
    settings: &SettingsModel,
    language: &str,
    compact: bool,
) -> f32 {
    groups_height(&groups(settings, language), compact)
}

pub(super) fn draw_detail_sections(
    context: &egui::Context,
    settings: &SettingsModel,
    language: &str,
    rect: Rect,
    top: f32,
    metrics: UiMetrics,
    layout: &mut HomeLayout,
) {
    draw_groups(
        context,
        settings,
        &groups(settings, language),
        rect,
        top,
        metrics,
        layout,
    );
}
