use super::*;

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
        metrics.nav_label_size + 2.0
    } else {
        metrics.settings_row_label_size_desktop
    };
    let inner = area.shrink2(Vec2::new(
        metrics.settings_row_inset + Space::S4.px(&metrics),
        Space::S16.px(&metrics),
    ));
    draw_text(
        painter,
        inner.left_top(),
        Align2::LEFT_TOP,
        localized(field.label, language),
        crate::fonts::regular(label_size),
        metrics.text_primary,
        None,
    );
    let help_font = crate::fonts::regular(metrics.screen_card_subtitle_size + 2.0);
    draw_text(
        painter,
        inner.left_top() + Vec2::new(0.0, label_size + Space::S6.px(&metrics)),
        Align2::LEFT_TOP,
        truncate_to_width(
            &painter,
            &localized(field.help, language),
            &help_font,
            inner.width(),
        ),
        help_font,
        metrics.text_muted,
        None,
    );
    let height = metrics.screen_control_height;
    let input = Rect::from_min_max(
        Pos2::new(inner.left(), inner.bottom() - height),
        inner.right_bottom(),
    );
    if field.kind == FieldKind::Action {
        let width = metrics.settings_extended_addon_action_width;
        let button = Rect::from_min_size(input.min, Vec2::new(width, height));
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
            let screen = area.shrink(Space::S12.px(&metrics));
            let radius = metrics.card_radius * 0.6;
            painter.rect_filled(screen, radius, Color32::from_rgb(28, 32, 40));
            paint_vertical_gradient(
                &painter,
                Rect::from_min_max(
                    Pos2::new(screen.left(), screen.center().y),
                    screen.right_bottom(),
                ),
                Color32::TRANSPARENT,
                Color32::from_rgb(8, 9, 12),
            );
            components::subtitle_sample(
                &painter,
                screen,
                &SubtitleLook::from_values(&settings.values),
                &localized("settings.subtitle_preview_text", language),
                screen.height() * 0.17,
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
                painter.line_segment(
                    [
                        Pos2::new(area.left() + metrics.settings_row_inset, area.top()),
                        Pos2::new(area.right() - metrics.settings_row_inset, area.top()),
                    ],
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
            let response =
                ui.allocate_rect(Rect::from_min_size(rect.min, rect.size()), Sense::click());
            let lit = response.hovered() || recording;
            components::draw_keys(
                ui.painter(),
                rect.shrink2(Vec2::new(Space::S4.px(&metrics), 0.0)),
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
            draw_text(
                &painter,
                row.left_center() + Vec2::new(Space::S4.px(&metrics), 0.0),
                Align2::LEFT_CENTER,
                localized(&format!("shortcuts.{}", shortcut.id), language),
                FontId::proportional(size),
                metrics.text_primary,
                None,
            );
            let recording = settings.shortcut_recording.as_deref() == Some(shortcut.id.as_str());
            let label = if recording {
                localized("settings.shortcuts_recording", language)
            } else if shortcut.label.is_empty() {
                localized("settings.shortcuts_unassigned", language)
            } else {
                shortcut.label.clone()
            };
            let keys_size = size * 0.9;
            let width = components::keys_width(&painter, &label, keys_size)
                .max(metrics.settings_keycap_min_width)
                + Space::S8.px(&metrics);
            let height = row.height() - Space::S8.px(&metrics);
            let button = Rect::from_center_size(
                Pos2::new(
                    row.right() - Space::S4.px(&metrics) - width * 0.5,
                    row.center().y,
                ),
                Vec2::new(width, height),
            );
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
            if shortcut.custom {
                let reset = localized("settings.shortcuts_reset_one", language);
                let reset_width = pill_text_width(&painter, &reset, size, metrics);
                let reset_button = Rect::from_center_size(
                    Pos2::new(
                        button.left() - Space::S10.px(&metrics) - reset_width * 0.5,
                        row.center().y,
                    ),
                    Vec2::new(reset_width, height),
                );
                pill_button(
                    context,
                    layout,
                    NODE_SETTINGS_SHORTCUT_RESET_BASE + *index as u64,
                    reset_button,
                    &reset,
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
            draw_text(
                &painter,
                row.left_center() + Vec2::new(Space::S4.px(&metrics), 0.0),
                Align2::LEFT_CENTER,
                localized(action, language),
                label_font.clone(),
                metrics.text_primary,
                None,
            );
            components::draw_keys(
                &painter,
                Rect::from_min_max(
                    Pos2::new(row.left(), row.top()),
                    Pos2::new(row.right() - Space::S4.px(&metrics), row.bottom()),
                ),
                keys,
                label_font.size * 0.9,
                false,
                metrics.text_primary,
            );
        }
    })
}
