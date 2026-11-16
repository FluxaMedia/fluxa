use super::*;
use crate::components::{
    ButtonKind, InputKind, button_auto_width, button_with_text_size, texture_image,
};
use crate::dp::Dp;
use crate::flex::{Align as FlexAlign, Insets, Justify as FlexJustify};
use crate::gfx::Ui;
use crate::kit::{self, Role, Type, UiKit, layout_items};
use fluxa_skin::{Align, Justify, Resolved, ResolvedAction, truthy};
use serde_json::Value;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::sync::Mutex;

#[derive(Clone, Debug, PartialEq)]
pub enum SkinTarget {
    Action(ResolvedAction),
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
    metrics: UiMetrics,
    view: Vec2,
    assets: &'a RefCell<&'a mut A>,
    targets: &'a RefCell<Vec<(u64, SkinTarget)>>,
    fields: &'a RefCell<Vec<(u64, InputKind)>>,
    layout: &'a RefCell<HomeLayout>,
    next: &'a Cell<u64>,
    claimed: &'a RefCell<HashMap<String, u64>>,
    strips: &'a Cell<usize>,
}

const STRIPS: [&str; 16] = [
    "fluxa-skin-strip-0",
    "fluxa-skin-strip-1",
    "fluxa-skin-strip-2",
    "fluxa-skin-strip-3",
    "fluxa-skin-strip-4",
    "fluxa-skin-strip-5",
    "fluxa-skin-strip-6",
    "fluxa-skin-strip-7",
    "fluxa-skin-strip-8",
    "fluxa-skin-strip-9",
    "fluxa-skin-strip-10",
    "fluxa-skin-strip-11",
    "fluxa-skin-strip-12",
    "fluxa-skin-strip-13",
    "fluxa-skin-strip-14",
    "fluxa-skin-strip-15",
];

impl<A: HomeAssets> Env<'_, A> {
    fn claim(&self, key: &str, action: Option<&ResolvedAction>, disabled: bool) -> Option<u64> {
        let action = action.filter(|_| !disabled)?;
        self.claim_target(key, SkinTarget::Action(action.clone()))
    }

    fn claim_field(&self, key: &str, field: &str, kind: InputKind) -> Option<u64> {
        let node = self.claim_target(key, SkinTarget::Field(field.to_owned()))?;
        let mut fields = self.fields.borrow_mut();
        if !fields.iter().any(|(id, _)| *id == node) {
            fields.push((node, kind));
        }
        Some(node)
    }

    fn claim_target(&self, key: &str, target: SkinTarget) -> Option<u64> {
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

    fn strip(&self) -> Option<&'static str> {
        let index = self.strips.get();
        let name = STRIPS.get(index).copied()?;
        self.strips.set(index + 1);
        Some(name)
    }
}

pub fn draw_skin(
    context: &crate::gfx::Context,
    viewport: Viewport,
    tree: &[Resolved],
    assets: &mut impl HomeAssets,
) -> SkinFrame {
    let metrics = metrics_for_assets(viewport, assets);
    crate::kit::provide(context, metrics);
    let screen = Rect::from_min_max(Pos2::ZERO, Pos2::new(viewport.width, viewport.height));
    context
        .layer_painter(crate::gfx::LayerId::background())
        .rect_filled(screen, 0.0, background());
    let safe = viewport.safe_area();
    let assets = RefCell::new(assets);
    let targets = RefCell::new(Vec::new());
    let fields = RefCell::new(Vec::new());
    let layout = RefCell::new(HomeLayout::default());
    let next = Cell::new(0);
    let claimed = RefCell::new(HashMap::new());
    let strips = Cell::new(0);
    let env = Env {
        metrics,
        view: Vec2::new(viewport.width, viewport.height),
        assets: &assets,
        targets: &targets,
        fields: &fields,
        layout: &layout,
        next: &next,
        claimed: &claimed,
        strips: &strips,
    };
    prefetch(&env, tree);
    kit::screen_body(context, viewport, "fluxa-skin", (), |ui| {
        let items = tree.iter().map(|node| build(ui, node, &env)).collect();
        let root = layout_items::column(items).padding(Insets::xy(
            safe.left.max(safe.right),
            safe.top.max(safe.bottom),
        ));
        kit::lay(ui, vec![root]);
    });
    if let Ok(mut kinds) = FIELD_KINDS.lock() {
        *kinds = fields.into_inner();
    }
    SkinFrame {
        targets: targets.into_inner(),
        layout: layout.into_inner(),
    }
}

