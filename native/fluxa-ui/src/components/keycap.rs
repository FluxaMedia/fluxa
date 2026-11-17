use crate::gfx::{Align2, Color32, FontId, Painter, Pos2, Rect, Stroke, StrokeKind, Vec2};

use crate::draw_text;


#[derive(Clone, Copy, PartialEq)]
enum Pad {
    A,
    B,
    X,
    Y,
    DPad(Option<u8>),
    Stick,
}







