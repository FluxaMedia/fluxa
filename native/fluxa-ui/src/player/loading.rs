use super::*;
use crate::draw_text;
use crate::flex;

const BAR: usize = 0;
const LINE_BASE: usize = 1;

pub(super) fn draw_warnings(
    context: &egui::Context,
    origin: Pos2,
    player: &PlayerModel,
    elapsed: f32,
) {
    let rows = player.warnings.len();
    if rows == 0 {
        return;
    }
    let metrics = crate::metrics::active();
    let painter = context.layer_painter(egui::LayerId::new(
        egui::Order::Foreground,
        Id::new("fluxa-player-warnings"),
    ));
    let total = content_warning_duration(rows);
    let ramp = |value: f32| value.clamp(0.0, 1.0);
    let box_alpha = ramp(elapsed / WARNING_BAR).min(ramp((total - elapsed) / WARNING_BAR));
    let font = FontId::proportional(metrics.playerctl_warning_font);
    let lines = player
        .warnings
        .iter()
        .map(|(label, severity)| format!("{label} · {severity}"))
        .collect::<Vec<_>>();
    let text_width = lines
        .iter()
        .map(|line| flex::text_size(&painter, line, &font).x)
        .fold(0.0, f32::max);
    let row_height = metrics.playerctl_warning_row;
    let pad = metrics.playerctl_warning_pad;
    let lines_column = flex::column(
        lines
            .iter()
            .enumerate()
            .map(|(index, _)| flex::leaf(Vec2::new(text_width, row_height)).id(LINE_BASE + index)),
    );
    let bar = flex::leaf(Vec2::new(
        metrics.playerctl_warning_bar_width,
        rows as f32 * row_height,
    ))
    .id(BAR);
    let panel = flex::row([
        bar,
        flex::slot(metrics.playerctl_warning_bar_gap),
        lines_column,
    ])
    .padding(flex::Insets {
        left: metrics.playerctl_warning_bar_inset,
        top: pad,
        right: pad,
        bottom: pad,
    });
    let size = panel.outer_size();
    let placed = panel.solve(Rect::from_min_size(origin, size));
    painter.rect_filled(
        placed.rect,
        radius::control(),
        Color32::from_black_alpha((173.0 * box_alpha) as u8),
    );
    if let Some(bar) = placed.find(BAR) {
        painter.rect_filled(
            Rect::from_min_size(
                bar.min,
                Vec2::new(bar.width(), bar.height() * ramp(elapsed / WARNING_BAR)),
            ),
            bar.width() * 0.5,
            ink((230.0 * box_alpha) as u8),
        );
    }
    let fade_out_start = total - WARNING_BAR - rows as f32 * WARNING_STAGGER - WARNING_FADE;
    for (index, line) in lines.into_iter().enumerate() {
        let appear = ramp((elapsed - WARNING_BAR - index as f32 * WARNING_STAGGER) / WARNING_FADE);
        let leave = fade_out_start + (rows - 1 - index) as f32 * WARNING_STAGGER;
        let alpha = appear.min(1.0 - ramp((elapsed - leave) / WARNING_FADE));
        let Some(slot) = placed.find(LINE_BASE + index) else {
            continue;
        };
        draw_text(
            &painter,
            slot.left_center(),
            Align2::LEFT_CENTER,
            line,
            font.clone(),
            ink((255.0 * alpha) as u8),
            None,
        );
    }
}

pub(super) fn draw_loading(
    context: &egui::Context,
    painter: &egui::Painter,
    rect: Rect,
    player: &PlayerModel,
    assets: &mut impl HomeAssets,
    metrics: UiMetrics,
) {
    painter.rect_filled(rect, 0.0, Color32::BLACK);
    let url = player.background.as_deref();
    let target = backdrop_target_size(rect.width(), context.pixels_per_point());
    components::artwork_image(
        painter,
        rect,
        url,
        target,
        ArtworkPriority::Visible,
        crate::stylesheet::color("dim"),
        assets,
    );
    paint_vertical_gradient(
        painter,
        rect,
        Color32::from_black_alpha(26),
        shade(Shade::Scrim),
    );
    draw_loading_mark(context, painter, rect, player, assets, metrics);
}

pub(super) fn draw_buffering(
    context: &egui::Context,
    painter: &egui::Painter,
    rect: Rect,
    player: &PlayerModel,
    assets: &mut impl HomeAssets,
    metrics: UiMetrics,
) {
    if player.status.is_none() {
        return;
    }
    painter.rect_filled(rect, 0.0, Color32::from_black_alpha(110));
    draw_loading_mark(context, painter, rect, player, assets, metrics);
}

