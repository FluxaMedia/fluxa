use super::*;
use crate::{
    DownloadGroup, DownloadRow, DownloadStatus, DownloadsSummary, NODE_DOWNLOAD_BASE,
    NODE_DOWNLOAD_GROUP_BASE, NODE_DOWNLOAD_LOCATION, format_bytes, format_duration, localized,
};

const STORAGE_BLOCK: f32 = 96.0;
const SPEED_BLOCK: f32 = 96.0;

pub fn download_summary_height(_width: f32, metrics: UiMetrics) -> f32 {
    padding(metrics) * 3.0 + STORAGE_BLOCK + SPEED_BLOCK
}

fn speed_block(
    painter: &Painter,
    rect: Rect,
    summary: &DownloadsSummary,
    language: &str,
    metrics: UiMetrics,
) {
    draw_text(
        painter,
        rect.left_top(),
        Align2::LEFT_TOP,
        localized("downloads.speed_title", language),
        crate::fonts::regular(metrics.screen_card_subtitle_size),
        metrics.text_muted,
        Some(rect.width()),
    );
    draw_text(
        painter,
        rect.right_top(),
        Align2::RIGHT_TOP,
        format!("{}/s", format_bytes(summary.speed_bps as u64)),
        crate::fonts::medium(metrics.screen_card_title_size),
        metrics.text_primary,
        Some(rect.width()),
    );
    let graph = Rect::from_min_max(
        Pos2::new(rect.left(), rect.top() + 30.0),
        rect.right_bottom(),
    );
    painter.line_segment(
        [graph.left_bottom(), graph.right_bottom()],
        egui::Stroke::new(1.0, metrics.border),
    );
    let peak = summary.history.iter().copied().fold(1_048_576.0_f32, f32::max);
    let steps = summary.history.len().saturating_sub(1).max(1) as f32;
    let points: Vec<Pos2> = summary
        .history
        .iter()
        .enumerate()
        .map(|(index, speed)| {
            Pos2::new(
                graph.left() + graph.width() * index as f32 / steps,
                graph.bottom() - graph.height() * (speed / peak).clamp(0.0, 1.0),
            )
        })
        .collect();
    if points.len() >= 2 {
        painter.add(egui::Shape::line(
            points,
            egui::Stroke::new(2.0, metrics.accent),
        ));
    }
}

#[allow(clippy::too_many_arguments)]
fn storage_block(
    ui: &mut Ui,
    rect: Rect,
    summary: &DownloadsSummary,
    language: &str,
    metrics: UiMetrics,
    layout: &mut HomeLayout,
) {
    let painter = ui.painter().clone();
    let change = localized("downloads.change", language);
    let button_width = 112.0;
    let button = Rect::from_min_size(
        Pos2::new(rect.right() - button_width, rect.top() - 4.0),
        Vec2::new(button_width, 36.0),
    );
    let response = ui
        .scope_builder(egui::UiBuilder::new().max_rect(button), |ui| {
            pill_button(ui, None, &change, Some(button_width), 36.0, 14.0, false, None)
        })
        .inner;
    focus_action(layout, NODE_DOWNLOAD_LOCATION, &response);

    draw_text(
        &painter,
        rect.left_top(),
        Align2::LEFT_TOP,
        localized("downloads.storage_title", language),
        crate::fonts::regular(metrics.screen_card_subtitle_size),
        metrics.text_muted,
        Some(rect.width() - button_width - metrics.control_gap),
    );
    let Some(location) = summary.location.as_ref() else {
        return;
    };
    draw_text(
        &painter,
        Pos2::new(rect.left(), rect.top() + 26.0),
        Align2::LEFT_TOP,
        &location.path,
        crate::fonts::medium(metrics.screen_card_subtitle_size),
        metrics.text_primary,
        Some(rect.width() - button_width - metrics.control_gap),
    );
    let bar = Rect::from_min_size(
        Pos2::new(rect.left(), rect.top() + 62.0),
        Vec2::new(rect.width(), 6.0),
    );
    painter.rect_filled(bar, 3.0, metrics.border);
    if location.total_bytes > 0 {
        let used = 1.0 - location.free_bytes as f32 / location.total_bytes as f32;
        painter.rect_filled(
            Rect::from_min_size(
                bar.min,
                Vec2::new((bar.width() * used.clamp(0.0, 1.0)).max(6.0), 6.0),
            ),
            3.0,
            metrics.accent,
        );
    }
    draw_text(
        &painter,
        Pos2::new(rect.left(), bar.bottom() + 8.0),
        Align2::LEFT_TOP,
        localized("downloads.free_of", language)
            .replacen("%s", &format_bytes(location.free_bytes), 1)
            .replacen("%s", &format_bytes(location.total_bytes), 1),
        crate::fonts::regular(metrics.screen_card_subtitle_size),
        metrics.text_secondary,
        Some(rect.width()),
    );
}

