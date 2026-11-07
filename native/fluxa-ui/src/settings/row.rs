use super::page::Frame;
use super::*;

pub(super) fn draw_row(
    frame: Frame,
    assets: &mut impl HomeAssets,
    layout: &mut HomeLayout,
    row: (usize, usize, Rect, f32, f32),
) {
    let Frame {
        context,
        viewport,
        settings,
        language,
        metrics,
        painter,
        ..
    } = frame;
    let (index, slot, card, offset, extra) = row;
    let compact = viewport.is_compact();
    let tv = viewport.is_tv();
    let desktop = !compact && !tv;
    let row_metrics = described(metrics);
    let Some(setting) = settings_row_by_index(index) else {
        return;
    };
    let node = NODE_SETTINGS_ROW_BASE + index as u64;
    let is_swatches = setting.key == ACCENT_COLOR_KEY;
    let range = slider_range(setting.key);
    let row_rect = Rect::from_min_size(
        card.left_top()
            + Vec2::new(
                metrics.settings_row_inset,
                6.0 + slot as f32 * row_metrics.settings_row_spacing + offset,
            ),
        Vec2::new(
            card.width() - metrics.settings_row_inset * 2.0,
            row_metrics.settings_row_height + extra,
        ),
    );
    let wrapped = extra > 0.0 && !is_swatches && range.is_none();
    let text_center_y = if wrapped {
        row_rect.center().y
    } else {
        row_rect.top() + row_metrics.settings_row_height * 0.5
    };
    if slot > 0 {
        let y = row_rect.top()
            - (row_metrics.settings_row_spacing - row_metrics.settings_row_height) * 0.5;
        painter.line_segment(
            [
                Pos2::new(row_rect.left(), y),
                Pos2::new(row_rect.right(), y),
            ],
            egui::Stroke::new(stroke::hairline(), ink(tone::faint())),
        );
    }
    let is_toggle = setting.choices().is_empty();
    let is_app_icon = setting.key == APP_ICON_KEY;
    let sheet_row = (compact || tv || is_app_icon) && !is_toggle && !is_swatches && range.is_none();
    let value_below =
        sheet_row && !is_app_icon && compact && viewport.platform == UiPlatform::Android;
    let mut clicked = false;
    if sheet_row || (tv && range.is_some()) {
        layout.focusable.push((node, row_rect));
    }
    if is_toggle {
        layout.focusable.push((node, row_rect));
        components::placed_front(
            context,
            Id::new(("fluxa-settings-row", node)),
            row_rect.min,
            |ui| {
                let (_, response) = ui.allocate_exact_size(row_rect.size(), Sense::click());
                clicked = response.clicked();
            },
        );
    }
    let row_label_size = if !compact && !tv {
        metrics.settings_row_label_size_desktop
    } else {
        metrics.nav_label_size + 2.0
    };
    let subtitle_size = metrics.screen_card_subtitle_size + metrics.settings_subtitle_boost;
    let value_size = if !compact && !tv {
        metrics.settings_row_value_size_desktop
    } else {
        metrics.screen_card_subtitle_size + 1.0
    };
    let raw_value = (!is_toggle).then(|| {
        settings
            .value(setting.key)
            .and_then(serde_json::Value::as_str)
            .unwrap_or(setting.choices()[0])
    });
    let value = raw_value.map(|raw| option_label(setting.key, raw, language));
    let value_font = crate::fonts::regular(value_size);
    let option_labels = setting
        .choices()
        .iter()
        .map(|option| option_label(setting.key, option, language))
        .collect::<Vec<_>>();
    let segment_width = components::segmented_width(&painter, &option_labels, metrics);
    let segmented = !sheet_row
        && !is_toggle
        && !is_swatches
        && range.is_none()
        && !is_app_icon
        && (2..=3).contains(&option_labels.len());
    let value_width = if value_below || is_app_icon {
        0.0
    } else if segmented {
        segment_width
    } else if sheet_row {
        value.as_deref().map_or(0.0, |value| {
            painter
                .layout_no_wrap(value.to_owned(), value_font.clone(), Color32::WHITE)
                .size()
                .x
                .min(row_rect.width() * 0.4)
        }) + 22.0
    } else {
        setting
            .choices()
            .iter()
            .map(|option| {
                painter
                    .layout_no_wrap(
                        option_label(setting.key, option, language),
                        value_font.clone(),
                        text_primary(),
                    )
                    .size()
                    .x
            })
            .fold(0.0, f32::max)
            + 50.0
    }
    .clamp(0.0, row_rect.width() * 0.5)
    .max(if sheet_row { 0.0 } else { 92.0 });
    let value_width = if value_below || is_app_icon {
        0.0
    } else {
        value_width
    };
    let text_left = row_rect.left() + 4.0;
    let control_width = if is_swatches {
        0.0
    } else if range.is_some() {
        64.0
    } else if is_app_icon {
        (row_rect.height() - 16.0).min(44.0) + 4.0
    } else if is_toggle {
        metrics.settings_toggle_width + 3.0
    } else {
        value_width + 2.0
    };
    let text_width = (row_rect.right() - control_width - 12.0 - text_left).max(1.0);
    let label_font = crate::fonts::regular(row_label_size);
    let subtitle_font = crate::fonts::regular(subtitle_size);
    let text_height = row_label_size
        + 4.0
        + subtitle_size
        + if wrapped {
            metrics.settings_row_subtitle_extra
        } else {
            0.0
        };
    let text_top = text_center_y - text_height * 0.5;
    draw_text(
        &painter,
        Pos2::new(text_left, text_top),
        Align2::LEFT_TOP,
        truncate_to_width(
            &painter,
            &settings_row_label(setting, language),
            &label_font,
            text_width,
        ),
        label_font.clone(),
        metrics.text_primary,
        None,
    );
    let subtitle_top = text_top + row_label_size + 4.0;
    if is_app_icon {
        let icon = (row_rect.height() - 16.0).min(44.0);
        if let Some(texture) = raw_value.and_then(|id| assets.app_icon(id)) {
            painter.image(
                texture,
                Rect::from_min_size(
                    Pos2::new(
                        row_rect.right() - 4.0 - icon,
                        row_rect.center().y - icon * 0.5,
                    ),
                    Vec2::splat(icon),
                ),
                Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                Color32::WHITE,
            );
        }
    }
    if wrapped {
        painter.galley(
            Pos2::new(text_left, subtitle_top),
            wrapped_regular(
                &painter,
                &localized(
                    &format!("settings.row.{}.description", setting.key),
                    language,
                ),
                subtitle_size,
                ink(100),
                text_width,
                2,
            ),
            ink(100),
        );
    } else {
        draw_text(
            &painter,
            Pos2::new(text_left, subtitle_top),
            Align2::LEFT_TOP,
            truncate_to_width(
                &painter,
                &if value_below || is_app_icon || is_swatches {
                    value.clone().unwrap_or_default()
                } else {
                    localized(
                        &format!("settings.row.{}.description", setting.key),
                        language,
                    )
                },
                &subtitle_font,
                text_width,
            ),
            subtitle_font,
            if value_below || is_app_icon || is_swatches {
                ink(tone::subtle())
            } else {
                ink(if compact {
                    metrics.settings_swatch_alpha_compact
                } else {
                    metrics.settings_swatch_alpha
                })
            },
            None,
        );
    }
    let control = if is_swatches {
        Control::Swatches
    } else if range.is_some() {
        Control::Slider
    } else if sheet_row {
        Control::Sheet
    } else if raw_value.is_some() && segmented {
        Control::Segmented
    } else if raw_value.is_some() {
        Control::Dropdown
    } else {
        Control::Toggle
    };
    let cx = RowCtx {
        context,
        painter,
        settings,
        setting,
        language,
        metrics,
        row_metrics,
        compact,
        tv,
        node,
        row_rect,
        text_left,
        text_center_y,
        value_size,
        value_width,
        value_font,
        value,
        raw_value,
        option_labels,
        value_below,
        is_app_icon,
    };
    match control {
        Control::Swatches => swatches(&cx, layout),
        Control::Slider => slider(&cx, layout, &range.expect("slider rows have a range")),
        Control::Sheet => sheet(&cx),
        Control::Segmented => segmented_control(&cx, layout),
        Control::Dropdown => dropdown(&cx, layout),
        Control::Toggle => toggle(&cx),
    }
    if clicked {
        layout.activated = Some(node);
    }
}

