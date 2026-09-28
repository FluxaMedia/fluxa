use super::*;

pub(super) fn native_action_for_node(
    node: u64,
    home: &HomeModel,
    library: &LibraryModel,
    library_tab: LibraryTab,
    discover: &DiscoverModel,
    calendar: &CalendarModel,
    detail: &DetailModel,
    settings: &SettingsModel,
    route: Route,
    hero: Option<&HomeHero>,
) -> Option<NativeAction> {
    let destination = match node {
        NODE_HOME => Some(Route::Home),
        NODE_LIBRARY => Some(Route::Library),
        NODE_DISCOVER => Some(Route::Discover),
        NODE_CALENDAR => Some(Route::Calendar),
        NODE_PROFILE => Some(Route::Settings),
        _ => None,
    };
    if let Some(destination) = destination {
        return Some(NativeAction::Navigate { destination });
    }
    if route == Route::Discover {
        if node == fluxa_ui::NODE_DISCOVER_TYPE_BASE {
            return Some(NativeAction::DiscoverType {
                content_type: if node == fluxa_ui::NODE_DISCOVER_TYPE_BASE {
                    "movie".to_owned()
                } else {
                    "series".to_owned()
                },
            });
        }
        if node == fluxa_ui::NODE_DISCOVER_CATALOG_BASE {
            let index = (node - fluxa_ui::NODE_DISCOVER_CATALOG_BASE) as usize;
            if let Some(catalog) = discover
                .catalogs
                .iter()
                .filter(|catalog| catalog.content_type == discover.content_type)
                .nth(index)
            {
                return Some(NativeAction::DiscoverCatalog {
                    content_type: discover.content_type.clone(),
                    catalog_key: catalog.key.clone(),
                    extra_name: discover.selected_extra_name.clone(),
                    extra_value: discover.selected_extra_value.clone(),
                    query: discover.query.clone(),
                });
            }
        }
    }
    if route == Route::Calendar {
        if (fluxa_ui::NODE_CALENDAR_EVENT_BASE..fluxa_ui::NODE_CALENDAR_EVENT_BASE + 10)
            .contains(&node)
        {
            let day = calendar.selected_day?;
            let entry = calendar
                .entries_for_day(day)
                .nth((node - fluxa_ui::NODE_CALENDAR_EVENT_BASE) as usize)?;
            return Some(NativeAction::Detail {
                id: entry.card.id.clone()?,
                item_type: entry.card.item_type.clone()?,
                preview: entry.card.raw.clone(),
            });
        }
        if node == fluxa_ui::NODE_CALENDAR_PREV || node == fluxa_ui::NODE_CALENDAR_NEXT {
            let delta = if node == fluxa_ui::NODE_CALENDAR_PREV {
                -1
            } else {
                1
            };
            let mut year = calendar.year;
            let mut month = calendar.month + delta;
            if month < 1 {
                year -= 1;
                month = 12;
            } else if month > 12 {
                year += 1;
                month = 1;
            }
            return Some(NativeAction::CalendarMonth { year, month });
        }
    }
    if route == Route::Settings {
        if node == fluxa_ui::NODE_SETTINGS_BACK {
            return Some(NativeAction::Navigate {
                destination: Route::Home,
            });
        }
        if node == fluxa_ui::NODE_SETTINGS_SWITCH_PROFILE {
            return Some(NativeAction::Navigate {
                destination: Route::Profiles,
            });
        }
        if let Some(provider) = node
            .checked_sub(fluxa_ui::NODE_SETTINGS_ACCOUNT_BASE)
            .and_then(|slot| fluxa_ui::ACCOUNT_PROVIDERS.get(slot as usize))
        {
            return Some(NativeAction::AccountToggle {
                provider: (*provider).to_owned(),
            });
        }
        if (fluxa_ui::NODE_SETTINGS_SECTION_BASE
            ..fluxa_ui::NODE_SETTINGS_SECTION_BASE + fluxa_ui::SETTINGS_SECTIONS.len() as u64)
            .contains(&node)
        {
            return Some(NativeAction::SettingsSection {
                index: (node - fluxa_ui::NODE_SETTINGS_SECTION_BASE) as usize,
            });
        }
        if (fluxa_ui::NODE_SETTINGS_ROW_BASE
            ..fluxa_ui::NODE_SETTINGS_ROW_BASE
                + fluxa_ui::SETTINGS_SECTIONS
                    .iter()
                    .map(|section| section.rows.len())
                    .sum::<usize>() as u64)
            .contains(&node)
        {
            let index = (node - fluxa_ui::NODE_SETTINGS_ROW_BASE) as usize;
            let setting = fluxa_ui::settings_row_by_index(index)?;
            return Some(NativeAction::SettingsChange {
                key: setting.key.to_owned(),
                value: settings.next_value_for(setting, home.form_factor.into()),
            });
        }
    }
    if route == Route::Detail {
        if node == fluxa_ui::NODE_DETAIL_BACK {
            return Some(NativeAction::Back);
        }
        if node == fluxa_ui::NODE_DETAIL_WATCHLIST {
            return Some(NativeAction::ToggleWatchlist {
                item: detail.item.clone(),
            });
        }
        if node == fluxa_ui::NODE_DETAIL_PLAY {
            if !detail.id.is_empty() {
                let first = detail
                    .episodes
                    .iter()
                    .find(|episode| episode.season > 0)
                    .or(detail.episodes.first())
                    .map(|episode| episode.id.clone());
                return Some(NativeAction::StartPlayback {
                    item: playback_item(&detail.item, detail.resume.as_ref(), first),
                });
            }
        }
        if node >= fluxa_ui::NODE_DETAIL_EPISODE_BASE {
            let episode = detail
                .episodes
                .get((node - fluxa_ui::NODE_DETAIL_EPISODE_BASE) as usize)?;
            let mut item = detail.item.clone();
            item.as_object_mut()?
                .insert("lastVideoId".to_owned(), episode.id.clone().into());
            return Some(NativeAction::StartPlayback { item });
        }
        if node >= fluxa_ui::NODE_DETAIL_SEASON_BASE {
            return None;
        }
        if (fluxa_ui::NODE_DETAIL_SIMILAR_BASE..fluxa_ui::NODE_DETAIL_CAST_BASE).contains(&node) {
            let card = detail
                .similar
                .get((node - fluxa_ui::NODE_DETAIL_SIMILAR_BASE) as usize)?;
            return Some(NativeAction::Detail {
                id: card.id.as_ref()?.clone(),
                item_type: card.item_type.as_ref()?.clone(),
                preview: card.raw.clone(),
            });
        }
    }
    let (id, item_type, preview) = if route == Route::Library && node >= NODE_CARD_BASE {
        let card = library
            .cards(library_tab)
            .get((node - NODE_CARD_BASE) as usize)?;
        (card.id.as_ref()?, card.item_type.as_ref()?, &card.raw)
    } else if route == Route::Discover && node >= NODE_CARD_BASE {
        let card = discover.results.get((node - NODE_CARD_BASE) as usize)?;
        (card.id.as_ref()?, card.item_type.as_ref()?, &card.raw)
    } else if node == fluxa_ui::NODE_HERO_WATCHLIST {
        return Some(NativeAction::ToggleWatchlist {
            item: hero?.raw.clone(),
        });
    } else if node == NODE_PLAY || node == NODE_MORE_INFO {
        if let Some(hero) = hero
            && let (Some(id), Some(item_type)) = (hero.item_id.as_ref(), hero.item_type.as_ref())
        {
            if node == NODE_MORE_INFO {
                return Some(NativeAction::Detail {
                    id: id.clone(),
                    item_type: item_type.clone(),
                    preview: hero.raw.clone(),
                });
            }
            let series = matches!(item_type.as_str(), "series" | "tv" | "show");
            return Some(NativeAction::StartPlayback {
                item: playback_item(
                    &hero.raw,
                    home.resume_for(id),
                    series.then(|| format!("{id}:1:1")),
                ),
            });
        }
        (
            home.item_id.as_ref()?,
            home.item_type.as_ref()?,
            &Value::Null,
        )
    } else if node >= NODE_CARD_BASE {
        let card = home.card_at((node - NODE_CARD_BASE) as usize)?;
        (card.id.as_ref()?, card.item_type.as_ref()?, &card.raw)
    } else {
        return None;
    };
    if node == NODE_PLAY {
        Some(NativeAction::Play {
            id: id.clone(),
            item_type: item_type.clone(),
        })
    } else {
        Some(NativeAction::Detail {
            id: id.clone(),
            item_type: item_type.clone(),
            preview: preview.clone(),
        })
    }
}

