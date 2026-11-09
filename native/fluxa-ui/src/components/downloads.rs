use super::*;
use crate::Space;
use crate::flex::{self, Align, Insets, Justify};
use crate::kit::{Lattice, Part, Scene};
use crate::{
    DownloadGroup, DownloadRow, DownloadStatus, DownloadsSummary, NODE_DOWNLOAD_BASE,
    NODE_DOWNLOAD_GROUP_BASE, NODE_DOWNLOAD_LOCATION, format_bytes, format_duration, localized,
};

const TEXT: usize = 100;
const TOGGLE: usize = 101;
const REMOVE: usize = 102;
const TRACK: usize = 103;
const HEAD: usize = 104;
const THUMB: usize = 107;
const SCRIM: usize = 108;
const SPEED: usize = 105;
const TIMING: usize = 106;

const DOWNLOAD_BLUE: Color32 = Color32::from_rgb(110, 196, 255);

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

#[allow(clippy::too_many_arguments)]
pub fn download_status(
    ui: &mut Ui,
    summary: &DownloadsSummary,
    right_top: Pos2,
    max_width: f32,
    language: &str,
    metrics: UiMetrics,
    assets: &mut impl HomeAssets,
    layout: &mut HomeLayout,
) {
    let painter = ui.painter().clone();
    let line = metrics.control_downloads_line * 2.0;
    let strip = flex::row([
        flex::column([
            flex::leaf(Vec2::new(0.0, line)).id(1),
            flex::leaf(Vec2::new(0.0, line)).id(2),
        ])
        .align(Align::Stretch)
        .grow(1.0),
        flex::leaf(Vec2::splat(metrics.downloads_change_size)).id(0),
    ])
    .gap(metrics.control_gap)
    .align(Align::Center)
    .solve(Rect::from_min_size(
        Pos2::new(right_top.x - max_width, right_top.y),
        Vec2::new(max_width, metrics.downloads_tab_center * 2.0),
    ));
    let button = strip.find(0).unwrap_or(Rect::NOTHING);
    let response = ui
        .scope_builder(egui::UiBuilder::new().max_rect(button), |ui| {
            icon_button(
                ui,
                assets.icon("FolderOpen"),
                metrics.downloads_change_size,
                metrics.text_primary,
                false,
                false,
                true,
            )
        })
        .inner;
    focus_action(layout, NODE_DOWNLOAD_LOCATION, &response);

    let Some(location) = summary.location.as_ref() else {
        return;
    };
    let first = strip.find(1).unwrap_or(button);
    let second = strip.find(2).unwrap_or(button);
    let free = draw_text(
        &painter,
        first.right_center(),
        Align2::RIGHT_CENTER,
        localized("downloads.free", language).replacen("%s", &format_bytes(location.free_bytes), 1),
        crate::fonts::regular(metrics.screen_card_title_size),
        metrics.text_primary,
        None,
    );
    if summary.speed_bps >= 1.0 {
        draw_text(
            &painter,
            Pos2::new(
                free.left() - metrics.control_gap * metrics.control_downloads_speed_gap_ratio,
                first.center().y,
            ),
            Align2::RIGHT_CENTER,
            format!("{}/s", format_bytes(summary.speed_bps as u64)),
            crate::fonts::medium(metrics.screen_card_title_size),
            DOWNLOAD_BLUE,
            None,
        );
    }
    draw_text(
        &painter,
        second.right_center(),
        Align2::RIGHT_CENTER,
        &location.path,
        crate::fonts::regular(metrics.screen_card_title_size),
        metrics.text_secondary,
        Some(second.width().max(metrics.control_downloads_path_min)),
    );
}

pub(crate) fn download_toggle_node(index: usize) -> u64 {
    NODE_DOWNLOAD_BASE + index as u64 * 2
}

pub(crate) fn download_remove_node(index: usize) -> u64 {
    NODE_DOWNLOAD_BASE + index as u64 * 2 + 1
}

fn padding(metrics: UiMetrics) -> f32 {
    metrics.control_gap * metrics.control_downloads_pad_ratio
}

fn wrapped_galley(
    painter: &Painter,
    text: impl Into<String>,
    font: FontId,
    color: Color32,
    width: f32,
    max_rows: usize,
) -> std::sync::Arc<egui::Galley> {
    let mut job = egui::text::LayoutJob::simple(text.into(), font, color, width.max(1.0));
    job.wrap.max_rows = max_rows;
    job.wrap.overflow_character = Some('…');
    painter.ctx().fonts_mut(|fonts| fonts.layout_job(job))
}

