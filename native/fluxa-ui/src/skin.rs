use super::*;
use crate::components::{ButtonKind, InputKind, texture_image};
use crate::dp::Dp;
use crate::flex::{Align as FlexAlign, Insets, Justify as FlexJustify};
use crate::gfx::Ui;
use crate::kit::{self, Role, Type, UiKit, layout_items};
use fluxa_skin::{Align, Justify, Resolved, ResolvedAction, truthy};
use serde_json::Value;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

static GRID_WINDOWS: Mutex<fluxa_skin::Windows> = Mutex::new(fluxa_skin::Windows::new());

static STRIP_NAMES: Mutex<Option<HashMap<String, &'static str>>> = Mutex::new(None);
static FLOW_KEYS: Mutex<Vec<(usize, String)>> = Mutex::new(Vec::new());

pub fn row_strip(index: usize) -> Option<&'static str> {
    let flow = FLOW_KEYS.lock().ok()?;
    let prefix = format!("{}.", flow.iter().find(|(row, _)| *row == index)?.1);
    let names = STRIP_NAMES.lock().ok()?;
    names
        .as_ref()?
        .iter()
        .find(|(key, _)| key.starts_with(&prefix))
        .map(|(_, name)| *name)
}

fn strip_name(key: &str) -> &'static str {
    let mut names = STRIP_NAMES
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let names = names.get_or_insert_with(HashMap::new);
    if let Some(name) = names.get(key) {
        return name;
    }
    let name: &'static str = Box::leak(format!("fluxa-skin-strip:{key}").into_boxed_str());
    names.insert(key.to_owned(), name);
    name
}

thread_local! {
    static FLOW_NEAR: RefCell<Positions> = RefCell::new(Positions::default());
}

fn flow_owner(key: &str) -> Option<(&str, usize)> {
    let (each, index) = key.rsplit_once('.')?;
    let (owner, _) = each.rsplit_once('.')?;
    Some((owner, index.parse().ok()?))
}

fn flow_report(ctx: &crate::gfx::Context, key: &str, rect: Rect, clip: Rect) {
    let id = Id::new(("skin-flow-height", key));
    if rect.height() > 0.0 {
        ctx.data_mut(|data| {
            data.insert_temp(id, rect.height());
            data.insert_temp(Id::new("skin-flow-estimate"), rect.height());
        });
    }
    let near =
        rect.bottom() > clip.top() - clip.height() && rect.top() < clip.bottom() + clip.height();
    if !near {
        return;
    }
    if let Some((owner, index)) = flow_owner(key) {
        if let Ok(mut keys) = FLOW_KEYS.lock() {
            match keys.iter_mut().find(|(row, _)| *row == index) {
                Some(entry) => entry.1 = key.to_owned(),
                None => keys.push((index, key.to_owned())),
            }
        }
        FLOW_NEAR.with(|near| {
            let mut near = near.borrow_mut();
            let range = near.entry(owner.to_owned()).or_insert((index, index));
            range.0 = range.0.min(index);
            range.1 = range.1.max(index);
        });
    }
}

fn flow_publish(ctx: &crate::gfx::Context) {
    let near = FLOW_NEAR.with(|near| std::mem::take(&mut *near.borrow_mut()));
    let Ok(mut windows) = GRID_WINDOWS.lock() else {
        return;
    };
    for (owner, (first, last)) in near {
        let start = first.saturating_sub(1) / 2 * 2;
        let end = (last + 2).div_ceil(2) * 2;
        if windows.get(&owner) != Some(&(start, end)) {
            windows.insert(owner, (start, end));
            ctx.request_repaint();
        }
    }
}

fn flow_far<'a>(ui: &Ui, node: &Resolved) -> kit::Item<'a> {
    let ctx = ui.ctx().clone();
    let height = ctx
        .data(|data| {
            data.get_temp::<f32>(Id::new(("skin-flow-height", prop(node, "windowed"))))
                .or_else(|| data.get_temp::<f32>(Id::new("skin-flow-estimate")))
        })
        .unwrap_or(240.0);
    let key = prop(node, "windowed").into_owned();
    kit::shared(Vec2::new(0.0, height), move |ui, rect| {
        let clip = ui.clip_rect();
        flow_report(&ctx, &key, rect, clip);
    })
}

pub fn grid_windows() -> fluxa_skin::Windows {
    GRID_WINDOWS
        .lock()
        .map(|windows| windows.clone())
        .unwrap_or_default()
}

#[derive(Clone, Debug, PartialEq)]
pub enum SkinTarget {
    Action(Arc<ResolvedAction>),
    Field(String),
    Control,
}

pub struct SkinFrame {
    pub targets: Vec<(u64, SkinTarget)>,
    pub layout: HomeLayout,
}

static FIELD_KINDS: Mutex<Vec<(u64, InputKind)>> = Mutex::new(Vec::new());

pub(crate) fn field_kind(node: u64) -> Option<InputKind> {
    FIELD_KINDS
        .lock()
        .ok()?
        .iter()
        .find(|(id, _)| *id == node)
        .map(|(_, kind)| *kind)
}

struct Ancestor {
    classes: Vec<std::sync::Arc<str>>,
    key: String,
    state_id: Id,
}

fn class_name(class: &str) -> std::sync::Arc<str> {
    thread_local! {
        static NAMES: RefCell<HashMap<String, std::sync::Arc<str>, crate::fast::Build>> =
            RefCell::new(HashMap::default());
    }
    NAMES.with(|names| {
        let mut names = names.borrow_mut();
        if let Some(found) = names.get(class) {
            return found.clone();
        }
        let name: std::sync::Arc<str> = std::sync::Arc::from(class);
        names.insert(class.to_owned(), name.clone());
        name
    })
}

fn index_positions(nodes: &[Resolved], into: &mut Positions) {
    for (index, node) in nodes.iter().enumerate() {
        into.insert(node.key.clone(), (index, nodes.len()));
        index_positions(&node.children, into);
    }
}

type Positions = HashMap<String, (usize, usize), crate::fast::Build>;

thread_local! {
    static POSITIONS: RefCell<Vec<(Arc<Vec<Resolved>>, Arc<Positions>)>> = const { RefCell::new(Vec::new()) };
}

fn positions_of(tree: &Arc<Vec<Resolved>>) -> Arc<Positions> {
    POSITIONS.with(|cache| {
        let mut cache = cache.borrow_mut();
        if let Some((_, found)) = cache.iter().find(|(known, _)| Arc::ptr_eq(known, tree)) {
            return found.clone();
        }
        let mut positions = Positions::default();
        index_positions(tree, &mut positions);
        let positions = Arc::new(positions);
        if cache.len() >= PREFETCH_PLANS {
            cache.remove(0);
        }
        cache.push((tree.clone(), positions.clone()));
        positions
    })
}

type ClassParts = (&'static str, &'static [&'static str]);

fn class_parts(node: &Resolved, position: Option<(usize, usize)>) -> Option<ClassParts> {
    struct Entry {
        class: Box<str>,
        position: Option<(usize, usize)>,
        parts: Option<ClassParts>,
    }
    #[derive(Default)]
    struct Cache {
        generation: u64,
        entries: HashMap<u64, Vec<Entry>, crate::fast::Build>,
    }
    thread_local! {
        static CACHE: RefCell<Cache> = RefCell::new(Cache::default());
    }
    let class = node.class.as_deref()?;
    let generation = crate::stylesheet::generation();
    let key = crate::fast::hash((class, position));
    CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        if cache.generation != generation {
            cache.entries.clear();
            cache.generation = generation;
        }
        let bucket = cache.entries.entry(key).or_default();
        if let Some(found) = bucket
            .iter()
            .find(|entry| *entry.class == *class && entry.position == position)
        {
            return found.parts;
        }
        let parts = compute_class_parts(class, position);
        bucket.push(Entry {
            class: class.into(),
            position,
            parts,
        });
        parts
    })
}

fn compute_class_parts(class: &str, position: Option<(usize, usize)>) -> Option<ClassParts> {
    let classes: Vec<&str> = class.split_whitespace().collect();
    let index = classes
        .iter()
        .position(|class| crate::stylesheet::has_rule(class))?;
    let mut modifiers: Vec<String> = classes
        .iter()
        .enumerate()
        .filter(|(at, _)| *at != index)
        .map(|(_, class)| (*class).to_owned())
        .collect();
    if let Some((at, count)) = position {
        if at == 0 {
            modifiers.push("first-child".to_owned());
        }
        if at + 1 == count {
            modifiers.push("last-child".to_owned());
        }
        if count == 1 {
            modifiers.push("only-child".to_owned());
        }
        modifiers.push(format!("nth-child({})", at + 1));
        modifiers.push(format!(
            "nth-child({})",
            if at % 2 == 0 { "odd" } else { "even" }
        ));
    }
    let leaked: Vec<&'static str> = modifiers
        .into_iter()
        .map(|modifier| &*Box::leak(modifier.into_boxed_str()))
        .collect();
    Some((
        Box::leak(classes[index].to_owned().into_boxed_str()),
        Box::leak(leaked.into_boxed_slice()),
    ))
}

fn ancestor_states(
    context: &crate::gfx::Context,
    chain: &[std::rc::Rc<Ancestor>],
) -> Vec<(std::sync::Arc<str>, u8)> {
    let pointer = context.pointer_hover_pos();
    let mut found = Vec::new();
    for ancestor in chain {
        let recorded = context.data(|data| data.get_temp::<(Rect, u8)>(ancestor.state_id));
        let mut states = recorded.map_or(0, |(_, flags)| flags);
        if let Some((rect, _)) = recorded
            && pointer.is_some_and(|pos| rect.contains(pos))
        {
            states |= crate::stylesheet::HOVERED;
        }
        for class in &ancestor.classes {
            found.push((std::sync::Arc::clone(class), states));
        }
    }
    found
}

fn paint_box(
    painter: &crate::gfx::Painter,
    rect: Rect,
    radius: f32,
    style: &crate::stylesheet::Style,
) {
    if let Some(glow) = style.shadow
        && glow.color.a() > 0
    {
        painter.add(
            crate::gfx::Shadow {
                offset: [glow.x.round() as i8, glow.y.round() as i8],
                blur: glow.blur.clamp(0.0, 255.0) as u8,
                spread: glow.spread.clamp(0.0, 255.0) as u8,
                color: glow.color.gamma_multiply(style.opacity),
            }
            .as_shape(rect, radius),
        );
    }
    if style.backdrop > 0.0 {
        painter.add(crate::gfx::Shape::Callback(crate::gfx::PaintCallback {
            rect,
            callback: std::sync::Arc::new(crate::Glass {
                radius: radius.min(rect.height() * 0.5).min(rect.width() * 0.5),
                tint: Color32::TRANSPARENT,
                refraction: 0.0,
                bevel: 1.0,
                rim: 0.0,
                frost: (style.backdrop / 16.0).clamp(0.0, 1.0),
            }),
        }));
    }
    match style.gradient {
        Some(gradient) => gradient_fill(painter, rect, radius, gradient, style.opacity),
        None => {
            painter.rect_filled(rect, radius, style.fill.gamma_multiply(style.opacity));
        }
    }
}

fn gradient_fill(
    painter: &crate::gfx::Painter,
    rect: Rect,
    radius: f32,
    gradient: crate::stylesheet::Gradient,
    opacity: f32,
) {
    let radius = radius
        .min(rect.width() / 2.0)
        .min(rect.height() / 2.0)
        .max(0.0);
    let mut outline = Vec::new();
    let corners = [
        (
            Pos2::new(rect.right() - radius, rect.top() + radius),
            -std::f32::consts::FRAC_PI_2,
        ),
        (
            Pos2::new(rect.right() - radius, rect.bottom() - radius),
            0.0,
        ),
        (
            Pos2::new(rect.left() + radius, rect.bottom() - radius),
            std::f32::consts::FRAC_PI_2,
        ),
        (
            Pos2::new(rect.left() + radius, rect.top() + radius),
            std::f32::consts::PI,
        ),
    ];
    let steps = if radius > 0.5 { 10 } else { 1 };
    for (center, start) in corners {
        for step in 0..=steps {
            let angle = start + std::f32::consts::FRAC_PI_2 * step as f32 / steps as f32;
            let normal = Vec2::new(angle.cos(), angle.sin());
            outline.push((center + normal * radius, normal));
        }
    }
    let angle = gradient.angle.to_radians();
    let direction = Vec2::new(angle.sin(), -angle.cos());
    let length = (rect.width() * angle.sin()).abs() + (rect.height() * angle.cos()).abs();
    let color_at = |point: Pos2| {
        let t = ((point - rect.center()).dot(direction) / length.max(0.001) + 0.5).clamp(0.0, 1.0);
        let stops = &gradient.stops[..gradient.len.max(1)];
        let mut color = stops[0].1;
        if t >= stops[stops.len() - 1].0 {
            color = stops[stops.len() - 1].1;
        } else {
            for pair in stops.windows(2) {
                if t >= pair[0].0 && t <= pair[1].0 {
                    let span = (pair[1].0 - pair[0].0).max(0.0001);
                    color = pair[0].1.lerp_to_gamma(pair[1].1, (t - pair[0].0) / span);
                    break;
                }
            }
        }
        color.gamma_multiply(opacity)
    };
    let mut mesh = crate::gfx::epaint::Mesh::default();
    let vertex = |pos: Pos2, color: Color32| crate::gfx::epaint::Vertex {
        pos,
        uv: Pos2::ZERO,
        color,
    };
    mesh.vertices
        .push(vertex(rect.center(), color_at(rect.center())));
    let count = outline.len() as u32;
    for (point, normal) in &outline {
        let color = color_at(*point);
        mesh.vertices.push(vertex(*point - *normal * 0.5, color));
    }
    for (point, normal) in &outline {
        mesh.vertices
            .push(vertex(*point + *normal * 0.5, Color32::TRANSPARENT));
    }
    for index in 0..count {
        let next = (index + 1) % count;
        mesh.indices.extend([0, 1 + index, 1 + next]);
        let (inner, inner_next) = (1 + index, 1 + next);
        let (outer, outer_next) = (1 + count + index, 1 + count + next);
        mesh.indices
            .extend([inner, outer, outer_next, inner, outer_next, inner_next]);
    }
    painter.add(crate::gfx::Shape::mesh(mesh));
}

struct Env<'a, A: HomeAssets> {
    metrics: &'a UiMetrics,
    view: Vec2,
    assets: &'a RefCell<&'a mut A>,
    targets: &'a RefCell<Vec<(u64, SkinTarget)>>,
    fields: &'a RefCell<Vec<(u64, InputKind)>>,
    layout: &'a RefCell<HomeLayout>,
    next: &'a Cell<u64>,
    claimed: &'a RefCell<HashMap<String, u64, crate::fast::Build>>,
    ancestors: &'a RefCell<Vec<std::rc::Rc<Ancestor>>>,
    positions: &'a Positions,
    inner: &'a RefCell<Vec<Option<f32>>>,
}

