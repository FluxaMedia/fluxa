use super::*;
use crate::components::{ButtonKind, button_auto_width, button_with_text_size};
use crate::dp::Dp;
use crate::flex::{Align as FlexAlign, Insets, Justify as FlexJustify};
use crate::gfx::Ui;
use crate::kit::{self, Role, Type, layout_items};
use fluxa_skin::{Align, Justify, Resolved, ResolvedAction, truthy};
use std::cell::{Cell, RefCell};

struct Env<'a, A: HomeAssets> {
    metrics: UiMetrics,
    assets: &'a RefCell<&'a mut A>,
    actions: &'a RefCell<Vec<(u64, ResolvedAction)>>,
    next: &'a Cell<u64>,
}

impl<A: HomeAssets> Env<'_, A> {
    fn claim(&self, action: Option<&ResolvedAction>, disabled: bool) -> Option<u64> {
        let action = action.filter(|_| !disabled)?;
        let slot = self.next.get();
        if slot >= NODE_SKIN_COUNT {
            return None;
        }
        self.next.set(slot + 1);
        let node = NODE_SKIN_BASE + slot;
        self.actions.borrow_mut().push((node, action.clone()));
        Some(node)
    }
}

pub fn draw_skin(
    context: &crate::gfx::Context,
    viewport: Viewport,
    tree: &[Resolved],
    assets: &mut impl HomeAssets,
) -> Vec<(u64, ResolvedAction)> {
    let metrics = metrics_for_assets(viewport, assets);
    crate::kit::provide(context, metrics);
    let screen = Rect::from_min_max(Pos2::ZERO, Pos2::new(viewport.width, viewport.height));
    context
        .layer_painter(crate::gfx::LayerId::background())
        .rect_filled(screen, 0.0, background());
    let safe = viewport.safe_area();
    let bounds = Rect::from_min_max(
        Pos2::new(safe.left, safe.top),
        Pos2::new(viewport.width - safe.right, viewport.height - safe.bottom),
    );
    let assets = RefCell::new(assets);
    let actions = RefCell::new(Vec::new());
    let next = Cell::new(0);
    let env = Env {
        metrics,
        assets: &assets,
        actions: &actions,
        next: &next,
    };
    crate::gfx::Area::new(Id::new("fluxa-skin-body"))
        .fixed_pos(bounds.min)
        .show(context, |ui| {
            ui.set_max_size(bounds.size());
            ui.set_item_spacing(Vec2::ZERO);
            let items = tree.iter().map(|node| build(ui, node, &env)).collect();
            kit::lay_in(ui, bounds, layout_items::column(items));
        });
    actions.into_inner()
}

