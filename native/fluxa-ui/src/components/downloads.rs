use super::*;
use crate::{
    DownloadGroup, DownloadRow, DownloadStatus, DownloadsSummary, NODE_DOWNLOAD_BASE,
    NODE_DOWNLOAD_LOCATION, format_bytes, format_duration, localized,
};

const SUMMARY_WIDE: f32 = 720.0;
const SUMMARY_BLOCK: f32 = 112.0;

pub fn download_summary_height(width: f32, metrics: UiMetrics) -> f32 {
    let pad = padding(metrics);
    if width >= SUMMARY_WIDE {
        SUMMARY_BLOCK + pad * 2.0
    } else {
        SUMMARY_BLOCK * 2.0 + pad * 3.0
    }
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
    let (speed, storage) = if width >= SUMMARY_WIDE {
        let half = (inner.width() - pad * 2.0) * 0.5;
        (
            Rect::from_min_size(inner.min, Vec2::new(half, SUMMARY_BLOCK)),
            Rect::from_min_size(
                Pos2::new(inner.right() - half, inner.top()),
                Vec2::new(half, SUMMARY_BLOCK),
            ),
        )
    } else {
        (
            Rect::from_min_size(inner.min, Vec2::new(inner.width(), SUMMARY_BLOCK)),
            Rect::from_min_size(
                Pos2::new(inner.left(), inner.top() + SUMMARY_BLOCK + pad),
                Vec2::new(inner.width(), SUMMARY_BLOCK),
            ),
        )
    };
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

fn header_height(width: f32) -> f32 {
    (width * 0.16).clamp(96.0, 150.0)
}

pub fn download_group_height(group: &DownloadGroup, width: f32, metrics: UiMetrics) -> f32 {
    let gap = metrics.control_gap;
    header_height(width) + gap + group.rows.len() as f32 * (row_height(metrics) + gap)
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

#[allow(clippy::too_many_arguments)]
pub fn download_group(
    ui: &mut Ui,
    group: &DownloadGroup,
    first_index: usize,
    origin: Pos2,
    width: f32,
    language: &str,
    metrics: UiMetrics,
    assets: &mut impl HomeAssets,
    layout: &mut HomeLayout,
) {
    let visible = ui.painter().clip_rect();
    let gap = metrics.control_gap;
    let header = Rect::from_min_size(origin, Vec2::new(width, header_height(width)));
    let total = download_group_height(group, width, metrics);
    if !visible.intersects(Rect::from_min_size(origin, Vec2::new(width, total))) {
        return;
    }

    let painter = ui.painter().clone();
    let radius = metrics.card_radius;
    painter.rect_filled(header, radius, metrics.surface_raised);
    rounded_artwork(
        &painter,
        header,
        radius,
        group.background.as_deref().or(group.poster.as_deref()),
        artwork_target_size(header.size(), painter.ctx().pixels_per_point()),
        ArtworkPriority::Visible,
        Color32::WHITE,
        assets,
    );
    painter.rect_filled(header, radius, shade(Shade::Scrim));

    let inset = padding(metrics) * 1.5;
    let bounds = Vec2::new(header.width() * 0.4, header.height() * 0.5);
    let logo = title_logo(
        painter.ctx().pixels_per_point(),
        group.logo.as_deref(),
        bounds,
        ArtworkPriority::Visible,
        assets,
    );
    match logo {
        Some((texture, size)) => {
            let rect = Rect::from_min_size(
                Pos2::new(header.left() + inset, header.center().y - size.y * 0.5),
                size,
            );
            texture_image(&painter, texture, rect, full_uv(), Color32::WHITE);
        }
        None => {
            draw_text(
                &painter,
                Pos2::new(header.left() + inset, header.center().y),
                Align2::LEFT_CENTER,
                &group.title,
                crate::fonts::medium(metrics.screen_title_size * 0.8),
                metrics.text_primary,
                Some(header.width() * 0.6),
            );
        }
    }
    if group.is_series {
        let summary = localized("downloads.folder_summary", language)
            .replacen("%s", &group.completed().to_string(), 1)
            .replacen("%s", &format_bytes(group.bytes_done()), 1);
        draw_text(
            &painter,
            Pos2::new(header.right() - inset, header.bottom() - inset),
            Align2::RIGHT_BOTTOM,
            summary,
            crate::fonts::regular(metrics.screen_card_subtitle_size),
            metrics.text_primary,
            Some(header.width() * 0.5),
        );
    }

    let mut y = header.bottom() + gap;
    let height = row_height(metrics);
    for (offset, row) in group.rows.iter().enumerate() {
        let rect = Rect::from_min_size(Pos2::new(origin.x, y), Vec2::new(width, height));
        if visible.intersects(rect) {
            download_row(
                ui,
                rect,
                row,
                first_index + offset,
                language,
                metrics,
                assets,
                layout,
            );
        }
        y += height + gap;
    }
}