pub(super) fn playback_item(
    item: &Value,
    resume: Option<&HomeCard>,
    first_video: Option<String>,
) -> Value {
    let mut item = item.clone();
    let Some(fields) = item.as_object_mut() else {
        return item;
    };
    if let Some(resume) = resume.and_then(|card| card.raw.as_object()) {
        for (key, value) in resume {
            if key.starts_with("last") || matches!(key.as_str(), "timeOffset" | "duration") {
                fields.insert(key.clone(), value.clone());
            }
        }
        if !fields.contains_key("lastVideoId")
            && let Some(video) = resume.get("videoId")
        {
            fields.insert("lastVideoId".to_owned(), video.clone());
        }
    } else if let Some(video) = first_video {
        fields.insert("lastVideoId".to_owned(), video.into());
    }
    item
}

pub(super) fn refresh_library_view(state: &mut RendererState) {
    state.library.query = state.library_query.clone();
    state.library.sort_by = state.library_sort.clone();
    if let Some(snapshot) = state.core_snapshot.as_ref() {
        let library = snapshot
            .get("library")
            .cloned()
            .unwrap_or_else(|| json!({}));
        if let Some(plan) = core_value(
            "libraryViewPlan",
            json!({
                "watchlist": library.get("watchlist"),
                "watching": library.get("continueWatching"),
                "completed": library.get("completed"),
                "dropped": library.get("dropped"),
                "favorites": library.get("liked"),
                "progress": library.get("progress"),
                "tab": state.library_tab.core_tab_key(),
                "query": state.library_query,
                "sortBy": state.library_sort,
            }),
        ) {
            state.library.apply_core_plan(&plan, state.library_tab);
        }
    }
}

