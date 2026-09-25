use super::*;

pub(super) fn shared_home_model(
    home: &Value,
    billboard: &Value,
    hero_slides: &Value,
) -> SharedHomeModel {
    let snapshot = json!({
        "home": {
            "categories": home.get("categories").cloned().unwrap_or_else(|| json!([])),
            "continueWatching": home
                .get("continueWatching")
                .cloned()
                .unwrap_or_else(|| json!([])),
            "billboard": billboard,
            "heroSlides": hero_slides,
        }
    });
    home_model_from_core_snapshot(&snapshot, UiFormFactorJson::Desktop)
}

fn shared_desktop_viewport(ui: &egui::Ui) -> Viewport {
    Viewport::new(
        ui.ctx().content_rect().width() as u32,
        ui.ctx().content_rect().height() as u32,
        UiFormFactor::Desktop,
    )
}

fn discover_extra_filters(query: &str, extra_name: &str, extra_value: &str) -> Value {
    let mut extra = serde_json::Map::new();
    if !query.trim().is_empty() {
        extra.insert("search".to_owned(), Value::String(query.to_owned()));
    }
    if !extra_name.is_empty() && !extra_value.is_empty() && extra_name != "search" {
        extra.insert(extra_name.to_owned(), Value::String(extra_value.to_owned()));
    }
    Value::Object(extra)
}