impl<A: HomeAssets> Env<'_, A> {
    fn claim(
        &self,
        key: &str,
        action: Option<&Arc<ResolvedAction>>,
        disabled: bool,
    ) -> Option<u64> {
        let action = action.filter(|_| !disabled)?;
        self.claim_target(key, SkinTarget::Action(action.clone()))
    }

    fn claim_field(&self, key: &str, field: &str, kind: InputKind) -> Option<u64> {
        crate::capture::mark_dirty();
        let node = self.claim_target(key, SkinTarget::Field(field.to_owned()))?;
        let mut fields = self.fields.borrow_mut();
        if !fields.iter().any(|(id, _)| *id == node) {
            fields.push((node, kind));
        }
        Some(node)
    }

    fn claim_target(&self, key: &str, target: SkinTarget) -> Option<u64> {
        crate::capture::note_claim(key, &target);
        if let Some(node) = self.claimed.borrow().get(key) {
            return Some(*node);
        }
        let slot = self.next.get();
        if slot >= NODE_SKIN_COUNT {
            return None;
        }
        self.next.set(slot + 1);
        let node = NODE_SKIN_BASE + slot;
        self.claimed.borrow_mut().insert(key.to_owned(), node);
        self.targets.borrow_mut().push((node, target));
        Some(node)
    }
}

pub fn draw_skin(
    context: &crate::gfx::Context,
    viewport: Viewport,
    tree: &Arc<Vec<Resolved>>,
    assets: &mut impl HomeAssets,
) -> SkinFrame {
    draw_tree(context, viewport, tree, assets, true, 1.0, 0)
}

pub fn draw_skin_overlay(
    context: &crate::gfx::Context,
    viewport: Viewport,
    tree: &Arc<Vec<Resolved>>,
    assets: &mut impl HomeAssets,
) -> SkinFrame {
    draw_tree(context, viewport, tree, assets, false, 1.0, 0)
}

pub fn draw_skin_pinned(
    context: &crate::gfx::Context,
    viewport: Viewport,
    tree: &Arc<Vec<Resolved>>,
    assets: &mut impl HomeAssets,
    reserved: u64,
) -> SkinFrame {
    draw_tree(
        context,
        viewport,
        tree,
        assets,
        false,
        1.0,
        crate::nodes::NODE_SKIN_COUNT - reserved,
    )
}

pub fn draw_skin_faded(
    context: &crate::gfx::Context,
    viewport: Viewport,
    tree: &Arc<Vec<Resolved>>,
    assets: &mut impl HomeAssets,
    opacity: f32,
) -> SkinFrame {
    draw_tree(context, viewport, tree, assets, false, opacity, 0)
}

pub fn draw_skin_transient(
    context: &crate::gfx::Context,
    viewport: Viewport,
    tree: Option<&Arc<Vec<Resolved>>>,
    assets: &mut impl HomeAssets,
    key: &str,
    reserved: u64,
) -> Option<SkinFrame> {
    let id = Id::new(("skin-transient", key));
    let kept = id.with("tree");
    let first_slot = if reserved == 0 {
        0
    } else {
        crate::nodes::NODE_SKIN_COUNT - reserved
    };
    let previous: Option<Arc<Vec<Resolved>>> = context.data(|data| data.get_temp(kept));
    match tree {
        Some(tree) => {
            if previous.is_none() {
                crate::motion::reset(context, id);
            }
            context.data_mut(|data| data.insert_temp(kept, tree.clone()));
            let opacity = if previous.is_none() {
                crate::motion::appear(
                    context,
                    id,
                    crate::motion::normal(),
                    crate::motion::Ease::Out,
                )
            } else {
                crate::motion::presence(context, id, true, crate::motion::normal())
            };
            Some(draw_tree(
                context, viewport, tree, assets, false, opacity, first_slot,
            ))
        }
        None => {
            let previous = previous?;
            let opacity = crate::motion::presence(context, id, false, crate::motion::normal());
            if opacity <= 0.001 {
                context.data_mut(|data| data.remove::<Arc<Vec<Resolved>>>(kept));
                crate::motion::reset(context, id);
                return None;
            }
            draw_tree(
                context, viewport, &previous, assets, false, opacity, first_slot,
            );
            None
        }
    }
}

fn draw_tree(
    context: &crate::gfx::Context,
    viewport: Viewport,
    tree: &Arc<Vec<Resolved>>,
    assets: &mut impl HomeAssets,
    opaque: bool,
    opacity: f32,
    first_slot: u64,
) -> SkinFrame {
    let metrics = metrics_for_assets(viewport, assets);
    crate::kit::provide(context, metrics);
    let screen = Rect::from_min_max(Pos2::ZERO, Pos2::new(viewport.width, viewport.height));
    if opaque {
        let painter = context.layer_painter(crate::gfx::LayerId::background());
        painter.rect_filled(screen, 0.0, background());
        crate::paint::paint_ambient(&painter, screen, &*assets);
    }
    let safe = viewport.safe_area();
    let assets = RefCell::new(assets);
    let targets = RefCell::new(Vec::new());
    let fields = RefCell::new(Vec::new());
    let layout = RefCell::new(HomeLayout::default());
    let next = Cell::new(first_slot);
    let claimed = RefCell::new(HashMap::default());
    let ancestors = RefCell::new(Vec::new());
    let inner = RefCell::new(Vec::new());
    let positions = positions_of(tree);
    let env = Env {
        metrics: &metrics,
        view: Vec2::new(viewport.width, viewport.height),
        assets: &assets,
        targets: &targets,
        fields: &fields,
        layout: &layout,
        next: &next,
        claimed: &claimed,
        ancestors: &ancestors,
        positions: &positions,
        inner: &inner,
    };
    prefetch(&env, tree);
    let body = |ui: &mut Ui| {
        if opacity < 1.0 {
            ui.multiply_opacity(opacity);
        }
        let (pinned, flowing): (Vec<_>, Vec<_>) =
            tree.iter().partition(|node| node.kind == "fixed");
        let items = flowing
            .into_iter()
            .chain(pinned)
            .map(|node| build(ui, node, &env))
            .collect();
        let root = layout_items::column(items).padding(Insets::xy(
            safe.left.max(safe.right),
            safe.top.max(safe.bottom),
        ));
        kit::lay(ui, vec![root]);
    };
    if opaque {
        kit::screen_body(context, viewport, "fluxa-skin", (), body);
    } else {
        kit::overlay_body(context, viewport, "fluxa-skin", body);
    }
    flow_publish(context);
    if let Ok(mut kinds) = FIELD_KINDS.lock() {
        *kinds = fields.into_inner();
    }
    SkinFrame {
        targets: targets.into_inner(),
        layout: layout.into_inner(),
    }
}

struct PrefetchPlan {
    tree: Arc<Vec<Resolved>>,
    sheet: Arc<crate::stylesheet::Sheet>,
    metrics: Arc<UiMetrics>,
    view: Vec2,
    requests: Vec<(String, [u32; 2])>,
    logos: Vec<(String, [u32; 2])>,
    issued: Option<std::time::Instant>,
}

const PREFETCH_REISSUE: std::time::Duration = std::time::Duration::from_millis(250);

const PREFETCH_PLANS: usize = 6;

thread_local! {
    static PREFETCH: RefCell<Vec<PrefetchPlan>> = const { RefCell::new(Vec::new()) };
}

fn prefetch<A: HomeAssets>(env: &Env<'_, A>, tree: &Arc<Vec<Resolved>>) {
    let sheet = crate::stylesheet::sheet();
    let metrics = crate::metrics::active();
    PREFETCH.with(|plans| {
        let mut plans = plans.borrow_mut();
        let at = plans.iter().position(|plan| {
            Arc::ptr_eq(&plan.tree, tree)
                && Arc::ptr_eq(&plan.sheet, &sheet)
                && Arc::ptr_eq(&plan.metrics, &metrics)
                && plan.view == env.view
        });
        let at = at.unwrap_or_else(|| {
            let mut requests = Vec::new();
            let mut logos = Vec::new();
            plan_prefetch(env, tree, None, None, &mut requests, &mut logos);
            if plans.len() >= PREFETCH_PLANS {
                plans.remove(0);
            }
            plans.push(PrefetchPlan {
                tree: tree.clone(),
                sheet,
                metrics,
                view: env.view,
                requests,
                logos,
                issued: None,
            });
            plans.len() - 1
        });
        if plans[at]
            .issued
            .is_some_and(|issued| issued.elapsed() < PREFETCH_REISSUE)
        {
            return;
        }
        plans[at].issued = Some(std::time::Instant::now());
        let mut assets = env.assets.borrow_mut();
        for (url, target) in &plans[at].requests {
            assets.prefetch_for(Some(url), *target, ArtworkPriority::Prefetch);
        }
        for (id, target) in &plans[at].logos {
            let Some(card) = crate::card_registry::card(id) else {
                continue;
            };
            crate::card_logos::want(&card);
            if let Some(url) = crate::card_logos::logo_of(&card) {
                assets.prefetch_for(Some(&url), *target, ArtworkPriority::Prefetch);
            }
        }
    });
}

fn plan_prefetch<A: HomeAssets>(
    env: &Env<'_, A>,
    nodes: &[Resolved],
    hint: Option<f32>,
    expand: Option<f32>,
    into: &mut Vec<(String, [u32; 2])>,
    logos: &mut Vec<(String, [u32; 2])>,
) {
    for node in nodes {
        let width = card_width(node, env).or(hint);
        let expand = if prop(node, "expand").is_empty() || !prop(node, "expandOff").is_empty() {
            expand
        } else {
            width
        };
        let expanded_only =
            node.props.get("expandedOnly").is_some_and(truthy) || !prop(node, "logoOf").is_empty();
        if expanded_only && matches!(node.kind.as_str(), "image" | "video") {
            let Some(base) = expand else {
                continue;
            };
            let logo = prop(node, "logoOf").into_owned();
            if !logo.is_empty() {
                let share = prop(node, "share").parse::<f32>().unwrap_or(1.0);
                let size = Vec2::new(
                    node.width
                        .unwrap_or_else(|| expanded_width(base, 1.0, env.metrics) * share),
                    node.height.unwrap_or(base * 1.5),
                );
                logos.push((logo, crate::artwork_target_size(size, 1.0)));
            } else if node.kind == "image"
                && let Some(url) = node.src.clone().filter(|url| !url.is_empty())
            {
                let size = Vec2::new(
                    expanded_width(base, 1.0, env.metrics),
                    node.height.unwrap_or(base * 1.5),
                );
                into.push((url, crate::artwork_target_size(size, 1.0)));
            }
            continue;
        }
        if node.kind == "image" {
            let style = class_parts(node, None)
                .map(|(rule, modifiers)| crate::stylesheet::base_with(&rule, &modifiers));
            let size = match (width, style.and_then(|style| style.aspect)) {
                (Some(width), Some(ratio)) => Vec2::new(width, width / ratio),
                (Some(width), None) if node.fill => Vec2::new(width, width * 0.56),
                _ => Vec2::new(env.view.x, env.view.y * 0.62),
            };
            if let Some(url) = node.src.clone().filter(|url| !url.is_empty()) {
                into.push((url, crate::artwork_target_size(size, 1.0)));
            }
        }
        plan_prefetch(env, &node.children, width, expand, into, logos);
    }
}

const EXPAND_PATIENCE: f64 = 1.5;

thread_local! {
    static EXPAND: Cell<Option<(f32, Option<f64>)>> = const { Cell::new(None) };
}

fn expanding_card(ui: &Ui, node: &Resolved) -> bool {
    if prop(node, "expand").is_empty() || !prop(node, "expandOff").is_empty() {
        return false;
    }
    let ctx = ui.ctx();
    let moving = ctx
        .data(|data| data.get_temp::<(Option<f64>, f32)>(Id::new(("skin-expand", &node.key))))
        .is_some_and(|(since, progress)| since.is_some() || progress > 0.0);
    moving
        || ctx
            .data(|data| data.get_temp::<(Rect, u8)>(Id::new(("skin-state", &node.key))))
            .is_some_and(|(rect, flags)| {
                flags & crate::stylesheet::FOCUSED != 0
                    || ctx
                        .pointer_hover_pos()
                        .is_some_and(|pos| rect.contains(pos))
            })
}

fn expand_ease(
    ui: &Ui,
    node: &Resolved,
    delay: f64,
    seconds: f32,
    ready: impl FnOnce() -> bool,
) -> f32 {
    let ctx = ui.ctx();
    let id = Id::new(("skin-expand", &node.key));
    let (mut since, mut progress): (Option<f64>, f32) =
        ctx.data(|data| data.get_temp(id)).unwrap_or((None, 0.0));
    if progress > 0.0 || since.is_some() {
        crate::capture::mark_dirty();
    }
    if kit::probing() {
        return progress;
    }
    let now = ctx.input(|input| input.time());
    let open = ctx
        .data(|data| data.get_temp::<(Rect, u8)>(Id::new(("skin-state", &node.key))))
        .is_some_and(|(rect, flags)| {
            flags & crate::stylesheet::FOCUSED != 0
                || ctx
                    .pointer_hover_pos()
                    .is_some_and(|pos| rect.contains(pos))
        });
    if open {
        since.get_or_insert(now);
    } else {
        since = None;
    }
    let prepared = open && ready();
    let shown = since.is_some_and(|since| {
        let waited = now - since - delay;
        waited >= 0.0 && (progress > 0.0 || prepared || waited >= EXPAND_PATIENCE)
    });
    let step = ctx.input(|input| input.stable_dt()).min(0.05) / seconds.max(0.01);
    progress = match (shown, delay <= 0.0) {
        (true, true) => 1.0,
        (false, true) => 0.0,
        (true, false) => (progress + step).min(1.0),
        (false, false) => (progress - step).max(0.0),
    };
    ctx.data_mut(|data| data.insert_temp(id, (since, progress)));
    if progress > 0.0 || since.is_some() {
        crate::capture::mark_dirty();
    }
    if let Some(since) = since.filter(|_| !shown) {
        ctx.request_repaint_after(std::time::Duration::from_secs_f64(
            (delay - (now - since)).max(0.0),
        ));
    } else if (shown && progress < 1.0) || (!shown && progress > 0.0) {
        ctx.request_repaint();
    }
    crate::gfx::emath::easing::cubic_out(progress)
}

fn expanded_targets(
    node: &Resolved,
    base: f32,
    metrics: &UiMetrics,
    art: &mut Vec<(String, [u32; 2])>,
    logos: &mut Vec<(String, [u32; 2])>,
) {
    for child in node.children.iter() {
        let logo = prop(child, "logoOf");
        if child.kind == "image" && !logo.is_empty() {
            let share = prop(child, "share").parse::<f32>().unwrap_or(1.0);
            let size = Vec2::new(
                child
                    .width
                    .unwrap_or_else(|| expanded_width(base, 1.0, metrics) * share),
                child.height.unwrap_or(base * 1.5),
            );
            logos.push((logo.into_owned(), crate::artwork_target_size(size, 1.0)));
        } else if child.kind == "image" && child.props.get("expandedOnly").is_some_and(truthy) {
            if let Some(url) = child.src.clone().filter(|url| !url.is_empty()) {
                let size = Vec2::new(
                    expanded_width(base, 1.0, metrics),
                    child.height.unwrap_or(base * 1.5),
                );
                art.push((url, crate::artwork_target_size(size, 1.0)));
            }
        }
        expanded_targets(child, base, metrics, art, logos);
    }
}

