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
    pub fn px(self) -> f32 {
        crate::metrics::theme().space[self as usize]
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Radius {
    Md,
    Sheet,
}

impl Radius {
    pub fn px(self) -> f32 {
        crate::metrics::theme().radius[self as usize]
    }

    pub fn corner(self) -> u8 {
        self.px().round().clamp(0.0, 255.0) as u8
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum IconSize {
    Sm,
    Md,
    Xl,
}

impl IconSize {
    pub fn px(self) -> f32 {
        crate::metrics::theme().icon[self as usize]
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

#[derive(Clone, Copy, Default, Debug)]
struct Memo {
    border: Option<Vec2>,
    min: Option<Vec2>,
    width_dependent: Option<bool>,
}

thread_local! {
    static SOLVING: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
}

fn memoising() -> bool {
    SOLVING.with(|depth| depth.get() > 0)
}

#[derive(Clone, Debug)]
pub(crate) enum Rel {
    Share(f32),
    Calc(
        std::sync::Arc<crate::stylesheet::math::Expr>,
        crate::stylesheet::math::Env,
    ),
}

impl Rel {
    fn at(&self, basis: f32) -> f32 {
        match self {
            Rel::Share(share) => basis * share,
            Rel::Calc(expr, env) => expr
                .eval(&crate::stylesheet::math::Env {
                    percent: Some(basis),
                    ..*env
                })
                .unwrap_or(0.0),
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Node {
    id: Option<usize>,
    kind: Kind,
    width: Option<f32>,
    height: Option<f32>,
    width_rel: Option<Rel>,
    height_rel: Option<Rel>,
    percent_basis: Option<Vec2>,
    min_w: Option<f32>,
    min_h: Option<f32>,
    max_w: Option<f32>,
    max_h: Option<f32>,
    aspect: Option<f32>,
    basis: Option<f32>,
    grow: f32,
    shrink: f32,
    align_self: Option<Align>,
    margin: Insets,
    padding: Insets,
    gap: f32,
    line_gap: f32,
    align: Option<Align>,
    justify: Option<Justify>,
    placement: (Option<Align>, Option<Align>),
    wrap: bool,
    pan: Option<u64>,
    children: Vec<Node>,
    memo: std::cell::Cell<Memo>,
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
    leaf(Vec2::ZERO).width(width).align_self(Align::Stretch)
}

pub(crate) fn fill() -> Node {
    leaf(Vec2::ZERO).grow(1.0).align_self(Align::Stretch)
}

pub(crate) fn text_size(painter: &crate::gfx::Painter, text: &str, font: &FontId) -> Vec2 {
    painter
        .layout_no_wrap(text.to_owned(), font.clone(), Color32::WHITE)
        .size()
}

pub(crate) fn measured(measure: impl Fn(Option<f32>) -> Vec2 + 'static) -> Node {
    Node::new(Kind::Text(Measure(std::rc::Rc::new(measure))))
}

pub(crate) fn text(painter: &crate::gfx::Painter, text: &str, font: &FontId) -> Node {
    leaf(text_size(painter, text, font))
}

const LINT_SLACK: f32 = 0.5;
const PAN_SLACK: f32 = 2.0;

thread_local! {
    static MUTED: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
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
    pub bleeds: bool,
    pub pan: Option<Pan>,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Pan {
    pub key: u64,
    pub viewport: Rect,
    pub content: f32,
}

impl Placed {
    pub fn find(&self, id: usize) -> Option<Rect> {
        if self.id == Some(id) {
            return Some(self.rect);
        }
        self.children.iter().find_map(|child| child.find(id))
    }

    pub fn rects_by_id(&self) -> std::collections::HashMap<usize, Rect> {
        let mut rects = std::collections::HashMap::new();
        self.collect_ids(&mut rects);
        rects
    }

    fn collect_ids(&self, rects: &mut std::collections::HashMap<usize, Rect>) {
        if let Some(id) = self.id {
            rects.entry(id).or_insert(self.rect);
        }
        for child in &self.children {
            child.collect_ids(rects);
        }
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
            if visible
                && !hollow
                && !child.bleeds
                && self.pan.is_none()
                && !room.contains_rect(child.rect)
            {
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
            width_rel: None,
            height_rel: None,
            percent_basis: None,
            min_w: None,
            min_h: None,
            max_w: None,
            max_h: None,
            aspect: None,
            basis: None,
            grow: 0.0,
            shrink: 1.0,
            align_self: None,
            margin: Insets::default(),
            padding: Insets::default(),
            gap: 0.0,
            line_gap: 0.0,
            align: None,
            justify: None,
            placement: (None, None),
            wrap: false,
            pan: None,
            children: Vec::new(),
            memo: std::cell::Cell::default(),
        }
    }

    fn reset_memo(&self) {
        self.memo.set(Memo::default());
        for child in &self.children {
            child.reset_memo();
        }
    }

    fn container(kind: Kind, children: impl IntoIterator<Item = Node>) -> Self {
        let mut node = Self::new(kind);
        node.children = children.into_iter().collect();
        node
    }

    pub fn with_children(mut self, children: impl IntoIterator<Item = Node>) -> Self {
        self.children = children.into_iter().collect();
        if let Some(space) = self.percent_basis {
            self.resolve_children(space);
        }
        self
    }

    pub fn percent_of(mut self, space: Vec2) -> Self {
        self.percent_basis = Some(space);
        self
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

    pub fn width_pct(self, share: f32) -> Self {
        self.width_rel(Rel::Share(share.max(0.0)))
    }

    pub fn height_pct(self, share: f32) -> Self {
        self.height_rel(Rel::Share(share.max(0.0)))
    }

    pub fn width_rel(mut self, rel: Rel) -> Self {
        self.width_rel = Some(rel);
        self
    }

    pub fn height_rel(mut self, rel: Rel) -> Self {
        self.height_rel = Some(rel);
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

    pub fn aspect(mut self, ratio: f32) -> Self {
        self.aspect = Some(ratio).filter(|ratio| *ratio > 0.0);
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
        self.align = Some(align);
        self
    }

    pub fn justify(mut self, justify: Justify) -> Self {
        self.justify = Some(justify);
        self
    }

    pub fn wrap(mut self) -> Self {
        self.wrap = true;
        self
    }

    pub fn placed(mut self, horizontal: Option<Align>, vertical: Option<Align>) -> Self {
        self.placement = (horizontal, vertical);
        self
    }

    pub fn pan(mut self, key: u64) -> Self {
        self.pan = Some(key);
        self
    }

    fn content_size(&self) -> Vec2 {
        match &self.kind {
            Kind::Leaf(size) => *size,
            Kind::Text(measure) => (measure.0)(None),
            Kind::Flex(Axis::Row) if self.wrap && self.width.is_some() => {
                let avail = (self.width.unwrap_or(0.0) - self.padding.width()).max(0.0);
                Vec2::new(avail, self.wrapped_lines_height(avail))
            }
            Kind::Flex(axis) => self.pack(*axis, Node::outer_size),
            Kind::Stack(_) => self
                .children
                .iter()
                .map(Node::outer_size)
                .fold(Vec2::ZERO, |acc, size| acc.max(size)),
        }
    }

    fn pack(&self, axis: Axis, size_of: fn(&Node) -> Vec2) -> Vec2 {
        let (mut main_total, mut cross_max) = (0.0, 0.0_f32);
        for child in &self.children {
            let size = size_of(child);
            main_total += main(axis, size);
            cross_max = cross_max.max(cross(axis, size));
        }
        let gaps = self.gap * self.children.len().saturating_sub(1) as f32;
        vec(axis, main_total + gaps, cross_max)
    }

    fn wrapped_lines_height(&self, avail: f32) -> f32 {
        let (mut used, mut line_cross, mut total) = (0.0, 0.0_f32, 0.0);
        let mut lines = 0;
        for child in &self.children {
            let size = child.outer_size();
            let needed = if used == 0.0 {
                size.x
            } else {
                used + self.gap + size.x
            };
            if used > 0.0 && needed > avail {
                total += line_cross;
                lines += 1;
                used = size.x;
                line_cross = size.y;
            } else {
                used = needed;
                line_cross = line_cross.max(size.y);
            }
        }
        if !self.children.is_empty() {
            total += line_cross;
            lines += 1;
        }
        total + self.line_gap * (lines.max(1) - 1) as f32
    }

    fn width_dependent(&self) -> bool {
        if !memoising() {
            return self.compute_width_dependent();
        }
        let mut memo = self.memo.get();
        if let Some(dependent) = memo.width_dependent {
            return dependent;
        }
        let dependent = self.compute_width_dependent();
        memo.width_dependent = Some(dependent);
        self.memo.set(memo);
        dependent
    }

    fn compute_width_dependent(&self) -> bool {
        match &self.kind {
            Kind::Text(_) => true,
            Kind::Flex(Axis::Row) if self.wrap => true,
            Kind::Flex(_) => self.children.iter().any(Node::width_dependent),
            _ => false,
        }
    }

    fn height_for_width(&self, width: f32) -> Option<f32> {
        if self.height.is_some() || !self.width_dependent() {
            return None;
        }
        let border = self
            .width
            .unwrap_or(width - self.margin.width())
            .min(self.max_w.unwrap_or(f32::INFINITY))
            .max(self.min_w.unwrap_or(0.0));
        let inner = (border - self.padding.width()).max(0.0);
        let content = match &self.kind {
            Kind::Text(measure) => (measure.0)(Some(inner)).y,
            Kind::Flex(Axis::Row) if self.wrap => self.wrapped_lines_height(inner),
            Kind::Flex(Axis::Row) => self.row_content_height(inner),
            Kind::Flex(Axis::Column) => self.column_content_height(inner),
            Kind::Leaf(_) | Kind::Stack(_) => return None,
        };
        let height = (content + self.padding.height())
            .min(self.max_h.unwrap_or(f32::INFINITY))
            .max(self.min_h.unwrap_or(0.0));
        Some(height + self.margin.height())
    }

    fn row_content_height(&self, inner: f32) -> f32 {
        let line: Vec<usize> = (0..self.children.len()).collect();
        let (lengths, _) = self.flex_lengths(Axis::Row, &line, inner, f32::INFINITY);
        self.children
            .iter()
            .zip(lengths)
            .map(|(child, length)| {
                child
                    .height_for_width(length)
                    .unwrap_or_else(|| child.outer_size().y)
            })
            .fold(0.0, f32::max)
    }

    fn column_content_height(&self, inner: f32) -> f32 {
        let gaps = self.gap * self.children.len().saturating_sub(1) as f32;
        let total: f32 = self
            .children
            .iter()
            .map(|child| {
                child
                    .height_for_width(self.cross_width(child, inner))
                    .unwrap_or_else(|| child.outer_size().y)
            })
            .sum();
        total + gaps
    }

    fn cross_width(&self, child: &Node, avail: f32) -> f32 {
        let align = child.align_self.or(self.align).unwrap_or(Align::Stretch);
        if align == Align::Stretch && child.width.is_none() && child.aspect.is_none() {
            avail
        } else {
            child.outer_size().x.min(avail)
        }
    }

    fn min_content(&self) -> Vec2 {
        match &self.kind {
            Kind::Leaf(size) => *size,
            Kind::Text(measure) => (measure.0)(Some(0.0)),
            Kind::Flex(_) if self.wrap => self
                .children
                .iter()
                .map(Node::min_outer)
                .fold(Vec2::ZERO, |acc, size| acc.max(size)),
            Kind::Flex(axis) => self.pack(*axis, Node::min_outer),
            Kind::Stack(_) => self
                .children
                .iter()
                .map(Node::min_outer)
                .fold(Vec2::ZERO, |acc, size| acc.max(size)),
        }
    }

    fn min_outer(&self) -> Vec2 {
        if !memoising() {
            return self.compute_min_outer();
        }
        let mut memo = self.memo.get();
        if let Some(size) = memo.min {
            return size;
        }
        let size = self.compute_min_outer();
        memo.min = Some(size);
        self.memo.set(memo);
        size
    }

    fn compute_min_outer(&self) -> Vec2 {
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
        if !memoising() {
            return self.compute_border_size();
        }
        let mut memo = self.memo.get();
        if let Some(size) = memo.border {
            return size;
        }
        let size = self.compute_border_size();
        memo.border = Some(size);
        self.memo.set(memo);
        size
    }

    fn compute_border_size(&self) -> Vec2 {
        let content = self.content_size();
        let padded = Vec2::new(
            content.x + self.padding.width(),
            content.y + self.padding.height(),
        );
        let clamp_w = |w: f32| {
            w.min(self.max_w.unwrap_or(f32::INFINITY))
                .max(self.min_w.unwrap_or(0.0))
        };
        let clamp_h = |h: f32| {
            h.min(self.max_h.unwrap_or(f32::INFINITY))
                .max(self.min_h.unwrap_or(0.0))
        };
        match (self.aspect, self.width, self.height) {
            (Some(ratio), Some(width), None) => {
                let w = clamp_w(width);
                Vec2::new(w, clamp_h(w / ratio))
            }
            (Some(ratio), None, Some(height)) => {
                let h = clamp_h(height);
                Vec2::new(clamp_w(h * ratio), h)
            }
            _ => Vec2::new(
                clamp_w(self.width.unwrap_or(padded.x)),
                clamp_h(self.height.unwrap_or(padded.y)),
            ),
        }
    }

    pub fn grows(&self) -> bool {
        self.grow > 0.0 || self.children.iter().any(Node::grows)
    }

    pub fn outer_size(&self) -> Vec2 {
        crate::stats::count(&crate::stats::OUTER_CALLS);
        let border = self.border_size();
        Vec2::new(
            border.x + self.margin.width(),
            border.y + self.margin.height(),
        )
    }

    fn span(&self, axis: Axis) -> (f32, f32, f32) {
        crate::stats::count(&crate::stats::SPAN_CALLS);
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

    pub fn solve(&self, bounds: Rect) -> Placed {
        let timer = crate::stats::start();
        crate::stats::count(&crate::stats::SOLVES);
        self.reset_memo();
        SOLVING.with(|depth| depth.set(depth.get() + 1));
        let placed = self.place(bounds);
        SOLVING.with(|depth| depth.set(depth.get() - 1));
        timer.add(&crate::stats::LAYOUT_NS);
        report(&placed);
        placed
    }

    fn place(&self, bounds: Rect) -> Placed {
        let bounds = self.margin.shrink(bounds);
        let inner = self.padding.shrink(bounds);
        let resolved;
        let node = if self.children.iter().any(Node::relative) {
            resolved = self.with_percent_sizes(inner.size());
            &resolved
        } else {
            self
        };
        let children = match &node.kind {
            Kind::Leaf(_) | Kind::Text(_) => Vec::new(),
            Kind::Flex(axis) => node.solve_flex(*axis, inner),
            Kind::Stack(origin) => node
                .children
                .iter()
                .map(|child| {
                    let horizontal = child.placement.0.or(child.align_self).or(node.align);
                    let vertical = child.placement.1.or(match node.justify {
                        Some(Justify::Center) => Some(Align::Center),
                        Some(Justify::End) => Some(Align::End),
                        Some(_) => Some(Align::Start),
                        None => None,
                    });
                    child.place(node.stack_slot(*origin, child, inner, (horizontal, vertical)))
                })
                .collect(),
        };
        let pan = self.pan.filter(|_| !self.wrap).and_then(|key| {
            let right = children
                .iter()
                .map(|child| child.rect.right())
                .fold(inner.left(), f32::max);
            (matches!(self.kind, Kind::Flex(Axis::Row)) && right > inner.right() + PAN_SLACK).then(
                || Pan {
                    key,
                    viewport: inner,
                    content: right - inner.left(),
                },
            )
        });
        Placed {
            id: self.id,
            rect: bounds,
            children,
            layered: matches!(self.kind, Kind::Stack(_)),
            bleeds: self.margin.left < 0.0
                || self.margin.top < 0.0
                || self.margin.right < 0.0
                || self.margin.bottom < 0.0,
            pan,
        }
    }

    fn stack_slot(
        &self,
        origin: Align2,
        child: &Node,
        inner: Rect,
        (horizontal, vertical): (Option<Align>, Option<Align>),
    ) -> Rect {
        let outer = child.outer_size();
        let mut size = if child.grow > 0.0 {
            inner.size()
        } else {
            outer.min(inner.size())
        };
        if horizontal == Some(Align::Stretch) && child.width.is_none() {
            size.x = inner.width();
        }
        if vertical == Some(Align::Stretch) && child.height.is_none() {
            size.y = inner.height();
        }
        let base = origin.align_size_within_rect(size, inner);
        let free = inner.size() - size;
        let place = |axis: Option<Align>, free: f32, at: f32, start: f32| match axis {
            Some(Align::Start | Align::Stretch) => start,
            Some(Align::Center) => start + free * 0.5,
            Some(Align::End) => start + free,
            None => at,
        };
        let min = Pos2::new(
            place(horizontal, free.x, base.min.x, inner.left()),
            place(vertical, free.y, base.min.y, inner.top()),
        );
        Rect::from_min_size(min, size)
    }

    fn relative(&self) -> bool {
        self.width_rel.is_some() || self.height_rel.is_some()
    }

    fn with_percent_sizes(&self, space: Vec2) -> Node {
        let mut node = self.clone();
        node.resolve_children(space);
        node
    }

    fn resolve_children(&mut self, space: Vec2) {
        for child in &mut self.children {
            if let Some(rel) = child.width_rel.take() {
                child.width = Some((rel.at(space.x) - child.margin.width()).max(0.0));
                child.memo.set(Memo::default());
            }
            if let Some(rel) = child.height_rel.take() {
                child.height = Some((rel.at(space.y) - child.margin.height()).max(0.0));
                child.memo.set(Memo::default());
            }
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
                        .map(|&i| {
                            let child = &self.children[i];
                            let wrapped = child
                                .height_for_width(child.span(axis).0.min(main_avail))
                                .filter(|_| axis == Axis::Row);
                            wrapped.unwrap_or_else(|| cross(axis, child.outer_size()))
                        })
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

    fn flex_lengths(
        &self,
        axis: Axis,
        line: &[usize],
        main_avail: f32,
        cross_avail: f32,
    ) -> (Vec<f32>, f32) {
        let count = line.len();
        let gaps = self.gap * count.saturating_sub(1) as f32;
        let spans: Vec<(f32, f32, f32)> = line
            .iter()
            .map(|&i| {
                let child = &self.children[i];
                let (base, min, max) = child.span(axis);
                let wrapped = Some(axis)
                    .filter(|axis| *axis == Axis::Column)
                    .and_then(|_| child.height_for_width(self.cross_width(child, cross_avail)));
                match wrapped {
                    Some(height) => (height, min.min(height), max.max(height)),
                    None => (base, min, max),
                }
            })
            .collect();
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
        } else if free < -0.01 && (axis == Axis::Row || self.height.is_some()) {
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
        (lengths, free)
    }

    fn place_line(&self, axis: Axis, line: &[usize], rect: Rect) -> Vec<Rect> {
        let count = line.len();
        if count == 0 {
            return Vec::new();
        }
        let main_avail = main(axis, rect.size());
        let cross_avail = cross(axis, rect.size());
        let (lengths, free) = self.flex_lengths(axis, line, main_avail, cross_avail);

        let free = free.max(0.0);
        let (mut cursor, gap) = match self.justify.unwrap_or(Justify::Start) {
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
            let align = child.align_self.or(self.align).unwrap_or(Align::Stretch);
            let mut wanted = cross(axis, child.outer_size());
            if axis == Axis::Row
                && let Some(height) = child.height_for_width(length)
            {
                wanted = height;
            }
            if let Some(ratio) = child.aspect.filter(|_| child.fixed_cross(axis).is_none()) {
                let main_margin = match axis {
                    Axis::Row => child.margin.width(),
                    Axis::Column => child.margin.height(),
                };
                let inner = length - main_margin;
                wanted = match axis {
                    Axis::Row => inner / ratio + child.margin.height(),
                    Axis::Column => inner * ratio + child.margin.width(),
                };
            }
            let stretch = align == Align::Stretch
                && child.fixed_cross(axis).is_none()
                && child.aspect.is_none();
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

pub(crate) fn centered(parent: Rect, size: Vec2) -> Rect {
    anchored(parent, Align2::CENTER_CENTER, size, 0.0)
}

pub(crate) fn scaled(parent: Rect, scale: f32) -> Rect {
    Rect::from_center_size(parent.center(), parent.size() * scale)
}

pub(crate) fn centered_at_x(parent: Rect, x: f32, size: Vec2) -> Rect {
    Rect::from_center_size(Pos2::new(x, parent.center().y), size)
}

pub(crate) fn width_clip(parent: Rect, width: f32) -> Rect {
    Rect::from_min_size(parent.min, Vec2::new(width, parent.height()))
}

pub(crate) fn with_height(parent: Rect, height: f32) -> Rect {
    Rect::from_min_size(parent.min, Vec2::new(parent.width(), height))
}

pub(crate) fn trailing_point(parent: Rect, gap: f32) -> Rect {
    Rect::from_min_size(Pos2::new(parent.max.x + gap, parent.min.y), Vec2::ZERO)
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

    fn wrapping_text(natural: f32, line: f32) -> Node {
        measured(move |width| match width {
            None => Vec2::new(natural, line),
            Some(width) => {
                let lines = (natural / width.max(1.0)).ceil().max(1.0);
                Vec2::new(width.min(natural), lines * line)
            }
        })
        .min_width(0.0)
    }

    #[test]
    fn cards_grow_when_text_inside_wraps_after_shrinking() {
        let card = |id| {
            row([
                column([wrapping_text(200.0, 10.0)]).id(id),
                boxed(100.0, 10.0).shrink(0.0),
            ])
            .id(id + 10)
        };
        let placed = column([card(1), card(2)])
            .gap(4.0)
            .solve(bounds(220.0, 200.0));
        assert_eq!(placed.find(11).unwrap().height(), 20.0);
        assert_eq!(placed.find(12).unwrap().top(), 24.0);
        assert_eq!(placed.find(1).unwrap().width(), 120.0);
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
    fn fixed_width_wrap_reports_every_line() {
        let chips = (0..5).map(|_| boxed(40.0, 20.0));
        let tree = row(chips).wrap().gap(10.0).line_gap(8.0).width(140.0);
        assert_eq!(tree.outer_size(), Vec2::new(140.0, 20.0 * 2.0 + 8.0));
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
        let out = rects(
            row([boxed(100.0, 10.0), boxed(100.0, 10.0)]),
            bounds(120.0, 10.0),
        );
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
            row([
                boxed(10.0, 10.0).align_self(Align::End),
                boxed(10.0, 10.0).max_width(5.0),
            ]),
            bounds(100.0, 40.0),
        );
        assert_eq!(out[0].top(), 30.0);
        assert_eq!(out[1].width(), 5.0);
    }

    #[test]
    fn auto_height_column_keeps_rows_when_bounds_are_shorter() {
        let rows = column((0..3).map(|index| {
            row([
                slot(100.0).height(20.0).id(index * 2),
                fill().height(20.0).id(index * 2 + 1),
            ])
            .gap(8.0)
        }));
        let bounds = Rect::from_min_size(Pos2::new(10.0, 10.0), Vec2::new(300.0, 0.0));
        let placed = rows.solve(bounds);
        for index in 0..3 {
            let label = placed.find(index * 2).unwrap();
            let value = placed.find(index * 2 + 1).unwrap();
            assert_eq!(label.height(), 20.0);
            assert_eq!(label.top(), 10.0 + 20.0 * index as f32);
            assert_eq!(value.height(), 20.0);
            assert_eq!(value.left(), 118.0);
            assert_eq!(value.width(), 192.0);
        }
    }

    #[test]
    fn fill_slot_in_a_start_aligned_row_spans_the_row_height_for_centered_text() {
        let placed = row([fill().id(0)])
            .padding(Insets {
                left: 40.0,
                right: 12.0,
                ..Default::default()
            })
            .align(Align::Start)
            .solve(Rect::from_min_size(Pos2::ZERO, Vec2::new(200.0, 60.0)));
        let slot = placed.find(0).unwrap();
        assert_eq!(slot.top(), 0.0);
        assert_eq!(slot.height(), 60.0);
        assert_eq!(slot.center().y, 30.0);
    }

    #[test]
    fn text_leaf_in_a_centered_row_is_vertically_centered() {
        let placed = row([leaf(Vec2::new(50.0, 10.0)).id(0)])
            .align(Align::Center)
            .solve(Rect::from_min_size(Pos2::ZERO, Vec2::new(200.0, 60.0)));
        assert_eq!(placed.find(0).unwrap().center().y, 30.0);
    }

    #[test]
    fn grow_stops_at_max_width_and_hands_the_rest_to_siblings() {
        let out = rects(row([fill().max_width(50.0), fill()]), bounds(200.0, 10.0));
        assert_eq!(out[0].width(), 50.0);
        assert_eq!(out[1].width(), 150.0);
    }

    #[test]
    fn wrapping_row_without_a_width_pushes_the_next_sibling_below_every_line() {
        let chips = (0..5).map(|i| boxed(40.0, 20.0).id(i));
        let tree = column([row(chips).wrap().gap(10.0), boxed(10.0, 10.0).id(9)]);
        let placed = tree.solve(bounds(140.0, 200.0));
        let last_line = placed.find(3).unwrap();
        let below = placed.find(9).unwrap();
        assert!(below.top() >= last_line.bottom());
    }

    #[test]
    fn aspect_derives_height_from_width() {
        let tree = boxed(0.0, 0.0).width(160.0).aspect(16.0 / 9.0);
        assert_eq!(tree.outer_size(), Vec2::new(160.0, 90.0));
    }

    #[test]
    fn aspect_derives_width_from_height() {
        let tree = boxed(0.0, 0.0).height(210.0).aspect(2.0 / 3.0);
        assert_eq!(tree.outer_size(), Vec2::new(140.0, 210.0));
    }

    #[test]
    fn aspect_follows_the_max_width_clamp() {
        let tree = boxed(0.0, 0.0)
            .width(900.0)
            .max_width(400.0)
            .aspect(16.0 / 9.0);
        assert_eq!(tree.outer_size(), Vec2::new(400.0, 225.0));
    }

    #[test]
    fn explicit_width_and_height_beat_the_ratio() {
        let tree = boxed(0.0, 0.0).width(100.0).height(10.0).aspect(2.0);
        assert_eq!(tree.outer_size(), Vec2::new(100.0, 10.0));
    }

    #[test]
    fn aspect_item_that_grows_in_a_row_takes_its_height_from_the_final_width() {
        let out = rects(
            row([boxed(0.0, 0.0).grow(1.0).aspect(2.0)]).align(Align::Start),
            bounds(200.0, 400.0),
        );
        assert_eq!(out[0].size(), Vec2::new(200.0, 100.0));
    }

    #[test]
    fn aspect_item_in_a_stretching_column_is_not_stretched_taller_than_its_ratio() {
        let out = rects(
            column([boxed(0.0, 0.0).width(100.0).aspect(2.0)]).align(Align::Stretch),
            bounds(300.0, 400.0),
        );
        assert_eq!(out[0].size(), Vec2::new(100.0, 50.0));
    }

    #[test]
    fn percent_width_fills_the_content_box_of_a_stack() {
        let placed = stack(Align2::LEFT_TOP, [boxed(10.0, 10.0).width_pct(1.0).id(0)])
            .padding(Insets::all(10.0))
            .solve(bounds(200.0, 100.0));
        let rect = placed.find(0).unwrap();
        assert_eq!(rect.min, Pos2::new(10.0, 10.0));
        assert_eq!(rect.width(), 180.0);
        assert_eq!(rect.height(), 10.0);
    }

    #[test]
    fn percent_height_fills_the_content_box_of_a_stack() {
        let placed = stack(Align2::LEFT_TOP, [boxed(10.0, 10.0).height_pct(1.0).id(0)])
            .padding(Insets::all(10.0))
            .solve(bounds(200.0, 100.0));
        assert_eq!(placed.find(0).unwrap().height(), 80.0);
    }

    #[test]
    fn percent_width_in_a_column_beats_start_alignment() {
        let out = rects(
            column([boxed(10.0, 10.0).width_pct(0.5)]).align(Align::Start),
            bounds(200.0, 100.0),
        );
        assert_eq!(out[0].width(), 100.0);
    }

    #[test]
    fn percent_height_in_a_column_is_a_share_of_the_main_axis() {
        let out = rects(
            column([boxed(10.0, 10.0).height_pct(0.25), boxed(10.0, 10.0)]).gap(0.0),
            bounds(100.0, 200.0),
        );
        assert_eq!(out[0].height(), 50.0);
        assert_eq!(out[1].top(), 50.0);
    }

    #[test]
    fn percent_width_in_a_row_is_a_share_of_the_main_axis() {
        let out = rects(
            row([boxed(10.0, 10.0).width_pct(1.0)]).padding(Insets::xy(20.0, 0.0)),
            bounds(200.0, 10.0),
        );
        assert_eq!(out[0].left(), 20.0);
        assert_eq!(out[0].width(), 160.0);
    }

    #[test]
    fn percent_height_in_a_row_is_a_share_of_the_cross_axis() {
        let out = rects(
            row([boxed(10.0, 10.0).height_pct(1.0)]).align(Align::Start),
            bounds(100.0, 60.0),
        );
        assert_eq!(out[0].height(), 60.0);
    }

    #[test]
    fn percent_size_leaves_room_for_the_childs_margin() {
        let out = rects(
            column([boxed(10.0, 10.0)
                .width_pct(1.0)
                .margin(Insets::xy(5.0, 0.0))]),
            bounds(100.0, 50.0),
        );
        assert_eq!(out[0].left(), 5.0);
        assert_eq!(out[0].width(), 90.0);
    }

    fn calc(text: &str) -> Rel {
        Rel::Calc(
            std::sync::Arc::new(crate::stylesheet::math::Expr::parse(text).unwrap()),
            crate::stylesheet::math::Env::default(),
        )
    }

    #[test]
    fn calc_width_is_measured_from_the_padded_content_box() {
        let placed = stack(
            Align2::LEFT_TOP,
            [boxed(10.0, 10.0).width_rel(calc("calc(100% - 20px)")).id(0)],
        )
        .padding(Insets::all(10.0))
        .solve(bounds(200.0, 100.0));
        assert_eq!(placed.find(0).unwrap().width(), 160.0);
    }

    #[test]
    fn calc_height_in_a_column_resolves_against_the_main_axis() {
        let out = rects(
            column([
                boxed(10.0, 10.0).height_rel(calc("min(50%, 70px)")),
                boxed(10.0, 10.0),
            ])
            .gap(0.0),
            bounds(100.0, 200.0),
        );
        assert_eq!(out[0].height(), 70.0);
        assert_eq!(out[1].top(), 70.0);
    }

    #[test]
    fn calc_sizes_are_recomputed_for_every_solve() {
        let tree = column([boxed(10.0, 10.0).width_rel(calc("calc(50% + 10px)")).id(0)])
            .align(Align::Start);
        let narrow = tree.solve(bounds(100.0, 50.0));
        assert_eq!(narrow.find(0).unwrap().width(), 60.0);
        let wide = tree.solve(bounds(300.0, 50.0));
        assert_eq!(wide.find(0).unwrap().width(), 160.0);
    }

    #[test]
    fn calc_child_does_not_inflate_a_hugging_parent() {
        let tree = row([boxed(30.0, 10.0).width_rel(calc("calc(100% - 4px)")).id(0)]);
        assert_eq!(tree.outer_size(), Vec2::new(30.0, 10.0));
        let placed = tree.solve(bounds(100.0, 10.0));
        assert_eq!(placed.find(0).unwrap().width(), 96.0);
        assert_eq!(tree.outer_size(), Vec2::new(30.0, 10.0));
    }

    #[test]
    fn explicit_percent_basis_ignores_what_the_hugging_parent_contains() {
        let tree = column([])
            .padding(Insets::all(10.0))
            .gap(0.0)
            .percent_of(Vec2::new(180.0, 80.0))
            .with_children([boxed(10.0, 10.0).height_pct(0.5).id(0), boxed(10.0, 20.0)]);
        assert_eq!(tree.outer_size(), Vec2::new(30.0, 80.0));
        let placed = tree.solve(bounds(200.0, 100.0));
        assert_eq!(placed.find(0).unwrap().height(), 40.0);
    }

    #[test]
    fn stack_keeps_children_top_left_by_default() {
        let placed = stack(Align2::LEFT_TOP, [boxed(20.0, 10.0).id(0)]).solve(bounds(100.0, 50.0));
        assert_eq!(placed.find(0).unwrap().min, Pos2::ZERO);
    }

    #[test]
    fn stack_align_and_justify_place_every_child() {
        let placed = stack(Align2::LEFT_TOP, [boxed(20.0, 10.0).id(0)])
            .align(Align::End)
            .justify(Justify::Center)
            .solve(bounds(100.0, 50.0));
        assert_eq!(placed.find(0).unwrap().min, Pos2::new(80.0, 20.0));
    }

    #[test]
    fn stack_align_stretch_widens_children_without_a_width() {
        let placed = stack(
            Align2::LEFT_TOP,
            [boxed(20.0, 10.0).id(0), boxed(20.0, 10.0).width(30.0).id(1)],
        )
        .align(Align::Stretch)
        .solve(bounds(100.0, 50.0));
        assert_eq!(placed.find(0).unwrap().width(), 100.0);
        assert_eq!(placed.find(0).unwrap().height(), 10.0);
        assert_eq!(placed.find(1).unwrap().width(), 30.0);
    }

    #[test]
    fn stack_child_placed_at_a_corner_sits_there() {
        let placed = stack(
            Align2::LEFT_TOP,
            [
                boxed(20.0, 10.0)
                    .placed(Some(Align::End), Some(Align::End))
                    .id(0),
                boxed(20.0, 10.0).id(1),
            ],
        )
        .solve(bounds(100.0, 50.0));
        assert_eq!(placed.find(0).unwrap().max, Pos2::new(100.0, 50.0));
        assert_eq!(placed.find(1).unwrap().min, Pos2::ZERO);
    }

    #[test]
    fn stack_child_placement_beats_the_stack_alignment() {
        let placed = stack(
            Align2::LEFT_TOP,
            [boxed(20.0, 10.0).placed(Some(Align::Start), None).id(0)],
        )
        .align(Align::End)
        .justify(Justify::End)
        .solve(bounds(100.0, 50.0));
        let rect = placed.find(0).unwrap();
        assert_eq!(rect.left(), 0.0);
        assert_eq!(rect.bottom(), 50.0);
    }

    #[test]
    fn wrapping_row_squeezed_inside_a_row_grows_as_tall_as_its_lines() {
        let chips = (0..5).map(|i| boxed(40.0, 20.0).id(i));
        let placed = row([row(chips).wrap().gap(10.0).grow(1.0).id(9)])
            .align(Align::Start)
            .solve(bounds(140.0, 200.0));
        let inner = placed.find(9).unwrap();
        assert_eq!(inner.width(), 140.0);
        assert_eq!(inner.height(), 50.0);
    }

    #[test]
    fn wrapping_row_inside_a_wrapping_row_pushes_the_next_line_down() {
        let chips = (0..5).map(|i| boxed(40.0, 20.0).id(i));
        let outer = row([row(chips).wrap().gap(10.0).id(9), boxed(40.0, 20.0).id(10)])
            .wrap()
            .gap(10.0)
            .align(Align::Start);
        let placed = outer.solve(bounds(140.0, 300.0));
        let inner = placed.find(9).unwrap();
        let after = placed.find(10).unwrap();
        assert!(after.top() >= inner.bottom());
    }

    #[test]
    fn grow_leftover_after_a_max_width_still_honours_justify() {
        let out = rects(
            row([
                fill().max_width(50.0),
                boxed(20.0, 10.0).grow(1.0).max_width(30.0),
            ])
            .justify(Justify::End),
            bounds(200.0, 10.0),
        );
        assert_eq!(out[1].right(), 200.0);
    }

    #[test]
    fn items_shrink_in_proportion_to_their_size_and_stop_at_their_minimum() {
        let out = rects(
            row([
                boxed(300.0, 10.0).min_width(250.0),
                boxed(100.0, 10.0).min_width(0.0),
            ]),
            bounds(300.0, 10.0),
        );
        assert_eq!(out[0].width(), 250.0);
        assert_eq!(out[1].width(), 50.0);
    }

    fn nested(depth: usize, fan: usize) -> Node {
        if depth == 0 {
            return boxed(40.0, 20.0);
        }
        let children = (0..fan).map(|_| nested(depth - 1, fan));
        let node = if depth % 2 == 0 {
            row(children)
        } else {
            column(children)
        };
        node.gap(4.0).padding(Insets::all(2.0))
    }

    #[test]
    fn nested_solve_measures_each_text_once() {
        let calls = std::rc::Rc::new(std::cell::Cell::new(0));
        let counter = calls.clone();
        let probe = Node::new(Kind::Text(Measure(std::rc::Rc::new(move |width| {
            if width.is_none() {
                counter.set(counter.get() + 1);
            }
            Vec2::new(40.0, 12.0)
        }))));
        let mut tree = probe;
        for depth in 0..8 {
            tree = if depth % 2 == 0 {
                row([tree, boxed(10.0, 10.0)])
            } else {
                column([tree, boxed(10.0, 10.0)])
            }
            .gap(4.0)
            .padding(Insets::all(2.0));
        }
        tree.solve(bounds(1280.0, 800.0));
        assert_eq!(calls.get(), 1);
    }

    fn count(node: &Node) -> usize {
        1 + node.children.iter().map(count).sum::<usize>()
    }

    #[test]
    #[ignore]
    fn solve_cost_by_depth() {
        for depth in 2..=9 {
            let tree = nested(depth, 2);
            crate::stats::take();
            let started = std::time::Instant::now();
            let runs = 20;
            for _ in 0..runs {
                tree.solve(bounds(1280.0, 800.0));
            }
            let micros = started.elapsed().as_micros() as f64 / runs as f64;
            let [_, _, _, _, _, _, spans, outers] = crate::stats::take();
            println!(
                "depth={depth} nodes={} solve={micros:.0}us spans={} outers={}",
                count(&tree),
                spans / runs,
                outers / runs
            );
        }
    }
}
