use crate::flex::{self, Align, Insets, Justify, Node};
use crate::gfx::{Rect, Sense, Ui, UiBuilder, Vec2};
use crate::kit::Space;

thread_local! {
    static PROBING: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
}

pub(crate) fn probing() -> bool {
    PROBING.with(|depth| depth.get() > 0)
}

fn probe_run(ui: &mut Ui, add: impl FnOnce(&mut Ui)) {
    PROBING.with(|depth| depth.set(depth.get() + 1));
    add(ui);
    PROBING.with(|depth| depth.set(depth.get() - 1));
}

type Paint<'a> = Box<dyn FnOnce(&mut Ui, Rect) + 'a>;

type Probe<'a> = (
    std::rc::Rc<std::cell::Cell<Vec2>>,
    std::rc::Rc<std::cell::RefCell<dyn FnMut(&mut Ui) + 'a>>,
);

pub(crate) struct Item<'a> {
    probe: Option<Probe<'a>>,
    node: Node,
    vertical: bool,
    spacer: Option<f32>,
    paint: Option<Paint<'a>>,
    children: Vec<Item<'a>>,
}

pub(crate) fn item<'a>(size: Vec2, paint: impl FnOnce(&mut Ui, Rect) + 'a) -> Item<'a> {
    Item {
        probe: None,
        node: flex::leaf(size),
        vertical: false,
        spacer: None,
        paint: Some(Box::new(paint)),
        children: Vec::new(),
    }
}

pub(crate) fn widget<'a>(ui: &mut Ui, mut add: impl FnMut(&mut Ui) + 'a) -> Item<'a> {
    let mut probe = ui.new_child(
        UiBuilder::new()
            .max_rect(ui.available_rect_before_wrap())
            .sizing_pass()
            .invisible(),
    );
    probe_run(&mut probe, &mut add);
    let size = probe.min_rect().size();
    item(size, move |ui, _| add(ui))
}

pub(crate) fn lazy<'a>(add: impl FnMut(&mut Ui) + 'a) -> Item<'a> {
    let add: std::rc::Rc<std::cell::RefCell<dyn FnMut(&mut Ui) + 'a>> =
        std::rc::Rc::new(std::cell::RefCell::new(add));
    let size = std::rc::Rc::new(std::cell::Cell::new(Vec2::ZERO));
    let reader = size.clone();
    let painter = add.clone();
    let mut built = item(Vec2::ZERO, move |ui, _| (painter.borrow_mut())(ui));
    built.node = flex::measured(move |_| reader.get());
    built.probe = Some((size, add));
    built
}

pub(crate) fn gap<'a>(px: f32) -> Item<'a> {
    Item {
        probe: None,
        node: flex::leaf(Vec2::ZERO).shrink(0.0),
        vertical: false,
        spacer: Some(px),
        paint: None,
        children: Vec::new(),
    }
}

pub(crate) fn space<'a>(ui: &Ui, space: Space) -> Item<'a> {
    use crate::kit::UiKit;
    gap(space.px(&ui.tokens()))
}

pub(crate) fn fill<'a>() -> Item<'a> {
    Item {
        probe: None,
        node: flex::fill(),
        vertical: false,
        spacer: None,
        paint: None,
        children: Vec::new(),
    }
}

pub(crate) fn row<'a>(children: Vec<Item<'a>>) -> Item<'a> {
    Item {
        probe: None,
        node: flex::row([]),
        vertical: false,
        spacer: None,
        paint: None,
        children,
    }
}

pub(crate) fn column<'a>(children: Vec<Item<'a>>) -> Item<'a> {
    Item {
        probe: None,
        node: flex::column([]),
        vertical: true,
        spacer: None,
        paint: None,
        children,
    }
}

impl<'a> Item<'a> {
    fn with(mut self, apply: impl FnOnce(Node) -> Node) -> Self {
        self.node = apply(self.node);
        self
    }

