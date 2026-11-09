use super::page::Frame;
use super::*;
use crate::dp::Dp;
use crate::flex::{self, Align, Insets, Justify};
use egui::Ui;

pub(super) struct PlacedRow {
    index: usize,
    card: Rect,
    top: f32,
    extra: f32,
    first: bool,
}

pub(super) fn chevron(painter: &egui::Painter, bounds: Rect, color: Color32) {
    let tip = bounds.right_center();
    let stroke = egui::Stroke::new(stroke::icon(), color);
    painter.line_segment([bounds.left_top(), tip], stroke);
    painter.line_segment([bounds.left_bottom(), tip], stroke);
}

const LABEL: usize = 1;
const SUBTITLE: usize = 2;
const SLOT: usize = 3;
const MINUS: usize = 4;
const READOUT: usize = 5;
const PLUS: usize = 6;
const TRACK: usize = 7;
const VALUE: usize = 8;
const CHEVRON: usize = 9;

pub(super) fn place_rows(
    ui: &mut Ui,
    card: Rect,
    rows: &[usize],
    language: &str,
    metrics: UiMetrics,
) -> Vec<PlacedRow> {
    let extras: Vec<f32> = rows
        .iter()
        .map(|&index| row_extra(ui.painter(), index, card.width(), language, metrics))
        .collect();
    let stack = flex::column(extras.iter().enumerate().map(|(position, extra)| {
        flex::leaf(Vec2::ZERO)
            .width(card.width())
            .height(metrics.settings_row_spacing + extra)
            .id(position)
    }))
    .padding(Insets {
        top: Space::S6.px(&metrics),
        ..Insets::default()
    });
    let placed = stack.solve(card);
    rows.iter()
        .zip(&extras)
        .enumerate()
        .map(|(position, (&index, &extra))| PlacedRow {
            index,
            card,
            top: placed
                .find(position)
                .map_or(0.0, |slot| slot.min.y - card.top()),
            extra,
            first: position == 0,
        })
        .collect()
}

pub(super) fn description_overflows(
    painter: &egui::Painter,
    setting: &SettingsRow,
    language: &str,
    row_width: f32,
    metrics: UiMetrics,
) -> bool {
    if setting.key == APP_ICON_KEY {
        return false;
    }
    let labels = setting
        .choices()
        .iter()
        .map(|option| option_label(setting.key, option, language))
        .collect::<Vec<_>>();
    let control = if labels.is_empty() {
        metrics.settings_toggle_width + Dp::D3.px()
    } else if (2..=3).contains(&labels.len()) {
        components::segmented_width(painter, &labels, metrics) + Space::S2.px(&metrics)
    } else {
        let value_font = crate::fonts::regular(metrics.settings_row_value_size_desktop);
        let widest = labels
            .iter()
            .map(|label| crate::paint::text_width(painter, label, value_font.clone()))
            .fold(0.0, f32::max);
        (widest + Dp::D48.px())
            .clamp(0.0, row_width * metrics.settings_value_max_share)
            .max(Dp::D96.px())
            + Space::S2.px(&metrics)
    };
    let text_width = row_width - control - Space::S12.px(&metrics) - Space::S4.px(&metrics);
    let description = localized(
        &format!("settings.row.{}.description", setting.key),
        language,
    );
    let font =
        crate::fonts::regular(metrics.screen_card_subtitle_size + metrics.settings_subtitle_boost);
    crate::paint::text_width(painter, &description, font) > text_width
}

