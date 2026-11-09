use super::account::{account_group, account_row};
use super::page::{padded, stacked_lines};
use super::*;
use crate::dp::Dp;
use crate::draw_text;
use crate::flex::{self, Align, Insets};

const LABEL: usize = 1;
const FIELD: usize = 2;
const PATTERN: usize = 3;
const ARROW: usize = 4;
const REPLACEMENT: usize = 5;

const FORM_FIELDS: usize = 7;
const LAYOUT_FIELDS: usize = 9;
const FIELD_LABELS: [&str; BADGE_INPUT_COUNT] = [
    "settings.badges_import_hint",
    "settings.badges_field_name",
    "settings.badges_field_pattern",
    "settings.badges_field_color",
    "settings.badges_field_text_color",
    "settings.badges_field_border_color",
    "settings.badges_field_image",
    "settings.formatter_field_pattern",
    "settings.formatter_field_replacement",
    "settings.layout_field_name",
    "settings.layout_field_detail",
];

pub fn badge_input(node: u64) -> Option<usize> {
    if let Some(offset) = node.checked_sub(NODE_SETTINGS_LAYOUT_INPUT_BASE) {
        let index = LAYOUT_FIELDS + offset as usize;
        return (index < BADGE_INPUT_COUNT).then_some(index);
    }
    let index = node.checked_sub(NODE_SETTINGS_BADGE_INPUT_BASE)? as usize;
    (index < LAYOUT_FIELDS).then_some(index)
}

fn input_node(index: usize) -> u64 {
    if index >= LAYOUT_FIELDS {
        NODE_SETTINGS_LAYOUT_INPUT_BASE + (index - LAYOUT_FIELDS) as u64
    } else {
        NODE_SETTINGS_BADGE_INPUT_BASE + index as u64
    }
}

pub fn badge_input_label(index: usize) -> &'static str {
    FIELD_LABELS[index]
}

fn inset_row(card: &mut egui::Ui, height: f32, metrics: UiMetrics) -> Rect {
    let width = card.max_rect().width();
    padded(
        card.allocate_space(Vec2::new(width, height)).1,
        Insets::xy(metrics.settings_row_inset, 0.0),
    )
}

fn field_label(
    painter: &egui::Painter,
    row: Rect,
    index: usize,
    language: &str,
    size: f32,
    metrics: UiMetrics,
) {
    flex::paint_text(
        painter,
        row,
        &localized(FIELD_LABELS[index], language),
        &crate::fonts::regular(size),
        metrics.text_primary,
        Align2::LEFT_CENTER,
    );
}

fn trimmed(row: Rect, right: f32, gap: f32) -> Rect {
    padded(
        row,
        Insets {
            right: (row.right() - right + gap).max(0.0),
            ..Insets::default()
        },
    )
}

fn form_row(
    card: &mut egui::Ui,
    painter: &egui::Painter,
    metrics: UiMetrics,
    divided: bool,
) -> Rect {
    let row = inset_row(card, metrics.settings_form_row, metrics);
    if divided {
        account_divider_at(painter, row.left(), row.right(), row.top());
    }
    row
}

fn form_input(row: Rect, metrics: UiMetrics, start: f32) -> Rect {
    flex::row([
        flex::slot(row.width() * start),
        flex::leaf(Vec2::ZERO)
            .height(metrics.screen_control_height)
            .grow(1.0)
            .id(FIELD),
    ])
    .align(Align::Center)
    .solve(row)
    .find(FIELD)
    .unwrap_or(row)
}

pub(super) fn pills_right(
    context: &egui::Context,
    painter: &egui::Painter,
    layout: &mut HomeLayout,
    row: Rect,
    pills: &[(u64, String)],
    size: f32,
) -> f32 {
    let metrics = crate::metrics::active();
    let widths: Vec<f32> = pills
        .iter()
        .map(|(_, label)| pill_text_width(painter, label, size, metrics))
        .collect();
    let height = metrics
        .screen_control_height
        .min(row.height() - Space::S4.px(&metrics));
    let area = Rect::from_min_max(
        row.min,
        Pos2::new(row.right() - Space::S4.px(&metrics), row.bottom()),
    );
    let slots = crate::flex::row(widths.iter().map(|width| crate::flex::slot(*width)))
        .gap(Space::S8.px(&metrics))
        .align(crate::flex::Align::Stretch)
        .justify(crate::flex::Justify::End)
        .solve(area)
        .children
        .into_iter()
        .map(|slot| slot.rect)
        .collect::<Vec<_>>();
    for ((id, label), slot) in pills.iter().zip(&slots) {
        let button = Rect::from_center_size(slot.center(), Vec2::new(slot.width(), height));
        pill_button(context, layout, *id, button, label, size);
    }
    slots
        .first()
        .map_or(area.right(), |slot| slot.left() - Space::S8.px(&metrics))
}