fn expand_ready<A: HomeAssets>(node: &Resolved, env: &Env<'_, A>) -> bool {
    let Some(base) = card_width_base(node, env) else {
        return true;
    };
    let mut art = Vec::new();
    let mut logos = Vec::new();
    expanded_targets(node, base, env.metrics, &mut art, &mut logos);
    let mut assets = env.assets.borrow_mut();
    let mut ready = true;
    for (url, target) in &art {
        ready &= assets
            .texture_for(Some(url), *target, ArtworkPriority::Visible)
            .is_some();
    }
    for (id, target) in &logos {
        let Some(card) = crate::card_registry::card(id) else {
            continue;
        };
        crate::card_logos::urgent(&card);
        if crate::card_logos::awaiting(&card) {
            ready = false;
        } else if let Some(url) = crate::card_logos::logo_of(&card) {
            ready &= assets
                .texture_for(Some(&url), *target, ArtworkPriority::Visible)
                .is_some();
        }
    }
    ready
}

fn state_scale<A: HomeAssets>(ui: &Ui, node: &Resolved, env: &Env<'_, A>) -> f32 {
    let Some((rule, modifiers)) = class_parts(node, env.positions.get(&node.key).copied()) else {
        return 1.0;
    };
    let ctx = ui.ctx();
    let (hovered, focused) = ctx
        .data(|data| data.get_temp::<(Rect, u8)>(Id::new(("skin-state", &node.key))))
        .map_or((false, false), |(rect, flags)| {
            (
                ctx.pointer_hover_pos()
                    .is_some_and(|pos| rect.contains(pos)),
                flags & crate::stylesheet::FOCUSED != 0,
            )
        });
    crate::stylesheet::resolve_with(
        ctx,
        &rule,
        Id::new(("skin-pop", &node.key)),
        crate::stylesheet::State {
            hovered,
            focused,
            enabled: true,
            ..Default::default()
        },
        Some(env.metrics),
        &modifiers,
    )
    .scale
}

fn lift_ease(ui: &Ui, node: &Resolved) -> f32 {
    let ctx = ui.ctx();
    let raised = ctx
        .data(|data| data.get_temp::<(Rect, u8)>(Id::new(("skin-state", &node.key))))
        .is_some_and(|(rect, flags)| {
            flags & crate::stylesheet::FOCUSED != 0
                || ctx
                    .pointer_hover_pos()
                    .is_some_and(|pos| rect.contains(pos))
        });
    crate::motion::toggle(
        ctx,
        Id::new(("skin-lift", &node.key)),
        raised,
        crate::motion::fast(),
    )
}

fn build<'a, A: HomeAssets>(ui: &mut Ui, node: &Resolved, env: &'a Env<'a, A>) -> kit::Item<'a> {
    if node.kind == "far" {
        return flow_far(ui, node);
    }
    let item = build_node(ui, node, env);
    let key = prop(node, "windowed").into_owned();
    if key.is_empty() {
        return item;
    }
    let ctx = ui.ctx().clone();
    layout_items::column(vec![item]).painted_shared(move |ui, rect| {
        let clip = ui.clip_rect();
        flow_report(&ctx, &key, rect, clip);
    })
}

pub static CARD_HITS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
pub static CARD_MISSES: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
pub static BUILT_NODES: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

fn build_node<'a, A: HomeAssets>(
    ui: &mut Ui,
    node: &Resolved,
    env: &'a Env<'a, A>,
) -> kit::Item<'a> {
    BUILT_NODES.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let pop = prop(node, "pop") == "true";
    let scale_now = if pop { state_scale(ui, node, env) } else { 1.0 };
    let popped = pop.then_some(scale_now).filter(|scale| *scale > 1.001);
    let lifted = (prop(node, "lift") == "true").then(|| lift_ease(ui, node));
    let expand_delay: Option<f64> = prop(node, "expand")
        .parse()
        .ok()
        .filter(|_| prop(node, "expandOff").is_empty());
    let mut grown;
    let mut expanding = None;
    let node = match expand_delay {
        Some(delay) => {
            let ease = expand_ease(
                ui,
                node,
                delay,
                env.metrics.screen_poster_expand_seconds,
                || expand_ready(node, env),
            );
            if ease <= 0.0 {
                expanding = Some((ease, node.props.get("expandBase").and_then(Value::as_f64)));
                node
            } else {
                grown = node.clone();
                if let Some(base) = node.width.or_else(|| {
                    class_parts(node, env.positions.get(&node.key).copied()).and_then(
                        |(rule, modifiers)| crate::stylesheet::base_with(&rule, &modifiers).width,
                    )
                }) {
                    let wide = base * 1.5 * env.metrics.screen_poster_expand_aspect;
                    grown.width = Some(base + (wide - base).max(0.0) * ease);
                    grown
                        .props
                        .insert("expandBase".to_owned(), Value::from(base));
                }
                expanding = Some((ease, grown.props.get("expandBase").and_then(Value::as_f64)));
                &grown
            }
        }
        None => node,
    };
    let mut scaled;
    let mut pop_growth = None;
    let node = match (popped, expanding.map_or(0.0, |(ease, _)| ease)) {
        (Some(scale), ease) if ease == 0.0 => {
            let base = node.width.or_else(|| {
                class_parts(node, env.positions.get(&node.key).copied()).and_then(
                    |(rule, modifiers)| crate::stylesheet::base_with(&rule, &modifiers).width,
                )
            });
            match base {
                Some(base) => {
                    scaled = node.clone();
                    scaled.width = Some(base * scale);
                    pop_growth = Some(Vec2::new(base * (scale - 1.0), base * 1.5 * (scale - 1.0)));
                    &scaled
                }
                None => node,
            }
        }
        _ => node,
    };
    let sheet_style = class_parts(node, env.positions.get(&node.key).copied())
        .map(|(rule, modifiers)| crate::stylesheet::base_with(&rule, &modifiers));
    let parent_inner = env.inner.borrow().last().copied().flatten();
    let own_width = node
        .props
        .get("expandBase")
        .and_then(Value::as_f64)
        .map(|base| base as f32)
        .or(node.width)
        .or(node.width_vw.map(|share| share * env.view.x))
        .or(sheet_style.as_ref().and_then(|style| style.width));
    let sides = node.padding.map_or(0.0, |[_, right, _, left]| right + left);
    let inner = own_width.or(parent_inner).map(|width| width - sides);
    env.inner.borrow_mut().push(inner);
    let scoped = !node.children.is_empty() && node.class.is_some();
    if scoped {
        env.ancestors.borrow_mut().push(std::rc::Rc::new(Ancestor {
            classes: node
                .class
                .as_deref()
                .unwrap_or_default()
                .split_whitespace()
                .map(class_name)
                .collect(),
            key: node.key.clone(),
            state_id: Id::new(("skin-state", &node.key)),
        }));
    }
    let outer = expanding.map(|state| EXPAND.replace(Some(state)));
    let item = match node.kind.as_str() {
        "row" => {
            let row = layout_items::row(children(ui, node, env));
            if node.props.get("wrap").is_some_and(truthy) {
                row.wrap()
            } else {
                row
            }
        }
        "column" => layout_items::column(children(ui, node, env)),
        "stack" => layout_items::stack(children(ui, node, env)),
        "grid" => grid(ui, node, env),
        "page" => layout_items::column(children(ui, node, env))
            .padding(Insets::xy(env.metrics.gutter, 0.0))
            .align_self(FlexAlign::Stretch),
        "scroll" => scroll(ui, node, env),
        "pager" => pager(ui, node, env),
        "spacer" => spacer(node),
        "text" => text(ui, node, env),
        "image" => image(ui, node, env),
        "video" => video(node),
        "fixed" => fixed(ui, node, env),
        "icon" => icon(node, env),
        "input" => input(ui, node, env),
        "qr" => qr(node),
        _ => layout_items::gap(0.0),
    };
    if let Some(outer) = outer {
        EXPAND.set(outer);
    }
    if scoped {
        env.ancestors.borrow_mut().pop();
    }
    env.inner.borrow_mut().pop();
    let margin = sheet_style.as_ref().map_or(0.0, |style| style.margin);
    let item = frame(
        boxed(item, node, env),
        node,
        env.view,
        sheet_style.as_ref(),
        parent_inner,
    );
    let lead = rank_lead(node, own_width);
    let item = if lead > 0.0 {
        layout_items::row(vec![layout_items::gap(lead), item])
    } else {
        item
    };
    let item = match pop_growth {
        Some(extra) => item.margin(Insets {
            left: -extra.x * 0.5,
            right: -extra.x * 0.5,
            top: -extra.y * 0.5,
            bottom: -extra.y * 0.5,
        }),
        None => item,
    };
    let item = match lifted {
        Some(ease) => {
            let reach = env.metrics.card_lift;
            layout_items::column(vec![item]).padding(Insets {
                left: 0.0,
                top: reach * (1.0 - ease),
                right: 0.0,
                bottom: reach * ease,
            })
        }
        None => item,
    };
    if margin > 0.0 {
        let mut wrapper = layout_items::column(vec![item]).padding(Insets::all(margin));
        if node.fill {
            wrapper = wrapper.grow(1.0).align_self(FlexAlign::Stretch);
        } else if let Some(grow) = node.grow {
            wrapper = wrapper.grow(grow);
        }
        return wrapper;
    }
    item
}

fn group_hovered(context: &crate::gfx::Context, group: &str) -> bool {
    let rect = context.data(|data| data.get_temp::<Rect>(Id::new(("skin-group", group))));
    rect.zip(context.pointer_hover_pos())
        .is_some_and(|(rect, pos)| rect.expand(6.0).contains(pos))
}

fn boxed<'a, A: HomeAssets>(
    item: kit::Item<'a>,
    node: &Resolved,
    env: &'a Env<'a, A>,
) -> kit::Item<'a> {
    let container = matches!(node.kind.as_str(), "row" | "column" | "stack");
    let class = class_parts(node, env.positions.get(&node.key).copied());
    let chain = if class.is_some() {
        env.ancestors.borrow().clone()
    } else {
        Vec::new()
    };
    let target = if container {
        env.claim(
            &node.key,
            node.action.as_ref(),
            node.props.get("disabled").is_some_and(truthy),
        )
    } else {
        None
    };
    let selected = node.props.get("selected").is_some_and(truthy);
    let disabled = node.props.get("disabled").is_some_and(truthy);
    let span = node
        .props
        .get("span")
        .map(|value| match value {
            Value::String(text) => text.trim().parse::<f64>().unwrap_or(1.0),
            other => other.as_f64().unwrap_or(1.0),
        })
        .map_or(1.0, |share| share.clamp(0.0, 1.0) as f32);
    let start = node
        .props
        .get("start")
        .map(|value| match value {
            Value::String(text) => text.trim().parse::<f64>().unwrap_or(0.0),
            other => other.as_f64().unwrap_or(0.0),
        })
        .map_or(0.0, |share| share.clamp(0.0, 1.0) as f32);
    let group = node
        .props
        .get("group")
        .and_then(Value::as_str)
        .map(str::to_owned);
    let drag = prop(node, "drag").into_owned();
    let indicator = prop(node, "indicator").into_owned();
    let tint = hex_color(&prop(node, "tint"));
    let outline = hex_color(&prop(node, "border"));
    let slots: Vec<(String, Option<u64>)> = if drag.is_empty() && indicator.is_empty() {
        Vec::new()
    } else {
        node.children
            .iter()
            .filter(|child| !is_float(child))
            .map(|child| {
                (
                    child.key.clone(),
                    env.claimed.borrow().get(&child.key).copied(),
                )
            })
            .collect()
    };
    let drag_max = node
        .props
        .get("max")
        .and_then(|value| match value {
            Value::Number(number) => number.as_f64(),
            Value::String(text) => text.parse().ok(),
            _ => None,
        })
        .unwrap_or(0.0);
    let layout = env.layout;
    let popover = prop(node, "popover").into_owned();
    let hover_delay: Option<f64> = prop(node, "hover").parse().ok();
    let compact = node.props.get("hoverCompact").is_some_and(truthy);
    let floats: Vec<Resolved> = node
        .children
        .iter()
        .filter(|child| is_float(child))
        .cloned()
        .collect();
    if !container
        || (class.is_none()
            && target.is_none()
            && group.is_none()
            && drag.is_empty()
            && indicator.is_empty()
            && popover.is_empty()
            && hover_delay.is_none())
    {
        return item;
    }
    let class = class;
    let metrics = env.metrics;
    let key = node.key.clone();
    item.painted_shared(move |ui, rect| {
        if kit::probing() {
            return;
        }
        if let Some(group) = &group
            && !kit::probing()
        {
            ui.ctx()
                .data_mut(|data| data.insert_temp(Id::new(("skin-group", group)), rect));
        }
        let sense = if !drag.is_empty() || !indicator.is_empty() {
            Sense::click_and_drag()
        } else if !popover.is_empty() {
            Sense::click()
        } else if target.is_some() {
            Sense::click()
        } else {
            Sense::hover()
        };
        let response = ui.interact(rect, Id::new(("skin-box", &key)), sense);
        if !drag.is_empty() && response.hovered() {
            ui.ctx()
                .set_cursor_icon(crate::gfx::CursorIcon::PointingHand);
        }
        if drag == "seek" && drag_max > 0.0 {
            let hover = response.hover_pos().map(|pointer| {
                let fraction = ((pointer.x - rect.left()) / rect.width().max(1.0)).clamp(0.0, 1.0);
                (rect, pointer.x, drag_max * fraction as f64)
            });
            layout.borrow_mut().seek_hover = hover.map(|(_, _, time)| time);
            ui.ctx()
                .data_mut(|data| data.insert_temp(Id::new("player-seek-hover"), hover));
        }
        if !drag.is_empty()
            && drag_max > 0.0
            && !kit::probing()
            && (response.clicked() || response.dragged())
            && let Some(pointer) = response.interact_pointer_pos()
        {
            let fraction = ((pointer.x - rect.left()) / rect.width().max(1.0)).clamp(0.0, 1.0);
            if drag == "seek" {
                layout.borrow_mut().seek_to = Some(drag_max * fraction as f64);
            }
        }
        let selectable = (!drag.is_empty() || !indicator.is_empty()) && !kit::probing();
        let mut dragged_x = None;
        let found: Vec<IndicatorSlot> = if selectable {
            slots
                .iter()
                .filter_map(|(child, node)| {
                    let (rect, flags) = ui
                        .ctx()
                        .data(|data| data.get_temp::<(Rect, u8)>(Id::new(("skin-state", child))))?;
                    Some(IndicatorSlot {
                        rect,
                        selected: flags & crate::stylesheet::SELECTED != 0,
                        node: *node,
                    })
                })
                .collect()
        } else {
            Vec::new()
        };
        if selectable && drag == "select" {
            if response.dragged() {
                dragged_x = response.interact_pointer_pos().map(|pointer| pointer.x);
            }
            if response.drag_stopped()
                && let Some(pointer) = response.interact_pointer_pos()
                && let Some(slot) =
                    found
                        .iter()
                        .filter(|slot| slot.node.is_some())
                        .min_by(|a, b| {
                            let da = (a.rect.center().x - pointer.x).abs();
                            let db = (b.rect.center().x - pointer.x).abs();
                            da.total_cmp(&db)
                        })
                && let Some(node) = slot.node
            {
                crate::interact::activate(ui, node, slot.rect);
            }
        }
        let has_focus = response.has_focus()
            || target.is_some_and(|node| crate::interact::is_focused(ui.ctx(), node));
        if !kit::probing() {
            let flags = u8::from(has_focus) * crate::stylesheet::FOCUSED
                | u8::from(response.is_pointer_button_down_on()) * crate::stylesheet::PRESSED
                | u8::from(selected) * crate::stylesheet::SELECTED;
            ui.ctx().data_mut(|data| {
                data.insert_temp(Id::new(("skin-state", &key)), (rect, flags));
            });
            crate::capture::note_state(&key, rect, flags);
        }
        if let Some((class, modifiers)) = &class {
            let ancestors = ancestor_states(ui.ctx(), &chain);
            let style = crate::stylesheet::within(&ancestors, || {
                crate::stylesheet::resolve_with(
                    ui.ctx(),
                    class,
                    response.id,
                    crate::stylesheet::State {
                        hovered: response.hovered(),
                        pressed: response.is_pointer_button_down_on(),
                        focused: has_focus,
                        enabled: !disabled,
                        selected,
                    },
                    Some(metrics),
                    &modifiers,
                )
            });
            let mut style = style;
            if let Some(tint) = tint {
                style.fill = tint;
                style.gradient = None;
            }
            if let Some(outline) = outline {
                style.border = outline;
                style.stroke = style.stroke.max(1.0);
            }
            let radius = style.radius.of(rect.height());
            let painter = ui.painter();
            let mut rect = rect;
            let width = rect.width();
            rect.min.x += width * start;
            rect.max.x = rect.min.x + width * span;
            paint_box(painter, rect, radius, &style);
            if style.stroke > 0.0 && style.border.a() > 0 {
                painter.rect_stroke(
                    rect,
                    radius,
                    crate::gfx::Stroke::new(style.stroke, style.border),
                    crate::gfx::StrokeKind::Inside,
                );
            }
        }
        if !indicator.is_empty() && selectable {
            paint_indicator(ui, &key, &indicator, &found, dragged_x, metrics);
        }
        if !popover.is_empty() && !floats.is_empty() && !kit::probing() {
            crate::gfx::Popup::from_toggle_button_response(&response)
                .id(Id::new(("skin-popover", &key)))
                .gap(6.0)
                .width(rect.width().max(160.0))
                .close_behavior(crate::gfx::PopupCloseBehavior::CloseOnClick)
                .frame(crate::gfx::Frame::NONE)
                .show(|ui| {
                    for float in &floats {
                        let content = build(ui, float, env);
                        kit::lay_hug(ui, content);
                    }
                });
        }
        if let Some(delay) = hover_delay
            && !floats.is_empty()
            && !kit::probing()
        {
            hover_float(
                ui,
                &key,
                rect,
                response.hovered(),
                delay,
                compact,
                &floats,
                env,
            );
        }
        if let Some(node) = target.filter(|_| !kit::probing()) {
            response.focus_node(ui, node);
        }
    })
}

