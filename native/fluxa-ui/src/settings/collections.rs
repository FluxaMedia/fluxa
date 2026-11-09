use super::account::account_group;
use super::badges::{account_divider_at, pills_right};
use super::page::padded;
use super::*;
use crate::dp::Dp;
use crate::flex::{self, Align, Insets};

const LABEL: usize = 1;
const FIELD: usize = 2;
const TITLE: usize = 3;
const DETAIL: usize = 4;
const TRACK: usize = 5;
const UP: usize = 6;
const DOWN: usize = 7;

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

pub(super) enum Line {
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
    Switch {
        label: String,
        node: u64,
        on: bool,
    },
    Section {
        label: String,
        node: u64,
        on: Option<bool>,
        up: Option<u64>,
        down: Option<u64>,
    },
}

pub(super) struct Group {
    pub(super) title: String,
    pub(super) lines: Vec<Line>,
}

fn provider_index(provider: &str) -> usize {
    match provider {
        "tmdb" => 1,
        "trakt" => 2,
        _ => 0,
    }
}

pub(super) fn toggle(label: &str, node: u64, on: bool, language: &str) -> Line {
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

fn line_height(line: &Line, metrics: UiMetrics) -> f32 {
    let compact = metrics.compact;
    match line {
        Line::Action { pills, .. } if compact && pills.len() > 1 => {
            metrics.settings_row + metrics.settings_action
        }
        _ => metrics.settings_row,
    }
}

pub(super) fn draw_collections(
    context: &crate::gfx::Context,
    settings: &SettingsModel,
    language: &str,
    rect: Rect,
    top: f32,
    metrics: UiMetrics,
    layout: &mut HomeLayout,
) {
    let groups = groups(settings, language);
    flow_area(context, "fluxa-settings-collections", rect, top, |flow| {
        if let Some(status) = &settings.collection_status {
            let block = flow
                .allocate_space(Vec2::new(rect.width(), metrics.settings_status_height))
                .1;
            let painter = flow.painter().clone();
            let font = crate::fonts::regular(metrics.screen_card_subtitle_size + Dp::D2.px());
            flex::paint_text(
                &painter,
                padded(
                    block,
                    Insets {
                        left: metrics.settings_row_inset,
                        bottom: (block.height() - flex::text_size(&painter, status, &font).y)
                            .max(0.0),
                        ..Insets::default()
                    },
                ),
                status,
                &font,
                metrics.text_muted,
                Align2::LEFT_TOP,
            );
        }
        draw_group_flow(context, settings, &groups, flow, metrics, layout);
    });
}

pub(super) fn draw_groups(
    context: &crate::gfx::Context,
    settings: &SettingsModel,
    groups: &[Group],
    rect: Rect,
    top: f32,
    metrics: UiMetrics,
    layout: &mut HomeLayout,
) {
    flow_area(context, "fluxa-settings-groups", rect, top, |flow| {
        draw_group_flow(context, settings, groups, flow, metrics, layout);
    });
}

fn draw_group_flow(
    context: &crate::gfx::Context,
    settings: &SettingsModel,
    groups: &[Group],
    flow: &mut crate::gfx::Ui,
    metrics: UiMetrics,
    layout: &mut HomeLayout,
) {
    let size = metrics.settings_row_label_size_desktop;
    let muted = crate::fonts::regular(metrics.screen_card_subtitle_size + Dp::D2.px());
    for group in groups {
        account_group(flow, &group.title, metrics, |card| {
            let painter = card.painter().clone();
            let width = card.max_rect().width();
            for (index, line) in group.lines.iter().enumerate() {
                let height = line_height(line, metrics);
                let row = padded(
                    card.allocate_space(Vec2::new(width, height)).1,
                    Insets::xy(metrics.settings_row_inset, 0.0),
                );
                if index > 0 {
                    account_divider_at(&painter, row.left(), row.right(), row.top());
                }
                draw_line(
                    context, &painter, layout, line, row, size, &muted, metrics, settings,
                );
            }
        });
        flow.add_space(metrics.settings_group_gap);
    }
}

fn draw_line(
    context: &crate::gfx::Context,
    painter: &crate::gfx::Painter,
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
    let label_font = crate::fonts::regular(size);
    let line_label = |text: &str, slot: Rect| {
        flex::paint_text(
            painter,
            slot,
            text,
            &label_font,
            metrics.text_primary,
            Align2::LEFT_CENTER,
        );
    };
    let input_height = metrics.settings_extended_input_height;
    match line {
        Line::Input { node, label, hint } => {
            let label_width = crate::paint::text_width(painter, label, label_font.clone());
            let cells = flex::row([
                flex::leaf(Vec2::new(label_width, size)).id(LABEL),
                flex::leaf(Vec2::ZERO)
                    .height(input_height)
                    .grow(1.0)
                    .id(FIELD),
            ])
            .gap(Space::S16.px(&metrics))
            .align(Align::Center)
            .solve(row);
            line_label(label, cells.find(LABEL).unwrap_or(row));
            let input = cells.find(FIELD).unwrap_or(row);
            settings_panel_input(context, layout, *node, input, value(*node), hint, metrics);
        }
        Line::Action { node, hint, pills } => {
            let stacked = row.height() > metrics.settings_row;
            let field = padded(
                row,
                Insets {
                    bottom: (row.height() - metrics.settings_row).max(0.0),
                    ..Insets::default()
                },
            );
            let gap = Space::S4.px(&metrics);
            let right = if stacked {
                let buttons = Rect::from_min_max(
                    Pos2::new(row.left(), field.bottom() - gap),
                    Pos2::new(row.right() + gap, row.bottom() - gap),
                );
                pills_right(context, painter, layout, buttons, pills, size - 2.0);
                row.right() + gap
            } else {
                pills_right(context, painter, layout, row, pills, size - 2.0)
            };
            let input = flex::row([flex::leaf(Vec2::ZERO)
                .height(input_height)
                .grow(1.0)
                .id(FIELD)])
            .align(Align::Center)
            .padding(Insets {
                right: (field.right() - right + gap).max(0.0),
                ..Insets::default()
            })
            .solve(field)
            .find(FIELD)
            .unwrap_or(field);
            settings_panel_input(context, layout, *node, input, value(*node), hint, metrics);
        }
        Line::Choice { label, pills } => {
            pills_right(context, painter, layout, row, pills, size - 2.0);
            line_label(label, row);
        }
        Line::Item {
            title,
            detail,
            pills,
        } => {
            let right = pills_right(context, painter, layout, row, pills, size - 2.0);
            let band = padded(
                row,
                Insets {
                    right: (row.right() - right + Space::S12.px(&metrics)).max(0.0),
                    ..Insets::default()
                },
            );
            if detail.is_empty() {
                line_label(title, band);
            } else {
                let lines = flex::column([
                    flex::leaf(Vec2::ZERO).height(Dp::D8.px() * 2.0).id(TITLE),
                    flex::leaf(Vec2::ZERO).height(Dp::D10.px() * 2.0).id(DETAIL),
                ])
                .align(Align::Stretch)
                .padding(Insets {
                    top: band.height() * 0.5 - Dp::D8.px() * 2.0,
                    ..Insets::default()
                })
                .solve(band);
                line_label(title, lines.find(TITLE).unwrap_or(band));
                flex::paint_text(
                    painter,
                    lines.find(DETAIL).unwrap_or(band),
                    detail,
                    muted,
                    metrics.text_muted,
                    Align2::LEFT_CENTER,
                );
            }
        }
        Line::Switch { label, node, on } => {
            layout.focusable.push((*node, row));
            let mut clicked = false;
            components::placed_front(
                context,
                Id::new(("fluxa-settings-switch", *node)),
                row.min,
                |ui| {
                    let (_, response) = ui.allocate_exact_size(row.size(), Sense::click());
                    clicked = response.clicked();
                },
            );
            if clicked {
                layout.activated = Some(*node);
            }
            let cells = flex::row([
                flex::leaf(Vec2::new(0.0, size)).grow(1.0).id(LABEL),
                flex::leaf(Vec2::new(
                    metrics.settings_toggle_width,
                    metrics.settings_toggle_height,
                ))
                .id(TRACK),
            ])
            .gap(Space::S12.px(&metrics))
            .align(Align::Center)
            .padding(Insets {
                right: Dp::D3.px(),
                ..Insets::default()
            })
            .solve(row);
            let track = cells.find(TRACK).unwrap_or(row);
            components::toggle(
                context,
                painter,
                Id::new(("fluxa-settings-switch-track", *node)),
                track,
                *on,
                metrics,
            );
            line_label(label, cells.find(LABEL).unwrap_or(row));
        }
        Line::Section {
            label,
            node,
            on,
            up,
            down,
        } => {
            let button_pad = (Space::S20.px(&metrics) - metrics.settings_action * 0.5).max(0.0);
            let cell = |id: usize| {
                flex::row([flex::leaf(Vec2::splat(metrics.settings_action)).id(id)]).padding(
                    Insets {
                        right: button_pad,
                        ..Insets::default()
                    },
                )
            };
            let hit_width = metrics.settings_toggle_width + Dp::D6.px() * 2.0;
            let tail = if on.is_some() {
                hit_width + Dp::D3.px() - Dp::D6.px() + Space::S8.px(&metrics)
            } else {
                0.0
            };
            let cells = flex::row([
                flex::leaf(Vec2::new(0.0, size)).grow(1.0).id(LABEL),
                cell(UP),
                cell(DOWN),
                flex::leaf(Vec2::ZERO).width(tail),
            ])
            .align(Align::Center)
            .solve(row);
            let track = flex::anchored(
                row,
                Align2::RIGHT_CENTER,
                Vec2::new(
                    metrics.settings_toggle_width,
                    metrics.settings_toggle_height,
                ),
                Dp::D3.px(),
            );
            let hit = track.expand2(Vec2::new(Dp::D6.px(), Dp::D12.px()));
            let mut activated = None;
            if let Some(on) = on {
                layout.focusable.push((*node, hit));
                components::placed_front(
                    context,
                    Id::new(("fluxa-settings-section-toggle", *node)),
                    hit.min,
                    |ui| {
                        let (_, response) = ui.allocate_exact_size(hit.size(), Sense::click());
                        if response.clicked() {
                            activated = Some(*node);
                        }
                    },
                );
                components::toggle(
                    context,
                    painter,
                    Id::new(("fluxa-settings-section-track", *node)),
                    track,
                    *on,
                    metrics,
                );
            }
            for (target, id, is_up) in [(*down, DOWN, false), (*up, UP, true)] {
                let button = cells.find(id).unwrap_or(row);
                let color = if target.is_some() {
                    metrics.text_primary
                } else {
                    metrics.ink(tone::faint())
                };
                let center = button.center();
                let lift = if is_up { -Dp::D3.px() } else { Dp::D3.px() };
                let reach = Dp::D6.px();
                let stroke = crate::gfx::Stroke::new(stroke::icon(), color);
                painter.line_segment(
                    [
                        center + Vec2::new(-reach, -lift),
                        center + Vec2::new(0.0, lift),
                    ],
                    stroke,
                );
                painter.line_segment(
                    [
                        center + Vec2::new(0.0, lift),
                        center + Vec2::new(reach, -lift),
                    ],
                    stroke,
                );
                let Some(target) = target else {
                    continue;
                };
                layout.focusable.push((target, button));
                components::placed_front(
                    context,
                    Id::new(("fluxa-settings-section-move", target)),
                    button.min,
                    |ui| {
                        let (_, response) = ui.allocate_exact_size(button.size(), Sense::click());
                        if response.clicked() {
                            activated = Some(target);
                        }
                    },
                );
            }
            if activated.is_some() {
                layout.activated = activated;
            }
            line_label(
                label,
                padded(
                    cells.find(LABEL).unwrap_or(row),
                    Insets {
                        right: Space::S12.px(&metrics),
                        ..Insets::default()
                    },
                ),
            );
        }
        Line::Note(text) => {
            flex::paint_text(
                painter,
                row,
                text,
                muted,
                metrics.text_muted,
                Align2::LEFT_CENTER,
            );
        }
    }
}