pub(super) fn draw_shared_settings_screen(
    ui: &mut egui::Ui,
    runtime: &mut FluxaRuntime,
    artwork: &mut ArtworkRegistry,
    background_texture: TextureId,
    active_page: &mut String,
    screen_state: &mut NativeScreenState,
    pending_effects: &mut Vec<Value>,
    status: &mut String,
) {
    let mut settings: SettingsModel = settings_model_from_core_snapshot(runtime.snapshot());
    settings.active_section = screen_state.settings_section;
    settings.addon_url = screen_state.settings_addon_url.clone();
    settings.plugin_url = screen_state.settings_plugin_url.clone();
    let layout = draw_settings(
        ui.ctx(),
        shared_desktop_viewport(ui),
        &settings,
        &DesktopHomeAssets {
            background: background_texture,
            artwork,
        },
        None,
    );
    if let Some(value) = layout.text_input.as_ref() {
        match layout.text_input_node {
            Some(fluxa_ui::NODE_SETTINGS_ADDON_URL) => {
                screen_state.settings_addon_url = value.clone()
            }
            Some(fluxa_ui::NODE_SETTINGS_PLUGIN_URL) => {
                screen_state.settings_plugin_url = value.clone()
            }
            _ => {}
        }
    }
    if let Some((key, value)) = layout.setting_change.as_ref() {
        if let Ok(update) = runtime.dispatch(json!({
            "type": "settingsChanged",
            "key": key,
            "value": value,
        })) {
            pending_effects.extend(update.effects);
            *status = format!("Updated {key}");
        }
        if key == "integrationLibrarySource" || key == "continueWatchingSource" {
            if let Ok(update) = runtime.dispatch(json!({"type": "libraryHydrateRequested"})) {
                pending_effects.extend(update.effects);
            }
        }
    }
    if let Some(node) = layout.activated {
        if shared_desktop_navigation(node, active_page, runtime, pending_effects) {
            return;
        }
        if node == fluxa_ui::NODE_SETTINGS_BACK {
            if let Ok(update) = runtime.dispatch(json!({
                "type": "navigationRequested",
                "route": "home",
                "params": null,
            })) {
                pending_effects.extend(update.effects);
            }
            *active_page = "Home".to_owned();
            *status = "Home opened".to_owned();
        } else if (fluxa_ui::NODE_SETTINGS_SECTION_BASE
            ..fluxa_ui::NODE_SETTINGS_SECTION_BASE + fluxa_ui::SETTINGS_SECTIONS.len() as u64)
            .contains(&node)
        {
            screen_state.settings_section = (node - fluxa_ui::NODE_SETTINGS_SECTION_BASE) as usize;
        } else if node == fluxa_ui::NODE_SETTINGS_ADDON_INSTALL
            && !screen_state.settings_addon_url.trim().is_empty()
        {
            if let Ok(update) = runtime.dispatch(json!({"type":"addonInstallRequested", "transportUrl":screen_state.settings_addon_url.trim(), "forceRefresh":false})) {
                pending_effects.extend(update.effects);
            }
            screen_state.settings_addon_url.clear();
        } else if node == fluxa_ui::NODE_SETTINGS_ADDON_REFRESH {
            let profile = runtime
                .snapshot()
                .pointer("/profile/active")
                .cloned()
                .unwrap_or(Value::Null);
            if let Ok(update) = runtime.dispatch(
                json!({"type":"addonsRefreshRequested", "profile":profile, "forceRefresh":true}),
            ) {
                pending_effects.extend(update.effects);
            }
        } else if node == fluxa_ui::NODE_SETTINGS_PLUGIN_INSTALL
            && !screen_state.settings_plugin_url.trim().is_empty()
        {
            if let Ok(update) = runtime.dispatch(json!({"type":"pluginRepositoryAddRequested", "manifestUrl":screen_state.settings_plugin_url.trim()})) {
                pending_effects.extend(update.effects);
            }
            screen_state.settings_plugin_url.clear();
        } else if (fluxa_ui::NODE_SETTINGS_PLUGIN_REPOSITORY_BASE
            ..fluxa_ui::NODE_SETTINGS_PLUGIN_REPOSITORY_BASE + 4)
            .contains(&node)
        {
            let index = (node - fluxa_ui::NODE_SETTINGS_PLUGIN_REPOSITORY_BASE) as usize;
            if let Some(url) = settings
                .plugins
                .get("repositories")
                .and_then(Value::as_array)
                .and_then(|items| items.get(index))
                .and_then(|repo| repo.get("manifestUrl"))
                .and_then(Value::as_str)
            {
                if let Ok(update) = runtime
                    .dispatch(json!({"type":"pluginRepositoryRemoveRequested", "manifestUrl":url}))
                {
                    pending_effects.extend(update.effects);
                }
            }
        } else if (fluxa_ui::NODE_SETTINGS_PLUGIN_REFRESH_BASE
            ..fluxa_ui::NODE_SETTINGS_PLUGIN_REFRESH_BASE + 4)
            .contains(&node)
        {
            let index = (node - fluxa_ui::NODE_SETTINGS_PLUGIN_REFRESH_BASE) as usize;
            if let Some(url) = settings
                .plugins
                .get("repositories")
                .and_then(Value::as_array)
                .and_then(|items| items.get(index))
                .and_then(|repo| repo.get("manifestUrl"))
                .and_then(Value::as_str)
            {
                if let Ok(update) = runtime
                    .dispatch(json!({"type":"pluginRepositoryAddRequested", "manifestUrl":url}))
                {
                    pending_effects.extend(update.effects);
                }
            }
        } else if (fluxa_ui::NODE_SETTINGS_PLUGIN_SCRAPER_BASE
            ..fluxa_ui::NODE_SETTINGS_PLUGIN_SCRAPER_BASE + 4)
            .contains(&node)
        {
            let index = (node - fluxa_ui::NODE_SETTINGS_PLUGIN_SCRAPER_BASE) as usize;
            if let Some(scraper) = settings
                .plugins
                .get("scrapers")
                .and_then(Value::as_array)
                .and_then(|items| items.get(index))
            {
                if let Some(id) = scraper.get("id").and_then(Value::as_str) {
                    let enabled = !scraper
                        .get("enabled")
                        .and_then(Value::as_bool)
                        .unwrap_or(false);
                    if let Ok(update) = runtime.dispatch(
                        json!({"type":"pluginScraperToggled", "scraperId":id, "enabled":enabled}),
                    ) {
                        pending_effects.extend(update.effects);
                    }
                }
            }
        } else if (fluxa_ui::NODE_SETTINGS_ROW_BASE
            ..fluxa_ui::NODE_SETTINGS_ROW_BASE
                + fluxa_ui::SETTINGS_SECTIONS
                    .iter()
                    .map(|section| section.rows.len())
                    .sum::<usize>() as u64)
            .contains(&node)
        {
            let index = (node - fluxa_ui::NODE_SETTINGS_ROW_BASE) as usize;
            if let Some(setting) = fluxa_ui::settings_row_by_index(index) {
                let value = settings.next_value_for(setting, UiFormFactor::Desktop);
                if let Ok(update) = runtime.dispatch(json!({
                    "type": "settingsChanged",
                    "key": setting.key,
                    "value": value,
                })) {
                    pending_effects.extend(update.effects);
                }
                *status = format!("Updated {}", setting.label);
            }
        }
    }
}

