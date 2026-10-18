use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Space {
    Xs,
    Sm,
    Md,
    Lg,
    Xl,
    Xxl,
}

impl Space {
    pub fn px(self, metrics: &UiMetrics) -> f32 {
        metrics.space[self as usize]
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Axis {
    Row,
    Column,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Align {
    Start,
    Center,
    End,
    Stretch,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Justify {
    Start,
    Center,
    End,
    SpaceBetween,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Child {
    size: Vec2,
    grow: f32,
    min: f32,
    max: f32,
}

impl Child {
    pub fn new(size: Vec2) -> Self {
        Self {
            size,
            grow: 0.0,
            min: 0.0,
            max: f32::INFINITY,
        }
    }

    pub fn fill() -> Self {
        Self::new(Vec2::ZERO).flex(1.0)
    }

    pub fn flex(mut self, grow: f32) -> Self {
        self.grow = grow;
        self
    }

    pub fn min(mut self, min: f32) -> Self {
        self.min = min;
        self
    }

    pub fn max(mut self, max: f32) -> Self {
        self.max = max;
        self
    }
}

pub(crate) struct Flex {
    axis: Axis,
    gap: f32,
    padding: f32,
    align: Align,
    justify: Justify,
    children: Vec<Child>,
}

impl Flex {
    pub fn row() -> Self {
        Self::new(Axis::Row)
    }

    pub fn column() -> Self {
        Self::new(Axis::Column)
    }

    fn new(axis: Axis) -> Self {
        Self {
            axis,
            gap: 0.0,
            padding: 0.0,
            align: Align::Start,
            justify: Justify::Start,
            children: Vec::new(),
        }
    }

    pub fn gap(mut self, gap: f32) -> Self {
        self.gap = gap.max(0.0);
        self
    }

    pub fn padding(mut self, padding: f32) -> Self {
        self.padding = padding.max(0.0);
        self
    }

    pub fn align(mut self, align: Align) -> Self {
        self.align = align;
        self
    }

    pub fn justify(mut self, justify: Justify) -> Self {
        self.justify = justify;
        self
    }

    pub fn child(mut self, child: Child) -> Self {
        self.children.push(child);
        self
    }

    pub fn children(mut self, children: impl IntoIterator<Item = Child>) -> Self {
        self.children.extend(children);
        self
    }

    fn main(&self, size: Vec2) -> f32 {
        match self.axis {
            Axis::Row => size.x,
            Axis::Column => size.y,
        }
    }

    fn cross(&self, size: Vec2) -> f32 {
        match self.axis {
            Axis::Row => size.y,
            Axis::Column => size.x,
        }
    }

    pub fn solve(&self, bounds: Rect) -> Vec<Rect> {
        let count = self.children.len();
        if count == 0 {
            return Vec::new();
        }
        let inner = bounds.shrink(self.padding);
        let main_avail = self.main(inner.size()).max(0.0);
        let cross_avail = self.cross(inner.size()).max(0.0);
        let gaps = self.gap * (count - 1) as f32;

        let mut lengths: Vec<f32> = self
            .children
            .iter()
            .map(|child| self.main(child.size).clamp(child.min, child.max))
            .collect();
        let mut free = main_avail - gaps - lengths.iter().sum::<f32>();

        if free > 0.0 {
            let mut open: Vec<usize> = (0..count)
                .filter(|&i| self.children[i].grow > 0.0)
                .collect();
            while free > 0.01 && !open.is_empty() {
                let total: f32 = open.iter().map(|&i| self.children[i].grow).sum();
                let share = free;
                let mut spent = 0.0;
                let mut saturated = Vec::new();
                for &i in &open {
                    let child = &self.children[i];
                    let want = share * child.grow / total;
                    let next = (lengths[i] + want).min(child.max);
                    spent += next - lengths[i];
                    lengths[i] = next;
                    if next >= child.max {
                        saturated.push(i);
                    }
                }
                free -= spent;
                open.retain(|i| !saturated.contains(i));
                if saturated.is_empty() {
                    break;
                }
            }
        } else if free < 0.0 {
            let mut deficit = -free;
            for _ in 0..count {
                let shrinkable: f32 = (0..count)
                    .map(|i| (lengths[i] - self.children[i].min).max(0.0))
                    .sum();
                if shrinkable <= 0.0 || deficit <= 0.01 {
                    break;
                }
                let cut = deficit.min(shrinkable);
                for i in 0..count {
                    let room = (lengths[i] - self.children[i].min).max(0.0);
                    lengths[i] -= cut * room / shrinkable;
                }
                deficit -= cut;
            }
            free = -deficit;
        }

        let free = free.max(0.0);
        let (mut cursor, gap) = match self.justify {
            Justify::Start => (0.0, self.gap),
            Justify::Center => (free * 0.5, self.gap),
            Justify::End => (free, self.gap),
            Justify::SpaceBetween if count > 1 => (0.0, self.gap + free / (count - 1) as f32),
            Justify::SpaceBetween => (0.0, self.gap),
        };

        let mut rects = Vec::with_capacity(count);
        for (child, length) in self.children.iter().zip(lengths) {
            let wanted = self.cross(child.size);
            let cross = match self.align {
                Align::Stretch => cross_avail,
                _ => wanted.min(cross_avail),
            };
            let offset = match self.align {
                Align::Start | Align::Stretch => 0.0,
                Align::Center => (cross_avail - cross) * 0.5,
                Align::End => cross_avail - cross,
            };
            let (min, size) = match self.axis {
                Axis::Row => (
                    Pos2::new(inner.left() + cursor, inner.top() + offset),
                    Vec2::new(length, cross),
                ),
                Axis::Column => (
                    Pos2::new(inner.left() + offset, inner.top() + cursor),
                    Vec2::new(cross, length),
                ),
            };
            rects.push(Rect::from_min_size(min, size));
            cursor += length + gap;
        }
        rects
    }
}

pub(crate) fn stack(bounds: Rect, size: Vec2, align: Align2) -> Rect {
    align.align_size_within_rect(size.min(bounds.size()), bounds)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bounds(width: f32, height: f32) -> Rect {
        Rect::from_min_size(Pos2::ZERO, Vec2::new(width, height))
    }

    #[test]
    fn flex_children_split_leftover_by_weight() {
        let rects = Flex::row()
            .gap(10.0)
            .child(Child::new(Vec2::new(50.0, 20.0)))
            .child(Child::fill())
            .child(Child::fill().flex(2.0))
            .solve(bounds(260.0, 20.0));
        assert!((rects[1].width() - 190.0 / 3.0).abs() < 0.01);
        assert!((rects[2].width() - 380.0 / 3.0).abs() < 0.01);
        assert_eq!(rects[2].right(), 260.0);
    }

    #[test]
    fn max_width_hands_the_rest_to_siblings() {
        let rects = Flex::row()
            .child(Child::fill().max(40.0))
            .child(Child::fill())
            .solve(bounds(200.0, 10.0));
        assert_eq!(rects[0].width(), 40.0);
        assert_eq!(rects[1].width(), 160.0);
    }

    #[test]
    fn overflow_shrinks_down_to_min() {
        let rects = Flex::row()
            .child(Child::new(Vec2::new(100.0, 10.0)).min(95.0))
            .child(Child::new(Vec2::new(100.0, 10.0)))
            .solve(bounds(120.0, 10.0));
        assert!(rects[0].width() >= 95.0);
        assert!((rects[0].width() + rects[1].width() - 120.0).abs() < 0.01);
    }

    #[test]
    fn centered_column_with_cross_stretch() {
        let rects = Flex::column()
            .gap(8.0)
            .align(Align::Stretch)
            .justify(Justify::Center)
            .child(Child::new(Vec2::new(10.0, 30.0)))
            .child(Child::new(Vec2::new(10.0, 30.0)))
            .solve(bounds(100.0, 100.0));
        assert_eq!(rects[0].top(), 16.0);
        assert_eq!(rects[0].width(), 100.0);
        assert_eq!(rects[1].bottom(), 84.0);
    }

    #[test]
    fn space_between_pins_first_and_last() {
        let rects = Flex::row()
            .justify(Justify::SpaceBetween)
            .align(Align::Center)
            .children((0..3).map(|_| Child::new(Vec2::splat(20.0))))
            .solve(bounds(200.0, 40.0));
        assert_eq!(rects[0].left(), 0.0);
        assert_eq!(rects[2].right(), 200.0);
        assert_eq!(rects[1].top(), 10.0);
    }

    #[test]
    fn padding_insets_children() {
        let rects = Flex::row()
            .padding(12.0)
            .align(Align::Stretch)
            .child(Child::fill())
            .solve(bounds(100.0, 50.0));
        assert_eq!(
            rects[0],
            Rect::from_min_max(Pos2::new(12.0, 12.0), Pos2::new(88.0, 38.0))
        );
    }
}
