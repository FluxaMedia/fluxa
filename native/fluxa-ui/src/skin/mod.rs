use super::*;
use crate::components::{InputKind, texture_image};
use crate::flex::{Align as FlexAlign, Insets, Justify as FlexJustify};
use crate::gfx::Ui;
use crate::kit::{self, Role, Type, layout_items};
use crate::stylesheet::{Alignment, Justify as StyleJustify};
use fluxa_skin::{Align, Justify, Resolved, ResolvedAction, truthy};
use serde_json::Value;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

mod boxed;
mod cache;
mod expand;
mod flow;
mod grid;
mod indicator;
mod media;
mod pinned;
mod prefetch;
mod scroll;
mod style;
#[cfg(test)]
mod tests;
mod typography;

pub use self::boxed::skin_group_rect;
use self::boxed::*;
use self::cache::*;
use self::expand::*;
use self::flow::*;
pub use self::flow::{grid_windows, row_strip};
use self::grid::*;
use self::indicator::*;
use self::media::*;
pub use self::media::{
    PlayingVideo, VideoRequest, set_player_video, set_playing_video, take_video_request,
};
use self::pinned::*;
use self::prefetch::*;
use self::scroll::*;
use self::style::*;
use self::typography::{hex_color, text};

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

struct Env<'a, A: HomeAssets> {
    view: Cell<Vec2>,
    safe: Insets,
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
            TOP_LAYER.set(true);
            let frame = draw_tree(context, viewport, tree, assets, false, opacity, first_slot);
            TOP_LAYER.set(false);
            Some(frame)
        }
        None => {
            let previous = previous?;
            let opacity = crate::motion::presence(context, id, false, crate::motion::normal());
            if opacity <= 0.001 {
                context.data_mut(|data| data.remove::<Arc<Vec<Resolved>>>(kept));
                crate::motion::reset(context, id);
                return None;
            }
            TOP_LAYER.set(true);
            draw_tree(
                context, viewport, &previous, assets, false, opacity, first_slot,
            );
            TOP_LAYER.set(false);
            None
        }
    }
}

thread_local! {
    static TOP_LAYER: Cell<bool> = const { Cell::new(false) };
}