pub(super) fn draw_shared_detail_screen(
    ui: &mut egui::Ui,
    runtime: &mut FluxaRuntime,
    artwork: &mut ArtworkRegistry,
    background_texture: TextureId,
    active_page: &mut String,
    pending_effects: &mut Vec<Value>,
    status: &mut String,
) {
    let detail: DetailModel = detail_model_from_core_snapshot(runtime.snapshot());
    let mut assets = DesktopHomeAssets {
        background: background_texture,
        artwork,
    };
    let layout = draw_detail(
        ui.ctx(),
        shared_desktop_viewport(ui),
        &detail,
        &mut assets,
        None,
    );
    if let Some(node) = layout.activated {
        if shared_desktop_navigation(node, active_page, runtime, pending_effects) {
            return;
        }
        match node {
            fluxa_ui::NODE_DETAIL_BACK => {
                if let Ok(update) = runtime.dispatch(json!({
                    "type": "navigationRequested",
                    "route": "home",
                    "params": null,
                })) {
                    pending_effects.extend(update.effects);
                }
                *active_page = "Home".to_owned();
                *status = "Home opened".to_owned();
            }
            fluxa_ui::NODE_DETAIL_PLAY => {
                if detail.item.is_object() {
                    if let Ok(update) = runtime.dispatch(json!({
                        "type": "directPlaybackRequested",
                        "meta": detail.item,
                        "language": "en",
                        "profile": null,
                    })) {
                        pending_effects.extend(update.effects);
                    }
                }
            }
            fluxa_ui::NODE_DETAIL_WATCHLIST => {
                if let Ok(update) = runtime.dispatch(json!({
                    "type": "toggleWatchlistRequested",
                    "item": detail.item,
                    "profile": null,
                })) {
                    pending_effects.extend(update.effects);
                }
                *status = "Watchlist updated".to_owned();
            }
            fluxa_ui::NODE_DETAIL_COMPLETED | fluxa_ui::NODE_DETAIL_DROPPED => {
                let list = if node == fluxa_ui::NODE_DETAIL_COMPLETED {
                    "completed"
                } else {
                    "dropped"
                };
                if let Ok(update) = runtime.dispatch(json!({
                    "type": "toggleLibraryStatusRequested",
                    "list": list,
                    "item": detail.item,
                    "profile": null,
                })) {
                    pending_effects.extend(update.effects);
                }
                *status = format!("{} status updated", list);
            }
            fluxa_ui::NODE_DETAIL_FAVORITE => {
                if let Ok(update) = runtime.dispatch(json!({
                    "type": "setFeedbackRequested",
                    "id": detail.id,
                    "value": !detail.favorite,
                    "meta": detail.item,
                })) {
                    pending_effects.extend(update.effects);
                }
                *status = "Favorite updated".to_owned();
            }
            node if node >= fluxa_ui::NODE_DETAIL_SIMILAR_BASE => {
                if let Some(card) = detail
                    .similar
                    .get((node - fluxa_ui::NODE_DETAIL_SIMILAR_BASE) as usize)
                {
                    if let (Some(id), Some(content_type)) =
                        (card.id.clone(), card.item_type.clone())
                    {
                        dispatch_shared_detail_load(
                            runtime,
                            pending_effects,
                            active_page,
                            id,
                            content_type,
                        );
                    }
                }
            }
            _ => {}
        }
    }
}

fn shared_desktop_navigation(
    node: u64,
    active_page: &mut String,
    runtime: &mut FluxaRuntime,
    pending_effects: &mut Vec<Value>,
) -> bool {
    let (page, route) = match node {
        fluxa_ui::NODE_HOME => ("Home", "home"),
        fluxa_ui::NODE_LIBRARY => ("Library", "library"),
        fluxa_ui::NODE_DISCOVER => ("Discover", "discover"),
        fluxa_ui::NODE_CALENDAR => ("Calendar", "calendar"),
        fluxa_ui::NODE_PROFILE => ("Settings", "settings"),
        _ => return false,
    };
    enter_shared_desktop_page(page, route, active_page, runtime, pending_effects);
    true
}

