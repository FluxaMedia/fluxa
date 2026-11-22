use super::*;
use crate::draw_text;

pub(super) fn draw_seek_preview(
    context: &crate::gfx::Context,
    track: Rect,
    pointer_x: f32,
    time: f64,
    player: &PlayerModel,
) {
    let painter = context.layer_painter(crate::gfx::LayerId::new(
        crate::gfx::Order::Foreground,
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
    let font = FontId::proportional(crate::vars::px("text-sm"));
    draw_text(
        &painter,
        Pos2::new(x, bottom),
        Align2::CENTER_BOTTOM,
        format_time(time),
        font.clone(),
        crate::metrics::theme().text_primary,
        None,
    );
    bottom -= 18.0;
    if let Some(chapter) = chapter {
        draw_text(
            &painter,
            Pos2::new(x, bottom),
            Align2::CENTER_BOTTOM,
            chapter,
            font,
            crate::metrics::theme().text_primary,
            Some(width),
        );
        bottom -= 20.0;
    }
    if let Some(texture) = thumbnail {
        let frame = Rect::from_min_max(
            Pos2::new(x - width * 0.5, bottom - width * 9.0 / 16.0),
            Pos2::new(x + width * 0.5, bottom),
        );
        painter.rect_filled(
            frame.expand(1.0),
            crate::vars::px("player-thumb-radius"),
            Color32::from_black_alpha(crate::vars::byte("player-thumb-backdrop")),
        );
        painter.image(texture, frame, full_uv(), Color32::WHITE);
    }
}
