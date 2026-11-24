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
    skin.resolve("home", &json!({}), &mut Default::default())
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
    skin.resolve("home", &data, &mut Default::default())
        .unwrap()
        .unwrap()
}

fn render(tree: &[Resolved]) -> (Vec<(u64, ResolvedAction)>, Vec<(u64, Rect)>) {
    render_in(tree, Viewport::new(1280, 800, UiFormFactor::Desktop))
}

fn render_in(
    tree: &[Resolved],
    viewport: Viewport,
) -> (Vec<(u64, ResolvedAction)>, Vec<(u64, Rect)>) {
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
        .resolve("home", &data, &mut Default::default())
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

#[test]
fn grid_column_width_from_css_fits_as_many_columns_as_the_width_allows() {
    crate::reload_styles_with(".autogrid { column-width: 200px; gap: 10px; }").unwrap();
    let cell = |index: usize| {
        json!({
            "type": "row",
            "padding": 8,
            "action": {"name": "choose", "args": {"key": "k", "value": index.to_string()}},
            "children": [{"type": "text", "text": format!("c{index}")}]
        })
    };
    let tree = skin_from(json!({
        "type": "grid",
        "class": "autogrid",
        "children": (0..12).map(cell).collect::<Vec<_>>()
    }));
    let (_, rects) = render(&tree);
    let mut lefts: Vec<i32> = rects
        .iter()
        .map(|(_, rect)| rect.min.x.round() as i32)
        .collect();
    lefts.sort_unstable();
    lefts.dedup();
    crate::metrics::enter(Viewport::new(1280, 800, UiFormFactor::Desktop));
    let inner = 1280.0
        - crate::vars::px("gutter") * 2.0
        - crate::sidebar::side_inset(1280.0, UiFormFactor::Desktop);
    let expected = (((inner + 10.0) / 210.0).floor() as usize).max(1);
    assert_eq!(lefts.len(), expected, "{lefts:?}");
}

#[test]
fn strip_takes_its_gap_from_the_class() {
    crate::reload_styles_with(".strip-gap { gap: 24px; }").unwrap();
    let cell = |index: usize| {
        json!({
            "type": "row",
            "width": 100,
            "height": 40,
            "action": {"name": "choose", "args": {"key": "k", "value": index.to_string()}},
            "children": [{"type": "text", "text": format!("c{index}")}]
        })
    };
    let tree = skin_from(json!({
        "type": "scroll",
        "class": "strip-gap",
        "children": (0..3).map(cell).collect::<Vec<_>>()
    }));
    let (_, rects) = render(&tree);
    let mut lefts: Vec<f32> = rects.iter().map(|(_, rect)| rect.min.x).collect();
    lefts.sort_by(f32::total_cmp);
    assert_eq!(lefts.len(), 3);
    assert_eq!(lefts[1] - lefts[0], 124.0);
    assert_eq!(lefts[2] - lefts[1], 124.0);
}

#[test]
fn row_children_center_across_unless_the_class_says_otherwise() {
    let tree: Vec<Resolved> = serde_json::from_value::<fluxa_skin::Node>(json!({
        "type": "row", "padding": 0,
        "children": [
            {"type": "fixed", "width": 20, "height": 60},
            {"type": "row", "children": [], "width": 20, "height": 20, "action": {"name": "back"}}
        ]
    }))
    .map(|node| {
        fluxa_skin::resolve(
            &node,
            &Default::default(),
            &json!({}),
            &mut Default::default(),
        )
        .unwrap()
    })
    .unwrap();
    let (_, targets) = render(&tree);
    assert_eq!(targets.len(), 1);
    assert_eq!(targets[0].1.min.y, 20.0);
}

fn sized_target(css: &str, class: &str) -> Rect {
    crate::reload_styles_with(css).unwrap();
    let tree = skin_from(json!({
        "type": "column",
        "children": [{
            "type": "row",
            "class": class,
            "action": {"name": "back"},
            "children": []
        }]
    }));
    let (_, targets) = render(&tree);
    assert_eq!(targets.len(), 1);
    targets[0].1
}

#[test]
fn ratio_with_only_a_width_sets_the_height() {
    let rect = sized_target(".thumb { width: 160px; aspect-ratio: 16 / 9; }", "thumb");
    assert_eq!(rect.size(), Vec2::new(160.0, 90.0));
}

#[test]
fn ratio_with_only_a_height_sets_the_width() {
    let rect = sized_target(".art { height: 210px; aspect-ratio: 2 / 3; }", "art");
    assert_eq!(rect.size(), Vec2::new(140.0, 210.0));
}

#[test]
fn full_width_row_inside_a_padded_column_stops_at_the_padding() {
    crate::reload_styles_with(
        ".frame { padding: 20px; align-items: flex-start; } .bar { width: 100%; height: 30px; }",
    )
    .unwrap();
    let tree = skin_from(json!({
        "type": "column",
        "class": "frame",
        "children": [{
            "type": "row",
            "class": "bar",
            "action": {"name": "back"},
            "children": []
        }]
    }));
    let (_, targets) = render(&tree);
    assert_eq!(targets.len(), 1);
    let frame_width = 1280.0;
    assert_eq!(targets[0].1.width(), frame_width - 40.0);
}

fn padded_bar(css: &str, class: &str) -> Rect {
    crate::reload_styles_with(&format!(
        ".frame {{ padding: 20px; align-items: flex-start; }} {css}"
    ))
    .unwrap();
    let tree = skin_from(json!({
        "type": "column",
        "class": "frame",
        "children": [{
            "type": "row",
            "class": class,
            "action": {"name": "back"},
            "children": []
        }]
    }));
    let (_, targets) = render(&tree);
    assert_eq!(targets.len(), 1);
    targets[0].1
}

#[test]
fn calc_width_subtracts_from_the_padded_parent() {
    let rect = padded_bar(".bar { width: calc(100% - 10px); height: 30px; }", "bar");
    assert_eq!(rect.width(), 1230.0);
}

#[test]
fn clamp_width_follows_the_parent_between_bounds() {
    let rect = padded_bar(
        ".bar { width: clamp(100px, 50%, 400px); height: 30px; }",
        "bar",
    );
    assert_eq!(rect.width(), 400.0);
    let rect = padded_bar(
        ".bar { width: clamp(100px, 10%, 400px); height: 30px; }",
        "bar",
    );
    assert_eq!(rect.width(), 124.0);
}

#[test]
fn viewport_and_rem_units_mix_without_a_parent_share() {
    let rect = padded_bar(".bar { width: calc(50vw + 1rem); height: 30px; }", "bar");
    assert_eq!(rect.width(), 640.0 + 16.0);
    let rect = padded_bar(".bar { width: min(50vw, 300px); height: 30px; }", "bar");
    assert_eq!(rect.width(), 300.0);
}

#[test]
fn calc_height_in_a_stack_is_measured_from_the_content_box() {
    crate::reload_styles_with(
        ".frame { padding: 20px; height: 100vh; } .bar { width: 50px; height: calc(100% - 10px); }",
    )
    .unwrap();
    let tree = skin_from(json!({
        "type": "stack",
        "class": "frame",
        "children": [{
            "type": "row",
            "class": "bar",
            "action": {"name": "back"},
            "children": []
        }]
    }));
    let (_, targets) = render(&tree);
    assert_eq!(targets[0].1.height(), 800.0 - 40.0 - 10.0);
}

#[test]
fn a_later_width_replaces_an_earlier_one_of_another_unit() {
    let css = ".bar { width: 200px; height: 30px; } .wide { width: 100%; } .calc { width: calc(100% - 40px); } .fixed { width: 90px; }";
    assert_eq!(padded_bar(css, "bar wide").width(), 1240.0);
    assert_eq!(padded_bar(css, "bar calc").width(), 1200.0);
    assert_eq!(padded_bar(css, "bar wide fixed").width(), 90.0);
    assert_eq!(padded_bar(css, "bar calc wide").width(), 1240.0);
    assert_eq!(padded_bar(css, "bar wide calc").width(), 1200.0);
}

#[test]
fn full_size_root_layer_fills_the_viewport_without_viewport_units() {
    crate::reload_styles_with(".pct-box { width: 100%; height: 100%; }").unwrap();
    let tree = skin_from(json!({
        "type": "stack",
        "class": "pct-box",
        "children": [{
            "type": "column",
            "class": "pct-box",
            "children": [{
                "type": "row",
                "class": "pct-box",
                "action": {"name": "back"},
                "children": []
            }]
        }]
    }));
    let (_, targets) = render(&tree);
    assert_eq!(targets[0].1.size(), Vec2::new(1280.0, 800.0));
}

fn corner_tree(stack_class: &str, child_class: &str) -> Vec<Resolved> {
    skin_from(json!({
        "type": "column",
        "children": [{
            "type": "stack",
            "class": stack_class,
            "children": [{
                "type": "row",
                "class": child_class,
                "action": {"name": "back"},
                "children": []
            }]
        }]
    }))
}

#[test]
fn stack_class_place_moves_its_children_to_the_corner() {
    crate::reload_styles_with(
        ".scene { width: 400px; height: 300px; place: bottom right; } .tile { width: 50px; height: 20px; }",
    )
    .unwrap();
    let (_, targets) = render(&corner_tree("scene", "tile"));
    assert_eq!(targets[0].1.max, Pos2::new(400.0, 300.0));
}

#[test]
fn stack_child_class_place_overrides_the_stack_default() {
    crate::reload_styles_with(
        ".scene { width: 400px; height: 300px; } .tile { width: 50px; height: 20px; place: bottom right; }",
    )
    .unwrap();
    let (_, targets) = render(&corner_tree("scene", "tile"));
    assert_eq!(targets[0].1.max, Pos2::new(400.0, 300.0));
}

#[test]
fn stack_children_stay_top_left_without_a_place() {
    crate::reload_styles_with(
        ".scene { width: 400px; height: 300px; } .tile { width: 50px; height: 20px; }",
    )
    .unwrap();
    let (_, targets) = render(&corner_tree("scene", "tile"));
    assert_eq!(targets[0].1.min, Pos2::ZERO);
}

#[test]
fn column_children_stretch_across_unless_the_class_says_otherwise() {
    crate::reload_styles_with(
        ".stack-col { width: 300px; } .start-col { width: 300px; align-items: flex-start; }",
    )
    .unwrap();
    let tile = |class: &str| {
        skin_from(json!({
            "type": "column",
            "class": class,
            "children": [{"type": "row", "height": 20, "action": {"name": "back"}, "children": []}]
        }))
    };
    let (_, stretched) = render(&tile("stack-col"));
    assert_eq!(stretched[0].1.width(), 300.0);
    let (_, started) = render(&tile("start-col"));
    assert_eq!(started[0].1.width(), 0.0);
}

fn sweep_positions() -> Vec<Option<Pos2>> {
    let mut path = vec![Some(Pos2::new(200.0, 300.0)); 4];
    for step in 0..14 {
        path.push(Some(Pos2::new(
            60.0 + step as f32 * 85.0,
            160.0 + step as f32 * 30.0,
        )));
        path.push(Some(Pos2::new(
            60.0 + step as f32 * 85.0,
            160.0 + step as f32 * 30.0,
        )));
        path.push(Some(Pos2::new(
            60.0 + step as f32 * 85.0,
            160.0 + step as f32 * 30.0,
        )));
    }
    path.extend(vec![Some(Pos2::new(640.0, 400.0)); 4]);
    path.push(None);
    path.extend(vec![None; 3]);
    path
}

struct Dump {
    output: Option<crate::gfx::RunOutput>,
    focus: Vec<(u64, Rect)>,
    targets: String,
}

impl Drop for Dump {
    fn drop(&mut self) {
        if let Some(output) = self.output.take() {
            output.drop_without_applying_deltas();
        }
    }
}

fn run_sweep(mode: replay::Mode, tree: &Arc<Vec<Resolved>>, path: &[Option<Pos2>]) -> Vec<Dump> {
    replay::force_mode(Some(mode));
    let viewport = Viewport::new(1280, 800, UiFormFactor::Desktop);
    let context = crate::gfx::Context::default();
    context.set_fonts(crate::fonts::definitions());
    let mut assets = NoAssets;
    let mut last = None;
    let mut dumps = Vec::new();
    for (tick, pos) in path.iter().enumerate() {
        let mut events = Vec::new();
        if *pos != last {
            events.push(match pos {
                Some(pos) => crate::gfx::Event::PointerMoved(*pos),
                None => crate::gfx::Event::PointerGone,
            });
        }
        last = *pos;
        let mut frame = None;
        crate::begin_focus_frame(&context, None);
        let output = context.run_ui(
            crate::gfx::RawInput {
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1280.0, 800.0))),
                time: Some(tick as f64 / 60.0),
                events,
                ..Default::default()
            },
            |ui| frame = Some(draw_skin(ui.ctx(), viewport, tree, &mut assets)),
        );
        let frame = frame.unwrap();
        dumps.push(Dump {
            output: Some(output),
            focus: crate::take_focus_targets(&context).0,
            targets: format!("{:?}{:?}", frame.targets.len(), frame.layout),
        });
    }
    replay::force_mode(None);
    dumps
}