pub(super) fn settings_action_json(node: u64, settings: &SettingsModel) -> Option<Value> {
    if node == fluxa_ui::NODE_SETTINGS_ADDON_INSTALL {
        let url = settings.addon_url.trim();
        return (!url.is_empty()).then(
            || json!({"type":"addonInstallRequested", "transportUrl":url, "forceRefresh":false}),
        );
    }
    if node == fluxa_ui::NODE_SETTINGS_ADDON_REFRESH {
        return Some(
            json!({"type":"addonsRefreshRequested", "profile":settings.profile, "forceRefresh":true}),
        );
    }
    for (field, value) in fluxa_ui::POSTER_FIELDS.iter().zip(&settings.poster_fields) {
        if node == field.save {
            return Some(json!({"type":"settingsChanged", "key":field.key, "value":value.trim()}));
        }
        if node == field.clear {
            return Some(json!({"type":"settingsChanged", "key":field.key, "value":""}));
        }
    }
    if node == fluxa_ui::NODE_SETTINGS_PLUGIN_INSTALL {
        let url = settings.plugin_url.trim();
        return (!url.is_empty())
            .then(|| json!({"type":"pluginRepositoryAddRequested", "manifestUrl":url}));
    }
    let repositories = settings
        .plugins
        .get("repositories")
        .and_then(Value::as_array);
    if (fluxa_ui::NODE_SETTINGS_PLUGIN_REPOSITORY_BASE
        ..fluxa_ui::NODE_SETTINGS_PLUGIN_REPOSITORY_BASE + 4)
        .contains(&node)
    {
        let index = (node - fluxa_ui::NODE_SETTINGS_PLUGIN_REPOSITORY_BASE) as usize;
        let url = repositories?.get(index)?.get("manifestUrl")?.as_str()?;
        return Some(json!({"type":"pluginRepositoryRemoveRequested", "manifestUrl":url}));
    }
    if (fluxa_ui::NODE_SETTINGS_PLUGIN_REFRESH_BASE
        ..fluxa_ui::NODE_SETTINGS_PLUGIN_REFRESH_BASE + 4)
        .contains(&node)
    {
        let index = (node - fluxa_ui::NODE_SETTINGS_PLUGIN_REFRESH_BASE) as usize;
        let url = repositories?.get(index)?.get("manifestUrl")?.as_str()?;
        return Some(json!({"type":"pluginRepositoryAddRequested", "manifestUrl":url}));
    }
    if (fluxa_ui::NODE_SETTINGS_PLUGIN_SCRAPER_BASE
        ..fluxa_ui::NODE_SETTINGS_PLUGIN_SCRAPER_BASE + 4)
        .contains(&node)
    {
        let index = (node - fluxa_ui::NODE_SETTINGS_PLUGIN_SCRAPER_BASE) as usize;
        let scraper = settings.plugins.get("scrapers")?.as_array()?.get(index)?;
        let id = scraper.get("id")?.as_str()?;
        let enabled = !scraper
            .get("enabled")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        return Some(json!({"type":"pluginScraperToggled", "scraperId":id, "enabled":enabled}));
    }
    None
}

