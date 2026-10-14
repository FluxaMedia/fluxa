use super::collections::{Group, Line, draw_groups, groups_height, toggle};
use super::*;

fn groups(settings: &SettingsModel, language: &str) -> Vec<Group> {
    let tr = |key: &str| localized(key, language);
    let selected = settings
        .ambient_catalogs
        .iter()
        .filter(|row| row.selected)
        .count();
    let mut catalogs = vec![
        Line::Note(
            tr("ambient.catalogs_count")
                .replace("%1", &selected.to_string())
                .replace("%2", &settings.ambient_limit.to_string()),
        ),
        toggle(
            "ambient.same_as_hero",
            NODE_SETTINGS_AMBIENT_SAME_AS_HERO,
            selected == 0,
            language,
        ),
    ];
    for (index, row) in settings.ambient_catalogs.iter().enumerate() {
        catalogs.push(Line::Choice {
            label: row.name.clone(),
            pills: vec![(
                NODE_SETTINGS_AMBIENT_CATALOG_BASE + index as u64,
                tr(if row.selected {
                    "common.on"
                } else {
                    "common.off"
                }),
            )],
        });
    }
    vec![
        Group {
            title: tr("settings.section.ambient_mode"),
            lines: vec![
                toggle(
                    "ambient.enabled",
                    NODE_SETTINGS_AMBIENT_ENABLED,
                    settings.bool_value("ambientModeEnabled"),
                    language,
                ),
                Line::Choice {
                    label: tr("ambient.start_after"),
                    pills: vec![(
                        NODE_SETTINGS_AMBIENT_TIMEOUT,
                        tr("ambient.minutes").replace("%s", &settings.ambient_minutes.to_string()),
                    )],
                },
            ],
        },
        Group {
            title: tr("ambient.catalogs"),
            lines: catalogs,
        },
    ]
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
