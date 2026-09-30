use super::*;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct PanelRow {
    pub label: String,
    pub value: Option<String>,
    pub primary: bool,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct PlayerPanel {
    pub title: String,
    pub message: Option<String>,
    pub rows: Vec<PanelRow>,
}

pub(super) fn draw_panel(
    chrome: &Chrome,
    ui: &mut egui::Ui,
    layout: &mut HomeLayout,
    panel: &PlayerPanel,
) {
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