pub(super) fn enter_shared_desktop_page(
    page: &str,
    route: &str,
    active_page: &mut String,
    runtime: &mut FluxaRuntime,
    pending_effects: &mut Vec<Value>,
) {
    *active_page = page.to_owned();
    if let Ok(update) = runtime.dispatch(json!({
        "type": "navigationRequested",
        "route": route,
        "params": null,
    })) {
        pending_effects.extend(update.effects);
    }

    let snapshot = runtime.snapshot();
    let language = snapshot
        .pointer("/profile/active/language")
        .and_then(Value::as_str)
        .unwrap_or("en");
    match route {
        "discover" => {
            let content_type = snapshot
                .pointer("/discover/contentType")
                .and_then(Value::as_str)
                .filter(|value| !value.trim().is_empty())
                .unwrap_or("movie");
            if let Ok(update) = runtime.dispatch(json!({
                "type": "discoverRequested",
                "loadCatalogFilters": true,
                "contentType": content_type,
                "filters": {"catalogKey": null, "extra": {}},
                "language": language,
            })) {
                pending_effects.extend(update.effects);
            }
        }
        "calendar" => {
            let (year, month) = super::current_year_month();
            if let Ok(update) = runtime.dispatch(json!({
                "type": "calendarMonthRequested",
                "year": year,
                "month": month,
            })) {
                pending_effects.extend(update.effects);
            }
        }
        _ => {}
    }
}

fn dispatch_shared_detail_load(
    runtime: &mut FluxaRuntime,
    pending_effects: &mut Vec<Value>,
    active_page: &mut String,
    id: String,
    content_type: String,
) {
    *active_page = "Detail".to_owned();
    if let Ok(update) = runtime.dispatch(json!({
        "type": "detailLoadRequested",
        "id": id.clone(),
        "contentType": content_type.clone(),
        "language": "en",
        "profile": null,
    })) {
        pending_effects.extend(update.effects);
    }
    if let Ok(update) = runtime.dispatch(json!({
        "type": "detailSecondaryRequested",
        "id": id,
        "contentType": content_type,
        "language": "en",
        "profile": null,
        "similarTitlesSource": "auto",
    })) {
        pending_effects.extend(update.effects);
    }
}

pub(super) fn draw_shared_library_screen(
    ui: &mut egui::Ui,
    runtime: &mut FluxaRuntime,
    artwork: &mut ArtworkRegistry,
    background_texture: TextureId,
    active_page: &mut String,
    screen_state: &mut NativeScreenState,
    pending_effects: &mut Vec<Value>,
    status: &mut String,
) {
    let tab = match screen_state.library_tab.as_str() {
        "Watching" => fluxa_ui::LibraryTab::Watching,
        "Completed" => fluxa_ui::LibraryTab::Completed,
        "Dropped" => fluxa_ui::LibraryTab::Dropped,
        "Favorites" | "Liked" => fluxa_ui::LibraryTab::Liked,
        "Airing" => fluxa_ui::LibraryTab::Airing,
        "Rated" => fluxa_ui::LibraryTab::Rated,
        "History" => fluxa_ui::LibraryTab::History,
        _ => fluxa_ui::LibraryTab::Watchlist,
    };
    let cache_key = (
        runtime.revision(),
        tab.core_tab_key().to_owned(),
        screen_state.library_query.clone(),
        screen_state.library_sort.clone(),
    );
    let model = if let Some((key, model)) = &screen_state.library_model_cache {
        if *key == cache_key {
            model.clone()
        } else {
            let mut model = fluxa_ui::library_model_from_core_snapshot(runtime.snapshot());
            model.query = screen_state.library_query.clone();
            model.sort_by = screen_state.library_sort.clone();
            if let Some(plan) =
                runtime.library_view_plan(tab.core_tab_key(), &model.query, &model.sort_by)
            {
                model.apply_core_plan(&plan, tab);
            }
            screen_state.library_model_cache = Some((cache_key, model.clone()));
            model
        }
    } else {
        let mut model = fluxa_ui::library_model_from_core_snapshot(runtime.snapshot());
        model.query = screen_state.library_query.clone();
        model.sort_by = screen_state.library_sort.clone();
        if let Some(plan) =
            runtime.library_view_plan(tab.core_tab_key(), &model.query, &model.sort_by)
        {
            model.apply_core_plan(&plan, tab);
        }
        screen_state.library_model_cache = Some((cache_key, model.clone()));
        model
    };
    let viewport = shared_desktop_viewport(ui);
    let mut assets = DesktopHomeAssets {
        background: background_texture,
        artwork,
    };
    let layout = fluxa_ui::draw_library(ui.ctx(), viewport, &model, tab, &mut assets, None);
    if let Some(query) = &layout.text_input {
        screen_state.library_query = query.clone();
    }
    if let Some(node) = layout.activated {
        if shared_desktop_navigation(node, active_page, runtime, pending_effects) {
            if node == fluxa_ui::NODE_LIBRARY {
                pending_effects.extend(
                    runtime
                        .dispatch(json!({"type":"libraryHydrateRequested", "profileId":null}))
                        .map(|update| update.effects)
                        .unwrap_or_default(),
                );
            }
            return;
        }
        if (fluxa_ui::NODE_LIBRARY_TAB_BASE
            ..fluxa_ui::NODE_LIBRARY_TAB_BASE + fluxa_ui::LibraryTab::ALL.len() as u64)
            .contains(&node)
        {
            if let Some(tab) =
                fluxa_ui::LibraryTab::ALL.get((node - fluxa_ui::NODE_LIBRARY_TAB_BASE) as usize)
            {
                screen_state.library_tab = tab.label().to_owned();
            }
        } else if node == fluxa_ui::NODE_LIBRARY_SORT {
            screen_state.library_sort = match screen_state.library_sort.as_str() {
                "recent" => "title",
                "title" => "rating",
                _ => "recent",
            }
            .to_owned();
        } else if node >= fluxa_ui::NODE_CARD_BASE {
            if let Some(card) = model
                .cards(tab)
                .get((node - fluxa_ui::NODE_CARD_BASE) as usize)
            {
                if let (Some(id), Some(content_type)) = (card.id.clone(), card.item_type.clone()) {
                    dispatch_shared_detail_load(
                        runtime,
                        pending_effects,
                        active_page,
                        id,
                        content_type,
                    );
                }
            }
        }
    }
    if let Some((key, value)) = layout.filter_change {
        if key == "librarySort" {
            screen_state.library_sort = value;
            screen_state.library_model_cache = None;
        }
    }
    if let Some((key, value)) = layout.setting_change {
        if key == "integrationLibrarySource" {
            if let Ok(update) = runtime.dispatch(json!({
                "type": "settingsChanged",
                "key": key,
                "value": value,
            })) {
                pending_effects.extend(update.effects);
            }
            if let Ok(update) =
                runtime.dispatch(json!({"type":"libraryHydrateRequested", "profileId":null}))
            {
                pending_effects.extend(update.effects);
            }
            screen_state.library_model_cache = None;
        }
    }
    let _ = status;
}

