use super::collections::{Group, Line, draw_groups, groups_height};
use super::*;

fn groups(settings: &SettingsModel, language: &str) -> Vec<Group> {
    let tr = |key: &str| localized(key, language);
    let mut lines = vec![Line::Note(tr("settings.hero_catalogs_default_note"))];
    for (index, row) in settings.hero_catalogs.iter().enumerate() {
        lines.push(Line::Switch {
            label: row.name.clone(),
            node: NODE_SETTINGS_HERO_CATALOG_BASE + index as u64,
            on: row.selected,
        });
    }
    vec![Group {
        title: tr("settings.hero_catalogs"),
        lines,
    }]
}

pub(super) fn hero_height(settings: &SettingsModel, language: &str, compact: bool) -> f32 {
    groups_height(&groups(settings, language), compact)
}

pub(super) fn draw_hero_settings(
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