pub(super) fn draw_badges(
    context: &egui::Context,
    settings: &SettingsModel,
    assets: &mut impl HomeAssets,
    language: &str,
    flow: &mut egui::Ui,
    metrics: UiMetrics,
    layout: &mut HomeLayout,
) {
    let size = metrics.settings_row_label_size_desktop;
    let muted = crate::fonts::regular(metrics.screen_card_subtitle_size + Dp::D2.px());
    let gap = metrics.settings_group_gap;

    account_group(
        flow,
        &localized("settings.group.badges_import", language),
        metrics,
        |card| {
            let painter = card.painter().clone();
            let row = inset_row(card, metrics.screen_control_height, metrics);
            let right = pills_right(
                context,
                &painter,
                layout,
                row,
                &[(
                    NODE_SETTINGS_BADGE_IMPORT,
                    localized("settings.badges_import", language),
                )],
                size,
            );
            settings_panel_input(
                context,
                layout,
                NODE_SETTINGS_BADGE_INPUT_BASE,
                trimmed(row, right, Space::S4.px(&metrics)),
                &settings.badge_fields[0],
                localized(FIELD_LABELS[0], language),
                metrics,
            );
            if let Some(status) = &settings.badge_status {
                card.add_space(Space::S8.px(&metrics));
                let status_row = inset_row(card, metrics.settings_status_height, metrics);
                draw_text(
                    &painter,
                    status_row.left_top(),
                    Align2::LEFT_TOP,
                    status,
                    muted.clone(),
                    metrics.text_muted,
                    Some(status_row.width()),
                );
            }
        },
    );
    flow.add_space(gap);

    account_group(
        flow,
        &localized("settings.group.badges_packs", language),
        metrics,
        |card| {
            let painter = card.painter().clone();
            if settings.badge_packs.is_empty() {
                let row = account_row(card, &painter, metrics, false);
                flex::paint_text(
                    &painter,
                    row,
                    &localized("settings.badges_no_packs", language),
                    &muted,
                    metrics.text_muted,
                    Align2::LEFT_CENTER,
                );
            }
            for (index, pack) in settings.badge_packs.iter().enumerate() {
                let row = account_row(card, &painter, metrics, index > 0);
                let toggle = if pack.active {
                    "settings.badges_disable"
                } else {
                    "settings.badges_enable"
                };
                let right = pills_right(
                    context,
                    &painter,
                    layout,
                    row,
                    &[
                        (
                            NODE_SETTINGS_BADGE_PACK_TOGGLE_BASE + index as u64,
                            localized(toggle, language),
                        ),
                        (
                            NODE_SETTINGS_BADGE_PACK_REMOVE_BASE + index as u64,
                            localized("settings.badges_remove", language),
                        ),
                    ],
                    size,
                );
                let (name, count) = stacked_lines(
                    trimmed(row, right, Space::S12.px(&metrics)),
                    Space::S10.px(&metrics),
                    Space::S12.px(&metrics),
                );
                flex::paint_text(
                    &painter,
                    name,
                    &pack.name,
                    &crate::fonts::regular(size),
                    if pack.active {
                        metrics.text_primary
                    } else {
                        metrics.text_muted
                    },
                    Align2::LEFT_CENTER,
                );
                flex::paint_text(
                    &painter,
                    count,
                    &format!(
                        "{} {}",
                        pack.count,
                        localized("settings.badges_count_unit", language)
                    ),
                    &muted,
                    metrics.text_muted,
                    Align2::LEFT_CENTER,
                );
            }
        },
    );
    flow.add_space(gap);

    account_group(
        flow,
        &localized("settings.group.badges_player", language),
        metrics,
        |card| {
            let painter = card.painter().clone();
            for (index, choice) in settings.player_badge_choices.iter().enumerate() {
                let row = account_row(card, &painter, metrics, index > 0);
                let label = if choice.selected {
                    "settings.badges_player_selected"
                } else {
                    "settings.badges_player_use"
                };
                let right = pills_right(
                    context,
                    &painter,
                    layout,
                    row,
                    &[(
                        NODE_SETTINGS_BADGE_PLAYER_BASE + index as u64,
                        localized(label, language),
                    )],
                    size,
                );
                flex::paint_text(
                    &painter,
                    trimmed(row, right, Space::S12.px(&metrics)),
                    &choice.name,
                    &crate::fonts::regular(size),
                    if choice.selected {
                        metrics.text_primary
                    } else {
                        metrics.text_muted
                    },
                    Align2::LEFT_CENTER,
                );
            }
        },
    );
    flow.add_space(gap);

    account_group(
        flow,
        &localized("settings.group.badges_custom", language),
        metrics,
        |card| {
            let painter = card.painter().clone();
            for index in 1..FORM_FIELDS {
                let row = form_row(card, &painter, metrics, index > 1);
                field_label(&painter, row, index, language, size, metrics);
                let hint = if (3..=5).contains(&index) {
                    "#RRGGBB".to_owned()
                } else {
                    String::new()
                };
                settings_panel_input(
                    context,
                    layout,
                    input_node(index),
                    form_input(row, metrics, 0.4),
                    &settings.badge_fields[index],
                    hint,
                    metrics,
                );
            }
            let style_row = form_row(card, &painter, metrics, true);
            draw_text(
                &painter,
                Pos2::new(style_row.left(), style_row.center().y),
                Align2::LEFT_CENTER,
                localized("settings.badges_field_style", language),
                crate::fonts::regular(size),
                metrics.text_primary,
                None,
            );
            let style_key = if settings.badge_outline {
                "settings.option.outline"
            } else {
                "settings.option.filled"
            };
            pills_right(
                context,
                &painter,
                layout,
                style_row,
                &[(NODE_SETTINGS_BADGE_STYLE, localized(style_key, language))],
                size,
            );
            let action_row = form_row(card, &painter, metrics, true);
            let preview = &settings.badge_preview;
            let font_size = metrics.screen_card_subtitle_size + Space::S2.px(&metrics);
            let (galley, chip_size) = components::badge_chip_layout(&painter, preview, font_size);
            let chip = flex::anchored(action_row, Align2::LEFT_CENTER, chip_size, 0.0);
            components::paint_badge_chip(&painter, chip, galley, preview, assets);
            pills_right(
                context,
                &painter,
                layout,
                action_row,
                &[
                    (
                        NODE_SETTINGS_BADGE_CLEAR,
                        localized("settings.badges_clear", language),
                    ),
                    (
                        NODE_SETTINGS_BADGE_SAVE,
                        localized("settings.badges_save", language),
                    ),
                ],
                size,
            );
        },
    );
    flow.add_space(gap);

    if !settings.badge_custom.is_empty() {
        draw_custom_list(context, settings, assets, language, flow, metrics, layout);
        flow.add_space(gap);
    }
    draw_layout(context, settings, language, flow, metrics, layout);
    flow.add_space(gap);
    draw_formatter(context, settings, assets, language, flow, metrics, layout);
}

