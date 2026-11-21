use super::*;

pub(super) struct CardEntry {
    pub(super) parent: Arc<Vec<Resolved>>,
    pub(super) sig: CardSig,
    pub(super) origin: Pos2,
    pub(super) clip: Rect,
    pub(super) shapes: Vec<crate::gfx::ClippedShape>,
    pub(super) registers: Vec<(u64, Rect)>,
    pub(super) states: Vec<(String, Rect, u8)>,
    pub(super) claims: Vec<(String, SkinTarget)>,
    pub(super) touches: Vec<crate::capture::Touch>,
}

#[derive(Clone, PartialEq)]
pub(super) struct CardSig {
    pub(super) size: [u32; 2],
    pub(super) ppp: u32,
    pub(super) metrics: Option<crate::metrics::MetricsKey>,
    pub(super) generation: u64,
    pub(super) epoch: u64,
    pub(super) opacity: u32,
}

pub(super) const CARD_CACHE_LIMIT: usize = 512;

thread_local! {
    static CARDS: RefCell<HashMap<(usize, usize), std::rc::Rc<CardEntry>, crate::fast::Build>> =
        RefCell::new(HashMap::default());
}

pub(super) fn card_sig(ui: &Ui, size: Vec2) -> CardSig {
    CardSig {
        size: [size.x.to_bits(), size.y.to_bits()],
        ppp: ui.ctx().pixels_per_point().to_bits(),
        metrics: crate::metrics::active_key(),
        generation: crate::stylesheet::generation(),
        epoch: crate::capture::epoch(),
        opacity: ui.opacity().to_bits(),
    }
}

pub(super) fn key_hash(key: &str) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = crate::FastHasher::default();
    key.hash(&mut hasher);
    hasher.finish()
}

pub(super) fn fully_in_view(clip: Rect, top: f32, height: f32) -> bool {
    top >= clip.top() && top + height <= clip.bottom()
}

pub(super) fn card_busy(ctx: &crate::gfx::Context, states: &[(String, Rect, u8)]) -> bool {
    let pointer = hover_pos(ctx);
    states.iter().any(|(key, _, _)| {
        ctx.data(|data| data.get_temp::<(Rect, u8)>(Id::new(("skin-state", key))))
            .is_some_and(|(rect, flags)| {
                flags & (crate::stylesheet::FOCUSED | crate::stylesheet::PRESSED) != 0
                    || pointer.is_some_and(|pos| rect.contains(pos))
            })
    })
}

pub(super) fn cached_card<'a, A: HomeAssets>(
    ui: &mut Ui,
    parent: &Arc<Vec<Resolved>>,
    index: usize,
    node: &Resolved,
    size: Vec2,
    env: &'a Env<'a, A>,
) -> kit::Item<'a> {
    let slot = (Arc::as_ptr(parent) as usize, index);
    let sig = card_sig(ui, size);
    let found = CARDS.with(|cards| cards.borrow().get(&slot).cloned());
    if let Some(entry) = found
        && entry.sig == sig
        && Arc::ptr_eq(&entry.parent, parent)
        && !card_busy(ui.ctx(), &entry.states)
        && entry.touches.iter().all(|touch| {
            env.assets
                .borrow_mut()
                .texture_for(touch.url.as_deref(), touch.size, touch.priority)
                == Some(touch.id)
        })
    {
        for (key, target) in &entry.claims {
            env.claim_target(key, target.clone());
        }
        CARD_HITS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        return kit::shared(size, move |ui, rect| replay_card(ui, rect, &entry));
    }
    CARD_MISSES.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    crate::capture::begin();
    let item = build(ui, node, env);
    let built = crate::capture::end();
    let parent = parent.clone();
    kit::shared(size, move |ui, rect| {
        let ctx = ui.ctx().clone();
        let layer = ui.layer_id();
        let start = ctx.layer_cursor(layer);
        let clip = ui.clip_rect();
        crate::capture::begin();
        kit::lay_in(ui, rect, item);
        let painted = crate::capture::end();
        if built.dirty
            || painted.dirty
            || card_busy(&ctx, &painted.states)
            || !fully_in_view(clip, rect.top(), rect.height())
        {
            return;
        }
        let mut touches = built.touches;
        touches.extend(painted.touches);
        let entry = CardEntry {
            parent,
            sig,
            origin: rect.min,
            clip,
            shapes: ctx.layer_shapes_since(layer, start),
            registers: painted.registers,
            states: painted.states,
            claims: built.claims,
            touches,
        };
        CARDS.with(|cards| {
            let mut cards = cards.borrow_mut();
            if cards.len() >= CARD_CACHE_LIMIT {
                cards.clear();
            }
            cards.insert(slot, std::rc::Rc::new(entry));
        });
    })
}

