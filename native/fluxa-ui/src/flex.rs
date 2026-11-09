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
    Md,
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
    SpaceAround,
    SpaceBetween,
    SpaceEvenly,
}

#[derive(Clone)]
pub(crate) struct Measure(std::rc::Rc<dyn Fn(Option<f32>) -> Vec2>);

impl std::fmt::Debug for Measure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Measure")
    }
}

#[derive(Clone, Debug)]
enum Kind {
    Leaf(Vec2),
    Text(Measure),
    Flex(Axis),
    Stack(Align2),
}

#[derive(Clone, Debug)]
pub(crate) struct Node {
    id: Option<usize>,
    kind: Kind,
    width: Option<f32>,
    height: Option<f32>,
    min_w: Option<f32>,
    min_h: Option<f32>,
    max_w: Option<f32>,
    max_h: Option<f32>,
    basis: Option<f32>,
    grow: f32,
    shrink: f32,
    align_self: Option<Align>,
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

pub(crate) fn anchored(bounds: Rect, align: Align2, size: Vec2, inset: f32) -> Rect {
    let tree = stack(align, [leaf(size).id(0)]).padding(Insets::all(inset));
    tree.solve(bounds)
        .find(0)
        .unwrap_or(Rect::from_min_size(bounds.min, size))
}

pub(crate) fn slot(width: f32) -> Node {
    leaf(Vec2::ZERO).width(width)
}

pub(crate) fn fill() -> Node {
    leaf(Vec2::ZERO).grow(1.0)
}

pub(crate) fn text_size(painter: &crate::gfx::Painter, text: &str, font: &FontId) -> Vec2 {
    painter
        .layout_no_wrap(text.to_owned(), font.clone(), Color32::WHITE)
        .size()
}

pub(crate) fn measured(measure: impl Fn(Option<f32>) -> Vec2 + 'static) -> Node {
    Node::new(Kind::Text(Measure(std::rc::Rc::new(measure))))
}

pub(crate) fn wrapped_text(painter: &crate::gfx::Painter, text: &str, font: &FontId) -> Node {
    let painter = painter.clone();
    let text = text.to_owned();
    let font = font.clone();
    measured(move |width| {
        painter
            .layout(
                text.clone(),
                font.clone(),
                Color32::WHITE,
                width.unwrap_or(f32::INFINITY),
            )
            .size()
    })
}

pub(crate) fn text(painter: &crate::gfx::Painter, text: &str, font: &FontId) -> Node {
    leaf(text_size(painter, text, font))
}

const LINT_SLACK: f32 = 0.5;

thread_local! {
    static MUTED: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

pub(crate) fn muted<R>(run: impl FnOnce() -> R) -> R {
    let before = MUTED.with(|flag| flag.replace(true));
    let result = run();
    MUTED.with(|flag| flag.set(before));
    result
}

fn report(placed: &Placed) {
    if MUTED.with(std::cell::Cell::get) {
        return;
    }
    static SINK: std::sync::OnceLock<Option<String>> = std::sync::OnceLock::new();
    let Some(path) = SINK.get_or_init(|| std::env::var("FLUXA_LAYOUT_LINT").ok()) else {
        return;
    };
    let found = placed.lint();
    if found.is_empty() {
        return;
    }
    use std::io::Write;
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
    {
        for line in found {
            let _ = writeln!(file, "{line}");
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Placed {
    pub id: Option<usize>,
    pub rect: Rect,
    pub children: Vec<Placed>,
    pub layered: bool,
}

impl Placed {
    pub fn find(&self, id: usize) -> Option<Rect> {
        if self.id == Some(id) {
            return Some(self.rect);
        }
        self.children.iter().find_map(|child| child.find(id))
    }

    pub fn lint(&self) -> Vec<String> {
        let mut out = Vec::new();
        self.lint_into("root", &mut out);
        out
    }

    fn label(&self, path: &str, index: usize) -> String {
        match self.children[index].id {
            Some(id) => format!("{path}/#{id}"),
            None => format!("{path}/{index}"),
        }
    }

    fn lint_into(&self, path: &str, out: &mut Vec<String>) {
        let room = self.rect.expand(LINT_SLACK);
        let hollow = self.rect.width() <= LINT_SLACK || self.rect.height() <= LINT_SLACK;
        for (index, child) in self.children.iter().enumerate() {
            let name = self.label(path, index);
            let visible = child.rect.width() > LINT_SLACK && child.rect.height() > LINT_SLACK;
            if visible && !hollow && !room.contains_rect(child.rect) {
                out.push(format!(
                    "{name} overflows its parent: {:?} not inside {:?}",
                    child.rect, self.rect
                ));
            }
            if !self.layered {
                for (other, sibling) in self.children.iter().enumerate().skip(index + 1) {
                    let shared = child.rect.intersect(sibling.rect);
                    if shared.width() > LINT_SLACK && shared.height() > LINT_SLACK {
                        out.push(format!(
                            "{name} overlaps {}: {:?} and {:?}",
                            self.label(path, other),
                            child.rect,
                            sibling.rect
                        ));
                    }
                }
            }
            child.lint_into(&name, out);
        }
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
            min_w: None,
            min_h: None,
            max_w: None,
            max_h: None,
            basis: None,
            grow: 0.0,
            shrink: 1.0,
            align_self: None,
            margin: Insets::default(),
            padding: Insets::default(),
            gap: 0.0,
            line_gap: 0.0,
            align: Align::Stretch,
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

    pub fn id_of(&self) -> Option<usize> {
        self.id
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

    pub fn min_width(mut self, value: f32) -> Self {
        self.min_w = Some(value);
        self
    }

    pub fn min_height(mut self, value: f32) -> Self {
        self.min_h = Some(value);
        self
    }

    pub fn max_width(mut self, value: f32) -> Self {
        self.max_w = Some(value);
        self
    }

    pub fn max_height(mut self, value: f32) -> Self {
        self.max_h = Some(value);
        self
    }

    pub fn basis(mut self, value: f32) -> Self {
        self.basis = Some(value);
        self
    }

    pub fn shrink(mut self, shrink: f32) -> Self {
        self.shrink = shrink.max(0.0);
        self
    }

    pub fn align_self(mut self, align: Align) -> Self {
        self.align_self = Some(align);
        self
    }

    pub fn margin(mut self, margin: Insets) -> Self {
        self.margin = margin;
        self
    }

    pub fn grow(mut self, grow: f32) -> Self {
        self.grow = grow;
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

    fn measured(&self, width: Option<f32>) -> Option<Vec2> {
        match &self.kind {
            Kind::Text(measure) => Some((measure.0)(width)),
            _ => None,
        }
    }

    fn content_size(&self) -> Vec2 {
        match &self.kind {
            Kind::Leaf(size) => *size,
            Kind::Text(measure) => (measure.0)(None),
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

    fn min_content(&self) -> Vec2 {
        match &self.kind {
            Kind::Leaf(size) => *size,
            Kind::Text(measure) => (measure.0)(Some(0.0)),
            Kind::Flex(axis) => {
                let sizes: Vec<Vec2> = self.children.iter().map(Node::min_outer).collect();
                if self.wrap {
                    return sizes.iter().fold(Vec2::ZERO, |acc, size| acc.max(*size));
                }
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
                .map(Node::min_outer)
                .fold(Vec2::ZERO, |acc, size| acc.max(size)),
        }
    }

    fn min_outer(&self) -> Vec2 {
        let content = self.min_content();
        let padded = Vec2::new(
            content.x + self.padding.width(),
            content.y + self.padding.height(),
        );
        let x = match (self.min_w, self.width) {
            (Some(min), _) => min,
            (None, Some(width)) => width.min(padded.x),
            (None, None) => padded.x,
        };
        let y = match (self.min_h, self.height) {
            (Some(min), _) => min,
            (None, Some(height)) => height.min(padded.y),
            (None, None) => padded.y,
        };
        Vec2::new(x + self.margin.width(), y + self.margin.height())
    }

    fn border_size(&self) -> Vec2 {
        let content = self.content_size();
        let padded = Vec2::new(
            content.x + self.padding.width(),
            content.y + self.padding.height(),
        );
        let w = self.width.unwrap_or(padded.x);
        let h = self.height.unwrap_or(padded.y);
        Vec2::new(
            w.min(self.max_w.unwrap_or(f32::INFINITY))
                .max(self.min_w.unwrap_or(0.0)),
            h.min(self.max_h.unwrap_or(f32::INFINITY))
                .max(self.min_h.unwrap_or(0.0)),
        )
    }

    pub fn outer_size(&self) -> Vec2 {
        let border = self.border_size();
        Vec2::new(
            border.x + self.margin.width(),
            border.y + self.margin.height(),
        )
    }

    fn span(&self, axis: Axis) -> (f32, f32, f32) {
        let outer = main(axis, self.outer_size());
        let margin = main(axis, Vec2::new(self.margin.width(), self.margin.height()));
        let base = match self.basis {
            Some(basis) => basis + margin,
            None => outer,
        };
        let min = main(axis, self.min_outer()).min(base);
        let max = match axis {
            Axis::Row => self.max_w,
            Axis::Column => self.max_h,
        }
        .map_or(f32::INFINITY, |max| max + margin);
        (base, min, max.max(base).max(min))
    }

    fn fixed(&self, axis: Axis) -> Option<f32> {
        match axis {
            Axis::Row => self.width,
            Axis::Column => self.height,
        }
    }

    pub fn solve(&self, bounds: Rect) -> Placed {
        let placed = self.place(bounds);
        report(&placed);
        placed
    }

    fn place(&self, bounds: Rect) -> Placed {
        let bounds = self.margin.shrink(bounds);
        let inner = self.padding.shrink(bounds);
        let children = match &self.kind {
            Kind::Leaf(_) | Kind::Text(_) => Vec::new(),
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
                    child.place(align.align_size_within_rect(size, inner))
                })
                .collect(),
        };
        Placed {
            id: self.id,
            rect: bounds,
            children,
            layered: matches!(self.kind, Kind::Stack(_)),
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
            .map(|(child, slot)| child.place(slot))
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
            let mut frozen = vec![false; count];
            for _ in 0..count {
                let weights: Vec<f32> = (0..count)
                    .map(|i| {
                        if frozen[i] {
                            0.0
                        } else {
                            self.children[line[i]].shrink * lengths[i]
                        }
                    })
                    .collect();
                let total: f32 = weights.iter().sum();
                if total <= 0.0 || deficit <= 0.01 {
                    break;
                }
                let mut spent = 0.0;
                let mut clamped = false;
                for i in 0..count {
                    if weights[i] <= 0.0 {
                        continue;
                    }
                    let cut = deficit * weights[i] / total;
                    let next = (lengths[i] - cut).max(spans[i].1);
                    if next > lengths[i] - cut {
                        frozen[i] = true;
                        clamped = true;
                    }
                    spent += lengths[i] - next;
                    lengths[i] = next;
                }
                deficit -= spent;
                if !clamped {
                    break;
                }
            }
            free = -deficit;
        }

        let free = free.max(0.0);
        let (mut cursor, gap) = match self.justify {
            Justify::Start => (0.0, self.gap),
            Justify::Center => (free * 0.5, self.gap),
            Justify::End => (free, self.gap),
            Justify::SpaceAround => {
                let share = free / count as f32;
                (share * 0.5, self.gap + share)
            }
            Justify::SpaceBetween if count > 1 => (0.0, self.gap + free / (count - 1) as f32),
            Justify::SpaceBetween => (0.0, self.gap),
            Justify::SpaceEvenly => {
                let share = free / (count + 1) as f32;
                (share, self.gap + share)
            }
        };

        let mut slots = Vec::with_capacity(count);
        for (&index, length) in line.iter().zip(lengths) {
            let child = &self.children[index];
            let align = child.align_self.unwrap_or(self.align);
            let mut wanted = cross(axis, child.outer_size());
            if axis == Axis::Row && child.height.is_none() {
                let inner = length - child.margin.width() - child.padding.width();
                if let Some(size) = child.measured(Some(inner.max(0.0))) {
                    wanted = size.y + child.padding.height() + child.margin.height();
                }
            }
            let stretch = align == Align::Stretch && child.fixed_cross(axis).is_none();
            let size = if stretch {
                cross_avail
            } else {
                wanted.min(cross_avail)
            };
            let shift = match align {
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

pub(crate) fn paint_text(
    painter: &crate::gfx::Painter,
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

    #[test]
    fn anchored_pins_to_the_requested_corner_inside_the_inset() {
        let area = Rect::from_min_size(Pos2::new(10.0, 20.0), Vec2::new(100.0, 80.0));
        let rect = anchored(area, Align2::LEFT_BOTTOM, Vec2::new(30.0, 10.0), 4.0);
        assert_eq!(rect.min, Pos2::new(14.0, 86.0));
        assert_eq!(rect.size(), Vec2::new(30.0, 10.0));
    }

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
    fn overflow_shrinks_down_to_min_width() {
        let out = rects(
            row([
                boxed(100.0, 10.0).min_width(95.0),
                boxed(100.0, 10.0).min_width(0.0),
            ]),
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
    fn lint_flags_children_that_overflow_or_overlap() {
        let overflowing = row([
            boxed(80.0, 10.0).min_width(80.0),
            boxed(80.0, 10.0).min_width(80.0),
        ])
        .solve(bounds(100.0, 10.0));
        assert!(!overflowing.lint().is_empty());
        let layered = stack(
            Align2::CENTER_CENTER,
            [boxed(20.0, 20.0), boxed(10.0, 10.0)],
        )
        .solve(bounds(100.0, 100.0));
        assert!(layered.lint().is_empty());
    }

    #[test]
    fn lint_is_clean_for_a_well_formed_tree() {
        let tree = column([
            row([boxed(20.0, 10.0), fill(), boxed(30.0, 10.0)]).gap(5.0),
            row([boxed(40.0, 12.0)]),
        ])
        .gap(8.0)
        .padding(Insets::all(10.0));
        assert!(tree.solve(bounds(200.0, 200.0)).lint().is_empty());
    }

    #[test]
    fn space_between_pins_children_to_both_edges() {
        let out = rects(
            row([boxed(20.0, 10.0), boxed(20.0, 10.0), boxed(20.0, 10.0)])
                .justify(Justify::SpaceBetween),
            bounds(100.0, 10.0),
        );
        assert_eq!(out[0].left(), 0.0);
        assert_eq!(out[1].left(), 40.0);
        assert_eq!(out[2].right(), 100.0);
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
    fn items_do_not_shrink_below_content_without_an_explicit_min() {
        let out = rects(row([boxed(100.0, 10.0), boxed(100.0, 10.0)]), bounds(120.0, 10.0));
        assert_eq!(out[0].width(), 100.0);
    }

    #[test]
    fn shrink_zero_keeps_the_item_and_margins_offset_it() {
        let out = rects(
            row([
                boxed(50.0, 10.0).shrink(0.0).margin(Insets::xy(5.0, 0.0)),
                boxed(80.0, 10.0).min_width(0.0),
            ]),
            bounds(100.0, 10.0),
        );
        assert_eq!(out[0].left(), 5.0);
        assert_eq!(out[0].width(), 50.0);
    }

    #[test]
    fn align_self_and_max_width_apply_per_item() {
        let out = rects(
            row([boxed(10.0, 10.0).align_self(Align::End), boxed(10.0, 10.0).max_width(5.0)]),
            bounds(100.0, 40.0),
        );
        assert_eq!(out[0].top(), 30.0);
        assert_eq!(out[1].width(), 5.0);
    }
}
