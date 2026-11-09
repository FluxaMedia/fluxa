use super::*;
use crate::kit::UiKit as _;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct PanelRow {
    pub label: String,
    pub value: Option<String>,
    pub primary: bool,
    pub selected: bool,
    pub heading: bool,
    pub group: bool,
    pub switch: Option<bool>,
    pub thumbnail: Option<String>,
    pub detail: Option<String>,
    pub frame: Option<f64>,
    pub badge: Option<String>,
    pub icon: Option<String>,
    pub chevron: bool,
    pub step: Option<PanelStep>,
    pub sync: Option<SyncRole>,
    pub subline: Option<String>,
    pub separated: bool,
    pub back: bool,
    pub chips: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SyncRole {
    Back,
    Reset,
    Reading,
    Minus,
    Plus,
    Auto,
    Method,
    Choice,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PanelStep {
    Value,
    Minus,
    Plus,
    Swatches(usize),
    Swatch,
    Choices(usize),
    Choice,
}

impl PanelStep {
    fn follows_lead(self) -> bool {
        matches!(self, Self::Minus | Self::Plus | Self::Swatch | Self::Choice)
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct PlayerPanel {
    pub title: String,
    pub message: Option<String>,
    pub rows: Vec<PanelRow>,
    pub list: bool,
    pub look: Option<crate::SubtitleLook>,
    pub chips: bool,
}

pub(super) fn draw_panel(
    chrome: &Chrome,
    ui: &mut egui::Ui,
    layout: &mut HomeLayout,
    panel: &PlayerPanel,
    assets: &mut impl HomeAssets,
    slide: f32,
) {
    if panel.rows.iter().any(|row| row.sync.is_some()) {
        super::sync_panel::draw(chrome, ui, layout, panel, slide);
        return;
    }
    if panel.chips && chrome.viewport.form_factor == UiFormFactor::Mobile {
        draw_chips(chrome, ui, layout, panel, slide);
        return;
    }
    if panel.list {
        draw_list(chrome, ui, layout, panel, assets, slide);
        return;
    }
    let rows = panel.rows.len().min(PLAYER_PANEL_ROW_LIMIT);
    let language = chrome.player.language.clone();
    crate::kit::provide(chrome.context, UiMetrics::for_viewport(chrome.viewport));
    crate::kit::dialog(chrome.context, "fluxa-player-panel", |ui| {
        crate::kit::text(&panel.title, crate::kit::Type::Subtitle)
            .bold()
            .show(ui);
        if let Some(message) = panel.message.as_deref() {
            crate::kit::space(ui, Space::S6);
            crate::kit::text(message, crate::kit::Type::Meta)
                .role(crate::kit::Role::Secondary)
                .show(ui);
        }
        crate::kit::space(ui, Space::S14);
        for (index, row) in panel.rows.iter().take(rows).enumerate() {
            let label = match row.value.as_deref() {
                Some(value) => format!("{}  {}", row.label, value),
                None => row.label.clone(),
            };
            let button = crate::kit::button(&label);
            let response = if row.primary {
                button.primary()
            } else {
                button
            }
            .show(ui);
            crate::components::focus_action(
                layout,
                NODE_PLAYER_PANEL_ROW_BASE + index as u64,
                &response,
            );
            crate::kit::space(ui, Space::S8);
        }
        let close = crate::kit::button(&localized("common.close", &language)).show(ui);
        crate::components::focus_action(layout, NODE_PLAYER_PANEL_CLOSE, &close);
    });
}

fn draw_list(
    chrome: &Chrome,
    ui: &mut egui::Ui,
    layout: &mut HomeLayout,
    panel: &PlayerPanel,
    assets: &mut impl HomeAssets,
    slide: f32,
) {
    let tokens = UiMetrics::for_viewport(chrome.viewport);
    let form_factor = chrome.viewport.form_factor;
    let mut row_h = tokens.button_height;
    let text = tokens.text.body;
    let pad = Space::S14.px(&tokens);
    let mut base = tokens.player_panel_narrow.of(chrome.rect.width());
    let rich = panel
        .rows
        .iter()
        .any(|row| row.thumbnail.is_some() || row.detail.is_some());
    if rich {
        row_h = tokens.button_height * 2.0;
        base = tokens.player_panel_wide.of(chrome.rect.width());
    } else if panel
        .rows
        .iter()
        .any(|row| row.subline.is_some() && !row.chips.is_empty())
    {
        row_h = tokens.button_height + Space::S20.px(&tokens);
    }
    let mut columns: Vec<(Option<&str>, Vec<(usize, &PanelRow)>)> = Vec::new();
    let mut back_row: Option<usize> = None;
    for (index, row) in panel.rows.iter().enumerate().take(PLAYER_PANEL_ROW_LIMIT) {
        if row.back {
            back_row = Some(index);
        } else if row.heading {
            columns.push((Some(row.label.as_str()), Vec::new()));
        } else if row.step.is_some_and(PanelStep::follows_lead) {
            continue;
        } else {
            if columns.is_empty() {
                columns.push((None, Vec::new()));
            }
            if let Some(column) = columns.last_mut() {
                column.1.push((index, row));
            }
        }
    }
    let count = columns.len().max(1) as f32;
    let gap = pad;
    let available = chrome.rect.width() - chrome.margin() * 2.0;
    let mobile = form_factor == UiFormFactor::Mobile;
    let drawer = mobile && panel.look.is_some();
    let sheet = mobile && !drawer;
    if drawer {
        base = base.max(300.0 - pad * 2.0);
    }
    let usable = if sheet {
        available.min(560.0)
    } else {
        available
    };
    let fit_width = (usable - pad * 2.0 - gap * (count - 1.0)) / count;
    let col_w = if sheet {
        fit_width
    } else {
        fit_width.min(base)
    };
    let width = if drawer {
        300.0_f32.min(available)
    } else {
        col_w * count + gap * (count - 1.0) + pad * 2.0
    };
    let col_w = if drawer { width - pad * 2.0 } else { col_w };
    let title_h = if panel.title.is_empty() && back_row.is_none() {
        0.0
    } else {
        row_h * if back_row.is_some() { 1.25 } else { 1.0 }
    };
    let head_h = if columns.iter().any(|(heading, _)| heading.is_some()) {
        row_h * 0.8
    } else {
        0.0
    };
    let reserved = match form_factor {
        UiFormFactor::Desktop => 120.0 + chrome.viewport.safe_bottom,
        _ if sheet => chrome.viewport.safe_bottom,
        _ => 32.0,
    };
    let available_height = if sheet {
        chrome.rect.height() * 0.78
    } else if panel.look.is_some() && form_factor != UiFormFactor::Desktop {
        chrome.rect.height() - chrome.margin()
    } else {
        chrome.rect.height()
    };
    let max_body = available_height
        - reserved
        - chrome.margin()
        - pad * 2.0
        - 12.0 * sheet as u8 as f32
        - title_h
        - head_h;
    let longest = columns
        .iter()
        .map(|(_, rows)| rows.len())
        .max()
        .unwrap_or(0);
    let visible = (((max_body / row_h).floor() as usize).max(3)).min(longest.max(1));
    let handle_h = if sheet { 12.0 } else { 0.0 };
    let height = pad * 2.0 + handle_h + title_h + head_h + visible as f32 * row_h;
    let card = match form_factor {
        UiFormFactor::Desktop => Rect::from_min_size(
            Pos2::new(
                chrome.rect.right() - chrome.margin() - width,
                chrome.rect.bottom() - reserved - height,
            ),
            Vec2::new(width, height),
        ),
        _ if drawer => Rect::from_min_max(
            Pos2::new(chrome.rect.right() - width, chrome.rect.top()),
            Pos2::new(chrome.rect.right(), chrome.rect.bottom()),
        ),
        _ if sheet => Rect::from_min_max(
            Pos2::new(
                chrome.rect.center().x - width * 0.5,
                chrome.rect.bottom() - height - reserved,
            ),
            Pos2::new(chrome.rect.center().x + width * 0.5, chrome.rect.bottom()),
        ),
        _ if panel.look.is_some() => Rect::from_min_size(
            Pos2::new(
                chrome.rect.center().x - width * 0.5,
                chrome.rect.top() + chrome.margin(),
            ),
            Vec2::new(width, height),
        ),
        _ => Rect::from_center_size(chrome.rect.center(), Vec2::new(width, height)),
    };
    let veil = chrome.modal_veil(ui, card, layout);
    if let Some(look) = panel.look.as_ref() {
        let lift = if chrome.player.controls_visible {
            super::tokens::subtitle_lift(form_factor)
        } else {
            0.0
        };
        let bottom = chrome.rect.bottom() - chrome.viewport.safe_bottom - 24.0 - lift;
        let right = if drawer {
            card.left()
        } else {
            chrome.rect.right()
        };
        let area = Rect::from_min_max(chrome.rect.min, Pos2::new(right, bottom));
        components::subtitle_sample(
            &veil,
            area,
            look,
            &localized("settings.subtitle_preview_text", &chrome.player.language),
            chrome.rect.height() * 0.05,
        );
    } else if form_factor != UiFormFactor::Desktop && !sheet {
        veil.rect_filled(chrome.rect, 0.0, shade(Shade::Veil));
    } else if sheet {
        veil.rect_filled(chrome.rect, 0.0, Color32::from_black_alpha(110));
    }
    let layer = veil.layer_id();
    let first = chrome
        .context
        .graphics_mut(|graphics| graphics.entry(layer).next_idx());
    let radius = if sheet {
        egui::CornerRadius {
            nw: 24,
            ne: 24,
            sw: 0,
            se: 0,
        }
    } else {
        egui::CornerRadius::same(tokens.card_radius as u8)
    };
    veil.rect_filled(card, radius, tokens.surface);
    veil.rect_stroke(
        card,
        radius,
        egui::Stroke::new(stroke::hairline(), tokens.border),
        egui::StrokeKind::Inside,
    );
    if sheet {
        veil.rect_filled(
            Rect::from_center_size(
                Pos2::new(card.center().x, card.top() + 9.0),
                Vec2::new(36.0, 4.0),
            ),
            2.0,
            crate::stylesheet::color("strong"),
        );
    }
    let content = Rect::from_min_max(
        Pos2::new(card.left() + pad, card.top() + pad + handle_h),
        card.max - Vec2::splat(pad),
    );
    let mut stack = flow_child(ui, content, egui::Layout::top_down(egui::Align::Min), 0.0);
    let header_rect = stack.allocate_space(Vec2::new(content.width(), title_h)).1;
    let mut header = flow_child(
        ui,
        header_rect,
        egui::Layout::left_to_right(egui::Align::Center),
        6.0,
    );
    let columns_rect = stack
        .allocate_space(Vec2::new(content.width(), head_h + visible as f32 * row_h))
        .1;
    let mut columns_row = flow_child(
        ui,
        columns_rect,
        egui::Layout::left_to_right(egui::Align::Min),
        gap,
    );
    if let Some(index) = back_row {
        let side = row_h * 0.85;
        let hit = header.allocate_space(Vec2::splat(side)).1;
        let node = NODE_PLAYER_PANEL_ROW_BASE + index as u64;
        let response = ui.interact(
            hit,
            Id::new(("fluxa-player-panel-back", node)),
            Sense::click(),
        );
        layout.focusable.push((node, hit));
        let on = chrome.focused == Some(node);
        if on || response.hovered() {
            veil.circle_filled(
                hit.center(),
                side * 0.5,
                if on {
                    text_primary()
                } else {
                    crate::stylesheet::color("track")
                },
            );
        }
        if let Some(texture) = chrome.icons.get("ArrowLeft") {
            veil.image(
                texture,
                Rect::from_center_size(hit.center(), Vec2::splat(text * 1.4)),
                full_uv(),
                if on { Color32::BLACK } else { ink(235) },
            );
        }
        if response.clicked() {
            layout.activated = Some(node);
        }
    } else {
        header.gap(Space::S8);
    }
    if title_h > 0.0 && !panel.title.is_empty() {
        let title_pos = header.allocate_space(Vec2::ZERO).1.left_center();
        veil.text(
            title_pos,
            Align2::LEFT_CENTER,
            &panel.title,
            FontId::proportional(text * 1.1),
            text_primary(),
        );
    }
    let check = chrome.icons.get("Check");
    for (column, (heading, rows)) in columns.iter().enumerate() {
        let slot = columns_row
            .allocate_space(Vec2::new(col_w, head_h + visible as f32 * row_h))
            .1;
        let mut column_ui = flow_child(ui, slot, egui::Layout::top_down(egui::Align::Min), 0.0);
        let heading_rect = column_ui.allocate_space(Vec2::new(col_w, head_h)).1;
        let rows_rect = column_ui
            .allocate_space(Vec2::new(col_w, visible as f32 * row_h))
            .1;
        if let Some(heading) = heading {
            let label = if sheet {
                heading.to_uppercase()
            } else {
                heading.to_string()
            };
            let mut heading_row = flow_child(
                ui,
                heading_rect,
                egui::Layout::left_to_right(egui::Align::Center),
                0.0,
            );
            heading_row.gap(Space::S8);
            let heading_pos = heading_row.allocate_space(Vec2::ZERO).1.left_center();
            veil.text(
                heading_pos,
                Align2::LEFT_CENTER,
                label,
                FontId::proportional(text * if sheet { 0.78 } else { 0.85 }),
                crate::stylesheet::color("glyph"),
            );
        }
        if sheet && column > 0 {
            let x = slot.left() - gap * 0.5;
            veil.line_segment(
                [
                    Pos2::new(x, slot.top() + 4.0),
                    Pos2::new(x, rows_rect.bottom()),
                ],
                egui::Stroke::new(stroke::hairline(), tokens.border),
            );
        }
        let state_id = Id::new(("fluxa-player-panel-offset", column));
        let mut start: usize = ui.data(|data| data.get_temp(state_id)).unwrap_or(0);
        let focused = rows
            .iter()
            .position(|(index, _)| {
                chrome.focused == Some(NODE_PLAYER_PANEL_ROW_BASE + *index as u64)
            })
            .or_else(|| {
                (chrome.focused.is_none() && start == 0)
                    .then(|| rows.iter().position(|(_, row)| row.selected))
                    .flatten()
            });
        let marked_id = Id::new(("fluxa-player-panel-marked", column));
        let marked = rows.iter().position(|(_, row)| row.selected);
        let previous: Option<Option<usize>> = ui.data(|data| data.get_temp(marked_id));
        ui.data_mut(|data| data.insert_temp(marked_id, marked));
        let followed = marked.filter(|_| previous.is_some_and(|before| before != marked));
        if let Some(position) = focused.or(followed) {
            if position < start {
                start = position;
            } else if position >= start + visible {
                start = position + 1 - visible;
            }
        }
        let column_rect = rows_rect;
        if ui.rect_contains_pointer(column_rect) {
            let scroll = ui.input(|input| input.smooth_scroll_delta.y);
            if scroll.abs() > 1.0 {
                let step = if scroll > 0.0 { -1 } else { 1 };
                start = (start as i32 + step).clamp(0, rows.len().saturating_sub(visible) as i32)
                    as usize;
            }
        }
        let drag = ui.interact(
            column_rect,
            Id::new(("fluxa-player-panel-drag", column)),
            Sense::drag(),
        );
        if drag.dragged() {
            let carry_id = Id::new(("fluxa-player-panel-carry", column));
            let carry: f32 = ui.data(|data| data.get_temp(carry_id)).unwrap_or(0.0)
                - drag.drag_delta().y / row_h;
            let whole = carry.trunc();
            ui.data_mut(|data| data.insert_temp(carry_id, carry - whole));
            start = (start as i32 + whole as i32).max(0) as usize;
        }
        start = start.min(rows.len().saturating_sub(visible));
        ui.data_mut(|data| data.insert_temp(state_id, start));
        let mut flow = ui.new_child(egui::UiBuilder::new().max_rect(Rect::from_min_size(
            Pos2::new(rows_rect.left(), rows_rect.top() - start as f32 * row_h),
            Vec2::new(col_w, row_h * rows.len() as f32),
        )));
        flow.spacing_mut().item_spacing = Vec2::ZERO;
        for (position, (index, row)) in rows.iter().enumerate() {
            let node = NODE_PLAYER_PANEL_ROW_BASE + *index as u64;
            let rect = flow.allocate_space(Vec2::new(col_w, row_h)).1;
            if row.group {
                if position >= start && position < start + visible {
                    veil.text(
                        Pos2::new(rect.left() + 8.0, rect.center().y + row_h * 0.12),
                        Align2::LEFT_CENTER,
                        if drawer {
                            row.label.to_uppercase()
                        } else {
                            row.label.clone()
                        },
                        FontId::proportional(text * 0.8),
                        crate::stylesheet::color("glyph"),
                    );
                }
                continue;
            }
            layout.focusable.push((node, rect));
            if position < start || position >= start + visible {
                continue;
            }
            let response = ui.interact(
                rect,
                Id::new(("fluxa-player-panel-row", node)),
                Sense::click(),
            );
            let on = chrome.focused == Some(node);
            let fill = if on {
                text_primary()
            } else if row.selected {
                crate::stylesheet::color("track")
            } else if response.hovered() {
                crate::stylesheet::color("well")
            } else {
                Color32::TRANSPARENT
            };
            veil.rect_filled(rect, row_h * 0.25, fill);
            if row.separated {
                let y = rect.top() - 1.0;
                veil.line_segment(
                    [
                        Pos2::new(rect.left() + 8.0, y),
                        Pos2::new(rect.right() - 8.0, y),
                    ],
                    egui::Stroke::new(stroke::hairline(), tokens.border),
                );
            }
            let ink = if on { Color32::BLACK } else { ink(255) };
            if rich {
                let mut body = flow_child(
                    ui,
                    Rect::from_min_max(
                        rect.min + Vec2::new(8.0, 8.0),
                        Pos2::new(rect.right() - 12.0, rect.bottom() - 8.0),
                    ),
                    egui::Layout::left_to_right(egui::Align::Min),
                    12.0,
                );
                let art = body
                    .allocate_space(Vec2::new((row_h - 16.0) * 16.0 / 9.0, row_h - 16.0))
                    .1;
                veil.rect_filled(art, radius::control(), crate::stylesheet::color("well"));
                if let Some(time) = row.frame {
                    let frame = chrome
                        .player
                        .filmstrip
                        .iter()
                        .filter(|frame| (frame.0 - time).abs() < 4.0)
                        .min_by(|a, b| (a.0 - time).abs().total_cmp(&(b.0 - time).abs()));
                    if let Some((_, texture, _)) = frame {
                        veil.image(*texture, art, full_uv(), Color32::WHITE);
                    }
                }
                components::rounded_artwork(
                    &veil,
                    art,
                    6.0,
                    row.thumbnail.as_deref(),
                    [320, 180],
                    ArtworkPriority::Visible,
                    Color32::WHITE,
                    assets,
                );
                let width = body.available_width();
                let left = body.allocate_space(Vec2::new(width, 0.0)).1.left();
                let title_font = FontId::proportional(text);
                veil.text(
                    Pos2::new(left, rect.top() + 12.0),
                    Align2::LEFT_TOP,
                    truncate_to_width(&veil, &row.label, &title_font, width),
                    title_font,
                    ink,
                );
                if let Some(detail) = row.detail.as_deref() {
                    let mut job = egui::text::LayoutJob::simple(
                        detail.to_owned(),
                        FontId::proportional(text * 0.85),
                        ink.gamma_multiply(0.6),
                        width,
                    );
                    job.wrap.max_rows = if form_factor == UiFormFactor::Mobile {
                        2
                    } else {
                        3
                    };
                    let galley = veil.layout_job(job);
                    veil.galley(Pos2::new(left, rect.top() + 12.0 + text * 1.5), galley, ink);
                }
                if response.clicked() {
                    layout.activated = Some(node);
                }
                continue;
            }
            if let Some(PanelStep::Swatches(count)) = row.step {
                let side = 20.0;
                let gap = 5.0;
                let mut swatches = flow_child(
                    ui,
                    rect.shrink2(Vec2::new(8.0, 0.0)),
                    egui::Layout::right_to_left(egui::Align::Center),
                    gap,
                );
                for offset in (1..=count).rev() {
                    let swatch = &panel.rows[*index + offset];
                    let step_node = NODE_PLAYER_PANEL_ROW_BASE + (*index + offset) as u64;
                    let hit = swatches.allocate_space(Vec2::splat(side)).1;
                    let response = ui.interact(
                        hit.expand(3.0),
                        Id::new(("fluxa-player-panel-step", step_node)),
                        Sense::click(),
                    );
                    layout.focusable.push((step_node, hit));
                    let color = swatch
                        .value
                        .as_deref()
                        .and_then(parse_hex)
                        .unwrap_or(Color32::GRAY);
                    veil.circle_filled(hit.center(), side * 0.5, color);
                    veil.circle_stroke(
                        hit.center(),
                        side * 0.5,
                        egui::Stroke::new(stroke::hairline(), Color32::from_white_alpha(60)),
                    );
                    if swatch.selected {
                        veil.circle_stroke(
                            hit.center(),
                            side * 0.5 + 3.0,
                            egui::Stroke::new(1.5, Color32::WHITE),
                        );
                    }
                    if response.clicked() {
                        layout.activated = Some(step_node);
                    }
                }
                veil.text(
                    Pos2::new(rect.left() + 8.0, rect.center().y),
                    Align2::LEFT_CENTER,
                    &row.label,
                    FontId::proportional(text),
                    ink,
                );
                continue;
            }
            if let Some(PanelStep::Choices(count)) = row.step {
                let font = FontId::proportional(text * 0.9);
                let gap = 6.0;
                let widths: Vec<f32> = (1..=count)
                    .map(|offset| {
                        crate::flex::text_size(&veil, &panel.rows[*index + offset].label, &font).x
                            + 22.0
                    })
                    .collect();
                let total = widths.iter().sum::<f32>() + gap * (count as f32 - 1.0);
                let mut chips = flow_child(
                    ui,
                    rect.shrink2(Vec2::new(8.0, 0.0)),
                    egui::Layout::left_to_right(egui::Align::Center),
                    0.0,
                );
                let spare = (chips.available_width() - total).max(0.0);
                chips.allocate_space(Vec2::new(spare, 0.0));
                chips.spacing_mut().item_spacing.x = gap;
                for (slot, width) in widths.iter().enumerate() {
                    let choice = &panel.rows[*index + 1 + slot];
                    let step_node = NODE_PLAYER_PANEL_ROW_BASE + (*index + 1 + slot) as u64;
                    let hit = chips.allocate_space(Vec2::new(*width, 30.0)).1;
                    let response = ui.interact(
                        hit,
                        Id::new(("fluxa-player-panel-step", step_node)),
                        Sense::click(),
                    );
                    layout.focusable.push((step_node, hit));
                    let (fill, color) = if choice.selected {
                        (text_primary(), Color32::BLACK)
                    } else {
                        (crate::stylesheet::color("well"), ink)
                    };
                    veil.rect_filled(hit, 15.0, fill);
                    veil.text(
                        hit.center(),
                        Align2::CENTER_CENTER,
                        &choice.label,
                        font.clone(),
                        color,
                    );
                    if response.clicked() {
                        layout.activated = Some(step_node);
                    }
                }
                if !row.label.is_empty() {
                    veil.text(
                        Pos2::new(rect.left() + 8.0, rect.center().y),
                        Align2::LEFT_CENTER,
                        &row.label,
                        FontId::proportional(text),
                        ink,
                    );
                }
                continue;
            }
            if row.step == Some(PanelStep::Value) {
                let font = FontId::proportional(text * 0.95);
                let button = 30.0;
                let mut stepper = flow_child(
                    ui,
                    rect.shrink2(Vec2::new(8.0, 0.0)),
                    egui::Layout::right_to_left(egui::Align::Center),
                    0.0,
                );
                let plus = stepper.allocate_space(Vec2::splat(button)).1;
                let value_slot = stepper.allocate_space(Vec2::new(62.0, button)).1;
                let minus = stepper.allocate_space(Vec2::splat(button)).1;
                for (hit, glyph, offset) in [(minus, "−", 1), (plus, "+", 2)] {
                    let step_node = NODE_PLAYER_PANEL_ROW_BASE + (*index + offset) as u64;
                    let step = ui.interact(
                        hit,
                        Id::new(("fluxa-player-panel-step", step_node)),
                        Sense::click(),
                    );
                    layout.focusable.push((step_node, hit));
                    veil.circle_filled(
                        hit.center(),
                        button * 0.5,
                        crate::ink(if step.is_pointer_button_down_on() {
                            tone::strong()
                        } else {
                            tone::soft()
                        }),
                    );
                    veil.text(
                        hit.center(),
                        Align2::CENTER_CENTER,
                        glyph,
                        FontId::proportional(text * 1.2),
                        ink,
                    );
                    if step.clicked() {
                        layout.activated = Some(step_node);
                    }
                }
                veil.text(
                    value_slot.center(),
                    Align2::CENTER_CENTER,
                    row.value.as_deref().unwrap_or_default(),
                    font.clone(),
                    ink,
                );
                veil.text(
                    Pos2::new(rect.left() + 8.0, rect.center().y),
                    Align2::LEFT_CENTER,
                    truncate_to_width(
                        &veil,
                        &row.label,
                        &font,
                        (minus.left() - rect.left() - 20.0).max(0.0),
                    ),
                    font,
                    ink,
                );
                continue;
            }
            let inner = Rect::from_min_max(
                rect.min + Vec2::new(8.0, 0.0),
                rect.max - Vec2::new(12.0, 0.0),
            );
            let mut trailing = flow_child(
                ui,
                inner,
                egui::Layout::right_to_left(egui::Align::Center),
                0.0,
            );
            let chevron = match chrome.icons.get("ChevronRight") {
                Some(texture) if row.chevron => {
                    let side = text * 1.1;
                    let slot = trailing.allocate_space(Vec2::splat(side)).1;
                    trailing.gap(Space::S8);
                    Some((texture, slot))
                }
                _ => None,
            };
            let switch = row.switch.map(|on_state| {
                let track = trailing.allocate_space(Vec2::new(40.0, 22.0)).1;
                trailing.gap(Space::S12);
                (on_state, track)
            });
            let check_mark = match check {
                Some(texture) if row.selected => {
                    let side = text * 1.2;
                    let slot = trailing.allocate_space(Vec2::splat(side)).1;
                    trailing.gap(Space::S8);
                    Some((texture, slot))
                }
                _ => None,
            };
            let value_text = row.value.as_deref().map(|value| {
                let font = FontId::proportional(text * 0.9);
                let value = truncate_to_width(&veil, value, &font, col_w * 0.4);
                let galley = veil.layout_no_wrap(value.to_string(), font, ink);
                let slot = trailing.allocate_space(galley.size()).1;
                trailing.gap(Space::S10);
                (galley, slot)
            });
            let right_limit = trailing.cursor().right();
            let mut leading = flow_child(
                ui,
                Rect::from_min_max(
                    rect.min + Vec2::new(8.0, 0.0),
                    Pos2::new(right_limit, rect.max.y),
                ),
                egui::Layout::left_to_right(egui::Align::Center),
                10.0,
            );
            let badge = row
                .badge
                .as_deref()
                .map(|badge| (badge, leading.allocate_space(Vec2::new(24.0, 16.0)).1));
            let icon = row
                .icon
                .as_deref()
                .and_then(|name| chrome.icons.get(name))
                .map(|texture| (texture, leading.allocate_space(Vec2::splat(text * 1.4)).1));
            if let Some((badge, pill)) = badge {
                let font = FontId::proportional(text * 0.62);
                veil.rect_stroke(
                    pill,
                    4.0,
                    egui::Stroke::new(stroke::hairline(), ink.gamma_multiply(0.5)),
                    egui::StrokeKind::Inside,
                );
                veil.text(
                    pill.center(),
                    Align2::CENTER_CENTER,
                    badge,
                    font,
                    ink.gamma_multiply(0.75),
                );
            }
            if let Some((texture, slot)) = icon {
                veil.image(texture, slot, full_uv(), ink);
            }
            if let Some((texture, slot)) = chevron {
                veil.image(texture, slot, full_uv(), ink);
            }
            if let Some((on_state, track)) = switch {
                components::toggle(
                    chrome.context,
                    &veil,
                    Id::new(("fluxa-player-panel-switch", node)),
                    track,
                    on_state,
                    tokens,
                );
            }
            if let Some((texture, slot)) = check_mark {
                veil.image(texture, slot, full_uv(), ink);
            }
            if let Some((galley, slot)) = value_text {
                veil.galley(slot.min, galley, ink);
            }
            let font = if row.selected {
                crate::fonts::bold(text * 1.08)
            } else {
                FontId::proportional(text)
            };
            let room = (leading.available_width() - 8.0).max(0.0);
            let left_edge = leading.allocate_space(Vec2::new(room, 0.0)).1.left();
            let stacked = row.subline.is_some() && !row.chips.is_empty();
            let lift = if stacked {
                text * 0.95
            } else if row.subline.is_some() || !row.chips.is_empty() {
                text * 0.5
            } else {
                0.0
            };
            veil.text(
                Pos2::new(left_edge, rect.center().y - lift),
                Align2::LEFT_CENTER,
                truncate_to_width(&veil, &row.label, &font, room),
                font,
                ink,
            );
            if let Some(subline) = row.subline.as_deref() {
                let small = FontId::proportional(text * 0.78);
                let y = if stacked {
                    rect.center().y - text * 0.05
                } else {
                    rect.center().y + text * 0.62
                };
                veil.text(
                    Pos2::new(left_edge, y),
                    Align2::LEFT_CENTER,
                    truncate_to_width(&veil, subline, &small, room),
                    small,
                    ink.gamma_multiply(0.55),
                );
            }
            if !row.chips.is_empty() {
                let font = FontId::proportional(text * 0.8);
                let chips_top = if stacked {
                    rect.center().y + text * 0.55
                } else {
                    rect.center().y + text * 0.14
                };
                let mut chip_row = flow_child(
                    ui,
                    Rect::from_min_max(
                        Pos2::new(left_edge, chips_top),
                        Pos2::new(right_limit, chips_top + text * 1.2),
                    ),
                    egui::Layout::left_to_right(egui::Align::Min),
                    6.0,
                );
                for chip in &row.chips {
                    let label = truncate_to_width(&veil, chip, &font, room.max(40.0));
                    let galley = veil.layout_no_wrap(label.to_string(), font.clone(), ink);
                    let size = galley.size();
                    let width = size.x + 16.0;
                    if width > chip_row.available_width() {
                        break;
                    }
                    let pill = chip_row.allocate_space(Vec2::new(width, text * 1.2)).1;
                    veil.rect_filled(pill, pill.height() * 0.5, ink.gamma_multiply(0.14));
                    veil.galley(
                        Pos2::new(pill.left() + 8.0, pill.center().y - size.y * 0.5),
                        galley,
                        ink.gamma_multiply(0.8),
                    );
                }
            }
            if response.clicked() {
                layout.activated = Some(node);
            }
        }
    }
    if slide < 1.0 && (sheet || drawer) {
        let offset = if drawer {
            Vec2::new((1.0 - slide) * card.width(), 0.0)
        } else {
            Vec2::new(0.0, (1.0 - slide) * card.height())
        };
        chrome.context.graphics_mut(|graphics| {
            let list = graphics.entry(layer);
            let last = list.next_idx();
            list.transform_range(
                first,
                last,
                egui::emath::TSTransform::from_translation(offset),
            );
        });
    }
}

fn flow_child(ui: &mut egui::Ui, rect: Rect, layout: egui::Layout, gap: f32) -> egui::Ui {
    let mut child = ui.new_child(egui::UiBuilder::new().max_rect(rect).layout(layout));
    child.spacing_mut().item_spacing = Vec2::new(gap, 0.0);
    child
}

fn parse_hex(value: &str) -> Option<Color32> {
    let hex = value.strip_prefix('#')?;
    let number = u32::from_str_radix(hex, 16).ok()?;
    Some(Color32::from_rgb(
        (number >> 16) as u8,
        (number >> 8) as u8,
        number as u8,
    ))
}

fn draw_chips(
    chrome: &Chrome,
    ui: &mut egui::Ui,
    layout: &mut HomeLayout,
    panel: &PlayerPanel,
    slide: f32,
) {
    use crate::flex::{leaf, row};
    let tokens = UiMetrics::for_viewport(chrome.viewport);
    let pad = 18.0;
    let width = (chrome.rect.width() - chrome.margin() * 2.0).min(560.0);
    let inner = width - pad * 2.0;
    let chip = Vec2::new(0.0, 36.0);
    let font = FontId::proportional(14.0);
    let chip_nodes = panel
        .rows
        .iter()
        .enumerate()
        .take(PLAYER_PANEL_ROW_LIMIT)
        .filter(|(_, item)| !item.heading)
        .map(|(index, item)| {
            let label = item.label.clone();
            let size = crate::flex::text_size(ui.painter(), &label, &font);
            leaf(Vec2::new(size.x + 32.0, chip.y)).id(index)
        })
        .collect::<Vec<_>>();
    let chips = row(chip_nodes).wrap().gap(8.0).line_gap(8.0);
    let chips_height = chips
        .solve(Rect::from_min_size(Pos2::ZERO, Vec2::new(inner, 400.0)))
        .tagged()
        .iter()
        .map(|(_, rect)| rect.bottom())
        .fold(chip.y, f32::max);
    let title_h = 28.0;
    let value_h = 44.0;
    let height = 12.0 + pad + title_h + value_h + chips_height + pad + chrome.viewport.safe_bottom;
    let card = Rect::from_min_max(
        Pos2::new(
            chrome.rect.center().x - width * 0.5,
            chrome.rect.bottom() - height,
        ),
        Pos2::new(chrome.rect.center().x + width * 0.5, chrome.rect.bottom()),
    );
    let veil = chrome.modal_veil(ui, card, layout);
    veil.rect_filled(chrome.rect, 0.0, Color32::from_black_alpha(110));
    let layer = veil.layer_id();
    let first = chrome
        .context
        .graphics_mut(|graphics| graphics.entry(layer).next_idx());
    let radius = egui::CornerRadius {
        nw: 24,
        ne: 24,
        sw: 0,
        se: 0,
    };
    veil.rect_filled(card, radius, tokens.surface);
    veil.rect_stroke(
        card,
        radius,
        egui::Stroke::new(stroke::hairline(), tokens.border),
        egui::StrokeKind::Inside,
    );
    veil.rect_filled(
        Rect::from_center_size(
            Pos2::new(card.center().x, card.top() + 9.0),
            Vec2::new(36.0, 4.0),
        ),
        2.0,
        crate::stylesheet::color("strong"),
    );
    let left = card.left() + pad;
    let mut top = card.top() + 12.0 + pad;
    veil.text(
        Pos2::new(left, top + title_h * 0.5),
        Align2::LEFT_CENTER,
        &panel.title,
        FontId::proportional(15.0),
        text_primary(),
    );
    top += title_h;
    let current = panel
        .rows
        .iter()
        .find(|item| item.selected)
        .map(|item| item.value.as_deref().unwrap_or(item.label.as_str()))
        .unwrap_or_default();
    veil.text(
        Pos2::new(left, top + value_h * 0.5),
        Align2::LEFT_CENTER,
        current,
        FontId::proportional(32.0),
        text_primary(),
    );
    top += value_h;
    let placed = chips.solve(Rect::from_min_size(
        Pos2::new(left, top),
        Vec2::new(inner, chips_height),
    ));
    for (index, rect) in placed.tagged() {
        let item = &panel.rows[index];
        let node = NODE_PLAYER_PANEL_ROW_BASE + index as u64;
        layout.focusable.push((node, rect));
        let response = ui.interact(
            rect,
            Id::new(("fluxa-player-panel-row", node)),
            Sense::click(),
        );
        let on = chrome.focused == Some(node);
        let (fill, ink_color) = if item.selected || on {
            (text_primary(), Color32::BLACK)
        } else {
            (crate::stylesheet::color("well"), ink(235))
        };
        veil.rect_filled(rect, rect.height() * 0.5, fill);
        veil.text(
            rect.center(),
            Align2::CENTER_CENTER,
            &item.label,
            font.clone(),
            ink_color,
        );
        if response.clicked() {
            layout.activated = Some(node);
        }
    }
    if slide < 1.0 {
        let offset = Vec2::new(0.0, (1.0 - slide) * card.height());
        chrome.context.graphics_mut(|graphics| {
            let list = graphics.entry(layer);
            let last = list.next_idx();
            list.transform_range(
                first,
                last,
                egui::emath::TSTransform::from_translation(offset),
            );
        });
    }
}