#[derive(Clone, Copy)]
enum Control {
    Swatches,
    Slider,
    Sheet,
    Segmented,
    Dropdown,
    Toggle,
}

struct RowCtx<'a> {
    context: &'a egui::Context,
    painter: &'a egui::Painter,
    settings: &'a SettingsModel,
    setting: &'a SettingsRow,
    language: &'a str,
    metrics: UiMetrics,
    row_metrics: UiMetrics,
    compact: bool,
    tv: bool,
    node: u64,
    row_rect: Rect,
    text_left: f32,
    text_center_y: f32,
    value_size: f32,
    value_width: f32,
    value_font: FontId,
    value: Option<String>,
    raw_value: Option<&'a str>,
    option_labels: Vec<String>,
    value_below: bool,
    is_app_icon: bool,
}

fn swatches(cx: &RowCtx, layout: &mut HomeLayout) {
    let RowCtx {
        context,
        setting,
        language,
        row_rect,
        text_left,
        row_metrics,
        metrics,
        node,
        raw_value,
        ..
    } = *cx;
    let current = raw_value.unwrap_or_default().to_ascii_uppercase();
    let count = setting.choices().len() as f32;
    let gap = metrics.settings_swatch_gap;
    let available = row_rect.right() - text_left - 4.0;
    let size = 40.0_f32.min((available - gap * (count - 1.0)) / count);
    let top = row_rect.top() + row_metrics.settings_row_height + 2.0;
    let strip = Rect::from_min_size(
        Pos2::new(text_left, top),
        Vec2::new(count * size + (count - 1.0) * gap, size),
    );
    for slot in 0..setting.choices().len() {
        layout.focusable.push((
            NODE_SETTINGS_SWATCH_BASE + slot as u64,
            Rect::from_min_size(
                strip.min + Vec2::new(slot as f32 * (size + gap), 0.0),
                Vec2::splat(size),
            )
            .expand(gap * 0.5),
        ));
    }
    components::placed_front(
        context,
        Id::new(("fluxa-settings-swatches", node)),
        strip.min,
        |ui| {
            ui.spacing_mut().item_spacing.x = gap;
            ui.horizontal(|ui| {
                for option in setting.choices() {
                    let (rect, response) =
                        ui.allocate_exact_size(Vec2::splat(size), Sense::click());
                    let glass = *option == "glass";
                    let color = if glass {
                        ink(tone::dim())
                    } else {
                        Color32::from_hex(option).unwrap_or(text_primary())
                    };
                    let active = option.eq_ignore_ascii_case(&current);
                    let grow = crate::motion::toggle(
                        ui.ctx(),
                        Id::new(("fluxa-swatch", *option)),
                        active || response.hovered(),
                        crate::motion::fast(),
                    );
                    let center = rect.center();
                    ui.painter()
                        .circle_filled(center, size * 0.5 - 4.0 + 2.0 * grow, color);
                    if glass {
                        let radius = size * 0.5 - 4.0 + 2.0 * grow;
                        ui.painter().circle_stroke(
                            center,
                            radius,
                            egui::Stroke::new(stroke::icon(), ink(tone::body())),
                        );
                        ui.painter().circle_filled(
                            center - Vec2::splat(radius * 0.38),
                            radius * 0.26,
                            ink(tone::subtle()),
                        );
                    }
                    if active {
                        ui.painter().circle_stroke(
                            center,
                            size * 0.5,
                            egui::Stroke::new(stroke::emphasis(), text_primary()),
                        );
                    } else if response.hovered() {
                        ui.painter().circle_stroke(
                            center,
                            size * 0.5,
                            egui::Stroke::new(stroke::hairline(), ink(tone::dim())),
                        );
                    }
                    let name = match *option {
                        "glass" => "settings.liquid_glass",
                        "#FFFFFF" => "auto.white",
                        "#E50914" => "auto.red",
                        "#3F7CFF" => "auto.blue",
                        "#54D17A" => "auto.green",
                        "#FF8A3D" => "auto.orange",
                        "#C084FC" => "auto.purple",
                        _ => "",
                    };
                    let response = if name.is_empty() {
                        response
                    } else {
                        response.on_hover_text(localized(name, language))
                    };
                    if response.clicked() && !active {
                        layout.setting_change = Some((
                            setting.key.to_owned(),
                            serde_json::Value::String((*option).to_owned()),
                        ));
                    }
                }
            });
        },
    );
}

