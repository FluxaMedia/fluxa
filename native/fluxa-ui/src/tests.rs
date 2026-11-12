use super::*;
use serde_json::json;

#[test]
fn artwork_decode_size_tracks_card_pixels_and_display_density() {
    let logical_card = Vec2::new(200.0, 100.0);

    assert_eq!(artwork_target_size(logical_card, 1.0), [200, 100]);
    assert_eq!(artwork_target_size(logical_card, 2.5), [500, 250]);
}

#[test]
fn hero_height_is_reduced_by_about_seven_percent_for_shared_layouts() {
    let viewport = Viewport::new(1920, 1080, UiFormFactor::Desktop);
    let original = (viewport.height as f32 * 0.66 + 120.0).clamp(728.0, 984.0);
    assert!((home_hero_height(viewport) - original * 0.93).abs() < 0.01);
}

struct EmptyHomeAssets;

impl HomeAssets for EmptyHomeAssets {
    fn background(&self) -> TextureId {
        TextureId::Managed(0)
    }

    fn texture(&mut self, _url: Option<&str>) -> Option<TextureId> {
        None
    }
}

fn draw_test_frame(
    viewport: Viewport,
    draw: impl FnMut(&crate::gfx::Context, &mut EmptyHomeAssets) -> HomeLayout,
) -> HomeLayout {
    draw_test_frames(viewport, 1, draw)
}

fn draw_test_frames(
    viewport: Viewport,
    frames: usize,
    mut draw: impl FnMut(&crate::gfx::Context, &mut EmptyHomeAssets) -> HomeLayout,
) -> HomeLayout {
    let context = crate::gfx::Context::default();
    context.set_fonts(crate::fonts::definitions());
    let mut assets = EmptyHomeAssets;
    let mut layout = HomeLayout::default();
    for _ in 0..frames {
        context
            .run_ui(
                crate::gfx::RawInput {
                    screen_rect: Some(Rect::from_min_size(
                        Pos2::ZERO,
                        Vec2::new(viewport.width, viewport.height),
                    )),
                    ..Default::default()
                },
                |ui| layout = draw(ui.ctx(), &mut assets),
            )
            .drop_without_applying_deltas();
    }
    layout
}

#[test]
fn core_snapshot_projection_keeps_home_rows_and_progress() {
    let model = home_model_from_core_snapshot(
        &json!({
            "home": {
                "billboard": {
                    "id": "tt42",
                    "type": "movie",
                    "name": "Featured",
                    "year": 2026,
                    "genres": ["Drama"],
                },
                "continueWatching": [{
                    "id": "tt1",
                    "type": "series",
                    "name": "Bleach",
                    "resumeProgressPercent": 25,
                    "continueWatchingPoster": "https://example.test/bleach.jpg",
                }],
                "categories": [
                    {
                        "id": "continue_watching",
                        "type": "continue_watching",
                        "name": "Continue Watching",
                        "items": [{"id": "tt1", "name": "Bleach"}],
                    },
                    {
                        "name": "Popular",
                        "items": [{"id": "tt2", "name": "Deadpool", "year": 2024}],
                    }
                ],
            }
        }),
        UiFormFactorJson::Tv,
    );

    assert_eq!(model.title, "Featured");
    assert_eq!(model.eyebrow, "2026 · Drama");
    assert_eq!(model.item_id.as_deref(), Some("tt42"));
    assert_eq!(model.form_factor, UiFormFactorJson::Tv);
    assert_eq!(model.cards[0].progress, 0.25);
    assert_eq!(
        model.rows.len(),
        1,
        "continue watching must not be duplicated as a catalog row"
    );
    assert_eq!(
        model.cards[0].artwork_url.as_deref(),
        Some("https://example.test/bleach.jpg")
    );
    assert_eq!(model.rows[0].title, "Popular");
    assert_eq!(model.rows[0].cards[0].title, "Deadpool");
    assert_eq!(model.rows[0].cards[0].subtitle, "2024");
}

#[test]
fn home_feed_titles_match_compose_display_titles() {
    assert_eq!(
        normalize_home_row_title("AIOMetadata | ElfHosted - TMDB Popular"),
        "TMDB Popular"
    );
    assert_eq!(normalize_home_row_title("TMDB Trending"), "TMDB Trending");
}

#[test]
fn poster_without_release_metadata_never_gets_continue_watching_copy() {
    let card = core_home_card(&serde_json::json!({ "name": "Reacher" }));

    assert!(card.subtitle.is_empty());
}

#[test]
fn poster_rows_prefer_poster_artwork_over_background_artwork() {
    let card = core_home_card_for_kind(
        &serde_json::json!({
            "name": "The Secret Woman",
            "background": "https://example.test/backdrop.jpg",
            "poster": "https://example.test/poster.jpg"
        }),
        HomeRowKind::Poster,
    );

    assert_eq!(
        card.artwork_url.as_deref(),
        Some("https://example.test/poster.jpg")
    );
}

#[test]
fn collection_shelves_keep_web_tile_shapes_and_hide_title_metadata() {
    let model = home_model_from_core_snapshot(
        &json!({
            "settings": {"values": {"gifAutoplayEnabled": false}},
            "home": {
                "categories": [{
                    "id": "streaming",
                    "type": "collection",
                    "name": "Streaming",
                    "items": [
                        {"id": "netflix", "type": "catalog_folder", "name": "Netflix", "reason": "square", "poster": "https://example.test/netflix.png", "hideTitle": true, "focusGifUrl": "https://example.test/netflix.webp", "focusGifEnabled": true},
                        {"id": "catalog", "type": "catalog_folder", "name": "Catalog", "reason": "wide", "poster": "https://example.test/catalog.png"}
                    ]
                }, {
                    "id": "user-folders",
                    "type": "collection_folder",
                    "name": "My folders",
                    "items": [{"id": "folder-1", "type": "catalog_folder", "name": "Anime", "reason": "wide"}]
                }]
            }
        }),
        UiFormFactorJson::Desktop,
    );

    let collection = &model.rows[0];
    assert_eq!(collection.kind, HomeRowKind::Collection);
    assert_eq!(
        home_collection_card_dimensions(
            UiMetrics::for_viewport(Viewport::new(1280, 720, UiFormFactor::Desktop)),
            &collection.cards[0]
        ),
        (150.0, 150.0)
    );
    assert_eq!(
        home_collection_card_dimensions(
            UiMetrics::for_viewport(Viewport::new(1280, 720, UiFormFactor::Desktop)),
            &collection.cards[1]
        ),
        (280.0, 158.0)
    );
    assert!(collection.cards[0].hide_title);
    assert_eq!(
        collection.cards[0].motion_url.as_deref(),
        Some("https://example.test/netflix.webp")
    );
    assert!(collection.cards[0].motion_enabled);
    assert!(!model.gif_autoplay_enabled);
    assert_eq!(collection.cards[0].row_kind, HomeRowKind::Collection);
    assert_eq!(
        model.rows.len(),
        2,
        "home collection folders must not be dropped"
    );
    assert_eq!(model.rows[1].kind, HomeRowKind::Collection);
    assert_eq!(model.rows[1].title, "My folders");
}

#[test]
fn continue_rows_prefer_continue_artwork_over_poster_artwork() {
    let card = core_home_card_for_kind(
        &serde_json::json!({
            "name": "Bleach",
            "poster": "https://example.test/poster.jpg",
            "continueWatchingBackground": "https://example.test/episode.jpg"
        }),
        HomeRowKind::Continue,
    );

    assert_eq!(
        card.artwork_url.as_deref(),
        Some("https://example.test/episode.jpg")
    );
}

#[test]
fn continue_rows_prefer_core_formatted_episode_label() {
    let card = core_home_card_for_kind(
        &serde_json::json!({
            "name": "Bleach",
            "lastEpisodeName": "The Day I Become a Shinigami",
            "episodeLabel": "S1:E1 The Day I Become a Shinigami",
            "lastVideoId": "show-id:1:1"
        }),
        HomeRowKind::Continue,
    );

    assert_eq!(card.subtitle, "S1:E1 The Day I Become a Shinigami");
}

#[test]
fn recommendation_actions_sit_above_the_bottom_inset() {
    let mut player = PlayerModel::default();
    player.recommendations = (0..3)
        .map(|_| HomeHero {
            title: "Example".to_owned(),
            eyebrow: "2024".to_owned(),
            description: "An example description for the recommendation.".to_owned(),
            ..Default::default()
        })
        .collect();
    let viewport = Viewport::new(1280, 720, UiFormFactor::Desktop);
    let layout = draw_test_frames(viewport, 3, |context, assets| {
        draw_player(context, viewport, &player, assets, None)
    });
    let play = layout
        .focusable
        .iter()
        .find(|(id, _)| *id == NODE_PLAYER_RECOMMENDATION_PLAY)
        .map(|(_, rect)| *rect)
        .expect("play is focusable");
    let dot = layout
        .focusable
        .iter()
        .find(|(id, _)| *id == NODE_PLAYER_RECOMMENDATION_BASE)
        .map(|(_, rect)| *rect)
        .expect("first dot is focusable");
    assert!((play.bottom() - (720.0 - 56.0)).abs() < 2.0);
    assert!((play.top() - dot.bottom() - 17.0).abs() < 2.0);
}

#[test]
fn source_rows_move_with_the_shell_scroll() {
    let mut player = PlayerModel::default();
    player.sources = Some(std::sync::Arc::new(
        (0..30)
            .map(|index| PlayerSource {
                addon: "ExampleAddon".to_owned(),
                name: format!("Example source {index}"),
                detail: "1080p".to_owned(),
                ..Default::default()
            })
            .collect(),
    ));
    let first_row_top = |scroll_y: f32| {
        let viewport = Viewport::new(390, 844, UiFormFactor::Mobile).with_scroll_y(scroll_y);
        let layout = draw_test_frames(viewport, 3, |context, assets| {
            draw_player(context, viewport, &player, assets, None)
        });
        layout
            .focusable
            .iter()
            .find(|(id, _)| *id == NODE_PLAYER_SOURCE_BASE)
            .map(|(_, rect)| rect.top())
            .expect("first source is focusable")
    };
    let rest = first_row_top(0.0);
    let scrolled = first_row_top(80.0);
    assert!((rest - scrolled - 80.0).abs() < 1.0);
}

#[test]
fn drawn_shelves_move_with_the_shell_scroll() {
    let mut home = HomeModel::default();
    home.show_hero_section = false;
    home.rows = (0..8)
        .map(|index| HomeRow {
            id: Some(format!("row-{index}")),
            title: format!("Row {index}"),
            type_label: None,
            cards: vec![HomeCard {
                title: "Example".to_owned(),
                ..Default::default()
            }],
            kind: HomeRowKind::Poster,
            can_load_more: false,
            catalog_page: None,
        })
        .collect();
    let first_card_top = |scroll_y: f32| {
        let viewport = Viewport::new(390, 844, UiFormFactor::Mobile).with_scroll_y(scroll_y);
        let context = crate::gfx::Context::default();
        context.set_fonts(crate::fonts::definitions());
        let mut assets = EmptyHomeAssets;
        let mut layout = HomeLayout::default();
        for _ in 0..3 {
            context
                .run_ui(
                    crate::gfx::RawInput {
                        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(390.0, 844.0))),
                        ..Default::default()
                    },
                    |ui| layout = draw_home_content(ui.ctx(), viewport, &home, &mut assets, None),
                )
                .drop_without_applying_deltas();
        }
        layout
            .focusable
            .iter()
            .find(|(id, _)| *id == NODE_CARD_BASE)
            .map(|(_, rect)| rect.top())
            .expect("first card is focusable")
    };
    let rest = first_card_top(0.0);
    let scrolled = first_card_top(60.0);
    assert!((rest - scrolled - 60.0).abs() < 1.0);
}

