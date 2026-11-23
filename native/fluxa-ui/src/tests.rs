use super::*;
use serde_json::json;

#[test]
fn artwork_decode_size_tracks_card_pixels_and_display_density() {
    let logical_card = Vec2::new(200.0, 100.0);

    assert_eq!(artwork_target_size(logical_card, 1.0), [200, 100]);
    assert_eq!(artwork_target_size(logical_card, 2.5), [500, 250]);
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
fn responsive_metrics_come_from_the_metrics_sheet() {
    crate::metrics::enter(Viewport::new(1920, 1080, UiFormFactor::Desktop));
    assert_eq!(crate::metrics::theme().accent, Color32::WHITE);
    assert_eq!(crate::metric("page-padding"), 24.0);
    assert_eq!(crate::metric("horizontal-card-width"), 300.0);
    assert_eq!(crate::metrics::theme().poster_width, 200.0);
    assert_eq!(crate::metrics::theme().poster_height, 300.0);
    assert_eq!(crate::metric("poster-height-ratio"), 1.5);
    assert_eq!(crate::metric("library-search-reserved-width"), 150.0);
    assert_eq!(crate::metric("focus-ring-width"), 2.5);
    assert_eq!(crate::metric("settings-nav-width"), 268.0);
    assert_eq!(crate::metric("settings-content-offset"), 300.0);
    assert_eq!(crate::metric("settings-row-height"), 52.0);

    crate::metrics::enter(Viewport::new(1100, 800, UiFormFactor::Desktop));
    assert!((140.0..200.0).contains(&crate::metrics::theme().poster_width));

    crate::metrics::enter(Viewport::new(390, 844, UiFormFactor::Mobile));
    assert_eq!(
        crate::metrics::theme().poster_height,
        crate::metrics::theme().poster_width * 1.5
    );
    assert_eq!(crate::metric("page-padding"), 16.0);
    assert_eq!(crate::metric("horizontal-card-width"), 244.0);
    assert_eq!(crate::metric("settings-nav-width"), 196.0);
    assert_eq!(crate::metric("settings-row-height"), 46.0);

    crate::metrics::enter(Viewport::new(1920, 1080, UiFormFactor::Tv));
    assert_eq!(crate::metric("horizontal-card-width"), 356.0);
    assert_eq!(crate::metric("calendar-panel-width-tv"), 440.0);
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
fn rows_follow_the_form_factor() {
    let applies = |key, form_factor, width, height| {
        super::settings::row_applies(key, Viewport::new(width, height, form_factor))
    };
    assert!(applies(
        "posterHoverPreview",
        UiFormFactor::Desktop,
        1920,
        1080
    ));
    assert!(!applies("posterHoverPreview", UiFormFactor::Tv, 1920, 1080));
    assert!(!applies(
        "posterHoverPreview",
        UiFormFactor::Mobile,
        390,
        844
    ));
    assert!(applies("expandedPosters", UiFormFactor::Tv, 1920, 1080));
    assert!(!applies("expandedPosters", UiFormFactor::Mobile, 390, 844));
    assert!(applies("playerGestures", UiFormFactor::Mobile, 390, 844));
    assert!(!applies(
        "playerGestures",
        UiFormFactor::Desktop,
        1920,
        1080
    ));
    assert!(!applies("playerGestures", UiFormFactor::Tv, 1920, 1080));
    assert!(applies("subtitleSize", UiFormFactor::Tv, 1920, 1080));
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
                kind: "Trailer".into(),
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
fn trailer_dropdown_filters_by_kind_and_defaults_to_trailers() {
    let video = |kind: &str| DetailTrailer {
        title: kind.to_owned(),
        kind: kind.to_owned(),
        thumbnail: None,
        url: format!("https://example.com/{kind}"),
    };
    let mut detail = DetailModel {
        trailer_cards: vec![
            video("Trailer"),
            video("Clip"),
            video("Trailer"),
            video("Teaser"),
        ],
        ..Default::default()
    };
    assert_eq!(detail.trailer_kinds(), ["Trailer", "Clip", "Teaser"]);
    assert_eq!(detail.shown_trailers().len(), 2);
    detail.selected_trailer_kind = Some("Clip".into());
    assert_eq!(detail.shown_trailers().len(), 1);
    detail.selected_trailer_kind = Some("Gone".into());
    assert_eq!(detail.current_trailer_kind().as_deref(), Some("Trailer"));
}

#[test]
fn skin_strings_win_over_bundled_ones_and_clear_again() {
    let mut table = HashMap::new();
    table.insert("common.play".to_owned(), "Watch now".to_owned());
    crate::set_skin_strings([("en".to_owned(), table)]);
    assert_eq!(crate::localized("common.play", "en-US"), "Watch now");
    crate::set_skin_strings([]);
    assert_eq!(crate::localized("common.play", "en-US"), "Play");
}

#[test]
fn css_root_sets_named_motion_durations() {
    let (_, motion, curve) = crate::stylesheet::Sheet::parse_with_motion(
        ":root { motion-fast: 90ms; motion-slow: 0.5s; motion-curve: 0.4 0 0.2 1; }",
    )
    .unwrap();
    assert_eq!(motion, [Some(0.09), None, Some(0.5)]);
    assert_eq!(curve, Some([0.4, 0.0, 0.2, 1.0]));
}

#[test]
fn season_opens_on_the_one_being_resumed() {
    let mut detail = full_detail();
    assert_eq!(detail.current_season(), 1);
    detail.resume = Some(HomeCard {
        raw: serde_json::json!({"lastEpisodeSeason": 3, "lastEpisodeNumber": 2}).into(),
        ..Default::default()
    });
    assert_eq!(detail.current_season(), 3);
    detail.selected_season = Some(2);
    assert_eq!(detail.current_season(), 2);
}