fn draw_custom_list(
    context: &egui::Context,
    settings: &SettingsModel,
    assets: &mut impl HomeAssets,
    language: &str,
    flow: &mut egui::Ui,
    metrics: UiMetrics,
    layout: &mut HomeLayout,
) {
    let size = metrics.settings_row_label_size_desktop;
    let font_size = metrics.screen_card_subtitle_size + Space::S2.px(&metrics);
    account_group(
        flow,
        &localized("settings.group.badges_yours", language),
        metrics,
        |card| {
            let painter = card.painter().clone();
            for (index, custom) in settings.badge_custom.iter().enumerate() {
                let row = account_row(card, &painter, metrics, index > 0);
                let toggle = if custom.enabled {
                    "settings.badges_disable"
                } else {
                    "settings.badges_enable"
                };
                pills_right(
                    context,
                    &painter,
                    layout,
                    row,
                    &[
                        (
                            NODE_SETTINGS_BADGE_EDIT_BASE + index as u64,
                            localized("settings.badges_edit", language),
                        ),
                        (
                            NODE_SETTINGS_BADGE_TOGGLE_BASE + index as u64,
                            localized(toggle, language),
                        ),
                        (
                            NODE_SETTINGS_BADGE_REMOVE_BASE + index as u64,
                            localized("settings.badges_remove", language),
                        ),
                    ],
                    size,
                );
                let (galley, chip_size) =
                    components::badge_chip_layout(&painter, &custom.badge, font_size);
                let chip = flex::anchored(row, Align2::LEFT_CENTER, chip_size, 0.0);
                components::paint_badge_chip(&painter, chip, galley, &custom.badge, assets);
                if !custom.enabled {
                    painter.rect_filled(
                        chip.expand(Space::S2.px(&metrics)),
                        chip.height() * 0.5,
                        metrics.surface.gamma_multiply(0.7),
                    );
                }
            }
        },
    );
}

