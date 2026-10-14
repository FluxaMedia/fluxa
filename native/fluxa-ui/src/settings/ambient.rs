use super::collections::{Group, Line, draw_groups, groups_height};
use super::*;

fn groups(settings: &SettingsModel, language: &str) -> Vec<Group> {
    let tr = |key: &str| localized(key, language);
    let selected = settings
        .ambient_catalogs
        .iter()
        .filter(|row| row.selected)
        .count();
    let mut lines = vec![
        Line::Note(
            tr("ambient.catalogs_count")
                .replace("%1", &selected.to_string())
                .replace("%2", &settings.ambient_limit.to_string()),
        ),
        Line::Switch {
            label: tr("ambient.same_as_hero"),
            node: NODE_SETTINGS_AMBIENT_SAME_AS_HERO,
            on: selected == 0,
        },
    ];
    for (index, row) in settings.ambient_catalogs.iter().enumerate() {
        lines.push(Line::Switch {
            label: row.name.clone(),
            node: NODE_SETTINGS_AMBIENT_CATALOG_BASE + index as u64,
            on: row.selected,
        });
    }
    vec![Group {
        title: tr("ambient.catalogs"),
        lines,
    }]
}

pub(super) fn ambient_height(settings: &SettingsModel, language: &str, compact: bool) -> f32 {
    groups_height(&groups(settings, language), compact)
}

pub(super) fn draw_ambient_settings(
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
