use super::*;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct StatsSection {
    pub title: String,
    pub rows: Vec<(String, String)>,
}

pub(super) fn draw_stats(chrome: &Chrome) {
    let sections = &chrome.player.stats;
    if sections.is_empty() {
        return;
    }
    let metrics = super::tokens::PlayerTokens::for_form_factor(chrome.viewport.form_factor);
    let tokens = UiMetrics::for_viewport(chrome.viewport);
    let language = &chrome.player.language;
    let line = metrics.stats_text * 1.55;
    let rows: usize = sections.iter().map(|section| section.rows.len() + 1).sum();
    let gaps = sections.len().saturating_sub(1) as f32 * metrics.stats_text * 0.6;
    let height = metrics.stats_pad * 2.0 + line * (rows as f32 + 1.0) + gaps;
    let width = metrics
        .stats_width
        .min(chrome.rect.width() - chrome.margin() * 2.0);
    let top = if chrome.player.controls_visible {
        72.0
    } else {
        chrome.toast_top() + 48.0
    };
    let panel = Rect::from_min_size(
        Pos2::new(chrome.rect.right() - chrome.margin() - width, top),
        Vec2::new(width, height),
    );
    chrome
        .painter
        .rect_filled(panel, tokens.card_radius, shade(Shade::Strong));
    chrome.painter.rect_stroke(
        panel,
        tokens.card_radius,
        egui::Stroke::new(stroke::HAIRLINE, tokens.border),
        egui::StrokeKind::Inside,
    );
    let inner = panel.shrink(metrics.stats_pad);
    let mut y = inner.top();
    chrome.text(
        Pos2::new(inner.left(), y),
        Align2::LEFT_TOP,
        &localized("player.playback_summary", language),
        metrics.stats_text * 1.15,
        inner.width(),
        255,
    );
    y += line;
    for (index, section) in sections.iter().enumerate() {
        if index > 0 {
            y += metrics.stats_text * 0.6;
        }
        chrome.text(
            Pos2::new(inner.left(), y),
            Align2::LEFT_TOP,
            &localized(&section.title, language),
            metrics.stats_text,
            inner.width(),
            255,
        );
        y += line;
        for (label, value) in &section.rows {
            chrome.text(
                Pos2::new(inner.left(), y),
                Align2::LEFT_TOP,
                &localized(label, language),
                metrics.stats_text,
                inner.width() * 0.4,
                150,
            );
            let max = inner.width() * 0.58;
            match value.split_once(" → ") {
                Some((from, to)) => {
                    let arrow = metrics.stats_text;
                    let gap = metrics.stats_text * 0.4;
                    let font = FontId::proportional(metrics.stats_text);
                    let to_width = chrome
                        .painter
                        .layout_no_wrap(to.to_owned(), font.clone(), ink(tone::TEXT))
                        .size()
                        .x
                        .min(max * 0.5);
                    chrome.text(
                        Pos2::new(inner.right(), y),
                        Align2::RIGHT_TOP,
                        to,
                        metrics.stats_text,
                        to_width,
                        230,
                    );
                    let arrow_right = inner.right() - to_width - gap;
                    if let Some(icon) = chrome.icons.get("ArrowRight") {
                        chrome.painter.image(
                            icon,
                            Rect::from_min_size(
                                Pos2::new(arrow_right - arrow, y + (line - arrow) * 0.5 - 1.0),
                                Vec2::splat(arrow),
                            ),
                            Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                            ink(tone::TEXT),
                        );
                    }
                    chrome.text(
                        Pos2::new(arrow_right - arrow - gap, y),
                        Align2::RIGHT_TOP,
                        from,
                        metrics.stats_text,
                        max - to_width - arrow - gap * 2.0,
                        230,
                    );
                }
                None => chrome.text(
                    Pos2::new(inner.right(), y),
                    Align2::RIGHT_TOP,
                    value,
                    metrics.stats_text,
                    max,
                    230,
                ),
            }
            y += line;
        }
    }
}
