use super::page::padded;
use super::*;
use crate::flex::{self, Align, Insets, Justify};

const LABEL: usize = 1;
const HELP: usize = 2;
const INPUT: usize = 3;
const NAME: usize = 4;
const RESET: usize = 5;
const KEYS: usize = 6;

pub(super) fn draw_field(
    context: &egui::Context,
    painter: &egui::Painter,
    settings: &SettingsModel,
    index: usize,
    area: Rect,
    metrics: UiMetrics,
    compact: bool,
    layout: &mut HomeLayout,
) {
    let language = settings.language();
    let field = &POSTER_FIELDS[index];
    let label_size = if compact {
        metrics.nav_label_size + metrics.settings_row_label_boost
    } else {
        metrics.settings_row_label_size_desktop
    };
    let help_size = metrics.screen_card_subtitle_size + metrics.settings_field_help_boost;
    let help_font = crate::fonts::regular(help_size);
    let height = metrics.screen_control_height;
    let column = flex::column([
        flex::leaf(Vec2::ZERO).height(label_size).id(LABEL),
        flex::leaf(Vec2::ZERO).height(help_size).id(HELP),
        flex::fill(),
        flex::leaf(Vec2::ZERO).height(height).id(INPUT),
    ])
    .align(Align::Stretch)
    .gap(Space::S6.px(&metrics))
    .padding(Insets::xy(
        metrics.settings_row_inset + Space::S4.px(&metrics),
        Space::S16.px(&metrics),
    ))
    .solve(area);
    let label = column.find(LABEL).unwrap_or(area);
    let help = column.find(HELP).unwrap_or(area);
    let input = column.find(INPUT).unwrap_or(area);
    draw_text(
        painter,
        label.left_top(),
        Align2::LEFT_TOP,
        localized(field.label, language),
        crate::fonts::regular(label_size),
        metrics.text_primary,
        None,
    );
    flex::paint_text(
        painter,
        help,
        &localized(field.help, language),
        &help_font,
        metrics.text_muted,
        Align2::LEFT_TOP,
    );
    if field.kind == FieldKind::Action {
        let width = metrics.settings_extended_addon_action_width;
        let button = flex::anchored(input, Align2::LEFT_CENTER, Vec2::new(width, height), 0.0);
        layout.focusable.push((field.input, button));
        components::placed_front(
            context,
            Id::new(("fluxa-settings-form", field.input)),
            button.min,
            |ui| {
                if components::button(
                    ui,
                    &localized(field.hint, language),
                    width,
                    height,
                    components::ButtonKind::Secondary,
                    metrics,
                )
                .clicked()
                {
                    layout.activated = Some(field.input);
                }
            },
        );
        return;
    }
    settings_panel_input(
        context,
        layout,
        field.input,
        input,
        &settings.poster_fields[index],
        localized(field.hint, language),
        metrics,
    );
}

pub fn setup_accepts(field_key: &str, pref: &str) -> bool {
    let lower = pref.to_lowercase();
    if ["key", "token", "password", "secret", "epoch"]
        .iter()
        .any(|word| lower.contains(word))
    {
        return false;
    }
    match field_key {
        "metadatakitSetup" => pref.starts_with("metadatakit") || pref.starts_with("catalog"),
        _ => {
            (pref.starts_with("stream") && !pref.starts_with("streamProxy"))
                || pref.starts_with("debrid")
        }
    }
}

pub fn setup_export(field_key: &str, values: &serde_json::Value) -> String {
    let picked: serde_json::Map<String, serde_json::Value> = values
        .as_object()
        .into_iter()
        .flatten()
        .filter(|(pref, value)| {
            setup_accepts(field_key, pref) && !value.is_array() && !value.is_object()
        })
        .map(|(pref, value)| (pref.clone(), value.clone()))
        .collect();
    serde_json::Value::Object(picked).to_string()
}

pub(super) fn draw_subtitle_preview(
    settings: &SettingsModel,
    flow: &mut egui::Ui,
    language: &str,
    metrics: UiMetrics,
) -> Rect {
    account_group(
        flow,
        &localized("settings.group.subtitle_preview", language),
        metrics,
        |card| {
            let painter = card.painter().clone();
            let area = card
                .allocate_space(Vec2::new(
                    card.max_rect().width(),
                    metrics.settings_subtitle_preview_height,
                ))
                .1;
            let screen = padded(area, Insets::all(Space::S12.px(&metrics)));
            let radius = metrics.card_radius * metrics.settings_preview_radius_share;
            painter.rect_filled(screen, radius, Color32::from_rgb(28, 32, 40));
            let halves = flex::column([flex::fill().id(0), flex::fill().id(1)])
                .align(Align::Stretch)
                .solve(screen);
            paint_vertical_gradient(
                &painter,
                halves.find(1).unwrap_or(screen),
                Color32::TRANSPARENT,
                Color32::from_rgb(8, 9, 12),
            );
            components::subtitle_sample(
                &painter,
                screen,
                &SubtitleLook::from_values(&settings.values),
                &localized("settings.subtitle_preview_text", language),
                screen.height() * metrics.settings_preview_text_share,
            );
        },
    )
}