pub(super) fn edit_text(state: &mut RendererState, node: u64, edit: impl FnOnce(&mut String)) -> bool {
    let route = state.route;
    let text = match node {
        fluxa_ui::NODE_LIBRARY_SEARCH if route == Route::Library => &mut state.library_query,
        fluxa_ui::NODE_DISCOVER_SEARCH if route == Route::Discover => &mut state.discover.query,
        fluxa_ui::NODE_SETTINGS_SEARCH if route == Route::Settings => &mut state.settings.search,
        fluxa_ui::NODE_SETTINGS_ADDON_URL if route == Route::Settings => &mut state.settings.addon_url,
        fluxa_ui::NODE_SETTINGS_PLUGIN_URL if route == Route::Settings => &mut state.settings.plugin_url,
        _ if route == Route::Settings && fluxa_ui::poster_field(node).is_some() => {
            &mut state.settings.poster_fields[fluxa_ui::poster_field(node).unwrap_or_default()]
        }
        _ => return false,
    };
    let before = text.clone();
    edit(text);
    if *text == before {
        return true;
    }
    match node {
        fluxa_ui::NODE_LIBRARY_SEARCH => refresh_library_view(state),
        fluxa_ui::NODE_DISCOVER_SEARCH => request_discover_search(state),
        _ => {}
    }
    true
}

pub(super) fn focused_text(state: &RendererState) -> Option<String> {
    let node = state.ui.focused()?;
    let route = state.route;
    Some(match node {
        fluxa_ui::NODE_LIBRARY_SEARCH if route == Route::Library => state.library_query.clone(),
        fluxa_ui::NODE_DISCOVER_SEARCH if route == Route::Discover => state.discover.query.clone(),
        fluxa_ui::NODE_SETTINGS_SEARCH if route == Route::Settings => state.settings.search.clone(),
        fluxa_ui::NODE_SETTINGS_ADDON_URL if route == Route::Settings => state.settings.addon_url.clone(),
        fluxa_ui::NODE_SETTINGS_PLUGIN_URL if route == Route::Settings => state.settings.plugin_url.clone(),
        _ if route == Route::Settings => state.settings.poster_fields[fluxa_ui::poster_field(node)?].clone(),
        _ => return None,
    })
}

