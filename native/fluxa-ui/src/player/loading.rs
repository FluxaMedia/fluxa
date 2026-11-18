use super::*;
use crate::dp::Dp;
use crate::draw_text;
use crate::flex;

pub(super) fn draw_loading(
    context: &crate::gfx::Context,
    painter: &crate::gfx::Painter,
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
    context: &crate::gfx::Context,
    painter: &crate::gfx::Painter,
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
    context: &crate::gfx::Context,
    painter: &crate::gfx::Painter,
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
    let mut rows_top = center.y + Dp::D20.px();
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
                        let time = context.input(|input| input.time()) as f32;
                        let phase = 0.5 - 0.5 * (time * std::f32::consts::PI / 1.08).cos();
                        let alpha = 0.38 + 0.48 * phase;
                        let scaled = flex::scaled(logo_rect, 0.992 + 0.02 * phase);
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
                    FontId::proportional(Dp::D40.px()),
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
        ])
        .align(crate::flex::Align::Start);
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
            .map(|(index, _)| flex::leaf(Vec2::new(0.0, Dp::D28.px())).id(index)),
    )
    .align(flex::Align::Stretch);
    let bounds = crate::gfx::rect_at(
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

pub(super) fn scrim_edge(
    painter: &crate::gfx::Painter,
    rect: Rect,
    reach: f32,
    top: bool,
    peak: u8,
) {
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
