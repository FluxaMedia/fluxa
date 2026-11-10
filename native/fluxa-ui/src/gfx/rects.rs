use super::{Pos2, Rect, Vec2};

pub fn rect_at(min: Pos2, size: Vec2) -> Rect {
    Rect::from_min_size(min, size)
}

pub fn rect_around(center: Pos2, size: Vec2) -> Rect {
    Rect::from_center_size(center, size)
}

pub fn square_around(center: Pos2, side: f32) -> Rect {
    Rect::from_center_size(center, Vec2::splat(side))
}
