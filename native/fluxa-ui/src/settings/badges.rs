use super::account::{account_divider, account_group, account_row};
use super::*;

const FORM_ROW: f32 = 52.0;
const IMPORT_HEIGHT: f32 = 76.0;
const STATUS_HEIGHT: f32 = 24.0;
const FIELD_LABELS: [&str; BADGE_INPUT_COUNT] = [
    "settings.badges_import_hint",
    "settings.badges_field_name",
    "settings.badges_field_pattern",
    "settings.badges_field_color",
    "settings.badges_field_text_color",
    "settings.badges_field_border_color",
    "settings.badges_field_image",
];

pub fn badge_input(node: u64) -> Option<usize> {
    let index = node.checked_sub(NODE_SETTINGS_BADGE_INPUT_BASE)? as usize;
    (index < BADGE_INPUT_COUNT).then_some(index)
}

pub fn badge_input_label(index: usize) -> &'static str {
    FIELD_LABELS[index]
}

fn import_height(settings: &SettingsModel) -> f32 {
    IMPORT_HEIGHT
        + if settings.badge_status.is_some() {
            STATUS_HEIGHT
        } else {
            0.0
        }
}

fn form_height() -> f32 {
    FORM_ROW * 8.0 + 12.0
}

fn block(height: f32) -> f32 {
    APPEARANCE_GROUP_HEADING_HEIGHT + height + APPEARANCE_GROUP_GAP
}

pub(super) fn badges_height(settings: &SettingsModel, metrics: UiMetrics) -> f32 {
    let packs = settings.badge_packs.len().max(1);
    let mut height = block(import_height(settings))
        + block(settings_group_card_height(packs, metrics))
        + block(form_height());
    if !settings.badge_custom.is_empty() {
        height += block(settings_group_card_height(
            settings.badge_custom.len(),
            metrics,
        ));
    }
    height
}

pub(super) fn pills_right(
    context: &egui::Context,
    painter: &egui::Painter,
    layout: &mut HomeLayout,
    row: Rect,
    pills: &[(u64, String)],
    size: f32,
) -> f32 {
    let mut right = row.right() - 4.0;
    for (id, label) in pills.iter().rev() {
        let width = painter
            .layout_no_wrap(label.clone(), FontId::proportional(size), Color32::WHITE)
            .size()
            .x
            + 32.0;
        let button = Rect::from_center_size(
            Pos2::new(right - width * 0.5, row.center().y),
            Vec2::new(width, 32.0_f32.min(row.height() - 4.0)),
        );
        pill_button(context, layout, *id, button, label, size);
        right -= width + 8.0;
    }
    right
}