fn prefetch<A: HomeAssets>(env: &Env<'_, A>, nodes: &[Resolved]) {
    for node in nodes {
        let (url, size) = match node.kind.as_str() {
            "image" => (node.src.clone(), Vec2::new(env.view.x, env.view.y * 0.62)),
            "poster_card" => {
                let (width, height) = card_size(node, env.metrics);
                (
                    node.props
                        .get("image")
                        .and_then(|value| value.as_str())
                        .map(str::to_owned),
                    Vec2::new(width, height),
                )
            }
            _ => (None, Vec2::ZERO),
        };
        if let Some(url) = url.filter(|url| !url.is_empty()) {
            let target = crate::artwork_target_size(size, 1.0);
            env.assets
                .borrow_mut()
                .prefetch_for(Some(&url), target, ArtworkPriority::Prefetch);
        }
        prefetch(env, &node.children);
    }
}

fn build<'a, A: HomeAssets>(ui: &mut Ui, node: &Resolved, env: &'a Env<'a, A>) -> kit::Item<'a> {
    let item = match node.kind.as_str() {
        "row" => layout_items::row(children(ui, node, env)),
        "column" => layout_items::column(children(ui, node, env)),
        "stack" => layout_items::stack(children(ui, node, env)),
        "scroll" => scroll(ui, node, env),
        "pager" => pager(ui, node, env),
        "spacer" => spacer(node),
        "text" => text(ui, node, env),
        "button" => button(ui, node, env),
        "image" => image(ui, node, env),
        "poster_card" => poster_card(ui, node, env),
        "progress" => progress(node, env.metrics),
        "title_logo" => title_logo(ui, node, env),
        "hero_scrim" => hero_scrim(env.metrics),
        "avatar" => avatar(node, env),
        "icon_button" => icon_button(ui, node, env),
        "icon" => icon(node, env),
        "input" => input(ui, node, env),
        "toggle" => toggle(ui, node, env),
        "dropdown" => dropdown(ui, node, env),
        _ => layout_items::gap(0.0),
    };
    frame(boxed(item, node, env), node, env.view)
}

fn boxed<'a, A: HomeAssets>(
    item: kit::Item<'a>,
    node: &Resolved,
    env: &'a Env<'a, A>,
) -> kit::Item<'a> {
    let container = matches!(node.kind.as_str(), "row" | "column" | "stack");
    let class = node
        .class
        .clone()
        .filter(|class| crate::stylesheet::has_rule(class));
    let target = if container {
        env.claim(&node.key, node.action.as_ref(), false)
    } else {
        None
    };
    if !container || (class.is_none() && target.is_none()) {
        return item;
    }
    let metrics = env.metrics;
    let key = node.key.clone();
    item.painted(move |ui, rect| {
        let sense = if target.is_some() {
            Sense::click()
        } else {
            Sense::hover()
        };
        let response = ui.interact(rect, Id::new(("skin-box", &key)), sense);
        if let Some(class) = &class {
            let style = crate::stylesheet::resolve(
                ui.ctx(),
                class,
                response.id,
                crate::stylesheet::State {
                    hovered: response.hovered(),
                    pressed: response.is_pointer_button_down_on(),
                    focused: response.has_focus(),
                    enabled: true,
                    selected: false,
                },
                Some(&metrics),
            );
            let radius = style.radius.of(rect.height());
            let painter = ui.painter();
            painter.rect_filled(rect, radius, style.fill.gamma_multiply(style.opacity));
            if style.stroke > 0.0 && style.border.a() > 0 {
                painter.rect_stroke(
                    rect,
                    radius,
                    crate::gfx::Stroke::new(style.stroke, style.border),
                    crate::gfx::StrokeKind::Inside,
                );
            }
        }
        if let Some(node) = target.filter(|_| !kit::probing()) {
            response.focus_node(ui, node);
        }
    })
}

