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
                    screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1280.0, 800.0))),
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
            SkinTarget::Action(action) => Some((node, (*action).clone())),
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
                    screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1280.0, 800.0))),
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
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../shared/skins/default");
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
    assert!(names.iter().filter(|name| **name == "navigate").count() >= 3);
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