fn hover_float<'a, A: HomeAssets>(
    ui: &mut Ui,
    key: &str,
    rect: Rect,
    hovered: bool,
    delay: f64,
    compact: bool,
    floats: &[Resolved],
    env: &'a Env<'a, A>,
) {
    let ctx = ui.ctx().clone();
    let since_id = Id::new(("skin-hover", key));
    let shown_id = Id::new(("skin-hover-rect", key));
    let now = ctx.input(|input| input.time());
    let shown: Option<Rect> = ctx.data(|data| data.get_temp(shown_id));
    if shown.is_some() {
        crate::capture::mark_dirty();
    }
    let inside = shown
        .zip(ctx.pointer_hover_pos())
        .is_some_and(|(shown, pointer)| shown.contains(pointer));
    if !hovered && !inside {
        ctx.data_mut(|data| {
            data.remove::<f64>(since_id);
            data.remove::<Rect>(shown_id);
        });
        return;
    }
    let since = ctx.data_mut(|data| match data.get_temp::<f64>(since_id) {
        Some(since) => since,
        None => {
            data.insert_temp(since_id, now);
            now
        }
    });
    let wait = since + delay - now;
    if wait > 0.0 {
        ctx.request_repaint_after(std::time::Duration::from_secs_f64(wait));
        return;
    }
    let popover = if compact {
        env.metrics.card_popover_min
    } else {
        env.metrics.card_popover_max
    };
    let width = popover.of(rect.width());
    let area = crate::gfx::Area::new(Id::new(("skin-hover-area", key)))
        .order(crate::gfx::Order::Tooltip)
        .fixed_pos(rect.center())
        .pivot(crate::gfx::Align2::CENTER_CENTER)
        .constrain(true)
        .show(&ctx, |ui| {
            ui.set_max_width(width);
            for float in floats {
                let mut sized = float.clone();
                sized.width = Some(width);
                let content = build(ui, &sized, env);
                kit::lay_hug(ui, content);
            }
        });
    let used = area.response.rect;
    ctx.data_mut(|data| data.insert_temp(shown_id, used));
}

fn lens(ctx: &crate::gfx::Context, keys: &[String], rect: Rect) -> Rect {
    for key in keys {
        let found = ctx.data(|data| data.get_temp::<(f32, f32, f32)>(Id::new(("skin-lens", key))));
        if let Some((x, reach, magnify)) = found {
            let t = 1.0 - ((rect.center().x - x).abs() / reach.max(1.0)).clamp(0.0, 1.0);
            let scale = 1.0 + magnify * t * t * (3.0 - 2.0 * t);
            return crate::flex::scaled(rect, scale);
        }
    }
    rect
}

fn ancestor_keys<A: HomeAssets>(env: &Env<'_, A>) -> Vec<String> {
    env.ancestors
        .borrow()
        .iter()
        .map(|ancestor| ancestor.key.clone())
        .collect()
}

fn indicator_width(rects: &[Rect], x: f32) -> f32 {
    let first = rects[0];
    if x <= first.center().x {
        return first.width();
    }
    for pair in rects.windows(2) {
        let (a, b) = (pair[0], pair[1]);
        if x <= b.center().x {
            let t = (x - a.center().x) / (b.center().x - a.center().x).max(1.0);
            return a.width() + (b.width() - a.width()) * t;
        }
    }
    rects[rects.len() - 1].width()
}

struct IndicatorSlot {
    rect: Rect,
    selected: bool,
    node: Option<u64>,
}

fn paint_indicator(
    ui: &Ui,
    key: &str,
    class: &str,
    slots: &[IndicatorSlot],
    dragged_x: Option<f32>,
    metrics: &UiMetrics,
) {
    let Some(first) = slots.first() else {
        return;
    };
    let rects: Vec<Rect> = slots.iter().map(|slot| slot.rect).collect();
    let inner = rects.iter().fold(first.rect, |acc, rect| acc.union(*rect));
    let first_x = first.rect.center().x;
    let last_x = slots[slots.len() - 1].rect.center().x;
    let active = slots
        .iter()
        .find(|slot| slot.selected)
        .map(|slot| slot.rect.center().x);
    let Some(target) = dragged_x.map(|x| x.clamp(first_x, last_x)).or(active) else {
        return;
    };
    let x = crate::motion::ease(
        ui.ctx(),
        Id::new(("skin-indicator-x", key)),
        target,
        crate::motion::normal(),
        crate::motion::Ease::Linear,
    );
    let style = crate::stylesheet::resolve_with(
        ui.ctx(),
        class,
        Id::new(("skin-indicator", key)),
        crate::stylesheet::State {
            hovered: false,
            pressed: false,
            focused: false,
            enabled: true,
            selected: false,
        },
        Some(metrics),
        &[],
    );
    let width = indicator_width(&rects, x);
    let reach = width * style.stretch;
    let stretch = (target - x).abs().min(reach);
    let squash = if reach > 0.0 {
        (stretch / reach).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let pill = crate::flex::centered_at_x(
        inner,
        x,
        Vec2::new(
            width + stretch,
            inner.height() * (1.0 - style.squash * squash),
        ),
    )
    .intersect(inner);
    if style.magnify > 0.0 {
        ui.ctx().data_mut(|data| {
            data.insert_temp(
                Id::new(("skin-lens", key)),
                (x, width * style.reach, style.magnify),
            );
        });
    }
    paint_box(ui.painter(), pill, style.radius.of(pill.height()), &style);
    if dragged_x.is_some() || stretch > 0.5 {
        ui.ctx().request_repaint();
    }
}

fn is_float(node: &Resolved) -> bool {
    node.props.get("float").is_some_and(truthy)
}

fn grid<'a, A: HomeAssets>(ui: &mut Ui, node: &Resolved, env: &'a Env<'a, A>) -> kit::Item<'a> {
    let metrics = env.metrics;
    let own = node.width.or(node.width_vw.map(|share| share * env.view.x));
    let margin = if own.is_some() { 0.0 } else { metrics.gutter };
    let inner = own.unwrap_or(env.view.x - metrics.gutter * 2.0);
    let columns = node
        .props
        .get("columns")
        .and_then(|value| match value {
            Value::String(text) => text.trim().parse::<u64>().ok(),
            other => other.as_u64(),
        })
        .filter(|columns| *columns > 0);
    let (cell, gap) = match columns {
        Some(columns) => {
            let gap = node.gap.unwrap_or(metrics.control_gap);
            let cell = ((inner - gap * (columns - 1) as f32) / columns as f32)
                .floor()
                .max(1.0);
            (cell, gap)
        }
        None => {
            let sizes = crate::layout::PosterGrid::new(inner, metrics);
            (sizes.card_width, sizes.gap)
        }
    };
    request_more(ui.ctx(), node, row_hint(&node.children, ui.ctx()), env);
    let top_id = Id::new(("skin-grid-top", &node.key));
    let row_id = Id::new(("skin-grid-row", &node.key));
    let ctx = ui.ctx().clone();
    let top = ctx.data(|data| data.get_temp::<f32>(top_id));
    let row = node
        .children
        .iter()
        .find_map(|child| {
            ctx.data(|data| data.get_temp::<(Rect, u8)>(Id::new(("skin-state", &child.key))))
        })
        .map(|(rect, _)| rect.height())
        .filter(|height| *height > 1.0)
        .or_else(|| ctx.data(|data| data.get_temp::<f32>(row_id)));
    if let Some(row) = row {
        ctx.data_mut(|data| data.insert_temp(row_id, row));
    }
    let known = top.zip(row);
    let clip = ui.clip_rect();
    let columns = (((inner + gap) / (cell + gap)).floor() as usize).max(1);
    if let Some((top, row)) = known {
        let pitch = (row + gap).max(1.0);
        let screens = (clip.height() / pitch).ceil() as usize + 1;
        let first = ((clip.top() - top) / pitch).floor().max(0.0) as usize;
        let last = ((clip.bottom() - top) / pitch).floor().max(0.0) as usize;
        let chunk = columns * 4;
        let start = first.saturating_sub(screens * 2) * columns / chunk * chunk;
        let end = ((last + screens * 2 + 1) * columns).div_ceil(chunk) * chunk;
        if let Ok(mut windows) = GRID_WINDOWS.lock()
            && windows.get(&node.key) != Some(&(start, end))
        {
            windows.insert(node.key.clone(), (start, end));
            ctx.request_repaint();
        }
    }
    let items = node
        .children
        .iter()
        .enumerate()
        .map(|(index, child)| {
            let far = child.kind == "far"
                || known.is_some_and(|(top, row)| {
                    let y = top + (index / columns) as f32 * (row + gap);
                    y + row < clip.top() - clip.height() * 0.5
                        || y > clip.bottom() + clip.height() * 0.5
                });
            if far {
                far_card(child, cell, known.map(|(_, row)| row), env)
            } else {
                let mut child = child.clone();
                if child.width.is_none() {
                    child.width = Some(cell);
                }
                build(ui, &child, env)
            }
        })
        .collect();
    layout_items::row(items)
        .wrap()
        .gap(gap)
        .padding(Insets::xy(margin, 0.0))
        .align_self(FlexAlign::Stretch)
        .painted(move |ui, rect| {
            if !kit::probing() {
                ui.ctx()
                    .data_mut(|data| data.insert_temp(top_id, rect.top()));
            }
        })
}

fn row_hint(cards: &[Resolved], ctx: &crate::gfx::Context) -> f32 {
    cards
        .iter()
        .find_map(|child| {
            ctx.data(|data| data.get_temp::<(Rect, u8)>(Id::new(("skin-state", &child.key))))
        })
        .map_or(300.0, |(rect, _)| rect.height())
}

fn request_more<A: HomeAssets>(
    ctx: &crate::gfx::Context,
    node: &Resolved,
    row: f32,
    env: &Env<'_, A>,
) {
    let Some(request) = node.props.get("more").filter(|more| more.is_object()) else {
        return;
    };
    let Some(skip) = request.get("skip").and_then(Value::as_i64) else {
        return;
    };
    let Some((offset, max)) = kit::scroll_state(ctx, "fluxa-skin") else {
        return;
    };
    if offset < (max - env.view.y * 8.0 - row * 12.0).max(0.0) {
        return;
    }
    let id = Id::new(("skin-grid-more", &node.key));
    let now = ctx.input(|input| input.time());
    let count = node.children.len();
    let retry = std::time::Duration::from_secs(2);
    let due = ctx.data_mut(|data| {
        let last = data.get_temp::<(i64, usize, f64)>(id);
        let fresh = last.is_none_or(|(prev, prev_count, at)| {
            prev != skip || prev_count != count || now - at >= retry.as_secs_f64()
        });
        if fresh {
            data.insert_temp(id, (skip, count, now));
        }
        fresh
    });
    if due {
        env.layout.borrow_mut().load_more.push(request.clone());
    }
    ctx.request_repaint_after(retry);
}

fn icon<'a, A: HomeAssets>(node: &Resolved, env: &'a Env<'a, A>) -> kit::Item<'a> {
    let metrics = env.metrics;
    let name = prop(node, "name").into_owned();
    let size = node
        .props
        .get("size")
        .and_then(|value| value.as_f64())
        .map_or(metrics.icon_button_size * 0.5, |size| size as f32);
    let chain: Vec<(std::sync::Arc<str>, u8)> = env
        .ancestors
        .borrow()
        .iter()
        .flat_map(|ancestor| {
            ancestor
                .classes
                .iter()
                .map(|class| (std::sync::Arc::clone(class), 0))
        })
        .collect();
    let tint = class_parts(node, None)
        .map(|(rule, modifiers)| {
            crate::stylesheet::within(&chain, || {
                let style = crate::stylesheet::base_with(&rule, &modifiers);
                if style.text_declared {
                    style.text
                } else {
                    metrics.text_primary
                }
            })
        })
        .unwrap_or(metrics.text_primary);
    let assets = env.assets;
    let keys = ancestor_keys(env);
    let logo = prop(node, "logo").into_owned();
    if !logo.is_empty() {
        let found = assets.borrow().logo(&logo);
        let Some((texture, aspect)) = found else {
            return layout_items::gap(0.0);
        };
        return kit::shared(aspect * size, move |ui, rect| {
            let rect = lens(ui.ctx(), &keys, rect);
            ui.painter()
                .image(texture, rect, crate::paint::full_uv(), Color32::WHITE);
        });
    }
    let spinning = prop(node, "spin") == "true";
    kit::shared(Vec2::splat(size), move |ui, rect| {
        let rect = lens(ui.ctx(), &keys, rect);
        let Some(texture) = assets.borrow().icon(&name) else {
            return;
        };
        if !spinning {
            ui.painter()
                .image(texture, rect, crate::paint::full_uv(), tint);
            return;
        }
        let angle = ui.input(|input| input.time()) as f32 * std::f32::consts::TAU;
        let (sin, cos) = angle.sin_cos();
        let center = rect.center();
        let uv = crate::paint::full_uv();
        let mut mesh = crate::gfx::epaint::Mesh::with_texture(texture);
        let corners = [
            (rect.left_top(), uv.left_top()),
            (rect.right_top(), uv.right_top()),
            (rect.right_bottom(), uv.right_bottom()),
            (rect.left_bottom(), uv.left_bottom()),
        ];
        for (pos, uv) in corners {
            let d = pos - center;
            mesh.vertices.push(crate::gfx::epaint::Vertex {
                pos: center + Vec2::new(d.x * cos - d.y * sin, d.x * sin + d.y * cos),
                uv,
                color: tint,
            });
        }
        mesh.indices.extend([0, 1, 2, 0, 2, 3]);
        ui.painter().add(crate::gfx::Shape::mesh(mesh));
        ui.ctx().request_repaint();
    })
}

fn expanded_width(base: f32, ease: f32, metrics: &UiMetrics) -> f32 {
    base * (1.0 + (metrics.screen_poster_expand_aspect * 1.5 - 1.0) * ease)
}

fn children<'a, A: HomeAssets>(
    ui: &mut Ui,
    node: &Resolved,
    env: &'a Env<'a, A>,
) -> Vec<kit::Item<'a>> {
    let (ease, base) = EXPAND.get().unwrap_or((0.0, None));
    node.children
        .iter()
        .filter(|child| !is_float(child))
        .filter(|child| {
            let flag = |name: &str| child.props.get(name).is_some_and(truthy);
            let after = child
                .props
                .get("after")
                .and_then(Value::as_f64)
                .unwrap_or(0.0) as f32;
            (!flag("expandedOnly") || ease > after.max(0.0)) && (!flag("posterOnly") || ease < 0.5)
        })
        .map(|child| {
            match (
                base,
                child.props.get("expandedOnly").is_some_and(truthy)
                    && matches!(child.kind.as_str(), "image" | "video"),
            ) {
                (Some(base), true) => {
                    let mut sized = child.clone();
                    let base = base as f32;
                    let full = expanded_width(base, 1.0, env.metrics);
                    sized.width = Some(expanded_width(base, ease, env.metrics));
                    sized.height = child.height.or(Some(base * 1.5));
                    sized
                        .props
                        .insert("expandedW".to_owned(), Value::from(full));
                    sized
                        .props
                        .insert("expandedH".to_owned(), Value::from(base * 1.5));
                    sized.props.insert("posterW".to_owned(), Value::from(base));
                    build(ui, &sized, env)
                }
                _ => build_in(ui, child, node, env),
            }
        })
        .collect()
}