#[test]
fn home_scroll_moves_content_but_keeps_navigation_fixed() {
    let mut home = HomeModel::default_empty();
    home.show_hero_section = false;
    home.rows = vec![HomeRow {
        id: Some("popular".to_owned()),
        title: "Popular".to_owned(),
        type_label: None,
        cards: vec![HomeCard {
            title: "Example".to_owned(),
            ..Default::default()
        }],
        kind: HomeRowKind::Poster,
        can_load_more: false,
        catalog_page: None,
    }];
    let viewport = Viewport::new(390, 844, UiFormFactor::Mobile);
    let at_top = home_layout(viewport, &home);
    home.scroll_offset = 40.0;
    let scrolled = home_layout(viewport, &home);
    let card_y = |layout: &HomeLayout| {
        layout
            .focusable
            .iter()
            .find(|(id, _)| *id == NODE_CARD_BASE)
            .map(|(_, rect)| rect.top())
            .expect("first home card should be focusable")
    };
    let navigation_y = |layout: &HomeLayout| {
        layout
            .focusable
            .iter()
            .find(|(id, _)| *id == NODE_HOME)
            .map(|(_, rect)| rect.top())
            .expect("home navigation should be focusable")
    };
    assert!((card_y(&at_top) - card_y(&scrolled) - 40.0).abs() < 0.01);
    assert!((navigation_y(&at_top) - navigation_y(&scrolled)).abs() < 0.01);
}

#[test]
fn home_shelf_heading_and_spacing_prevent_adjacent_row_overlap() {
    let mut home = HomeModel::default_empty();
    home.show_hero_section = false;
    home.cards.push(HomeCard {
        title: "Continue item".to_owned(),
        ..Default::default()
    });
    home.rows.push(HomeRow {
        id: Some("popular".to_owned()),
        title: "TMDB Popular".to_owned(),
        type_label: None,
        cards: vec![HomeCard {
            title: "Popular item".to_owned(),
            ..Default::default()
        }],
        kind: HomeRowKind::Poster,
        can_load_more: false,
        catalog_page: None,
    });

    let viewport = Viewport::new(390, 844, UiFormFactor::Mobile);
    let metrics = UiMetrics::for_viewport(viewport);
    let context = crate::gfx::Context::default();
    context.set_fonts(crate::fonts::definitions());
    let mut assets = EmptyHomeAssets;
    let mut rendered_layout = None;
    context
        .run_ui(
            crate::gfx::RawInput {
                screen_rect: Some(Rect::from_min_size(
                    Pos2::ZERO,
                    Vec2::new(viewport.width, viewport.height),
                )),
                ..Default::default()
            },
            |ui| {
                rendered_layout = Some(draw_home(ui.ctx(), viewport, &home, &mut assets, None));
            },
        )
        .drop_without_applying_deltas();
    let layout = rendered_layout.expect("Home should render in the the backend frame");
    let first = layout
        .focusable
        .iter()
        .find(|(id, _)| *id == NODE_CARD_BASE)
        .map(|(_, rect)| *rect)
        .expect("Continue Watching card should have geometry");
    let second = layout
        .focusable
        .iter()
        .find(|(id, _)| *id == NODE_CARD_BASE + 1)
        .map(|(_, rect)| *rect)
        .expect("TMDB Popular card should have geometry");

    assert!(
        second.top() >= first.bottom() + metrics.section_gap + metrics.vertical_spacing,
        "next shelf starts before the previous card and its shelf gap finish"
    );
}

#[test]
fn home_model_keeps_all_loaded_catalog_cards_and_pagination_metadata() {
    let items = (0..24)
        .map(|index| {
            serde_json::json!({
                "id": format!("item-{index}"),
                "type": "movie",
                "name": format!("Movie {index}"),
                "poster": format!("https://images.example/{index}.jpg"),
            })
        })
        .collect::<Vec<_>>();
    let snapshot = serde_json::json!({
        "home": {
            "categories": [{
                "id": "catalog.popular",
                "name": "Popular",
                "type": "catalog",
                "canLoadMore": true,
                "items": items,
            }],
        },
    });

    let home = home_model_from_core_snapshot(&snapshot, UiFormFactorJson::Mobile);
    assert_eq!(home.rows.len(), 1);
    assert_eq!(home.rows[0].id.as_deref(), Some("catalog.popular"));
    assert_eq!(home.rows[0].cards.len(), 24);
    assert!(home.rows[0].can_load_more);
}

#[test]
fn mobile_home_navigation_respects_real_system_bottom_inset() {
    let home = HomeModel::default();
    let without_inset = home_layout(Viewport::new(390, 844, UiFormFactor::Mobile), &home);
    let with_three_button_inset = home_layout(
        Viewport::new(390, 844, UiFormFactor::Mobile).with_safe_bottom(48.0),
        &home,
    );
    let nav_top = |layout: &HomeLayout| {
        layout
            .focusable
            .iter()
            .find(|(id, _)| *id == NODE_HOME)
            .map(|(_, rect)| rect.top())
            .expect("home navigation should be focusable")
    };

    assert!((nav_top(&without_inset) - nav_top(&with_three_button_inset) - 48.0).abs() < 0.01);
}

#[test]
fn mobile_bottom_navigation_splits_the_floating_bar_into_four_slots() {
    let viewport = Viewport::new(390, 844, UiFormFactor::Mobile);
    let metrics = UiMetrics::for_viewport(viewport);
    let navigation = navigation_focus_rects(viewport, metrics);
    let slots = [NODE_HOME, NODE_LIBRARY, NODE_DISCOVER, NODE_PROFILE].map(|node| {
        navigation
            .iter()
            .find(|(id, _)| *id == node)
            .map(|(_, rect)| *rect)
            .expect("all bottom destinations should have a hit target")
    });
    let width = slots[0].width();

    let inner = mobile_nav_rect(viewport).shrink(nav_bar_padding());
    assert!((slots[0].left() - inner.left()).abs() < 0.01);
    assert!((slots[3].right() - inner.right()).abs() < 0.01);
    assert!(slots.iter().all(|rect| (rect.width() - width).abs() < 0.01));
}

#[test]
fn mobile_library_filters_share_one_row_inside_the_viewport() {
    let viewport = Viewport::new(320, 720, UiFormFactor::Mobile);
    let mut library = LibraryModel::default();
    library.statuses = vec!["watchlist".to_owned(), "watching".to_owned()];
    let layout = draw_test_frame(viewport, |context, assets| {
        draw_library(context, viewport, &library, LibraryTab::Watchlist, assets)
    });
    let controls = [NODE_LIBRARY_STATUS, NODE_LIBRARY_TYPE, NODE_LIBRARY_SORT].map(|id| {
        layout
            .focusable
            .iter()
            .find(|(node, _)| *node == id)
            .unwrap()
            .1
    });
    assert!(
        controls
            .iter()
            .all(|rect| (rect.top() - controls[0].top()).abs() < 0.5)
    );
    assert!(
        controls
            .windows(2)
            .all(|pair| pair[0].right() <= pair[1].left())
    );
    assert!(controls[2].right() <= viewport.width);
}

#[test]
fn mobile_continue_cards_leave_the_next_card_peeking() {
    let metrics = UiMetrics::for_viewport(Viewport::new(390, 844, UiFormFactor::Mobile));
    let (width, height, _) = home_row_dimensions(metrics, HomeRowKind::Continue);

    let content = 390.0 - metrics.page_padding * 2.0;
    assert!(width > content * 0.5 && width < content);
    assert!((height - width * metrics.horizontal_card_height_ratio).abs() < 0.01);
}

#[test]
fn narrow_desktop_window_centers_hero_like_a_phone() {
    let mut home = HomeModel::default();
    home.item_id = Some("tt42".to_owned());
    let play_rect = |viewport: Viewport| {
        home_layout(viewport, &home)
            .focusable
            .into_iter()
            .find(|(id, _)| *id == NODE_PLAY)
            .map(|(_, rect)| rect)
            .unwrap()
    };

    let narrow = Viewport::new(430, 932, UiFormFactor::Desktop);
    assert!((play_rect(narrow).center().x - narrow.width * 0.5).abs() < 0.01);

    let wide = Viewport::new(1280, 800, UiFormFactor::Mobile);
    assert!(play_rect(wide).center().x < wide.width * 0.5);
}

#[test]
fn mobile_hero_centers_play() {
    let mut home = HomeModel::default();
    home.item_id = Some("tt42".to_owned());
    let viewport = Viewport::new(390, 844, UiFormFactor::Mobile);
    let play = home_layout(viewport, &home)
        .focusable
        .into_iter()
        .find(|(id, _)| *id == NODE_PLAY)
        .map(|(_, rect)| rect)
        .expect("mobile hero should expose its Play action");
    assert!((play.center().x - viewport.width * 0.5).abs() < 0.01);
}

#[test]
fn home_carousels_share_horizontal_geometry_and_scroll_limits() {
    let viewport = Viewport::new(390, 844, UiFormFactor::Mobile);
    let mut home = HomeModel::default();
    let first_layout = home_layout(viewport, &home);
    let card_rect = |layout: &HomeLayout| {
        layout
            .focusable
            .iter()
            .find(|(id, _)| *id == NODE_CARD_BASE)
            .map(|(_, rect)| *rect)
            .expect("first Home card should have a hit target")
    };
    let first = card_rect(&first_layout);

    let mut regions = Vec::new();
    draw_test_frame(viewport, |context, assets| {
        let layout = draw_home(context, viewport, &home, assets, None);
        regions = take_scroll_regions(context);
        layout
    });
    let row = regions
        .iter()
        .find(|region| region.id == ScrollId::Row(0))
        .expect("first Home row should report its scroll region");
    assert!(row.rect.contains(first.center()));
    assert!(row.max > 0.0);

    home.row_scroll_offsets = vec![24.0];
    let shifted = card_rect(&home_layout(viewport, &home));
    assert!((first.left() - shifted.left() - 24.0).abs() < 0.01);
}

#[test]
fn library_cards_scroll_without_moving_tabs_or_navigation() {
    let viewport = Viewport::new(390, 844, UiFormFactor::Mobile);
    let mut library = LibraryModel::default();
    library.statuses = vec!["watchlist".to_owned()];
    library.watchlist = (0..18)
        .map(|index| HomeCard {
            title: format!("Title {index}"),
            ..Default::default()
        })
        .collect();
    assert!(library_scroll_max(viewport, &library, LibraryTab::Watchlist) > 0.0);

    let first = draw_test_frames(viewport, 3, |context, assets| {
        draw_library(context, viewport, &library, LibraryTab::Watchlist, assets)
    });
    let scrolled_viewport = viewport.with_scroll_y(52.0);
    let scrolled = draw_test_frames(scrolled_viewport, 3, |context, assets| {
        draw_library(
            context,
            scrolled_viewport,
            &library,
            LibraryTab::Watchlist,
            assets,
        )
    });
    let rect = |layout: &HomeLayout, id| {
        layout
            .focusable
            .iter()
            .find(|(node, _)| *node == id)
            .map(|(_, rect)| *rect)
            .expect("expected screen control geometry")
    };

    assert!(
        (rect(&first, NODE_CARD_BASE).bottom() - rect(&scrolled, NODE_CARD_BASE).bottom() - 52.0)
            .abs()
            < 0.01
    );
    assert_eq!(
        rect(&first, NODE_LIBRARY_STATUS),
        rect(&scrolled, NODE_LIBRARY_STATUS)
    );
    assert_eq!(rect(&first, NODE_LIBRARY), rect(&scrolled, NODE_LIBRARY));
}