pub(super) fn draw_shared_discover_screen(
    ui: &mut egui::Ui,
    runtime: &mut FluxaRuntime,
    artwork: &mut ArtworkRegistry,
    background_texture: TextureId,
    active_page: &mut String,
    screen_state: &mut NativeScreenState,
    pending_effects: &mut Vec<Value>,
    status: &mut String,
) {
    let model_started = std::time::Instant::now();
    let cache_key = (
        runtime.revision(),
        screen_state.discover_content_type.clone(),
        screen_state.discover_catalog_key.clone(),
        screen_state.discover_extra_value.clone(),
    );
    let cached = screen_state.discover_model_cache.take();
    let mut model = match cached {
        Some((previous_key, model))
            if previous_key.1 == cache_key.1
                && previous_key.2 == cache_key.2
                && previous_key.3 == cache_key.3 =>
        { model }
        _ => fluxa_ui::discover_model_from_core_snapshot(runtime.snapshot()),
    };
    let snapshot = runtime.snapshot();
    if screen_state
        .discover_projection_pending
        .is_some_and(|(generation, _)| generation != model.generation)
    {
        screen_state.discover_projection_pending = None;
    }
    while let Ok((generation, start, cards)) = screen_state.discover_projection_receiver.try_recv() {
        if screen_state
            .discover_projection_pending
            .is_some_and(|(pending_generation, _)| pending_generation == generation)
        {
            screen_state.discover_projection_pending = None;
        }
        if generation == model.generation && start == model.results.len() {
            model.results.extend(cards);
        }
    }
    if !fluxa_ui::refresh_discover_model_from_core_snapshot(snapshot, &mut model) {
        model = fluxa_ui::discover_model_from_core_snapshot(snapshot);
        screen_state.discover_projection_pending = None;
    }
    if let Some(items) = snapshot
        .pointer("/discover/results")
        .and_then(Value::as_array)
        && model.results.len() < items.len()
        && screen_state.discover_projection_pending.is_none()
    {
        let start = model.results.len();
        let generation = model.generation;
        let batch = items.iter().skip(start).cloned().collect::<Vec<_>>();
        screen_state.discover_projection_pending = Some((generation, items.len()));
        let sender = screen_state.discover_projection_sender.clone();
        let repaint = ui.ctx().clone();
        std::thread::spawn(move || {
            let started = std::time::Instant::now();
            let count = batch.len();
            let cards = fluxa_ui::discover_cards_from_core_values(batch);
            eprintln!(
                "[fluxa-native] discover projection: {count} cards prepared off-thread in {:.2}ms",
                started.elapsed().as_secs_f64() * 1_000.0,
            );
            let _ = sender.send((generation, start, cards));
            repaint.request_repaint();
        });
    }
    model.content_type = screen_state.discover_content_type.clone();
    if screen_state.discover_catalog_key.is_empty()
        && let Some(catalog) = model
            .catalogs
            .iter()
            .find(|catalog| catalog.content_type == screen_state.discover_content_type)
    {
        screen_state.discover_catalog_key = catalog.key.clone();
        if let Ok(update) = runtime.dispatch(json!({
            "type":"discoverRequested",
            "contentType":screen_state.discover_content_type,
            "filters":{
                "catalogKey":screen_state.discover_catalog_key,
                "extra":discover_extra_filters(
                    &model.query,
                    &model.selected_extra_name,
                    &screen_state.discover_extra_value,
                )
            },
            "language":model.language
        })) {
            pending_effects.extend(update.effects);
        }
    }
    model.selected_catalog_key = screen_state.discover_catalog_key.clone();
    model.selected_extra_name = model
        .catalogs
        .iter()
        .find(|catalog| {
            catalog.key == screen_state.discover_catalog_key
                && catalog.content_type == screen_state.discover_content_type
        })
        .and_then(|catalog| {
            catalog
                .extras
                .iter()
                .find(|(name, _)| name == &model.selected_extra_name)
                .or_else(|| catalog.extras.first())
        })
        .map(|(name, _)| name.clone())
        .unwrap_or_default();
    if screen_state.discover_extra_value.is_empty() {
        screen_state.discover_extra_value = model.selected_extra_value.clone();
    }
    model.selected_extra_value = screen_state.discover_extra_value.clone();
    let model_elapsed = model_started.elapsed();
    let viewport = shared_desktop_viewport(ui);
    let mut assets = DesktopHomeAssets {
        background: background_texture,
        artwork,
    };
    let draw_started = std::time::Instant::now();
    let layout = fluxa_ui::draw_discover(ui.ctx(), viewport, &model, &mut assets, None);
    let draw_elapsed = draw_started.elapsed();
    if model_elapsed.as_millis() >= 8 || draw_elapsed.as_millis() >= 8 {
        eprintln!(
            "[fluxa-native] discover breakdown model={:.1}ms draw={:.1}ms items={} max={:.0}",
            model_elapsed.as_secs_f64() * 1_000.0,
            draw_elapsed.as_secs_f64() * 1_000.0,
            model.results.len(),
            fluxa_ui::discover_scroll_max(viewport, &model),
        );
    }
    for request in &layout.load_more {
        let dispatch_started = std::time::Instant::now();
        if let Ok(update) = runtime.dispatch_discover_page(request.clone()) {
            pending_effects.extend(update.effects);
        }
        let dispatch_ms = dispatch_started.elapsed().as_secs_f64() * 1_000.0;
        if dispatch_ms >= 4.0 {
            eprintln!(
                "[fluxa-native] discover page dispatch on UI thread: {:.1}ms (skip={})",
                dispatch_ms,
                request.pointer("/skip").and_then(Value::as_i64).unwrap_or(-1),
            );
        }
    }
    // Rendering is shared with Android; translate the shared search-field event
    // into the same Core discover request rather than dropping it on desktop.
    if layout.text_input_node == Some(fluxa_ui::NODE_DISCOVER_SEARCH)
        && let Some(query) = layout.text_input.as_deref()
    {
        if let Ok(update) = runtime.dispatch(json!({
            "type":"discoverRequested",
            "contentType":screen_state.discover_content_type,
            "filters":{
                "catalogKey":screen_state.discover_catalog_key,
                "extra":discover_extra_filters(
                    query,
                    &model.selected_extra_name,
                    &screen_state.discover_extra_value,
                )
            },
            "language":model.language
        })) {
            pending_effects.extend(update.effects);
        }
    }
    if let Some((key, value)) = layout.filter_change {
        match key.as_str() {
            "discover:contentType" => {
                screen_state.discover_content_type = value;
                screen_state.discover_catalog_key.clear();
                screen_state.discover_extra_value.clear();
                if let Ok(update) = runtime.dispatch(json!({
                    "type":"discoverRequested",
                    "loadCatalogFilters":true,
                    "contentType":screen_state.discover_content_type,
                    "filters":{
                        "catalogKey":null,
                        "extra":discover_extra_filters(
                            &model.query,
                            &model.selected_extra_name,
                            &screen_state.discover_extra_value,
                        )
                    },
                    "language":model.language,
                })) {
                    pending_effects.extend(update.effects);
                }
            }
            "discover:catalog" => {
                screen_state.discover_catalog_key = value;
                screen_state.discover_extra_value.clear();
                if let Ok(update) = runtime.dispatch(json!({
                    "type":"discoverRequested",
                    "contentType":screen_state.discover_content_type,
                    "filters":{
                        "catalogKey":screen_state.discover_catalog_key,
                        "extra":discover_extra_filters(
                            &model.query,
                            &model.selected_extra_name,
                            &screen_state.discover_extra_value,
                        )
                    },
                    "language":model.language
                })) {
                    pending_effects.extend(update.effects);
                }
            }
            "discover:extra" => {
                screen_state.discover_extra_value = value;
                let extra = model.selected_extra_name.as_str();
                let value = screen_state.discover_extra_value.as_str();
                let extra_filters = discover_extra_filters(&model.query, extra, value);
                if let Ok(update) = runtime.dispatch(json!({
                    "type":"discoverRequested",
                    "contentType":screen_state.discover_content_type,
                    "filters":{"catalogKey":screen_state.discover_catalog_key,"extra":extra_filters},
                    "language":model.language
                })) {
                    pending_effects.extend(update.effects);
                }
            }
            _ => {}
        }
    }
    if let Some(node) = layout.activated {
        if shared_desktop_navigation(node, active_page, runtime, pending_effects) {
            return;
        }
        if (fluxa_ui::NODE_DISCOVER_TYPE_BASE..fluxa_ui::NODE_DISCOVER_TYPE_BASE + 2)
            .contains(&node)
        {
            screen_state.discover_content_type = if node == fluxa_ui::NODE_DISCOVER_TYPE_BASE {
                "movie"
            } else {
                "series"
            }
            .to_owned();
            if let Ok(update) = runtime.dispatch(json!({
                "type":"discoverRequested",
                "loadCatalogFilters":true,
                "contentType":screen_state.discover_content_type,
                "filters":{"catalogKey":null,"extra":discover_extra_filters(&model.query, &model.selected_extra_name, &screen_state.discover_extra_value)},
                "language":model.language,
            })) { pending_effects.extend(update.effects); }
        } else if (fluxa_ui::NODE_DISCOVER_CATALOG_BASE..fluxa_ui::NODE_CARD_BASE).contains(&node) {
            if let Some(catalog) = model
                .catalogs
                .get((node - fluxa_ui::NODE_DISCOVER_CATALOG_BASE) as usize)
            {
                screen_state.discover_catalog_key = catalog.key.clone();
                if let Ok(update) = runtime.dispatch(json!({"type":"discoverRequested", "contentType":model.content_type, "filters":{"catalogKey":catalog.key,"extra":{}}, "language":"en"})) { pending_effects.extend(update.effects); }
            }
        } else if node >= fluxa_ui::NODE_CARD_BASE {
            if let Some(card) = model
                .results
                .get((node - fluxa_ui::NODE_CARD_BASE) as usize)
            {
                if let (Some(id), Some(content_type)) = (card.id.clone(), card.item_type.clone()) {
                    dispatch_shared_detail_load(
                        runtime,
                        pending_effects,
                        active_page,
                        id,
                        content_type,
                    );
                }
            }
        }
    }
    screen_state.discover_model_cache = Some((cache_key, model));
    let _ = status;
}