#[test]
fn skipping_frames_keeps_every_frame_identical_across_a_pointer_sweep() {
    let tree = Arc::new(sample_tree(3, 12));
    let path = sweep_positions();
    let reference = run_sweep(replay::Mode::Off, &tree, &path);
    let (skipped_before, _, _) = replay::skipped_builds();
    let skipping = run_sweep(replay::Mode::On, &tree, &path);
    let (skipped_after, _, _) = replay::skipped_builds();
    assert_eq!(reference.len(), skipping.len());
    for (frame, (a, b)) in reference.iter().zip(&skipping).enumerate() {
        assert!(
            a.output
                .as_ref()
                .unwrap()
                .same_paint(b.output.as_ref().unwrap()),
            "paint differs at frame {frame}"
        );
        assert_eq!(a.focus, b.focus, "focus targets differ at frame {frame}");
        assert_eq!(a.targets, b.targets, "layout differs at frame {frame}");
    }
    assert!(skipped_after > skipped_before, "nothing was skipped");
}

struct Rig {
    context: crate::gfx::Context,
    tick: u32,
}

struct Shot {
    tree: Arc<Vec<Resolved>>,
    width: u32,
    opacity: f32,
    reserved: u64,
    events: Vec<crate::gfx::Event>,
    ppp: f32,
}

