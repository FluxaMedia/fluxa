use super::*;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct StatsSection {
    pub title: String,
    pub rows: Vec<(String, String)>,
}

fn pack(
    sections: &[StatsSection],
    columns: usize,
    line: f32,
    gap: f32,
    limit: f32,
) -> Vec<Vec<usize>> {
    let mut packed: Vec<Vec<usize>> = vec![Vec::new()];
    let mut used = line;
    for (index, section) in sections.iter().enumerate() {
        let height = line * (section.rows.len() + 1) as f32;
        let empty = packed.last().is_some_and(Vec::is_empty);
        let spacing = if empty { 0.0 } else { gap };
        if !empty && used + spacing + height > limit && packed.len() < columns {
            packed.push(vec![index]);
            used = height;
        } else {
            packed.last_mut().unwrap().push(index);
            used += spacing + height;
        }
    }
    packed
}

fn column_height(
    sections: &[StatsSection],
    column: &[usize],
    first: bool,
    line: f32,
    gap: f32,
) -> f32 {
    let header = if first { line } else { 0.0 };
    column
        .iter()
        .enumerate()
        .fold(header, |height, (position, index)| {
            let spacing = if position > 0 { gap } else { 0.0 };
            height + spacing + line * (sections[*index].rows.len() + 1) as f32
        })
}

pub(super) fn draw_stats(chrome: &Chrome) {
    let sections = &chrome.player.stats;
    if sections.is_empty() {
        return;
    }
    let metrics = UiMetrics::for_viewport(chrome.viewport);
    let tokens = UiMetrics::for_viewport(chrome.viewport);
    let language = &chrome.player.language;
    let line = metrics.text.meta * 1.55;
    let gap = metrics.text.meta * 0.6;
    let top = if chrome.player.controls_visible {
        72.0
    } else {
        chrome.toast_top() + 48.0
    };
    let available_width = chrome.rect.width() - chrome.margin() * 2.0;
    let available_height =
        (chrome.rect.bottom() - chrome.margin() - top - Space::S14.px(&metrics) * 2.0)
            .max(line * 4.0);
    let column_width = tokens
        .player_panel_narrow
        .of(chrome.rect.width())
        .min(available_width);
    let spacing = Space::S14.px(&metrics);
    let max_columns =
        (((available_width + spacing) / (column_width + spacing)).floor() as usize).max(1);
    let mut columns = pack(sections, 1, line, gap, available_height);
    for count in 2..=max_columns {
        let tallest = columns
            .iter()
            .enumerate()
            .map(|(at, column)| column_height(sections, column, at == 0, line, gap))
            .fold(0.0, f32::max);
        if tallest <= available_height {
            break;
        }
        columns = pack(sections, count, line, gap, available_height);
    }
    let tallest = columns
        .iter()
        .enumerate()
        .map(|(at, column)| column_height(sections, column, at == 0, line, gap))
        .fold(0.0, f32::max);
    let width = column_width * columns.len() as f32
        + spacing * columns.len().saturating_sub(1) as f32
        + Space::S14.px(&metrics) * 2.0;
    let panel = Rect::from_min_size(
        Pos2::new(chrome.rect.right() - chrome.margin() - width, top),
        Vec2::new(width, tallest + Space::S14.px(&metrics) * 2.0),
    );
    chrome
        .painter
        .rect_filled(panel, tokens.card_radius, shade(Shade::Strong));
    chrome.painter.rect_stroke(
        panel,
        tokens.card_radius,
        egui::Stroke::new(stroke::hairline(), tokens.border),
        egui::StrokeKind::Inside,
    );
    let inner_top = panel.top() + Space::S14.px(&metrics);
    for (at, column) in columns.iter().enumerate() {
        let left = panel.left() + Space::S14.px(&metrics) + (column_width + spacing) * at as f32;
        let inner =
            Rect::from_min_size(Pos2::new(left, inner_top), Vec2::new(column_width, tallest));
        let mut y = inner.top();
        if at == 0 {
            chrome.text(
                Pos2::new(inner.left(), y),
                Align2::LEFT_TOP,
                &localized("player.playback_summary", language),
                metrics.text.meta * 1.15,
                inner.width(),
                255,
            );
            y += line;
        }
        for (position, index) in column.iter().enumerate() {
            if position > 0 {
                y += gap;
            }
            draw_section(chrome, &metrics, inner, &mut y, line, &sections[*index]);
        }
    }
}

fn draw_section(
    chrome: &Chrome,
    metrics: &UiMetrics,
    inner: Rect,
    y: &mut f32,
    line: f32,
    section: &StatsSection,
) {
    let language = &chrome.player.language;
    chrome.text(
        Pos2::new(inner.left(), *y),
        Align2::LEFT_TOP,
        &localized(&section.title, language),
        metrics.text.meta,
        inner.width(),
        255,
    );
    *y += line;
    for (label, value) in &section.rows {
        let label = localized(label, language);
        let label_width = chrome
            .painter
            .layout_no_wrap(
                label.clone(),
                FontId::proportional(metrics.text.meta),
                crate::stylesheet::color("text"),
            )
            .size()
            .x
            .min(inner.width() * 0.45);
        chrome.text(
            Pos2::new(inner.left(), *y),
            Align2::LEFT_TOP,
            &label,
            metrics.text.meta,
            label_width,
            150,
        );
        let max = inner.width() - label_width - metrics.text.meta;
        match value.split_once(" → ") {
            Some((from, to)) => {
                let arrow = metrics.text.meta;
                let gap = metrics.text.meta * 0.4;
                let font = FontId::proportional(metrics.text.meta);
                let to_width = chrome
                    .painter
                    .layout_no_wrap(
                        to.to_owned(),
                        font.clone(),
                        crate::stylesheet::color("text"),
                    )
                    .size()
                    .x
                    .min(max * 0.5);
                chrome.text(
                    Pos2::new(inner.right(), *y),
                    Align2::RIGHT_TOP,
                    to,
                    metrics.text.meta,
                    to_width,
                    230,
                );
                let arrow_right = inner.right() - to_width - gap;
                if let Some(icon) = chrome.icons.get("ArrowRight") {
                    chrome.painter.image(
                        icon,
                        Rect::from_min_size(
                            Pos2::new(arrow_right - arrow, *y + (line - arrow) * 0.5 - 1.0),
                            Vec2::splat(arrow),
                        ),
                        Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                        crate::stylesheet::color("text"),
                    );
                }
                chrome.text(
                    Pos2::new(arrow_right - arrow - gap, *y),
                    Align2::RIGHT_TOP,
                    from,
                    metrics.text.meta,
                    max - to_width - arrow - gap * 2.0,
                    230,
                );
            }
            None => chrome.text(
                Pos2::new(inner.right(), *y),
                Align2::RIGHT_TOP,
                value,
                metrics.text.meta,
                max,
                230,
            ),
        }
        *y += line;
    }
}