#[test]
fn compact_calendar_grid_stays_inside_the_viewport() {
    let viewport = Viewport::new(320, 720, UiFormFactor::Mobile);
    let calendar = CalendarModel {
        year: 2026,
        month: 9,
        ..Default::default()
    };
    let layout = draw_test_frame(viewport, |context, assets| {
        draw_calendar(context, viewport, &calendar, assets)
    });

    for (node, rect) in layout.focusable {
        if (NODE_CALENDAR_DAY_BASE..NODE_CALENDAR_DAY_BASE + 32).contains(&node) {
            assert!(
                rect.left() >= 0.0,
                "day cell starts outside the screen: {rect:?}"
            );
            assert!(
                rect.right() <= viewport.width + 0.1,
                "day cell overflows: {rect:?}"
            );
        }
    }
}

#[test]
fn library_filter_options_come_from_the_core_plan() {
    let mut model = LibraryModel::default();
    model.apply_core_plan(
        &serde_json::json!({
            "statuses": ["watchlist", "watching"],
            "types": ["anime"],
            "sorts": ["tracker", "title"]
        }),
        LibraryTab::Watchlist,
    );
    assert_eq!(model.statuses, ["watchlist", "watching"]);
    assert_eq!(model.types, ["anime"]);
    assert_eq!(model.sorts, ["tracker", "title"]);
}

#[test]
fn settings_choices_cycle_without_coercing_strings_to_booleans() {
    let model = SettingsModel {
        values: serde_json::json!({"language":"en", "posterHideTitles":false}),
        last_write_error: None,
        active_section: 0,
        ..Default::default()
    };
    let language = SETTINGS_SECTIONS
        .iter()
        .flat_map(|section| section.rows)
        .find(|row| row.key == "language")
        .unwrap();
    assert_eq!(model.next_value(language), "tr");
    let compact = SETTINGS_SECTIONS
        .iter()
        .flat_map(|section| section.rows)
        .find(|row| row.key == "posterHideTitles")
        .unwrap();
    assert_eq!(model.next_value(compact), true);
    assert_eq!(localized("nav.library", "tr"), "Kütüphane");
    let defaults = SettingsModel::default();
    assert_eq!(defaults.next_value(language), "tr");
    let download = SETTINGS_SECTIONS
        .iter()
        .flat_map(|section| section.rows)
        .find(|row| row.key == "downloadSourceSelectionMode")
        .unwrap();
    assert_eq!(defaults.next_value(download), "regex");
    let player = SETTINGS_SECTIONS
        .iter()
        .flat_map(|section| section.rows)
        .find(|row| row.key == "preferredPlayer")
        .unwrap();
    assert_eq!(
        defaults.next_value_for(player, UiFormFactor::Desktop),
        "external"
    );
    assert_eq!(
        defaults.next_value_for(player, UiFormFactor::Tv),
        "external"
    );
}

#[test]
fn secondary_screen_controls_stay_inside_mobile_and_tv_widths() {
    for viewport in [
        Viewport::new(320, 720, UiFormFactor::Mobile),
        Viewport::new(1920, 1080, UiFormFactor::Tv),
    ] {
        let library = LibraryModel::default();
        let library_layout = draw_test_frame(viewport, |context, assets| {
            draw_library(context, viewport, &library, LibraryTab::Watchlist, assets)
        });
        let mut settings = SettingsModel::default();
        settings.active_section = SETTINGS_SECTIONS
            .iter()
            .position(|section| section.title == "Details")
            .unwrap();
        settings.section_open = true;
        let settings_layout = draw_test_frame(viewport, |context, assets| {
            draw_settings(context, viewport, &settings, assets)
        });
        for (id, rect) in library_layout
            .focusable
            .iter()
            .chain(settings_layout.focusable.iter())
        {
            assert!(
                rect.left() >= -1.0 && rect.right() <= viewport.width + 1.0,
                "node {id} exceeds {:?} viewport: {rect:?}",
                viewport.form_factor
            );
        }
        assert!(
            library_layout
                .focusable
                .iter()
                .any(|(id, _)| *id == NODE_LIBRARY_SEARCH)
        );
        assert_eq!(
            settings_layout
                .focusable
                .iter()
                .filter(|(id, _)| *id >= NODE_SETTINGS_ROW_BASE
                    && *id
                        < NODE_SETTINGS_ROW_BASE
                            + SETTINGS_SECTIONS
                                .iter()
                                .map(|section| section.rows.len())
                                .sum::<usize>() as u64)
                .count(),
            SETTINGS_SECTIONS[settings.active_section]
                .rows
                .iter()
                .filter(|row| super::settings::row_applies(row.key, viewport))
                .filter(|row| {
                    viewport.form_factor == UiFormFactor::Tv
                        || super::settings::slider_range(row.key).is_none()
                })
                .count()
        );
    }
}

#[test]
fn selected_calendar_day_exposes_releases_to_touch_and_dpad() {
    let viewport = Viewport::new(390, 844, UiFormFactor::Mobile);
    let calendar = CalendarModel {
        year: 2026,
        month: 9,
        selected_day: Some(14),
        entries: vec![CalendarEntry {
            date: "2026-09-14".to_owned(),
            card: HomeCard {
                id: Some("show".to_owned()),
                title: "Episode".to_owned(),
                ..Default::default()
            },
            ..Default::default()
        }],
        ..Default::default()
    };
    let layout = draw_test_frame(viewport, |context, assets| {
        draw_calendar(context, viewport, &calendar, assets)
    });
    assert!(
        layout
            .focusable
            .iter()
            .any(|(id, _)| *id == NODE_CALENDAR_CLOSE_DAY)
    );
    assert!(
        layout
            .focusable
            .iter()
            .any(|(id, rect)| *id == NODE_CALENDAR_EVENT_BASE && rect.right() <= viewport.width)
    );
}

#[test]
fn detail_actions_wrap_inside_a_narrow_mobile_layout() {
    let viewport = Viewport::new(320, 720, UiFormFactor::Mobile);
    let detail = DetailModel {
            title: "A long title that needs to wrap on a compact phone".to_owned(),
            meta_line: "Series · 2026 · Drama".to_owned(),
            description: "A longer synopsis that should wrap to the available width instead of pushing the action row off screen. ".repeat(3),
            favorites_enabled: true,
            ..Default::default()
        };
    let layout = draw_test_frame(viewport, |context, assets| {
        draw_detail(context, viewport, &detail, assets)
    });
    for node in [
        NODE_DETAIL_PLAY,
        NODE_DETAIL_WATCHLIST,
        NODE_DETAIL_COMPLETED,
        NODE_DETAIL_DROPPED,
        NODE_DETAIL_FAVORITE,
        NODE_DETAIL_BACK,
    ] {
        let rect = layout
            .focusable
            .iter()
            .find(|(id, _)| *id == node)
            .map(|(_, rect)| *rect)
            .expect("detail action should be focusable");
        assert!(rect.left() >= 0.0 && rect.right() <= viewport.width + 0.1);
    }
    let play = layout
        .focusable
        .iter()
        .find(|(id, _)| *id == NODE_DETAIL_PLAY)
        .unwrap()
        .1;
    let back = layout
        .focusable
        .iter()
        .find(|(id, _)| *id == NODE_DETAIL_BACK)
        .unwrap()
        .1;
    assert!(
        !back.intersects(play),
        "back button must not overlap the play action"
    );
}

#[test]
fn core_snapshot_projection_maps_library_tabs_to_shared_cards() {
    let model = library_model_from_core_snapshot(&json!({
        "profile": {"active": {"nuvioAccessToken":"token"}},
        "settings": {"values": {"integrationLibrarySource":"nuvio"}},
        "library": {
            "isLoading": false,
            "watchlist": [{
                "id": "movie:1",
                "type": "movie",
                "name": "Saved movie",
                "poster": "https://example.test/movie.jpg"
            }],
            "continueWatching": [{
                "id": "series:1",
                "type": "series",
                "name": "Watching show",
                "timeOffset": 30,
                "duration": 120
            }],
            "completed": [],
            "dropped": [],
            "liked": []
        }
    }));

    assert_eq!(model.cards(LibraryTab::Watchlist)[0].title, "Saved movie");
    assert_eq!(model.source, "nuvio");
    assert!(
        model
            .source_options
            .iter()
            .any(|(value, _)| value == "nuvio")
    );
    assert_eq!(model.cards(LibraryTab::Watching)[0].progress, 0.25);
    assert_eq!(model.cards(LibraryTab::Completed).len(), 0);
}

#[test]
fn core_snapshot_projection_maps_discover_catalogs_and_results() {
    let model = discover_model_from_core_snapshot(&json!({
        "discover": {
            "contentType": "series",
            "filters": {
                "catalogKey": "popular",
                "extra": {"genre": "drama", "search": "Dune"}
            },
            "catalogs": [{
                "key": "popular",
                "label": "Popular shows",
                "type": "series",
                "extras": [{"name":"genre", "options":["action", "drama"], "isRequired":true}]
            }],
            "results": [{"id": "series:1", "type": "series", "name": "A show", "year": 2026}],
            "isLoading": false,
            "catalogsLoading": false
        }
    }));

    assert_eq!(model.content_type, "series");
    assert_eq!(model.selected_catalog_key, "popular");
    assert_eq!(model.selected_extra_name, "genre");
    assert_eq!(model.selected_extra_value, "drama");
    assert_eq!(model.query, "Dune");
    assert_eq!(model.catalogs[0].key, "popular");
    assert_eq!(model.catalogs[0].content_type, "series");
    assert_eq!(model.catalogs[0].extras[0].0, "genre");
    assert_eq!(model.catalogs[0].extras[0].1, ["action", "drama"]);
    assert_eq!(model.catalogs[0].required_extras, ["genre"]);
    assert_eq!(model.results[0].title, "A show");
    assert!(!model.is_loading);
}

#[test]
fn discover_projects_all_results_and_requests_the_next_page_near_the_end() {
    let items = (0..80)
        .map(|index| json!({"id": format!("tt{index}"), "type":"movie", "name":format!("Movie {index}")}))
        .collect::<Vec<_>>();
    let model = discover_model_from_core_snapshot(&json!({
        "discover": {
            "contentType": "movie",
            "generation": 3,
            "filters": {
                "catalogKey": "popular",
                "transportUrl": "https://addon.example/manifest.json",
                "catalogId": "popular",
                "extra": {"genre":"Action"}
            },
            "results": items,
            "paging": {"nextSkip":80,"hasMore":true,"isLoading":false,"error":null}
        }
    }));
    assert_eq!(model.results.len(), 80);
    assert_eq!(model.next_page.as_ref().unwrap()["skip"], 80);
    let viewport = Viewport::new(390, 844, UiFormFactor::Mobile);
    let mut top = Vec::new();
    draw_test_frames(viewport, 3, |context, assets| {
        let layout = draw_discover(context, viewport, &model, assets, None);
        top.extend(layout.load_more.clone());
        layout
    });
    assert!(top.is_empty());
    let near_end = viewport.with_scroll_y(1.0e6);
    let mut bottom = Vec::new();
    draw_test_frames(near_end, 3, |context, assets| {
        let layout = draw_discover(context, near_end, &model, assets, None);
        bottom.extend(layout.load_more.clone());
        layout
    });
    assert_eq!(bottom.len(), 1);
    assert_eq!(bottom[0]["type"], "discoverPageRequested");
    assert_eq!(bottom[0]["skip"], 80);
}