pub fn download_summary(
    ui: &mut Ui,
    summary: &DownloadsSummary,
    origin: Pos2,
    width: f32,
    language: &str,
    metrics: UiMetrics,
    layout: &mut HomeLayout,
) {
    let height = download_summary_height(width, metrics);
    let card = Rect::from_min_size(origin, Vec2::new(width, height));
    let painter = ui.painter().clone();
    painter.rect_filled(card, metrics.card_radius, metrics.surface_raised);
    let pad = padding(metrics);
    let inner = card.shrink(pad);
    let storage = Rect::from_min_size(inner.min, Vec2::new(inner.width(), STORAGE_BLOCK));
    let speed = Rect::from_min_size(
        Pos2::new(inner.left(), storage.bottom() + pad),
        Vec2::new(inner.width(), SPEED_BLOCK),
    );
    painter.hline(
        inner.left()..=inner.right(),
        storage.bottom() + pad * 0.5,
        egui::Stroke::new(1.0, metrics.border),
    );
    speed_block(&painter, speed, summary, language, metrics);
    storage_block(ui, storage, summary, language, metrics, layout);
}

pub(crate) fn download_toggle_node(index: usize) -> u64 {
    NODE_DOWNLOAD_BASE + index as u64 * 2
}

pub(crate) fn download_remove_node(index: usize) -> u64 {
    NODE_DOWNLOAD_BASE + index as u64 * 2 + 1
}

fn padding(metrics: UiMetrics) -> f32 {
    metrics.control_gap * 1.25
}

fn row_height(metrics: UiMetrics) -> f32 {
    padding(metrics) * 2.0 + 62.0
}

fn episode_grid(width: f32, metrics: UiMetrics) -> (usize, f32) {
    let gap = metrics.control_gap;
    let columns = (((width + gap) / (300.0 + gap)).floor() as usize).max(1);
    (columns, (width - gap * (columns - 1) as f32) / columns as f32)
}

fn episode_height(card_width: f32, metrics: UiMetrics) -> f32 {
    card_width * 9.0 / 16.0
        + metrics.screen_card_title_size
        + metrics.screen_card_subtitle_size * 2.0
        + 28.0
}

pub fn download_group_height(
    group: &DownloadGroup,
    width: f32,
    metrics: UiMetrics,
    list: bool,
) -> f32 {
    let gap = metrics.control_gap;
    let count = group.rows.len();
    if list {
        return count as f32 * (row_height(metrics) + gap);
    }
    let (columns, card) = episode_grid(width, metrics);
    count.div_ceil(columns) as f32 * (episode_height(card, metrics) + gap * 1.5)
}

fn round_button(
    ui: &mut Ui,
    rect: Rect,
    id: Id,
    icon: Option<egui::TextureId>,
    node: u64,
    metrics: UiMetrics,
    layout: &mut HomeLayout,
) {
    let response = ui.interact(rect, id, Sense::click());
    let fill = if response.hovered() { 46 } else { 24 };
    ui.painter()
        .circle_filled(rect.center(), rect.height() * 0.5, ink(fill));
    if let Some(icon) = icon {
        ui.painter().image(
            icon,
            Rect::from_center_size(rect.center(), Vec2::splat(rect.height() * 0.44)),
            full_uv(),
            metrics.text_primary,
        );
    }
    layout.focusable.push((node, rect));
    if response.clicked() {
        layout.activated = Some(node);
    }
}