impl Rig {
    fn new() -> Self {
        let context = crate::gfx::Context::default();
        context.set_fonts(crate::fonts::definitions());
        replay::force_mode(Some(replay::Mode::On));
        Self { context, tick: 0 }
    }

    fn frame(&mut self, shot: &Shot) -> bool {
        let viewport = Viewport::new(shot.width, 800, UiFormFactor::Desktop);
        let before = replay::skipped_builds().0;
        let mut assets = NoAssets;
        crate::begin_focus_frame(&self.context, None);
        self.context.set_ppp_for_test(shot.ppp);
        self.tick += 1;
        let output = self.context.run_ui(
            crate::gfx::RawInput {
                screen_rect: Some(Rect::from_min_size(
                    Pos2::ZERO,
                    Vec2::new(shot.width as f32, 800.0),
                )),
                time: Some(self.tick as f64 / 60.0),
                events: shot.events.clone(),
                ..Default::default()
            },
            |ui| {
                if shot.reserved > 0 {
                    draw_skin_pinned(ui.ctx(), viewport, &shot.tree, &mut assets, shot.reserved);
                } else {
                    draw_skin_faded(ui.ctx(), viewport, &shot.tree, &mut assets, shot.opacity);
                }
            },
        );
        output.drop_without_applying_deltas();
        let _ = crate::take_focus_targets(&self.context);
        replay::skipped_builds().0 > before
    }
}

