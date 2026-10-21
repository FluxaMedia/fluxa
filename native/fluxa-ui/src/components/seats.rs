use super::*;
use crate::WatchMemberView;

const GAP: f32 = 8.0;
const MAX_SEATS: usize = 6;

pub(crate) fn seat_count(members: usize, capacity: u32) -> usize {
    (capacity as usize).max(members).min(MAX_SEATS)
}

pub(crate) fn seat_row_width(seats: usize, radius: f32) -> f32 {
    seats as f32 * radius * 2.0 + seats.saturating_sub(1) as f32 * GAP
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
    for index in 0..seats {
        let center = left_center + Vec2::new(index as f32 * (radius * 2.0 + GAP) + radius, 0.0);
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
                    egui::Stroke::new(
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
                    egui::Stroke::new(1.2, Color32::from_white_alpha(60)),
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
