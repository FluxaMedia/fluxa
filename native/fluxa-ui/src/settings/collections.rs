use super::account::account_group;
use super::badges::{account_divider_at, pills_right};
use super::*;

const ROW: f32 = 52.0;
const STATUS_HEIGHT: f32 = 24.0;
const VIEW_MODES: [&str; 3] = ["TABBED_GRID", "ROWS", "FOLLOW_LAYOUT"];
const SOURCE_LABELS: [[&str; 3]; 3] = [
    [
        "collections.source_addon_id",
        "collections.source_type",
        "collections.source_catalog",
    ],
    [
        "collections.source_tmdb_id",
        "collections.source_media",
        "collections.source_tmdb_kind",
    ],
    [
        "collections.source_trakt_id",
        "collections.source_media",
        "",
    ],
];

pub fn collection_input(node: u64) -> Option<usize> {
    let index = node.checked_sub(NODE_SETTINGS_COLLECTION_INPUT_BASE)? as usize;
    (index < COLLECTION_INPUT_COUNT).then_some(index)
}

pub fn collection_input_label(index: usize) -> &'static str {
    match index {
        0 => "collections.import_hint",
        1 | 2 => "collections.field_name",
        3 => "collections.field_folder_name",
        4 => "collections.field_cover",
        5 => "collections.field_emoji",
        6 => "collections.field_gif",
        _ => "collections.field_source",
    }
}

pub fn collection_view_modes() -> &'static [&'static str] {
    &VIEW_MODES
}

enum Line {
    Input {
        node: u64,
        label: String,
        hint: String,
    },
    Action {
        node: u64,
        hint: String,
        pills: Vec<(u64, String)>,
    },
    Choice {
        label: String,
        pills: Vec<(u64, String)>,
    },
    Item {
        title: String,
        detail: String,
        pills: Vec<(u64, String)>,
    },
    Note(String),
}

struct Group {
    title: String,
    lines: Vec<Line>,
}

fn provider_index(provider: &str) -> usize {
    match provider {
        "tmdb" => 1,
        "trakt" => 2,
        _ => 0,
    }
}

fn toggle(label: &str, node: u64, on: bool, language: &str) -> Line {
    Line::Choice {
        label: localized(label, language),
        pills: vec![(
            node,
            localized(if on { "common.on" } else { "common.off" }, language),
        )],
    }
}