pub(super) fn draw_field_group(
    context: &egui::Context,
    settings: &SettingsModel,
    flow: &mut egui::Ui,
    title: &str,
    fields: &[usize],
    metrics: UiMetrics,
    compact: bool,
    layout: &mut HomeLayout,
) -> Rect {
    account_group(flow, title, metrics, |card| {
        let painter = card.painter().clone();
        for (slot, &index) in fields.iter().enumerate() {
            let area = card
                .allocate_space(Vec2::new(
                    card.max_rect().width(),
                    metrics.settings_field_height,
                ))
                .1;
            if slot > 0 {
                let rule = padded(area, Insets::xy(metrics.settings_row_inset, 0.0));
                painter.line_segment(
                    [rule.left_top(), rule.right_top()],
                    egui::Stroke::new(stroke::hairline(), crate::stylesheet::color("outline")),
                );
            }
            draw_field(
                context, &painter, settings, index, area, metrics, compact, layout,
            );
        }
    })
}

pub(super) fn keycap_button(
    context: &egui::Context,
    layout: &mut HomeLayout,
    id: u64,
    rect: Rect,
    label: &str,
    size: f32,
    recording: bool,
    metrics: UiMetrics,
) {
    components::placed_front(
        context,
        Id::new(("fluxa-settings-keycap", id)),
        rect.min,
        |ui| {
            let response = ui.allocate_rect(rect, Sense::click());
            let lit = response.hovered() || recording;
            components::draw_keys(
                ui.painter(),
                padded(rect, Insets::xy(Space::S4.px(&metrics), 0.0)),
                label,
                size,
                lit,
                text_primary(),
            );
            components::focus_action(layout, id, &response);
        },
    );
}

pub(super) fn draw_shortcut_group(
    context: &egui::Context,
    flow: &mut egui::Ui,
    title: &str,
    rows: &[(usize, &ShortcutRow)],
    settings: &SettingsModel,
    language: &str,
    metrics: UiMetrics,
    layout: &mut HomeLayout,
) -> Rect {
    account_group(flow, title, metrics, |card| {
        let painter = card.painter().clone();
        let size = metrics.settings_row_label_size_desktop;
        for (position, (index, shortcut)) in rows.iter().enumerate() {
            let row = account_row(card, &painter, metrics, position > 0);
            let recording = settings.shortcut_recording.as_deref() == Some(shortcut.id.as_str());
            let label = if recording {
                localized("settings.shortcuts_recording", language)
            } else if shortcut.label.is_empty() {
                localized("settings.shortcuts_unassigned", language)
            } else {
                shortcut.label.clone()
            };
            let keys_size = size * metrics.settings_keys_share;
            let width = components::keys_width(&painter, &label, keys_size)
                .max(Space::S48.px(&metrics))
                + Space::S8.px(&metrics);
            let height = row.height() - Space::S8.px(&metrics);
            let reset = shortcut
                .custom
                .then(|| localized("settings.shortcuts_reset_one", language));
            let mut cells = vec![flex::fill().id(NAME)];
            if let Some(reset) = &reset {
                cells.push(
                    flex::leaf(Vec2::ZERO)
                        .width(pill_text_width(&painter, reset, size, metrics))
                        .height(height)
                        .id(RESET),
                );
            }
            cells.push(flex::leaf(Vec2::ZERO).width(width).height(height).id(KEYS));
            let line = flex::row(cells)
                .align(Align::Center)
                .gap(Space::S10.px(&metrics))
                .padding(Insets::xy(Space::S4.px(&metrics), 0.0))
                .solve(row);
            if let Some(name) = line.find(NAME) {
                flex::paint_text(
                    &painter,
                    name,
                    &localized(&format!("shortcuts.{}", shortcut.id), language),
                    &FontId::proportional(size),
                    metrics.text_primary,
                    Align2::LEFT_CENTER,
                );
            }
            if let Some(button) = line.find(KEYS) {
                keycap_button(
                    context,
                    layout,
                    NODE_SETTINGS_SHORTCUT_BASE + *index as u64,
                    button,
                    &label,
                    keys_size,
                    recording,
                    metrics,
                );
            }
            if let (Some(reset), Some(button)) = (&reset, line.find(RESET)) {
                pill_button(
                    context,
                    layout,
                    NODE_SETTINGS_SHORTCUT_RESET_BASE + *index as u64,
                    button,
                    reset,
                    size,
                );
            }
        }
    })
}

pub(super) fn draw_binding_group(
    flow: &mut egui::Ui,
    title: &str,
    rows: &[(&str, &str)],
    language: &str,
    metrics: UiMetrics,
) -> Rect {
    account_group(flow, title, metrics, |card| {
        let painter = card.painter().clone();
        let label_font = FontId::proportional(metrics.settings_row_label_size_desktop);
        for (index, (action, keys)) in rows.iter().enumerate() {
            let row = account_row(card, &painter, metrics, index > 0);
            flex::paint_text(
                &painter,
                padded(row, Insets::xy(Space::S4.px(&metrics), 0.0)),
                &localized(action, language),
                &label_font,
                metrics.text_primary,
                Align2::LEFT_CENTER,
            );
            components::draw_keys(
                &painter,
                padded(
                    row,
                    Insets {
                        right: Space::S4.px(&metrics),
                        ..Insets::default()
                    },
                ),
                keys,
                label_font.size * metrics.settings_keys_share,
                false,
                metrics.text_primary,
            );
        }
    })
}
