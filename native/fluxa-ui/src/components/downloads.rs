use super::*;
use crate::Space;
use crate::dp::Dp;
use crate::{DownloadGroup, DownloadRow, DownloadStatus, format_bytes, format_duration, localized};

pub fn download_status_height(metrics: UiMetrics) -> f32 {
    if metrics.compact {
        metrics.downloads_change_size + Space::S8.px(&metrics)
    } else {
        0.0
    }
}

pub fn download_sections_width(width: f32, _metrics: UiMetrics) -> f32 {
    width
}

fn padding(metrics: UiMetrics) -> f32 {
    metrics.control_gap * metrics.control_downloads_pad_ratio
}

const SUBTITLE_LINES: usize = 2;

fn download_grid(width: f32, metrics: UiMetrics) -> crate::PosterGrid {
    let mut grid = crate::PosterGrid::new(width, &metrics);
    let extra_lines = (SUBTITLE_LINES - 1) as f32;
    grid.card_height +=
        extra_lines * metrics.screen_card_subtitle_size * metrics.control_downloads_line_ratio;
    grid
}

fn row_height(metrics: UiMetrics) -> f32 {
    padding(metrics) * 2.0 + Dp::D64.px()
}

fn episode_grid(width: f32, metrics: UiMetrics) -> (usize, f32) {
    let gap = metrics.control_gap;
    let columns =
        (((width + gap) / (metrics.control_downloads_card_min + gap)).floor() as usize).max(1);
    (
        columns,
        (width - gap * (columns - 1) as f32) / columns as f32,
    )
}

fn episode_height(card_width: f32, metrics: UiMetrics) -> f32 {
    card_width * metrics.control_downloads_aspect
        + metrics.screen_card_title_size
        + metrics.screen_card_subtitle_size * 2.0
        + Dp::D28.px()
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
    count.div_ceil(columns) as f32
        * (episode_height(card, metrics) + gap * metrics.control_downloads_row_gap_ratio)
}

pub(crate) fn status_label(row: &DownloadRow, language: &str) -> String {
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

pub(crate) fn bytes_label(row: &DownloadRow, language: &str) -> String {
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

pub(crate) fn timing_label(row: &DownloadRow, language: &str) -> String {
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

fn heading_height(metrics: UiMetrics) -> f32 {
    metrics.screen_card_title_size
        + 4.0
        + metrics.control_gap * metrics.control_downloads_row_gap_ratio
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
    let grid = download_grid(width, metrics);
    SECTIONS
        .iter()
        .map(|(_, done)| section_members(groups, *done).len())
        .filter(|count| *count > 0)
        .map(|count| heading_height(metrics) + grid.height(count) + metrics.section_gap)
        .sum()
}