fn groups(settings: &SettingsModel, language: &str) -> Vec<Group> {
    let tr = |key: &str| localized(key, language);
    let mut groups = vec![Group {
        title: tr("collections.group_transfer"),
        lines: vec![Line::Action {
            node: NODE_SETTINGS_COLLECTION_INPUT_BASE,
            hint: tr("collections.import_hint"),
            pills: vec![
                (NODE_SETTINGS_COLLECTION_IMPORT, tr("collections.import")),
                (NODE_SETTINGS_COLLECTION_EXPORT, tr("collections.export")),
            ],
        }],
    }];
    let mut list: Vec<Line> = settings
        .collections
        .iter()
        .enumerate()
        .map(|(index, row)| Line::Item {
            title: row.title.clone(),
            detail: format!("{} {}", row.folders, tr("collections.folders_unit")),
            pills: vec![
                (
                    NODE_SETTINGS_COLLECTION_EDIT_BASE + index as u64,
                    tr("collections.edit"),
                ),
                (
                    NODE_SETTINGS_COLLECTION_DELETE_BASE + index as u64,
                    tr("common.delete"),
                ),
            ],
        })
        .collect();
    if list.is_empty() {
        list.push(Line::Note(tr("collections.none")));
    }
    groups.push(Group {
        title: tr("collections.group_list"),
        lines: list,
    });
    groups.push(Group {
        title: tr("collections.group_new"),
        lines: vec![Line::Action {
            node: NODE_SETTINGS_COLLECTION_INPUT_BASE + 1,
            hint: tr("collections.field_name"),
            pills: vec![(NODE_SETTINGS_COLLECTION_ADD, tr("collections.add"))],
        }],
    });
    let Some(edit) = &settings.collection_edit else {
        return groups;
    };
    let mode = VIEW_MODES
        .iter()
        .position(|mode| mode.eq_ignore_ascii_case(&edit.view_mode))
        .unwrap_or(2);
    groups.push(Group {
        title: tr("collections.group_settings"),
        lines: vec![
            Line::Action {
                node: NODE_SETTINGS_COLLECTION_INPUT_BASE + 2,
                hint: tr("collections.field_name"),
                pills: vec![(NODE_SETTINGS_COLLECTION_RENAME, tr("collections.rename"))],
            },
            Line::Choice {
                label: tr("collections.view_mode"),
                pills: vec![(
                    NODE_SETTINGS_COLLECTION_VIEW_MODE,
                    tr(&format!(
                        "collections.view_{}",
                        VIEW_MODES[mode].to_lowercase()
                    )),
                )],
            },
            toggle(
                "collections.show_all_tab",
                NODE_SETTINGS_COLLECTION_SHOW_ALL,
                edit.show_all_tab,
                language,
            ),
            toggle(
                "collections.pin_to_top",
                NODE_SETTINGS_COLLECTION_PIN,
                edit.pin_to_top,
                language,
            ),
            toggle(
                "collections.focus_glow",
                NODE_SETTINGS_COLLECTION_GLOW,
                edit.focus_glow,
                language,
            ),
            toggle(
                "collections.show_on_home",
                NODE_SETTINGS_COLLECTION_SHOW_HOME,
                edit.show_on_home,
                language,
            ),
        ],
    });
    let mut folders: Vec<Line> = edit
        .folders
        .iter()
        .enumerate()
        .map(|(index, row)| Line::Item {
            title: row.title.clone(),
            detail: format!("{} {}", row.folders, tr("collections.sources_unit")),
            pills: vec![
                (
                    NODE_SETTINGS_FOLDER_EDIT_BASE + index as u64,
                    tr("collections.edit"),
                ),
                (
                    NODE_SETTINGS_FOLDER_DELETE_BASE + index as u64,
                    tr("common.delete"),
                ),
            ],
        })
        .collect();
    if folders.is_empty() {
        folders.push(Line::Note(tr("collections.no_folders")));
    }
    if edit.folder.is_none() {
        folders.push(Line::Choice {
            label: tr("collections.folder_new"),
            pills: vec![
                (NODE_SETTINGS_FOLDER_NEW, tr("collections.add")),
                (NODE_SETTINGS_COLLECTION_CLOSE, tr("common.close")),
            ],
        });
    }
    groups.push(Group {
        title: tr("collections.group_folders"),
        lines: folders,
    });
    let Some(folder) = &edit.folder else {
        return groups;
    };
    let input = |index: usize| Line::Input {
        node: NODE_SETTINGS_COLLECTION_INPUT_BASE + index as u64,
        label: tr(collection_input_label(index)),
        hint: String::new(),
    };
    groups.push(Group {
        title: tr("collections.group_folder"),
        lines: vec![
            input(3),
            input(4),
            input(5),
            input(6),
            Line::Choice {
                label: tr("collections.shape"),
                pills: vec![(
                    NODE_SETTINGS_FOLDER_SHAPE,
                    tr(&format!("collections.shape_{}", folder.shape)),
                )],
            },
            toggle(
                "collections.hide_title",
                NODE_SETTINGS_FOLDER_HIDE_TITLE,
                folder.hide_title,
                language,
            ),
            Line::Choice {
                label: String::new(),
                pills: vec![
                    (NODE_SETTINGS_FOLDER_CANCEL, tr("common.cancel")),
                    (NODE_SETTINGS_FOLDER_SAVE, tr("collections.folder_save")),
                ],
            },
        ],
    });
    let provider = provider_index(&folder.provider);
    let mut sources: Vec<Line> = folder
        .sources
        .iter()
        .enumerate()
        .map(|(index, label)| Line::Item {
            title: label.clone(),
            detail: String::new(),
            pills: vec![(
                NODE_SETTINGS_SOURCE_REMOVE_BASE + index as u64,
                tr("common.remove"),
            )],
        })
        .collect();
    if sources.is_empty() {
        sources.push(Line::Note(tr("collections.no_sources")));
    }
    sources.push(Line::Choice {
        label: tr("collections.source_provider"),
        pills: vec![(
            NODE_SETTINGS_SOURCE_PROVIDER,
            tr(&format!("collections.provider_{}", folder.provider)),
        )],
    });
    for (offset, label) in SOURCE_LABELS[provider].iter().enumerate() {
        if label.is_empty() {
            continue;
        }
        sources.push(Line::Input {
            node: NODE_SETTINGS_COLLECTION_INPUT_BASE + 7 + offset as u64,
            label: tr(label),
            hint: String::new(),
        });
    }
    sources.push(Line::Choice {
        label: String::new(),
        pills: vec![(NODE_SETTINGS_SOURCE_ADD, tr("collections.source_add"))],
    });
    groups.push(Group {
        title: tr("collections.group_sources"),
        lines: sources,
    });
    groups
}