impl Drop for Rig {
    fn drop(&mut self) {
        replay::force_mode(None);
    }
}

impl Shot {
    fn new(tree: &Arc<Vec<Resolved>>) -> Self {
        Self {
            tree: tree.clone(),
            width: 1280,
            opacity: 1.0,
            reserved: 0,
            events: Vec::new(),
            ppp: 1.0,
        }
    }
}

fn settle(rig: &mut Rig, shot: &Shot) {
    for _ in 0..40 {
        rig.frame(shot);
    }
    assert!(rig.frame(shot), "an unchanged frame should be skipped");
}

#[test]
fn unchanged_frame_is_skipped_after_it_settles() {
    let tree = Arc::new(sample_tree(2, 6));
    let mut rig = Rig::new();
    settle(&mut rig, &Shot::new(&tree));
}

#[test]
fn a_new_tree_is_never_skipped() {
    let tree = Arc::new(sample_tree(2, 6));
    let mut rig = Rig::new();
    settle(&mut rig, &Shot::new(&tree));
    let other = Arc::new(tree.as_ref().clone());
    assert!(!rig.frame(&Shot::new(&other)));
}

#[test]
fn a_resized_viewport_is_never_skipped() {
    let tree = Arc::new(sample_tree(2, 6));
    let mut rig = Rig::new();
    settle(&mut rig, &Shot::new(&tree));
    let mut shot = Shot::new(&tree);
    shot.width = 1100;
    assert!(!rig.frame(&shot));
}

#[test]
fn a_scale_change_is_never_skipped() {
    let tree = Arc::new(sample_tree(2, 6));
    let mut rig = Rig::new();
    settle(&mut rig, &Shot::new(&tree));
    let mut shot = Shot::new(&tree);
    shot.ppp = 2.0;
    assert!(!rig.frame(&shot));
}

#[test]
fn a_different_opacity_is_never_skipped() {
    let tree = Arc::new(sample_tree(2, 6));
    let mut rig = Rig::new();
    settle(&mut rig, &Shot::new(&tree));
    let mut shot = Shot::new(&tree);
    shot.opacity = 0.5;
    assert!(!rig.frame(&shot));
}