pub(super) fn replay_shapes(
    ui: &mut Ui,
    at: Pos2,
    origin: Pos2,
    clip: Rect,
    shapes: &[crate::gfx::ClippedShape],
) -> Vec2 {
    let delta = at - origin;
    let transform = crate::gfx::TSTransform::from_translation(delta);
    let current = ui.clip_rect();
    let moved = shapes
        .iter()
        .map(|shape| {
            let outer = shape.clip_rect == clip;
            let mut shape = shape.clone();
            shape.transform(transform);
            if outer {
                shape.clip_rect = current;
            } else {
                shape.clip_rect = shape.clip_rect.intersect(current);
            }
            shape
        })
        .collect();
    ui.ctx().layer_add_shapes(ui.layer_id(), moved);
    delta
}

pub(super) fn replay_card(ui: &mut Ui, rect: Rect, entry: &CardEntry) {
    let ctx = ui.ctx().clone();
    let delta = replay_shapes(ui, rect.min, entry.origin, entry.clip, &entry.shapes);
    for (key, state, flags) in &entry.states {
        let state = state.translate(delta);
        ctx.data_mut(|data| data.insert_temp(Id::new(("skin-state", key)), (state, *flags)));
        crate::capture::note_state(key, state, *flags);
    }
    for (node, at) in &entry.registers {
        activate(ui, at.translate(delta), Some(*node));
    }
    crate::capture::note_touches(&entry.touches);
}

pub(super) struct StripEntry {
    pub(super) parent: Arc<Vec<Resolved>>,
    pub(super) sig: StripSig,
    pub(super) origin: Pos2,
    pub(super) clip: Rect,
    pub(super) used: Vec2,
    pub(super) shapes: Vec<crate::gfx::ClippedShape>,
    pub(super) registers: Vec<(u64, Rect)>,
    pub(super) states: Vec<(String, Rect, u8)>,
    pub(super) claims: Vec<(String, SkinTarget)>,
    pub(super) touches: Vec<crate::capture::Touch>,
    pub(super) regions: Vec<crate::ScrollRegion>,
}

#[derive(Clone, PartialEq)]
pub(super) struct StripSig {
    pub(super) card: CardSig,
    pub(super) shown: u32,
    pub(super) reach: u32,
}

thread_local! {
    pub(super) static STRIPS: RefCell<HashMap<usize, std::rc::Rc<StripEntry>, crate::fast::Build>> =
        RefCell::new(HashMap::default());
}

pub(super) fn cached_strip<A: HomeAssets>(
    ui: &mut Ui,
    parent: &Arc<Vec<Resolved>>,
    sig: &StripSig,
    env: &Env<'_, A>,
) -> bool {
    let slot = Arc::as_ptr(parent) as usize;
    let Some(entry) = STRIPS.with(|strips| strips.borrow().get(&slot).cloned()) else {
        return false;
    };
    if entry.sig != *sig
        || !Arc::ptr_eq(&entry.parent, parent)
        || card_busy(ui.ctx(), &entry.states)
        || !entry.touches.iter().all(|touch| {
            env.assets
                .borrow_mut()
                .texture_for(touch.url.as_deref(), touch.size, touch.priority)
                == Some(touch.id)
        })
    {
        return false;
    }
    let ctx = ui.ctx().clone();
    let rect = ui.max_rect();
    for (key, target) in &entry.claims {
        env.claim_target(key, target.clone());
    }
    let delta = replay_shapes(ui, rect.min, entry.origin, entry.clip, &entry.shapes);
    for (key, state, flags) in &entry.states {
        let state = state.translate(delta);
        ctx.data_mut(|data| data.insert_temp(Id::new(("skin-state", key)), (state, *flags)));
        crate::capture::note_state(key, state, *flags);
    }
    for (node, at) in &entry.registers {
        activate(ui, at.translate(delta), Some(*node));
    }
    for region in &entry.regions {
        crate::report_scroll_region(
            &ctx,
            crate::ScrollRegion {
                rect: region.rect.translate(delta),
                ..*region
            },
        );
    }
    crate::capture::note_touches(&entry.touches);
    ui.allocate_exact_size(entry.used, Sense::hover());
    CARD_HITS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    true
}
