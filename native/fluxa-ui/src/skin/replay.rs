use super::*;
use std::rc::Rc;

thread_local! {
    static COUNTS: Cell<(u64, u64, u64)> = const { Cell::new((0, 0, 0)) };
}

fn count(apply: impl FnOnce(&mut (u64, u64, u64))) {
    COUNTS.with(|counts| {
        let mut value = counts.get();
        apply(&mut value);
        counts.set(value);
    });
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Mode {
    Off,
    On,
    Verify,
}

#[cfg(test)]
thread_local! {
    static FORCED: std::cell::Cell<Option<Mode>> = const { std::cell::Cell::new(None) };
}

#[cfg(test)]
pub(super) fn force_mode(mode: Option<Mode>) {
    FORCED.with(|forced| forced.set(mode));
}

pub(super) fn mode() -> Mode {
    #[cfg(test)]
    if let Some(forced) = FORCED.with(std::cell::Cell::get) {
        return forced;
    }
    static MODE: std::sync::OnceLock<Mode> = std::sync::OnceLock::new();
    *MODE.get_or_init(|| match std::env::var("FLUXA_FRAME_SKIP").as_deref() {
        Ok("1") => Mode::On,
        Ok("verify") => Mode::Verify,
        _ => Mode::Off,
    })
}

#[derive(Clone, PartialEq)]
pub(super) struct FrameSig {
    pub(super) card: CardSig,
    pub(super) tree: usize,
    pub(super) viewport: u64,
    pub(super) opaque: bool,
    pub(super) first_slot: u64,
    pub(super) origin: [u32; 2],
    pub(super) clip: [u32; 4],
    pub(super) globals: u64,
    pub(super) auto: (bool, u64),
    pub(super) keyboard: Option<Id>,
    pub(super) latest: Option<[u32; 2]>,
    pub(super) hover: Option<[u32; 2]>,
}

struct Entry {
    tree: Arc<Vec<Resolved>>,
    sig: FrameSig,
    shapes: Vec<crate::gfx::ClippedShape>,
    used: Rect,
    widgets: Vec<crate::capture::Widget>,
    focus: Vec<crate::capture::Focus>,
    cursors: Vec<crate::gfx::CursorIcon>,
    regions: Vec<crate::ScrollRegion>,
    touches: Vec<crate::capture::Touch>,
    targets: Vec<(u64, SkinTarget)>,
    fields: Vec<(u64, InputKind)>,
    layout: HomeLayout,
}

thread_local! {
    static ENTRIES: RefCell<HashMap<u64, Rc<Entry>, crate::fast::Build>> =
        RefCell::new(HashMap::default());
}

fn bits(pos: Pos2) -> [u32; 2] {
    [pos.x.to_bits(), pos.y.to_bits()]
}

fn rect_bits(rect: Rect) -> [u32; 4] {
    [
        rect.min.x.to_bits(),
        rect.min.y.to_bits(),
        rect.max.x.to_bits(),
        rect.max.y.to_bits(),
    ]
}

pub(super) fn frame_sig(
    ui: &Ui,
    tree: &Arc<Vec<Resolved>>,
    viewport: Viewport,
    opaque: bool,
    first_slot: u64,
) -> FrameSig {
    let ctx = ui.ctx();
    let size = Vec2::new(viewport.width, viewport.height);
    FrameSig {
        card: card_sig(ui, size),
        tree: Arc::as_ptr(tree) as usize,
        viewport: crate::fast::hash(format!("{viewport:?}")),
        opaque,
        first_slot,
        origin: bits(ui.max_rect().min),
        clip: rect_bits(ui.clip_rect()),
        globals: crate::inputs::fingerprint(ctx),
        auto: crate::interact::auto_state(ctx),
        keyboard: ctx.keyboard_focus(),
        latest: ctx.pointer_latest_pos().map(bits),
        hover: hover_pos(ctx).map(bits),
    }
}

fn layout_quiet(layout: &HomeLayout) -> bool {
    layout.activated.is_none()
        && layout.text_input.is_none()
        && layout.text_input_node.is_none()
        && layout.setting_change.is_none()
        && layout.filter_change.is_none()
        && layout.load_more.is_empty()
        && layout.seek_to.is_none()
        && layout.player_gesture.is_none()
        && layout.profiles.is_none()
        && layout.welcome.is_none()
}

fn textures_match<A: HomeAssets>(entry: &Entry, env: &Env<'_, A>) -> bool {
    entry.touches.iter().all(|touch| {
        env.assets
            .borrow_mut()
            .texture_for(touch.url.as_deref(), touch.size, touch.priority)
            == Some(touch.id)
    })
}

pub(super) struct Session {
    slot: u64,
    sig: FrameSig,
    tree: Arc<Vec<Resolved>>,
    layer: crate::gfx::LayerId,
    start: crate::gfx::ShapeIdx,
    candidate: Option<Rc<Entry>>,
}

pub(super) enum Plan {
    Off,
    Replayed,
    Build(Session),
}

pub(super) fn begin<A: HomeAssets>(
    ui: &mut Ui,
    env: &Env<'_, A>,
    tree: &Arc<Vec<Resolved>>,
    viewport: Viewport,
    opaque: bool,
    first_slot: u64,
) -> Plan {
    let active = mode();
    if active == Mode::Off || !ui.ctx().input_is_quiet() {
        return Plan::Off;
    }
    let sig = frame_sig(ui, tree, viewport, opaque, first_slot);
    let slot = crate::fast::hash((opaque, first_slot));
    let found = ENTRIES
        .with(|entries| entries.borrow().get(&slot).cloned())
        .filter(|entry| {
            entry.sig == sig && Arc::ptr_eq(&entry.tree, tree) && textures_match(entry, env)
        });
    if let Some(entry) = &found
        && active == Mode::On
    {
        apply(ui, env, entry);
        count(|counts| counts.0 += 1);
        return Plan::Replayed;
    }
    crate::capture::begin();
    let layer = ui.layer_id();
    Plan::Build(Session {
        slot,
        sig,
        tree: tree.clone(),
        layer,
        start: ui.ctx().layer_cursor(layer),
        candidate: found,
    })
}

impl Session {
    pub(super) fn finish<A: HomeAssets>(self, ui: &mut Ui, env: &Env<'_, A>) {
        let log = crate::capture::end();
        count(|counts| counts.1 += 1);
        let layout = env.layout.borrow().clone();
        if log.dirty || log.strict || !layout_quiet(&layout) {
            ENTRIES.with(|entries| entries.borrow_mut().remove(&self.slot));
            return;
        }
        let entry = Entry {
            tree: self.tree,
            sig: self.sig,
            shapes: ui.ctx().layer_shapes_since(self.layer, self.start),
            used: ui.min_rect(),
            widgets: log.widgets,
            focus: log.focus,
            cursors: log.cursors,
            regions: log.regions,
            touches: log.touches,
            targets: env.targets.borrow().clone(),
            fields: env.fields.borrow().clone(),
            layout,
        };
        if let Some(previous) = &self.candidate
            && !same_output(previous, &entry)
        {
            count(|counts| counts.2 += 1);
        }
        ENTRIES.with(|entries| entries.borrow_mut().insert(self.slot, Rc::new(entry)));
    }
}

fn same_output(a: &Entry, b: &Entry) -> bool {
    a.shapes == b.shapes
        && a.used == b.used
        && a.widgets.len() == b.widgets.len()
        && a.widgets.iter().zip(&b.widgets).all(|(x, y)| {
            x.rect == y.rect && x.id == y.id && x.sense == y.sense && x.clip == y.clip
        })
        && a.focus.len() == b.focus.len()
        && a.focus.iter().zip(&b.focus).all(|(x, y)| {
            (x.node, x.rect, x.clicked, x.auto) == (y.node, y.rect, y.clicked, y.auto)
        })
        && a.cursors == b.cursors
        && format!("{:?}{:?}{:?}", a.regions, a.targets.len(), a.layout)
            == format!("{:?}{:?}{:?}", b.regions, b.targets.len(), b.layout)
}

fn apply<A: HomeAssets>(ui: &mut Ui, env: &Env<'_, A>, entry: &Entry) {
    *env.targets.borrow_mut() = entry.targets.clone();
    *env.fields.borrow_mut() = entry.fields.clone();
    *env.layout.borrow_mut() = entry.layout.clone();
    let origin = ui.max_rect().min;
    replay_shapes(ui, origin, origin, rect_of(entry.sig.clip), &entry.shapes);
    for widget in &entry.widgets {
        ui.replay_widget(widget);
    }
    for focus in &entry.focus {
        crate::interact::replay_focus(ui, focus);
    }
    let ctx = ui.ctx().clone();
    for icon in &entry.cursors {
        ctx.replay_cursor(*icon);
    }
    for region in &entry.regions {
        crate::report_scroll_region(&ctx, *region);
    }
    crate::capture::note_touches(&entry.touches);
    ui.advance_cursor_after_rect(entry.used);
}

fn rect_of(bits: [u32; 4]) -> Rect {
    Rect::from_min_max(
        Pos2::new(f32::from_bits(bits[0]), f32::from_bits(bits[1])),
        Pos2::new(f32::from_bits(bits[2]), f32::from_bits(bits[3])),
    )
}

pub(super) fn skipped_builds() -> (u64, u64, u64) {
    COUNTS.with(Cell::get)
}

pub fn take_skips() -> (u64, u64, u64) {
    COUNTS.with(|counts| counts.replace((0, 0, 0)))
}

pub(crate) fn forget() {
    ENTRIES.with(|entries| entries.borrow_mut().clear());
}