fn draw_loading_mark(
    context: &egui::Context,
    painter: &egui::Painter,
    rect: Rect,
    player: &PlayerModel,
    assets: &mut impl HomeAssets,
    metrics: UiMetrics,
) {
    let center = rect.center();
    let logo_box = Vec2::new(
        metrics
            .playerctl_loading_logo_width
            .min(rect.width() - Space::S32.px(&metrics)),
        metrics.playerctl_loading_logo_height,
    );
    let logo = player.logo.as_deref();
    let logo_texture = components::title_logo(
        context.pixels_per_point(),
        logo,
        logo_box,
        ArtworkPriority::Visible,
        assets,
    );
    let mut rows_top = center.y + metrics.playerctl_loading_text_offset;
    if player.error.is_none() {
        match logo_texture {
            Some((texture, fitted)) => {
                let logo_rect = flex::anchored(rect, Align2::CENTER_CENTER, fitted, 0.0);
                match player.load_progress {
                    Some(progress) => {
                        let shown = crate::motion::ease(
                            context,
                            Id::new("fluxa-player-load-progress"),
                            progress,
                            crate::motion::slow(),
                            crate::motion::Ease::Linear,
                        );
                        painter.image(
                            texture,
                            logo_rect,
                            full_uv(),
                            Color32::from_rgba_unmultiplied(184, 184, 184, 89),
                        );
                        let mut lit = logo_rect;
                        lit.set_right(logo_rect.left() + logo_rect.width() * shown);
                        painter.with_clip_rect(lit).image(
                            texture,
                            logo_rect,
                            full_uv(),
                            Color32::WHITE,
                        );
                    }
                    None => {
                        let time = context.input(|input| input.time) as f32;
                        let phase = 0.5 - 0.5 * (time * std::f32::consts::PI / 1.08).cos();
                        let alpha = 0.38 + 0.48 * phase;
                        let scaled =
                            Rect::from_center_size(center, fitted * (0.992 + 0.02 * phase));
                        painter.image(
                            texture,
                            scaled,
                            full_uv(),
                            Color32::from_white_alpha((255.0 * alpha) as u8),
                        );
                        context.request_repaint();
                    }
                }
                rows_top = logo_rect.bottom() + Space::S14.px(&metrics);
            }
            None => {
                draw_text(
                    painter,
                    center,
                    Align2::CENTER_CENTER,
                    &player.title,
                    FontId::proportional(metrics.playerctl_loading_title),
                    text_primary(),
                    None,
                );
            }
        }
    }

    if let Some(error) = player.error.as_deref() {
        let row = Space::S32.px(&metrics);
        let rows = flex::column([
            flex::leaf(Vec2::new(rect.width(), row)).id(0),
            flex::leaf(Vec2::new(rect.width(), row)).id(1),
        ]);
        let placed = rows.solve(flex::anchored(
            rect,
            Align2::CENTER_CENTER,
            rows.outer_size(),
            0.0,
        ));
        let (Some(headline), Some(detail)) = (placed.find(0), placed.find(1)) else {
            return;
        };
        draw_text(
            painter,
            headline.center(),
            Align2::CENTER_CENTER,
            localized("player.playback_error", &player.language),
            FontId::proportional(metrics.text.title),
            text_primary(),
            None,
        );
        draw_text(
            painter,
            detail.center(),
            Align2::CENTER_CENTER,
            truncate_to_width(
                painter,
                error,
                &FontId::proportional(metrics.text.body),
                rect.width() - Space::S32.px(&metrics) * 2.0,
            ),
            FontId::proportional(metrics.text.body),
            metrics.text_secondary,
            None,
        );
        return;
    }
    let status = player
        .status
        .as_ref()
        .map(|(headline, _)| headline.clone())
        .unwrap_or_else(|| localized("player.status_starting_playback", &player.language));
    let mut lines = Vec::new();
    if let Some(episode) = player.episode_title.as_deref() {
        lines.push((
            episode.to_owned(),
            metrics.text.subtitle,
            metrics.text_primary,
        ));
    }
    lines.push((status, metrics.text.body, metrics.text_secondary));
    let rows = flex::column(
        lines
            .iter()
            .enumerate()
            .map(|(index, _)| flex::leaf(Vec2::new(0.0, metrics.playerctl_loading_row)).id(index)),
    )
    .align(flex::Align::Stretch);
    let bounds = Rect::from_min_size(
        Pos2::new(rect.left(), rows_top),
        Vec2::new(rect.width(), rows.outer_size().y),
    );
    let placed = rows.solve(bounds);
    for (index, (text, size, color)) in lines.into_iter().enumerate() {
        if let Some(slot) = placed.find(index) {
            draw_text(
                painter,
                slot.center(),
                Align2::CENTER_CENTER,
                text,
                FontId::proportional(size),
                color,
                None,
            );
        }
    }
}

pub(super) fn scrims(painter: &egui::Painter, rect: Rect) {
    let reach = (rect.height() * 0.5).min(420.0);
    scrim_edge(painter, rect, reach, true, 255);
    scrim_edge(painter, rect, reach, false, 255);
}

pub(super) fn scrim_edge(painter: &egui::Painter, rect: Rect, reach: f32, top: bool, peak: u8) {
    let stops = [(0.0, 1.0), (0.35, 0.84), (0.7, 0.35), (1.0, 0.0)];
    for pair in stops.windows(2) {
        let ((from, dense), (to, sparse)) = (pair[0], pair[1]);
        let dense = Color32::from_black_alpha((dense * peak as f32) as u8);
        let sparse = Color32::from_black_alpha((sparse * peak as f32) as u8);
        if top {
            let band = Rect::from_min_max(
                Pos2::new(rect.left(), rect.top() + from * reach),
                Pos2::new(rect.right(), rect.top() + to * reach),
            );
            paint_vertical_gradient(painter, band, dense, sparse);
        } else {
            let band = Rect::from_min_max(
                Pos2::new(rect.left(), rect.bottom() - to * reach),
                Pos2::new(rect.right(), rect.bottom() - from * reach),
            );
            paint_vertical_gradient(painter, band, sparse, dense);
        }
    }
}
