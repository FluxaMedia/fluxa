use super::*;

#[derive(Clone, Copy)]
pub(crate) struct Lattice {
    pub origin: Pos2,
    pub cell: Vec2,
    pub gap: f32,
    pub row_gap: f32,
    pub columns: usize,
}

impl Lattice {}
