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
    mut draw: impl FnMut(&egui::Context, &mut EmptyHomeAssets) -> HomeLayout,
) -> HomeLayout {
    let context = egui::Context::default();
    let mut assets = EmptyHomeAssets;
    let mut layout = HomeLayout::default();
    let _ = context.run_ui(
        egui::RawInput {
            screen_rect: Some(Rect::from_min_size(
                Pos2::ZERO,
                Vec2::new(viewport.width, viewport.height),
            )),
            ..Default::default()
        },
        |ui| layout = draw(ui.ctx(), &mut assets),
    );
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
    assert_eq!(model.eyebrow, "Movie · Drama · 2026");
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
        home_collection_card_dimensions(&collection.cards[0]),
        (150.0, 150.0)
    );
    assert_eq!(
        home_collection_card_dimensions(&collection.cards[1]),
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
fn home_scroll_moves_content_but_keeps_navigation_fixed() {
    let mut home = HomeModel::default_empty();
    home.show_hero_section = false;
    home.rows = vec![HomeRow {
        id: Some("popular".to_owned()),
        title: "Popular".to_owned(),
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
    let context = egui::Context::default();
    let mut assets = EmptyHomeAssets;
    let mut rendered_layout = None;
    let _ = context.run_ui(
        egui::RawInput {
            screen_rect: Some(Rect::from_min_size(
                Pos2::ZERO,
                Vec2::new(viewport.width, viewport.height),
            )),
            ..Default::default()
        },
        |ui| {
            rendered_layout = Some(draw_home(ui.ctx(), viewport, &home, &mut assets, None));
        },
    );
    let layout = rendered_layout.expect("Home should render in the egui frame");
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
fn mobile_home_bottom_navigation_uses_five_equal_full_width_slots() {
    let viewport = Viewport::new(390, 844, UiFormFactor::Mobile);
    let metrics = UiMetrics::for_viewport(viewport);
    let navigation = navigation_focus_rects(viewport, metrics);
    let slots = [
        NODE_HOME,
        NODE_LIBRARY,
        NODE_DISCOVER,
        NODE_CALENDAR,
        NODE_PROFILE,
    ]
    .map(|node| {
        navigation
            .iter()
            .find(|(id, _)| *id == node)
            .map(|(_, rect)| *rect)
            .expect("all bottom destinations should have a hit target")
    });
    let width = slots[0].width();

    assert!((slots[0].left() - 8.0).abs() < 0.01);
    assert!((slots[4].right() - (viewport.width - 8.0)).abs() < 0.01);
    assert!(slots.iter().all(|rect| (rect.width() - width).abs() < 0.01));
}

#[test]
fn mobile_library_tabs_remain_one_clipped_horizontal_row() {
    let viewport = Viewport::new(320, 720, UiFormFactor::Mobile);
    let library = LibraryModel::default();
    let layout = draw_test_frame(viewport, |context, assets| {
        draw_library(
            context,
            viewport,
            &library,
            LibraryTab::Watchlist,
            assets,
            None,
        )
    });
    let tabs = layout
        .focusable
        .iter()
        .filter(|(id, _)| {
            (NODE_LIBRARY_TAB_BASE..NODE_LIBRARY_TAB_BASE + LibraryTab::ALL.len() as u64)
                .contains(id)
        })
        .map(|(_, rect)| *rect)
        .collect::<Vec<_>>();
    assert_eq!(tabs.len(), LibraryTab::ALL.len());
    assert!(
        tabs.iter()
            .all(|rect| (rect.top() - tabs[0].top()).abs() < 0.5)
    );
    assert!(tabs.iter().all(|rect| rect.right() <= viewport.width + 1.0));
}

#[test]
fn mobile_home_continue_cards_use_compose_compact_dimensions() {
    let metrics = UiMetrics::for_viewport(Viewport::new(390, 844, UiFormFactor::Mobile));
    let (width, height, _) = home_row_dimensions(metrics, HomeRowKind::Continue);

    assert_eq!(width, 196.0);
    assert!((height - 109.76).abs() < 0.01);
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
fn mobile_home_uses_only_the_compose_primary_hero_action() {
    let mut home = HomeModel::default();
    home.item_id = Some("tt42".to_owned());
    let viewport = Viewport::new(390, 844, UiFormFactor::Mobile);
    let layout = home_layout(viewport, &home);

    let play = layout
        .focusable
        .iter()
        .find(|(id, _)| *id == NODE_PLAY)
        .map(|(_, rect)| *rect)
        .expect("mobile hero should expose its Play action");
    assert_eq!(play.width(), 108.0);
    assert_eq!(play.height(), 42.0);
    assert!((play.center().x - viewport.width * 0.5).abs() < 0.01);
    assert!(!layout.focusable.iter().any(|(id, _)| *id == NODE_MORE_INFO));
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

    assert_eq!(home_row_at_y(viewport, &home, first.center().y), Some(0));
    assert!(home_row_scroll_max(viewport, &home, 0) > 0.0);

    home.row_scroll_offsets = vec![24.0];
    let shifted = card_rect(&home_layout(viewport, &home));
    assert!((first.left() - shifted.left() - 24.0).abs() < 0.01);
}

#[test]
fn library_cards_scroll_without_moving_tabs_or_navigation() {
    let viewport = Viewport::new(390, 844, UiFormFactor::Mobile);
    let mut library = LibraryModel::default();
    library.watchlist = (0..18)
        .map(|index| HomeCard {
            title: format!("Title {index}"),
            ..Default::default()
        })
        .collect();
    assert!(library_scroll_max(viewport, &library, LibraryTab::Watchlist) > 0.0);

    let first = draw_test_frame(viewport, |context, assets| {
        draw_library(
            context,
            viewport,
            &library,
            LibraryTab::Watchlist,
            assets,
            None,
        )
    });
    let scrolled_viewport = viewport.with_scroll_y(52.0);
    let scrolled = draw_test_frame(scrolled_viewport, |context, assets| {
        draw_library(
            context,
            scrolled_viewport,
            &library,
            LibraryTab::Watchlist,
            assets,
            None,
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
        (rect(&first, NODE_CARD_BASE).top() - rect(&scrolled, NODE_CARD_BASE).top() - 52.0).abs()
            < 0.01
    );
    assert_eq!(
        rect(&first, NODE_LIBRARY_TAB_BASE),
        rect(&scrolled, NODE_LIBRARY_TAB_BASE)
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
        draw_calendar(context, viewport, &calendar, assets, None)
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
fn smart_library_lists_come_from_core_plan() {
    let mut model = LibraryModel::default();
    model.apply_core_plan(
        &serde_json::json!({"smartLists": {
            "airing": [{"id":"a","type":"series","name":"Airing title"}],
            "rated": [{"id":"r","type":"movie","name":"Rated title"}],
            "history": [{"id":"h","type":"movie","name":"Watched title"}]
        }}),
        LibraryTab::Watchlist,
    );
    assert_eq!(model.cards(LibraryTab::Airing)[0].title, "Airing title");
    assert_eq!(model.cards(LibraryTab::Rated)[0].title, "Rated title");
    assert_eq!(model.cards(LibraryTab::History)[0].title, "Watched title");
}

#[test]
fn settings_choices_cycle_without_coercing_strings_to_booleans() {
    let model = SettingsModel {
        values: serde_json::json!({"language":"en", "posterHideTitles":false}),
        last_write_error: None,
        active_section: 0,
        ..Default::default()
    };
    let language = settings_row_by_index(0).unwrap();
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
    assert_eq!(defaults.next_value(download), "best");
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
        "exoplayer"
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
            draw_library(
                context,
                viewport,
                &library,
                LibraryTab::Watchlist,
                assets,
                None,
            )
        });
        let mut settings = SettingsModel::default();
        settings.active_section = 1;
        let settings_layout = draw_test_frame(viewport, |context, assets| {
            draw_settings(context, viewport, &settings, assets, None)
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
            SETTINGS_SECTIONS[settings.active_section].rows.len()
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
        }],
        ..Default::default()
    };
    let layout = draw_test_frame(viewport, |context, assets| {
        draw_calendar(context, viewport, &calendar, assets, None)
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
            ..Default::default()
        };
    let layout = draw_test_frame(viewport, |context, assets| {
        draw_detail(context, viewport, &detail, assets, None)
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
        back.top() >= play.top(),
        "wrapped detail action row must not overlap"
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
    let viewport = Viewport::new(1280, 720, UiFormFactor::Desktop);
    let top = draw_test_frame(viewport, |context, assets| {
        draw_discover(context, viewport, &model, assets, None)
    });
    assert!(top.load_more.is_empty());
    let near_end = viewport.with_scroll_y(discover_scroll_max(viewport, &model));
    let bottom = draw_test_frame(near_end, |context, assets| {
        draw_discover(context, near_end, &model, assets, None)
    });
    assert_eq!(bottom.load_more.len(), 1);
    assert_eq!(bottom.load_more[0]["type"], "discoverPageRequested");
    assert_eq!(bottom.load_more[0]["skip"], 80);
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
    assert!(filters[2].right() <= viewport.width);
}

#[test]
fn discover_poster_grid_adapts_columns_to_available_width() {
    let phone_rows = grid_row_count(12, 328.0, 136.0, 12.0);
    let tablet_rows = grid_row_count(12, 760.0, 160.0, 12.0);

    assert_eq!(phone_rows, 6, "phone layout should fit two posters per row");
    assert_eq!(
        tablet_rows, 3,
        "tablet layout should fit four posters per row"
    );
}

#[test]
fn discover_grid_fills_width_and_starts_after_filters() {
    let viewport = Viewport::new(1880, 920, UiFormFactor::Desktop);
    let metrics = UiMetrics::for_viewport(viewport);
    let available = viewport.width - 2.0 * screen_margin(viewport, metrics);
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
        poster_gap > 0.0 && poster_gap <= 8.0,
        "poster gap is {poster_gap}px"
    );
    assert!(
        (rect(NODE_CARD_BASE + columns as u64 - 1).right()
            - (viewport.width - screen_margin(viewport, metrics)))
        .abs()
            <= 2.0
    );
    assert!(rect(NODE_CARD_BASE + columns as u64).top() > rect(NODE_CARD_BASE).top());
}

#[test]
fn discover_results_scroll_below_fixed_filters() {
    let viewport = Viewport::new(1280, 720, UiFormFactor::Desktop);
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
    assert!(discover_scroll_max(viewport, &discover) >= 80.0);
    let first = draw_test_frame(viewport, |context, assets| {
        draw_discover(context, viewport, &discover, assets, None)
    });
    let scrolled_viewport = viewport.with_scroll_y(80.0);
    let scrolled = draw_test_frame(scrolled_viewport, |context, assets| {
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
    let metrics = UiMetrics::for_viewport(viewport);
    let columns = discover_grid_geometry(
        viewport.width - 2.0 * screen_margin(viewport, metrics),
        metrics.poster_card_width,
        metrics.horizontal_spacing,
    )
    .0;
    let second_row = NODE_CARD_BASE + columns as u64;
    let card_delta = rect(&first, second_row).top() - rect(&scrolled, second_row).top();
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
            "trailers": [{"url":"https://example.test/trailer.mp4"}],
            "mdblistRatings": {"imdb": 8.2, "tmdb": 76, "metacritic": 71}
        },
        "library": {
            "completed": [{"id":"tt42"}],
            "dropped": [],
            "liked": [{"id":"tt42"}]
        },
        "settings": {
            "values": {"animationsEnabled": false},
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
    assert!(!settings.bool_value("animationsEnabled"));
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
    assert_eq!(desktop.poster_card_height, 240.0);
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
    assert_eq!(desktop.home_hero_synopsis_size_desktop, 20.0);
    assert_eq!(desktop.settings_nav_width, 268.0);
    assert_eq!(desktop.settings_content_offset, 300.0);
    assert_eq!(desktop.settings_content_max_width_desktop, 2400.0);
    assert_eq!(desktop.settings_row_height, 44.0);
    assert_eq!(desktop.settings_row_label_size_desktop, 18.0);
    assert_eq!(mobile.settings_nav_width, 196.0);
    assert_eq!(mobile.settings_row_height, 32.0);
}

#[test]
fn desktop_settings_uses_sidebar_title_without_redundant_back_header() {
    let viewport = Viewport::new(1920, 1080, UiFormFactor::Desktop);
    let context = egui::Context::default();
    let settings = SettingsModel::default();
    let assets = EmptyHomeAssets;
    let _ = context.run_ui(
        egui::RawInput {
            screen_rect: Some(Rect::from_min_size(
                Pos2::ZERO,
                Vec2::new(viewport.width, viewport.height),
            )),
            ..Default::default()
        },
        |context| {
            let layout = draw_settings(context, viewport, &settings, &assets, None);
            assert!(
                layout
                    .focusable
                    .iter()
                    .all(|(node, _)| *node != NODE_SETTINGS_BACK),
                "desktop Settings should not show a redundant Back button"
            );
            let first_category = layout
                .focusable
                .iter()
                .find(|(node, _)| *node == NODE_SETTINGS_SECTION_BASE)
                .map(|(_, rect)| *rect)
                .unwrap();
            assert!(
                first_category.top() < 200.0,
                "desktop Settings should start near the top of the window"
            );
        },
    );
}

#[test]
fn native_appearance_exposes_web_preferences_and_uses_white_default_accent() {
    let appearance = SETTINGS_SECTIONS
        .iter()
        .find(|section| section.title == "Appearance")
        .unwrap();
    let keys: Vec<_> = appearance.rows.iter().map(|row| row.key).collect();
    for key in [
        "accentColorArgb",
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
    assert_eq!(
        super::settings::parse_settings_color("#FFFFFF"),
        Some(Color32::WHITE)
    );
    assert_eq!(
        super::settings::parse_settings_color("#E50914"),
        Some(Color32::from_rgb(229, 9, 20))
    );
    assert_eq!(super::settings::parse_settings_color("not-a-color"), None);
}

#[test]
fn native_appearance_groups_are_separate_cards() {
    let viewport = Viewport::new(1920, 1080, UiFormFactor::Desktop);
    let metrics = UiMetrics::for_viewport(viewport);
    let section = SETTINGS_SECTIONS
        .iter()
        .find(|section| section.title == "Appearance")
        .unwrap();
    let height = super::settings::settings_card_height(
        viewport,
        metrics,
        section,
        &SettingsModel::default(),
    );
    let content = Rect::from_min_size(Pos2::ZERO, Vec2::new(1200.0, height));
    let mut previous_bottom = 0.0;
    for start in [0, 4, 8, 16, 24] {
        let (card, _) = super::settings::appearance_group_layout(start, content, metrics).unwrap();
        assert!(
            card.top() > previous_bottom,
            "appearance groups must have visible gaps"
        );
        previous_bottom = card.bottom();
    }
    let (last_card, _) = super::settings::appearance_group_layout(34, content, metrics).unwrap();
    assert!(last_card.bottom() <= content.bottom());
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
    let context = egui::Context::default();
    let mut heights = Vec::new();
    let _ = context.run_ui(
        egui::RawInput {
            screen_rect: Some(Rect::from_min_size(
                Pos2::ZERO,
                Vec2::new(viewport.width, viewport.height),
            )),
            ..Default::default()
        },
        |ui| {
            let options = vec![("recent".to_owned(), "Recently updated".to_owned())];
            let (select, _) =
                components::dropdown(ui, "dropdown-test", "recent", &options, 240.0, metrics);
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
    );

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
        options: &[],
    };
    let seek_thumbnails = SettingsRow {
        label: "Seek thumbnail preview",
        key: "seekThumbnailEnabled",
        options: &[],
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
