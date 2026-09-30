use super::*;

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
    let painter = context.layer_painter(egui::LayerId::new(
        egui::Order::Foreground,
        Id::new("fluxa-player-warnings"),
    ));
    let total = content_warning_duration(rows);
    let ramp = |value: f32| value.clamp(0.0, 1.0);
    let box_alpha = ramp(elapsed / WARNING_BAR).min(ramp((total - elapsed) / WARNING_BAR));
    let font = FontId::proportional(14.0);
    let row_height = 20.0;
    let lines = player
        .warnings
        .iter()
        .map(|(label, severity)| format!("{label} · {severity}"))
        .collect::<Vec<_>>();
    let text_width = lines
        .iter()
        .map(|line| {
            painter
                .layout_no_wrap(line.clone(), font.clone(), Color32::WHITE)
                .size()
                .x
        })
        .fold(0.0, f32::max);
    let panel = Rect::from_min_size(
        origin,
        Vec2::new(text_width + 34.0, rows as f32 * row_height + 20.0),
    );
    painter.rect_filled(
        panel,
        6.0,
        Color32::from_black_alpha((173.0 * box_alpha) as u8),
    );
    let bar = panel.height() - 20.0;
    painter.rect_filled(
        Rect::from_min_size(
            panel.min + Vec2::new(12.0, 10.0),
            Vec2::new(3.0, bar * ramp(elapsed / WARNING_BAR)),
        ),
        1.5,
        Color32::from_white_alpha((230.0 * box_alpha) as u8),
    );
    let fade_out_start = total - WARNING_BAR - rows as f32 * WARNING_STAGGER - WARNING_FADE;
    for (index, line) in lines.into_iter().enumerate() {
        let appear = ramp((elapsed - WARNING_BAR - index as f32 * WARNING_STAGGER) / WARNING_FADE);
        let leave = fade_out_start + (rows - 1 - index) as f32 * WARNING_STAGGER;
        let alpha = appear.min(1.0 - ramp((elapsed - leave) / WARNING_FADE));
        painter.text(
            panel.min + Vec2::new(24.0, 10.0 + index as f32 * row_height + row_height * 0.5),
            Align2::LEFT_CENTER,
            line,
            font.clone(),
            Color32::from_white_alpha((255.0 * alpha) as u8),
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
        Color32::from_white_alpha(89),
        assets,
    );
    paint_vertical_gradient(
        painter,
        rect,
        Color32::from_black_alpha(26),
        Color32::from_black_alpha(166),
    );

    let center = rect.center();
    let logo_box = Vec2::new(480.0_f32.min(rect.width() - 32.0), 160.0);
    let logo = player.logo.as_deref();
    let logo_texture = components::title_logo(
        context.pixels_per_point(),
        logo,
        logo_box,
        ArtworkPriority::Visible,
        assets,
    );
    let mut below = center.y + 36.0;
    if player.error.is_none() {
        match logo_texture {
            Some((texture, fitted)) => {
                let logo_rect = Rect::from_center_size(center, fitted);
                match player.load_progress {
                    Some(progress) => {
                        let shown = context.animate_value_with_time(
                            Id::new("fluxa-player-load-progress"),
                            progress,
                            0.42,
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
                below = logo_rect.bottom() + 28.0;
            }
            None => {
                painter.text(
                    center,
                    Align2::CENTER_CENTER,
                    &player.title,
                    FontId::proportional(40.0),
                    Color32::WHITE,
                );
            }
        }
    }

    if let Some(error) = player.error.as_deref() {
        painter.text(
            center - Vec2::new(0.0, 16.0),
            Align2::CENTER_CENTER,
            localized("player.playback_error", &player.language),
            FontId::proportional(22.0),
            Color32::WHITE,
        );
        painter.text(
            center + Vec2::new(0.0, 16.0),
            Align2::CENTER_CENTER,
            truncate_to_width(
                painter,
                error,
                &FontId::proportional(14.0),
                rect.width() - 64.0,
            ),
            FontId::proportional(14.0),
            metrics.text_secondary,
        );
        return;
    }
    if let Some(episode) = player.episode_title.as_deref() {
        painter.text(
            Pos2::new(center.x, below),
            Align2::CENTER_CENTER,
            episode,
            FontId::proportional(16.0),
            metrics.text_primary,
        );
        below += 28.0;
    }
    let status = player
        .status
        .as_ref()
        .map(|(headline, _)| headline.clone())
        .unwrap_or_else(|| localized("player.status_starting_playback", &player.language));
    painter.text(
        Pos2::new(center.x, below),
        Align2::CENTER_CENTER,
        status,
        FontId::proportional(14.0),
        metrics.text_secondary,
    );
}

pub(super) fn scrims(painter: &egui::Painter, rect: Rect) {
    let band = 18.0;
    for index in 0..16 {
        let fade = 1.0 - index as f32 / 16.0;
        let top = rect.top() + index as f32 * band;
        painter.rect_filled(
            Rect::from_min_max(
                Pos2::new(rect.left(), top),
                Pos2::new(rect.right(), (top + band).min(rect.bottom())),
            ),
            0.0,
            Color32::from_black_alpha((152.0 * fade.powf(1.7)).round() as u8),
        );
        let bottom = rect.bottom() - index as f32 * band;
        painter.rect_filled(
            Rect::from_min_max(
                Pos2::new(rect.left(), (bottom - band).max(rect.top())),
                Pos2::new(rect.right(), bottom),
            ),
            0.0,
            Color32::from_black_alpha((218.0 * fade.powf(1.45)).round() as u8),
        );
    }
}