pub(super) fn draw_row(
    frame: Frame,
    assets: &mut impl HomeAssets,
    layout: &mut HomeLayout,
    row: PlacedRow,
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
    let PlacedRow {
        index,
        card,
        top,
        extra,
        first,
    } = row;
    let compact = viewport.is_compact();
    let tv = viewport.is_tv();
    let row_metrics = described(metrics);
    let Some(setting) = settings_row_by_index(index) else {
        return;
    };
    let node = NODE_SETTINGS_ROW_BASE + index as u64;
    let is_swatches = setting.key == ACCENT_COLOR_KEY;
    let range = slider_range(setting.key);
    let row_height = row_metrics.settings_row_height + extra;
    let row_rect = flex::column([flex::leaf(Vec2::ZERO)
        .width(card.width() - metrics.settings_row_inset * 2.0)
        .height(row_height)
        .id(0)])
    .padding(Insets {
        left: metrics.settings_row_inset,
        top,
        ..Insets::default()
    })
    .solve(Rect::from_min_size(
        card.left_top(),
        Vec2::new(card.width(), top + row_height),
    ))
    .find(0)
    .unwrap_or(Rect::NOTHING);
    let wrapped = extra > 0.0 && !is_swatches && range.is_none();
    let zone = flex::row([flex::leaf(Vec2::ZERO)
        .width(row_rect.width())
        .height(if wrapped {
            row_rect.height()
        } else {
            row_metrics.settings_row_height
        })
        .id(0)])
    .solve(row_rect)
    .find(0)
    .unwrap_or(row_rect);
    if !first {
        let y = row_rect.top()
            - (row_metrics.settings_row_spacing - row_metrics.settings_row_height) * 0.5;
        painter.line_segment(
            [
                Pos2::new(row_rect.left(), y),
                Pos2::new(row_rect.right(), y),
            ],
            egui::Stroke::new(
                stroke::hairline(),
                crate::stylesheet::look(context, "row.divider").border,
            ),
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
        metrics.nav_label_size + Dp::D2.px()
    };
    let subtitle_size = metrics.screen_card_subtitle_size + metrics.settings_subtitle_boost;
    let value_size = if !compact && !tv {
        metrics.settings_row_value_size_desktop
    } else {
        metrics.screen_card_subtitle_size + Dp::D1.px()
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
            flex::text_size(&painter, value, &value_font)
                .x
                .min(row_rect.width() * metrics.settings_sheet_value_share)
        }) + Dp::D20.px()
    } else {
        setting
            .choices()
            .iter()
            .map(|option| {
                flex::text_size(
                    &painter,
                    &option_label(setting.key, option, language),
                    &value_font,
                )
                .x
            })
            .fold(0.0, f32::max)
            + Dp::D48.px()
    }
    .clamp(0.0, row_rect.width() * metrics.settings_value_max_share)
    .max(if sheet_row { 0.0 } else { Dp::D96.px() });
    let value_width = if value_below || is_app_icon {
        0.0
    } else {
        value_width
    };
    let app_icon_size = (row_rect.height() - Dp::D16.px()).min(Dp::D44.px());
    let control_width = if is_swatches {
        0.0
    } else if range.is_some() {
        Dp::D64.px()
    } else if is_app_icon {
        app_icon_size + Space::S4.px(&metrics)
    } else if is_toggle {
        metrics.settings_toggle_width + Dp::D3.px()
    } else {
        value_width + Space::S2.px(&metrics)
    };
    let label_font = crate::fonts::regular(row_label_size);
    let subtitle_font = crate::fonts::regular(subtitle_size);
    let subtitle_height = subtitle_size
        + if wrapped {
            metrics.settings_row_subtitle_extra
        } else {
            0.0
        };
    let texts = flex::column([
        flex::leaf(Vec2::new(0.0, row_label_size)).id(LABEL),
        flex::leaf(Vec2::new(0.0, subtitle_height)).id(SUBTITLE),
    ])
    .align(Align::Stretch)
    .gap(Space::S4.px(&metrics))
    .grow(1.0);
    let line = flex::row([texts, flex::leaf(Vec2::ZERO).width(control_width).id(SLOT)])
        .align(Align::Center)
        .gap(Space::S12.px(&metrics))
        .padding(Insets {
            left: Space::S4.px(&metrics),
            ..Insets::default()
        })
        .solve(zone);
    let label_rect = line.find(LABEL).unwrap_or(zone);
    let subtitle_rect = line.find(SUBTITLE).unwrap_or(zone);
    let slot = line.find(SLOT).unwrap_or(zone);
    let text_width = label_rect.width().max(1.0);
    flex::paint_text(
        &painter,
        label_rect,
        &settings_row_label(setting, language),
        &label_font,
        metrics.text_primary,
        Align2::LEFT_TOP,
    );
    if is_app_icon {
        if let Some(texture) = raw_value.and_then(|id| assets.app_icon(id)) {
            painter.image(
                texture,
                flex::anchored(
                    slot,
                    Align2::RIGHT_CENTER,
                    Vec2::splat(app_icon_size),
                    Space::S4.px(&metrics),
                ),
                full_uv(),
                Color32::WHITE,
            );
        }
    }
    if wrapped {
        let wrapped_tint = crate::stylesheet::look(context, "row.description.wrapped").text;
        painter.galley(
            subtitle_rect.min,
            wrapped_regular(
                &painter,
                &localized(
                    &format!("settings.row.{}.description", setting.key),
                    language,
                ),
                subtitle_size,
                wrapped_tint,
                text_width,
                2,
            ),
            wrapped_tint,
        );
    } else {
        let below = value_below || is_app_icon || is_swatches;
        flex::paint_text(
            &painter,
            subtitle_rect,
            &if below {
                value.clone().unwrap_or_default()
            } else {
                localized(
                    &format!("settings.row.{}.description", setting.key),
                    language,
                )
            },
            &subtitle_font,
            crate::stylesheet::look(
                context,
                if below {
                    "row.value"
                } else if compact {
                    "row.description.compact"
                } else {
                    "row.description"
                },
            )
            .text,
            Align2::LEFT_TOP,
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
        zone,
        slot,
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
    zone: Rect,
    slot: Rect,
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
        row_metrics,
        metrics,
        node,
        raw_value,
        ..
    } = *cx;
    let current = raw_value.unwrap_or_default().to_ascii_uppercase();
    let count = setting.choices().len() as f32;
    let gap = metrics.settings_swatch_gap;
    let edge = Space::S4.px(&metrics);
    let available = row_rect.width() - edge * 2.0;
    let size = Dp::D40.px().min((available - gap * (count - 1.0)) / count);
    let strip =
        flex::row((0..setting.choices().len()).map(|slot| flex::leaf(Vec2::splat(size)).id(slot)))
            .gap(gap)
            .padding(Insets {
                left: edge,
                top: row_metrics.settings_row_height + Space::S2.px(&metrics),
                ..Insets::default()
            })
            .solve(row_rect);
    let cells = strip.tagged();
    for (slot, rect) in &cells {
        layout.focusable.push((
            NODE_SETTINGS_SWATCH_BASE + *slot as u64,
            rect.expand(gap * 0.5),
        ));
    }
    let Some(origin) = cells.first().map(|(_, rect)| rect.min) else {
        return;
    };
    components::placed_front(
        context,
        Id::new(("fluxa-settings-swatches", node)),
        origin,
        |ui| {
            for (option, (_, cell)) in setting.choices().iter().zip(&cells) {
                let response = ui.allocate_rect(*cell, Sense::click());
                let rect = *cell;
                let glass = *option == "glass";
                let glass_look = crate::stylesheet::look(ui.ctx(), "swatch.glass");
                let color = if glass {
                    glass_look.fill
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
                let pop = Space::S2.px(&metrics);
                let ring = size * 0.5 - Space::S4.px(&metrics) + pop * grow;
                ui.painter().circle_filled(center, ring, color);
                if glass {
                    ui.painter().circle_stroke(
                        center,
                        ring,
                        egui::Stroke::new(stroke::icon(), glass_look.border),
                    );
                    ui.painter().circle_filled(
                        center - Vec2::splat(ring * 0.38),
                        ring * 0.26,
                        glass_look.text,
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
                        egui::Stroke::new(
                            stroke::hairline(),
                            crate::stylesheet::look(ui.ctx(), "swatch.hover").border,
                        ),
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
        compact,
        tv,
        node,
        row_rect,
        zone,
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
    let readout_font = crate::fonts::bold(metrics.screen_card_subtitle_size + Dp::D6.px());
    let steps = !compact && !tv;
    let step_size = Dp::D28.px();
    let readout_size = flex::text_size(&painter, &readout, &readout_font);
    let mut header = Vec::new();
    if steps {
        header.push(flex::leaf(Vec2::splat(step_size)).id(MINUS));
    }
    header
        .push(flex::leaf(Vec2::new(readout_size.x.max(Dp::D48.px()), readout_size.y)).id(READOUT));
    if steps {
        header.push(flex::leaf(Vec2::splat(step_size)).id(PLUS));
    }
    let header = flex::row(header)
        .align(Align::Center)
        .justify(Justify::End)
        .gap(Space::S8.px(&metrics))
        .padding(Insets {
            right: Space::S6.px(&metrics),
            ..Insets::default()
        })
        .solve(zone);
    if let Some(rect) = header.find(READOUT) {
        flex::paint_text(
            &painter,
            rect,
            &readout,
            &readout_font,
            metrics.text_primary,
            Align2::CENTER_CENTER,
        );
    }
    for (id, sign) in [(MINUS, -1.0), (PLUS, 1.0)] {
        let Some(rect) = header.find(id) else {
            continue;
        };
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
    let knob = Dp::D10.px();
    let edge = Space::S4.px(&metrics);
    let track = flex::column([flex::leaf(Vec2::new(0.0, Dp::D20.px())).id(TRACK)])
        .align(Align::Stretch)
        .padding(Insets {
            left: edge + knob,
            right: edge + knob,
            top: cx.row_metrics.settings_row_height - edge,
            bottom: 0.0,
        })
        .solve(row_rect)
        .find(TRACK)
        .unwrap_or(row_rect);
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
            Pos2::new(x, track.center().y + Dp::D14.px()),
            align,
            option_label(setting.key, &range.format(*mark), language),
            mark_font.clone(),
            crate::stylesheet::look(context, "row.mark").text,
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
                    height: Dp::D8.px(),
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
        zone,
        value_below,
        is_app_icon,
        ..
    } = *cx;
    if value_below || is_app_icon {
        return;
    }
    let value = cx.value.as_deref().unwrap_or_default();
    let text = flex::text_size(&painter, value, &cx.value_font);
    let line = flex::row([
        flex::leaf(Vec2::new(
            text.x
                .min(zone.width() * metrics.settings_sheet_value_share),
            text.y,
        ))
        .id(VALUE),
        flex::leaf(Vec2::new(Dp::D4.px(), Dp::D6.px() * 2.0)).id(CHEVRON),
    ])
    .align(Align::Center)
    .justify(Justify::End)
    .gap(Dp::D10.px())
    .padding(Insets {
        right: Space::S6.px(&metrics),
        ..Insets::default()
    })
    .solve(zone);
    if let Some(chevron) = line.find(CHEVRON) {
        self::chevron(
            painter,
            chevron,
            crate::stylesheet::look(painter.ctx(), "row.chevron").border,
        );
    }
    if let Some(rect) = line.find(VALUE) {
        flex::paint_text(
            &painter,
            rect,
            value,
            &cx.value_font,
            metrics.text_secondary,
            Align2::RIGHT_CENTER,
        );
    }
}

fn segmented_control(cx: &RowCtx, layout: &mut HomeLayout) {
    let RowCtx {
        context,
        setting,
        metrics,
        node,
        slot,
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
    let origin = flex::anchored(
        slot,
        Align2::LEFT_CENTER,
        Vec2::new(value_width, metrics.screen_control_height),
        0.0,
    )
    .min;
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
        slot,
        zone,
        value_width,
        value_size,
        raw_value,
        ..
    } = *cx;
    let raw_value = raw_value.unwrap_or_default();
    let value = cx.value.clone().unwrap_or_default();
    let value_rect = flex::anchored(
        slot,
        Align2::LEFT_CENTER,
        Vec2::new(
            value_width,
            metrics
                .screen_control_height
                .min(zone.height() - Space::S4.px(&metrics)),
        ),
        0.0,
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
        slot,
        ..
    } = *cx;
    let enabled = settings.bool_value(setting.key);
    let track_rect = flex::anchored(
        slot,
        Align2::LEFT_CENTER,
        Vec2::new(
            metrics.settings_toggle_width,
            metrics.settings_toggle_height,
        ),
        0.0,
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
