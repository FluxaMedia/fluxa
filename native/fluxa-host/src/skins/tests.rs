use super::*;
use std::collections::BTreeMap;

fn action(name: &str, args: &[(&str, Value)]) -> ResolvedAction {
    ResolvedAction {
        name: name.to_owned(),
        args: args
            .iter()
            .map(|(key, value)| ((*key).to_owned(), value.clone()))
            .collect::<BTreeMap<_, _>>(),
    }
}

#[test]
fn open_detail_carries_id_and_type() {
    let native = native_action(&action(
        "open_detail",
        &[("id", json!("tt1")), ("type", json!("movie"))],
    ));
    assert!(
        matches!(native, Some(NativeAction::Detail { id, item_type, .. }) if id == "tt1" && item_type == "movie")
    );
}

#[test]
fn navigate_rejects_unknown_screens() {
    assert!(native_action(&action("navigate", &[("screen", json!("library"))])).is_some());
    assert!(native_action(&action("navigate", &[("screen", json!("nowhere"))])).is_none());
}

#[test]
fn missing_arguments_produce_no_action() {
    assert!(native_action(&action("play", &[("id", json!("tt1"))])).is_none());
}

#[test]
fn setting_values_keep_their_type() {
    let native = native_action(&action(
        "set_setting",
        &[("key", json!("autoplay")), ("value", json!(true))],
    ));
    assert!(
        matches!(native, Some(NativeAction::SettingsChange { value, .. }) if value == json!(true))
    );
}

#[test]
fn extension_actions_are_not_native() {
    assert!(native_action(&action("ext.example.sync", &[])).is_none());
}

#[test]
fn field_edits_land_in_the_named_field() {
    remember(SkinFrame {
        targets: vec![(40_000_001, SkinTarget::Field("query".to_owned()))],
        layout: HomeLayout::default(),
    });
    assert!(is_field(40_000_001));
    assert!(edit_field(40_000_001, |text| text.push_str("dune")));
    assert_eq!(field_text(40_000_001).as_deref(), Some("dune"));
    assert!(!edit_field(40_000_002, |_| {}));
}

fn shown_on(node: &fluxa_skin::Resolved, form: FormFactor) -> bool {
    let name = match form {
        FormFactor::Desktop => "desktop",
        FormFactor::Mobile => "mobile",
        FormFactor::Tv => "tv",
    };
    node.class
        .as_deref()
        .unwrap_or_default()
        .split_whitespace()
        .filter_map(|class| class.strip_prefix("only-"))
        .all(|forms| forms.split('-').any(|only| only == name))
}

fn count_actions(
    nodes: &[fluxa_skin::Resolved],
    name: &str,
    form: FormFactor,
    found: &mut Vec<Value>,
) {
    for node in nodes.iter().filter(|node| shown_on(node, form)) {
        if let Some(action) = node.action.as_ref().filter(|action| action.name == name) {
            found.push(json!(action.args));
        }
        count_actions(&node.children, name, form, found);
    }
}

#[test]
fn default_menu_lists_every_item_on_each_form() {
    let skin = fluxa_skin::load_embedded(DEFAULT_FILES).unwrap();
    let data = json!({
        "const": skin.constants(),
        "menu": {
            "title": "Language",
            "x": 300.0,
            "y": 200.0,
            "items": [
                {"index": 0, "label": "English", "icon": null, "appIcon": null, "selected": true},
                {"index": 1, "label": "Turkish", "icon": null, "appIcon": null, "selected": false},
                {"index": 2, "label": "Arabic", "icon": null, "appIcon": null, "selected": false},
            ],
        },
    });
    for form in [FormFactor::Desktop, FormFactor::Mobile, FormFactor::Tv] {
        let tree = skin
            .resolve("menu", &data, form, &mut Default::default())
            .unwrap()
            .unwrap();
        let mut picks = Vec::new();
        count_actions(&tree, "menu_pick", form, &mut picks);
        assert_eq!(picks.len(), 3, "{form:?}");
        let mut dismiss = Vec::new();
        count_actions(&tree, "menu_dismiss", form, &mut dismiss);
        assert!(!dismiss.is_empty(), "{form:?}");
    }
}

