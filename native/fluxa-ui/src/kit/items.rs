use crate::flex::{self, Align, Insets, Justify, Node};
use crate::gfx::{Id, Painter, Pos2, Rect, Sense, Ui, UiBuilder, Vec2};
use crate::kit::Space;

thread_local! {
    static PROBING: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
}

const PROBE_SALT: &str = "fluxa-probe";

pub(crate) fn probing() -> bool {
    PROBING.with(|depth| depth.get() > 0)
}

fn probe_run(ui: &mut Ui, add: impl FnOnce(&mut Ui)) {
    PROBING.with(|depth| depth.set(depth.get() + 1));
    add(ui);
    PROBING.with(|depth| depth.set(depth.get() - 1));
}

enum Paint<'a> {
    Ui(Box<dyn FnOnce(&mut Ui, Rect) + 'a>),
    Shared(Box<dyn FnOnce(&mut Ui, Rect) + 'a>),
    Direct(Box<dyn FnOnce(&Painter, Rect) + 'a>),
}

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
        paint: Some(Paint::Ui(Box::new(paint))),
        children: Vec::new(),
    }
}

pub(crate) fn shared<'a>(size: Vec2, paint: impl FnOnce(&mut Ui, Rect) + 'a) -> Item<'a> {
    Item {
        probe: None,
        node: flex::leaf(size),
        vertical: false,
        spacer: None,
        paint: Some(Paint::Shared(Box::new(paint))),
        children: Vec::new(),
    }
}

pub(crate) fn direct<'a>(size: Vec2, paint: impl FnOnce(&Painter, Rect) + 'a) -> Item<'a> {
    Item {
        probe: None,
        node: flex::leaf(size),
        vertical: false,
        spacer: None,
        paint: Some(Paint::Direct(Box::new(paint))),
        children: Vec::new(),
    }
}

pub(crate) fn widget<'a>(ui: &mut Ui, mut add: impl FnMut(&mut Ui) + 'a) -> Item<'a> {
    let mut probe = ui.new_child(
        UiBuilder::new()
            .id_salt(PROBE_SALT)
            .max_rect(ui.available_rect_before_wrap())
            .sizing_pass()
            .invisible(),
    );
    probe_run(&mut probe, &mut add);
    let size = probe.min_rect().size();
    item(size, move |ui, _| add(ui))
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