fn build<'a, A: HomeAssets>(ui: &mut Ui, node: &Resolved, env: &'a Env<'a, A>) -> kit::Item<'a> {
    let item = match node.kind.as_str() {
        "row" => layout_items::row(children(ui, node, env)),
        "column" => layout_items::column(children(ui, node, env)),
        "stack" => layout_items::stack(children(ui, node, env)),
        "spacer" => spacer(node),
        "text" => text(ui, node),
        "button" => button(ui, node, env),
        "image" => image(ui, node, env),
        "poster_card" => poster_card(ui, node, env),
        "progress" => progress(node, env.metrics),
        _ => layout_items::gap(0.0),
    };
    frame(item, node)
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

fn frame<'a>(mut item: kit::Item<'a>, node: &Resolved) -> kit::Item<'a> {
    if let Some(width) = node.width {
        item = item.width(width);
    }
    if let Some(height) = node.height {
        item = item.height(height);
    }
    if let Some(grow) = node.grow {
        item = item.grow(grow);
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

fn text<'a>(ui: &mut Ui, node: &Resolved) -> kit::Item<'a> {
    let content = node.text.clone().unwrap_or_default();
    let (kind, role) = text_style(node.style.as_deref());
    kit::widget(ui, move |ui| {
        kit::text(content.clone(), kind).role(role).show(ui);
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
    let target = env.claim(node.action.as_ref(), disabled);
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
    let width = node.width.unwrap_or(env.metrics.poster_card_width);
    let size = Vec2::new(width, node.height.unwrap_or(width * 1.5));
    let source = node.src.clone();
    let assets = env.assets;
    kit::widget(ui, move |ui| {
        kit::image(source.as_deref(), size)
            .rounded()
            .show(ui, &mut **assets.borrow_mut());
    })
}

fn poster_card<'a, A: HomeAssets>(
    ui: &mut Ui,
    node: &Resolved,
    env: &'a Env<'a, A>,
) -> kit::Item<'a> {
    let metrics = env.metrics;
    let width = node.width.unwrap_or(metrics.poster_card_width);
    let art = Vec2::new(width, node.height.unwrap_or(width * 1.5));
    let title = prop(node, "title");
    let subtitle = prop(node, "subtitle");
    let image = node
        .props
        .get("image")
        .and_then(|value| value.as_str())
        .map(str::to_owned);
    let target = env.claim(node.action.as_ref(), false);
    let assets = env.assets;
    kit::widget(ui, move |ui| {
        let artwork = kit::widget(ui, |ui| {
            kit::image(image.as_deref(), art)
                .rounded()
                .show(ui, &mut **assets.borrow_mut());
        });
        let title = kit::widget(ui, |ui| {
            kit::text(title.clone(), Type::Body).show(ui);
        });
        let mut parts = vec![artwork, layout_items::gap(Space::S8.px(&metrics)), title];
        if !subtitle.is_empty() {
            parts.push(kit::widget(ui, |ui| {
                kit::text(subtitle.clone(), Type::Meta)
                    .role(Role::Muted)
                    .show(ui);
            }));
        }
        let rect = kit::lay_hug(ui, layout_items::column(parts).width(width));
        if let Some(node) = target.filter(|_| !kit::probing()) {
            let response = ui.interact(rect, Id::new(("skin-card", node)), Sense::click());
            response.focus_node(ui, node);
        }
    })
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

    fn sample_tree(cards: usize) -> Vec<Resolved> {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../fixtures/skins/sample");
        let skin = load_skin(&dir).unwrap();
        let cards: Vec<_> = (0..cards)
            .map(|n| json!({"id": format!("tt{n}"), "itemType": "movie", "title": format!("Title {n}"), "subtitle": "2024", "artworkUrl": null}))
            .collect();
        let data = json!({"home": {"title": "Welcome", "eyebrow": "Tonight", "cards": cards}});
        skin.resolve("home", &data, FormFactor::Desktop)
            .unwrap()
            .unwrap()
    }

    fn render(tree: &[Resolved]) -> (Vec<(u64, ResolvedAction)>, Vec<(u64, Rect)>) {
        let viewport = Viewport::new(1280, 800, UiFormFactor::Desktop);
        let context = crate::gfx::Context::default();
        context.set_fonts(crate::fonts::definitions());
        let mut assets = NoAssets;
        let mut actions = Vec::new();
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
                    |ui| actions = draw_skin(ui.ctx(), viewport, tree, &mut assets),
                )
                .drop_without_applying_deltas();
            targets = crate::take_focus_targets(&context).0;
        }
        (actions, targets)
    }

    #[test]
    fn every_action_gets_a_focus_target_inside_the_screen() {
        let (actions, targets) = render(&sample_tree(4));
        assert_eq!(actions.len(), 3 + 4);
        assert_eq!(targets.len(), actions.len());
        let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(1280.0, 800.0));
        for (node, rect) in &targets {
            assert!(actions.iter().any(|(id, _)| id == node));
            assert!(screen.contains_rect(*rect), "{node} at {rect:?}");
            assert!(rect.width() > 0.0 && rect.height() > 0.0);
        }
    }

    #[test]
    fn buttons_sit_in_a_row_without_overlapping() {
        let (_, targets) = render(&sample_tree(0));
        let mut rects: Vec<Rect> = targets.iter().map(|(_, rect)| *rect).collect();
        rects.sort_by(|a, b| a.min.x.total_cmp(&b.min.x));
        assert_eq!(rects.len(), 3);
        assert!(rects.windows(2).all(|pair| pair[0].max.x <= pair[1].min.x));
        assert!(
            rects
                .windows(2)
                .all(|pair| (pair[0].min.y - pair[1].min.y).abs() < 1.0)
        );
    }

    #[test]
    fn cards_in_a_row_do_not_overlap() {
        let (_, targets) = render(&sample_tree(5));
        let mut cards: Vec<Rect> = targets.iter().skip(3).map(|(_, rect)| *rect).collect();
        cards.sort_by(|a, b| a.min.x.total_cmp(&b.min.x));
        assert_eq!(cards.len(), 5);
        assert!(cards.windows(2).all(|pair| pair[0].max.x <= pair[1].min.x));
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
}