#[test]
fn player_panel_rows_activate_by_index() {
    use fluxa_ui::{PanelRow, PanelStep, PlayerModel, PlayerPanel};
    let skin = fluxa_skin::load_embedded(DEFAULT_FILES).unwrap();
    let row = |label: &str| PanelRow {
        label: label.to_owned(),
        ..Default::default()
    };
    let step = |label: &str, step: PanelStep| PanelRow {
        step: Some(step),
        ..row(label)
    };
    let player = PlayerModel {
        panel: Some(PlayerPanel {
            title: "Audio".to_owned(),
            rows: vec![
                PanelRow {
                    heading: true,
                    ..row("Group")
                },
                row("Plain"),
                step("Size", PanelStep::Value),
                step("-", PanelStep::Minus),
                step("+", PanelStep::Plus),
                step("Color", PanelStep::Swatches(2)),
                PanelRow {
                    value: Some("#ff0000".to_owned()),
                    ..step("", PanelStep::Swatch)
                },
                PanelRow {
                    value: Some("#00ff00".to_owned()),
                    ..step("", PanelStep::Swatch)
                },
            ],
            list: true,
            ..Default::default()
        }),
        ..Default::default()
    };
    let data = json!({
        "const": skin.constants(),
        "player": fluxa_ui::player_data(&player),
    });
    for form in [FormFactor::Desktop, FormFactor::Mobile, FormFactor::Tv] {
        let tree = skin
            .resolve("player", &data, form, &mut Default::default())
            .unwrap()
            .unwrap();
        let mut rows = Vec::new();
        count_actions(&tree, "player_row", form, &mut rows);
        assert_eq!(rows.len(), 5, "{form:?}");
    }
}

#[test]
fn account_page_routes_service_buttons_to_legacy_nodes() {
    let skin = fluxa_skin::load_embedded(DEFAULT_FILES).unwrap();
    let settings = SettingsModel::default();
    let data = json!({
        "const": skin.constants(),
        "nav": fluxa_ui::nav_data("settings"),
        "settings": settings.values,
        "fields": {},
        "prefs": fluxa_ui::settings_data(&settings, fluxa_ui::UiFormFactor::Desktop),
    });
    let tree = skin
        .resolve(
            "settings",
            &data,
            FormFactor::Desktop,
            &mut Default::default(),
        )
        .unwrap()
        .unwrap();
    let mut nodes = Vec::new();
    count_actions(&tree, "node", FormFactor::Desktop, &mut nodes);
    let dump = format!("{nodes:?}");
    for id in [
        fluxa_ui::NODE_SETTINGS_SWITCH_PROFILE,
        fluxa_ui::NODE_SETTINGS_LOG_OUT,
        fluxa_ui::NODE_SETTINGS_ACCOUNT_BASE,
    ] {
        assert!(
            dump.contains(&format!("\"{id}\"")),
            "{id} missing in {dump}"
        );
    }
}

#[test]
fn addon_cards_expose_row_actions_and_install_fill() {
    let skin = fluxa_skin::load_embedded(DEFAULT_FILES).unwrap();
    let mut settings = SettingsModel::default();
    settings.active_section = fluxa_ui::addons_page();
    settings.addons = vec![json!({
        "transportUrl": "https://example.com/manifest.json",
        "manifest": {"name": "ExampleAddon", "version": "1.0.0", "resources": ["catalog"], "types": ["movie"]},
    })];
    let data = json!({
        "const": skin.constants(),
        "nav": fluxa_ui::nav_data("settings"),
        "settings": settings.values,
        "fields": {},
        "prefs": fluxa_ui::settings_data(&settings, fluxa_ui::UiFormFactor::Desktop),
    });
    let tree = skin
        .resolve(
            "settings",
            &data,
            FormFactor::Desktop,
            &mut Default::default(),
        )
        .unwrap()
        .unwrap();
    let mut nodes = Vec::new();
    count_actions(&tree, "node", FormFactor::Desktop, &mut nodes);
    let dump = format!("{nodes:?}");
    let remove = fluxa_ui::addon_action_node(0, 3);
    assert!(dump.contains(&format!("\"{remove}\"")), "{dump}");
    assert!(
        dump.contains(&format!("{}=addonUrl", fluxa_ui::NODE_SETTINGS_ADDON_URL)),
        "{dump}"
    );
}

#[test]
fn plugin_and_server_pages_resolve() {
    let skin = fluxa_skin::load_embedded(DEFAULT_FILES).unwrap();
    for title in ["Plugins", "Servers"] {
        let mut settings = SettingsModel::default();
        settings.active_section = fluxa_ui::SETTINGS_SECTIONS
            .iter()
            .position(|section| section.title == title)
            .unwrap();
        settings.plugins = json!({
            "repositories": [{"name": "ExampleRepo", "manifestUrl": "https://example.com/m.json", "scraperCount": 1}],
            "scrapers": [{"id": "a", "name": "ExampleSource", "enabled": true}],
        });
        let data = json!({
            "const": skin.constants(),
            "nav": fluxa_ui::nav_data("settings"),
            "settings": settings.values,
            "fields": {},
            "prefs": fluxa_ui::settings_data(&settings, fluxa_ui::UiFormFactor::Desktop),
        });
        let tree = skin
            .resolve(
                "settings",
                &data,
                FormFactor::Desktop,
                &mut Default::default(),
            )
            .unwrap()
            .unwrap();
        let mut nodes = Vec::new();
        count_actions(&tree, "node", FormFactor::Desktop, &mut nodes);
        let dump = format!("{nodes:?}");
        let expected = if title == "Plugins" {
            fluxa_ui::NODE_SETTINGS_PLUGIN_INSTALL
        } else {
            fluxa_ui::NODE_SETTINGS_SERVER_CONNECT
        };
        assert!(dump.contains(&format!("\"{expected}\"")), "{title}: {dump}");
    }
}