fn draw_layout(
    context: &egui::Context,
    settings: &SettingsModel,
    language: &str,
    flow: &mut egui::Ui,
    metrics: UiMetrics,
    layout: &mut HomeLayout,
) {
    let size = metrics.settings_row_label_size_desktop;
    account_group(
        flow,
        &localized("settings.group.layout", language),
        metrics,
        |card| {
            let painter = card.painter().clone();
            let lines =
                settings.layout_preview.len().max(1) as f32 * metrics.settings_layout_preview_line;
            card.add_space(metrics.settings_layout_preview_pad);
            let preview = inset_row(card, lines, metrics);
            if settings.layout_preview.is_empty() {
                flex::paint_text(
                    &painter,
                    preview,
                    &localized("settings.layout_original", language),
                    &crate::fonts::regular(size),
                    metrics.text_muted,
                    Align2::LEFT_TOP,
                );
            }
            let preview_lines = flex::column((0..settings.layout_preview.len()).map(|index| {
                flex::leaf(Vec2::ZERO)
                    .height(metrics.settings_layout_preview_line)
                    .id(index)
            }))
            .align(Align::Stretch)
            .solve(preview);
            for (index, line) in settings.layout_preview.iter().enumerate() {
                let font = if index == 0 {
                    FontId::proportional(size + Space::S2.px(&metrics))
                } else {
                    crate::fonts::regular(size)
                };
                flex::paint_text(
                    &painter,
                    preview_lines.find(index).unwrap_or(preview),
                    line,
                    &font,
                    if index == 0 {
                        metrics.text_primary
                    } else {
                        metrics.text_muted
                    },
                    Align2::LEFT_TOP,
                );
            }
            card.add_space(Space::S12.px(&metrics));
            for index in LAYOUT_FIELDS..BADGE_INPUT_COUNT {
                let row = form_row(card, &painter, metrics, true);
                field_label(&painter, row, index, language, size, metrics);
                settings_panel_input(
                    context,
                    layout,
                    input_node(index),
                    form_input(row, metrics, 0.3),
                    &settings.badge_fields[index],
                    "{resolution}".to_owned(),
                    metrics,
                );
            }
            let action_row = form_row(card, &painter, metrics, true);
            pills_right(
                context,
                &painter,
                layout,
                action_row,
                &[
                    (
                        NODE_SETTINGS_LAYOUT_USE_CURRENT,
                        localized("settings.layout_use_current", language),
                    ),
                    (
                        NODE_SETTINGS_LAYOUT_SAVE,
                        localized("settings.layout_save", language),
                    ),
                ],
                size,
            );
        },
    );
}