#[test]
fn discover_prefetches_a_followup_page_before_the_initial_batch_is_exhausted() {
    let items = (0..20)
        .map(|index| json!({"id": format!("tt{index}"), "type":"movie", "name":format!("Movie {index}")}))
        .collect::<Vec<_>>();
    let model = discover_model_from_core_snapshot(&json!({
        "discover": {
            "contentType": "movie",
            "generation": 4,
            "filters": {
                "catalogKey": "popular",
                "transportUrl": "https://addon.example/manifest.json",
                "catalogId": "popular",
                "extra": {}
            },
            "results": items,
            "paging": {"nextSkip":20,"hasMore":true,"isLoading":false,"error":null}
        }
    }));
    let viewport = Viewport::new(1280, 720, UiFormFactor::Desktop);
    let top = draw_test_frame(viewport, |context, assets| {
        draw_discover(context, viewport, &model, assets, None)
    });
    assert_eq!(top.load_more.len(), 1);
    assert_eq!(top.load_more[0]["skip"], 20);
}

#[test]
fn mobile_discover_search_sits_above_three_horizontal_filters() {
    let viewport = Viewport::new(360, 800, UiFormFactor::Mobile);
    let discover = DiscoverModel {
        language: "en".to_owned(),
        content_type: "movie".to_owned(),
        content_types: vec!["movie".to_owned(), "series".to_owned()],
        selected_catalog_key: "popular".to_owned(),
        catalogs: vec![DiscoverCatalog {
            key: "popular".to_owned(),
            label: "Popular movies".to_owned(),
            content_type: "movie".to_owned(),
            extras: vec![("genre".to_owned(), vec!["Action".to_owned()])],
            required_extras: vec![],
        }],
        ..Default::default()
    };
    let layout = draw_test_frame(viewport, |context, assets| {
        draw_discover(context, viewport, &discover, assets, None)
    });
    let rect_for = |node| {
        layout
            .focusable
            .iter()
            .find(|(id, _)| *id == node)
            .map(|(_, rect)| *rect)
            .unwrap()
    };
    let search = rect_for(NODE_DISCOVER_SEARCH);
    let filters = [
        rect_for(NODE_DISCOVER_TYPE_BASE),
        rect_for(NODE_DISCOVER_CATALOG_BASE),
        rect_for(NODE_DISCOVER_EXTRA),
    ];
    assert!(search.bottom() < filters[0].top());
    assert!((filters[0].top() - filters[1].top()).abs() < 0.1);
    assert!((filters[1].top() - filters[2].top()).abs() < 0.1);
    assert!(filters[0].right() <= filters[1].left());
    assert!(filters[1].right() <= filters[2].left());
}

#[test]
fn recent_searches_list_under_search_only_while_it_is_focused() {
    let viewport = Viewport::new(1280, 800, UiFormFactor::Desktop);
    let discover = DiscoverModel {
        language: "en".to_owned(),
        content_type: "movie".to_owned(),
        recent_searches: vec!["dune".to_owned(), "alien".to_owned()],
        ..Default::default()
    };
    let drawn = |focused| {
        draw_test_frame(viewport, |context, assets| {
            draw_discover(context, viewport, &discover, assets, focused)
        })
    };
    let closed = drawn(None);
    assert!(
        closed
            .focusable
            .iter()
            .all(|(id, _)| discover_recent(*id).is_none())
    );
    let open = drawn(Some(NODE_DISCOVER_SEARCH));
    let search = open
        .focusable
        .iter()
        .find(|(id, _)| *id == NODE_DISCOVER_SEARCH)
        .map(|(_, rect)| *rect)
        .unwrap();
    let rows = open
        .focusable
        .iter()
        .filter(|(id, _)| discover_recent(*id).is_some())
        .collect::<Vec<_>>();
    assert_eq!(rows.len(), 2);
    assert!(rows.iter().all(|(_, rect)| rect.top() > search.bottom()));
}

#[test]
fn discover_grid_fills_width_and_starts_after_filters() {
    let viewport = Viewport::new(1880, 920, UiFormFactor::Desktop);
    let metrics = UiMetrics::for_viewport(viewport);
    let available = viewport.width - 2.0 * screen_margin(metrics);
    let (columns, card_width) = discover_grid_geometry(
        available,
        metrics.poster_card_width,
        metrics.horizontal_spacing,
    );
    assert!(columns > 1);
    assert!(
        (columns as f32 * card_width
            + columns.saturating_sub(1) as f32 * metrics.horizontal_spacing
            - available)
            .abs()
            <= 1.1
    );

    let discover = DiscoverModel {
        content_type: "movie".to_owned(),
        results: (0..columns + 1)
            .map(|index| HomeCard {
                title: format!("Movie {index}"),
                ..Default::default()
            })
            .collect(),
        ..Default::default()
    };
    let layout = draw_test_frame(viewport, |context, assets| {
        draw_discover(context, viewport, &discover, assets, None)
    });
    let rect = |node| {
        layout
            .focusable
            .iter()
            .find(|(id, _)| *id == node)
            .map(|(_, rect)| *rect)
            .unwrap()
    };
    let poster_gap = rect(NODE_CARD_BASE).top() - rect(NODE_DISCOVER_EXTRA).bottom();
    assert!(
        poster_gap > 0.0 && poster_gap <= metrics.section_gap * 2.0,
        "poster gap is {poster_gap}px"
    );
    assert!(
        (rect(NODE_CARD_BASE + columns as u64 - 1).right()
            - (viewport.width - screen_margin(metrics)))
        .abs()
            <= 2.0
    );
    assert!(rect(NODE_CARD_BASE + columns as u64).top() > rect(NODE_CARD_BASE).top());
}

#[test]
fn discover_results_scroll_below_fixed_filters() {
    let viewport = Viewport::new(390, 844, UiFormFactor::Mobile);
    let discover = DiscoverModel {
        content_type: "movie".to_owned(),
        results: (0..80)
            .map(|index| HomeCard {
                title: format!("Movie {index}"),
                ..Default::default()
            })
            .collect(),
        ..Default::default()
    };
    let first = draw_test_frames(viewport, 3, |context, assets| {
        draw_discover(context, viewport, &discover, assets, None)
    });
    let scrolled_viewport = viewport.with_scroll_y(80.0);
    let scrolled = draw_test_frames(scrolled_viewport, 3, |context, assets| {
        draw_discover(context, scrolled_viewport, &discover, assets, None)
    });
    let rect = |layout: &HomeLayout, node| {
        layout
            .focusable
            .iter()
            .find(|(id, _)| *id == node)
            .map(|(_, rect)| *rect)
            .unwrap()
    };
    let header_delta =
        rect(&first, NODE_DISCOVER_EXTRA).top() - rect(&scrolled, NODE_DISCOVER_EXTRA).top();
    let card_delta =
        rect(&first, NODE_CARD_BASE).bottom() - rect(&scrolled, NODE_CARD_BASE).bottom();
    assert!(header_delta.abs() < 1.0);
    assert!((card_delta - 80.0).abs() < 1.0);
}

#[test]
fn core_snapshot_projection_maps_detail_and_settings() {
    let snapshot = json!({
        "detail": {
            "id": "tt42",
            "contentType": "movie",
            "isLoading": false,
            "isInWatchlist": true,
            "meta": {
                "name": "Featured movie",
                "type": "movie",
                "year": 2026,
                "overview": "A useful description.",
                "posterUrl": "https://example.test/poster.jpg"
            },
            "similarItems": [{"id": "tt43", "type": "movie", "name": "Similar"}],
            "franchiseItems": [{"id": "tt41", "type": "movie", "name": "Prequel"}],
            "trailers": [{"url":"https://example.test/trailer.mp4"}],
            "mdblistRatings": {"imdb": 8.2, "tmdb": 76, "metacritic": 71}
        },
        "library": {
            "completed": [{"id":"tt42"}],
            "dropped": [],
            "liked": [{"id":"tt42"}]
        },
        "settings": {
            "values": {"compactCards": false},
            "lastWriteError": null
        }
    });
    let detail = detail_model_from_core_snapshot(&snapshot);
    let settings = settings_model_from_core_snapshot(&snapshot);
    assert_eq!(detail.id, "tt42");
    assert_eq!(detail.title, "Featured movie");
    assert!(detail.in_watchlist);
    assert!(detail.completed);
    assert!(!detail.dropped);
    assert!(detail.favorite);
    assert_eq!(detail.ratings.len(), 3);
    assert_eq!(detail.trailers[0], "https://example.test/trailer.mp4");
    assert_eq!(detail.similar[0].title, "Similar");
    assert_eq!(detail.franchise[0].title, "Prequel");
    assert_eq!(
        detail
            .linked_card(NODE_DETAIL_FRANCHISE_BASE)
            .unwrap()
            .title,
        "Prequel"
    );
    assert_eq!(
        detail.linked_card(NODE_DETAIL_SIMILAR_BASE).unwrap().title,
        "Similar"
    );
    assert!(!settings.bool_value("compactCards"));
    assert!(settings.bool_value("notificationsEnabled"));
}

#[test]
fn responsive_metrics_are_loaded_from_shared_token_contract() {
    let desktop = UiMetrics::for_viewport(Viewport::new(1920, 1080, UiFormFactor::Desktop));
    let mobile = UiMetrics::for_viewport(Viewport::new(390, 844, UiFormFactor::Mobile));
    let tv = UiMetrics::for_viewport(Viewport::new(1920, 1080, UiFormFactor::Tv));

    assert_eq!(desktop.accent, Color32::WHITE);
    assert_eq!(desktop.page_padding, 24.0);
    assert_eq!(desktop.horizontal_card_width, 300.0);
    assert_eq!(desktop.poster_card_width, 200.0);
    assert_eq!(desktop.poster_card_height, 300.0);
    let narrow = UiMetrics::for_viewport(Viewport::new(1100, 800, UiFormFactor::Desktop));
    assert!((140.0..200.0).contains(&narrow.poster_card_width));
    assert_eq!(mobile.poster_card_height, mobile.poster_card_width * 1.5);
    assert_eq!(mobile.page_padding, 16.0);
    assert_eq!(mobile.horizontal_card_width, 244.0);
    assert_eq!(tv.horizontal_card_width, 356.0);
    assert_eq!(desktop.poster_height_ratio, 1.5);
    assert_eq!(desktop.library_search_reserved_width, 150.0);
    assert_eq!(desktop.calendar_panel_width_tv, 440.0);
    assert_eq!(desktop.focus_ring_width, 2.5);
    assert_eq!(desktop.settings_extended_line_spacing_tv, 42.0);
    assert_eq!(desktop.home_hero_content_max_width_desktop, 980.0);
    assert_eq!(desktop.home_hero_logo_max_width_desktop, 820.0);
    assert_eq!(desktop.home_hero_synopsis_size_desktop, 16.0);
    assert_eq!(desktop.settings_nav_width, 268.0);
    assert_eq!(desktop.settings_content_offset, 300.0);
    assert_eq!(desktop.settings_content_max_width_desktop, 720.0);
    assert_eq!(desktop.settings_row_height, 52.0);
    assert_eq!(desktop.settings_row_label_size_desktop, 18.0);
    assert_eq!(mobile.settings_nav_width, 196.0);
    assert_eq!(mobile.settings_row_height, 46.0);
}