pub(super) fn draw_shared_calendar_screen(
    ui: &mut egui::Ui,
    runtime: &mut FluxaRuntime,
    artwork: &mut ArtworkRegistry,
    background_texture: TextureId,
    active_page: &mut String,
    screen_state: &mut NativeScreenState,
    pending_effects: &mut Vec<Value>,
    status: &mut String,
) {
    if screen_state.calendar_selected_day.is_some()
        && ui.ctx().input(|input| input.key_pressed(egui::Key::Escape))
    {
        screen_state.calendar_selected_day = None;
    }
    let mut model = fluxa_ui::calendar_model_from_core_snapshot(runtime.snapshot());
    model.selected_day = screen_state.calendar_selected_day;
    let mut assets = DesktopHomeAssets {
        background: background_texture,
        artwork,
    };
    let layout = fluxa_ui::draw_calendar(
        ui.ctx(),
        shared_desktop_viewport(ui),
        &model,
        &mut assets,
        None,
    );
    if let Some(node) = layout.activated {
        if shared_desktop_navigation(node, active_page, runtime, pending_effects) {
            return;
        }
        if node == fluxa_ui::NODE_CALENDAR_CLOSE_DAY {
            screen_state.calendar_selected_day = None;
            return;
        }
        if (fluxa_ui::NODE_CALENDAR_DAY_BASE..fluxa_ui::NODE_CALENDAR_DAY_BASE + 32).contains(&node)
        {
            screen_state.calendar_selected_day =
                Some((node - fluxa_ui::NODE_CALENDAR_DAY_BASE) as u32);
            return;
        }
        if (fluxa_ui::NODE_CALENDAR_EVENT_BASE..fluxa_ui::NODE_CALENDAR_EVENT_BASE + 10)
            .contains(&node)
        {
            if let Some(day) = model.selected_day {
                if let Some(entry) = model
                    .entries_for_day(day)
                    .nth((node - fluxa_ui::NODE_CALENDAR_EVENT_BASE) as usize)
                {
                    if let (Some(id), Some(item_type)) =
                        (entry.card.id.clone(), entry.card.item_type.clone())
                    {
                        dispatch_shared_detail_load(
                            runtime,
                            pending_effects,
                            active_page,
                            id,
                            item_type,
                        );
                    }
                }
            }
            return;
        }
        let delta = if node == fluxa_ui::NODE_CALENDAR_PREV {
            -1
        } else if node == fluxa_ui::NODE_CALENDAR_NEXT {
            1
        } else {
            0
        };
        if delta != 0 {
            screen_state.calendar_selected_day = None;
            let mut year = if model.year > 0 {
                model.year
            } else {
                current_year_month().0
            };
            let mut month = if model.month > 0 {
                model.month
            } else {
                current_year_month().1
            };
            month += delta;
            if month < 1 {
                year -= 1;
                month = 12;
            }
            if month > 12 {
                year += 1;
                month = 1;
            }
            if let Ok(update) = runtime
                .dispatch(json!({"type":"calendarMonthRequested", "year":year, "month":month}))
            {
                pending_effects.extend(update.effects);
            }
        }
    }
    let _ = status;
}