fn icon<'a, A: HomeAssets>(node: &Resolved, env: &'a Env<'a, A>) -> kit::Item<'a> {
    let metrics = env.metrics;
    let name = prop(node, "name");
    let size = node
        .props
        .get("size")
        .and_then(|value| value.as_f64())
        .map_or(metrics.icon_button_size * 0.5, |size| size as f32);
    let tint = node
        .class
        .as_deref()
        .filter(|class| crate::stylesheet::has_rule(class))
        .map(crate::stylesheet::text_color)
        .unwrap_or(metrics.text_primary);
    let assets = env.assets;
    kit::item(Vec2::splat(size), move |ui, rect| {
        if let Some(texture) = assets.borrow().icon(&name) {
            ui.painter()
                .image(texture, rect, crate::paint::full_uv(), tint);
        }
    })
}

fn children<'a, A: HomeAssets>(
    ui: &mut Ui,
    node: &Resolved,
    env: &'a Env<'a, A>,
) -> Vec<kit::Item<'a>> {
    node.children
        .iter()
        .map(|child| build(ui, child, env))
        .collect()
}

fn frame<'a>(mut item: kit::Item<'a>, node: &Resolved, view: Vec2) -> kit::Item<'a> {
    let width = node.width.or(node.width_vw.map(|share| share * view.x));
    let height = node.height.or(node.height_vh.map(|share| share * view.y));
    if let Some(width) = width {
        item = item.width(width);
    }
    if let Some(height) = height {
        item = item.height(height);
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
    if let Some(padding) = node.padding {
        item = item.padding(Insets::all(padding));
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

fn text<'a, A: HomeAssets>(ui: &mut Ui, node: &Resolved, env: &'a Env<'a, A>) -> kit::Item<'a> {
    let content = node.text.clone().unwrap_or_default();
    let (kind, role) = text_style(node.style.as_deref());
    if !crate::emoji::contains(&content) {
        return kit::widget(ui, move |ui| {
            kit::text(content.clone(), kind).role(role).show(ui);
        });
    }
    let assets = env.assets;
    kit::widget(ui, move |ui| {
        let theme = ui.tokens();
        crate::emoji::label(
            ui,
            &mut **assets.borrow_mut(),
            &content,
            kind.size(&theme),
            role.color(&theme),
            crate::emoji::Label::default(),
        );
    })
}

fn button<'a, A: HomeAssets>(ui: &mut Ui, node: &Resolved, env: &'a Env<'a, A>) -> kit::Item<'a> {
    let kind = match node.props.get("variant").and_then(|value| value.as_str()) {
        Some("secondary") => ButtonKind::Secondary,
        Some("selected") => ButtonKind::Selected,
        Some("danger") => ButtonKind::Danger,
        _ => ButtonKind::Primary,
    };
    let disabled = node.props.get("disabled").is_some_and(truthy);
    let label = node.text.clone().unwrap_or_default();
    let width = node.width;
    let metrics = env.metrics;
    let text_size = metrics.nav_label_size + Dp::D2.px();
    let target = env.claim(&node.key, node.action.as_ref(), disabled);
    kit::widget(ui, move |ui| {
        let response = match width {
            Some(width) => button_with_text_size(
                ui,
                &label,
                width,
                metrics.screen_control_height,
                kind,
                text_size,
                metrics,
            ),
            None => button_auto_width(ui, &label, kind, text_size, metrics),
        };
        if let Some(node) = target.filter(|_| !kit::probing()) {
            response.focus_node(ui, node);
        }
    })
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
        return kit::item(Vec2::ZERO, move |ui, rect| {
            let mut assets = assets.borrow_mut();
            let target = [rect.width().max(1.0) as u32, rect.height().max(1.0) as u32];
            let texture = assets.texture_for(source.as_deref(), target, ArtworkPriority::Visible);
            if let Some(texture) = texture {
                let size = assets.texture_size(source.as_deref()).unwrap_or(target);
                ui.painter().image(
                    texture,
                    rect,
                    crate::paint::cover_uv_focus(size, rect, [0.5, focus_y]),
                    Color32::WHITE,
                );
            }
        })
        .grow(1.0)
        .align_self(FlexAlign::Stretch);
    }
    let width = node.width.unwrap_or(env.metrics.poster_card_width);
    let size = Vec2::new(width, node.height.unwrap_or(width * 1.5));
    kit::widget(ui, move |ui| {
        kit::image(source.as_deref(), size)
            .rounded()
            .show(ui, &mut **assets.borrow_mut());
    })
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
    let now = ui.ctx().input(|input| input.time());
    let (mut index, mut since) = ui
        .ctx()
        .data(|data| data.get_temp::<(usize, f64)>(id))
        .unwrap_or((0, now));
    if index >= count {
        index = 0;
    }
    if count > 1 && now - since >= interval {
        index = (index + 1) % count;
        since = now;
    }
    ui.ctx()
        .data_mut(|data| data.insert_temp(id, (index, since)));
    if count > 1 {
        ui.ctx()
            .request_repaint_after(std::time::Duration::from_secs_f64(interval));
    }
    build(ui, &node.children[index], env)
}

fn poster_card<'a, A: HomeAssets>(
    ui: &mut Ui,
    node: &Resolved,
    env: &'a Env<'a, A>,
) -> kit::Item<'a> {
    let metrics = env.metrics;
    let collection = !prop(node, "shape").is_empty();
    let (width, art_height) = card_size(node, env.metrics);
    let art = Vec2::new(width, art_height);
    let title = prop(node, "title");
    let subtitle = if collection {
        String::new()
    } else {
        prop(node, "subtitle")
    };
    let title = if prop(node, "hideTitle") == "true" {
        String::new()
    } else {
        title
    };
    let image = node
        .props
        .get("image")
        .and_then(|value| value.as_str())
        .map(str::to_owned);
    let target = env.claim(&node.key, node.action.as_ref(), false);
    let assets = env.assets;
    let motion = prop(node, "motion");
    let motion_on = collection && !motion.is_empty() && prop(node, "motionEnabled") != "false";
    let autoplay = prop(node, "autoplay") != "false";
    let artwork = kit::widget(ui, move |ui| {
        let mut assets = assets.borrow_mut();
        let picture = kit::image(image.as_deref(), art).rounded();
        let picture = if collection {
            picture.contain()
        } else {
            picture
        };
        let response = picture.show(ui, &mut **assets);
        if motion_on && !kit::probing() {
            motion_overlay(ui, response.rect, &motion, autoplay, &mut **assets);
        }
    });
    let mut parts = vec![artwork];
    if !title.is_empty() {
        parts.push(layout_items::gap(Space::S8.px(&metrics)));
        parts.push(kit::widget(ui, move |ui| {
            kit::text(title.clone(), Type::Body).show(ui);
        }));
    }
    if !subtitle.is_empty() {
        parts.push(kit::widget(ui, move |ui| {
            kit::text(subtitle.clone(), Type::Meta)
                .role(Role::Muted)
                .show(ui);
        }));
    }
    layout_items::column(parts)
        .width(width)
        .painted(move |ui, rect| activate(ui, rect, target))
}