#[test]
fn native_appearance_exposes_web_preferences_and_uses_white_default_accent() {
    let keys: Vec<_> = SETTINGS_SECTIONS
        .iter()
        .filter(|section| section.category == "Appearance")
        .flat_map(|section| section.rows.iter().map(|row| row.key))
        .collect();
    for key in [
        "uiScale",
        "navLayout",
        "posterWidthPreset",
        "continueWatchingArtwork",
        "homeHeroAutoplayTrailer",
        "detailSeasonSelectorMode",
        "episodeCardsLayout",
    ] {
        assert!(keys.contains(&key), "native Appearance is missing {key}");
    }
    for key in ["navBarPosition", "navItemsAlign"] {
        assert!(
            !keys.contains(&key),
            "native Appearance should not expose {key}"
        );
    }
}

#[test]
fn shortcuts_only_listed_on_desktop() {
    let listed = |form_factor, width, height| {
        super::settings::categories(Viewport::new(width, height, form_factor))
            .any(|index| SETTINGS_SECTIONS[index].title == "Shortcuts")
    };
    assert!(listed(UiFormFactor::Desktop, 1920, 1080));
    assert!(!listed(UiFormFactor::Tv, 1920, 1080));
    assert!(!listed(UiFormFactor::Mobile, 390, 844));
}

#[test]
fn settings_search_spans_every_section() {
    let settings = SettingsModel {
        search: "subtitle".to_owned(),
        ..Default::default()
    };
    let viewport = Viewport::new(1920, 1080, UiFormFactor::Desktop);
    let groups = super::settings::visible_groups(&settings, viewport);
    let rows: Vec<_> = groups
        .iter()
        .flat_map(|(_, rows)| rows)
        .map(|&index| settings_row_by_index(index).unwrap().key)
        .collect();
    assert!(rows.contains(&"subtitleSize"));
    assert!(rows.contains(&"downloadSubtitleLanguage"));
    assert!(!rows.contains(&"uiScale"));
}

#[test]
fn layout_follows_width_not_host_device() {
    for form_factor in [UiFormFactor::Desktop, UiFormFactor::Mobile] {
        assert_eq!(
            Viewport::new(430, 932, form_factor).size_class(),
            SizeClass::Compact
        );
        assert_eq!(
            Viewport::new(720, 1024, form_factor).size_class(),
            SizeClass::Medium
        );
        assert_eq!(
            Viewport::new(1280, 800, form_factor).size_class(),
            SizeClass::Expanded
        );
    }
}

#[test]
fn tv_never_falls_back_to_the_phone_layout() {
    let _guard = screens();
    let tv = Viewport::new(480, 270, UiFormFactor::Tv);
    assert!(!tv.is_compact());
    assert!(!tv.is_phone());
    assert_eq!(tv.size_class(), SizeClass::Expanded);
    let bar = nav_bar_rect(tv).expect("top bar");
    assert!(
        bar.top() < tv.height * 0.25,
        "TV nav sits at the top: {bar:?}"
    );
}

#[test]
fn tablets_use_the_top_bar_and_only_phones_get_the_bottom_bar() {
    let _guard = screens();
    let phone = Viewport::new(430, 932, UiFormFactor::Mobile);
    assert!(phone.is_phone());
    let bottom = nav_bar_rect(phone).expect("bottom bar");
    assert!(bottom.top() > phone.height * 0.5);
    for width in [700, 1000] {
        let tablet = Viewport::new(width, 1024, UiFormFactor::Mobile);
        assert!(tablet.is_tablet() && !tablet.is_phone());
        let bar = nav_bar_rect(tablet).expect("top bar");
        assert!(
            bar.top() < tablet.height * 0.25,
            "{width}dp tablet: {bar:?}"
        );
    }
}

#[test]
fn artwork_requests_use_provider_size_variants() {
    let url = "https://image.tmdb.org/t/p/w600_and_h900_bestv2/poster.jpg";
    assert!(artwork_request_url(url, [300, 450]).contains("/t/p/w500/poster.jpg"));
    assert!(artwork_request_url(url, [1920, 1080]).contains("/t/p/original/poster.jpg"));
    assert_eq!(
        artwork_request_url("https://example.test/poster.webp", [300, 450]),
        "https://example.test/poster.webp"
    );
}

#[test]
fn shared_dropdowns_keep_full_control_height_instead_of_collapsing_to_text() {
    let viewport = Viewport::new(1920, 1080, UiFormFactor::Desktop);
    let metrics = UiMetrics::for_viewport(viewport);
    let context = crate::gfx::Context::default();
    context.set_fonts(crate::fonts::definitions());
    let mut heights = Vec::new();
    context
        .run_ui(
            crate::gfx::RawInput {
                screen_rect: Some(Rect::from_min_size(
                    Pos2::ZERO,
                    Vec2::new(viewport.width, viewport.height),
                )),
                ..Default::default()
            },
            |ui| {
                let options = vec![("recent".to_owned(), "Recently updated".to_owned())];
                let (select, _) = components::dropdown(
                    ui,
                    "dropdown-test",
                    "recent",
                    &options,
                    240.0,
                    false,
                    metrics,
                );
                let placeholder = components::dropdown_placeholder(
                    ui,
                    "dropdown-placeholder-test",
                    "Genre",
                    240.0,
                    metrics,
                );
                heights.push(select.rect.height());
                heights.push(placeholder.rect.height());
            },
        )
        .drop_without_applying_deltas();

    assert!(heights.iter().all(|height| *height >= 38.0));
}

#[test]
fn discover_preserves_the_web_three_filter_slots_without_catalogs() {
    let viewport = Viewport::new(1920, 1080, UiFormFactor::Desktop);
    let discover = DiscoverModel {
        language: "en".to_owned(),
        content_type: "movie".to_owned(),
        content_types: vec!["movie".to_owned(), "series".to_owned()],
        ..Default::default()
    };
    let layout = draw_test_frame(viewport, |context, assets| {
        draw_discover(context, viewport, &discover, assets, None)
    });
    for node in [
        NODE_DISCOVER_TYPE_BASE,
        NODE_DISCOVER_CATALOG_BASE,
        NODE_DISCOVER_EXTRA,
    ] {
        assert!(
            layout.focusable.iter().any(|(id, _)| *id == node),
            "Discover must retain a type, catalog, and genre control slot"
        );
    }
}

#[test]
fn discover_keeps_three_filters_on_one_desktop_row_when_space_is_tight() {
    let viewport = Viewport::new(700, 900, UiFormFactor::Desktop);
    let discover = DiscoverModel {
        language: "en".to_owned(),
        content_type: "movie".to_owned(),
        content_types: vec!["movie".to_owned(), "series".to_owned()],
        selected_catalog_key: "popular".to_owned(),
        catalogs: vec![DiscoverCatalog {
            key: "popular".to_owned(),
            label: "Popular movies".to_owned(),
            content_type: "movie".to_owned(),
            extras: vec![("genre".to_owned(), vec!["Action".to_owned()])],
            required_extras: vec![],
        }],
        ..Default::default()
    };
    let layout = draw_test_frame(viewport, |context, assets| {
        draw_discover(context, viewport, &discover, assets, None)
    });
    let rect_for = |node| {
        layout
            .focusable
            .iter()
            .find(|(id, _)| *id == node)
            .map(|(_, rect)| *rect)
            .expect("Discover filter should be focusable")
    };
    let controls = [
        rect_for(NODE_DISCOVER_TYPE_BASE),
        rect_for(NODE_DISCOVER_CATALOG_BASE),
        rect_for(NODE_DISCOVER_EXTRA),
    ];
    assert!(
        controls
            .iter()
            .all(|rect| (rect.top() - controls[0].top()).abs() < 1.0),
        "filter row wrapped: {controls:?}"
    );
    assert!(controls[2].right() <= viewport.width);
}

#[test]
fn settings_row_labels_reuse_existing_web_translations() {
    let anime_skip = SettingsRow {
        label: "Anime skip integration",
        key: "useAnimeSkip",
        fixed: &[],
    };
    let seek_thumbnails = SettingsRow {
        label: "Seek thumbnail preview",
        key: "seekThumbnailEnabled",
        fixed: &[],
    };

    assert_eq!(
        super::settings::settings_row_label(&anime_skip, "tr"),
        localized("settings.use_animeskip", "tr")
    );
    assert_eq!(
        super::settings::settings_row_label(&seek_thumbnails, "tr"),
        localized("settings.seek_thumbnails", "tr")
    );
}

#[test]
fn clicking_library_search_then_typing_reports_the_query() {
    let viewport = Viewport::new(1600, 900, UiFormFactor::Desktop);
    let context = crate::gfx::Context::default();
    context.set_fonts(crate::fonts::definitions());
    let mut assets = EmptyHomeAssets;
    let library = LibraryModel::default();
    let page = PageLayout::new(viewport, UiMetrics::for_viewport(viewport), false);
    let target = page.search.center();
    let frames = [
        Vec::new(),
        Vec::new(),
        vec![crate::gfx::Event::PointerMoved(target)],
        vec![crate::gfx::Event::PointerButton {
            pos: target,
            button: crate::gfx::PointerButton::Primary,
            pressed: true,
            modifiers: crate::gfx::Modifiers::NONE,
        }],
        vec![crate::gfx::Event::PointerButton {
            pos: target,
            button: crate::gfx::PointerButton::Primary,
            pressed: false,
            modifiers: crate::gfx::Modifiers::NONE,
        }],
        vec![crate::gfx::Event::Text("du".to_owned())],
    ];
    let mut typed = None;
    for (frame, events) in frames.into_iter().enumerate() {
        context
            .run_ui(
                crate::gfx::RawInput {
                    screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1600.0, 900.0))),
                    time: Some(frame as f64 / 60.0),
                    events,
                    ..Default::default()
                },
                |ui| {
                    let layout = draw_library(
                        ui.ctx(),
                        viewport,
                        &library,
                        LibraryTab::Watchlist,
                        &mut assets,
                    );
                    if layout.text_input.is_some() {
                        typed = layout.text_input;
                    }
                },
            )
            .drop_without_applying_deltas();
    }
    assert_eq!(typed.as_deref(), Some("du"));
}

#[test]
fn scrolled_detail_swipe_lands_on_episode_row() {
    let detail = DetailModel {
        content_type: "series".to_owned(),
        episodes: (1..=8)
            .map(|number| DetailEpisode {
                id: format!("tt1:1:{number}"),
                season: 1,
                number,
                ..Default::default()
            })
            .collect(),
        episode_layout: Some(EpisodeLayout::Cards),
        ..Default::default()
    };
    let viewport = Viewport::new(412, 915, UiFormFactor::Mobile);
    let viewport = viewport.with_scroll_y(1.0e6);
    let mut regions = Vec::new();
    draw_test_frames(viewport, 3, |context, assets| {
        let layout = draw_detail(context, viewport, &detail, assets);
        regions = take_scroll_regions(context);
        layout
    });
    let row = regions
        .iter()
        .find(|region| region.id == ScrollId::Row(1))
        .expect("episode row should report its scroll region");
    assert!(row.rect.top() < 915.0 && row.rect.bottom() > 0.0);
}

#[test]
fn library_hides_status_dropdown_when_source_has_no_statuses() {
    let viewport = Viewport::new(390, 844, UiFormFactor::Mobile);
    let library = LibraryModel::default();
    let layout = draw_test_frame(viewport, |context, assets| {
        draw_library(context, viewport, &library, LibraryTab::Watchlist, assets)
    });
    let has = |id| layout.focusable.iter().any(|(node, _)| *node == id);
    assert!(!has(NODE_LIBRARY_STATUS));
    assert!(has(NODE_LIBRARY_TYPE) && has(NODE_LIBRARY_SORT));
}