fn status_label(row: &DownloadRow, language: &str) -> String {
    match row.status {
        DownloadStatus::Downloading => format!("{:.0}%", row.percent),
        DownloadStatus::Paused => localized("downloads.status_paused", language),
        DownloadStatus::Completed => localized("downloads.status_downloaded", language),
        DownloadStatus::Queued => localized("downloads.status_queued", language),
        DownloadStatus::Failed => match row.error.as_deref() {
            Some("unsupported_source") => localized("downloads.unsupported_source", language),
            Some("playlist_stream") => localized("downloads.playlist_stream", language),
            Some("no_space") => localized("downloads.no_space", language),
            _ => localized("downloads.status_failed", language),
        },
    }
}

fn bytes_label(row: &DownloadRow, language: &str) -> String {
    let done = format_bytes(row.bytes_done);
    let mut text = match row.bytes_total {
        Some(total) => localized("downloads.bytes_progress", language)
            .replacen("%s", &done, 1)
            .replacen("%s", &format_bytes(total), 1),
        None => done,
    };
    if row.status == DownloadStatus::Downloading && row.speed_bps >= 1.0 {
        text.push_str(&format!(" · {}/s", format_bytes(row.speed_bps as u64)));
    }
    text
}

fn timing_label(row: &DownloadRow, language: &str) -> String {
    let elapsed = localized("downloads.elapsed", language).replacen(
        "%s",
        &format_duration(row.elapsed_ms),
        1,
    );
    match (row.status, row.eta_ms) {
        (DownloadStatus::Downloading, Some(eta)) => format!(
            "{elapsed} · {}",
            localized("downloads.remaining", language).replacen("%s", &format_duration(eta), 1)
        ),
        _ => elapsed,
    }
}

