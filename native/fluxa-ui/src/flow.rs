use egui::{Pos2, Rect};

pub(crate) struct Column {
    cursor: f32,
}

impl Column {
    pub(crate) fn from(top: f32) -> Self {
        Self { cursor: top }
    }

    pub(crate) fn place(&mut self, height: f32) -> f32 {
        let top = self.cursor;
        self.cursor += height;
        top
    }

    pub(crate) fn end(&self) -> f32 {
        self.cursor
    }
}

#[derive(Clone, Copy)]
pub(crate) enum Size {
    Fixed(f32),
    Fill,
}

fn spans<const N: usize>(start: f32, length: f32, gap: f32, sizes: [Size; N]) -> [(f32, f32); N] {
    let hidden = |size: &Size| matches!(size, Size::Fixed(extent) if *extent <= 0.0);
    let shown = sizes.iter().filter(|size| !hidden(size)).count();
    let fills = sizes
        .iter()
        .filter(|size| matches!(size, Size::Fill))
        .count();
    let fixed: f32 = sizes
        .iter()
        .map(|size| match size {
            Size::Fixed(extent) => extent.max(0.0),
            Size::Fill => 0.0,
        })
        .sum();
    let free = (length - fixed - gap * shown.saturating_sub(1) as f32).max(0.0);
    let mut cursor = start;
    sizes.map(|size| {
        if hidden(&size) {
            return (cursor, 0.0);
        }
        let extent = match size {
            Size::Fixed(extent) => extent,
            Size::Fill => free / fills as f32,
        };
        let span = (cursor, extent);
        cursor += extent + gap;
        span
    })
}

pub(crate) fn row<const N: usize>(rect: Rect, gap: f32, sizes: [Size; N]) -> [Rect; N] {
    spans(rect.left(), rect.width(), gap, sizes).map(|(left, width)| {
        Rect::from_min_max(
            Pos2::new(left, rect.top()),
            Pos2::new(left + width, rect.bottom()),
        )
    })
}

pub(crate) fn stack<const N: usize>(rect: Rect, gap: f32, sizes: [Size; N]) -> [Rect; N] {
    spans(rect.top(), rect.height(), gap, sizes).map(|(top, height)| {
        Rect::from_min_max(
            Pos2::new(rect.left(), top),
            Pos2::new(rect.right(), top + height),
        )
    })
}