fn prop(node: &Resolved, key: &str) -> String {
    node.props
        .get(key)
        .map(fluxa_skin::display)
        .unwrap_or_default()
}

fn progress<'a>(node: &Resolved, metrics: UiMetrics) -> kit::Item<'a> {
    let value = node
        .props
        .get("value")
        .and_then(|value| value.as_f64())
        .unwrap_or(0.0)
        .clamp(0.0, 1.0) as f32;
    let width = node.width.unwrap_or(metrics.poster_card_width);
    let height = node.height.unwrap_or(Space::S4.px(&metrics));
    kit::item(Vec2::new(width, height), move |ui, rect| {
        let radius = height * 0.5;
        ui.painter()
            .rect_filled(rect, radius, metrics.surface_raised);
        let mut filled = rect;
        filled.max.x = rect.min.x + rect.width() * value;
        ui.painter().rect_filled(filled, radius, metrics.accent);
    })
}

fn input<'a, A: HomeAssets>(ui: &mut Ui, node: &Resolved, env: &'a Env<'a, A>) -> kit::Item<'a> {
    let metrics = env.metrics;
    let field = prop(node, "key");
    let hint = prop(node, "placeholder");
    let value = prop(node, "value");
    let kind = match prop(node, "kind").as_str() {
        "search" => InputKind::Search,
        "password" => InputKind::Password,
        "url" => InputKind::Url,
        "number" => InputKind::Number { secret: false },
        "multiline" => InputKind::Multiline,
        _ => InputKind::Text,
    };
    let width = node.width.unwrap_or(metrics.horizontal_card_width);
    let target = Some(field.as_str())
        .filter(|field| !field.is_empty())
        .and_then(|field| env.claim_field(&node.key, field, kind));
    let layout = env.layout;
    kit::widget(ui, move |ui| {
        let mut text = value.clone();
        let response = crate::components::text_input(ui, &mut text, &hint, width, kind, metrics);
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

fn toggle<'a, A: HomeAssets>(ui: &mut Ui, node: &Resolved, env: &'a Env<'a, A>) -> kit::Item<'a> {
    let metrics = env.metrics;
    let field = prop(node, "key");
    let on = node.props.get("value").is_some_and(truthy);
    let label = node.text.clone().unwrap_or_default();
    let flip = ResolvedAction {
        name: "set_setting".to_owned(),
        args: [
            ("key".to_owned(), Value::String(field.clone())),
            ("value".to_owned(), Value::Bool(!on)),
        ]
        .into(),
    };
    let target = env.claim(&node.key, Some(&flip), field.is_empty());
    let name = kit::widget(ui, move |ui| {
        kit::text(label.clone(), Type::Body).show(ui);
    });
    let switch = kit::widget(ui, move |ui| {
        let response = crate::components::switch(ui, Id::new(("skin-switch", &field)), on, metrics);
        if let Some(node) = target.filter(|_| !kit::probing()) {
            response.focus_node(ui, node);
        }
    });
    layout_items::row(vec![name, layout_items::fill(), switch])
        .align(FlexAlign::Center)
        .width(node.width.unwrap_or(metrics.horizontal_card_width))
}

fn options_of(node: &Resolved) -> Vec<(String, String)> {
    let Some(Value::Array(items)) = node.props.get("options") else {
        return Vec::new();
    };
    items
        .iter()
        .filter_map(|item| match item {
            Value::String(text) => Some((text.clone(), text.clone())),
            Value::Object(map) => {
                let value = map.get("value").map(fluxa_skin::display)?;
                let label = map
                    .get("label")
                    .map_or_else(|| value.clone(), fluxa_skin::display);
                Some((value, label))
            }
            _ => None,
        })
        .collect()
}

fn dropdown<'a, A: HomeAssets>(ui: &mut Ui, node: &Resolved, env: &'a Env<'a, A>) -> kit::Item<'a> {
    let metrics = env.metrics;
    let field = prop(node, "key");
    let selected = prop(node, "value");
    let options = options_of(node);
    let width = node.width.unwrap_or(metrics.horizontal_card_width);
    let layout = env.layout;
    let target = env.claim_target(&node.key, SkinTarget::Control);
    kit::widget(ui, move |ui| {
        let (response, choice) = crate::components::dropdown(
            ui,
            ("skin-dropdown", &field),
            &selected,
            &options,
            width,
            metrics.compact,
            metrics,
        );
        if let Some(node) = target.filter(|_| !kit::probing()) {
            response.focus_node(ui, node);
        }
        if let Some(choice) = choice.filter(|_| !kit::probing() && !field.is_empty()) {
            layout.borrow_mut().setting_change = Some((field.clone(), Value::String(choice)));
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

    fn sample_tree(rows: usize, per_row: usize) -> Vec<Resolved> {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../fixtures/skins/sample");
        let skin = load_skin(&dir).unwrap();
        let rows: Vec<_> = (0..rows)
            .map(|row| json!({"title": format!("Row {row}"), "cards": (0..per_row).map(card).collect::<Vec<_>>()}))
            .collect();
        let data = json!({"home": {"title": "Welcome", "eyebrow": "Tonight", "rows": rows}});
        skin.resolve("home", &data, FormFactor::Desktop)
            .unwrap()
            .unwrap()
    }

    fn render(tree: &[Resolved]) -> (Vec<(u64, ResolvedAction)>, Vec<(u64, Rect)>) {
        let viewport = Viewport::new(1280, 800, UiFormFactor::Desktop);
        let context = crate::gfx::Context::default();
        context.set_fonts(crate::fonts::definitions());
        let mut assets = NoAssets;
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
                    |ui| frame = Some(draw_skin(ui.ctx(), viewport, tree, &mut assets)),
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
            fluxa_skin::resolve(&node, &Default::default(), &json!({}), FormFactor::Desktop)
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
            .resolve("home", &data, FormFactor::Desktop)
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
            "type": "button", "text": "Off", "props": {"disabled": true},
            "action": {"name": "back"}
        }))
        .map(|node| {
            fluxa_skin::resolve(&node, &Default::default(), &json!({}), FormFactor::Desktop)
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
                {"type": "toggle", "text": "Autoplay", "props": {"key": "autoplay", "value": false}},
                {"type": "dropdown", "props": {"key": "quality", "value": "hd", "options": ["hd", "sd"]}}
            ]
        }))
        .map(|node| {
            fluxa_skin::resolve(&node, &Default::default(), &json!({}), FormFactor::Desktop)
                .unwrap()
        })
        .unwrap();
        let (actions, targets) = render(&tree);
        assert_eq!(targets.len(), 3);
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

fn scroll<'a, A: HomeAssets>(ui: &mut Ui, node: &Resolved, env: &'a Env<'a, A>) -> kit::Item<'a> {
    let name = env.strip();
    let node = node.clone();
    kit::widget(ui, move |ui| {
        let gap = node.gap.unwrap_or(0.0);
        let offset = name.map_or(0.0, |name| {
            crate::scroll_region::strip_offset(ui.ctx(), name)
        });
        let reach = ui.ctx().content_rect().width();
        let mut x = 0.0;
        let items = node
            .children
            .iter()
            .map(|child| {
                let Some(width) = card_width(child, env) else {
                    return build(ui, child, env);
                };
                let start = x;
                x += width + gap;
                if start + width < offset - reach * 0.5 || start > offset + reach * 1.5 {
                    far_card(child, width, env)
                } else {
                    build(ui, child, env)
                }
            })
            .collect();
        let strip = layout_items::row(items).gap(gap);
        match name {
            Some(name) => {
                crate::components::pan_strip(ui, name, |ui| {
                    kit::lay_hug(ui, strip);
                });
            }
            None => {
                kit::lay_hug(ui, strip);
            }
        }
    })
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

fn card_size(node: &Resolved, metrics: UiMetrics) -> (f32, f32) {
    let (width, height) = match prop(node, "shape").to_ascii_lowercase().as_str() {
        "" => (metrics.poster_card_width, metrics.poster_card_width * 1.5),
        "wide" | "landscape" => (
            metrics.collection_wide_width,
            metrics.collection_wide_height,
        ),
        "square" => (
            metrics.collection_square_size,
            metrics.collection_square_size,
        ),
        _ => (
            metrics.collection_poster_width,
            metrics.collection_poster_height,
        ),
    };
    match node.width {
        Some(custom) => (custom, node.height.unwrap_or(custom * height / width)),
        None => (width, node.height.unwrap_or(height)),
    }
}

fn card_width<A: HomeAssets>(node: &Resolved, env: &Env<'_, A>) -> Option<f32> {
    (node.kind == "poster_card").then(|| card_size(node, env.metrics).0)
}

fn far_card<'a, A: HomeAssets>(node: &Resolved, width: f32, env: &'a Env<'a, A>) -> kit::Item<'a> {
    let target = env.claim(&node.key, node.action.as_ref(), false);
    let height = card_size(node, env.metrics).1;
    kit::item(Vec2::new(width, height), move |ui, rect| {
        activate(ui, rect, target)
    })
}

fn title_logo<'a, A: HomeAssets>(
    ui: &mut Ui,
    node: &Resolved,
    env: &'a Env<'a, A>,
) -> kit::Item<'a> {
    let logo = prop(node, "logo");
    let title = prop(node, "title");
    let width = node.width.unwrap_or(env.metrics.poster_card_width * 1.5);
    let bounds = Vec2::new(width, node.height.unwrap_or(width * 0.35));
    let assets = env.assets;
    kit::widget(ui, move |ui| {
        let found = crate::components::title_logo(
            ui.ctx().pixels_per_point(),
            Some(logo.as_str()).filter(|url| !url.is_empty()),
            bounds,
            ArtworkPriority::Visible,
            &mut **assets.borrow_mut(),
        );
        match found {
            Some((texture, size)) => {
                let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
                texture_image(ui.painter(), texture, rect, full_uv(), Color32::WHITE);
            }
            None => {
                kit::text(title.clone(), Type::Headline).show(ui);
            }
        }
    })
}

fn hero_scrim<'a>(metrics: UiMetrics) -> kit::Item<'a> {
    kit::item(Vec2::ZERO, move |ui, rect| {
        paint_hero_scrim(ui.painter(), rect, metrics);
    })
    .grow(1.0)
    .align_self(FlexAlign::Stretch)
}

fn avatar<'a, A: HomeAssets>(node: &Resolved, env: &'a Env<'a, A>) -> kit::Item<'a> {
    let metrics = env.metrics;
    let size = node
        .props
        .get("size")
        .and_then(|value| value.as_f64())
        .map_or(metrics.icon_button_size, |size| size as f32);
    let image = node
        .props
        .get("image")
        .and_then(|value| value.as_str())
        .map(str::to_owned);
    let name = prop(node, "name");
    let target = env.claim(&node.key, node.action.as_ref(), false);
    let assets = env.assets;
    kit::item(Vec2::splat(size), move |ui, rect| {
        crate::components::avatar(
            ui.painter(),
            &mut **assets.borrow_mut(),
            rect.center(),
            size * 0.5,
            &name,
            image.as_deref(),
            metrics,
        );
        activate(ui, rect, target);
    })
}

fn icon_button<'a, A: HomeAssets>(
    ui: &mut Ui,
    node: &Resolved,
    env: &'a Env<'a, A>,
) -> kit::Item<'a> {
    let metrics = env.metrics;
    let name = prop(node, "icon");
    let disabled = node.props.get("disabled").is_some_and(truthy);
    let target = env.claim(&node.key, node.action.as_ref(), disabled);
    let assets = env.assets;
    kit::widget(ui, move |ui| {
        let icon = assets.borrow().icon(&name);
        let response = crate::components::icon_button(
            ui,
            icon,
            metrics.icon_button_size,
            metrics.text_primary,
            true,
            false,
            !disabled,
        );
        if let Some(node) = target.filter(|_| !kit::probing()) {
            response.focus_node(ui, node);
        }
    })
}