#[test]
fn empty_home_offers_addons() {
    let skin = fluxa_skin::load_embedded(DEFAULT_FILES).unwrap();
    let data = json!({
        "const": skin.constants(),
        "nav": fluxa_ui::nav_data("home"),
        "home": {"heroSlides": [], "rows": [], "cards": [], "isLoading": false, "addonsPage": 5},
    });
    let tree = skin
        .resolve("home", &data, FormFactor::Desktop, &mut Default::default())
        .unwrap()
        .unwrap();
    let mut pages = Vec::new();
    count_actions(&tree, "open_settings", FormFactor::Desktop, &mut pages);
    assert!(pages.iter().any(|args| args["page"] == 5), "{pages:?}");
}

#[test]
fn skip_next_and_warning_layers_resolve() {
    use fluxa_ui::{NextEpisodeCard, PlayerModel, SkipCard, SkipKind};
    let skin = fluxa_skin::load_embedded(DEFAULT_FILES).unwrap();
    let player = PlayerModel {
        skip: Some(SkipCard {
            kind: SkipKind::Intro,
            seek_to: 90.0,
            progress: 0.4,
        }),
        next_episode: Some(NextEpisodeCard {
            label: "S1E2".to_owned(),
            title: "Second".to_owned(),
            thumbnail: Some("https://example.com/t.jpg".to_owned()),
            countdown: Some(5),
        }),
        warnings: vec![("Violence".to_owned(), "Moderate".to_owned())],
        warnings_elapsed: Some(1.0),
        ..Default::default()
    };
    let data = json!({
        "const": skin.constants(),
        "player": fluxa_ui::player_data(&player),
    });
    let tree = skin
        .resolve(
            "player",
            &data,
            FormFactor::Desktop,
            &mut Default::default(),
        )
        .unwrap()
        .unwrap();
    let mut ops = Vec::new();
    count_actions(&tree, "player", FormFactor::Desktop, &mut ops);
    let dump = format!("{ops:?}");
    for op in ["skip", "next_play", "next_dismiss"] {
        assert!(dump.contains(op), "{op} missing in {dump}");
    }
}

#[test]
fn github_links_become_archive_urls() {
    assert_eq!(
        github_archive("https://github.com/example/skin"),
        "https://github.com/example/skin/archive/HEAD.zip"
    );
    assert_eq!(
        github_archive("https://github.com/example/skin/tree/dev/"),
        "https://github.com/example/skin/archive/dev.zip"
    );
    assert_eq!(
        github_archive("https://example.com/skin.zip"),
        "https://example.com/skin.zip"
    );
}

#[test]
fn toast_and_tip_screens_resolve() {
    let skin = fluxa_skin::load_embedded(DEFAULT_FILES).unwrap();
    let data = json!({
        "toast": {"text": "Saved"},
        "tip": {"text": "Hint", "x": 100.0, "y": 80.0},
    });
    let data = json!({
        "toast": {"text": "Saved"},
        "tip": {"text": "Hint", "x": 100.0, "y": 80.0},
        "update": {"version": "1.2.3"},
        "window": {"icon": "CornersOut"},
        "mini": {"playIcon": "PauseFilled", "ratio": 0.4},
        "const": skin.constants(),
    });
    for screen in ["toast", "tip", "update", "window", "mini"] {
        let tree = skin
            .resolve(screen, &data, FormFactor::Desktop, &mut Default::default())
            .unwrap()
            .unwrap();
        assert!(!tree.is_empty(), "{screen}");
    }
}

#[test]
fn report_and_addon_dialogs_route_to_legacy_nodes() {
    let skin = fluxa_skin::load_embedded(DEFAULT_FILES).unwrap();
    let mut settings = SettingsModel::default();
    settings.report_open = true;
    settings.report_title = "Crash".to_owned();
    settings.report_body = "Details".to_owned();
    let mut ids = Vec::new();
    for dialog in [fluxa_ui::dialog_data(&settings)] {
        let data = json!({"dialog": dialog, "const": skin.constants()});
        let tree = skin
            .resolve(
                "dialog",
                &data,
                FormFactor::Desktop,
                &mut Default::default(),
            )
            .unwrap()
            .unwrap();
        count_actions(&tree, "node", FormFactor::Desktop, &mut ids);
    }
    assert!(ids.len() >= 2, "{ids:?}");
}