fn slider(cx: &RowCtx, layout: &mut HomeLayout, range: &SettingsRange) {
    let RowCtx {
        context,
        painter,
        settings,
        setting,
        language,
        metrics,
        row_metrics,
        compact,
        tv,
        node,
        row_rect,
        text_left,
        text_center_y,
        ..
    } = *cx;
    let current = settings
        .number_value(setting.key)
        .unwrap_or(range.min)
        .clamp(range.min, range.max);
    let drag_id = Id::new(("fluxa-settings-slider-drag", node));
    let dragging = context.data(|data| data.get_temp::<f64>(drag_id));
    let shown = dragging.unwrap_or(current);
    let readout = option_label(setting.key, &range.format(shown), language);
    let readout_font = crate::fonts::bold(metrics.screen_card_subtitle_size + 6.0);
    let steps = !compact && !tv;
    let step_size = 28.0;
    let mut right = row_rect.right() - 6.0;
    if steps {
        right -= step_size + 8.0;
    }
    let readout_galley =
        painter.layout_no_wrap(readout.clone(), readout_font.clone(), metrics.text_primary);
    let readout_width = readout_galley.size().x.max(48.0);
    draw_text(
        &painter,
        Pos2::new(right - readout_width * 0.5, text_center_y),
        Align2::CENTER_CENTER,
        readout,
        readout_font,
        metrics.text_primary,
        None,
    );
    if steps {
        let minus = Rect::from_center_size(
            Pos2::new(right - readout_width - 8.0 - step_size * 0.5, text_center_y),
            Vec2::splat(step_size),
        );
        let plus = Rect::from_center_size(
            Pos2::new(right + 8.0 + step_size * 0.5, text_center_y),
            Vec2::splat(step_size),
        );
        for (rect, sign) in [(minus, -1.0), (plus, 1.0)] {
            let target = range.shift(current, sign as i32);
            let enabled = target.is_some();
            let mut pressed = false;
            components::placed_front(
                context,
                Id::new(("fluxa-settings-step", node, sign as i32)),
                rect.min,
                |ui| {
                    pressed = components::step_button(ui, step_size, sign > 0.0, enabled).clicked();
                },
            );
            if let (true, Some(target)) = (pressed, target) {
                layout.setting_change = Some((
                    setting.key.to_owned(),
                    serde_json::Value::String(range.format(target)),
                ));
            }
        }
    }
    let knob = 11.0;
    let track = Rect::from_min_size(
        Pos2::new(
            text_left + knob,
            row_rect.top() + row_metrics.settings_row_height - 4.0,
        ),
        Vec2::new(row_rect.right() - text_left - knob * 2.0 - 4.0, 20.0),
    );
    let mark_font = crate::fonts::regular(metrics.screen_card_subtitle_size);
    for (position, mark) in range.marks.iter().enumerate() {
        let (x, align) = if position == 0 {
            (track.left(), Align2::LEFT_TOP)
        } else if position + 1 == range.marks.len() {
            (track.right(), Align2::RIGHT_TOP)
        } else {
            (
                track.left() + track.width() * range.ratio(*mark),
                Align2::CENTER_TOP,
            )
        };
        draw_text(
            &painter,
            Pos2::new(x, track.center().y + 14.0),
            align,
            option_label(setting.key, &range.format(*mark), language),
            mark_font.clone(),
            ink(tone::muted()),
            None,
        );
    }
    let mut picked = None;
    components::placed_front(
        context,
        Id::new(("fluxa-settings-slider", node)),
        track.min,
        |ui| {
            ui.set_min_size(track.size());
            let output = components::slider(
                ui,
                &ui.painter().clone(),
                components::Slider {
                    id: Id::new(("settings-slider", node)),
                    track,
                    ratio: range.ratio(shown),
                    fill: metrics.accent,
                    overdrive: None,
                    amount: 1.0,
                    height: 8.0,
                    knob,
                    bubble: Some(option_label(setting.key, &range.format(shown), language)),
                },
            );
            let value = output.value.map(|ratio| range.snap(ratio));
            if let Some(value) = value {
                ui.ctx().data_mut(|data| data.insert_temp(drag_id, value));
            }
            if output.response.drag_stopped() || output.response.clicked() {
                picked = value.or(dragging);
                ui.ctx().data_mut(|data| data.remove::<f64>(drag_id));
            }
        },
    );
    if let Some(value) = picked {
        layout.setting_change = Some((
            setting.key.to_owned(),
            serde_json::Value::String(range.format(value)),
        ));
    }
}