#[cfg(test)]
mod discover_adapter_tests {
    use super::{discover_extra_filters, enter_shared_desktop_page};
    use crate::FluxaRuntime;
    use serde_json::json;

    #[test]
    fn discover_filters_keep_search_and_selected_extra_together() {
        assert_eq!(
            discover_extra_filters("space", "genre", "sci-fi"),
            json!({"search": "space", "genre": "sci-fi"})
        );
        assert_eq!(discover_extra_filters("", "genre", ""), json!({}));
    }

    #[test]
    fn entering_discover_and_calendar_dispatches_their_core_loads() {
        let mut runtime = FluxaRuntime::new(json!({})).expect("initialize Fluxa Core runtime");
        let mut pending = Vec::new();
        let mut page = "Home".to_owned();

        enter_shared_desktop_page(
            "Discover",
            "discover",
            &mut page,
            &mut runtime,
            &mut pending,
        );
        assert_eq!(page, "Discover");
        assert!(pending.iter().any(|effect| {
            effect.get("type").and_then(serde_json::Value::as_str)
                == Some("readDiscoverCatalogFilters")
        }));

        pending.clear();
        enter_shared_desktop_page(
            "Calendar",
            "calendar",
            &mut page,
            &mut runtime,
            &mut pending,
        );
        assert_eq!(page, "Calendar");
        assert!(pending.iter().any(|effect| {
            effect.get("type").and_then(serde_json::Value::as_str) == Some("readCalendarMonth")
        }));
    }
}