fn fixed_width(node: &Resolved) -> f32 {
    node.width
        .or_else(|| {
            (node.kind == "icon").then(|| {
                node.props
                    .get("size")
                    .and_then(Value::as_f64)
                    .map_or(0.0, |size| size as f32)
            })
        })
        .unwrap_or(0.0)
}

fn build_in<'a, A: HomeAssets>(
    ui: &mut Ui,
    child: &Resolved,
    parent: &Resolved,
    env: &'a Env<'a, A>,
) -> kit::Item<'a> {
    let inherited = env.inner.borrow().last().copied().flatten();
    let shared = parent.kind == "row" && child.grow.is_some() && child.width.is_none();
    let Some(width) = inherited.filter(|_| shared) else {
        return build(ui, child, env);
    };
    let siblings = parent.children.iter().filter(|other| !is_float(other));
    let count = siblings.clone().count();
    let reserved: f32 = siblings
        .filter(|other| other.key != child.key)
        .map(fixed_width)
        .sum::<f32>()
        + parent.gap.unwrap_or(0.0) * count.saturating_sub(1) as f32;
    env.inner
        .borrow_mut()
        .push(Some((width - reserved).max(0.0)));
    let item = build(ui, child, env);
    env.inner.borrow_mut().pop();
    item
}

fn frame<'a>(
    mut item: kit::Item<'a>,
    node: &Resolved,
    view: Vec2,
    style: Option<&crate::stylesheet::Style>,
    parent: Option<f32>,
) -> kit::Item<'a> {
    let aspect = style.and_then(|style| style.aspect);
    let mut width = node
        .width
        .or(node.width_vw.map(|share| share * view.x))
        .or(style.and_then(|style| style.width));
    let mut height = node
        .height
        .or(node.height_vh.map(|share| share * view.y))
        .or(style.and_then(|style| style.height));
    if let (Some(ratio), None) = (aspect, height) {
        width = width.or(parent);
        height = width.map(|width| width / ratio);
    }
    if let Some(width) = width {
        item = item.width(width);
    }
    if let Some(height) = height {
        item = item.height(height);
    }
    if let Some(max) = style
        .and_then(|style| style.max_width)
        .filter(|_| node.kind != "text")
    {
        item = item.max_width(max);
    }
    if let Some(grow) = node.grow {
        item = item.grow(grow);
    }
    if node.fill {
        item = item.grow(1.0).align_self(FlexAlign::Stretch);
    }
    if let Some(gap) = node.gap {
        item = item.gap(gap);
    }
    if let Some([top, right, bottom, left]) = node.padding {
        item = item.padding(Insets {
            left,
            top,
            right,
            bottom,
        });
    }
    if let Some(align) = node.align {
        item = item.align(match align {
            Align::Start => FlexAlign::Start,
            Align::Center => FlexAlign::Center,
            Align::End => FlexAlign::End,
            Align::Stretch => FlexAlign::Stretch,
        });
    }
    if let Some(justify) = node.justify {
        item = item.justify(match justify {
            Justify::Start => FlexJustify::Start,
            Justify::Center => FlexJustify::Center,
            Justify::End => FlexJustify::End,
            Justify::SpaceBetween => FlexJustify::SpaceBetween,
            Justify::SpaceAround => FlexJustify::SpaceAround,
            Justify::SpaceEvenly => FlexJustify::SpaceEvenly,
        });
    }
    item
}

fn spacer<'a>(node: &Resolved) -> kit::Item<'a> {
    match node.width.or(node.height) {
        Some(px) if node.grow.is_none() => layout_items::gap(px),
        _ => layout_items::fill(),
    }
}

fn text_style(style: Option<&str>) -> (Type, Role) {
    let mut kind = Type::Body;
    let mut role = Role::Primary;
    for word in style.unwrap_or_default().split_whitespace() {
        match word {
            "headline" => kind = Type::Headline,
            "title" => kind = Type::Title,
            "subtitle" => kind = Type::Subtitle,
            "body" => kind = Type::Body,
            "meta" => kind = Type::Meta,
            "label" => kind = Type::Label,
            "secondary" => role = Role::Secondary,
            "muted" => role = Role::Muted,
            _ => {}
        }
    }
    (kind, role)
}

fn text_rule(
    ui: &Ui,
    node: &Resolved,
    metrics: &UiMetrics,
    chain: &[std::rc::Rc<Ancestor>],
    position: Option<(usize, usize)>,
) -> Option<crate::stylesheet::Style> {
    let (class, modifiers) = class_parts(node, position)?;
    let follow = prop(node, "follow").into_owned();
    let hovered = !follow.is_empty() && group_hovered(ui.ctx(), &follow);
    let ancestors = ancestor_states(ui.ctx(), chain);
    Some(crate::stylesheet::within(&ancestors, || {
        crate::stylesheet::resolve_with(
            ui.ctx(),
            &class,
            Id::new(("skin-text", &node.key)),
            crate::stylesheet::State {
                hovered,
                pressed: false,
                focused: false,
                enabled: true,
                selected: false,
            },
            Some(metrics),
            &modifiers,
        )
    }))
}

fn hex_color(text: &str) -> Option<Color32> {
    let digits = text.strip_prefix('#')?;
    let number = u32::from_str_radix(digits, 16).ok()?;
    if digits.len() == 8 {
        return Some(Color32::from_rgba_unmultiplied(
            (number >> 24) as u8,
            (number >> 16) as u8,
            (number >> 8) as u8,
            number as u8,
        ));
    }
    Some(Color32::from_rgb(
        (number >> 16) as u8,
        (number >> 8) as u8,
        number as u8,
    ))
}

fn text<'a, A: HomeAssets>(ui: &mut Ui, node: &Resolved, env: &'a Env<'a, A>) -> kit::Item<'a> {
    let content = node.text.clone().unwrap_or_default();
    let (kind, role) = text_style(node.style.as_deref());
    let section = node
        .style
        .as_deref()
        .is_some_and(|style| style.split_whitespace().any(|word| word == "section"));
    let section_size = env.metrics.screen_section_title_size;
    if !crate::emoji::contains(&content) {
        let style = text_rule(
            ui,
            node,
            env.metrics,
            &env.ancestors.borrow(),
            env.positions.get(&node.key).copied(),
        );
        let width = style.as_ref().and_then(|style| style.max_width);
        let opacity = style.as_ref().map_or(1.0, |style| style.opacity);
        let color = hex_color(
            node.props
                .get("color")
                .and_then(Value::as_str)
                .unwrap_or_default(),
        )
        .or_else(|| {
            style
                .as_ref()
                .filter(|style| style.text_declared)
                .map(|style| style.text)
        });
        let font_size = style.as_ref().and_then(|style| style.font_size);
        let bold = style
            .as_ref()
            .and_then(|style| style.weight)
            .is_some_and(|weight| weight >= 600.0);
        let lines = node
            .props
            .get("lines")
            .and_then(|value| value.as_u64())
            .filter(|lines| *lines > 0)
            .map(|lines| lines as usize);
        if let Some(rows) = lines {
            let size = font_size.unwrap_or(env.metrics.hero_synopsis_size.max(14.0));
            let color = color.unwrap_or(env.metrics.text_primary);
            let grown = match EXPAND.get() {
                Some((ease, Some(base))) if ease > 0.0 => {
                    Some(expanded_width(base as f32, ease, env.metrics))
                }
                _ => None,
            };
            let centered = prop(node, "center") == "true";
            let inset = prop(node, "inset").parse::<f32>().unwrap_or(0.0);
            let explicit = node.width.or(grown.map(|width| width - inset * 2.0));
            let inherited = env.inner.borrow().last().copied().flatten();
            let font = match style.as_ref().and_then(|style| style.weight) {
                Some(weight) if weight >= 600.0 => crate::fonts::bold(size),
                Some(weight) if weight >= 500.0 => crate::fonts::medium(size),
                _ => crate::fonts::regular(size),
            };
            if opacity >= 1.0 {
                let wrap_width = explicit.or(inherited).unwrap_or(560.0);
                let mut job = crate::gfx::text::LayoutJob::simple(content, font, color, wrap_width);
                job.wrap.max_rows = rows;
                job.wrap.overflow_character = Some('…');
                if centered {
                    job.halign = crate::gfx::Align::Center;
                }
                let galley = ui.ctx().fonts_mut(|fonts| fonts.layout_job(job));
                let size = if centered {
                    Vec2::new(wrap_width, galley.size().y)
                } else {
                    galley.size()
                };
                return kit::direct(size, move |painter, rect| {
                    let origin = if centered {
                        crate::gfx::Pos2::new(rect.center().x, rect.min.y)
                    } else {
                        rect.min
                    };
                    painter.galley_with_override_text_color(origin, galley, color);
                });
            }
            return kit::widget(ui, move |ui| {
                if opacity < 1.0 {
                    ui.multiply_opacity(opacity);
                }
                let wrap_width = explicit.or(inherited).unwrap_or(560.0);
                let mut job = crate::gfx::text::LayoutJob::simple(
                    content.clone(),
                    font.clone(),
                    color,
                    wrap_width,
                );
                job.wrap.max_rows = rows;
                job.wrap.overflow_character = Some('…');
                if centered {
                    job.halign = crate::gfx::Align::Center;
                }
                let galley = ui.ctx().fonts_mut(|fonts| fonts.layout_job(job));
                let size = if centered {
                    Vec2::new(wrap_width, galley.size().y)
                } else {
                    galley.size()
                };
                let (rect, _) = ui.allocate_exact_size(size, crate::gfx::Sense::hover());
                let origin = if centered {
                    crate::gfx::Pos2::new(rect.center().x, rect.min.y)
                } else {
                    rect.min
                };
                ui.painter()
                    .galley_with_override_text_color(origin, galley, color);
            });
        }
        if opacity >= 1.0 {
            let mut text = kit::text(content, kind).role(role).inert();
            if bold {
                text = text.bold();
            }
            if let Some(color) = color {
                text = text.color(color);
            }
            if let Some(size) = font_size {
                text = text.size(size);
            } else if section {
                text = text.size(section_size);
            }
            let wrap = ui.available_rect_before_wrap().width();
            let (galley, tint) = text.galley(ui, wrap);
            let size = galley.size();
            let ctx = ui.ctx().clone();
            let item = kit::direct(size, move |painter, rect| {
                let galley = if rect.width() + 0.5 < size.x {
                    let ui_ctx = &ctx;
                    let mut job = crate::gfx::text::LayoutJob::simple(
                        galley.job.text.clone(),
                        galley.job.sections[0].format.font_id.clone(),
                        tint,
                        rect.width(),
                    );
                    job.wrap.max_width = rect.width();
                    ui_ctx.fonts_mut(|fonts| fonts.layout_job(job))
                } else {
                    galley
                };
                match width {
                    Some(width) => {
                        let clip =
                            crate::flex::width_clip(rect, width).intersect(painter.clip_rect());
                        painter.with_clip_rect(clip).galley(rect.min, galley, tint);
                    }
                    None => painter.galley(rect.min, galley, tint),
                }
            });
            return match width {
                Some(width) => item.max_width(width),
                None => item,
            };
        }
        let item = kit::widget(ui, move |ui| {
            if opacity < 1.0 {
                ui.multiply_opacity(opacity);
            }
            if let Some(width) = width {
                let clip = crate::flex::width_clip(ui.max_rect(), width);
                ui.set_clip_rect(clip.intersect(ui.clip_rect()));
            }
            let mut text = kit::text(content.clone(), kind).role(role).inert();
            if bold {
                text = text.bold();
            }
            if let Some(color) = color {
                text = text.color(color);
            }
            if let Some(size) = font_size {
                text = text.size(size);
            }
            if section && font_size.is_none() {
                text.size(section_size).show(ui);
            } else {
                text.show(ui);
            }
        });
        return match width {
            Some(width) => item.max_width(width),
            None => item,
        };
    }
    let assets = env.assets;
    kit::widget(ui, move |ui| {
        let theme = ui.tokens();
        crate::emoji::label(
            ui,
            &mut **assets.borrow_mut(),
            &content,
            if section {
                section_size
            } else {
                kind.size(&theme)
            },
            role.color(&theme),
            crate::emoji::Label::default(),
        );
    })
}