fn sheet(cx: &RowCtx) {
    let RowCtx {
        painter,
        metrics,
        row_rect,
        value_below,
        is_app_icon,
        ..
    } = *cx;
    let value_font = cx.value_font.clone();
    if !value_below && !is_app_icon {
        let tip = row_rect.right_center() - Vec2::new(6.0, 0.0);
        let stroke = egui::Stroke::new(stroke::icon(), ink(tone::muted()));
        painter.line_segment([tip + Vec2::new(-5.0, -6.0), tip], stroke);
        painter.line_segment([tip + Vec2::new(-5.0, 6.0), tip], stroke);
    }
    if !value_below && !is_app_icon {
        let tip = row_rect.right_center() - Vec2::new(6.0, 0.0);
        draw_text(
            &painter,
            tip - Vec2::new(16.0, 0.0),
            Align2::RIGHT_CENTER,
            truncate_to_width(
                &painter,
                cx.value.as_deref().unwrap_or_default(),
                &cx.value_font,
                row_rect.width() * 0.4,
            ),
            value_font,
            metrics.text_secondary,
            None,
        );
    }
}

fn segmented_control(cx: &RowCtx, layout: &mut HomeLayout) {
    let RowCtx {
        context,
        setting,
        metrics,
        node,
        row_rect,
        value_width,
        raw_value,
        ..
    } = *cx;
    let raw_value = raw_value.unwrap_or_default();
    let selected = setting
        .choices()
        .iter()
        .position(|option| *option == raw_value)
        .unwrap_or_default();
    let height = metrics.screen_control_height;
    let origin = Pos2::new(
        row_rect.right() - 2.0 - value_width,
        row_rect.center().y - height * 0.5,
    );
    let labels = cx
        .option_labels
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>();
    components::placed_front(
        context,
        Id::new(("fluxa-settings-segmented", node)),
        origin,
        |ui| {
            ui.set_max_width(value_width);
            let picked = components::segmented(
                ui,
                Id::new(("settings-segment", node)),
                &labels,
                selected,
                metrics,
            );
            layout.focusable.push((node, ui.min_rect()));
            if let Some(index) = picked {
                layout.setting_change = Some((
                    setting.key.to_owned(),
                    serde_json::Value::String(setting.choices()[index].to_owned()),
                ));
            }
        },
    );
}