pub(super) fn draw_badges(
    context: &egui::Context,
    settings: &SettingsModel,
    assets: &mut impl HomeAssets,
    language: &str,
    rect: Rect,
    mut top: f32,
    metrics: UiMetrics,
    layout: &mut HomeLayout,
) {
    let painter = context.layer_painter(egui::LayerId::background());
    let size = metrics.settings_row_label_size_desktop;
    let muted = crate::fonts::regular(metrics.screen_card_subtitle_size + 2.0);

    let card = account_group(
        &painter,
        rect,
        top,
        import_height(settings),
        &localized("settings.group.badges_import", language),
        false,
        metrics,
    );
    let row = Rect::from_min_size(
        card.left_top() + Vec2::new(metrics.settings_row_inset, 20.0),
        Vec2::new(card.width() - metrics.settings_row_inset * 2.0, 36.0),
    );
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
        Rect::from_min_max(row.left_top(), Pos2::new(right - 4.0, row.bottom())),
        &settings.badge_fields[0],
        localized(FIELD_LABELS[0], language),
        metrics,
    );
    if let Some(status) = &settings.badge_status {
        painter.text(
            Pos2::new(row.left(), row.bottom() + 8.0),
            Align2::LEFT_TOP,
            truncate_to_width(&painter, status, &muted, row.width()),
            muted.clone(),
            metrics.text_muted,
        );
    }
    top = card.bottom() + APPEARANCE_GROUP_GAP;

    let rows = settings.badge_packs.len().max(1);
    let card = account_group(
        &painter,
        rect,
        top,
        settings_group_card_height(rows, metrics),
        &localized("settings.group.badges_packs", language),
        false,
        metrics,
    );
    if settings.badge_packs.is_empty() {
        let row = account_row(card, 0, metrics);
        painter.text(
            Pos2::new(row.left(), row.center().y),
            Align2::LEFT_CENTER,
            localized("settings.badges_no_packs", language),
            muted.clone(),
            metrics.text_muted,
        );
    }
    for (index, pack) in settings.badge_packs.iter().enumerate() {
        let row = account_row(card, index, metrics);
        if index > 0 {
            account_divider(&painter, row, metrics);
        }
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
        painter.text(
            Pos2::new(row.left(), row.center().y - 9.0),
            Align2::LEFT_CENTER,
            truncate_to_width(
                &painter,
                &pack.name,
                &crate::fonts::regular(size),
                right - row.left() - 12.0,
            ),
            crate::fonts::regular(size),
            if pack.active {
                metrics.text_primary
            } else {
                metrics.text_muted
            },
        );
        painter.text(
            Pos2::new(row.left(), row.center().y + 11.0),
            Align2::LEFT_CENTER,
            format!(
                "{} {}",
                pack.count,
                localized("settings.badges_count_unit", language)
            ),
            muted.clone(),
            metrics.text_muted,
        );
    }
    top = card.bottom() + APPEARANCE_GROUP_GAP;

    let card = account_group(
        &painter,
        rect,
        top,
        form_height(),
        &localized("settings.group.badges_custom", language),
        false,
        metrics,
    );
    for index in 1..BADGE_INPUT_COUNT {
        let row = Rect::from_min_size(
            card.left_top()
                + Vec2::new(
                    metrics.settings_row_inset,
                    6.0 + (index - 1) as f32 * FORM_ROW,
                ),
            Vec2::new(card.width() - metrics.settings_row_inset * 2.0, FORM_ROW),
        );
        if index > 1 {
            account_divider_at(&painter, row.left(), row.right(), row.top());
        }
        painter.text(
            Pos2::new(row.left(), row.center().y),
            Align2::LEFT_CENTER,
            localized(FIELD_LABELS[index], language),
            crate::fonts::regular(size),
            metrics.text_primary,
        );
        let input = Rect::from_min_max(
            Pos2::new(row.left() + row.width() * 0.4, row.center().y - 18.0),
            Pos2::new(row.right(), row.center().y + 18.0),
        );
        settings_panel_input(
            context,
            layout,
            NODE_SETTINGS_BADGE_INPUT_BASE + index as u64,
            input,
            &settings.badge_fields[index],
            if (3..=5).contains(&index) {
                "#RRGGBB".to_owned()
            } else {
                String::new()
            },
            metrics,
        );
    }
    let style_row = Rect::from_min_size(
        card.left_top()
            + Vec2::new(
                metrics.settings_row_inset,
                6.0 + (BADGE_INPUT_COUNT - 1) as f32 * FORM_ROW,
            ),
        Vec2::new(card.width() - metrics.settings_row_inset * 2.0, FORM_ROW),
    );
    account_divider_at(
        &painter,
        style_row.left(),
        style_row.right(),
        style_row.top(),
    );
    painter.text(
        Pos2::new(style_row.left(), style_row.center().y),
        Align2::LEFT_CENTER,
        localized("settings.badges_field_style", language),
        crate::fonts::regular(size),
        metrics.text_primary,
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
    let action_row = Rect::from_min_size(style_row.left_bottom(), style_row.size());
    account_divider_at(
        &painter,
        action_row.left(),
        action_row.right(),
        action_row.top(),
    );
    let preview = &settings.badge_preview;
    let font_size = metrics.screen_card_subtitle_size + 2.0;
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
    top = card.bottom() + APPEARANCE_GROUP_GAP;

    if settings.badge_custom.is_empty() {
        return;
    }
    let card = account_group(
        &painter,
        rect,
        top,
        settings_group_card_height(settings.badge_custom.len(), metrics),
        &localized("settings.group.badges_yours", language),
        false,
        metrics,
    );
    for (index, custom) in settings.badge_custom.iter().enumerate() {
        let row = account_row(card, index, metrics);
        if index > 0 {
            account_divider(&painter, row, metrics);
        }
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
        let (galley, chip_size) = components::badge_chip_layout(&painter, &custom.badge, font_size);
        let chip = Rect::from_min_size(
            Pos2::new(row.left(), row.center().y - chip_size.y * 0.5),
            chip_size,
        );
        components::paint_badge_chip(&painter, chip, galley, &custom.badge, assets);
        if !custom.enabled {
            painter.rect_filled(
                chip.expand(2.0),
                chip.height() * 0.5,
                metrics.surface.gamma_multiply(0.7),
            );
        }
    }
}

pub(super) fn account_divider_at(painter: &egui::Painter, left: f32, right: f32, y: f32) {
    painter.line_segment(
        [Pos2::new(left, y), Pos2::new(right, y)],
        egui::Stroke::new(1.0, Color32::from_white_alpha(12)),
    );
}