fn group_height(group: &Group) -> f32 {
    APPEARANCE_GROUP_HEADING_HEIGHT + group.lines.len() as f32 * ROW + 12.0 + APPEARANCE_GROUP_GAP
}

pub(super) fn collections_height(settings: &SettingsModel, language: &str) -> f32 {
    let status = if settings.collection_status.is_some() {
        STATUS_HEIGHT
    } else {
        0.0
    };
    groups(settings, language)
        .iter()
        .map(group_height)
        .sum::<f32>()
        + status
}

pub(super) fn draw_collections(
    context: &egui::Context,
    settings: &SettingsModel,
    language: &str,
    rect: Rect,
    mut top: f32,
    metrics: UiMetrics,
    layout: &mut HomeLayout,
) {
    let painter = context.layer_painter(egui::LayerId::background());
    let size = metrics.settings_row_label_size_desktop;
    let muted = crate::fonts::regular(metrics.screen_card_subtitle_size + 2.0);
    if let Some(status) = &settings.collection_status {
        painter.text(
            Pos2::new(rect.left() + metrics.settings_row_inset, top),
            Align2::LEFT_TOP,
            truncate_to_width(&painter, status, &muted, rect.width()),
            muted.clone(),
            metrics.text_muted,
        );
        top += STATUS_HEIGHT;
    }
    for group in groups(settings, language) {
        let card = account_group(
            &painter,
            rect,
            top,
            group.lines.len() as f32 * ROW + 12.0,
            &group.title,
            false,
            metrics,
        );
        for (index, line) in group.lines.iter().enumerate() {
            let row = Rect::from_min_size(
                card.left_top() + Vec2::new(metrics.settings_row_inset, 6.0 + index as f32 * ROW),
                Vec2::new(card.width() - metrics.settings_row_inset * 2.0, ROW),
            );
            if index > 0 {
                account_divider_at(&painter, row.left(), row.right(), row.top());
            }
            draw_line(
                context, &painter, layout, line, row, size, &muted, metrics, settings,
            );
        }
        top = card.bottom() + APPEARANCE_GROUP_GAP;
    }
}

fn draw_line(
    context: &egui::Context,
    painter: &egui::Painter,
    layout: &mut HomeLayout,
    line: &Line,
    row: Rect,
    size: f32,
    muted: &FontId,
    metrics: UiMetrics,
    settings: &SettingsModel,
) {
    let value = |node: u64| {
        collection_input(node)
            .map(|index| settings.collection_fields[index].as_str())
            .unwrap_or_default()
    };
    match line {
        Line::Input { node, label, hint } => {
            painter.text(
                Pos2::new(row.left(), row.center().y),
                Align2::LEFT_CENTER,
                label,
                crate::fonts::regular(size),
                metrics.text_primary,
            );
            let input = Rect::from_min_max(
                Pos2::new(row.left() + row.width() * 0.4, row.center().y - 18.0),
                Pos2::new(row.right(), row.center().y + 18.0),
            );
            settings_panel_input(context, layout, *node, input, value(*node), hint, metrics);
        }
        Line::Action { node, hint, pills } => {
            let right = pills_right(context, painter, layout, row, pills, size);
            settings_panel_input(
                context,
                layout,
                *node,
                Rect::from_min_max(
                    Pos2::new(row.left(), row.center().y - 18.0),
                    Pos2::new(right - 4.0, row.center().y + 18.0),
                ),
                value(*node),
                hint,
                metrics,
            );
        }
        Line::Choice { label, pills } => {
            pills_right(context, painter, layout, row, pills, size);
            painter.text(
                Pos2::new(row.left(), row.center().y),
                Align2::LEFT_CENTER,
                label,
                crate::fonts::regular(size),
                metrics.text_primary,
            );
        }
        Line::Item {
            title,
            detail,
            pills,
        } => {
            let right = pills_right(context, painter, layout, row, pills, size);
            let font = crate::fonts::regular(size);
            let offset = if detail.is_empty() { 0.0 } else { 9.0 };
            painter.text(
                Pos2::new(row.left(), row.center().y - offset),
                Align2::LEFT_CENTER,
                truncate_to_width(painter, title, &font, right - row.left() - 12.0),
                font,
                metrics.text_primary,
            );
            if !detail.is_empty() {
                painter.text(
                    Pos2::new(row.left(), row.center().y + 11.0),
                    Align2::LEFT_CENTER,
                    detail,
                    muted.clone(),
                    metrics.text_muted,
                );
            }
        }
        Line::Note(text) => {
            painter.text(
                Pos2::new(row.left(), row.center().y),
                Align2::LEFT_CENTER,
                text,
                muted.clone(),
                metrics.text_muted,
            );
        }
    }
}