    pub fn width(self, value: f32) -> Self {
        self.with(|node| node.width(value))
    }

    pub fn height(self, value: f32) -> Self {
        self.with(|node| node.height(value))
    }

    pub fn min_width(self, value: f32) -> Self {
        self.with(|node| node.min_width(value))
    }

    pub fn max_width(self, value: f32) -> Self {
        self.with(|node| node.max_width(value))
    }

    pub fn grow(self, value: f32) -> Self {
        self.with(|node| node.grow(value))
    }

    pub fn shrink(self, value: f32) -> Self {
        self.with(|node| node.shrink(value))
    }

    pub fn basis(self, value: f32) -> Self {
        self.with(|node| node.basis(value))
    }

    pub fn margin(self, value: Insets) -> Self {
        self.with(|node| node.margin(value))
    }

    pub fn padding(self, value: Insets) -> Self {
        self.with(|node| node.padding(value))
    }

    pub fn gap(self, value: f32) -> Self {
        self.with(|node| node.gap(value))
    }

    pub fn align(self, value: Align) -> Self {
        self.with(|node| node.align(value))
    }

    pub fn align_self(self, value: Align) -> Self {
        self.with(|node| node.align_self(value))
    }

    pub fn justify(self, value: Justify) -> Self {
        self.with(|node| node.justify(value))
    }

    pub fn wrap(self) -> Self {
        self.with(Node::wrap)
    }
}

fn build<'a>(
    item: Item<'a>,
    parent_vertical: bool,
    paints: &mut Vec<Paint<'a>>,
    probes: &mut Vec<Probe<'a>>,
) -> Node {
    let mut node = item.node;
    if let Some(probe) = item.probe {
        probes.push(probe);
    }
    if let Some(px) = item.spacer {
        node = if parent_vertical {
            node.height(px)
        } else {
            node.width(px)
        };
    }
    if let Some(paint) = item.paint {
        node = node.id(paints.len());
        paints.push(paint);
    }
    if item.children.is_empty() {
        return node;
    }
    let children: Vec<Node> = item
        .children
        .into_iter()
        .map(|child| build(child, item.vertical, paints, probes))
        .collect();
    node.with_children(children)
}

pub(crate) fn lay_in(ui: &mut Ui, bounds: Rect, root: Item<'_>) -> Rect {
    lay_rect(ui, bounds, root, false)
}

fn lay_rect(ui: &mut Ui, mut bounds: Rect, root: Item<'_>, hug_height: bool) -> Rect {
    let mut paints = Vec::new();
    let mut probes = Vec::new();
    let tree = build(root, false, &mut paints, &mut probes);
    for (size, add) in probes {
        let mut probe = ui.new_child(UiBuilder::new().max_rect(bounds).sizing_pass().invisible());
        probe_run(&mut probe, |ui| (add.borrow_mut())(ui));
        size.set(probe.min_rect().size());
    }
    if hug_height {
        bounds.max.y = bounds.min.y + tree.outer_size().y;
    }
    let placed = tree.solve(bounds);
    for (index, paint) in paints.into_iter().enumerate() {
        let Some(rect) = placed.find(index) else {
            continue;
        };
        ui.scope_builder(UiBuilder::new().max_rect(rect), |ui| paint(ui, rect));
    }
    placed.rect
}

pub(crate) fn lay(ui: &mut Ui, children: Vec<Item<'_>>) -> Rect {
    let bounds = ui.available_rect_before_wrap();
    let spacing = ui.item_spacing();
    let root = if ui.layout().is_horizontal() {
        row(children).gap(spacing.x)
    } else {
        column(children).gap(spacing.y)
    };
    let used = lay_rect(ui, bounds, root, true);
    ui.allocate_rect(used, Sense::hover());
    used
}

pub(crate) fn at<R>(ui: &mut Ui, rect: Rect, add: impl FnOnce(&mut Ui) -> R) -> R {
    ui.scope_builder(UiBuilder::new().max_rect(rect), add).inner
}