#[test]
fn skip_and_next_cards_stay_on_screen_for_every_form_factor() {
    for form_factor in [
        UiFormFactor::Desktop,
        UiFormFactor::Mobile,
        UiFormFactor::Tv,
    ] {
        let viewport = Viewport::new(960, 540, form_factor);
        for controls_visible in [true, false] {
            let player = PlayerModel {
                passthrough: true,
                duration: 1400.0,
                position: 1300.0,
                controls_visible,
                skip: Some(SkipCard {
                    kind: SkipKind::Outro,
                    seek_to: 1390.0,
                    progress: 0.5,
                }),
                next_episode: Some(NextEpisodeCard {
                    label: "S1:E2".to_owned(),
                    title: "Second".to_owned(),
                    thumbnail: None,
                    countdown: Some(5),
                }),
                ..PlayerModel::default()
            };
            let layout = draw_test_frames(viewport, 3, |context, assets| {
                draw_player(context, viewport, &player, assets, None)
            });
            let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(960.0, 540.0));
            for node in [
                NODE_PLAYER_SKIP,
                NODE_PLAYER_NEXT_PLAY,
                NODE_PLAYER_NEXT_DISMISS,
            ] {
                let (_, rect) = layout
                    .focusable
                    .iter()
                    .find(|(id, _)| *id == node)
                    .unwrap_or_else(|| panic!("{node} missing on {form_factor:?}"));
                assert!(
                    screen.contains_rect(*rect),
                    "{node} off screen on {form_factor:?}"
                );
            }
        }
    }
}

#[test]
fn app_extras_feed_the_detail_model() {
    let snapshot = serde_json::json!({
        "detail": {
            "id": "tt1",
            "contentType": "series",
            "meta": {
                "name": "Show",
                "year": "2026",
                "videos": [
                    {"id": "tt1:0:1", "season": 0, "episode": 1},
                    {"id": "tt1:1:1", "season": 1, "episode": 1}
                ],
                "app_extras": {
                    "certification": "TV-14",
                    "certificationLocal": "TV-MA",
                    "seasonPosters": {"1": "https://img/s1.jpg", "0": "https://img/s0.jpg", "x": "bad"},
                    "directors": [{"name": "Dana", "character": "Dana", "photo": null}],
                    "writers": [{"name": "Dana", "character": "Dana"}, {"name": "Lee"}],
                    "watchProviders": [{"name": "Crunchyroll", "url": "u"}, "Netflix", {"name": "Netflix"}],
                    "releaseDates": {"results": [{"iso_3166_1": "US", "release_dates": [
                        {"type": 4, "release_date": "2026-07-01T00:00:00.000Z"},
                        {"type": 3, "release_date": "2026-05-01T00:00:00.000Z"}
                    ]}]}
                }
            }
        }
    });
    let detail = detail_model_from_core_snapshot(&snapshot);
    assert_eq!(
        detail.season_posters,
        vec![
            (1, "https://img/s1.jpg".to_owned()),
            (0, "https://img/s0.jpg".to_owned())
        ]
    );
    assert_eq!(detail.facts.get(1).map(String::as_str), Some("TV-MA"));
    let crew = detail
        .crew
        .iter()
        .map(|member| member.name.as_str())
        .collect::<Vec<_>>();
    assert_eq!(crew, ["Dana", "Dana", "Lee"]);
    assert!(detail.info_lines[0].contains("2026-05-01"));
    assert!(detail.info_lines[0].contains("2026-07-01"));
    assert!(detail.info_lines[1].contains("Crunchyroll, Netflix"));
}

#[test]
fn show_info_lists_crew_dates_and_country_from_addon_meta() {
    let meta = json!({
        "director": ["Rob Savage"],
        "writer": ["Josh Malerman", "Nathan Elston"],
        "year": "2026",
        "released": "2026-10-08T00:00:00.000Z",
        "runtime": "1h33min",
        "country": "United States of America"
    });
    let info = detail_show_info(&meta, "en");
    let values = info
        .iter()
        .map(|(_, value)| value.as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        values,
        [
            "Rob Savage",
            "Josh Malerman, Nathan Elston",
            "2026",
            "2026-10-08",
            "1h33min",
            "United States of America"
        ]
    );
    assert!(detail_show_info(&json!({}), "en").is_empty());
}

#[test]
fn trailer_cards_keep_title_thumbnail_and_url() {
    let detail = json!({"trailers": [
        {"title": "Official Trailer 2", "url": "https://youtu.be/a", "thumbnail": "https://img/a.jpg"},
        {"title": "No link"},
        {"title": "Teaser", "url": "https://youtu.be/b"}
    ]});
    let cards = detail_trailer_cards(&detail);
    assert_eq!(cards.len(), 2);
    assert_eq!(cards[0].title, "Official Trailer 2");
    assert_eq!(cards[0].thumbnail.as_deref(), Some("https://img/a.jpg"));
    assert_eq!(cards[1].url, "https://youtu.be/b");
}

#[test]
fn hidden_sections_take_no_space_and_order_is_respected() {
    let viewport = Viewport::new(1280, 720, UiFormFactor::Desktop);
    let mut detail = DetailModel {
        trailer_cards: vec![DetailTrailer::default()],
        show_info: (0..12)
            .map(|index| (format!("Label {index}"), "Value".to_owned()))
            .collect(),
        ..Default::default()
    };
    let full = measured_detail_max(viewport, &detail);
    detail.sections = Some(vec![DetailSection::ShowInfo]);
    let info_only = measured_detail_max(viewport, &detail);
    detail.sections = Some(Vec::new());
    assert!(full > info_only);
    assert!(info_only > measured_detail_max(viewport, &detail));
}

#[test]
fn show_info_rows_each_take_their_row_height() {
    let viewport = Viewport::new(1280, 720, UiFormFactor::Desktop);
    let row = UiMetrics::for_viewport(viewport).detail_info_row;
    let rows = |count: usize| DetailModel {
        sections: Some(vec![DetailSection::ShowInfo]),
        show_info: (0..count)
            .map(|index| (format!("Label {index}"), "Value".to_owned()))
            .collect(),
        ..Default::default()
    };
    let few = measured_detail_max(viewport, &rows(2));
    let many = measured_detail_max(viewport, &rows(12));
    assert!(many - few >= 9.0 * row, "{few} {many} {row}");
}

#[test]
fn posters_mode_falls_back_to_chips_without_season_posters() {
    let mut detail = DetailModel {
        season_selector: SeasonSelector::from_setting(Some("tabs")),
        ..Default::default()
    };
    assert_eq!(detail.season_selector_mode(), SeasonSelector::Chips);
    detail.season_posters = vec![(1, "https://img/s1.jpg".to_owned())];
    assert_eq!(detail.season_selector_mode(), SeasonSelector::Posters);
    assert_eq!(
        SeasonSelector::from_setting(Some("dropdown")),
        SeasonSelector::Dropdown
    );
}

#[test]
fn episode_layout_defaults_follow_form_factor_and_setting() {
    let mut detail = DetailModel::default();
    assert_eq!(detail.episode_layout_for(true), EpisodeLayout::List);
    assert_eq!(detail.episode_layout_for(false), EpisodeLayout::Cards);
    detail.episode_layout = EpisodeLayout::from_setting(Some("grid"));
    assert_eq!(detail.episode_layout_for(true), EpisodeLayout::List);
    assert_eq!(detail.episode_layout_for(false), EpisodeLayout::Grid);
    assert_eq!(EpisodeLayout::from_setting(Some("auto")), None);
}

#[test]
fn unwatched_episodes_hide_still_and_spoilers() {
    let library = serde_json::json!({"watched": {"tt1:1:1": true}});
    let meta = serde_json::json!({"videos": [
        {"id": "tt1:1:1", "season": 1, "episode": 1, "title": "Pilot", "overview": "Seen"},
        {"id": "tt1:1:2", "season": 1, "episode": 2, "title": "Next", "overview": "Secret"}
    ]});
    let episodes = detail_episodes(&meta, &library);
    let mut detail = DetailModel {
        show_episode_titles: true,
        show_episode_descriptions: true,
        blur_unwatched: true,
        ..DetailModel::default()
    };
    assert!(episodes[0].watched && !episodes[1].watched);
    let seen = detail.episode_view(&episodes[0]);
    assert!(!seen.hide_still && seen.title == "Pilot");
    let next = detail.episode_view(&episodes[1]);
    assert!(next.hide_still && next.title == "Next");
    detail.hide_spoilers = true;
    let next = detail.episode_view(&episodes[1]);
    assert!(next.title.is_empty() && next.overview.is_none());
    assert_eq!(detail.episode_view(&episodes[0]).overview, Some("Seen"));
}

#[test]
fn folder_all_tab_merges_sources_without_duplicates() {
    let snapshot = serde_json::json!({
        "navigation": {"params": {"folderId": "f"}},
        "home": {"categories": [
            {"id": "f", "type": "collection_folder", "name": "Folder", "showAllTab": true, "viewMode": "TABBED_GRID"},
            {"id": "f#0", "type": "collection_folder_source", "folderId": "f", "name": "A", "canLoadMore": false,
             "items": [{"id": "1", "type": "movie", "name": "One"}, {"id": "2", "type": "movie", "name": "Two"}]},
            {"id": "f#1", "type": "collection_folder_source", "folderId": "f", "name": "B", "canLoadMore": false,
             "items": [{"id": "2", "type": "movie", "name": "Two"}, {"id": "3", "type": "movie", "name": "Three"}]}
        ]}
    });
    let folder = folder_model_from_core_snapshot(&snapshot);
    assert!(folder.has_all_tab);
    assert_eq!(folder.tabs.len(), 3);
    assert_eq!(folder.cards(0).len(), 3);
    assert_eq!(folder.cards(2).len(), 2);
}

#[test]
fn folder_grid_scrolls_under_the_header() {
    let viewport = Viewport::new(390, 844, UiFormFactor::Mobile);
    let folder = FolderModel {
        id: "f".to_owned(),
        title: "Folder".to_owned(),
        language: "en".to_owned(),
        tabs: vec![FolderTab {
            label: "A".to_owned(),
            cards: (0..60)
                .map(|index| HomeCard {
                    title: format!("Movie {index}"),
                    ..Default::default()
                })
                .collect(),
            ..Default::default()
        }],
        ..Default::default()
    };
    let rest = draw_test_frames(viewport, 3, |context, assets| {
        draw_folder(context, viewport, &folder, 0, assets, None)
    });
    let scrolled_viewport = viewport.with_scroll_y(120.0);
    let scrolled = draw_test_frames(scrolled_viewport, 3, |context, assets| {
        draw_folder(context, scrolled_viewport, &folder, 0, assets, None)
    });
    let bottom = |layout: &HomeLayout| {
        layout
            .focusable
            .iter()
            .find(|(node, _)| *node == NODE_CARD_BASE)
            .map(|(_, rect)| rect.bottom())
            .expect("first card is focusable")
    };
    assert!((bottom(&rest) - bottom(&scrolled) - 120.0).abs() < 1.0);
}

#[test]
fn download_sections_scroll_under_the_sections_row() {
    let viewport = Viewport::new(390, 844, UiFormFactor::Mobile);
    let library = LibraryModel {
        language: "en".to_owned(),
        downloads_open: true,
        downloads: (0..30)
            .map(|index| DownloadGroup {
                id: format!("group-{index}"),
                title: format!("Title {index}"),
                rows: vec![DownloadRow::default()],
                ..Default::default()
            })
            .collect(),
        ..Default::default()
    };
    let rest = draw_test_frames(viewport, 3, |context, assets| {
        draw_library(context, viewport, &library, LibraryTab::Watchlist, assets)
    });
    let scrolled_viewport = viewport.with_scroll_y(150.0);
    let scrolled = draw_test_frames(scrolled_viewport, 3, |context, assets| {
        draw_library(
            context,
            scrolled_viewport,
            &library,
            LibraryTab::Watchlist,
            assets,
        )
    });
    let rect = |layout: &HomeLayout, node| {
        layout
            .focusable
            .iter()
            .find(|(id, _)| *id == node)
            .map(|(_, rect)| *rect)
    };
    assert_eq!(rect(&rest, NODE_LIBRARY), rect(&scrolled, NODE_LIBRARY));
    assert!(
        rest.focusable.len() != scrolled.focusable.len() || rest.focusable != scrolled.focusable
    );
}