#[test]
fn a_moved_pointer_is_never_skipped() {
    let tree = Arc::new(sample_tree(2, 6));
    let mut rig = Rig::new();
    settle(&mut rig, &Shot::new(&tree));
    let mut shot = Shot::new(&tree);
    shot.events = vec![crate::gfx::Event::PointerMoved(Pos2::new(300.0, 300.0))];
    assert!(!rig.frame(&shot));
    shot.events.clear();
    assert!(!rig.frame(&shot));
    assert!(rig.frame(&shot));
}

#[test]
fn a_key_event_is_never_skipped() {
    let tree = Arc::new(sample_tree(2, 6));
    let mut rig = Rig::new();
    settle(&mut rig, &Shot::new(&tree));
    let mut shot = Shot::new(&tree);
    shot.events = vec![crate::gfx::Event::Key {
        key: crate::gfx::Key::ArrowDown,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: Default::default(),
    }];
    assert!(!rig.frame(&shot));
}

#[test]
fn a_global_setting_change_is_never_skipped() {
    let tree = Arc::new(sample_tree(2, 6));
    let mut rig = Rig::new();
    settle(&mut rig, &Shot::new(&tree));
    crate::set_liquid_glass(true);
    let skipped = rig.frame(&Shot::new(&tree));
    crate::set_liquid_glass(false);
    assert!(!skipped);
}

#[test]
fn a_stylesheet_change_is_never_skipped() {
    let tree = Arc::new(sample_tree(2, 6));
    let mut rig = Rig::new();
    settle(&mut rig, &Shot::new(&tree));
    crate::capture::bump_epoch();
    assert!(!rig.frame(&Shot::new(&tree)));
}

#[test]
fn a_different_node_slot_range_is_never_skipped() {
    let tree = Arc::new(sample_tree(2, 6));
    let mut rig = Rig::new();
    settle(&mut rig, &Shot::new(&tree));
    let mut shot = Shot::new(&tree);
    shot.reserved = 4;
    assert!(!rig.frame(&shot));
}

#[test]
fn notched_polygon_triangulates_without_overlap() {
    let points = [
        Pos2::new(0.0, 0.0),
        Pos2::new(10.0, 0.0),
        Pos2::new(10.0, 10.0),
        Pos2::new(5.0, 4.0),
        Pos2::new(0.0, 10.0),
    ];
    let indices = triangulate(&points);
    assert_eq!(indices.len(), 9);
    let area: f32 = indices
        .chunks(3)
        .map(|tri| {
            cross_area(
                points[tri[0] as usize],
                points[tri[1] as usize],
                points[tri[2] as usize],
            )
        })
        .sum();
    assert!((area - 70.0).abs() < 0.01);
}

fn cross_area(a: Pos2, b: Pos2, c: Pos2) -> f32 {
    ((b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x)).abs() / 2.0
}

#[test]
fn bleed_reaches_the_screen_edge_while_inset_keeps_content_in_the_safe_area() {
    crate::reload_styles_with(
        ".backdrop { bleed: top left right; height: 200px; } .content { inset: left right; height: 40px; } .leaf { width: 10px; height: 10px; }",
    )
    .unwrap();
    let tree = skin_from(json!({
        "type": "column",
        "children": [{
            "type": "stack",
            "class": "backdrop",
            "action": {"name": "back"},
            "children": [{
                "type": "row",
                "class": "content",
                "children": [{
                    "type": "row",
                    "class": "leaf",
                    "action": {"name": "back"},
                    "children": []
                }]
            }]
        }]
    }));
    let viewport = Viewport::new(1280, 800, UiFormFactor::Desktop).with_safe_area(30.0, 40.0, 50.0);
    let (_, targets) = render_in(&tree, viewport);
    assert_eq!(targets.len(), 2);
    let rects: Vec<Rect> = targets.iter().map(|(_, rect)| *rect).collect();
    let backdrop = rects
        .iter()
        .copied()
        .max_by(|a, b| a.area().total_cmp(&b.area()))
        .unwrap();
    let leaf = rects
        .iter()
        .copied()
        .min_by(|a, b| a.area().total_cmp(&b.area()))
        .unwrap();
    assert_eq!(backdrop.min, Pos2::ZERO);
    assert_eq!(backdrop.max.x, 1280.0);
    assert_eq!(backdrop.height(), 230.0);
    assert_eq!(leaf.min.x, 40.0);
}