fn dropdown(cx: &RowCtx, layout: &mut HomeLayout) {
    let RowCtx {
        context,
        setting,
        language,
        metrics,
        node,
        row_rect,
        value_width,
        value_size,
        raw_value,
        ..
    } = *cx;
    let raw_value = raw_value.unwrap_or_default();
    let value = cx.value.clone().unwrap_or_default();
    let value_rect = Rect::from_center_size(
        row_rect.right_center() - Vec2::new(value_width * 0.5 + 2.0, 0.0),
        Vec2::new(
            value_width,
            metrics.screen_control_height.min(row_rect.height() - 4.0),
        ),
    );
    let mut selected = raw_value.to_owned();
    components::placed_front(
        context,
        Id::new(("fluxa-settings-dropdown", node)),
        value_rect.min,
        |ui| {
            ui.set_min_size(value_rect.size());
            ui.set_max_size(value_rect.size());
            let choices = setting
                .choices()
                .iter()
                .map(|option| {
                    let label = option_label(setting.key, option, language);
                    ((*option).to_owned(), label)
                })
                .collect::<Vec<_>>();
            let response = components::choice_field(
                ui,
                Id::new(("settings-choice", node)),
                value_rect.size(),
                value_size,
                &value,
                &choices,
                &mut selected,
                components::dropdown_popup_width(ui, &choices, value_rect.width(), metrics),
                true,
                false,
                metrics,
            );
            layout.focusable.push((node, response.rect));
        },
    );
    if selected != raw_value {
        layout.setting_change = Some((setting.key.to_owned(), serde_json::Value::String(selected)));
    }
}

fn toggle(cx: &RowCtx) {
    let RowCtx {
        context,
        painter,
        settings,
        setting,
        metrics,
        node,
        row_rect,
        ..
    } = *cx;
    let enabled = settings.bool_value(setting.key);
    let track_rect = Rect::from_center_size(
        row_rect.right_center() - Vec2::new(metrics.settings_toggle_width * 0.5 + 3.0, 0.0),
        Vec2::new(
            metrics.settings_toggle_width,
            metrics.settings_toggle_height,
        ),
    );
    components::toggle(
        context,
        &painter,
        Id::new(("fluxa-settings-toggle", node)),
        track_rect,
        enabled,
        metrics,
    );
}