#[derive(Clone, Debug)]
pub struct VideoRequest {
    pub item: String,
    pub urls: Vec<String>,
    pub delay: f64,
}

#[derive(Clone, Debug)]
pub struct PlayingVideo {
    pub item: String,
    pub texture: crate::gfx::TextureId,
    pub subtitle: Option<String>,
}

pub fn take_video_request(context: &crate::gfx::Context) -> Option<VideoRequest> {
    context.data_mut(|data| {
        let request = data.get_temp::<VideoRequest>(Id::new("skin-video-request"));
        data.remove::<VideoRequest>(Id::new("skin-video-request"));
        request
    })
}

pub fn set_playing_video(context: &crate::gfx::Context, playing: Option<PlayingVideo>) {
    context.data_mut(|data| match playing {
        Some(playing) => data.insert_temp(Id::new("skin-video-playing"), playing),
        None => data.remove::<PlayingVideo>(Id::new("skin-video-playing")),
    });
}

fn video<'a>(node: &Resolved) -> kit::Item<'a> {
    let item = prop(node, "item").into_owned();
    let delay = node
        .props
        .get("delay")
        .and_then(|value| match value {
            Value::Number(number) => number.as_f64(),
            Value::String(text) => text.parse().ok(),
            _ => None,
        })
        .unwrap_or(2.0)
        .max(0.0);
    let urls: Vec<String> = node
        .props
        .get("urls")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|url| url.as_str().map(str::to_owned))
        .collect();
    let subtitles = node.props.get("subtitles").is_some_and(truthy);
    if prop(node, "source") == "player" {
        let radius = node
            .props
            .get("radius")
            .and_then(Value::as_f64)
            .unwrap_or(0.0) as f32;
        return kit::item(Vec2::ZERO, move |ui, rect| {
            if kit::probing() {
                return;
            }
            let texture = ui
                .ctx()
                .data(|data| data.get_temp::<crate::gfx::TextureId>(Id::new("skin-player-video")));
            ui.painter().rect_filled(rect, radius, Color32::BLACK);
            if let Some(texture) = texture {
                crate::components::rounded_image(
                    ui.painter(),
                    texture,
                    rect,
                    radius,
                    crate::paint::full_uv(),
                    Color32::WHITE,
                );
            }
        })
        .grow(1.0)
        .align_self(FlexAlign::Stretch);
    }
    kit::item(Vec2::ZERO, move |ui, rect| {
        if kit::probing() || item.is_empty() {
            return;
        }
        let context = ui.ctx().clone();
        context.data_mut(|data| {
            data.insert_temp(
                Id::new("skin-video-request"),
                VideoRequest {
                    item: item.to_string(),
                    urls: urls.clone(),
                    delay,
                },
            );
        });
        let playing = context
            .data(|data| data.get_temp::<PlayingVideo>(Id::new("skin-video-playing")))
            .filter(|playing| playing.item == item);
        if let Some(playing) = playing {
            crate::paint::paint_trailer(
                &context,
                ui.painter(),
                playing.texture,
                rect,
                Some(&item),
                255,
            );
            if subtitles && let Some(text) = &playing.subtitle {
                crate::paint::paint_trailer_subtitle(&context, rect, text);
            }
        }
    })
    .grow(1.0)
    .align_self(FlexAlign::Stretch)
}

fn image<'a, A: HomeAssets>(ui: &mut Ui, node: &Resolved, env: &'a Env<'a, A>) -> kit::Item<'a> {
    let source = node.src.clone();
    let assets = env.assets;
    if prop(node, "fit") == "cover" {
        let focus_y = node
            .props
            .get("focus")
            .and_then(|value| value.as_f64())
            .map_or(0.5, |value| value.clamp(0.0, 1.0) as f32);
        let view_width = env.view.x;
        return kit::item(Vec2::ZERO, move |ui, rect| {
            let mut assets = assets.borrow_mut();
            let target = crate::backdrop_target_size(view_width, ui.ctx().pixels_per_point());
            let texture = assets
                .texture_for(source.as_deref(), target, ArtworkPriority::Hero)
                .or_else(|| assets.cached_texture(source.as_deref()));
            if let Some(texture) = texture {
                let size = assets.texture_size(source.as_deref()).unwrap_or(target);
                let profile = assets.artwork_focus(source.as_deref());
                let mut focus = crate::paint::framed_focus(size, rect, profile.as_deref());
                if profile.is_none() {
                    focus[1] = focus_y;
                }
                ui.painter().with_clip_rect(rect).image(
                    texture,
                    rect,
                    crate::paint::cover_uv_focus(size, rect, focus),
                    Color32::from_white_alpha(210),
                );
            }
        })
        .grow(1.0)
        .align_self(FlexAlign::Stretch);
    }
    let share = prop(node, "share").parse::<f32>().ok();
    let (width, share_final) = match (share, EXPAND.get()) {
        (Some(share), Some((ease, Some(base)))) => (
            expanded_width(base as f32, ease, env.metrics) * share,
            Some(expanded_width(base as f32, 1.0, env.metrics) * share),
        ),
        _ => (node.width.unwrap_or(env.metrics.poster_card_width), None),
    };
    let size = if node.fill {
        Vec2::ZERO
    } else {
        Vec2::new(width, node.height.unwrap_or(width * 1.5))
    };
    let contain = prop(node, "fit") == "contain";
    let autoplay = prop(node, "autoplay") != "false";
    let start = prop(node, "position") == "start";
    let reveal = prop(node, "reveal").parse::<f32>().ok();
    let motion = prop(node, "motion").into_owned();
    let metrics = env.metrics;
    let keys = ancestor_keys(env);
    let overlay_card = prop(node, "overlayCard").into_owned();
    let logo_of = prop(node, "logoOf").into_owned();
    let fallback = prop(node, "fallback").into_owned();
    let number = |name: &str| prop(node, name).parse::<f32>().ok();
    let final_size = number("expandedW")
        .zip(number("expandedH"))
        .map(|(w, h)| Vec2::new(w, h))
        .or_else(|| share_final.map(|w| Vec2::new(w, node.height.unwrap_or(w))));
    let poster_size = number("posterW").map(|w| Vec2::new(w, w * 1.5));
    let rank = prop(node, "rank").parse::<usize>().ok();
    let styled = class_parts(node, env.positions.get(&node.key).copied())
        .map(|(rule, modifiers)| crate::stylesheet::base_with(&rule, &modifiers));
    kit::shared(size, move |ui, rect| {
        let rect = lens(ui.ctx(), &keys, rect);
        let source = if logo_of.is_empty() {
            source.clone()
        } else {
            crate::card_registry::card(&logo_of).and_then(|card| {
                crate::card_logos::want(&card);
                crate::card_logos::logo_of(&card)
            })
        };
        let source = match card_art(ui, &overlay_card) {
            Some(url) => Some(url),
            None => source,
        };
        let radius = styled
            .as_ref()
            .map_or(metrics.card_radius, |style| style.radius.of(rect.height()))
            .min(rect.width().min(rect.height()) * 0.5);
        let fill = styled
            .as_ref()
            .map_or(Color32::TRANSPARENT, |style| style.fill);
        let tint = styled.as_ref().map_or(Color32::WHITE, |style| {
            Color32::from_white_alpha((style.opacity * 255.0) as u8)
        });
        let mut assets = assets.borrow_mut();
        if fill.a() > 0 {
            ui.painter().rect_filled(rect, radius, fill);
        }
        let ppp = ui.ctx().pixels_per_point();
        let target = crate::artwork_target_size(final_size.unwrap_or(rect.size()), ppp);
        let mut source = source;
        let texture = match assets.texture_for(source.as_deref(), target, ArtworkPriority::Visible)
        {
            Some(texture) => texture,
            None => {
                let held = (!fallback.is_empty())
                    .then(|| {
                        let size = poster_size.unwrap_or(rect.size());
                        assets.texture_for(
                            Some(&*fallback),
                            crate::artwork_target_size(size, ppp),
                            ArtworkPriority::Visible,
                        )
                    })
                    .flatten();
                let Some(held) = held else {
                    return;
                };
                source = Some(fallback.to_string());
                held
            }
        };
        let (dest, uv) = match (contain, assets.texture_size(source.as_deref())) {
            (true, Some(size)) => {
                let mut dest = kit::fit_inside(rect, size);
                if start {
                    dest = dest.translate(Vec2::new(rect.left() - dest.left(), 0.0));
                }
                (dest, crate::paint::full_uv())
            }
            (false, Some(size)) => (rect, crate::paint::cover_uv(size, rect)),
            _ => (rect, crate::paint::full_uv()),
        };
        let radius = if contain { 0.0 } else { radius };
        match reveal {
            Some(share) => {
                let share = share.clamp(0.0, 1.0);
                let ghost = Color32::from_white_alpha(tint.a() / 4);
                crate::components::rounded_image(ui.painter(), texture, dest, radius, uv, ghost);
                let mut clip = dest;
                clip.set_right(dest.left() + dest.width() * share);
                let painter = ui.painter().with_clip_rect(clip);
                crate::components::rounded_image(&painter, texture, dest, radius, uv, tint);
            }
            None => crate::components::rounded_image(ui.painter(), texture, dest, radius, uv, tint),
        }
        if !motion.is_empty() && !kit::probing() {
            motion_overlay(ui, rect, &motion, autoplay, &mut **assets);
        }
        if kit::probing() {
            return;
        }
        if let Some(card) = (!overlay_card.is_empty())
            .then(|| crate::card_registry::card(&overlay_card))
            .flatten()
        {
            let custom = crate::poster_overlay::custom_url(ui.painter().ctx(), &card).is_some();
            if card.is_landscape() {
                if !custom {
                    crate::components::landscape_logo(ui.painter(), rect, &card, &mut **assets);
                }
                crate::poster_overlay::paint_landscape_band(ui.painter(), rect, &card);
            } else {
                if let Some(logo) = crate::poster_overlay::textless_art(ui.painter().ctx(), &card)
                    .and_then(|art| art.logo)
                {
                    crate::components::textless_logo(ui.painter(), rect, &logo, &mut **assets);
                }
                let tones = assets.artwork_tones(card.artwork_url.as_deref());
                crate::poster_overlay::paint(ui.painter(), rect, &card, radius, tones);
            }
        }
        if let Some(rank) = rank {
            crate::poster_overlay::paint_beside(
                ui.painter(),
                rect,
                rank,
                metrics.accent,
                metrics.background,
            );
            crate::poster_overlay::paint_overlay(
                ui.painter(),
                rect,
                rank,
                metrics.accent,
                metrics.background,
            );
        }
    })
}

fn card_art(ui: &Ui, id: &str) -> Option<String> {
    let card = crate::card_registry::card(id)?;
    crate::poster_overlay::custom_url(ui.painter().ctx(), &card).or_else(|| {
        card.is_landscape()
            .then(|| card.poster_art().map(str::to_owned))
            .flatten()
    })
}

type PagerState = (usize, f64, Option<usize>, f64);

fn pager_step(state: PagerState, count: usize, delta: isize, now: f64) -> PagerState {
    let (index, ..) = state;
    let next = (index as isize + delta).rem_euclid(count as isize) as usize;
    (next, now, Some(index), now)
}

fn faded<'a>(item: kit::Item<'a>, alpha: f32, height: Option<f32>) -> kit::Item<'a> {
    let wrapper = kit::item(Vec2::ZERO, move |ui, rect| {
        ui.set_opacity(alpha);
        kit::lay_in(ui, rect, item);
    })
    .grow(1.0)
    .align_self(FlexAlign::Stretch);
    match height {
        Some(height) => wrapper.height(height),
        None => wrapper,
    }
}

fn pager<'a, A: HomeAssets>(ui: &mut Ui, node: &Resolved, env: &'a Env<'a, A>) -> kit::Item<'a> {
    let count = node.children.len();
    if count == 0 {
        return layout_items::column(Vec::new());
    }
    let interval = node
        .props
        .get("interval")
        .and_then(|value| value.as_f64())
        .unwrap_or(8.0)
        .max(1.0);
    let id = Id::new(("skin-pager", &node.key));
    let drag_id = Id::new(("skin-pager-drag", &node.key));
    let now = ui.ctx().input(|input| input.time());
    let mut state = ui
        .ctx()
        .data(|data| data.get_temp::<PagerState>(id))
        .unwrap_or((0, now, None, f64::NEG_INFINITY));
    if state.0 >= count {
        state = (0, now, None, f64::NEG_INFINITY);
    }
    if count > 1 && now - state.1 >= interval {
        state = pager_step(state, count, 1, now);
    }
    ui.ctx().data_mut(|data| data.insert_temp(id, state));
    if count > 1 {
        ui.ctx()
            .request_repaint_after(std::time::Duration::from_secs_f64(interval));
    }
    let linear = ((now - state.3) / 0.6).clamp(0.0, 1.0) as f32;
    let eased = 1.0 - (1.0 - linear).powi(3);
    let (index, previous) = (state.0, state.2.filter(|previous| *previous < count));
    let height_of = |child: &Resolved| {
        child
            .height
            .or(child.height_vh.map(|share| share * env.view.y))
    };
    let mut layers = Vec::new();
    if let Some(previous) = previous.filter(|_| linear < 1.0) {
        ui.ctx().request_repaint();
        layers.push(faded(
            build(ui, &node.children[previous], env),
            1.0,
            height_of(&node.children[previous]),
        ));
        layers.push(faded(
            build(ui, &node.children[index], env),
            eased,
            height_of(&node.children[index]),
        ));
    } else {
        layers.push(
            build(ui, &node.children[index], env)
                .grow(1.0)
                .align_self(FlexAlign::Stretch),
        );
    }
    if count > 1 {
        layers.push(
            kit::item(Vec2::ZERO, move |ui, rect| {
                let context = ui.ctx().clone();
                let pointer = context.input(|input| input.pointer().interact_pos());
                if context.input(|input| input.pointer().primary_pressed())
                    && let Some(position) = pointer.filter(|position| rect.contains(*position))
                {
                    context.data_mut(|data| data.insert_temp(drag_id, position));
                }
                if !context.input(|input| input.pointer().primary_released()) {
                    return;
                }
                let Some(start) =
                    context.data_mut(|data| data.remove_temp::<crate::gfx::Pos2>(drag_id))
                else {
                    return;
                };
                let Some(end) = pointer else {
                    return;
                };
                let (dx, dy) = (end.x - start.x, end.y - start.y);
                if dx.abs() < 60.0 || dy.abs() > dx.abs() {
                    return;
                }
                let now = context.input(|input| input.time());
                let current = context
                    .data(|data| data.get_temp::<PagerState>(id))
                    .unwrap_or((0, now, None, f64::NEG_INFINITY));
                let delta = if dx < 0.0 { 1 } else { -1 };
                context
                    .data_mut(|data| data.insert_temp(id, pager_step(current, count, delta, now)));
                context.request_repaint();
            })
            .grow(1.0)
            .align_self(FlexAlign::Stretch),
        );
    }
    if layers.len() == 1 {
        return layers.remove(0);
    }
    layout_items::stack(layers).align_self(FlexAlign::Stretch)
}

