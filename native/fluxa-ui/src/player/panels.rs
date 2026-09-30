use super::*;

pub(super) fn draw_mini_player(
    ui: &mut egui::Ui,
    painter: &egui::Painter,
    rect: Rect,
    compact: bool,
    player: &PlayerModel,
    layout: &mut HomeLayout,
) {
    let width = if compact { rect.width() * 0.46 } else { 448.0 };
    let mini = Rect::from_min_size(
        rect.min + Vec2::splat(if compact { 16.0 } else { 24.0 }),
        Vec2::new(width, width * 9.0 / 16.0),
    );
    painter.rect_filled(mini.expand(1.0), 8.0, Color32::from_white_alpha(28));
    match player.video {
        Some(texture) => {
            painter.add(
                egui::epaint::RectShape::filled(mini, 7.0, Color32::WHITE)
                    .with_texture(texture, full_uv()),
            );
        }
        None => {
            painter.rect_filled(mini, 7.0, Color32::BLACK);
        }
    }
    let hovered = ui.rect_contains_pointer(mini);
    let shown =
        ui.ctx()
            .animate_bool_with_time(Id::new("fluxa-player-mini-controls"), hovered, 0.2);
    if shown <= 0.0 {
        let hit = ui.interact(mini, Id::new("fluxa-player-mini"), Sense::click());
        if hit.clicked() {
            layout.activated = Some(NODE_PLAYER_RECOMMENDATIONS_CLOSE);
        }
        return;
    }
    let strip = Rect::from_min_max(Pos2::new(mini.left(), mini.bottom() - 46.0), mini.max);
    paint_vertical_gradient(
        painter,
        strip,
        Color32::TRANSPARENT,
        Color32::from_black_alpha((215.0 * shown) as u8),
    );
    let y = strip.center().y + 4.0;
    for (offset, icon, node) in [
        (
            -40.0,
            if player.paused { "play" } else { "pause" },
            NODE_PLAYER_TOGGLE,
        ),
        (0.0, "fullscreen", NODE_PLAYER_RECOMMENDATIONS_CLOSE),
        (40.0, "close", NODE_PLAYER_CLOSE),
    ] {
        let button = control(
            ui,
            painter,
            Rect::from_center_size(Pos2::new(mini.center().x + offset, y), Vec2::splat(32.0)),
            icon,
            false,
        );
        layout.focusable.push((node, button.rect));
        if button.clicked() {
            layout.activated = Some(node);
        }
    }
}

pub(super) fn draw_seek_preview(
    context: &egui::Context,
    track: Rect,
    pointer_x: f32,
    time: f64,
    player: &PlayerModel,
    metrics: UiMetrics,
) {
    let painter = context.layer_painter(egui::LayerId::new(
        egui::Order::Foreground,
        Id::new("fluxa-player-seek-preview"),
    ));
    let chapter = player.chapter_at(time);
    let thumbnail = player
        .thumbnail
        .filter(|(shown, _)| (shown - time).abs() <= 30.0)
        .map(|(_, texture)| texture);
    let width = if thumbnail.is_some() { 240.0 } else { 160.0 };
    let x = pointer_x.clamp(track.left() + width * 0.5, track.right() - width * 0.5);
    let mut bottom = track.top() - 10.0;
    let label_font = FontId::proportional(13.0);
    painter.text(
        Pos2::new(x, bottom),
        Align2::CENTER_BOTTOM,
        format_time(time),
        label_font.clone(),
        Color32::WHITE,
    );
    bottom -= 18.0;
    if let Some(chapter) = chapter {
        painter.text(
            Pos2::new(x, bottom),
            Align2::CENTER_BOTTOM,
            truncate_to_width(&painter, chapter, &label_font, width),
            label_font,
            metrics.text_primary,
        );
        bottom -= 20.0;
    }
    if let Some(texture) = thumbnail {
        let frame = Rect::from_min_max(
            Pos2::new(x - width * 0.5, bottom - width * 9.0 / 16.0),
            Pos2::new(x + width * 0.5, bottom),
        );
        painter.rect_filled(frame.expand(1.0), 4.0, Color32::from_white_alpha(40));
        painter.image(texture, frame, full_uv(), Color32::WHITE);
    }
}

pub(super) fn draw_pause_info(
    context: &egui::Context,
    painter: &egui::Painter,
    rect: Rect,
    player: &PlayerModel,
    assets: &mut impl HomeAssets,
) {
    let band = rect.width() / 24.0;
    for index in 0..24 {
        let fade = 1.0 - index as f32 / 24.0;
        let left = rect.left() + index as f32 * band;
        painter.rect_filled(
            Rect::from_min_max(
                Pos2::new(left, rect.top()),
                Pos2::new(left + band, rect.bottom()),
            ),
            0.0,
            Color32::from_black_alpha((200.0 * fade.powf(1.4)).round() as u8),
        );
    }
    let left = rect.left() + 64.0;
    let width = (rect.width() * 0.42).clamp(260.0, 620.0);
    let mut blocks: Vec<(f32, Box<dyn Fn(&egui::Painter, f32)>)> = Vec::new();
    let muted = Color32::from_white_alpha(190);

    let label = localized("player.youre_watching", &player.language);
    blocks.push((
        28.0,
        Box::new(move |painter, y| {
            painter.text(
                Pos2::new(left, y),
                Align2::LEFT_TOP,
                &label,
                FontId::proportional(16.0),
                muted,
            );
        }),
    ));
    let max_logo = Vec2::new(width.min(420.0), 96.0);
    let logo = components::title_logo(
        context.pixels_per_point(),
        player.logo.as_deref(),
        max_logo,
        ArtworkPriority::Hero,
        assets,
    );
    match logo {
        Some((texture, size)) => blocks.push((
            size.y + 16.0,
            Box::new(move |painter, y| {
                painter.image(
                    texture,
                    Rect::from_min_size(Pos2::new(left, y), size),
                    full_uv(),
                    Color32::WHITE,
                );
            }),
        )),
        None => {
            let galley =
                components::wrapped_text(painter, &player.title, 40.0, Color32::WHITE, width, 2);
            blocks.push((
                galley.size().y + 12.0,
                Box::new(move |painter, y| {
                    painter.galley(Pos2::new(left, y), galley.clone(), Color32::WHITE)
                }),
            ));
        }
    }
    let chapter = player
        .chapter_at(player.position)
        .map(|title| format!("{}: {title}", localized("player.chapter", &player.language)));
    for (text, size) in [
        (player.episode_title.clone(), 20.0),
        (chapter, 15.0),
        (player.description.clone(), 16.0),
    ] {
        let Some(text) = text else {
            continue;
        };
        let rows = if size == 16.0 { 5 } else { 1 };
        let galley = components::wrapped_text(painter, &text, size, muted, width, rows);
        blocks.push((
            galley.size().y + 10.0,
            Box::new(move |painter, y| painter.galley(Pos2::new(left, y), galley.clone(), muted)),
        ));
    }
    let height: f32 = blocks.iter().map(|(height, _)| height).sum();
    let mut y = rect.bottom() - 120.0 - height;
    for (height, draw) in blocks {
        draw(painter, y);
        y += height;
    }
}