fn draw_formatter(
    context: &egui::Context,
    settings: &SettingsModel,
    assets: &mut impl HomeAssets,
    language: &str,
    flow: &mut egui::Ui,
    metrics: UiMetrics,
    layout: &mut HomeLayout,
) {
    let size = metrics.settings_row_label_size_desktop;
    let muted = crate::fonts::regular(metrics.screen_card_subtitle_size + Dp::D2.px());
    let gap = metrics.settings_group_gap;

    account_group(
        flow,
        &localized("settings.group.formatter_presets", language),
        metrics,
        |card| {
            let painter = card.painter().clone();
            for (index, preset) in settings.formatter_presets.iter().enumerate() {
                let row = account_row(card, &painter, metrics, index > 0);
                let toggle = if preset.enabled {
                    "settings.badges_disable"
                } else {
                    "settings.badges_enable"
                };
                let right = pills_right(
                    context,
                    &painter,
                    layout,
                    row,
                    &[(
                        NODE_SETTINGS_FORMATTER_PRESET_BASE + index as u64,
                        localized(toggle, language),
                    )],
                    size,
                );
                let (name, description) = stacked_lines(
                    trimmed(row, right, Space::S12.px(&metrics)),
                    Space::S10.px(&metrics),
                    Space::S12.px(&metrics),
                );
                flex::paint_text(
                    &painter,
                    name,
                    &preset.name,
                    &crate::fonts::regular(size),
                    if preset.enabled {
                        metrics.text_primary
                    } else {
                        metrics.text_muted
                    },
                    Align2::LEFT_CENTER,
                );
                flex::paint_text(
                    &painter,
                    description,
                    &preset.description,
                    &muted,
                    metrics.text_muted,
                    Align2::LEFT_CENTER,
                );
            }
        },
    );
    flow.add_space(gap);

    account_group(
        flow,
        &localized("settings.group.formatter", language),
        metrics,
        |card| {
            let painter = card.painter().clone();
            for index in FORM_FIELDS..LAYOUT_FIELDS {
                let row = form_row(card, &painter, metrics, index > FORM_FIELDS);
                field_label(&painter, row, index, language, size, metrics);
                settings_panel_input(
                    context,
                    layout,
                    input_node(index),
                    form_input(row, metrics, 0.4),
                    &settings.badge_fields[index],
                    String::new(),
                    metrics,
                );
            }
            let action_row = form_row(card, &painter, metrics, true);
            let right = pills_right(
                context,
                &painter,
                layout,
                action_row,
                &[
                    (
                        NODE_SETTINGS_FORMATTER_CLEAR,
                        localized("settings.badges_clear", language),
                    ),
                    (
                        NODE_SETTINGS_FORMATTER_SAVE,
                        localized("settings.formatter_save", language),
                    ),
                ],
                size,
            );
            if let Some(status) = &settings.formatter_status {
                flex::paint_text(
                    &painter,
                    trimmed(action_row, right, Space::S12.px(&metrics)),
                    status,
                    &muted,
                    metrics.text_muted,
                    Align2::LEFT_CENTER,
                );
            }
        },
    );

    if settings.formatter_rules.is_empty() {
        return;
    }
    flow.add_space(gap);
    account_group(
        flow,
        &localized("settings.group.formatter_rules", language),
        metrics,
        |card| {
            let painter = card.painter().clone();
            for (index, rule) in settings.formatter_rules.iter().enumerate() {
                let row = account_row(card, &painter, metrics, index > 0);
                let toggle = if rule.enabled {
                    "settings.badges_disable"
                } else {
                    "settings.badges_enable"
                };
                let right = pills_right(
                    context,
                    &painter,
                    layout,
                    row,
                    &[
                        (
                            NODE_SETTINGS_FORMATTER_EDIT_BASE + index as u64,
                            localized("settings.badges_edit", language),
                        ),
                        (
                            NODE_SETTINGS_FORMATTER_TOGGLE_BASE + index as u64,
                            localized(toggle, language),
                        ),
                        (
                            NODE_SETTINGS_FORMATTER_REMOVE_BASE + index as u64,
                            localized("settings.badges_remove", language),
                        ),
                    ],
                    size,
                );
                let replacement = if rule.replacement.is_empty() {
                    localized("settings.formatter_removed", language)
                } else {
                    rule.replacement.clone()
                };
                let font = crate::fonts::regular(size);
                let color = if rule.enabled {
                    metrics.text_primary
                } else {
                    metrics.text_muted
                };
                let arrow = Space::S16.px(&metrics);
                let arrow_gap = Space::S10.px(&metrics);
                let avail = trimmed(row, right, Space::S12.px(&metrics));
                let pattern = painter.layout_no_wrap(rule.pattern.clone(), font.clone(), color);
                let cells = flex::row([
                    flex::leaf(Vec2::new(0.0, size))
                        .width(
                            pattern
                                .size()
                                .x
                                .min(avail.width() * metrics.settings_value_max_share),
                        )
                        .id(PATTERN),
                    flex::leaf(Vec2::splat(arrow)).width(arrow).id(ARROW),
                    flex::leaf(Vec2::ZERO)
                        .grow(1.0)
                        .height(size)
                        .id(REPLACEMENT),
                ])
                .gap(arrow_gap)
                .align(Align::Center)
                .solve(avail);
                flex::paint_text(
                    &painter,
                    cells.find(PATTERN).unwrap_or(row),
                    &rule.pattern,
                    &font,
                    color,
                    Align2::LEFT_CENTER,
                );
                if let (Some(icon), Some(slot)) = (assets.icon("ArrowRight"), cells.find(ARROW)) {
                    painter.image(icon, slot, full_uv(), color);
                }
                flex::paint_text(
                    &painter,
                    cells.find(REPLACEMENT).unwrap_or(row),
                    &replacement,
                    &font,
                    color,
                    Align2::LEFT_CENTER,
                );
            }
        },
    );
}

pub(super) fn account_divider_at(painter: &egui::Painter, left: f32, right: f32, y: f32) {
    painter.line_segment(
        [Pos2::new(left, y), Pos2::new(right, y)],
        egui::Stroke::new(stroke::hairline(), crate::stylesheet::color("outline")),
    );
}