fn prop<'a>(node: &'a Resolved, key: &str) -> std::borrow::Cow<'a, str> {
    match node.props.get(key) {
        Some(Value::String(text)) => std::borrow::Cow::Borrowed(text.as_str()),
        Some(Value::Bool(true)) => std::borrow::Cow::Borrowed("true"),
        Some(Value::Bool(false)) => std::borrow::Cow::Borrowed("false"),
        Some(other) => std::borrow::Cow::Owned(fluxa_skin::display(other)),
        None => std::borrow::Cow::Borrowed(""),
    }
}

fn qr<'a>(node: &Resolved) -> kit::Item<'a> {
    let modules: Vec<bool> = prop(node, "modules").chars().map(|c| c == '1').collect();
    let count = prop(node, "count").parse::<usize>().unwrap_or(0);
    let side = node
        .props
        .get("size")
        .and_then(|value| value.as_f64())
        .map_or(240.0, |size| size as f32);
    kit::item(Vec2::splat(side), move |ui, rect| {
        let painter = ui.painter();
        painter.rect_filled(rect, 16.0, Color32::WHITE);
        if count == 0 || modules.len() < count * count {
            return;
        }
        let inner = rect.shrink(side * 0.06);
        let cell = inner.width() / count as f32;
        for (index, dark) in modules.iter().enumerate().take(count * count) {
            if !*dark {
                continue;
            }
            let min = inner.min + Vec2::new((index % count) as f32, (index / count) as f32) * cell;
            painter.rect_filled(
                Rect::from_min_max(min, min + Vec2::splat(cell.ceil())),
                0.0,
                Color32::BLACK,
            );
        }
    })
}

fn input<'a, A: HomeAssets>(ui: &mut Ui, node: &Resolved, env: &'a Env<'a, A>) -> kit::Item<'a> {
    let metrics = env.metrics;
    let field = prop(node, "key").into_owned();
    let hint = prop(node, "placeholder").into_owned();
    let value = prop(node, "value").into_owned();
    let kind = match &*prop(node, "kind") {
        "search" => InputKind::Search,
        "password" => InputKind::Password,
        "url" => InputKind::Url,
        "number" => InputKind::Number { secret: false },
        "multiline" => InputKind::Multiline,
        _ => InputKind::Text,
    };
    let fixed = node.width;
    let stretch = node.fill;
    let target = Some(&*field)
        .filter(|field| !field.is_empty())
        .and_then(|field| env.claim_field(&node.key, field, kind));
    let layout = env.layout;
    kit::widget(ui, move |ui| {
        let width = fixed.unwrap_or_else(|| {
            if stretch {
                ui.available_width()
            } else {
                metrics.horizontal_card_width
            }
        });
        let mut text = value.clone();
        let response = if matches!(kind, InputKind::Search) {
            crate::components::search_field(
                ui,
                &mut text,
                &hint,
                width,
                metrics.screen_control_height + 4.0,
                *metrics,
            )
        } else {
            crate::components::text_input(ui, &mut text, &hint, width, kind, *metrics)
        };
        let Some(node) = target.filter(|_| !kit::probing()) else {
            return;
        };
        response.clone().focus_node(ui, node);
        if response.changed() {
            let mut layout = layout.borrow_mut();
            layout.text_input = Some(text);
            layout.text_input_node = Some(node);
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use fluxa_skin::{FormFactor, load_skin};
    use serde_json::json;

    struct NoAssets;

    impl HomeAssets for NoAssets {
        fn background(&self) -> TextureId {
            TextureId::Managed(0)
        }

        fn texture(&mut self, _url: Option<&str>) -> Option<TextureId> {
            None
        }
    }

    fn card(n: usize) -> serde_json::Value {
        json!({"id": format!("tt{n}"), "itemType": "movie", "title": format!("Title {n}"), "subtitle": "2024", "artworkUrl": null})
    }

    fn skin_from(node: serde_json::Value) -> Vec<Resolved> {
        let manifest = json!({
            "schemaVersion": 1, "id": "t", "name": "t", "version": "1.0.0",
            "screens": {"home": "screens/home.json"}, "includes": ["includes.json"], "theme": "style.css"
        })
        .to_string();
        let node = node.to_string();
        let skin = fluxa_skin::load_embedded(&[
            ("skin.json", manifest.as_str()),
            ("includes.json", "{}"),
            ("screens/home.json", node.as_str()),
            ("style.css", ""),
        ])
        .unwrap();
        skin.resolve(
            "home",
            &json!({}),
            FormFactor::Desktop,
            &mut Default::default(),
        )
        .unwrap()
        .unwrap()
    }

    fn sample_tree(rows: usize, per_row: usize) -> Vec<Resolved> {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../fixtures/skins/sample");
        let skin = load_skin(&dir).unwrap();
        let rows: Vec<_> = (0..rows)
            .map(|row| json!({"title": format!("Row {row}"), "cards": (0..per_row).map(card).collect::<Vec<_>>()}))
            .collect();
        let data = json!({"home": {"title": "Welcome", "eyebrow": "Tonight", "rows": rows}});
        skin.resolve("home", &data, FormFactor::Desktop, &mut Default::default())
            .unwrap()
            .unwrap()
    }

    fn render(tree: &[Resolved]) -> (Vec<(u64, ResolvedAction)>, Vec<(u64, Rect)>) {
        let viewport = Viewport::new(1280, 800, UiFormFactor::Desktop);
        let context = crate::gfx::Context::default();
        context.set_fonts(crate::fonts::definitions());
        let mut assets = NoAssets;
        let tree = Arc::new(tree.to_vec());
        let mut frame = None;
        let mut targets = Vec::new();
        for _ in 0..2 {
            crate::begin_focus_frame(&context, None);
            context
                .run_ui(
                    crate::gfx::RawInput {
                        screen_rect: Some(Rect::from_min_size(
                            Pos2::ZERO,
                            Vec2::new(1280.0, 800.0),
                        )),
                        ..Default::default()
                    },
                    |ui| frame = Some(draw_skin(ui.ctx(), viewport, &tree, &mut assets)),
                )
                .drop_without_applying_deltas();
            targets = crate::take_focus_targets(&context).0;
        }
        let actions = frame
            .map(|frame| frame.targets)
            .unwrap_or_default()
            .into_iter()
            .filter_map(|(node, target)| match target {
                SkinTarget::Action(action) => Some((node, action)),
                SkinTarget::Field(_) | SkinTarget::Control => None,
            })
            .collect();
        (actions, targets)
    }

    #[test]
    fn text_inside_an_action_row_does_not_swallow_the_click() {
        let tree = skin_from(json!({
            "type": "row",
            "padding": 8,
            "action": {"name": "choose", "args": {"key": "section", "value": "calendar"}},
            "children": [{"type": "text", "text": "Calendar"}]
        }));
        let (actions, rects) = render(&tree);
        let tree = Arc::new(tree);
        let node = actions[0].0;
        let target = rects.iter().find(|(id, _)| *id == node).unwrap().1.center();
        let viewport = Viewport::new(1280, 800, UiFormFactor::Desktop);
        let context = crate::gfx::Context::default();
        context.set_fonts(crate::fonts::definitions());
        let mut assets = NoAssets;
        let button = |pressed| crate::gfx::Event::PointerButton {
            pos: target,
            button: crate::gfx::PointerButton::Primary,
            pressed,
            modifiers: crate::gfx::Modifiers::NONE,
        };
        let mut hit = None;
        for events in [
            vec![],
            vec![],
            vec![crate::gfx::Event::PointerMoved(target)],
            vec![button(true)],
            vec![button(false)],
        ] {
            crate::begin_focus_frame(&context, None);
            context
                .run_ui(
                    crate::gfx::RawInput {
                        screen_rect: Some(Rect::from_min_size(
                            Pos2::ZERO,
                            Vec2::new(1280.0, 800.0),
                        )),
                        events,
                        ..Default::default()
                    },
                    |ui| {
                        draw_skin(ui.ctx(), viewport, &tree, &mut assets);
                    },
                )
                .drop_without_applying_deltas();
            hit = hit.or(crate::take_focus_targets(&context).1);
        }
        assert_eq!(hit, Some(node));
    }

    #[test]
    fn every_action_gets_a_focus_target_with_a_real_size() {
        let (actions, targets) = render(&sample_tree(1, 3));
        assert_eq!(actions.len(), 2 + 3 + 3);
        assert_eq!(targets.len(), actions.len());
        for (node, rect) in &targets {
            assert!(actions.iter().any(|(id, _)| id == node));
            assert!(
                rect.width() > 0.0 && rect.height() > 0.0,
                "{node} at {rect:?}"
            );
        }
    }

    #[test]
    fn header_actions_sit_on_the_right_of_the_screen() {
        let (actions, targets) = render(&sample_tree(0, 0));
        let find = |name: &str, screen: &str| {
            let node = actions
                .iter()
                .find(|(_, a)| a.name == name && a.args["screen"] == json!(screen))
                .unwrap()
                .0;
            targets.iter().find(|(id, _)| *id == node).unwrap().1
        };
        let search = find("navigate", "discover");
        let profile = targets
            .iter()
            .map(|(_, rect)| *rect)
            .max_by(|a, b| a.max.x.total_cmp(&b.max.x))
            .unwrap();
        assert!(search.min.x > 640.0, "{search:?}");
        assert!(profile.max.x <= 1280.0, "{profile:?}");
    }

    #[test]
    fn strip_cards_keep_their_order_and_do_not_overlap() {
        let (actions, targets) = render(&sample_tree(1, 5));
        let mut cards: Vec<(String, Rect)> = targets
            .iter()
            .filter_map(|(node, rect)| {
                let (_, action) = actions.iter().find(|(id, _)| id == node)?;
                (action.name == "open_detail").then(|| (action.args["id"].to_string(), *rect))
            })
            .collect();
        assert_eq!(cards.len(), 5);
        cards.sort_by(|a, b| a.1.min.x.total_cmp(&b.1.min.x));
        assert!(
            cards
                .windows(2)
                .all(|pair| pair[0].1.max.x <= pair[1].1.min.x)
        );
        let ids: Vec<_> = cards.iter().map(|(id, _)| id.as_str()).collect();
        assert_eq!(ids, ["\"tt0\"", "\"tt1\"", "\"tt2\"", "\"tt3\"", "\"tt4\""]);
    }

    #[test]
    fn rows_stack_downwards_past_the_screen() {
        let (_, targets) = render(&sample_tree(6, 2));
        let lowest = targets
            .iter()
            .map(|(_, rect)| rect.max.y)
            .fold(0.0, f32::max);
        assert!(lowest > 800.0, "{lowest}");
    }

    #[test]
    fn styled_row_with_an_action_is_a_clickable_box() {
        let tree: Vec<Resolved> = serde_json::from_value::<fluxa_skin::Node>(json!({
            "type": "row", "class": "button.secondary", "padding": 12, "gap": 8,
            "action": {"name": "navigate", "args": {"screen": "library"}},
            "children": [{"type": "text", "text": "Library"}]
        }))
        .map(|node| {
            fluxa_skin::resolve(
                &node,
                &Default::default(),
                &json!({}),
                FormFactor::Desktop,
                &mut Default::default(),
            )
            .unwrap()
        })
        .unwrap();
        let (actions, targets) = render(&tree);
        assert_eq!(actions.len(), 1);
        assert_eq!(targets.len(), 1);
        assert!(targets[0].1.width() > 40.0 && targets[0].1.height() > 20.0);
    }

    #[test]
    fn default_home_renders_navbar_hero_and_shelves() {
        let dir =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../shared/skins/default");
        let skin = load_skin(&dir).unwrap();
        let slide = json!({"title": "Slide", "eyebrow": "2026", "itemId": "tt1", "itemType": "movie", "backgroundUrl": null, "logoUrl": null});
        let data = json!({
            "home": {
                "heroSlides": [slide],
                "rows": [{"title": "Row", "cards": (0..4).map(card).collect::<Vec<_>>()}]
            },
            "nav": crate::nav_data("home"),
        });
        let tree = skin
            .resolve("home", &data, FormFactor::Desktop, &mut Default::default())
            .unwrap()
            .unwrap();
        let (actions, targets) = render(&tree);
        let names: Vec<_> = actions
            .iter()
            .map(|(_, action)| action.name.as_str())
            .collect();
        assert_eq!(names.iter().filter(|name| **name == "navigate").count(), 3);
        assert!(names.contains(&"play") && names.contains(&"watchlist_toggle"));
        assert_eq!(
            names.iter().filter(|name| **name == "open_detail").count(),
            4
        );
        assert!(
            targets
                .iter()
                .all(|(_, rect)| rect.width() > 0.0 && rect.height() > 0.0)
        );
    }

    #[test]
    fn disabled_button_claims_no_target() {
        let tree: Vec<Resolved> = serde_json::from_value::<fluxa_skin::Node>(json!({
            "type": "row", "children": [], "props": {"disabled": true},
            "action": {"name": "back"}
        }))
        .map(|node| {
            fluxa_skin::resolve(
                &node,
                &Default::default(),
                &json!({}),
                FormFactor::Desktop,
                &mut Default::default(),
            )
            .unwrap()
        })
        .unwrap();
        let (actions, targets) = render(&tree);
        assert!(actions.is_empty() && targets.is_empty());
    }
    #[test]
    fn form_controls_claim_targets_and_toggle_flips_the_value() {
        let tree: Vec<Resolved> = serde_json::from_value::<fluxa_skin::Node>(json!({
            "type": "column", "gap": 12, "children": [
                {"type": "input", "props": {"key": "query", "placeholder": "Search"}},
                {"type": "row", "children": [], "action": {"name": "set_setting", "args": {"key": "autoplay", "value": true}}}
            ]
        }))
        .map(|node| {
            fluxa_skin::resolve(
                &node,
                &Default::default(),
                &json!({}),
                FormFactor::Desktop,
                &mut Default::default(),
            )
            .unwrap()
        })
        .unwrap();
        let (actions, targets) = render(&tree);
        assert_eq!(targets.len(), 2);
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0].1.args.get("value"), Some(&json!(true)));
    }
}

fn activate(ui: &mut Ui, rect: Rect, target: Option<u64>) {
    if let Some(node) = target.filter(|_| !kit::probing()) {
        let response = ui.interact(rect, Id::new(("skin-target", node)), Sense::click());
        response.focus_node(ui, node);
    }
}

fn scroll_y<'a, A: HomeAssets>(node: &Resolved, env: &'a Env<'a, A>) -> kit::Item<'a> {
    let node = node.clone();
    kit::item(Vec2::ZERO, move |ui, rect| {
        let gap = node.gap.unwrap_or(0.0);
        let mut items = Vec::new();
        for (index, item) in children(ui, &node, env).into_iter().enumerate() {
            if index > 0 && gap > 0.0 {
                items.push(layout_items::gap(gap));
            }
            items.push(item);
        }
        ui.set_clip_rect(rect);
        crate::components::pan_area(
            ui,
            ("skin-scroll-y", &node.key),
            |area| area.max_height(rect.height()).auto_shrink([false, false]),
            |ui| {
                kit::lay(ui, items);
            },
        );
    })
    .grow(1.0)
}

fn scroll<'a, A: HomeAssets>(ui: &mut Ui, node: &Resolved, env: &'a Env<'a, A>) -> kit::Item<'a> {
    if prop(node, "axis") == "y" {
        return scroll_y(node, env);
    }
    let name = Some(strip_name(&node.key));
    let node = node.clone();
    let share = node.grow.is_some() || node.fill;
    let within = prop(&node, "within") == "true";
    let item = kit::widget(ui, move |ui| {
        let gap = node.gap.unwrap_or(0.0);
        let offset = name.map_or(0.0, |name| {
            crate::scroll_region::strip_offset(ui.ctx(), name)
        });
        let reach = ui.ctx().content_rect().width();
        let height_id = Id::new(("skin-strip-height", &node.key, reach as i32));
        let measured = ui.ctx().data(|data| data.get_temp::<f32>(height_id));
        let probing = kit::probing();
        let rect = ui.max_rect();
        let clip = ui.clip_rect();
        let off_screen = measured.is_some_and(|height| {
            probing
                || rect.top() > clip.bottom() + clip.height() * 0.2
                || rect.top() + height < clip.top() - clip.height() * 0.2
        });
        let strip_slot = name
            .filter(|_| !probing && !off_screen && measured.is_some())
            .map(|name| {
                let shown = crate::components::resolve_strip(ui, name);
                let sig = StripSig {
                    card: card_sig(
                        ui,
                        Vec2::new(ui.max_rect().width(), measured.unwrap_or(0.0)),
                    ),
                    shown: shown.to_bits(),
                    reach: reach.to_bits(),
                };
                (name, shown, sig)
            });
        if let Some((_, _, sig)) = &strip_slot
            && cached_strip(ui, &node.children, sig, env)
        {
            return;
        }
        if strip_slot.is_some() {
            crate::capture::begin();
        }
        let mut x = 0.0;
        let mut near: Option<(usize, usize)> = None;
        let items = node
            .children
            .iter()
            .enumerate()
            .map(|(index, child)| {
                let Some(width) = card_width(child, env) else {
                    return build(ui, child, env).shrink(0.0);
                };
                let start = x;
                x += width + gap;
                let far = start + width < offset - reach * 0.1 || start > offset + reach * 1.1;
                if start + width >= offset - reach && start <= offset + reach * 2.0 {
                    near = Some(near.map_or((index, index), |(first, _)| (first, index)));
                }
                if child.kind == "far" || far || off_screen {
                    far_card(child, width, measured.filter(|_| off_screen), env).shrink(0.0)
                } else if let Some(height) = measured.filter(|_| !probing && !expanding_card(ui, child)) {
                    cached_card(
                        ui,
                        &node.children,
                        index,
                        child,
                        Vec2::new(width, height),
                        env,
                    )
                    .shrink(0.0)
                } else {
                    build(ui, child, env).shrink(0.0)
                }
            })
            .collect();
        if !probing
            && let Some((first, last)) = near
            && let Ok(mut windows) = GRID_WINDOWS.lock()
        {
            let window = (first / 4 * 4, (last + 1).div_ceil(4) * 4);
            if windows.get(&node.key) != Some(&window) {
                windows.insert(node.key.clone(), window);
                ui.ctx().request_repaint();
            }
        }
        let strip = layout_items::row(items).gap(gap);
        let ctx = ui.ctx().clone();
        let layer = ui.layer_id();
        let origin = ui.max_rect().min;
        let clip = ui.clip_rect();
        let start = ctx.layer_cursor(layer);
        match (name, &strip_slot) {
            (Some(name), Some((_, shown, _))) => {
                crate::components::pan_strip_shown(ui, name, *shown, !within, |ui| {
                    kit::lay_overflow(ui, strip);
                });
            }
            (Some(name), None) if within => {
                crate::components::pan_strip_within(ui, name, |ui| {
                    kit::lay_overflow(ui, strip);
                });
            }
            (Some(name), None) => {
                crate::components::pan_strip(ui, name, |ui| {
                    kit::lay_overflow(ui, strip);
                });
            }
            (None, _) => {
                kit::lay_hug(ui, strip);
            }
        }
        if let Some((_, _, sig)) = strip_slot {
            let log = crate::capture::end();
            let used = ui.min_rect().size();
            if !log.dirty && !card_busy(&ctx, &log.states) && fully_in_view(clip, origin.y, used.y) {
                let entry = StripEntry {
                    parent: node.children.clone(),
                    sig,
                    origin,
                    clip,
                    used,
                    shapes: ctx.layer_shapes_since(layer, start),
                    registers: log.registers,
                    states: log.states,
                    claims: log.claims,
                    touches: log.touches,
                    regions: log.regions,
                };
                STRIPS.with(|strips| {
                    let mut strips = strips.borrow_mut();
                    if strips.len() >= CARD_CACHE_LIMIT {
                        strips.clear();
                    }
                    strips.insert(
                        Arc::as_ptr(&node.children) as usize,
                        std::rc::Rc::new(entry),
                    );
                });
            }
        }
        if !probing && !off_screen {
            let height = ui.min_rect().height();
            ui.ctx()
                .data_mut(|data| data.insert_temp(height_id, height));
        }
    });
    if share {
        item.basis(0.0).min_width(0.0)
    } else {
        item
    }
}

fn motion_overlay(
    ui: &mut Ui,
    rect: Rect,
    url: &str,
    autoplay: bool,
    assets: &mut impl HomeAssets,
) {
    let clip = ui.clip_rect();
    let visible = rect.intersect(clip);
    let on_screen = visible.is_positive() && visible.area() >= rect.area() * 0.6;
    let hovered = ui.rect_contains_pointer(rect);
    let target = fluxa_artwork::animation_target_size(crate::artwork_target_size(
        rect.size(),
        ui.ctx().pixels_per_point(),
    ));
    if !(autoplay && on_screen || hovered) {
        return;
    }
    let Some(animated) = assets.animated_texture_for(Some(url), target, ArtworkPriority::Visible)
    else {
        return;
    };
    let dest = kit::fit_inside(rect, animated.image_size);
    ui.painter()
        .rect_filled(rect, 0.0, crate::stylesheet::color("well"));
    texture_image(
        ui.painter(),
        animated.texture,
        dest,
        animated.uv,
        Color32::WHITE,
    );
    ui.ctx().request_repaint();
}

fn rank_lead(node: &Resolved, width: Option<f32>) -> f32 {
    let rank = prop(node, "rank").parse::<usize>().ok();
    match (rank, width) {
        (Some(rank), Some(width)) => crate::poster_overlay::lead_width(width * 1.5, Some(rank)),
        _ => 0.0,
    }
}

fn card_width<A: HomeAssets>(node: &Resolved, env: &Env<'_, A>) -> Option<f32> {
    let width = card_width_base(node, env);
    width.map(|width| width + rank_lead(node, Some(width)))
}

fn card_width_base<A: HomeAssets>(node: &Resolved, env: &Env<'_, A>) -> Option<f32> {
    node.width
        .or(node.width_vw.map(|share| share * env.view.x))
        .or_else(|| {
            class_parts(node, None)
                .and_then(|(rule, modifiers)| crate::stylesheet::base_with(&rule, &modifiers).width)
        })
}

fn far_card<'a, A: HomeAssets>(
    node: &Resolved,
    width: f32,
    height: Option<f32>,
    env: &'a Env<'a, A>,
) -> kit::Item<'a> {
    let target = env.claim(&node.key, node.action.as_ref(), false);
    kit::shared(Vec2::new(width, height.unwrap_or(0.0)), move |ui, rect| {
        activate(ui, rect, target)
    })
}