#[allow(clippy::too_many_arguments)]
fn download_row(
    ui: &mut Ui,
    rect: Rect,
    row: &DownloadRow,
    index: usize,
    language: &str,
    metrics: UiMetrics,
    assets: &mut impl HomeAssets,
    layout: &mut HomeLayout,
) {
    let painter = ui.painter().clone();
    let pad = padding(metrics);
    painter.rect_filled(rect, metrics.card_radius, metrics.surface_raised);

    let button = 40.0;
    let mut right = rect.right() - pad;
    let remove = Rect::from_center_size(
        Pos2::new(right - button * 0.5, rect.center().y),
        Vec2::splat(button),
    );
    round_button(
        ui,
        remove,
        Id::new(("download-remove", &row.id)),
        assets.icon("Delete"),
        download_remove_node(index),
        metrics,
        layout,
    );
    right = remove.left() - metrics.control_gap;
    if row.status != DownloadStatus::Completed {
        let toggle = Rect::from_center_size(
            Pos2::new(right - button * 0.5, rect.center().y),
            Vec2::splat(button),
        );
        let icon = match row.status {
            DownloadStatus::Downloading | DownloadStatus::Queued => "PauseFilled",
            DownloadStatus::Paused => "PlayFilled",
            _ => "Refresh",
        };
        round_button(
            ui,
            toggle,
            Id::new(("download-toggle", &row.id)),
            assets.icon(icon),
            download_toggle_node(index),
            metrics,
            layout,
        );
        right = toggle.left() - metrics.control_gap;
    }

    let left = rect.left() + pad;
    let width = (right - left).max(1.0);
    let top = rect.top() + pad;

    let status = status_label(row, language);
    let status_rect = draw_text(
        &painter,
        Pos2::new(left + width, top),
        Align2::RIGHT_TOP,
        &status,
        crate::fonts::medium(metrics.screen_card_title_size),
        if row.status == DownloadStatus::Failed {
            metrics.text_secondary
        } else {
            metrics.text_primary
        },
        None,
    );

    let mut title_x = left;
    if row.season.is_some() || row.episode.is_some() {
        let code = localized("format.season_episode_code", language)
            .replacen("%s", &row.season.unwrap_or(0).to_string(), 1)
            .replacen("%s", &row.episode.unwrap_or(0).to_string(), 1);
        let used = draw_text(
            &painter,
            Pos2::new(left, top),
            Align2::LEFT_TOP,
            &code,
            crate::fonts::medium(metrics.screen_card_title_size),
            metrics.text_muted,
            None,
        );
        title_x = used.right() + metrics.control_gap;
    }
    draw_text(
        &painter,
        Pos2::new(title_x, top),
        Align2::LEFT_TOP,
        &row.title,
        crate::fonts::medium(metrics.screen_card_title_size),
        metrics.text_primary,
        Some((status_rect.left() - title_x - metrics.control_gap).max(1.0)),
    );

    let bar = Rect::from_min_size(Pos2::new(left, top + 30.0), Vec2::new(width, 6.0));
    painter.rect_filled(bar, 3.0, metrics.border);
    let fraction = match row.status {
        DownloadStatus::Completed => 1.0,
        _ => (row.percent / 100.0).clamp(0.0, 1.0),
    };
    if fraction > 0.0 {
        painter.rect_filled(
            Rect::from_min_size(bar.min, Vec2::new((bar.width() * fraction).max(6.0), 6.0)),
            3.0,
            if row.status == DownloadStatus::Failed {
                metrics.text_muted
            } else {
                metrics.accent
            },
        );
    }

    let detail_y = bar.bottom() + 8.0;
    let timing = timing_label(row, language);
    let timing_rect = draw_text(
        &painter,
        Pos2::new(left + width, detail_y),
        Align2::RIGHT_TOP,
        &timing,
        crate::fonts::regular(metrics.screen_card_subtitle_size),
        metrics.text_muted,
        Some(width * 0.55),
    );
    draw_text(
        &painter,
        Pos2::new(left, detail_y),
        Align2::LEFT_TOP,
        bytes_label(row, language),
        crate::fonts::regular(metrics.screen_card_subtitle_size),
        metrics.text_secondary,
        Some((timing_rect.left() - left - metrics.control_gap).max(1.0)),
    );
}

fn banner_summary(group: &DownloadGroup, language: &str) -> String {
    let active = group.active();
    if active > 0 {
        format!(
            "{} · {}",
            localized("downloads.active_count", language).replacen("%s", &active.to_string(), 1),
            format_bytes(group.bytes_done())
        )
    } else {
        localized("downloads.folder_summary", language)
            .replacen("%s", &group.completed().to_string(), 1)
            .replacen("%s", &format_bytes(group.bytes_done()), 1)
    }
}

