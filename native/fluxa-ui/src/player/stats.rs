use super::*;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct StatsSection {
    pub title: String,
    pub rows: Vec<(String, String)>,
}

struct StatsMetrics {
    width: f32,
    text: f32,
    pad: f32,
}

impl StatsMetrics {
    fn for_form_factor(form_factor: UiFormFactor) -> Self {
        match form_factor {
            UiFormFactor::Tv => Self {
                width: 520.0,
                text: 20.0,
                pad: 22.0,
            },
            UiFormFactor::Mobile => Self {
                width: 250.0,
                text: 11.5,
                pad: 10.0,
            },
            UiFormFactor::Desktop => Self {
                width: 310.0,
                text: 12.5,
                pad: 14.0,
            },
        }
    }
}

pub(super) fn draw_stats(chrome: &Chrome) {
    let sections = &chrome.player.stats;
    if sections.is_empty() {
        return;
    }
    let metrics = StatsMetrics::for_form_factor(chrome.viewport.form_factor);
    let tokens = UiMetrics::for_viewport(chrome.viewport);
    let language = &chrome.player.language;
    let line = metrics.text * 1.55;
    let rows: usize = sections.iter().map(|section| section.rows.len() + 1).sum();
    let gaps = sections.len().saturating_sub(1) as f32 * metrics.text * 0.6;
    let height = metrics.pad * 2.0 + line * (rows as f32 + 1.0) + gaps;
    let width = metrics
        .width
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
    let inner = panel.shrink(metrics.pad);
    let mut y = inner.top();
    chrome.text(
        Pos2::new(inner.left(), y),
        Align2::LEFT_TOP,
        &localized("player.playback_summary", language),
        metrics.text * 1.15,
        inner.width(),
        255,
    );
    y += line;
    for (index, section) in sections.iter().enumerate() {
        if index > 0 {
            y += metrics.text * 0.6;
        }
        chrome.text(
            Pos2::new(inner.left(), y),
            Align2::LEFT_TOP,
            &localized(&section.title, language),
            metrics.text,
            inner.width(),
            255,
        );
        y += line;
        for (label, value) in &section.rows {
            chrome.text(
                Pos2::new(inner.left(), y),
                Align2::LEFT_TOP,
                &localized(label, language),
                metrics.text,
                inner.width() * 0.4,
                150,
            );
            let max = inner.width() * 0.58;
            match value.split_once(" → ") {
                Some((from, to)) => {
                    let arrow = metrics.text;
                    let gap = metrics.text * 0.4;
                    let font = FontId::proportional(metrics.text);
                    let to_width = chrome
                        .painter
                        .layout_no_wrap(to.to_owned(), font.clone(), ink(230))
                        .size()
                        .x
                        .min(max * 0.5);
                    chrome.text(
                        Pos2::new(inner.right(), y),
                        Align2::RIGHT_TOP,
                        to,
                        metrics.text,
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
                            ink(230),
                        );
                    }
                    chrome.text(
                        Pos2::new(arrow_right - arrow - gap, y),
                        Align2::RIGHT_TOP,
                        from,
                        metrics.text,
                        max - to_width - arrow - gap * 2.0,
                        230,
                    );
                }
                None => chrome.text(
                    Pos2::new(inner.right(), y),
                    Align2::RIGHT_TOP,
                    value,
                    metrics.text,
                    max,
                    230,
                ),
            }
            y += line;
        }
    }
}