fn measured_detail_max(viewport: Viewport, detail: &DetailModel) -> f32 {
    let mut max = 0.0;
    draw_test_frames(viewport, 3, |context, assets| {
        let layout = draw_detail(context, viewport, detail, assets);
        max = crate::kit::scroll_state(context, "detail").map_or(0.0, |(_, max)| max);
        layout
    });
    max
}

fn rtl_galley(text: &str) -> std::sync::Arc<crate::gfx::Galley> {
    let context = crate::gfx::Context::default();
    context.set_fonts(crate::fonts::definitions());
    for _ in 0..2 {
        context
            .run_ui(crate::gfx::RawInput::default(), |_| {})
            .drop_without_applying_deltas();
    }
    context.fonts_mut(|fonts| {
        fonts.layout_no_wrap(text.to_owned(), crate::fonts::regular(20.0), Color32::WHITE)
    })
}

#[test]
fn arabic_word_is_joined_and_drawn_right_to_left() {
    let galley = rtl_galley("با");
    let glyphs = &galley.rows[0].glyphs;
    assert_eq!(glyphs[0].chr, '\u{0627}');
    assert_eq!(glyphs[1].chr, '\u{0628}');
    assert!(glyphs[0].pos.x < glyphs[1].pos.x);
    let isolated = rtl_galley("\u{0628}");
    assert_ne!(
        glyphs[1].uv_rect.size,
        isolated.rows[0].glyphs[0].uv_rect.size
    );
    assert!(
        glyphs
            .iter()
            .all(|glyph| glyph.uv_rect.size != crate::gfx::vec2(0.0, 0.0))
    );
}

#[test]
fn hebrew_title_has_glyphs_and_keeps_numbers_in_order() {
    let galley = rtl_galley("שלום 2024");
    let chars: String = galley.rows[0]
        .glyphs
        .iter()
        .map(|glyph| glyph.chr)
        .collect();
    assert!(chars.starts_with("2024"));
    assert!(
        galley.rows[0]
            .glyphs
            .iter()
            .filter(|glyph| !glyph.chr.is_whitespace())
            .all(|glyph| glyph.uv_rect.size != crate::gfx::vec2(0.0, 0.0))
    );
}

#[test]
fn mdblist_ratings_keep_source_order_and_precision() {
    let item = serde_json::json!({
        "mdblistRatings": {"trakt": 87.0, "tomatoes": 86.0, "imdb": 8.8}
    });
    let ratings = crate::mdblist_ratings(&item);
    assert_eq!(
        ratings,
        vec![
            ("IMDb".to_owned(), "8.8".to_owned()),
            ("Trakt".to_owned(), "87%".to_owned()),
            ("Rotten Tomatoes".to_owned(), "86%".to_owned()),
        ]
    );
}

#[test]
fn series_meta_line_counts_seasons_and_episodes_without_specials() {
    let item = serde_json::json!({
        "type": "series",
        "year": 2025,
        "videos": [
            {"season": 0, "number": 1},
            {"season": 1, "number": 1},
            {"season": 1, "number": 2},
            {"season": 2, "number": 1}
        ]
    });
    let line = crate::hero_meta_line(&item, "en");
    assert!(
        line.starts_with("Series · 2 Seasons · 3 Episodes · 2025"),
        "{line}"
    );
}

#[test]
fn arabic_and_hebrew_load_their_own_files() {
    assert_eq!(localized("nav.library", "ar"), "المكتبة");
    assert_eq!(localized("nav.library", "he"), "ספרייה");
    assert_eq!(localized("nav.library", "iw"), "ספרייה");
}

#[test]
fn action_menu_fades_out_after_the_host_stops_drawing_it() {
    let context = crate::gfx::Context::default();
    let viewport = Viewport::new(1280, 800, UiFormFactor::Desktop);
    let metrics = UiMetrics::for_viewport(viewport);
    let assets = EmptyHomeAssets;
    let items = [ActionMenuItem {
        icon: "Play",
        label: "Play".to_owned(),
        app_icon: None,
    }];
    let mut shapes = Vec::new();
    for tick in 0..90 {
        let input = crate::gfx::RawInput {
            time: Some(tick as f64 / 60.0),
            predicted_dt: 1.0 / 60.0,
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1280.0, 800.0))),
            ..Default::default()
        };
        let output = context.run_pass(input, |context| {
            if tick < 30 {
                draw_action_menu(
                    context,
                    viewport,
                    metrics,
                    &assets,
                    "Menu",
                    &items,
                    None,
                    Pos2::new(400.0, 300.0),
                    0.0,
                    1,
                );
            } else {
                draw_action_menu_closing(context, viewport, metrics, &assets);
            }
        });
        shapes.push(output.shape_count());
        output.drop_without_applying_deltas();
    }
    assert!(shapes[10] > 0);
    assert!(shapes[40] > 0);
    assert_eq!(shapes[89], 0);
}

#[test]
fn long_action_menu_caps_height_and_reaches_last_row() {
    for (width, height, form) in [
        (390, 844, UiFormFactor::Mobile),
        (1920, 1080, UiFormFactor::Tv),
        (1280, 800, UiFormFactor::Desktop),
    ] {
        let viewport = Viewport::new(width, height, form);
        let anchor = Pos2::new(100.0, 100.0);
        let top = action_menu_layout(viewport, 75, anchor, 0.0, 240.0);
        assert!(top.panel.height() < height as f32);
        assert!(top.max_scroll > 0.0);
        let scroll = action_menu_scroll_to(viewport, 75, anchor, 74, 0.0, 240.0);
        let end = action_menu_layout(viewport, 75, anchor, scroll, 240.0);
        assert!(end.body.contains(end.rows[74].center()));
        assert_eq!(action_menu_hit(&end, end.rows[74].center()), Some(74));
        assert_eq!(action_menu_hit(&end, end.rows[0].center()), None);
    }
}

#[test]
fn short_action_menu_does_not_scroll() {
    let viewport = Viewport::new(390, 844, UiFormFactor::Mobile);
    let layout = action_menu_layout(viewport, 4, Pos2::ZERO, 50.0, 240.0);
    assert_eq!(layout.max_scroll, 0.0);
    assert_eq!(action_menu_hit(&layout, layout.rows[0].center()), Some(0));
}

fn action_menu_paints_unaligned_marker(silenced: bool) -> bool {
    let viewport = Viewport::new(1280, 800, UiFormFactor::Desktop);
    let metrics = UiMetrics::for_viewport(viewport);
    let items: Vec<ActionMenuItem> = [
        "Resume",
        "Play manually",
        "Play on my TV",
        "Details",
        "Add to Watchlist",
        "Remove from Continue Watching",
    ]
    .into_iter()
    .map(|label| ActionMenuItem {
        icon: "play",
        label: label.to_owned(),
        app_icon: None,
    })
    .collect();
    let context = crate::gfx::Context::default();
    context.set_fonts(crate::fonts::definitions());
    if silenced {
        context.silence_layout_debug();
    }
    let assets = EmptyHomeAssets;
    let mut found = false;
    for frame in 0..4 {
        let output = context.run_ui(
            crate::gfx::RawInput {
                screen_rect: Some(Rect::from_min_size(
                    Pos2::ZERO,
                    Vec2::new(viewport.width, viewport.height),
                )),
                time: Some(frame as f64 / 60.0),
                ..Default::default()
            },
            |ui| {
                draw_action_menu(
                    ui.ctx(),
                    viewport,
                    metrics,
                    &assets,
                    "",
                    &items,
                    None,
                    Pos2::new(160.0, 520.0),
                    0.0,
                    1,
                );
            },
        );
        found |= output.paints_text("Unaligned");
        output.drop_without_applying_deltas();
    }
    found
}

#[test]
fn menu_rows_do_not_paint_egui_unaligned_markers() {
    assert!(action_menu_paints_unaligned_marker(false));
    assert!(!action_menu_paints_unaligned_marker(true));
}

fn full_detail() -> DetailModel {
    let card = |index: usize| HomeCard {
        id: Some(format!("tt{index}")),
        item_type: Some("movie".into()),
        title: format!("Related {index}"),
        artwork_url: Some(format!("https://img/{index}.jpg")),
        ..Default::default()
    };
    DetailModel {
        id: "series".into(),
        content_type: "series".into(),
        title: "Long Series".into(),
        description: "A show about things.".into(),
        episodes: (1..=3)
            .flat_map(|season| {
                (1..=6).map(move |number| DetailEpisode {
                    id: format!("s{season}e{number}"),
                    season,
                    number,
                    title: format!("Episode {number}"),
                    overview: "Something happens.".into(),
                    thumbnail: Some("https://img/e.jpg".into()),
                    ..Default::default()
                })
            })
            .collect(),
        season_posters: (1..=3)
            .map(|season| (season, format!("https://img/s{season}.jpg")))
            .collect(),
        similar: (0..8).map(card).collect(),
        franchise: (10..16).map(card).collect(),
        trailer_cards: (0..4)
            .map(|index| DetailTrailer {
                title: format!("Trailer {index}"),
                thumbnail: Some("https://img/t.jpg".into()),
                url: format!("https://youtu.be/{index}"),
            })
            .collect(),
        cast: (0..6)
            .map(|index| DetailCastMember {
                name: format!("Actor {index}"),
                role: Some("Role".into()),
                photo: Some("https://img/a.jpg".into()),
            })
            .collect(),
        ..Default::default()
    }
}

#[test]
fn detail_rails_lay_every_card_at_full_size() {
    let viewport = Viewport::new(1280, 800, UiFormFactor::Desktop);
    let detail = full_detail();
    let layout = draw_test_frames(viewport, 6, |context, assets| {
        draw_detail(context, viewport, &detail, assets)
    });
    let rail = |first: usize, count: usize| -> Vec<Rect> {
        (first..first + count)
            .map(|id| {
                layout
                    .focusable
                    .iter()
                    .find(|(key, _)| *key == id as u64)
                    .unwrap()
                    .1
            })
            .collect()
    };
    for (name, first, count) in [
        ("seasons", 1901, 3),
        ("episodes", 2000, 3),
        ("trailers", 1850, 4),
        ("franchise", 1000, 6),
        ("similar", 300, 6),
    ] {
        let cards = rail(first, count);
        let height = cards[0].height();
        assert!(height > 100.0, "{name} cards collapsed to {height}");
        for pair in cards.windows(2) {
            assert!(pair[0].width() > 100.0, "{name} card has no width");
            assert_eq!(pair[0].height(), pair[1].height(), "{name} heights differ");
            assert!(pair[1].min.x > pair[0].max.x, "{name} cards overlap");
        }
    }
    let seasons = rail(1901, 1)[0];
    assert_eq!((seasons.width(), seasons.height()), (116.0, 204.0));
}

