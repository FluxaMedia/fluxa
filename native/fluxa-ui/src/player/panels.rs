use super::*;
use crate::draw_text;



pub(super) fn draw_pause_info(
    context: &crate::gfx::Context,
    painter: &crate::gfx::Painter,
    rect: Rect,
    player: &PlayerModel,
    assets: &mut impl HomeAssets,
) {
    let metrics = crate::metrics::active();
    paint_horizontal_gradient(
        painter,
        rect,
        Color32::from_black_alpha(200),
        Color32::TRANSPARENT,
    );
    scrim_edge(painter, rect, (rect.height() * 0.8).min(640.0), false, 255);
    let compact = rect.width() < 900.0 || rect.height() < 520.0;
    let info = if compact {
        metrics.player_info_compact
    } else {
        metrics.player_info
    };
    let left = rect.left() + info.left;
    let width = info.width.of(rect.width());
    let mut blocks: Vec<(f32, Box<dyn Fn(&crate::gfx::Painter, f32)>)> = Vec::new();
    let muted = crate::stylesheet::color("text.body");

    let label = localized("player.youre_watching", &player.language);
    blocks.push((
        info.label_gap,
        Box::new(move |painter, y| {
            draw_text(
                painter,
                Pos2::new(left, y),
                Align2::LEFT_TOP,
                &label,
                FontId::proportional(info.label_font),
                muted,
                None,
            );
        }),
    ));
    let max_logo = Vec2::new(width.min(info.logo_width), info.logo_height);
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
                    crate::gfx::rect_at(Pos2::new(left, y), size),
                    full_uv(),
                    Color32::WHITE,
                );
            }),
        )),
        None => {
            let galley = components::wrapped_text(
                painter,
                &player.title,
                info.title_font,
                text_primary(),
                width,
                2,
            );
            blocks.push((
                galley.size().y + 12.0,
                Box::new(move |painter, y| {
                    painter.galley(Pos2::new(left, y), galley.clone(), text_primary())
                }),
            ));
        }
    }
    let chapter = player
        .chapter_at(player.position)
        .map(|title| format!("{}: {title}", localized("player.chapter", &player.language)));
    let scale = info.scale;
    let description_size = 16.0 * scale;
    for (text, size) in [
        (player.episode_title.clone(), 20.0 * scale),
        (chapter, 15.0 * scale),
        (player.description.clone(), description_size),
    ] {
        let Some(text) = text else {
            continue;
        };
        let rows = if size == description_size {
            info.description_rows
        } else {
            1
        };
        let galley = components::wrapped_text(painter, &text, size, muted, width, rows);
        blocks.push((
            galley.size().y + 10.0,
            Box::new(move |painter, y| painter.galley(Pos2::new(left, y), galley.clone(), muted)),
        ));
    }
    let height: f32 = blocks.iter().map(|(height, _)| height).sum();
    let mut y = rect.bottom() - info.bottom - height;
    for (height, draw) in blocks {
        draw(painter, y);
        y += height;
    }
}