pub(super) fn remember_actions(state: &mut RendererState, actions: Vec<UiAction>) {
    if actions.is_empty() {
        return;
    }
    for action in &actions {
        if let UiAction::TextInput { node, value } = action {
            edit_text(state, *node, |text| text.push_str(value));
            continue;
        }
        let node = match action {
            UiAction::Activated(node) | UiAction::PointerReleased(node) => Some(*node),
            _ => None,
        };
        if state.player.is_some() {
            if let Some(node) = node {
                player::activate(state, node);
            } else if matches!(action, UiAction::Back) {
                player::close(state);
            }
            continue;
        }
        if let Some(node) = node {
            if state.route == Route::Library && node == fluxa_ui::NODE_LIBRARY_SORT {
                state.library_sort = match state.library_sort.as_str() {
                    "recent" => "title",
                    "title" => "rating",
                    _ => "recent",
                }
                .to_owned();
                refresh_library_view(state);
                continue;
            }
            if state.route == Route::Calendar {
                if node == fluxa_ui::NODE_CALENDAR_CLOSE_DAY {
                    state.calendar.selected_day = None;
                    reset_ui(state);
                    continue;
                }
                if (fluxa_ui::NODE_CALENDAR_DAY_BASE..fluxa_ui::NODE_CALENDAR_DAY_BASE + 32)
                    .contains(&node)
                {
                    state.calendar.selected_day =
                        Some((node - fluxa_ui::NODE_CALENDAR_DAY_BASE) as u32);
                    reset_ui(state);
                    continue;
                }
            }
            if state.route == Route::Settings
                && logical_viewport(state).is_compact()
                && let Some(row) = node
                    .checked_sub(fluxa_ui::NODE_SETTINGS_ROW_BASE)
                    .and_then(|index| fluxa_ui::settings_row_by_index(index as usize))
                && !row.options.is_empty()
                && row.key != "accentColorArgb"
            {
                card_menu::open_setting(state, row);
                continue;
            }
            if state.route == Route::Settings
                && node == fluxa_ui::NODE_SETTINGS_BACK
                && state.settings.section_open
            {
                close_settings_section(state);
                continue;
            }
            if state.route == Route::Settings
                && (fluxa_ui::NODE_SETTINGS_SECTION_BASE
                    ..fluxa_ui::NODE_SETTINGS_SECTION_BASE
                        + fluxa_ui::SETTINGS_SECTIONS.len() as u64)
                    .contains(&node)
            {
                state.settings.active_section =
                    (node - fluxa_ui::NODE_SETTINGS_SECTION_BASE) as usize;
                state.settings.section_open = true;
                state.settings.search.clear();
                state.screen_scroll_offsets.remove(&Route::Settings);
                reset_ui(state);
                continue;
            }
            if state.route == Route::Settings
                && let Some(action_json) = settings_action_json(node, &state.settings)
            {
                if node == fluxa_ui::NODE_SETTINGS_ADDON_INSTALL {
                    state.settings.addon_url.clear();
                } else if node == fluxa_ui::NODE_SETTINGS_PLUGIN_INSTALL {
                    state.settings.plugin_url.clear();
                }
                state
                    .pending_native_actions
                    .push(NativeAction::CoreCommand {
                        command: action_json,
                    });
                reset_ui(state);
                continue;
            }
            if state.route == Route::Library
                && (fluxa_ui::NODE_LIBRARY_TAB_BASE
                    ..fluxa_ui::NODE_LIBRARY_TAB_BASE + LibraryTab::ALL.len() as u64)
                    .contains(&node)
            {
                state.library_tab =
                    LibraryTab::ALL[(node - fluxa_ui::NODE_LIBRARY_TAB_BASE) as usize];
                refresh_library_view(state);
                reset_ui(state);
                continue;
            }
            if state.route == Route::Detail && node == fluxa_ui::NODE_DETAIL_SHUFFLE {
                if let Some(item) = player::start_shuffle(state) {
                    state.pending_native_actions.push(NativeAction::StartPlayback { item });
                }
                continue;
            }
            if let Some(native_action) = native_action_for_node(
                node,
                &state.home,
                &state.library,
                state.library_tab,
                &state.discover,
                &state.calendar,
                &state.detail,
                &state.settings,
                state.route,
                state
                    .gpu
                    .as_ref()
                    .and_then(|gpu| fluxa_ui::active_home_hero(&gpu.egui_context, &state.home)),
            ) {
                if let NativeAction::Detail { preview, .. } = &native_action {
                    state.backdrop_prefetch = ["background", "poster"]
                        .into_iter()
                        .find_map(|key| preview.get(key).and_then(Value::as_str))
                        .map(ToOwned::to_owned);
                }
                state.pending_native_actions.push(native_action);
            }
        }
        if matches!(action, UiAction::Back) {
            if state.route == Route::Calendar && state.calendar.selected_day.is_some() {
                state.calendar.selected_day = None;
                reset_ui(state);
            } else if state.route == Route::Settings && state.settings.section_open {
                close_settings_section(state);
            } else {
                state.pending_native_actions.push(NativeAction::Back);
            }
        }
    }
    state.last_actions.extend(actions);
    // Keep the bridge bounded while the real scene/action dispatcher is being
    // connected. The queue is diagnostic for now, not the source of truth.
    const MAX_RETAINED_ACTIONS: usize = 32;
    if state.last_actions.len() > MAX_RETAINED_ACTIONS {
        let drain_count = state.last_actions.len() - MAX_RETAINED_ACTIONS;
        state.last_actions.drain(..drain_count);
    }
    const MAX_PENDING_NATIVE_ACTIONS: usize = 16;
    if state.pending_native_actions.len() > MAX_PENDING_NATIVE_ACTIONS {
        let drain_count = state.pending_native_actions.len() - MAX_PENDING_NATIVE_ACTIONS;
        state.pending_native_actions.drain(..drain_count);
    }
}