pub(crate) fn stack<'a>(children: Vec<Item<'a>>) -> Item<'a> {
    Item {
        probe: None,
        node: flex::stack(crate::gfx::Align2::LEFT_TOP, []),
        vertical: false,
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

    pub fn painted_shared(mut self, paint: impl FnOnce(&mut Ui, Rect) + 'a) -> Self {
        if self.paint.is_none() {
            self.paint = Some(Paint::Shared(Box::new(paint)));
        }
        self
    }

    pub fn painted(mut self, paint: impl FnOnce(&mut Ui, Rect) + 'a) -> Self {
        if self.paint.is_none() {
            self.paint = Some(Paint::Ui(Box::new(paint)));
        }
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

    pub fn pan(self, key: u64) -> Self {
        self.with(|node| node.pan(key))
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
    lay_rect(ui, bounds, root, false, false)
}

fn lay_rect(ui: &mut Ui, bounds: Rect, root: Item<'_>, hug_height: bool, hug_width: bool) -> Rect {
    lay_placed(ui, bounds, root, hug_height, hug_width, None, false)
}

pub(crate) fn lay_point(ui: &mut Ui, view: Rect, point: Pos2, root: Item<'_>) -> Rect {
    lay_placed(ui, view, root, true, true, Some(point), false)
}

fn lay_placed(
    ui: &mut Ui,
    mut bounds: Rect,
    root: Item<'_>,
    hug_height: bool,
    hug_width: bool,
    point: Option<Pos2>,
    overflow: bool,
) -> Rect {
    let mut paints = Vec::new();
    let mut probes = Vec::new();
    let tree = build(root, false, &mut paints, &mut probes);
    for (size, add) in probes {
        let mut probe = ui.new_child(
            UiBuilder::new()
                .id_salt(PROBE_SALT)
                .max_rect(bounds)
                .sizing_pass()
                .invisible(),
        );
        probe_run(&mut probe, |ui| (add.borrow_mut())(ui));
        size.set(probe.min_rect().size());
    }
    if let Some(point) = point {
        let size = tree.outer_size();
        let min = Pos2::new(
            point.x.min(bounds.right() - size.x).max(bounds.left()),
            point.y.min(bounds.bottom() - size.y).max(bounds.top()),
        );
        bounds = Rect::from_min_size(min, size);
    }
    if hug_height {
        bounds.max.y = bounds.min.y + tree.outer_size().y;
    }
    if overflow && !tree.grows() {
        bounds.max.x = f32::INFINITY;
    }
    if hug_width && !tree.grows() {
        bounds.max.x = bounds.max.x.min(bounds.min.x + tree.outer_size().x);
    }
    let placed = tree.solve(bounds);
    let mut spots = std::collections::HashMap::new();
    spot(ui, &placed, Vec2::ZERO, None, &mut spots);
    for (index, paint) in paints.into_iter().enumerate() {
        let Some(&(rect, clip)) = spots.get(&index) else {
            continue;
        };
        match paint {
            Paint::Ui(paint) => {
                let mut child = ui.new_child(UiBuilder::new().max_rect(rect));
                if let Some(clip) = clip {
                    child.set_clip_rect(clip.intersect(child.clip_rect()));
                }
                paint(&mut child, rect);
                ui.advance_cursor_after_rect(child.min_rect());
            }
            Paint::Shared(paint) => match clip {
                Some(clip) => {
                    let before = ui.clip_rect();
                    ui.set_clip_rect(before.intersect(clip));
                    paint(ui, rect);
                    ui.set_clip_rect(before);
                }
                None => paint(ui, rect),
            },
            Paint::Direct(paint) => {
                match clip {
                    Some(clip) => paint(
                        &ui.painter().with_clip_rect(clip.intersect(ui.clip_rect())),
                        rect,
                    ),
                    None => paint(ui.painter(), rect),
                }
                ui.advance_cursor_after_rect(rect);
            }
        }
    }
    placed.rect
}

fn spot(
    ui: &mut Ui,
    placed: &flex::Placed,
    shift: Vec2,
    clip: Option<Rect>,
    spots: &mut std::collections::HashMap<usize, (Rect, Option<Rect>)>,
) {
    let (shift, clip) = match placed.pan {
        Some(pan) => {
            let offset = pan_offset(ui, &pan, clip);
            (
                shift - Vec2::new(offset, 0.0),
                Some(clip.map_or(columns(pan.viewport), |outer| {
                    outer.intersect(columns(pan.viewport.translate(shift)))
                })),
            )
        }
        None => (shift, clip),
    };
    if let Some(id) = placed.id {
        spots
            .entry(id)
            .or_insert((placed.rect.translate(shift), clip));
    }
    for child in &placed.children {
        spot(ui, child, shift, clip, spots);
    }
}

fn columns(rect: Rect) -> Rect {
    Rect::from_x_y_ranges(rect.x_range(), f32::MIN / 2.0..=f32::MAX / 2.0)
}

fn pan_offset(ui: &mut Ui, pan: &flex::Pan, clip: Option<Rect>) -> f32 {
    let id = Id::new(("pan-x", pan.key));
    let max = (pan.content - pan.viewport.width()).max(0.0);
    let mut offset = ui.data(|data| data.get_temp::<f32>(id)).unwrap_or(0.0);
    let area = clip.map_or(pan.viewport, |outer| outer.intersect(pan.viewport));
    let response = ui.interact(area, id.with("drag"), Sense::drag());
    if response.dragged() {
        offset -= response.drag_delta().x;
    }
    if response.hovered() {
        offset -= ui.input(|input| input.smooth_scroll_delta().x);
    }
    let offset = offset.clamp(0.0, max);
    ui.data_mut(|data| data.insert_temp(id, offset));
    offset
}

pub(crate) fn lay(ui: &mut Ui, children: Vec<Item<'_>>) -> Rect {
    let bounds = ui.available_rect_before_wrap();
    let spacing = ui.item_spacing();
    let root = if ui.layout().is_horizontal() {
        row(children).gap(spacing.x)
    } else {
        column(children).gap(spacing.y)
    };
    let used = lay_rect(ui, bounds, root, true, ui.layout().is_horizontal());
    ui.allocate_rect(used, Sense::hover());
    used
}

pub(crate) fn lay_hug(ui: &mut Ui, root: Item<'_>) -> Rect {
    let bounds = ui.available_rect_before_wrap();
    let used = lay_rect(ui, bounds, root, true, true);
    ui.allocate_rect(used, Sense::hover());
    used
}

pub(crate) fn lay_overflow(ui: &mut Ui, root: Item<'_>) -> Rect {
    let bounds = ui.available_rect_before_wrap();
    let used = lay_placed(ui, bounds, root, true, true, None, true);
    ui.allocate_rect(used, Sense::hover());
    used
}
