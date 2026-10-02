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
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct PlayerPanel {
    pub title: String,
    pub message: Option<String>,
    pub rows: Vec<PanelRow>,
    pub list: bool,
}

pub(super) fn draw_panel(
    chrome: &Chrome,
    ui: &mut egui::Ui,
    layout: &mut HomeLayout,
    panel: &PlayerPanel,
    assets: &mut impl HomeAssets,
) {
    if panel.list {
        draw_list(chrome, ui, layout, panel, assets);
        return;
    }
    let tokens = UiMetrics::for_viewport(chrome.viewport);
    let (pill_height, text, pad) = match chrome.viewport.form_factor {
        UiFormFactor::Tv => (60.0, 22.0, 24.0),
        UiFormFactor::Mobile => (42.0, 14.0, 14.0),
        UiFormFactor::Desktop => (44.0, 15.0, 18.0),
    };
    let base: f32 = match chrome.viewport.form_factor {
        UiFormFactor::Tv => 560.0,
        UiFormFactor::Mobile => 320.0,
        UiFormFactor::Desktop => 400.0,
    };
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
    veil.rect_filled(chrome.rect, 0.0, Color32::from_black_alpha(150));
    veil.rect_filled(card, tokens.card_radius, tokens.surface);
    veil.rect_stroke(
        card,
        tokens.card_radius,
        egui::Stroke::new(1.0, tokens.border),
        egui::StrokeKind::Inside,
    );
    let inner = card.shrink(pad);
    let mut y = inner.top();
    veil.text(
        Pos2::new(inner.left(), y),
        Align2::LEFT_TOP,
        &panel.title,
        FontId::proportional(text * 1.15),
        Color32::WHITE,
    );
    y += text * 2.2;
    if let Some(message) = panel.message.as_deref() {
        veil.text(
            Pos2::new(inner.left(), y),
            Align2::LEFT_TOP,
            truncate_to_width(&veil, message, &FontId::proportional(text), inner.width()),
            FontId::proportional(text),
            Color32::from_white_alpha(170),
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
) {
    let tokens = UiMetrics::for_viewport(chrome.viewport);
    let form_factor = chrome.viewport.form_factor;
    let (mut row_h, text, pad, mut base) = match form_factor {
        UiFormFactor::Tv => (58.0, 21.0, 26.0, 400.0),
        UiFormFactor::Mobile => (44.0, 15.0, 14.0, 250.0),
        UiFormFactor::Desktop => (38.0, 14.0, 12.0, 270.0),
    };
    let rich = panel
        .rows
        .iter()
        .any(|row| row.thumbnail.is_some() || row.detail.is_some());
    if rich {
        (row_h, base) = match form_factor {
            UiFormFactor::Tv => (128.0, 760.0),
            UiFormFactor::Mobile => (88.0, 420.0),
            UiFormFactor::Desktop => (92.0, 460.0),
        };
    }
    let mut columns: Vec<(Option<&str>, Vec<(usize, &PanelRow)>)> = Vec::new();
    for (index, row) in panel.rows.iter().enumerate().take(PLAYER_PANEL_ROW_LIMIT) {
        if row.heading {
            columns.push((Some(row.label.as_str()), Vec::new()));
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
    let col_w = ((available - pad * 2.0 - gap * (count - 1.0)) / count).min(base);
    let width = col_w * count + gap * (count - 1.0) + pad * 2.0;
    let title_h = if panel.title.is_empty() { 0.0 } else { row_h };
    let head_h = if columns.iter().any(|(heading, _)| heading.is_some()) {
        row_h * 0.8
    } else {
        0.0
    };
    let reserved = match form_factor {
        UiFormFactor::Desktop => 120.0 + chrome.viewport.safe_bottom,
        _ => 32.0,
    };
    let max_body = chrome.rect.height() - reserved - chrome.margin() - pad * 2.0 - title_h - head_h;
    let longest = columns
        .iter()
        .map(|(_, rows)| rows.len())
        .max()
        .unwrap_or(0);
    let visible = (((max_body / row_h).floor() as usize).max(3)).min(longest.max(1));
    let height = pad * 2.0 + title_h + head_h + visible as f32 * row_h;
    let card = match form_factor {
        UiFormFactor::Desktop => Rect::from_min_size(
            Pos2::new(
                chrome.rect.right() - chrome.margin() - width,
                chrome.rect.bottom() - reserved - height,
            ),
            Vec2::new(width, height),
        ),
        _ => Rect::from_center_size(chrome.rect.center(), Vec2::new(width, height)),
    };
    let veil = chrome.context.layer_painter(egui::LayerId::new(
        egui::Order::Foreground,
        Id::new("fluxa-player-panel-veil"),
    ));
    if form_factor == UiFormFactor::Desktop {
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
    } else {
        veil.rect_filled(chrome.rect, 0.0, Color32::from_black_alpha(150));
    }
    veil.rect_filled(card, tokens.card_radius, tokens.surface);
    veil.rect_stroke(
        card,
        tokens.card_radius,
        egui::Stroke::new(1.0, tokens.border),
        egui::StrokeKind::Inside,
    );
    let mut top = card.top() + pad;
    if title_h > 0.0 {
        veil.text(
            Pos2::new(card.left() + pad + 8.0, top + title_h * 0.5),
            Align2::LEFT_CENTER,
            &panel.title,
            FontId::proportional(text * 1.1),
            Color32::WHITE,
        );
        top += title_h;
    }
    let check = chrome.icons.get("Check");
    for (column, (heading, rows)) in columns.iter().enumerate() {
        let left = card.left() + pad + column as f32 * (col_w + gap);
        if let Some(heading) = heading {
            veil.text(
                Pos2::new(left + 8.0, top + head_h * 0.5),
                Align2::LEFT_CENTER,
                *heading,
                FontId::proportional(text * 0.85),
                Color32::from_white_alpha(140),
            );
        }
        let rows_top = top + head_h;
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
                        &row.label,
                        FontId::proportional(text * 0.8),
                        Color32::from_white_alpha(120),
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
                Color32::WHITE
            } else if row.selected {
                Color32::from_white_alpha(22)
            } else if response.hovered() {
                Color32::from_white_alpha(12)
            } else {
                Color32::TRANSPARENT
            };
            veil.rect_filled(rect, row_h * 0.25, fill);
            let ink = if on {
                Color32::BLACK
            } else {
                Color32::from_white_alpha(if row.selected { 255 } else { 215 })
            };
            let mut right = rect.right() - 12.0;
            if rich {
                let art = Rect::from_min_size(
                    rect.min + Vec2::new(8.0, 8.0),
                    Vec2::new((row_h - 16.0) * 16.0 / 9.0, row_h - 16.0),
                );
                veil.rect_filled(art, 6.0, Color32::from_white_alpha(14));
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
                Pos2::new(rect.left() + 8.0, rect.center().y),
                Align2::LEFT_CENTER,
                truncate_to_width(
                    &veil,
                    &row.label,
                    &font,
                    (right - rect.left() - 16.0).max(0.0),
                ),
                font,
                ink,
            );
            if response.clicked() {
                layout.activated = Some(node);
            }
        }
    }
}