pub(super) fn take_native_actions(state: &mut RendererState) -> String {
    if state.pending_native_actions.is_empty() {
        return "[]".to_owned();
    }
    serde_json::to_string(&std::mem::take(&mut state.pending_native_actions))
        .unwrap_or_else(|_| "[]".to_owned())
}

pub(super) fn profile_language(profile: &Value) -> String {
    profile
        .get("language")
        .and_then(Value::as_str)
        .filter(|language| !language.is_empty())
        .unwrap_or("en")
        .to_owned()
}

pub(super) fn pull_session_snapshot(state: &mut RendererState) {
    let Some(session) = state.session.as_ref() else {
        return;
    };
    let revision = session.revision();
    if state.session_revision == Some(revision) {
        return;
    }
    state.session_revision = Some(revision);
    state.core_snapshot = Some(session.snapshot());
    state.core_snapshot_revision = state.core_snapshot_revision.wrapping_add(1);
    sync_home_from_core_snapshot(state);
}

pub(super) fn discover_command(
    profile: &Value,
    content_type: &str,
    catalog_key: &str,
    extra_name: &str,
    extra_value: &str,
    load_catalog_filters: bool,
) -> Value {
    let mut extra = serde_json::Map::new();
    if !extra_name.is_empty() && !extra_value.is_empty() {
        extra.insert(extra_name.to_owned(), json!(extra_value));
    }
    json!({
        "type": "discoverRequested",
        "loadCatalogFilters": load_catalog_filters,
        "contentType": content_type,
        "filters": {
            "catalogKey": (!catalog_key.is_empty()).then_some(catalog_key),
            "extra": extra,
        },
        "profile": profile,
        "language": profile_language(profile),
    })
}

pub(super) fn navigation(route: Route) -> Value {
    json!({"type": "navigationRequested", "route": route.as_str(), "params": {}})
}

pub(super) fn session_commands(action: &NativeAction, profile: &Value) -> Option<Vec<Value>> {
    let commands = match action {
        NativeAction::CoreCommand { command } => vec![command.clone()],
        NativeAction::LoadMore { .. } => Vec::new(),
        NativeAction::Back => vec![navigation(Route::Home)],
        NativeAction::Navigate { destination } => match destination {
            Route::Home => vec![
                navigation(Route::Home),
                json!({
                    "type": "refreshContinueWatchingRequested",
                    "profile": profile,
                    "language": profile_language(profile),
                    "source": "navigation",
                }),
            ],
            Route::Library => vec![
                navigation(Route::Library),
                json!({"type": "libraryHydrateRequested", "profileId": profile.get("id")}),
            ],
            Route::Discover => vec![
                navigation(Route::Discover),
                discover_command(profile, "movie", "", "", "", true),
            ],
            Route::Calendar => {
                let (year, month) = current_year_month();
                vec![
                    navigation(Route::Calendar),
                    json!({"type": "calendarMonthRequested", "profile": profile, "year": year, "month": month, "plannedItems": []}),
                ]
            }
            Route::Settings => vec![navigation(Route::Settings)],
            _ => return None,
        },
        NativeAction::DiscoverType { content_type } => {
            vec![discover_command(profile, content_type, "", "", "", true)]
        }
        NativeAction::DiscoverCatalog {
            content_type,
            catalog_key,
            extra_name,
            extra_value,
            ..
        }
        | NativeAction::DiscoverFilters {
            content_type,
            catalog_key,
            extra_name,
            extra_value,
            ..
        } => vec![discover_command(
            profile,
            content_type,
            catalog_key,
            extra_name,
            extra_value,
            false,
        )],
        NativeAction::CalendarMonth { year, month } => vec![
            json!({"type": "calendarMonthRequested", "profile": profile, "year": year, "month": month, "plannedItems": []}),
        ],
        NativeAction::Detail { id, item_type, .. } | NativeAction::Play { id, item_type } => vec![
            navigation(Route::Detail),
            json!({
                "type": "detailLoadRequested",
                "id": id,
                "contentType": item_type,
                "language": profile_language(profile),
                "profile": profile,
                "preview": match action {
                    NativeAction::Detail { preview, .. } => preview.clone(),
                    _ => Value::Null,
                },
            }),
        ],
        NativeAction::ToggleWatchlist { item } => {
            vec![json!({"type": "toggleWatchlistRequested", "item": item, "profile": profile})]
        }
        NativeAction::SettingsChange { key, value } => {
            vec![json!({"type": "settingsChanged", "key": key, "value": value})]
        }
        NativeAction::StartPlayback { item } => {
            vec![player::direct_playback_command(item, profile)]
        }
        NativeAction::SettingsSection { .. } | NativeAction::AccountToggle { .. } => return None,
    };
    Some(commands)
}

