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

fn spans(start: f32, length: f32, gap: f32, sizes: &[Size]) -> Vec<(f32, f32)> {
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
    sizes
        .iter()
        .map(|size| {
            if hidden(size) {
                return (cursor, 0.0);
            }
            let extent = match size {
                Size::Fixed(extent) => *extent,
                Size::Fill => free / fills as f32,
            };
            let span = (cursor, extent);
            cursor += extent + gap;
            span
        })
        .collect()
}

pub(crate) fn row_of(rect: Rect, gap: f32, sizes: &[Size]) -> Vec<Rect> {
    spans(rect.left(), rect.width(), gap, sizes)
        .into_iter()
        .map(|(left, width)| {
            Rect::from_min_max(
                Pos2::new(left, rect.top()),
                Pos2::new(left + width, rect.bottom()),
            )
        })
        .collect()
}

pub(crate) fn row<const N: usize>(rect: Rect, gap: f32, sizes: [Size; N]) -> [Rect; N] {
    let slots = row_of(rect, gap, &sizes);
    std::array::from_fn(|index| slots[index])
}

pub(crate) fn stack<const N: usize>(rect: Rect, gap: f32, sizes: [Size; N]) -> [Rect; N] {
    let slots = spans(rect.top(), rect.height(), gap, &sizes);
    std::array::from_fn(|index| {
        let (top, height) = slots[index];
        Rect::from_min_max(
            Pos2::new(rect.left(), top),
            Pos2::new(rect.right(), top + height),
        )
    })
}
