use egui::{FontId, Painter, Pos2, Rect, Vec2};

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

pub(crate) fn text(painter: &Painter, text: &str, font: FontId) -> Size {
    Size::Fixed(crate::paint::text_width(painter, text, font))
}

#[derive(Clone, Copy)]
pub(crate) enum Align {
    Start,
    Center,
    End,
}

fn aligned(start: f32, room: f32, extent: f32, align: Align) -> f32 {
    match align {
        Align::Start => start,
        Align::Center => start + (room - extent) * 0.5,
        Align::End => start + room - extent,
    }
}

pub(crate) fn place(slot: Rect, size: Vec2, x: Align, y: Align) -> Rect {
    Rect::from_min_size(
        Pos2::new(
            aligned(slot.left(), slot.width(), size.x, x),
            aligned(slot.top(), slot.height(), size.y, y),
        ),
        size,
    )
}

pub(crate) fn line(rect: Rect, gap: f32, widths: &[f32], align: Align) -> Vec<Rect> {
    let total = widths.iter().sum::<f32>() + gap * widths.len().saturating_sub(1) as f32;
    let mut left = aligned(rect.left(), rect.width(), total, align);
    widths
        .iter()
        .map(|width| {
            let slot = Rect::from_min_max(
                Pos2::new(left, rect.top()),
                Pos2::new(left + width, rect.bottom()),
            );
            left += width + gap;
            slot
        })
        .collect()
}

pub(crate) fn wrap(rect: Rect, gap: Vec2, sizes: &[Vec2]) -> Vec<Rect> {
    let mut cursor = rect.min;
    let mut line = 0.0_f32;
    sizes
        .iter()
        .map(|size| {
            if cursor.x > rect.left() && cursor.x + size.x > rect.right() {
                cursor = Pos2::new(rect.left(), cursor.y + line + gap.y);
                line = 0.0;
            }
            let slot = Rect::from_min_size(cursor, *size);
            cursor.x += size.x + gap.x;
            line = line.max(size.y);
            slot
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn area() -> Rect {
        Rect::from_min_size(Pos2::ZERO, Vec2::new(100.0, 40.0))
    }

    #[test]
    fn empty_child_gives_its_gap_to_the_fill() {
        let [fill, hidden, last] = row(
            area(),
            10.0,
            [Size::Fill, Size::Fixed(0.0), Size::Fixed(20.0)],
        );
        assert_eq!(fill.width(), 70.0);
        assert_eq!(hidden.width(), 0.0);
        assert_eq!(last.left(), 80.0);
    }

    #[test]
    fn wrap_starts_a_new_line_below_the_tallest_item() {
        let item = |height| Vec2::new(45.0, height);
        let slots = wrap(
            area(),
            Vec2::new(10.0, 4.0),
            &[item(10.0), item(16.0), item(10.0)],
        );
        assert_eq!(slots[1].left(), 55.0);
        assert_eq!(slots[2].min, Pos2::new(0.0, 20.0));
    }
}