#[test]
fn rails_below_the_fold_keep_their_height() {
    let context = crate::gfx::Context::default();
    context.set_fonts(crate::fonts::definitions());
    let mut heights = Vec::new();
    for frame in 0..3 {
        heights.clear();
        let output = context.run_ui(
            crate::gfx::RawInput {
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(400.0, 200.0))),
                time: Some(frame as f64 / 60.0),
                ..Default::default()
            },
            |ui| {
                ui.scope_builder(
                    crate::gfx::UiBuilder::new().max_rect(Rect::from_min_size(
                        Pos2::new(0.0, 180.0),
                        Vec2::new(400.0, 20.0),
                    )),
                    |ui| {
                        let out = crate::components::pan_row(
                            ui,
                            "rail",
                            |area| area,
                            |ui| {
                                ui.allocate_space(Vec2::new(900.0, 150.0));
                            },
                        );
                        heights.push(out.inner_rect.height());
                        let strip = crate::components::pan_strip(ui, "strip", |ui| {
                            ui.allocate_space(Vec2::new(900.0, 90.0));
                        });
                        heights.push(strip.inner_rect.height());
                    },
                );
            },
        );
        output.drop_without_applying_deltas();
    }
    assert_eq!(heights, [150.0, 90.0]);
}

#[test]
fn movie_without_extras_shows_no_empty_sections() {
    let detail = DetailModel::default();
    for section in DetailSection::ALL {
        assert!(!detail.has_content(section), "{section:?}");
    }
    let full = full_detail();
    for section in DetailSection::ALL {
        if section != DetailSection::ShowInfo {
            assert!(full.has_content(section), "{section:?}");
        }
    }
}

#[test]
fn meta_line_keeps_facts_and_ratings_on_one_row() {
    let context = crate::gfx::Context::default();
    context.set_fonts(crate::fonts::definitions());
    let mut assets = EmptyHomeAssets;
    let facts = vec!["2026".to_owned(), "12min".to_owned()];
    let ratings = vec![("IMDb".to_owned(), "8.7".to_owned())];
    let mut height = 0.0;
    for frame in 0..3 {
        let output = context.run_ui(
            crate::gfx::RawInput {
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(600.0, 300.0))),
                time: Some(frame as f64 / 60.0),
                ..Default::default()
            },
            |ui| {
                crate::kit::provide(
                    ui.ctx(),
                    UiMetrics::for_viewport(Viewport::new(1280, 800, UiFormFactor::Desktop)),
                );
                let top = ui.cursor().min.y;
                crate::components::meta_line(
                    ui,
                    &facts,
                    &ratings,
                    14.0,
                    Color32::WHITE,
                    Color32::GRAY,
                    &mut assets,
                );
                height = ui.cursor().min.y - top;
            },
        );
        output.drop_without_applying_deltas();
    }
    assert!(height > 10.0 && height < 30.0, "{height}");
}

fn welcome_fixture() -> WelcomeModel {
    let width = 25;
    WelcomeModel {
        language: "en".to_owned(),
        qr: Some(WelcomeQr {
            modules: (0..width * width).map(|index| index % 3 == 0).collect(),
            width,
        }),
        pair_code: Some(("nuvio.tv/link".to_owned(), "AUTURT".to_owned())),
        ..Default::default()
    }
}

fn profiles_fixture(mode: ProfilesMode) -> ProfilesModel {
    ProfilesModel {
        language: "en".to_owned(),
        mode,
        profiles: ["Ada", "Grace", "Linus", "Margaret", "Dennis"]
            .iter()
            .enumerate()
            .map(|(index, name)| ProfileEntry {
                id: format!("profile-{index}"),
                name: (*name).to_owned(),
                avatar_url: None,
                locked: index == 1,
                uses_primary_addons: false,
                uses_primary_plugins: false,
            })
            .collect(),
        primary_id: Some("profile-0".to_owned()),
        ..Default::default()
    }
}

static SCREENS: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn screens() -> std::sync::MutexGuard<'static, ()> {
    SCREENS
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn painted_bottom(
    viewport: Viewport,
    mut draw: impl FnMut(&crate::gfx::Context, &mut EmptyHomeAssets) -> HomeLayout,
) -> f32 {
    let _screens = screens();
    let context = crate::gfx::Context::default();
    context.set_fonts(crate::fonts::definitions());
    let mut assets = EmptyHomeAssets;
    let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(viewport.width, viewport.height));
    let mut bounds = None;
    for _ in 0..4 {
        let output = context.run_ui(
            crate::gfx::RawInput {
                screen_rect: Some(screen),
                ..Default::default()
            },
            |ui| {
                draw(ui.ctx(), &mut assets);
            },
        );
        bounds = output.text_bounds();
        output.drop_without_applying_deltas();
    }
    bounds.map_or(0.0, |bounds| bounds.max.y)
}

#[test]
fn fixed_full_screen_scenes_stay_inside_the_safe_area() {
    let sizes = [
        (UiFormFactor::Tv, 1920, 1080),
        (UiFormFactor::Tv, 1280, 720),
        (UiFormFactor::Tv, 960, 540),
        (UiFormFactor::Tv, 854, 480),
        (UiFormFactor::Desktop, 1280, 800),
        (UiFormFactor::Mobile, 390, 844),
    ];
    let mut overflowing = Vec::new();
    for (form, width, height) in sizes {
        let viewport = Viewport::new(width, height, form);
        let limit = viewport.height - viewport.safe_bottom - 0.5;
        let mut check = |name: &str, bottom: f32| {
            if bottom > limit {
                overflowing.push(format!(
                    "{name} {form:?} {width}x{height}: {bottom:.0} > {limit:.0}"
                ));
            }
        };
        let mut welcome = welcome_fixture();
        check(
            "welcome",
            painted_bottom(viewport, |context, assets| {
                draw_welcome(context, viewport, &mut welcome, assets)
            }),
        );
        for (name, mode) in [
            ("profiles-select", ProfilesMode::Select),
            ("profiles-form", ProfilesMode::Form),
        ]
        .into_iter()
        .filter(|_| form == UiFormFactor::Tv)
        {
            let mut model = profiles_fixture(mode);
            check(
                name,
                painted_bottom(viewport, |context, assets| {
                    draw_profiles(context, viewport, &mut model, assets)
                }),
            );
        }
    }
    assert!(overflowing.is_empty(), "{}", overflowing.join("\n"));
}

#[test]
fn pairing_steps_stay_on_one_line_when_the_code_arrives_late() {
    let _screens = screens();
    let viewport = Viewport::new(960, 540, UiFormFactor::Tv);
    let context = crate::gfx::Context::default();
    context.set_fonts(crate::fonts::definitions());
    let mut assets = EmptyHomeAssets;
    let mut model = welcome_fixture();
    let code = model.pair_code.take();
    let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(viewport.width, viewport.height));
    let mut rows = None;
    for frame in 0..8 {
        if frame == 3 {
            model.pair_code = code.clone();
        }
        let output = context.run_ui(
            crate::gfx::RawInput {
                screen_rect: Some(screen),
                ..Default::default()
            },
            |ui| {
                draw_welcome(ui.ctx(), viewport, &mut model, &mut assets);
            },
        );
        rows = output.text_rows("phone camera");
        output.drop_without_applying_deltas();
    }
    assert_eq!(rows, Some(1));
}

fn focus_targets_after(
    viewport: Viewport,
    mut draw: impl FnMut(&crate::gfx::Context, &mut EmptyHomeAssets) -> HomeLayout,
) -> Vec<(u64, Rect)> {
    let _screens = screens();
    let context = crate::gfx::Context::default();
    context.set_fonts(crate::fonts::definitions());
    let mut assets = EmptyHomeAssets;
    let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(viewport.width, viewport.height));
    let mut targets = Vec::new();
    for _ in 0..4 {
        crate::begin_focus_frame(&context, None);
        let output = context.run_ui(
            crate::gfx::RawInput {
                screen_rect: Some(screen),
                ..Default::default()
            },
            |ui| {
                draw(ui.ctx(), &mut assets);
            },
        );
        targets = crate::take_focus_targets(&context).0;
        output.drop_without_applying_deltas();
    }
    targets
}

#[test]
fn tv_full_screen_flows_expose_focus_targets_inside_the_screen() {
    for (width, height) in [(1920, 1080), (960, 540), (854, 480)] {
        let viewport = Viewport::new(width, height, UiFormFactor::Tv);
        let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(viewport.width, viewport.height));
        let mut welcome = welcome_fixture();
        let targets = focus_targets_after(viewport, |context, assets| {
            draw_welcome(context, viewport, &mut welcome, assets)
        });
        assert!(
            targets
                .iter()
                .any(|(node, rect)| *node == NODE_WELCOME_OFFLINE && screen.contains_rect(*rect)),
            "welcome {width}x{height}: {targets:?}"
        );
        let mut profiles = profiles_fixture(ProfilesMode::Select);
        let targets = focus_targets_after(viewport, |context, assets| {
            draw_profiles(context, viewport, &mut profiles, assets)
        });
        assert!(
            targets.len() > profiles.profiles.len()
                && targets.iter().all(|(_, rect)| screen.contains_rect(*rect)),
            "profiles {width}x{height}: {targets:?}"
        );
    }
}

#[test]
fn profile_form_avatars_and_settings_register_every_control_for_the_dpad() {
    let viewport = Viewport::new(960, 540, UiFormFactor::Tv);
    for (mode, minimum) in [
        (ProfilesMode::Form, 6),
        (ProfilesMode::Avatars, 2),
        (ProfilesMode::Settings, 2),
    ] {
        let mut model = profiles_fixture(mode);
        let targets = focus_targets_after(viewport, |context, assets| {
            draw_profiles(context, viewport, &mut model, assets)
        });
        let auto = targets
            .iter()
            .filter(|(node, _)| crate::is_auto_node(*node))
            .count();
        assert!(auto >= minimum, "{mode:?}: {targets:?}");
    }
}

fn folder_text_rows(
    viewport: Viewport,
    view_mode: FolderViewMode,
    loading: bool,
    needle: &str,
) -> usize {
    let _screens = screens();
    let context = crate::gfx::Context::default();
    context.set_fonts(crate::fonts::definitions());
    let mut assets = EmptyHomeAssets;
    let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(viewport.width, viewport.height));
    let model = FolderModel {
        language: "en".to_owned(),
        id: "recommended".to_owned(),
        title: "Recommended".to_owned(),
        view_mode,
        tabs: vec![FolderTab {
            category_id: "recommended".to_owned(),
            label: "Recommended".to_owned(),
            loading,
            ..Default::default()
        }],
        ..Default::default()
    };
    let mut rows = 0;
    for _ in 0..4 {
        let output = context.run_ui(
            crate::gfx::RawInput {
                screen_rect: Some(screen),
                ..Default::default()
            },
            |ui| {
                draw_folder(ui.ctx(), viewport, &model, 0, &mut assets, None);
            },
        );
        rows = output.text_rows(needle).unwrap_or(0);
        output.drop_without_applying_deltas();
    }
    rows
}

#[test]
fn empty_collection_folder_shows_only_the_empty_state() {
    for form in [
        UiFormFactor::Tv,
        UiFormFactor::Mobile,
        UiFormFactor::Desktop,
    ] {
        let viewport = Viewport::new(1280, 720, form);
        for mode in [FolderViewMode::Rows, FolderViewMode::TabbedGrid] {
            assert_eq!(
                folder_text_rows(viewport, mode, false, "Continue Watching"),
                0
            );
            assert_eq!(
                folder_text_rows(viewport, mode, true, "Continue Watching"),
                0
            );
            assert_eq!(
                folder_text_rows(viewport, mode, false, "This collection is empty"),
                1
            );
            assert_eq!(
                folder_text_rows(viewport, mode, true, "This collection is empty"),
                0
            );
        }
    }
}