#[allow(clippy::too_many_arguments)]
pub fn download_poster(
    ui: &mut Ui,
    group: &DownloadGroup,
    index: usize,
    rect: Rect,
    language: &str,
    metrics: UiMetrics,
    assets: &mut impl HomeAssets,
    layout: &mut HomeLayout,
) {
    let painter = ui.painter().clone();
    let card = crate::HomeCard {
        id: Some(group.id.clone()),
        title: group.title.clone(),
        subtitle: banner_summary(group, language),
        artwork_url: group.poster.clone().or_else(|| group.background.clone()),
        ..Default::default()
    };
    poster_card(&painter, rect, &card, metrics, assets, false);
    if !painter.clip_rect().intersects(rect) {
        return;
    }

    let node = NODE_DOWNLOAD_GROUP_BASE + index as u64;
    let response = ui.interact(rect, Id::new(("download-poster", node)), Sense::click());
    layout.focusable.push((node, rect));
    if response.clicked() {
        layout.activated = Some(node);
    }
    if response.hovered() {
        painter.rect_filled(rect, metrics.card_radius, ink(14));
    }

    let pad = padding(metrics);
    let status = if let Some(fraction) = group.active_fraction() {
        Some((format!("{:.0}%", fraction * 100.0), Some(fraction)))
    } else if group.active() > 0 {
        Some((String::new(), None))
    } else if group.any(DownloadStatus::Failed) {
        Some((localized("downloads.status_failed", language), None))
    } else if group.any(DownloadStatus::Paused) {
        Some((localized("downloads.status_paused", language), None))
    } else {
        None
    };
    let Some((headline, fraction)) = status else {
        return;
    };
    let height = (rect.height() * 0.34).max(72.0);
    let scrim = Rect::from_min_max(
        Pos2::new(rect.left(), rect.bottom() - height),
        rect.right_bottom(),
    );
    paint_vertical_gradient(&painter, scrim, Color32::TRANSPARENT, shade(Shade::Scrim));
    let mut y = rect.bottom() - pad;
    if let Some(fraction) = fraction {
        let bar = Rect::from_min_size(
            Pos2::new(rect.left() + pad, y - 4.0),
            Vec2::new(rect.width() - pad * 2.0, 4.0),
        );
        painter.rect_filled(bar, 2.0, ink(60));
        painter.rect_filled(
            Rect::from_min_size(bar.min, Vec2::new((bar.width() * fraction).max(4.0), 4.0)),
            2.0,
            metrics.accent,
        );
        y = bar.top() - 6.0;
    }
    if group.active() > 0 {
        let mut timing = localized("downloads.elapsed", language).replacen(
            "%s",
            &format_duration(group.elapsed_ms()),
            1,
        );
        if let Some(eta) = group.eta_ms() {
            timing = format!(
                "{} · {timing}",
                localized("downloads.remaining", language).replacen("%s", &format_duration(eta), 1)
            );
        }
        let timing_rect = draw_text(
            &painter,
            Pos2::new(rect.left() + pad, y),
            Align2::LEFT_BOTTOM,
            timing,
            crate::fonts::regular(metrics.screen_card_subtitle_size),
            metrics.text_secondary,
            Some(rect.width() - pad * 2.0),
        );
        y = timing_rect.top() - 2.0;
        let speed = draw_text(
            &painter,
            Pos2::new(rect.right() - pad, y),
            Align2::RIGHT_BOTTOM,
            format!("{}/s", format_bytes(group.speed_bps() as u64)),
            crate::fonts::regular(metrics.screen_card_subtitle_size),
            metrics.text_secondary,
            Some(rect.width() * 0.5),
        );
        draw_text(
            &painter,
            Pos2::new(rect.left() + pad, y),
            Align2::LEFT_BOTTOM,
            headline,
            crate::fonts::medium(metrics.screen_card_title_size + 2.0),
            metrics.text_primary,
            Some(rect.width() - speed.width() - pad * 3.0),
        );
    } else {
        draw_text(
            &painter,
            Pos2::new(rect.left() + pad, y),
            Align2::LEFT_BOTTOM,
            headline,
            crate::fonts::medium(metrics.screen_card_title_size),
            metrics.text_primary,
            Some(rect.width() - pad * 2.0),
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn episode_card(
    ui: &mut Ui,
    rect: Rect,
    row: &DownloadRow,
    group: &DownloadGroup,
    index: usize,
    language: &str,
    metrics: UiMetrics,
    assets: &mut impl HomeAssets,
    layout: &mut HomeLayout,
) {
    let painter = ui.painter().clone();
    let radius = metrics.card_radius;
    let thumb = Rect::from_min_size(rect.min, Vec2::new(rect.width(), rect.width() * 9.0 / 16.0));
    painter.rect_filled(thumb, radius, metrics.surface_raised);
    rounded_artwork(
        &painter,
        thumb,
        radius,
        row.thumbnail
            .as_deref()
            .or(group.background.as_deref())
            .or(group.poster.as_deref()),
        artwork_target_size(thumb.size(), painter.ctx().pixels_per_point()),
        ArtworkPriority::Visible,
        Color32::WHITE,
        assets,
    );
    let scrim = Rect::from_min_max(
        Pos2::new(thumb.left(), thumb.bottom() - thumb.height() * 0.5),
        thumb.right_bottom(),
    );
    paint_vertical_gradient(&painter, scrim, Color32::TRANSPARENT, shade(Shade::Scrim));

    let pad = padding(metrics);
    let fraction = match row.status {
        DownloadStatus::Completed => 1.0,
        _ => (row.percent / 100.0).clamp(0.0, 1.0),
    };
    let bar = Rect::from_min_size(
        Pos2::new(thumb.left() + pad, thumb.bottom() - pad - 4.0),
        Vec2::new(thumb.width() - pad * 2.0, 4.0),
    );
    painter.rect_filled(bar, 2.0, ink(60));
    if fraction > 0.0 {
        painter.rect_filled(
            Rect::from_min_size(bar.min, Vec2::new((bar.width() * fraction).max(4.0), 4.0)),
            2.0,
            metrics.accent,
        );
    }
    draw_text(
        &painter,
        Pos2::new(bar.left(), bar.top() - 6.0),
        Align2::LEFT_BOTTOM,
        status_label(row, language),
        crate::fonts::medium(metrics.screen_card_title_size),
        metrics.text_primary,
        Some(bar.width()),
    );

    let button = 34.0;
    let mut right = thumb.right() - pad;
    let remove = Rect::from_center_size(
        Pos2::new(right - button * 0.5, thumb.top() + pad + button * 0.5),
        Vec2::splat(button),
    );
    round_button(
        ui,
        remove,
        Id::new(("download-remove", &row.id)),
        assets.icon("Delete"),
        download_remove_node(index),
        metrics,
        layout,
    );
    right = remove.left() - metrics.control_gap;
    if row.status != DownloadStatus::Completed {
        let toggle = Rect::from_center_size(
            Pos2::new(right - button * 0.5, remove.center().y),
            Vec2::splat(button),
        );
        let icon = match row.status {
            DownloadStatus::Downloading | DownloadStatus::Queued => "PauseFilled",
            DownloadStatus::Paused => "PlayFilled",
            _ => "Refresh",
        };
        round_button(
            ui,
            toggle,
            Id::new(("download-toggle", &row.id)),
            assets.icon(icon),
            download_toggle_node(index),
            metrics,
            layout,
        );
    }

    let mut y = thumb.bottom() + 8.0;
    let mut title_x = rect.left();
    if row.season.is_some() || row.episode.is_some() {
        let code = localized("format.season_episode_code", language)
            .replacen("%s", &row.season.unwrap_or(0).to_string(), 1)
            .replacen("%s", &row.episode.unwrap_or(0).to_string(), 1);
        let used = draw_text(
            &painter,
            Pos2::new(title_x, y),
            Align2::LEFT_TOP,
            &code,
            crate::fonts::medium(metrics.screen_card_title_size),
            metrics.text_muted,
            None,
        );
        title_x = used.right() + metrics.control_gap;
    }
    let title = draw_text(
        &painter,
        Pos2::new(title_x, y),
        Align2::LEFT_TOP,
        &row.title,
        crate::fonts::medium(metrics.screen_card_title_size),
        metrics.text_primary,
        Some((rect.right() - title_x).max(1.0)),
    );
    y = title.bottom() + 4.0;
    for line in [bytes_label(row, language), timing_label(row, language)] {
        let used = draw_text(
            &painter,
            Pos2::new(rect.left(), y),
            Align2::LEFT_TOP,
            line,
            crate::fonts::regular(metrics.screen_card_subtitle_size),
            metrics.text_muted,
            Some(rect.width()),
        );
        y = used.bottom() + 2.0;
    }
}

#[allow(clippy::too_many_arguments)]
pub fn download_group(
    ui: &mut Ui,
    group: &DownloadGroup,
    first_index: usize,
    origin: Pos2,
    width: f32,
    list: bool,
    language: &str,
    metrics: UiMetrics,
    assets: &mut impl HomeAssets,
    layout: &mut HomeLayout,
) {
    let visible = ui.painter().clip_rect();
    let gap = metrics.control_gap;
    if list {
        let height = row_height(metrics);
        for (offset, row) in group.rows.iter().enumerate() {
            let rect = Rect::from_min_size(
                Pos2::new(origin.x, origin.y + offset as f32 * (height + gap)),
                Vec2::new(width, height),
            );
            if visible.intersects(rect) {
                download_row(ui, rect, row, first_index + offset, language, metrics, assets, layout);
            }
        }
        return;
    }
    let (columns, card) = episode_grid(width, metrics);
    let height = episode_height(card, metrics);
    for (offset, row) in group.rows.iter().enumerate() {
        let rect = Rect::from_min_size(
            Pos2::new(
                origin.x + (offset % columns) as f32 * (card + gap),
                origin.y + (offset / columns) as f32 * (height + gap * 1.5),
            ),
            Vec2::new(card, height),
        );
        if visible.intersects(rect) {
            episode_card(ui, rect, row, group, first_index + offset, language, metrics, assets, layout);
        }
    }
}

fn section_heading(painter: &Painter, pos: Pos2, key: &str, count: usize, language: &str, metrics: UiMetrics) {
    let used = draw_text(
        painter,
        pos,
        Align2::LEFT_TOP,
        localized(key, language),
        crate::fonts::medium(metrics.screen_card_title_size + 4.0),
        metrics.text_primary,
        None,
    );
    draw_text(
        painter,
        Pos2::new(used.right() + metrics.control_gap, pos.y + 2.0),
        Align2::LEFT_TOP,
        count.to_string(),
        crate::fonts::regular(metrics.screen_card_title_size),
        metrics.text_muted,
        None,
    );
}

fn heading_height(metrics: UiMetrics) -> f32 {
    metrics.screen_card_title_size + 4.0 + metrics.control_gap * 1.5
}

const SECTIONS: [(&str, bool); 2] = [
    ("downloads.section_active", false),
    ("downloads.section_done", true),
];

fn section_members(groups: &[DownloadGroup], done: bool) -> Vec<usize> {
    groups
        .iter()
        .enumerate()
        .filter(|(_, group)| (group.completed() == group.rows.len()) == done)
        .map(|(index, _)| index)
        .collect()
}

pub fn download_sections_height(groups: &[DownloadGroup], width: f32, metrics: UiMetrics) -> f32 {
    let grid = crate::PosterGrid::new(width, metrics);
    SECTIONS
        .iter()
        .map(|(_, done)| section_members(groups, *done).len())
        .filter(|count| *count > 0)
        .map(|count| heading_height(metrics) + grid.height(count) + metrics.section_gap)
        .sum()
}

#[allow(clippy::too_many_arguments)]
pub fn download_sections(
    ui: &mut Ui,
    groups: &[DownloadGroup],
    origin: Pos2,
    width: f32,
    language: &str,
    metrics: UiMetrics,
    assets: &mut impl HomeAssets,
    layout: &mut HomeLayout,
) {
    let grid = crate::PosterGrid::new(width, metrics);
    let painter = ui.painter().clone();
    let mut y = origin.y;
    for (key, done) in SECTIONS {
        let members = section_members(groups, done);
        if members.is_empty() {
            continue;
        }
        section_heading(&painter, Pos2::new(origin.x, y), key, members.len(), language, metrics);
        y += heading_height(metrics);
        let top = Pos2::new(origin.x, y);
        for (slot, index) in members.iter().enumerate() {
            let cell = grid.cell(top, slot);
            let poster = Rect::from_min_size(cell.min, Vec2::new(grid.card_width, grid.poster_height));
            download_poster(ui, &groups[*index], *index, poster, language, metrics, assets, layout);
        }
        y += grid.height(members.len()) + metrics.section_gap;
    }
}
