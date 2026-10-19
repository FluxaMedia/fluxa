#![allow(dead_code)]

use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Space {
    S2,
    S4,
    S6,
    S8,
    S10,
    S12,
    S14,
    S16,
    S20,
    S24,
    S32,
    S48,
}

impl Space {
    pub fn px(self, metrics: &UiMetrics) -> f32 {
        metrics.space[self as usize]
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Radius {
    Xs,
    Sm,
    Md,
    Lg,
    Sheet,
}

impl Radius {
    pub fn px(self, metrics: &UiMetrics) -> f32 {
        metrics.radius[self as usize]
    }

    pub fn corner(self, metrics: &UiMetrics) -> u8 {
        self.px(metrics).round().clamp(0.0, 255.0) as u8
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum IconSize {
    Sm,
    Md,
    Lg,
    Xl,
}

impl IconSize {
    pub fn px(self, metrics: &UiMetrics) -> f32 {
        metrics.icon[self as usize]
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Weight {
    Regular,
    Medium,
    Bold,
}

impl Weight {
    pub fn font(self, size: f32) -> FontId {
        match self {
            Weight::Regular => crate::fonts::regular(size),
            Weight::Medium => crate::fonts::medium(size),
            Weight::Bold => crate::fonts::bold(size),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct Insets {
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
}

impl Insets {
    pub fn all(value: f32) -> Self {
        Self::xy(value, value)
    }

    pub fn xy(x: f32, y: f32) -> Self {
        Self {
            left: x,
            top: y,
            right: x,
            bottom: y,
        }
    }

    pub fn top(mut self, value: f32) -> Self {
        self.top = value;
        self
    }

    pub fn bottom(mut self, value: f32) -> Self {
        self.bottom = value;
        self
    }

    pub fn left(mut self, value: f32) -> Self {
        self.left = value;
        self
    }

    pub fn right(mut self, value: f32) -> Self {
        self.right = value;
        self
    }

    fn width(self) -> f32 {
        self.left + self.right
    }

    fn height(self) -> f32 {
        self.top + self.bottom
    }

    fn shrink(self, rect: Rect) -> Rect {
        let min = Pos2::new(rect.left() + self.left, rect.top() + self.top);
        let max = Pos2::new(
            (rect.right() - self.right).max(min.x),
            (rect.bottom() - self.bottom).max(min.y),
        );
        Rect::from_min_max(min, max)
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
    SpaceAround,
}

#[derive(Clone, Debug)]
enum Kind {
    Leaf(Vec2),
    Flex(Axis),
    Stack(Align2),
}

#[derive(Clone, Debug)]
pub(crate) struct Node {
    id: Option<usize>,
    kind: Kind,
    width: Option<f32>,
    height: Option<f32>,
    min: Vec2,
    max: Vec2,
    grow: f32,
    margin: Insets,
    padding: Insets,
    gap: f32,
    line_gap: f32,
    align: Align,
    justify: Justify,
    wrap: bool,
    children: Vec<Node>,
}

pub(crate) fn row(children: impl IntoIterator<Item = Node>) -> Node {
    Node::container(Kind::Flex(Axis::Row), children)
}

pub(crate) fn column(children: impl IntoIterator<Item = Node>) -> Node {
    Node::container(Kind::Flex(Axis::Column), children)
}

pub(crate) fn stack(align: Align2, children: impl IntoIterator<Item = Node>) -> Node {
    Node::container(Kind::Stack(align), children)
}

pub(crate) fn leaf(size: Vec2) -> Node {
    Node::new(Kind::Leaf(size))
}

pub(crate) fn slot(width: f32) -> Node {
    leaf(Vec2::ZERO).width(width)
}

pub(crate) fn fill() -> Node {
    leaf(Vec2::ZERO).grow(1.0)
}

pub(crate) fn text_size(painter: &egui::Painter, text: &str, font: &FontId) -> Vec2 {
    painter
        .layout_no_wrap(text.to_owned(), font.clone(), Color32::WHITE)
        .size()
}

pub(crate) fn text(painter: &egui::Painter, text: &str, font: &FontId) -> Node {
    leaf(text_size(painter, text, font))
}

#[derive(Clone, Debug)]
pub(crate) struct Placed {
    pub id: Option<usize>,
    pub rect: Rect,
    pub children: Vec<Placed>,
}

impl Placed {
    pub fn find(&self, id: usize) -> Option<Rect> {
        if self.id == Some(id) {
            return Some(self.rect);
        }
        self.children.iter().find_map(|child| child.find(id))
    }

    pub fn bottom(&self) -> f32 {
        self.children
            .iter()
            .map(|child| child.rect.bottom().max(child.bottom()))
            .fold(self.rect.top(), f32::max)
    }

    pub fn tagged(&self) -> Vec<(usize, Rect)> {
        let mut out = Vec::new();
        self.collect(&mut out);
        out.sort_by_key(|(id, _)| *id);
        out
    }

    fn collect(&self, out: &mut Vec<(usize, Rect)>) {
        if let Some(id) = self.id {
            out.push((id, self.rect));
        }
        for child in &self.children {
            child.collect(out);
        }
    }
}

fn main(axis: Axis, size: Vec2) -> f32 {
    match axis {
        Axis::Row => size.x,
        Axis::Column => size.y,
    }
}

fn cross(axis: Axis, size: Vec2) -> f32 {
    match axis {
        Axis::Row => size.y,
        Axis::Column => size.x,
    }
}

fn vec(axis: Axis, main: f32, cross: f32) -> Vec2 {
    match axis {
        Axis::Row => Vec2::new(main, cross),
        Axis::Column => Vec2::new(cross, main),
    }
}

impl Node {
    fn new(kind: Kind) -> Self {
        Self {
            id: None,
            kind,
            width: None,
            height: None,
            min: Vec2::ZERO,
            max: Vec2::splat(f32::INFINITY),
            grow: 0.0,
            margin: Insets::default(),
            padding: Insets::default(),
            gap: 0.0,
            line_gap: 0.0,
            align: Align::Start,
            justify: Justify::Start,
            wrap: false,
            children: Vec::new(),
        }
    }

    fn container(kind: Kind, children: impl IntoIterator<Item = Node>) -> Self {
        let mut node = Self::new(kind);
        node.children = children.into_iter().collect();
        node
    }

    pub fn id(mut self, id: usize) -> Self {
        self.id = Some(id);
        self
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    pub fn height(mut self, height: f32) -> Self {
        self.height = Some(height);
        self
    }

    pub fn size(self, size: Vec2) -> Self {
        self.width(size.x).height(size.y)
    }

    pub fn min_width(mut self, value: f32) -> Self {
        self.min.x = value;
        self
    }

    pub fn max_width(mut self, value: f32) -> Self {
        self.max.x = value;
        self
    }

    pub fn min_height(mut self, value: f32) -> Self {
        self.min.y = value;
        self
    }

    pub fn max_height(mut self, value: f32) -> Self {
        self.max.y = value;
        self
    }

    pub fn grow(mut self, grow: f32) -> Self {
        self.grow = grow;
        self
    }

    pub fn margin(mut self, margin: Insets) -> Self {
        self.margin = margin;
        self
    }

    pub fn padding(mut self, padding: Insets) -> Self {
        self.padding = padding;
        self
    }

    pub fn gap(mut self, gap: f32) -> Self {
        self.gap = gap.max(0.0);
        self.line_gap = self.gap;
        self
    }

    pub fn line_gap(mut self, gap: f32) -> Self {
        self.line_gap = gap.max(0.0);
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

    pub fn wrap(mut self) -> Self {
        self.wrap = true;
        self
    }

    fn content_size(&self) -> Vec2 {
        match &self.kind {
            Kind::Leaf(size) => *size,
            Kind::Flex(axis) => {
                let sizes: Vec<Vec2> = self.children.iter().map(Node::outer_size).collect();
                let gaps = self.gap * sizes.len().saturating_sub(1) as f32;
                let main_total = sizes.iter().map(|size| main(*axis, *size)).sum::<f32>() + gaps;
                let cross_max = sizes
                    .iter()
                    .map(|size| cross(*axis, *size))
                    .fold(0.0, f32::max);
                vec(*axis, main_total, cross_max)
            }
            Kind::Stack(_) => self
                .children
                .iter()
                .map(Node::outer_size)
                .fold(Vec2::ZERO, |acc, size| acc.max(size)),
        }
    }

    fn border_size(&self) -> Vec2 {
        let content = self.content_size();
        let padded = Vec2::new(
            content.x + self.padding.width(),
            content.y + self.padding.height(),
        );
        Vec2::new(
            self.width.unwrap_or(padded.x).clamp(self.min.x, self.max.x),
            self.height
                .unwrap_or(padded.y)
                .clamp(self.min.y, self.max.y),
        )
    }

    pub fn outer_size(&self) -> Vec2 {
        self.border_size() + Vec2::new(self.margin.width(), self.margin.height())
    }

    fn span(&self, axis: Axis) -> (f32, f32, f32) {
        let extra = match axis {
            Axis::Row => self.margin.width(),
            Axis::Column => self.margin.height(),
        };
        let base = main(axis, self.outer_size());
        let min = match self.fixed(axis) {
            Some(_) => base,
            None => main(axis, self.min) + extra,
        };
        let max = match self.fixed(axis) {
            Some(_) => base,
            None => main(axis, self.max) + extra,
        };
        (base, min.min(base), max.max(base).max(min))
    }

    fn fixed(&self, axis: Axis) -> Option<f32> {
        match axis {
            Axis::Row => self.width,
            Axis::Column => self.height,
        }
    }

    pub fn solve(&self, bounds: Rect) -> Placed {
        let rect = self.margin.shrink(bounds);
        let inner = self.padding.shrink(rect);
        let children = match &self.kind {
            Kind::Leaf(_) => Vec::new(),
            Kind::Flex(axis) => self.solve_flex(*axis, inner),
            Kind::Stack(align) => self
                .children
                .iter()
                .map(|child| {
                    let outer = child.outer_size();
                    let size = if child.grow > 0.0 {
                        inner.size()
                    } else {
                        outer.min(inner.size())
                    };
                    child.solve(align.align_size_within_rect(size, inner))
                })
                .collect(),
        };
        Placed {
            id: self.id,
            rect,
            children,
        }
    }

    fn solve_flex(&self, axis: Axis, inner: Rect) -> Vec<Placed> {
        let main_avail = main(axis, inner.size()).max(0.0);
        let cross_avail = cross(axis, inner.size()).max(0.0);
        let mut lines: Vec<Vec<usize>> = vec![Vec::new()];
        let mut used = 0.0;
        for (index, child) in self.children.iter().enumerate() {
            let base = child.span(axis).0;
            let line = lines.last_mut().unwrap();
            let needed = if line.is_empty() {
                base
            } else {
                used + self.gap + base
            };
            if self.wrap && !line.is_empty() && needed > main_avail {
                lines.push(vec![index]);
                used = base;
            } else {
                line.push(index);
                used = needed;
            }
        }

        let line_cross: Vec<f32> = if self.wrap {
            lines
                .iter()
                .map(|line| {
                    line.iter()
                        .map(|&i| cross(axis, self.children[i].outer_size()))
                        .fold(0.0, f32::max)
                })
                .collect()
        } else {
            vec![cross_avail]
        };

        let mut slots = vec![Rect::NOTHING; self.children.len()];
        let mut offset = 0.0;
        for (line, line_cross) in lines.iter().zip(line_cross) {
            let line_rect = match axis {
                Axis::Row => Rect::from_min_size(
                    Pos2::new(inner.left(), inner.top() + offset),
                    Vec2::new(main_avail, line_cross),
                ),
                Axis::Column => Rect::from_min_size(
                    Pos2::new(inner.left() + offset, inner.top()),
                    Vec2::new(line_cross, main_avail),
                ),
            };
            for (&index, slot) in line.iter().zip(self.place_line(axis, line, line_rect)) {
                slots[index] = slot;
            }
            offset += line_cross + self.line_gap;
        }
        self.children
            .iter()
            .zip(slots)
            .map(|(child, slot)| child.solve(slot))
            .collect()
    }

    fn place_line(&self, axis: Axis, line: &[usize], rect: Rect) -> Vec<Rect> {
        let count = line.len();
        if count == 0 {
            return Vec::new();
        }
        let main_avail = main(axis, rect.size());
        let cross_avail = cross(axis, rect.size());
        let gaps = self.gap * (count - 1) as f32;
        let spans: Vec<(f32, f32, f32)> =
            line.iter().map(|&i| self.children[i].span(axis)).collect();
        let mut lengths: Vec<f32> = spans.iter().map(|span| span.0).collect();
        let mut free = main_avail - gaps - lengths.iter().sum::<f32>();

        if free > 0.01 {
            let mut open: Vec<usize> = (0..count)
                .filter(|&i| self.children[line[i]].grow > 0.0)
                .collect();
            while free > 0.01 && !open.is_empty() {
                let total: f32 = open.iter().map(|&i| self.children[line[i]].grow).sum();
                let share = free;
                let mut saturated = Vec::new();
                for &i in &open {
                    let want = share * self.children[line[i]].grow / total;
                    let next = (lengths[i] + want).min(spans[i].2);
                    free -= next - lengths[i];
                    lengths[i] = next;
                    if next >= spans[i].2 {
                        saturated.push(i);
                    }
                }
                if saturated.is_empty() {
                    break;
                }
                open.retain(|i| !saturated.contains(i));
            }
        } else if free < -0.01 {
            let mut deficit = -free;
            for _ in 0..count {
                let shrinkable: f32 = (0..count).map(|i| (lengths[i] - spans[i].1).max(0.0)).sum();
                if shrinkable <= 0.0 || deficit <= 0.01 {
                    break;
                }
                let cut = deficit.min(shrinkable);
                for i in 0..count {
                    lengths[i] -= cut * (lengths[i] - spans[i].1).max(0.0) / shrinkable;
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
            Justify::SpaceAround => {
                let share = free / count as f32;
                (share * 0.5, self.gap + share)
            }
        };

        let mut slots = Vec::with_capacity(count);
        for (&index, length) in line.iter().zip(lengths) {
            let child = &self.children[index];
            let wanted = cross(axis, child.outer_size());
            let stretch = self.align == Align::Stretch && child.fixed_cross(axis).is_none();
            let size = if stretch {
                cross_avail
            } else {
                wanted.min(cross_avail)
            };
            let shift = match self.align {
                Align::Start | Align::Stretch => 0.0,
                Align::Center => (cross_avail - size) * 0.5,
                Align::End => cross_avail - size,
            };
            let origin = match axis {
                Axis::Row => Pos2::new(rect.left() + cursor, rect.top() + shift),
                Axis::Column => Pos2::new(rect.left() + shift, rect.top() + cursor),
            };
            slots.push(Rect::from_min_size(origin, vec(axis, length, size)));
            cursor += length + gap;
        }
        slots
    }

    fn fixed_cross(&self, axis: Axis) -> Option<f32> {
        match axis {
            Axis::Row => self.height,
            Axis::Column => self.width,
        }
    }
}

pub(crate) fn content_height(node: &Node, width: f32) -> f32 {
    let tall = Rect::from_min_size(Pos2::ZERO, Vec2::new(width, 1.0e6));
    node.solve(tall).bottom() + node.padding.bottom + node.margin.bottom
}

pub(crate) fn scroll_view(
    ui: &mut egui::Ui,
    id: Id,
    size: Vec2,
    node: &Node,
    paint: impl FnOnce(&mut egui::Ui, &Placed),
) {
    let height = content_height(node, size.x).max(size.y);
    egui::ScrollArea::vertical()
        .id_salt(id)
        .max_height(size.y)
        .max_width(size.x)
        .auto_shrink([false, false])
        .show(ui, |ui| {
            let (rect, _) = ui.allocate_exact_size(Vec2::new(size.x, height), Sense::hover());
            paint(ui, &node.solve(rect));
        });
}

pub(crate) fn paint_text(
    painter: &egui::Painter,
    slot: Rect,
    text: &str,
    font: &FontId,
    color: Color32,
    align: Align2,
) {
    let fitted = crate::truncate_to_width(painter, text, font, slot.width());
    let anchor = align.pos_in_rect(&slot);
    draw_text(painter, anchor, align, fitted, font.clone(), color, None);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bounds(width: f32, height: f32) -> Rect {
        Rect::from_min_size(Pos2::ZERO, Vec2::new(width, height))
    }

    fn boxed(width: f32, height: f32) -> Node {
        leaf(Vec2::new(width, height))
    }

    fn rects(node: Node, bounds: Rect) -> Vec<Rect> {
        node.solve(bounds).children.iter().map(|c| c.rect).collect()
    }

    #[test]
    fn content_height_counts_wrapped_lines_and_padding() {
        let chips = (0..6).map(|_| boxed(40.0, 20.0));
        let tree = column([row(chips).wrap().gap(10.0)]).padding(Insets::all(8.0));
        assert_eq!(
            content_height(&tree, 116.0),
            8.0 + 20.0 * 3.0 + 10.0 * 2.0 + 8.0
        );
    }

    #[test]
    fn grow_splits_leftover_by_weight() {
        let out = rects(
            row([boxed(50.0, 20.0), fill(), fill().grow(2.0)]).gap(10.0),
            bounds(260.0, 20.0),
        );
        assert!((out[1].width() - 190.0 / 3.0).abs() < 0.01);
        assert!((out[2].width() - 380.0 / 3.0).abs() < 0.01);
        assert_eq!(out[2].right(), 260.0);
    }

    #[test]
    fn max_width_hands_the_rest_to_siblings() {
        let out = rects(row([fill().max_width(40.0), fill()]), bounds(200.0, 10.0));
        assert_eq!(out[0].width(), 40.0);
        assert_eq!(out[1].width(), 160.0);
    }

    #[test]
    fn overflow_shrinks_down_to_min_width() {
        let out = rects(
            row([boxed(100.0, 10.0).min_width(95.0), boxed(100.0, 10.0)]),
            bounds(120.0, 10.0),
        );
        assert!(out[0].width() >= 95.0);
        assert!((out[0].width() + out[1].width() - 120.0).abs() < 0.01);
    }

    #[test]
    fn nested_column_of_rows_places_every_leaf() {
        let tree = column([
            row([boxed(20.0, 10.0).id(0), boxed(30.0, 10.0).id(1)]).gap(5.0),
            row([boxed(40.0, 12.0).id(2)]),
        ])
        .gap(8.0)
        .padding(Insets::all(10.0));
        let placed = tree.solve(bounds(200.0, 200.0));
        assert_eq!(placed.find(0).unwrap().min, Pos2::new(10.0, 10.0));
        assert_eq!(placed.find(1).unwrap().min, Pos2::new(35.0, 10.0));
        assert_eq!(placed.find(2).unwrap().min, Pos2::new(10.0, 28.0));
    }

    #[test]
    fn container_hugs_its_content() {
        let tree = row([boxed(40.0, 20.0), boxed(60.0, 30.0)])
            .gap(6.0)
            .padding(Insets::xy(10.0, 4.0));
        assert_eq!(tree.outer_size(), Vec2::new(126.0, 38.0));
        let centered = stack(Align2::CENTER_CENTER, [tree.id(7)]).solve(bounds(300.0, 100.0));
        assert_eq!(centered.find(7).unwrap().center(), Pos2::new(150.0, 50.0));
    }

    #[test]
    fn padding_sides_and_margin_are_independent() {
        let tree = column([boxed(10.0, 10.0).id(0).margin(Insets::all(2.0).top(6.0))])
            .padding(Insets::default().top(20.0).left(5.0));
        let placed = tree.solve(bounds(100.0, 100.0));
        assert_eq!(placed.find(0).unwrap().min, Pos2::new(7.0, 26.0));
    }

    #[test]
    fn wrap_breaks_to_the_next_line_when_full() {
        let chips = (0..5).map(|i| boxed(40.0, 20.0).id(i));
        let placed = row(chips).wrap().gap(10.0).solve(bounds(140.0, 200.0));
        assert_eq!(placed.find(2).unwrap().min, Pos2::new(100.0, 0.0));
        assert_eq!(placed.find(3).unwrap().min, Pos2::new(0.0, 30.0));
        assert_eq!(placed.find(4).unwrap().min, Pos2::new(50.0, 30.0));
    }

    #[test]
    fn stretch_skips_children_with_a_fixed_cross_size() {
        let out = rects(
            column([boxed(10.0, 10.0), boxed(10.0, 10.0).width(30.0)]).align(Align::Stretch),
            bounds(100.0, 100.0),
        );
        assert_eq!(out[0].width(), 100.0);
        assert_eq!(out[1].width(), 30.0);
    }

    #[test]
    fn space_around_gives_edges_half_a_share() {
        let out = rects(
            row([boxed(20.0, 10.0), boxed(20.0, 10.0)]).justify(Justify::SpaceAround),
            bounds(100.0, 10.0),
        );
        assert_eq!(out[0].left(), 15.0);
        assert_eq!(out[1].left(), 65.0);
    }

    #[test]
    fn space_between_pins_first_and_last() {
        let out = rects(
            row((0..3).map(|_| boxed(20.0, 20.0)))
                .justify(Justify::SpaceBetween)
                .align(Align::Center),
            bounds(200.0, 40.0),
        );
        assert_eq!(out[0].left(), 0.0);
        assert_eq!(out[2].right(), 200.0);
        assert_eq!(out[1].top(), 10.0);
    }
}