fn fixed<'a, A: HomeAssets>(ui: &mut Ui, node: &Resolved, env: &'a Env<'a, A>) -> kit::Item<'a> {
    let number = |name: &str| {
        node.props.get(name).and_then(|value| match value {
            Value::Number(number) => number.as_f64(),
            Value::String(text) => text.parse().ok(),
            _ => None,
        })
    };
    if let (Some(x), Some(y)) = (number("x"), number("y")) {
        let content = layout_items::column(children(ui, node, env));
        let bottom = prop(node, "anchor") == "bottom";
        return kit::item(Vec2::ZERO, move |ui, _| {
            let view = ui.clip_rect();
            let point = if bottom {
                Pos2::new(view.left(), view.bottom())
            } else {
                Pos2::new(x as f32, y as f32)
            };
            kit::lay_point(ui, view, point, content);
        });
    }
    if prop(node, "snap") == "corners" {
        let content = layout_items::column(children(ui, node, env));
        let margin = margins(&prop(node, "margin"));
        let start = match &*prop(node, "corner") {
            "top-left" => 0,
            "top-right" => 1,
            "bottom-left" => 2,
            _ => 3,
        };
        let key = node.key.clone();
        return kit::item(Vec2::ZERO, move |ui, _| {
            let view = ui.clip_rect();
            let id = Id::new(("skin-snap", &key));
            let mut state = ui
                .ctx()
                .data(|data| data.get_temp::<SnapState>(id))
                .unwrap_or(SnapState {
                    corner: start,
                    pos: None,
                    size: Vec2::new(1.0, 1.0),
                });
            let corners = snap_corners(view, state.size, margin);
            let mut pos = state.pos.unwrap_or(corners[state.corner]);
            let response = ui.interact(Rect::from_min_size(pos, state.size), id, Sense::drag());
            if response.dragged() {
                pos += response.drag_delta();
                pos = pos.clamp(view.min, view.max - state.size);
                state.corner = nearest_corner(&corners, pos);
            } else {
                let target = corners[state.corner];
                let step = 1.0 - (-ui.ctx().input(|input| input.stable_dt()) * 18.0).exp();
                let next = pos + (target - pos) * step;
                if (target - next).length() < 0.5 {
                    pos = target;
                } else {
                    pos = next;
                    ui.ctx().request_repaint();
                }
            }
            let placed = kit::lay_point(ui, view, pos, content);
            state.pos = Some(pos);
            state.size = placed.size();
            crate::components::mark_mini_player(ui.ctx(), placed);
            ui.ctx().data_mut(|data| data.insert_temp(id, state));
        });
    }
    let content = layout_items::column(children(ui, node, env))
        .align(FlexAlign::Center)
        .padding(Insets::all(12.0));
    kit::item(Vec2::ZERO, move |ui, _| {
        let view = ui.clip_rect();
        kit::lay_in(ui, view, content);
    })
}

#[derive(Clone, Copy)]
struct SnapState {
    corner: usize,
    pos: Option<Pos2>,
    size: Vec2,
}

fn margins(text: &str) -> [f32; 4] {
    let values: Vec<f32> = text
        .split_whitespace()
        .filter_map(|part| part.parse().ok())
        .collect();
    match values.as_slice() {
        [all] => [*all; 4],
        [top, right, bottom, left] => [*top, *right, *bottom, *left],
        _ => [24.0; 4],
    }
}

fn snap_corners(view: Rect, size: Vec2, [top, right, bottom, left]: [f32; 4]) -> [Pos2; 4] {
    let right_x = view.right() - right - size.x;
    let bottom_y = view.bottom() - bottom - size.y;
    [
        Pos2::new(view.left() + left, view.top() + top),
        Pos2::new(right_x, view.top() + top),
        Pos2::new(view.left() + left, bottom_y),
        Pos2::new(right_x, bottom_y),
    ]
}

fn nearest_corner(corners: &[Pos2; 4], pos: Pos2) -> usize {
    (0..4)
        .min_by(|a, b| {
            (corners[*a] - pos)
                .length_sq()
                .total_cmp(&(corners[*b] - pos).length_sq())
        })
        .unwrap_or(3)
}

pub fn set_player_video(context: &crate::gfx::Context, texture: Option<crate::gfx::TextureId>) {
    context.data_mut(|data| match texture {
        Some(texture) => data.insert_temp(Id::new("skin-player-video"), texture),
        None => data.remove::<crate::gfx::TextureId>(Id::new("skin-player-video")),
    });
}

struct CardEntry {
    parent: Arc<Vec<Resolved>>,
    sig: CardSig,
    origin: Pos2,
    clip: Rect,
    shapes: Vec<egui::epaint::ClippedShape>,
    registers: Vec<(u64, Rect)>,
    states: Vec<(String, Rect, u8)>,
    claims: Vec<(String, SkinTarget)>,
    touches: Vec<crate::capture::Touch>,
}

#[derive(Clone, PartialEq)]
struct CardSig {
    size: [u32; 2],
    ppp: u32,
    metrics: Option<crate::metrics::MetricsKey>,
    generation: u64,
    epoch: u64,
}

const CARD_CACHE_LIMIT: usize = 512;

thread_local! {
    static CARDS: RefCell<HashMap<(usize, usize), std::rc::Rc<CardEntry>, crate::fast::Build>> =
        RefCell::new(HashMap::default());
}

fn card_sig(ui: &Ui, size: Vec2) -> CardSig {
    CardSig {
        size: [size.x.to_bits(), size.y.to_bits()],
        ppp: ui.ctx().pixels_per_point().to_bits(),
        metrics: crate::metrics::active_key(),
        generation: crate::stylesheet::generation(),
        epoch: crate::capture::epoch(),
    }
}

fn fully_in_view(clip: Rect, top: f32, height: f32) -> bool {
    top >= clip.top() && top + height <= clip.bottom()
}

fn card_busy(ctx: &crate::gfx::Context, states: &[(String, Rect, u8)]) -> bool {
    let pointer = ctx.pointer_hover_pos();
    states.iter().any(|(key, _, _)| {
        ctx.data(|data| data.get_temp::<(Rect, u8)>(Id::new(("skin-state", key))))
            .is_some_and(|(rect, flags)| {
                flags & (crate::stylesheet::FOCUSED | crate::stylesheet::PRESSED) != 0
                    || pointer.is_some_and(|pos| rect.contains(pos))
            })
    })
}

fn cached_card<'a, A: HomeAssets>(
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

fn replay_shapes(
    ui: &mut Ui,
    at: Pos2,
    origin: Pos2,
    clip: Rect,
    shapes: &[egui::epaint::ClippedShape],
) -> Vec2 {
    let delta = at - origin;
    let transform = egui::emath::TSTransform::from_translation(delta);
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

fn replay_card(ui: &mut Ui, rect: Rect, entry: &CardEntry) {
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

struct StripEntry {
    parent: Arc<Vec<Resolved>>,
    sig: StripSig,
    origin: Pos2,
    clip: Rect,
    used: Vec2,
    shapes: Vec<egui::epaint::ClippedShape>,
    registers: Vec<(u64, Rect)>,
    states: Vec<(String, Rect, u8)>,
    claims: Vec<(String, SkinTarget)>,
    touches: Vec<crate::capture::Touch>,
    regions: Vec<crate::ScrollRegion>,
}

#[derive(Clone, PartialEq)]
struct StripSig {
    card: CardSig,
    shown: u32,
    reach: u32,
}

thread_local! {
    static STRIPS: RefCell<HashMap<usize, std::rc::Rc<StripEntry>, crate::fast::Build>> =
        RefCell::new(HashMap::default());
}

fn cached_strip<A: HomeAssets>(
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
    ui.allocate_rect(Rect::from_min_size(rect.min, entry.used), Sense::hover());
    CARD_HITS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    true
}
