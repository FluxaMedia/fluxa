use super::*;

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
    if panel.chips && chrome.viewport.form_factor == UiFormFactor::Mobile {
        draw_chips(chrome, ui, layout, panel, slide);
        return;
    }
    if panel.list {
        draw_list(chrome, ui, layout, panel, assets, slide);
        return;
    }
    let tokens = UiMetrics::for_viewport(chrome.viewport);
    let player_tokens = super::tokens::PlayerTokens::for_form_factor(chrome.viewport.form_factor);
    let (pill_height, text, pad) = (player_tokens.pill, player_tokens.text, player_tokens.pad);
    let base = player_tokens.menu_width;
    let width = base.min(chrome.rect.width() - chrome.margin() * 2.0);
    let rows = panel.rows.len().min(PLAYER_PANEL_ROW_LIMIT);
    let message_height = if panel.message.is_some() {
        text * 1.8
    } else {
        0.0
    };
    let gap = 8.0;
    let height = pad * 2.0 + text * 2.2 + message_height + (rows + 1) as f32 * (pill_height + gap);
    let card = Rect::from_center_size(
        chrome.rect.center(),
        Vec2::new(width, height.min(chrome.rect.height() - 24.0)),
    );
    let veil = chrome.context.layer_painter(egui::LayerId::new(
        egui::Order::Foreground,
        Id::new("fluxa-player-panel-veil"),
    ));
    veil.rect_filled(chrome.rect, 0.0, shade(Shade::Veil));
    veil.rect_filled(card, tokens.card_radius, tokens.surface);
    veil.rect_stroke(
        card,
        tokens.card_radius,
        egui::Stroke::new(stroke::hairline(), tokens.border),
        egui::StrokeKind::Inside,
    );
    let inner = card.shrink(pad);
    let mut y = inner.top();
    veil.text(
        Pos2::new(inner.left(), y),
        Align2::LEFT_TOP,
        &panel.title,
        FontId::proportional(text * 1.15),
        text_primary(),
    );
    y += text * 2.2;
    if let Some(message) = panel.message.as_deref() {
        veil.text(
            Pos2::new(inner.left(), y),
            Align2::LEFT_TOP,
            truncate_to_width(&veil, message, &FontId::proportional(text), inner.width()),
            FontId::proportional(text),
            ink(tone::subtle()),
        );
        y += message_height;
    }
    for (index, row) in panel.rows.iter().take(rows).enumerate() {
        let label = match row.value.as_deref() {
            Some(value) => format!("{}  {}", row.label, value),
            None => row.label.clone(),
        };
        let rect = Rect::from_min_size(
            Pos2::new(inner.left(), y),
            Vec2::new(inner.width(), pill_height),
        );
        overlay::pill(
            ui,
            layout,
            rect,
            &label,
            text,
            row.primary,
            NODE_PLAYER_PANEL_ROW_BASE + index as u64,
        );
        y += pill_height + gap;
    }
    let close = Rect::from_min_size(
        Pos2::new(inner.left(), y),
        Vec2::new(inner.width(), pill_height),
    );
    overlay::pill(
        ui,
        layout,
        close,
        &localized("common.close", &chrome.player.language),
        text,
        false,
        NODE_PLAYER_PANEL_CLOSE,
    );
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
    let player_tokens = super::tokens::PlayerTokens::for_form_factor(form_factor);
    let (mut row_h, text, pad, mut base) = (
        player_tokens.list_row,
        player_tokens.list_text,
        player_tokens.list_pad,
        player_tokens.list_width,
    );
    let rich = panel
        .rows
        .iter()
        .any(|row| row.thumbnail.is_some() || row.detail.is_some());
    if rich {
        (row_h, base) = (player_tokens.rich_row, player_tokens.rich_width);
    }
    let mut columns: Vec<(Option<&str>, Vec<(usize, &PanelRow)>)> = Vec::new();
    for (index, row) in panel.rows.iter().enumerate().take(PLAYER_PANEL_ROW_LIMIT) {
        if row.heading {
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
    let title_h = if panel.title.is_empty() { 0.0 } else { row_h };
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
    let veil = chrome.context.layer_painter(egui::LayerId::new(
        egui::Order::Foreground,
        Id::new("fluxa-player-panel-veil"),
    ));
    let outside = ui.interact(
        chrome.rect,
        Id::new("fluxa-player-panel-outside"),
        Sense::click(),
    );
    if outside.clicked()
        && !outside
            .interact_pointer_pos()
            .is_some_and(|pos| card.contains(pos))
    {
        layout.activated = Some(NODE_PLAYER_PANEL_CLOSE);
    }
    if let Some(look) = panel.look.as_ref() {
        let bottom = chrome.rect.bottom() - chrome.viewport.safe_bottom - 24.0;
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
            ink(tone::strong()),
        );
    }
    let mut top = card.top() + pad + handle_h;
    if title_h > 0.0 {
        veil.text(
            Pos2::new(card.left() + pad + 8.0, top + title_h * 0.5),
            Align2::LEFT_CENTER,
            &panel.title,
            FontId::proportional(text * 1.1),
            text_primary(),
        );
        top += title_h;
    }
    let check = chrome.icons.get("Check");
    for (column, (heading, rows)) in columns.iter().enumerate() {
        let left = card.left() + pad + column as f32 * (col_w + gap);
        if let Some(heading) = heading {
            let label = if sheet {
                heading.to_uppercase()
            } else {
                heading.to_string()
            };
            veil.text(
                Pos2::new(left + 8.0, top + head_h * 0.5),
                Align2::LEFT_CENTER,
                label,
                FontId::proportional(text * if sheet { 0.78 } else { 0.85 }),
                ink(tone::muted()),
            );
        }
        let rows_top = top + head_h;
        if sheet && column > 0 {
            let x = left - gap * 0.5;
            veil.line_segment(
                [
                    Pos2::new(x, top + 4.0),
                    Pos2::new(x, rows_top + visible as f32 * row_h),
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
        if let Some(position) = focused {
            if position < start {
                start = position;
            } else if position >= start + visible {
                start = position + 1 - visible;
            }
        }
        let column_rect = Rect::from_min_size(
            Pos2::new(left, rows_top),
            Vec2::new(col_w, visible as f32 * row_h),
        );
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
        for (position, (index, row)) in rows.iter().enumerate() {
            let node = NODE_PLAYER_PANEL_ROW_BASE + *index as u64;
            let rect = Rect::from_min_size(
                Pos2::new(left, rows_top + (position as f32 - start as f32) * row_h),
                Vec2::new(col_w, row_h),
            );
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
                        ink(tone::muted()),
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
                ink(tone::soft())
            } else if response.hovered() {
                ink(tone::faint())
            } else {
                Color32::TRANSPARENT
            };
            veil.rect_filled(rect, row_h * 0.25, fill);
            let ink = if on {
                Color32::BLACK
            } else {
                ink(if row.selected { 255 } else { 215 })
            };
            let mut right = rect.right() - 12.0;
            if rich {
                let art = Rect::from_min_size(
                    rect.min + Vec2::new(8.0, 8.0),
                    Vec2::new((row_h - 16.0) * 16.0 / 9.0, row_h - 16.0),
                );
                veil.rect_filled(art, radius::control(), crate::ink(tone::faint()));
                if let Some(time) = row.frame {
                    let frame = chrome
                        .player
                        .filmstrip
                        .iter()
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
                let left = art.right() + 12.0;
                let width = (rect.right() - left - 12.0).max(0.0);
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
                let mut x = rect.right() - 8.0 - side;
                for offset in (1..=count).rev() {
                    let swatch = &panel.rows[*index + offset];
                    let step_node = NODE_PLAYER_PANEL_ROW_BASE + (*index + offset) as u64;
                    let hit = Rect::from_min_size(
                        Pos2::new(x, rect.center().y - side * 0.5),
                        Vec2::splat(side),
                    );
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
                    x -= side + gap;
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
                let mut x = rect.right() - 8.0 - total;
                if x < rect.left() + 8.0 {
                    x = rect.left() + 8.0;
                }
                for (slot, width) in widths.iter().enumerate() {
                    let choice = &panel.rows[*index + 1 + slot];
                    let step_node = NODE_PLAYER_PANEL_ROW_BASE + (*index + 1 + slot) as u64;
                    let hit = Rect::from_min_size(
                        Pos2::new(x, rect.center().y - 15.0),
                        Vec2::new(*width, 30.0),
                    );
                    let response = ui.interact(
                        hit,
                        Id::new(("fluxa-player-panel-step", step_node)),
                        Sense::click(),
                    );
                    layout.focusable.push((step_node, hit));
                    let (fill, color) = if choice.selected {
                        (text_primary(), Color32::BLACK)
                    } else {
                        (crate::ink(tone::faint()), ink)
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
                    x += width + gap;
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
                let plus = Rect::from_center_size(
                    Pos2::new(rect.right() - 8.0 - button * 0.5, rect.center().y),
                    Vec2::splat(button),
                );
                let minus = plus.translate(Vec2::new(-(button + 62.0), 0.0));
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
                    Pos2::new((minus.right() + plus.left()) * 0.5, rect.center().y),
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
            let mut left_edge = rect.left() + 8.0;
            if let Some(badge) = row.badge.as_deref() {
                let font = FontId::proportional(text * 0.62);
                let pill = Rect::from_min_size(
                    Pos2::new(left_edge, rect.center().y - 8.0),
                    Vec2::new(24.0, 16.0),
                );
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
                left_edge += 34.0;
            }
            if let Some(texture) = row.icon.as_deref().and_then(|name| chrome.icons.get(name)) {
                let side = text * 1.4;
                veil.image(
                    texture,
                    Rect::from_center_size(
                        Pos2::new(left_edge + side * 0.5, rect.center().y),
                        Vec2::splat(side),
                    ),
                    full_uv(),
                    ink,
                );
                left_edge += side + 10.0;
            }
            if row.chevron
                && let Some(texture) = chrome.icons.get("ArrowRight")
            {
                let side = text * 1.1;
                veil.image(
                    texture,
                    Rect::from_center_size(
                        Pos2::new(right - side * 0.5, rect.center().y),
                        Vec2::splat(side),
                    ),
                    full_uv(),
                    ink.gamma_multiply(0.6),
                );
                right -= side + 8.0;
            }
            if let Some(on_state) = row.switch {
                let track = Rect::from_center_size(
                    Pos2::new(right - 20.0, rect.center().y),
                    Vec2::new(40.0, 22.0),
                );
                components::toggle(
                    chrome.context,
                    &veil,
                    Id::new(("fluxa-player-panel-switch", node)),
                    track,
                    on_state,
                    tokens,
                );
                right -= 52.0;
            }
            if row.selected
                && let Some(check) = check
            {
                let side = text * 1.2;
                veil.image(
                    check,
                    Rect::from_center_size(
                        Pos2::new(right - side * 0.5, rect.center().y),
                        Vec2::splat(side),
                    ),
                    full_uv(),
                    ink,
                );
                right -= side + 8.0;
            }
            if let Some(value) = row.value.as_deref() {
                let font = FontId::proportional(text * 0.9);
                let value = truncate_to_width(&veil, value, &font, col_w * 0.4);
                let galley = veil.layout_no_wrap(value.to_string(), font, ink.gamma_multiply(0.6));
                let size = galley.size();
                veil.galley(
                    Pos2::new(right - size.x, rect.center().y - size.y * 0.5),
                    galley,
                    ink,
                );
                right -= size.x + 10.0;
            }
            let font = FontId::proportional(text);
            veil.text(
                Pos2::new(left_edge, rect.center().y),
                Align2::LEFT_CENTER,
                truncate_to_width(&veil, &row.label, &font, (right - left_edge - 8.0).max(0.0)),
                font,
                ink,
            );
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
    let veil = chrome.context.layer_painter(egui::LayerId::new(
        egui::Order::Foreground,
        Id::new("fluxa-player-panel-veil"),
    ));
    let outside = ui.interact(
        chrome.rect,
        Id::new("fluxa-player-panel-outside"),
        Sense::click(),
    );
    if outside.clicked()
        && !outside
            .interact_pointer_pos()
            .is_some_and(|pos| card.contains(pos))
    {
        layout.activated = Some(NODE_PLAYER_PANEL_CLOSE);
    }
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
        ink(tone::strong()),
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
            (ink(tone::faint()), ink(235))
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
