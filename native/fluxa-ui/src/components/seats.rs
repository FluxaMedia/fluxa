use super::*;
use crate::WatchMemberView;
use crate::flex;

const MAX_SEATS: usize = 6;

pub(crate) fn seat_count(members: usize, capacity: u32) -> usize {
    (capacity as usize).max(members).min(MAX_SEATS)
}

pub(crate) fn seat_row_width(seats: usize, radius: f32, metrics: UiMetrics) -> f32 {
    seats as f32 * radius * 2.0 + seats.saturating_sub(1) as f32 * metrics.seats_gap
}

pub(crate) fn seat_row(
    painter: &Painter,
    assets: &mut impl HomeAssets,
    members: &[WatchMemberView],
    seats: usize,
    left_center: Pos2,
    radius: f32,
    metrics: UiMetrics,
) {
    let width = seat_row_width(seats, radius, metrics);
    let bounds = Rect::from_min_size(
        Pos2::new(left_center.x, left_center.y - radius),
        Vec2::new(width, radius * 2.0),
    );
    let placed = flex::row(
        (0..seats)
            .map(|index| flex::leaf(Vec2::splat(radius * 2.0)).id(index).shrink(0.0))
            .collect::<Vec<_>>(),
    )
    .gap(metrics.seats_gap)
    .solve(bounds);
    for index in 0..seats {
        let center = placed.find(index).unwrap_or(bounds).center();
        match members.get(index) {
            Some(member) => {
                avatar(
                    painter,
                    assets,
                    center,
                    radius,
                    &member.name,
                    member.avatar.as_deref(),
                    metrics,
                );
                painter.circle_stroke(
                    center,
                    radius + 2.0,
                    crate::gfx::Stroke::new(
                        1.5,
                        Color32::from_white_alpha(if member.host { 200 } else { 70 }),
                    ),
                );
            }
            None => {
                painter.circle_filled(center, radius, Color32::from_white_alpha(14));
                painter.circle_stroke(
                    center,
                    radius - 0.5,
                    crate::gfx::Stroke::new(1.2, Color32::from_white_alpha(60)),
                );
                if let Some(plus) = assets.icon("Plus") {
                    painter.image(
                        plus,
                        Rect::from_center_size(center, Vec2::splat(radius)),
                        full_uv(),
                        Color32::from_white_alpha(90),
                    );
                }
            }
        }
    }
}