pub(super) fn route_actions_to_session(state: &mut RendererState) {
    let Some(session) = state.session.as_ref() else {
        start_playback_without_session(state);
        return;
    };
    if state.pending_native_actions.is_empty() {
        return;
    }
    let profile = session.active_profile();
    let mut unhandled = Vec::new();
    let mut open_profiles = false;
    let mut toggles = Vec::new();
    for action in std::mem::take(&mut state.pending_native_actions) {
        if let NativeAction::AccountToggle { provider } = action {
            toggles.push(provider);
            continue;
        }
        if matches!(&action, NativeAction::Navigate { destination } if *destination == Route::Profiles)
        {
            open_profiles = true;
            continue;
        }
        if let NativeAction::StartPlayback { item } = &action {
            player::keep_shuffle_for(&mut state.shuffle, item);
            state.player = Some(player::PlayerSession::new(item.clone()));
            state.ui = UiTree::default();
        }
        if matches!(&action, NativeAction::Navigate { destination } if *destination == Route::Discover)
            && !state.discover.catalogs.is_empty()
        {
            if let Err(error) = session.dispatch(navigation(Route::Discover)) {
                host_log(format!("core dispatch failed: {error}"));
            }
            continue;
        }
        match session_commands(&action, &profile) {
            Some(commands) => {
                for command in commands {
                    if let Err(error) = session.dispatch(command) {
                        host_log(format!("core dispatch failed: {error}"));
                    }
                }
            }
            None => unhandled.push(action),
        }
    }
    state.pending_native_actions = unhandled;
    for provider in toggles {
        accounts::toggle(state, &provider);
    }
    if open_profiles {
        reset_ui(state);
        profiles::open(state);
    }
}

pub(super) fn start_playback_without_session(state: &mut RendererState) {
    let profile = state
        .core_snapshot
        .as_ref()
        .and_then(|snapshot| snapshot.pointer("/profile/active").cloned())
        .unwrap_or(Value::Null);
    for index in 0..state.pending_native_actions.len() {
        let NativeAction::StartPlayback { item } = &state.pending_native_actions[index] else {
            continue;
        };
        player::keep_shuffle_for(&mut state.shuffle, item);
        let command = player::direct_playback_command(item, &profile);
        let mut player = player::PlayerSession::new(item.clone());
        player.stale = state
            .core_snapshot
            .as_ref()
            .and_then(|snapshot| snapshot.get("player"))
            .cloned();
        state.player = Some(player);
        reset_ui(state);
        state.pending_native_actions[index] = NativeAction::CoreCommand { command };
    }
}

fn close_settings_section(state: &mut RendererState) {
    state.settings.section_open = false;
    state.screen_scroll_offsets.remove(&Route::Settings);
    reset_ui(state);
}