fn hover_pos(context: &crate::gfx::Context) -> Option<Pos2> {
    if !TOP_LAYER.get() && crate::components::screen_covered(context) {
        return None;
    }
    context.pointer_hover_pos()
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
        view: Cell::new(Vec2::new(viewport.width, viewport.height)),
        safe,
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
            .map(|node| {
                let timer = crate::stats::start();
                let item = build(ui, node, &env);
                timer.add(&crate::stats::BUILD_NS);
                item
            })
            .collect();
        let root = layout_items::column(items).padding(safe);
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
                crate::vars::px("screen-poster-expand-seconds"),
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
                    let wide = base * 1.5 * crate::vars::px("screen-poster-expand-aspect");
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
    let sheet_style = sheet_style(node, env);
    if sheet_style.as_ref().is_some_and(|style| style.hidden) {
        return layout_items::gap(0.0);
    }
    let folded;
    let filled;
    let node = match &sheet_style {
        Some(style) if style.fill_space && !node.fill => {
            filled = Resolved {
                fill: true,
                ..node.clone()
            };
            &filled
        }
        _ => node,
    };
    let node = match &sheet_style {
        Some(style)
            if matches!(node.kind.as_str(), "spacer" | "text" | "input")
                && (node.width.is_none() && style.width.is_some()
                    || node.height.is_none() && style.height.is_some()
                    || node.grow.is_none() && style.grow.is_some()) =>
        {
            folded = Resolved {
                width: node.width.or(style.width),
                height: node.height.or(style.height),
                grow: node.grow.or(style.grow),
                ..node.clone()
            };
            &folded
        }
        _ => node,
    };
    let parent_inner = env.inner.borrow().last().copied().flatten();
    let own_width = node
        .props
        .get("expandBase")
        .and_then(Value::as_f64)
        .map(|base| base as f32)
        .or(node.width)
        .or(node.width_vw.map(|share| share * env.view.get().x))
        .or(sheet_style.as_ref().and_then(|style| style.width))
        .or(sheet_style
            .as_ref()
            .and_then(|style| style.width_vw)
            .map(|share| share * env.view.get().x))
        .or(sheet_style
            .as_ref()
            .and_then(|style| style.width_pct)
            .zip(parent_inner)
            .map(|(share, width)| share * width));
    let sides = node
        .padding
        .or(sheet_style.as_ref().and_then(|style| style.padding_box))
        .map_or(0.0, |[_, right, _, left]| right + left);
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
            inherited: sheet_style.unwrap_or_else(crate::stylesheet::Style::blank),
        }));
    }
    let outer = expanding.map(|state| EXPAND.replace(Some(state)));
    let item = match node.kind.as_str() {
        "row" => {
            let row = layout_items::row(children(ui, node, env));
            if node.props.get("wrap").is_some_and(truthy) {
                row.wrap()
            } else {
                row.pan(key_hash(&node.key))
            }
        }
        "column" => layout_items::column(children(ui, node, env)),
        "stack" => layout_items::stack(children(ui, node, env)),
        "grid" => grid(ui, node, env),
        "page" => layout_items::column(children(ui, node, env))
            .padding(Insets::xy(crate::vars::px("gutter"), 0.0))
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
        env.view.get(),
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
            let reach = crate::vars::px("card-lift");
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

fn is_float(node: &Resolved) -> bool {
    node.props.get("float").is_some_and(truthy)
}

fn icon<'a, A: HomeAssets>(node: &Resolved, env: &'a Env<'a, A>) -> kit::Item<'a> {
    let name = prop(node, "name").into_owned();
    let size = node
        .props
        .get("size")
        .and_then(|value| value.as_f64())
        .map_or(crate::vars::px("icon-button-size") * 0.5, |size| {
            size as f32
        });
    let chain = plain_frames(&env.ancestors.borrow());
    let tint = class_parts(node, None)
        .map(|(rule, modifiers)| {
            crate::stylesheet::within(&chain, || {
                let style = crate::stylesheet::base_with(&rule, &modifiers);
                if style.text_declared {
                    style.text
                } else {
                    crate::metrics::theme().text_primary
                }
            })
        })
        .unwrap_or(crate::metrics::theme().text_primary);
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

fn expanded_width(base: f32, ease: f32) -> f32 {
    base * (1.0 + (crate::vars::px("screen-poster-expand-aspect") * 1.5 - 1.0) * ease)
}

fn children<'a, A: HomeAssets>(
    ui: &mut Ui,
    node: &Resolved,
    env: &'a Env<'a, A>,
) -> Vec<kit::Item<'a>> {
    let (ease, base) = EXPAND.get().unwrap_or((0.0, None));
    let mut shown: Vec<&Resolved> = node
        .children
        .iter()
        .filter(|child| !is_float(child))
        .filter(|child| !is_hidden(child, env))
        .filter(|child| {
            let flag = |name: &str| child.props.get(name).is_some_and(truthy);
            let after = child
                .props
                .get("after")
                .and_then(Value::as_f64)
                .unwrap_or(0.0) as f32;
            (!flag("expandedOnly") || ease > after.max(0.0)) && (!flag("posterOnly") || ease < 0.5)
        })
        .collect();
    if crate::stylesheet::uses_order() {
        shown.sort_by_key(|child| {
            class_parts(child, env.positions.get(&child.key).copied())
                .map_or(0, |(rule, modifiers)| {
                    crate::stylesheet::base_with(&rule, &modifiers).order
                })
        });
    }
    shown
        .into_iter()
        .map(|child| {
            match (
                base,
                child.props.get("expandedOnly").is_some_and(truthy)
                    && matches!(child.kind.as_str(), "image" | "video"),
            ) {
                (Some(base), true) => {
                    let mut sized = child.clone();
                    let base = base as f32;
                    let full = expanded_width(base, 1.0);
                    sized.width = Some(expanded_width(base, ease));
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

fn is_hidden<'a, A: HomeAssets>(node: &Resolved, env: &'a Env<'a, A>) -> bool {
    node.class.is_some() && sheet_style(node, env).is_some_and(|style| style.hidden)
}

fn sheet_style<'a, A: HomeAssets>(
    node: &Resolved,
    env: &'a Env<'a, A>,
) -> Option<crate::stylesheet::Style> {
    let timer = crate::stats::start();
    crate::stats::count(&crate::stats::STYLE_CALLS);
    let (rule, modifiers) = class_parts(node, env.positions.get(&node.key).copied())?;
    let chain = plain_frames(&env.ancestors.borrow());
    let style = Some(crate::stylesheet::within(&chain, || {
        crate::stylesheet::base_with(&rule, &modifiers)
    }));
    timer.add(&crate::stats::STYLE_NS);
    style
}

fn gap_of<'a, A: HomeAssets>(node: &Resolved, env: &'a Env<'a, A>) -> Option<f32> {
    node.gap
        .or_else(|| sheet_style(node, env).and_then(|style| style.gap_set))
}

fn fixed_width<'a, A: HomeAssets>(node: &Resolved, env: &'a Env<'a, A>) -> f32 {
    node.width
        .or_else(|| sheet_style(node, env).and_then(|style| style.width))
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
    let own = sheet_style(child, env);
    let grows = child.grow.is_some() || own.as_ref().is_some_and(|style| style.grow.is_some());
    let sized = child.width.is_some() || own.as_ref().is_some_and(|style| style.width.is_some());
    let shared = parent.kind == "row" && grows && !sized;
    let Some(width) = inherited.filter(|_| shared) else {
        return build(ui, child, env);
    };
    let siblings = parent
        .children
        .iter()
        .filter(|other| !is_float(other) && !is_hidden(other, env));
    let count = siblings.clone().count();
    let reserved: f32 = siblings
        .filter(|other| other.key != child.key)
        .map(|other| fixed_width(other, env))
        .sum::<f32>()
        + gap_of(parent, env).unwrap_or(0.0) * count.saturating_sub(1) as f32;
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
        .or(style.and_then(|style| style.width))
        .or(style
            .and_then(|style| style.width_vw)
            .map(|share| share * view.x));
    let height = node
        .height
        .or(node.height_vh.map(|share| share * view.y))
        .or(style.and_then(|style| style.height))
        .or(style
            .and_then(|style| style.height_vh)
            .map(|share| share * view.y));
    let spans_parent = style.is_some_and(|style| style.width_pct.is_some());
    if aspect.is_some() && height.is_none() && !spans_parent {
        width = width.or(parent);
    }
    if let Some(ratio) = aspect {
        item = item.aspect(ratio);
    }
    if let Some(share) = style
        .and_then(|style| style.width_pct)
        .filter(|_| width.is_none())
    {
        item = item.width_pct(share);
    }
    if let Some(share) = style
        .and_then(|style| style.height_pct)
        .filter(|_| height.is_none())
    {
        item = item.height_pct(share);
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
    if let Some(grow) = node.grow.or(style.and_then(|style| style.grow)) {
        item = item.grow(grow);
    }
    if node.fill {
        item = item.grow(1.0).align_self(FlexAlign::Stretch);
    }
    if let Some(gap) = node.gap.or(style.and_then(|style| style.gap_set)) {
        item = item.gap(gap);
    }
    let padding = node.padding.or(style.and_then(|style| style.padding_box));
    if let Some([top, right, bottom, left]) = padding {
        item = item.padding(Insets {
            left,
            top,
            right,
            bottom,
        });
    }
    let placed = style.and_then(|style| style.place).map(|place| {
        let (cross, main) = if node.kind == "row" {
            (place.vertical, place.horizontal)
        } else {
            (place.horizontal, place.vertical)
        };
        (
            cross,
            main.map(|main| match main {
                Alignment::Center => StyleJustify::Center,
                Alignment::End => StyleJustify::End,
                _ => StyleJustify::Start,
            }),
        )
    });
    if let Some(place) = style.and_then(|style| style.place) {
        let flex = |align: Option<Alignment>| {
            align.map(|align| match align {
                Alignment::Start => FlexAlign::Start,
                Alignment::Center => FlexAlign::Center,
                Alignment::End => FlexAlign::End,
                Alignment::Stretch => FlexAlign::Stretch,
            })
        };
        item = item.placed(flex(place.horizontal), flex(place.vertical));
    }
    let align = node
        .align
        .map(|align| match align {
            Align::Start => Alignment::Start,
            Align::Center => Alignment::Center,
            Align::End => Alignment::End,
            Align::Stretch => Alignment::Stretch,
        })
        .or(style.and_then(|style| style.align))
        .or(placed.and_then(|(cross, _)| cross))
        .or((node.kind == "row").then_some(Alignment::Center));
    if let Some(align) = align {
        item = item.align(match align {
            Alignment::Start => FlexAlign::Start,
            Alignment::Center => FlexAlign::Center,
            Alignment::End => FlexAlign::End,
            Alignment::Stretch => FlexAlign::Stretch,
        });
    }
    let justify = node
        .justify
        .map(|justify| match justify {
            Justify::Start => StyleJustify::Start,
            Justify::Center => StyleJustify::Center,
            Justify::End => StyleJustify::End,
            Justify::SpaceBetween => StyleJustify::SpaceBetween,
            Justify::SpaceAround => StyleJustify::SpaceAround,
            Justify::SpaceEvenly => StyleJustify::SpaceEvenly,
        })
        .or(style.and_then(|style| style.justify))
        .or(placed.and_then(|(_, main)| main));
    if let Some(justify) = justify {
        item = item.justify(match justify {
            StyleJustify::Start => FlexJustify::Start,
            StyleJustify::Center => FlexJustify::Center,
            StyleJustify::End => FlexJustify::End,
            StyleJustify::SpaceBetween => FlexJustify::SpaceBetween,
            StyleJustify::SpaceAround => FlexJustify::SpaceAround,
            StyleJustify::SpaceEvenly => FlexJustify::SpaceEvenly,
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

fn prop<'a>(node: &'a Resolved, key: &str) -> std::borrow::Cow<'a, str> {
    match node.props.get(key) {
        Some(Value::String(text)) => std::borrow::Cow::Borrowed(text.as_str()),
        Some(Value::Bool(true)) => std::borrow::Cow::Borrowed("true"),
        Some(Value::Bool(false)) => std::borrow::Cow::Borrowed("false"),
        Some(other) => std::borrow::Cow::Owned(fluxa_skin::display(other)),
        None => std::borrow::Cow::Borrowed(""),
    }
}

fn input<'a, A: HomeAssets>(ui: &mut Ui, node: &Resolved, env: &'a Env<'a, A>) -> kit::Item<'a> {
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
    let tall = node.height;
    let target = Some(&*field)
        .filter(|field| !field.is_empty())
        .and_then(|field| env.claim_field(&node.key, field, kind));
    let layout = env.layout;
    kit::widget(ui, move |ui| {
        let width = fixed.unwrap_or_else(|| {
            if stretch {
                ui.available_width()
            } else {
                crate::vars::px("horizontal-card-width")
            }
        });
        let mut text = value.clone();
        let response = if matches!(kind, InputKind::Search) {
            crate::components::search_field(
                ui,
                &mut text,
                &hint,
                width,
                crate::vars::px("screen-control-height") + 4.0,
            )
        } else if let (InputKind::Multiline, Some(height)) = (kind, tall) {
            crate::components::text_area(ui, &mut text, &hint, Vec2::new(width, height))
        } else {
            crate::components::text_input(ui, &mut text, &hint, width, kind)
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
