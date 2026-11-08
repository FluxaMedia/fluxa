use super::account::{account_group, account_row};
use super::*;
use crate::draw_text;

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
    card.allocate_space(Vec2::new(width, height))
        .1
        .shrink2(Vec2::new(metrics.settings_row_inset, 0.0))
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
    let half = metrics.screen_control_height * 0.5;
    Rect::from_min_max(
        Pos2::new(row.left() + row.width() * start, row.center().y - half),
        Pos2::new(row.right(), row.center().y + half),
    )
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
        .settings_pill_height
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
    let muted = crate::fonts::regular(metrics.screen_card_subtitle_size + 2.0);
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
                Rect::from_min_max(
                    row.left_top(),
                    Pos2::new(right - Space::S4.px(&metrics), row.bottom()),
                ),
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
                draw_text(
                    &painter,
                    Pos2::new(row.left(), row.center().y),
                    Align2::LEFT_CENTER,
                    localized("settings.badges_no_packs", language),
                    muted.clone(),
                    metrics.text_muted,
                    None,
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
                draw_text(
                    &painter,
                    Pos2::new(row.left(), row.center().y - Space::S10.px(&metrics)),
                    Align2::LEFT_CENTER,
                    truncate_to_width(
                        &painter,
                        &pack.name,
                        &crate::fonts::regular(size),
                        right - row.left() - Space::S12.px(&metrics),
                    ),
                    crate::fonts::regular(size),
                    if pack.active {
                        metrics.text_primary
                    } else {
                        metrics.text_muted
                    },
                    None,
                );
                draw_text(
                    &painter,
                    Pos2::new(row.left(), row.center().y + Space::S12.px(&metrics)),
                    Align2::LEFT_CENTER,
                    format!(
                        "{} {}",
                        pack.count,
                        localized("settings.badges_count_unit", language)
                    ),
                    muted.clone(),
                    metrics.text_muted,
                    None,
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
                draw_text(
                    &painter,
                    Pos2::new(row.left(), row.center().y),
                    Align2::LEFT_CENTER,
                    truncate_to_width(
                        &painter,
                        &choice.name,
                        &crate::fonts::regular(size),
                        right - row.left() - Space::S12.px(&metrics),
                    ),
                    crate::fonts::regular(size),
                    if choice.selected {
                        metrics.text_primary
                    } else {
                        metrics.text_muted
                    },
                    None,
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
                draw_text(
                    &painter,
                    Pos2::new(row.left(), row.center().y),
                    Align2::LEFT_CENTER,
                    localized(FIELD_LABELS[index], language),
                    crate::fonts::regular(size),
                    metrics.text_primary,
                    None,
                );
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
            let chip = Rect::from_min_size(
                Pos2::new(action_row.left(), action_row.center().y - chip_size.y * 0.5),
                chip_size,
            );
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
                let chip = Rect::from_min_size(
                    Pos2::new(row.left(), row.center().y - chip_size.y * 0.5),
                    chip_size,
                );
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
                draw_text(
                    &painter,
                    preview.left_top(),
                    Align2::LEFT_TOP,
                    localized("settings.layout_original", language),
                    crate::fonts::regular(size),
                    metrics.text_muted,
                    None,
                );
            }
            for (index, line) in settings.layout_preview.iter().enumerate() {
                let font = if index == 0 {
                    FontId::proportional(size + Space::S2.px(&metrics))
                } else {
                    crate::fonts::regular(size)
                };
                draw_text(
                    &painter,
                    preview.left_top()
                        + Vec2::new(0.0, index as f32 * metrics.settings_layout_preview_line),
                    Align2::LEFT_TOP,
                    line,
                    font,
                    if index == 0 {
                        metrics.text_primary
                    } else {
                        metrics.text_muted
                    },
                    Some(preview.width()),
                );
            }
            card.add_space(Space::S12.px(&metrics));
            for index in LAYOUT_FIELDS..BADGE_INPUT_COUNT {
                let row = form_row(card, &painter, metrics, true);
                draw_text(
                    &painter,
                    Pos2::new(row.left(), row.center().y),
                    Align2::LEFT_CENTER,
                    localized(FIELD_LABELS[index], language),
                    crate::fonts::regular(size),
                    metrics.text_primary,
                    None,
                );
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
    let muted = crate::fonts::regular(metrics.screen_card_subtitle_size + 2.0);
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
                let font = crate::fonts::regular(size);
                let width = right - row.left() - Space::S12.px(&metrics);
                draw_text(
                    &painter,
                    Pos2::new(row.left(), row.center().y - Space::S10.px(&metrics)),
                    Align2::LEFT_CENTER,
                    &preset.name,
                    font,
                    if preset.enabled {
                        metrics.text_primary
                    } else {
                        metrics.text_muted
                    },
                    Some(width),
                );
                draw_text(
                    &painter,
                    Pos2::new(row.left(), row.center().y + Space::S12.px(&metrics)),
                    Align2::LEFT_CENTER,
                    &preset.description,
                    muted.clone(),
                    metrics.text_muted,
                    Some(width),
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
                draw_text(
                    &painter,
                    Pos2::new(row.left(), row.center().y),
                    Align2::LEFT_CENTER,
                    localized(FIELD_LABELS[index], language),
                    crate::fonts::regular(size),
                    metrics.text_primary,
                    None,
                );
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
                draw_text(
                    &painter,
                    Pos2::new(action_row.left(), action_row.center().y),
                    Align2::LEFT_CENTER,
                    status,
                    muted.clone(),
                    metrics.text_muted,
                    Some(right - action_row.left() - Space::S12.px(&metrics)),
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
                let avail = right - row.left() - Space::S12.px(&metrics);
                let pattern = painter.layout_no_wrap(rule.pattern.clone(), font.clone(), color);
                let pattern_width = pattern.size().x.min(avail * 0.5);
                draw_text(
                    &painter,
                    Pos2::new(row.left(), row.center().y),
                    Align2::LEFT_CENTER,
                    &rule.pattern,
                    font.clone(),
                    color,
                    Some(pattern_width),
                );
                let arrow_left = row.left() + pattern_width + arrow_gap;
                if let Some(icon) = assets.icon("ArrowRight") {
                    painter.image(
                        icon,
                        Rect::from_center_size(
                            Pos2::new(arrow_left + arrow / 2.0, row.center().y),
                            Vec2::splat(arrow),
                        ),
                        Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                        color,
                    );
                }
                let text_left = arrow_left + arrow + arrow_gap;
                draw_text(
                    &painter,
                    Pos2::new(text_left, row.center().y),
                    Align2::LEFT_CENTER,
                    &replacement,
                    font,
                    color,
                    Some(row.left() + avail - text_left),
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