#[test]
fn bundled_theme_parses() {
    let skin = fluxa_skin::load_embedded(DEFAULT_FILES).unwrap();
    let css = skin.theme().unwrap();
    fluxa_ui::reload_styles_with(css).unwrap();
}

#[test]
fn every_profile_mode_resolves() {
    use fluxa_ui::{ProfileEntry, ProfilesMode, ProfilesModel};
    let skin = fluxa_skin::load_embedded(DEFAULT_FILES).unwrap();
    let entry = |id: &str| ProfileEntry {
        id: id.to_owned(),
        name: id.to_owned(),
        locked: id == "b",
        ..Default::default()
    };
    let mut model = ProfilesModel {
        profiles: vec![entry("a"), entry("b")],
        ..Default::default()
    };
    let mut seen = Vec::new();
    for step in 0..5 {
        match step {
            1 => model.open_form(None),
            2 => model.mode = ProfilesMode::Avatars,
            3 => {
                model.mode = ProfilesMode::Select;
                model.confirm_delete = Some("a".to_owned());
            }
            4 => {
                model.confirm_delete = None;
                model.request_delete(1);
            }
            _ => {}
        }
        let data = json!({
            "const": skin.constants(),
            "settings": {},
            "fields": {},
            "profiles": fluxa_ui::profiles_data(&model, fluxa_ui::UiFormFactor::Desktop),
        });
        let tree = skin
            .resolve(
                "profiles",
                &data,
                FormFactor::Desktop,
                &mut Default::default(),
            )
            .unwrap()
            .unwrap();
        assert!(!tree.is_empty(), "step {step}");
        seen.push(format!("{tree:?}").len());
    }
    assert!(seen.windows(2).any(|pair| pair[0] != pair[1]));
}

#[test]
fn watch_room_and_entry_resolve() {
    let skin = fluxa_skin::load_embedded(DEFAULT_FILES).unwrap();
    let model = fluxa_ui::WatchTogetherModel {
        room_code: Some("ABC123".to_owned()),
        host: true,
        members: vec![
            fluxa_ui::WatchMemberView {
                id: "a".to_owned(),
                name: "Anna".to_owned(),
                you: true,
                ..Default::default()
            },
            fluxa_ui::WatchMemberView {
                id: "b".to_owned(),
                name: "Marcus".to_owned(),
                ..Default::default()
            },
        ],
        chat: vec![fluxa_ui::WatchChatView {
            name: "Marcus".to_owned(),
            text: "Hi".to_owned(),
            ..Default::default()
        }],
        ..Default::default()
    };
    for model in [model, Default::default()] {
        let data = json!({
            "const": skin.constants(),
            "settings": {},
            "fields": {},
            "watch": fluxa_ui::watch_data(&model),
        });
        for form in [FormFactor::Desktop, FormFactor::Mobile, FormFactor::Tv] {
            let tree = skin
                .resolve("watch", &data, form, &mut Default::default())
                .unwrap()
                .unwrap();
            assert!(!tree.is_empty(), "{form:?}");
        }
    }
}

#[test]
fn detail_modal_episodes_and_similar_carry_their_index() {
    let skin = fluxa_skin::load_embedded(DEFAULT_FILES).unwrap();
    let data = json!({
        "const": skin.constants(),
        "settings": {},
        "modal": {
            "title": "T", "backdrop": null, "meta": "", "description": "", "playLabel": "Play",
            "watchlistIcon": "Plus", "multiSeason": false, "seasons": [],
            "titles": {"episodes": "Episodes", "similar": "More"},
            "show": {"episodes": true, "similar": true},
            "episodes": [
                {"index": "0", "label": "S1E1", "title": "A", "overview": "", "thumb": null},
                {"index": "1", "label": "S1E2", "title": "B", "overview": "", "thumb": null},
            ],
            "similar": [{"title": "X", "artworkUrl": null}],
        },
    });
    let data =
        json!({"const": data["const"].clone(), "settings": {}, "detail": data["modal"].clone()});
    let tree = skin
        .resolve(
            "detail_modal",
            &data,
            FormFactor::Desktop,
            &mut Default::default(),
        )
        .unwrap()
        .unwrap();
    let mut found = Vec::new();
    count_actions(&tree, "modal", FormFactor::Desktop, &mut found);
    assert!(found.len() >= 6, "{found:?}");
}

#[test]
fn bundled_default_skin_loads() {
    let skin = fluxa_skin::load_embedded(DEFAULT_FILES).unwrap();
    assert_eq!(skin.manifest.id, "default");
}