fn wrapped_label(
    painter: &Painter,
    anchor: Pos2,
    align: Align2,
    text: impl Into<String>,
    font: FontId,
    color: Color32,
    width: f32,
    max_rows: usize,
) -> Rect {
    let galley = wrapped_galley(painter, text, font, color, width, max_rows);
    let rect = align.anchor_size(anchor, galley.size());
    painter.galley(rect.min, galley, color);
    rect
}

const SUBTITLE_LINES: usize = 2;

fn download_grid(width: f32, metrics: UiMetrics) -> crate::PosterGrid {
    let mut grid = crate::PosterGrid::new(width, metrics);
    let extra_lines = (SUBTITLE_LINES - 1) as f32;
    grid.card_height +=
        extra_lines * metrics.screen_card_subtitle_size * metrics.control_downloads_line_ratio;
    grid
}

fn row_height(metrics: UiMetrics) -> f32 {
    padding(metrics) * 2.0 + metrics.control_downloads_row_content
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
        + metrics.control_downloads_card_caption
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
            flex::anchored(
                rect,
                Align2::CENTER_CENTER,
                Vec2::splat(rect.height() * metrics.control_downloads_icon_ratio),
                0.0,
            ),
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

    let button = metrics.control_downloads_button;
    let mut cells = vec![flex::fill().id(TEXT).height(rect.height() - pad * 2.0)];
    if row.status != DownloadStatus::Completed {
        cells.push(flex::leaf(Vec2::splat(button)).id(TOGGLE));
    }
    cells.push(flex::leaf(Vec2::splat(button)).id(REMOVE));
    let placed = flex::row(cells)
        .gap(metrics.control_gap)
        .align(Align::Center)
        .padding(Insets::all(pad))
        .solve(rect);
    let remove = placed.find(REMOVE).unwrap_or(rect);
    round_button(
        ui,
        remove,
        Id::new(("download-remove", &row.id)),
        assets.icon("Delete"),
        download_remove_node(index),
        metrics,
        layout,
    );
    if let Some(toggle) = placed.find(TOGGLE) {
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

    let status = status_label(row, language);
    let status_color = if row.status == DownloadStatus::Failed {
        metrics.text_secondary
    } else {
        metrics.text_primary
    };
    let title_font = crate::fonts::medium(metrics.screen_card_title_size);
    let subtitle_font = crate::fonts::regular(metrics.screen_card_subtitle_size);
    let code = (row.season.is_some() || row.episode.is_some()).then(|| {
        localized("format.season_episode_code", language)
            .replacen("%s", &row.season.unwrap_or(0).to_string(), 1)
            .replacen("%s", &row.episode.unwrap_or(0).to_string(), 1)
    });
    let mut scene = Scene::default();
    let status_width = flex::text_size(&painter, &status, &title_font).x;
    let status_node = scene
        .add(
            &painter,
            Part::text(status, title_font.clone(), status_color),
        )
        .min_width(status_width);
    let code_node = code.map(|code| {
        let width = flex::text_size(&painter, &code, &title_font).x;
        scene
            .add(
                &painter,
                Part::text(code, title_font.clone(), metrics.text_muted),
            )
            .min_width(width)
    });
    let title_node = scene.add(
        &painter,
        Part::text(&row.title, title_font, metrics.text_primary),
    );
    let timing = timing_label(row, language);
    let timing_width = flex::text_size(&painter, &timing, &subtitle_font).x;
    let timing_node = scene
        .add(
            &painter,
            Part::text(timing, subtitle_font.clone(), metrics.text_muted),
        )
        .min_width(timing_width);
    let bytes_node = scene.add(
        &painter,
        Part::text(
            bytes_label(row, language),
            subtitle_font,
            metrics.text_secondary,
        ),
    );
    let lines = flex::column([
        flex::row([
            flex::row(code_node.into_iter().chain([title_node]))
                .gap(Space::S12.px(&metrics))
                .align(Align::Center)
                .justify(Justify::Start)
                .grow(1.0)
                .min_width(0.0),
            status_node,
        ])
        .align(Align::Center),
        flex::leaf(Vec2::new(0.0, metrics.control_downloads_bar)).id(TRACK),
        flex::row([bytes_node, flex::fill(), timing_node]).align(Align::Center),
    ])
    .gap(Space::S6.px(&metrics))
    .align(Align::Stretch)
    .solve(placed.find(TEXT).unwrap_or(rect));
    scene.paint(&lines, &painter, assets, metrics);
    let bar = lines.find(TRACK).unwrap_or(rect);
    painter.rect_filled(bar, bar.height() * 0.5, metrics.border);
    let fraction = match row.status {
        DownloadStatus::Completed => 1.0,
        _ => (row.percent / 100.0).clamp(0.0, 1.0),
    };
    if fraction > 0.0 {
        painter.rect_filled(
            Rect::from_min_size(
                bar.min,
                Vec2::new((bar.width() * fraction).max(bar.height()), bar.height()),
            ),
            bar.height() * 0.5,
            if row.status == DownloadStatus::Failed {
                metrics.text_muted
            } else {
                DOWNLOAD_BLUE
            },
        );
    }
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
        artwork_url: group.poster.clone().or_else(|| group.background.clone()),
        ..Default::default()
    };
    poster_card(&painter, rect, &card, metrics, assets, false);
    if !painter.clip_rect().intersects(rect) {
        return;
    }
    let summary_top = rect.bottom()
        + metrics.control_gap
        + metrics.screen_card_title_size
        + Space::S2.px(&metrics);
    wrapped_label(
        &painter,
        Pos2::new(rect.left(), summary_top),
        Align2::LEFT_TOP,
        banner_summary(group, language),
        crate::fonts::regular(metrics.screen_card_subtitle_size),
        metrics.ink(tone::subtle()),
        rect.width(),
        SUBTITLE_LINES,
    );

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
    let height = (rect.height() * metrics.control_downloads_scrim_ratio)
        .max(metrics.control_downloads_scrim_min);
    let scrim = flex::anchored(
        rect,
        Align2::LEFT_BOTTOM,
        Vec2::new(rect.width(), height),
        0.0,
    );
    paint_vertical_gradient(&painter, scrim, Color32::TRANSPARENT, shade(Shade::Scrim));
    let content_width = rect.width() - pad * 2.0;
    let thin = metrics.control_downloads_bar_thin;
    let tight = Space::S2.px(&metrics);
    let mut stack = Vec::new();
    let mut timing_galley = None;
    let mut speed_text = None;
    let mut headline_row = false;
    let headline_font = crate::fonts::medium(metrics.screen_card_title_size + 2.0);
    let small_font = crate::fonts::regular(metrics.screen_card_subtitle_size);
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
        let speed = format!("{}/s", format_bytes(group.speed_bps() as u64));
        let speed_size = flex::text_size(&painter, &speed, &small_font);
        let headline_size = flex::text_size(&painter, &headline, &headline_font);
        headline_row = headline_size.x + speed_size.x + metrics.control_gap <= content_width;
        let galley = wrapped_galley(
            &painter,
            timing,
            small_font.clone(),
            metrics.text_secondary,
            content_width,
            2,
        );
        if headline_row {
            stack.push(
                flex::row([
                    flex::leaf(headline_size),
                    flex::fill(),
                    flex::leaf(speed_size),
                ])
                .width(content_width)
                .id(HEAD)
                .align(Align::Center),
            );
        } else {
            stack.push(flex::leaf(headline_size).min_width(0.0).id(HEAD));
            stack.push(flex::leaf(Vec2::new(0.0, tight)));
            stack.push(flex::leaf(speed_size).min_width(0.0).id(SPEED));
        }
        stack.push(flex::leaf(Vec2::new(0.0, tight)));
        stack.push(flex::leaf(galley.size()).id(TIMING));
        timing_galley = Some(galley);
        speed_text = Some(speed);
    } else {
        let line = crate::fonts::medium(metrics.screen_card_title_size);
        let height = flex::text_size(&painter, &headline, &line).y;
        stack.push(flex::leaf(Vec2::new(content_width, height)).id(HEAD));
    }
    if fraction.is_some() {
        stack.push(flex::leaf(Vec2::new(0.0, Space::S6.px(&metrics))));
        stack.push(flex::leaf(Vec2::new(content_width, thin)).id(TRACK));
    }
    let placed = flex::column(stack)
        .justify(Justify::End)
        .padding(Insets::all(pad))
        .solve(rect);
    if let (Some(fraction), Some(bar)) = (fraction, placed.find(TRACK)) {
        painter.rect_filled(bar, bar.height() * 0.5, ink(60));
        painter.rect_filled(
            Rect::from_min_size(bar.min, Vec2::new((bar.width() * fraction).max(thin), thin)),
            bar.height() * 0.5,
            DOWNLOAD_BLUE,
        );
    }
    let head = placed.find(HEAD).unwrap_or(rect);
    if let (Some(galley), Some(slot)) = (timing_galley, placed.find(TIMING)) {
        painter.galley(slot.min, galley, metrics.text_secondary);
    }
    if let Some(speed) = speed_text {
        if headline_row {
            draw_text(
                &painter,
                head.right_bottom(),
                Align2::RIGHT_BOTTOM,
                speed,
                small_font,
                metrics.text_secondary,
                None,
            );
            draw_text(
                &painter,
                head.left_bottom(),
                Align2::LEFT_BOTTOM,
                headline,
                headline_font,
                metrics.text_primary,
                None,
            );
        } else {
            let speed_slot = placed.find(SPEED).unwrap_or(head);
            draw_text(
                &painter,
                speed_slot.left_bottom(),
                Align2::LEFT_BOTTOM,
                speed,
                small_font,
                metrics.text_secondary,
                Some(content_width),
            );
            draw_text(
                &painter,
                head.left_bottom(),
                Align2::LEFT_BOTTOM,
                headline,
                headline_font,
                metrics.text_primary,
                Some(content_width),
            );
        }
    } else {
        draw_text(
            &painter,
            head.left_bottom(),
            Align2::LEFT_BOTTOM,
            headline,
            crate::fonts::medium(metrics.screen_card_title_size),
            metrics.text_primary,
            Some(content_width),
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
    let pad = padding(metrics);
    let thin = metrics.control_downloads_bar_thin;
    let button = metrics.control_downloads_card_button;
    let mut buttons = vec![flex::leaf(Vec2::splat(button)).id(REMOVE)];
    if row.status != DownloadStatus::Completed {
        buttons.insert(0, flex::leaf(Vec2::splat(button)).id(TOGGLE));
    }
    let overlay = |align: Align2, node: flex::Node| flex::stack(align, [node]).grow(1.0);
    let thumb_height = rect.width() * metrics.control_downloads_aspect;
    let tree = flex::column([
        flex::stack(
            Align2::LEFT_TOP,
            [
                overlay(
                    Align2::LEFT_BOTTOM,
                    flex::leaf(Vec2::new(
                        rect.width(),
                        thumb_height * metrics.control_downloads_thumb_scrim,
                    ))
                    .id(SCRIM),
                ),
                overlay(
                    Align2::LEFT_BOTTOM,
                    flex::leaf(Vec2::new(rect.width() - pad * 2.0, thin)).id(TRACK),
                )
                .padding(Insets::all(pad)),
                overlay(
                    Align2::RIGHT_TOP,
                    flex::row(buttons).gap(metrics.control_gap),
                )
                .padding(Insets::all(pad)),
            ],
        )
        .width(rect.width())
        .height(thumb_height)
        .id(THUMB),
        flex::fill().id(TEXT),
    ])
    .gap(Space::S8.px(&metrics))
    .solve(rect);
    let thumb = tree.find(THUMB).unwrap_or(rect);
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
    paint_vertical_gradient(
        &painter,
        tree.find(SCRIM).unwrap_or(thumb),
        Color32::TRANSPARENT,
        shade(Shade::Scrim),
    );

    let fraction = match row.status {
        DownloadStatus::Completed => 1.0,
        _ => (row.percent / 100.0).clamp(0.0, 1.0),
    };
    let bar = tree.find(TRACK).unwrap_or(thumb);
    painter.rect_filled(bar, bar.height() * 0.5, ink(60));
    if fraction > 0.0 {
        painter.rect_filled(
            Rect::from_min_size(bar.min, Vec2::new((bar.width() * fraction).max(thin), thin)),
            bar.height() * 0.5,
            DOWNLOAD_BLUE,
        );
    }
    draw_text(
        &painter,
        bar.left_top() - Vec2::new(0.0, Space::S6.px(&metrics)),
        Align2::LEFT_BOTTOM,
        status_label(row, language),
        crate::fonts::medium(metrics.screen_card_title_size),
        metrics.text_primary,
        Some(bar.width()),
    );

    round_button(
        ui,
        tree.find(REMOVE).unwrap_or(thumb),
        Id::new(("download-remove", &row.id)),
        assets.icon("Delete"),
        download_remove_node(index),
        metrics,
        layout,
    );
    if let Some(toggle) = tree.find(TOGGLE) {
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

    let title_font = crate::fonts::medium(metrics.screen_card_title_size);
    let subtitle_font = crate::fonts::regular(metrics.screen_card_subtitle_size);
    let code = (row.season.is_some() || row.episode.is_some()).then(|| {
        localized("format.season_episode_code", language)
            .replacen("%s", &row.season.unwrap_or(0).to_string(), 1)
            .replacen("%s", &row.episode.unwrap_or(0).to_string(), 1)
    });
    let mut scene = Scene::default();
    let code_node = code.map(|code| {
        scene.add(
            &painter,
            Part::text(code, title_font.clone(), metrics.text_muted),
        )
    });
    let title_node = scene.add(
        &painter,
        Part::text(&row.title, title_font, metrics.text_primary),
    );
    let captions = [bytes_label(row, language), timing_label(row, language)].map(|line| {
        scene.add(
            &painter,
            Part::text(line, subtitle_font.clone(), metrics.text_muted),
        )
    });
    let [bytes_node, timing_node] = captions;
    let lines = flex::column([
        flex::row(code_node.into_iter().chain([title_node]))
            .gap(Space::S12.px(&metrics))
            .align(Align::Center),
        flex::row([bytes_node]),
        flex::row([timing_node]),
    ])
    .align(Align::Stretch)
    .gap(Space::S4.px(&metrics))
    .solve(tree.find(TEXT).unwrap_or(rect));
    scene.paint(&lines, &painter, assets, metrics);
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
    let row_gap = gap * metrics.control_downloads_row_gap_ratio;
    let lattice = if list {
        Lattice {
            origin,
            cell: Vec2::new(width, row_height(metrics)),
            gap: 0.0,
            row_gap: gap,
            columns: 1,
        }
    } else {
        let (columns, card) = episode_grid(width, metrics);
        Lattice {
            origin,
            cell: Vec2::new(card, episode_height(card, metrics)),
            gap,
            row_gap,
            columns,
        }
    };
    for (offset, row) in group.rows.iter().enumerate() {
        let rect = lattice.rect(offset);
        if !visible.intersects(rect) {
            continue;
        }
        if list {
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
        } else {
            episode_card(
                ui,
                rect,
                row,
                group,
                first_index + offset,
                language,
                metrics,
                assets,
                layout,
            );
        }
    }
}

fn section_heading(
    ui: &mut Ui,
    pos: Pos2,
    key: &str,
    count: usize,
    language: &str,
    metrics: UiMetrics,
    assets: &mut impl HomeAssets,
) {
    let painter = ui.painter().clone();
    let mut scene = Scene::default();
    let title = scene.add(
        &painter,
        Part::text(
            localized(key, language),
            crate::fonts::medium(metrics.screen_card_title_size + 4.0),
            metrics.text_primary,
        ),
    );
    let total = scene.add(
        &painter,
        Part::text(
            count.to_string(),
            crate::fonts::regular(metrics.screen_card_title_size),
            metrics.text_muted,
        ),
    );
    let placed = flex::row([title, total])
        .gap(Space::S12.px(&metrics))
        .solve(Rect::from_min_size(
            pos,
            Vec2::new(ui.available_width(), heading_height(metrics)),
        ));
    scene.paint(&placed, &painter, assets, metrics);
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
    let grid = download_grid(width, metrics);
    let mut y = origin.y;
    for (key, done) in SECTIONS {
        let members = section_members(groups, done);
        if members.is_empty() {
            continue;
        }
        section_heading(
            ui,
            Pos2::new(origin.x, y),
            key,
            members.len(),
            language,
            metrics,
            assets,
        );
        y += heading_height(metrics);
        let top = Pos2::new(origin.x, y);
        for (slot, index) in members.iter().enumerate() {
            let cell = grid.cell(top, slot);
            let poster =
                Rect::from_min_size(cell.min, Vec2::new(grid.card_width, grid.poster_height));
            download_poster(
                ui,
                &groups[*index],
                *index,
                poster,
                language,
                metrics,
                assets,
                layout,
            );
        }
        y += grid.height(members.len()) + metrics.section_gap;
    }
}
