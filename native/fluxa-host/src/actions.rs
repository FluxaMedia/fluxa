use super::*;

pub(super) fn native_action_for_node(
    node: u64,
    home: &HomeModel,
    library: &LibraryModel,
    library_tab: LibraryTab,
    discover: &DiscoverModel,
    folder: &FolderModel,
    folder_tab: usize,
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
        fluxa_ui::NODE_SHORTS => Some(Route::Shorts),
        NODE_PROFILE => Some(Route::Settings),
        fluxa_ui::NODE_SIDEBAR_SEARCH => Some(Route::Discover),
        fluxa_ui::NODE_SIDEBAR_DOWNLOADS => Some(Route::Library),
        _ => None,
    };
    if let Some(destination) = destination {
        return Some(NativeAction::Navigate { destination });
    }
    if route == Route::Discover {
        if let Some(entry) =
            fluxa_ui::discover_recent(node).and_then(|index| discover.recent_searches.get(index))
        {
            return Some(NativeAction::Search {
                query: entry.clone(),
            });
        }
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
        if node == fluxa_ui::NODE_SETTINGS_LOG_OUT {
            return Some(NativeAction::LogOut);
        }
        if let Some(provider) = node
            .checked_sub(fluxa_ui::NODE_SETTINGS_ACCOUNT_BASE)
            .and_then(|slot| fluxa_ui::ACCOUNT_PROVIDERS.get(slot as usize))
        {
            return Some(NativeAction::AccountToggle {
                provider: (*provider).to_owned(),
            });
        }
        if matches!(
            node,
            fluxa_ui::NODE_SETTINGS_SERVER_JELLYFIN
                | fluxa_ui::NODE_SETTINGS_SERVER_EMBY
                | fluxa_ui::NODE_SETTINGS_SERVER_PLEX
                | fluxa_ui::NODE_SETTINGS_SERVER_CONNECT
        ) || fluxa_ui::server_index(node).is_some()
        {
            return Some(NativeAction::MediaServer { node });
        }
        if let Some(index) = fluxa_ui::settings_page_for_node(node) {
            return Some(NativeAction::SettingsSection { index });
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
        if node == fluxa_ui::NODE_DETAIL_FAVORITE {
            return Some(NativeAction::ToggleFavorite {
                item: detail.item.clone(),
            });
        }
        if let Some(index) = node
            .checked_sub(fluxa_ui::NODE_DETAIL_TRAILER_BASE)
            .map(|offset| offset as usize)
            .filter(|index| *index < detail.trailer_cards.len())
        {
            let mut urls = vec![detail.trailer_cards[index].url.clone()];
            urls.extend(
                detail
                    .trailer_cards
                    .iter()
                    .enumerate()
                    .filter(|(other, _)| *other != index)
                    .map(|(_, card)| card.url.clone()),
            );
            return Some(NativeAction::PlayTrailer {
                item: detail.item.clone(),
                urls,
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
    if matches!(route, Route::Home | Route::Discover) && node >= fluxa_ui::NODE_PREVIEW_BASE {
        let slot = node - fluxa_ui::NODE_PREVIEW_BASE;
        let index = (slot / fluxa_ui::PREVIEW_ACTIONS) as usize;
        let card = if route == Route::Home {
            home.card_at(index)?
        } else {
            discover.results.get(index)?
        };
        let id = card.id.as_ref()?.clone();
        let item_type = card.item_type.as_ref()?.clone();
        return Some(match slot % fluxa_ui::PREVIEW_ACTIONS {
            0 => {
                let series = matches!(item_type.as_str(), "series" | "tv" | "show");
                NativeAction::StartPlayback {
                    item: playback_item(
                        &card.raw,
                        home.resume_for(&id),
                        series.then(|| format!("{id}:1:1")),
                    ),
                }
            }
            1 => NativeAction::ToggleWatchlist {
                item: card.raw.clone(),
            },
            2 => NativeAction::Detail {
                id,
                item_type,
                preview: card.raw.clone(),
            },
            3 => {
                let watched = library
                    .personal
                    .get(&id)
                    .is_some_and(|personal| personal.watched);
                NativeAction::CoreCommand {
                    command: json!({
                        "type": "markWatchedRequested",
                        "seriesId": id,
                        "videoIds": [id],
                        "watched": !watched,
                        "meta": card.raw,
                    }),
                }
            }
            _ => NativeAction::CoreCommand {
                command: json!({
                    "type": "clearPlaybackProgressRequested",
                    "meta": card.raw,
                }),
            },
        });
    }
    let (id, item_type, preview) = if route == Route::Library && node >= NODE_CARD_BASE {
        let card = library
            .cards(library_tab)
            .get((node - NODE_CARD_BASE) as usize)?;
        (card.id.as_ref()?, card.item_type.as_ref()?, &card.raw)
    } else if route == Route::Discover && node >= NODE_CARD_BASE {
        let card = discover.results.get((node - NODE_CARD_BASE) as usize)?;
        (card.id.as_ref()?, card.item_type.as_ref()?, &card.raw)
    } else if route == Route::Folder {
        if node == fluxa_ui::NODE_FOLDER_BACK {
            return Some(NativeAction::Back);
        }
        let card = folder.card_at(folder_tab, node.checked_sub(NODE_CARD_BASE)? as usize)?;
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
    } else if route == Route::Home && node >= NODE_CARD_BASE {
        let card = home.card_at((node - NODE_CARD_BASE) as usize)?;
        if card.row_kind == fluxa_ui::HomeRowKind::Collection {
            return Some(NativeAction::Folder {
                id: card.id.as_ref()?.clone(),
            });
        }
        if card.row_kind == fluxa_ui::HomeRowKind::Continue && !card.upcoming {
            return Some(NativeAction::StartPlayback {
                item: playback_item(&card.raw, Some(card), None),
            });
        }
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

fn known_logo(state: &RendererState, item: &Value) -> Option<String> {
    let id = item.get("id").and_then(Value::as_str)?;
    let home = &state.home;
    home.hero_slides
        .iter()
        .find(|hero| hero.item_id.as_deref() == Some(id))
        .and_then(|hero| hero.logo_url.clone())
        .or_else(|| {
            (home.item_id.as_deref() == Some(id))
                .then(|| home.logo_url.clone())
                .flatten()
        })
        .or_else(|| {
            home.cards
                .iter()
                .chain(home.rows.iter().flat_map(|row| row.cards.iter()))
                .find(|card| card.id.as_deref() == Some(id))
                .and_then(|card| card.logo_url.clone())
        })
        .or_else(|| {
            (state.detail.id == id)
                .then(|| state.detail.logo_url.clone())
                .flatten()
        })
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
    let foreign = fields
        .get("lastVideoId")
        .and_then(Value::as_str)
        .is_some_and(|video| !player::video_belongs_to(video, &Value::Object(fields.clone())));
    if foreign {
        let episode = fields
            .get("id")
            .and_then(Value::as_str)
            .zip(fields.get("lastEpisodeSeason").and_then(Value::as_i64))
            .zip(fields.get("lastEpisodeNumber").and_then(Value::as_i64))
            .map(|((id, season), number)| format!("{id}:{season}:{number}"));
        fields.retain(|key, _| {
            !key.starts_with("last")
                && !matches!(
                    key.as_str(),
                    "timeOffset" | "duration" | "videoId" | "video_id"
                )
        });
        if let Some(video) = episode {
            fields.insert("lastVideoId".to_owned(), video.into());
        }
    }
    item
}

pub(super) fn refresh_library_view(state: &mut RendererState) {
    state.library.query = state.library_query.clone();
    state.library.sort_by = state.library_sort.clone();
    state.library.content_type = state.library_type.clone();
    state.library.list_view = state.library_list;
    state.library.downloads_open = state.library_downloads;
    state.library.downloads_group = state.library_download_group.clone();
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
                "onHold": library.get("onHold"),
                "favorites": library.get("liked"),
                "progress": library.get("progress"),
                "tab": state.library_tab.core_tab_key(),
                "query": state.library_query,
                "sortBy": state.library_sort,
                "type": state.library_type,
                "source": state.library.source.clone(),
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
    if let Some((index, action)) = fluxa_ui::addon_action(node) {
        let url = fluxa_ui::addon_transport_url(settings.addons.get(index)?)?;
        return Some(match action {
            0 => json!({"type":"addonMoveRequested", "transportUrl":url, "offset":-1}),
            1 => json!({"type":"addonMoveRequested", "transportUrl":url, "offset":1}),
            2 => json!({"type":"addonInstallRequested", "transportUrl":url, "forceRefresh":true}),
            4 => json!({
                "type":"addonToggleRequested",
                "transportUrl":url,
                "enabled":!fluxa_ui::addon_enabled(settings.addons.get(index)?),
            }),
            _ => json!({"type":"addonRemoveRequested", "transportUrl":url}),
        });
    }
    if node == fluxa_ui::NODE_SETTINGS_PLUGIN_CONFIG_SAVE {
        let scraper = settings
            .plugins
            .get("scrapers")?
            .as_array()?
            .get(settings.plugin_config?)?;
        let values: serde_json::Map<String, Value> = fluxa_ui::plugin_config_fields(scraper)
            .iter()
            .zip(&settings.plugin_config_fields)
            .filter_map(|(field, value)| {
                Some((
                    field.get("name")?.as_str()?.to_owned(),
                    Value::String(value.trim().to_owned()),
                ))
            })
            .collect();
        return Some(json!({
            "type":"pluginScraperSettingsUpdated",
            "scraperId":scraper.get("id")?.as_str()?,
            "settings":values,
        }));
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
        ..fluxa_ui::NODE_SETTINGS_PLUGIN_REPOSITORY_BASE + 20)
        .contains(&node)
    {
        let index = (node - fluxa_ui::NODE_SETTINGS_PLUGIN_REPOSITORY_BASE) as usize;
        let url = repositories?.get(index)?.get("manifestUrl")?.as_str()?;
        return Some(json!({"type":"pluginRepositoryRemoveRequested", "manifestUrl":url}));
    }
    if (fluxa_ui::NODE_SETTINGS_PLUGIN_GROUP_BASE..fluxa_ui::NODE_SETTINGS_PLUGIN_GROUP_BASE + 20)
        .contains(&node)
    {
        let index = (node - fluxa_ui::NODE_SETTINGS_PLUGIN_GROUP_BASE) as usize;
        let repository = repositories?.get(index)?;
        let url = repository.get("manifestUrl")?.as_str()?;
        let enabled = !repository
            .get("groupResults")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        return Some(
            json!({"type":"pluginRepositoryGroupToggled", "manifestUrl":url, "enabled":enabled}),
        );
    }
    if (fluxa_ui::NODE_SETTINGS_PLUGIN_REFRESH_BASE
        ..fluxa_ui::NODE_SETTINGS_PLUGIN_REFRESH_BASE + 20)
        .contains(&node)
    {
        let index = (node - fluxa_ui::NODE_SETTINGS_PLUGIN_REFRESH_BASE) as usize;
        let url = repositories?.get(index)?.get("manifestUrl")?.as_str()?;
        return Some(json!({"type":"pluginRepositoryAddRequested", "manifestUrl":url}));
    }
    if (fluxa_ui::NODE_SETTINGS_PLUGIN_SCRAPER_BASE
        ..fluxa_ui::NODE_SETTINGS_PLUGIN_SCRAPER_BASE + 30)
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

fn open_plugin_config(state: &mut RendererState, index: usize) {
    let settings = &mut state.settings;
    if settings.plugin_config == Some(index) {
        settings.plugin_config = None;
        return;
    }
    let Some(scraper) = settings
        .plugins
        .get("scrapers")
        .and_then(Value::as_array)
        .and_then(|items| items.get(index))
    else {
        return;
    };
    let mut fields: [String; fluxa_ui::PLUGIN_CONFIG_FIELDS] = Default::default();
    for (slot, field) in fluxa_ui::plugin_config_fields(scraper).iter().enumerate() {
        let name = field
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let value = scraper
            .pointer(&format!("/settings/{name}"))
            .or_else(|| field.get("default"));
        fields[slot] = match value {
            Some(Value::String(text)) => text.clone(),
            Some(Value::Null) | None => String::new(),
            Some(other) => other.to_string(),
        };
    }
    settings.plugin_config_fields = fields;
    settings.plugin_config = Some(index);
}

pub(super) fn edit_text(
    state: &mut RendererState,
    node: u64,
    edit: impl FnOnce(&mut String),
) -> bool {
    let route = state.route;
    let text = match node {
        fluxa_ui::NODE_LIBRARY_SEARCH if route == Route::Library => &mut state.library_query,
        fluxa_ui::NODE_DISCOVER_SEARCH if route == Route::Discover => &mut state.discover.query,
        fluxa_ui::NODE_SIDEBAR_SEARCH => &mut state.discover.query,
        fluxa_ui::NODE_SETTINGS_SEARCH if route == Route::Settings => &mut state.settings.search,
        fluxa_ui::NODE_SETTINGS_ADDON_URL if route == Route::Settings => {
            &mut state.settings.addon_url
        }
        fluxa_ui::NODE_SETTINGS_PLUGIN_URL if route == Route::Settings => {
            &mut state.settings.plugin_url
        }
        fluxa_ui::NODE_SETTINGS_WATCH_CODE if route == Route::Settings => {
            &mut state.settings.watch_code
        }
        _ if route == Route::Settings && fluxa_ui::server_input(node).is_some() => {
            &mut state.settings.server_fields[fluxa_ui::server_input(node).unwrap_or_default()]
        }
        _ if route == Route::Settings && fluxa_ui::collection_input(node).is_some() => {
            &mut state.settings.collection_fields
                [fluxa_ui::collection_input(node).unwrap_or_default()]
        }
        _ if route == Route::Settings && fluxa_ui::badge_input(node).is_some() => {
            &mut state.settings.badge_fields[fluxa_ui::badge_input(node).unwrap_or_default()]
        }
        _ if route == Route::Settings && fluxa_ui::plugin_config_input(node).is_some() => {
            &mut state.settings.plugin_config_fields
                [fluxa_ui::plugin_config_input(node).unwrap_or_default()]
        }
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
        fluxa_ui::NODE_DISCOVER_SEARCH => schedule_discover_search(state),
        fluxa_ui::NODE_SIDEBAR_SEARCH => search_from_sidebar(state),
        fluxa_ui::NODE_SETTINGS_WATCH_CODE => {
            let code = &mut state.settings.watch_code;
            *code = code
                .chars()
                .filter(char::is_ascii_alphanumeric)
                .take(6)
                .collect::<String>()
                .to_uppercase();
        }
        _ => {}
    }
    true
}

pub(super) fn focused_text(state: &RendererState) -> Option<String> {
    node_text(state, state.ui.focused()?)
}

pub(super) fn node_text(state: &RendererState, node: u64) -> Option<String> {
    let route = state.route;
    Some(match node {
        fluxa_ui::NODE_LIBRARY_SEARCH if route == Route::Library => state.library_query.clone(),
        fluxa_ui::NODE_DISCOVER_SEARCH if route == Route::Discover => state.discover.query.clone(),
        fluxa_ui::NODE_SIDEBAR_SEARCH => state.discover.query.clone(),
        fluxa_ui::NODE_SETTINGS_SEARCH if route == Route::Settings => state.settings.search.clone(),
        fluxa_ui::NODE_SETTINGS_ADDON_URL if route == Route::Settings => {
            state.settings.addon_url.clone()
        }
        fluxa_ui::NODE_SETTINGS_PLUGIN_URL if route == Route::Settings => {
            state.settings.plugin_url.clone()
        }
        fluxa_ui::NODE_SETTINGS_WATCH_CODE if route == Route::Settings => {
            state.settings.watch_code.clone()
        }
        _ if route == Route::Settings && fluxa_ui::server_input(node).is_some() => {
            state.settings.server_fields[fluxa_ui::server_input(node)?].clone()
        }
        _ if route == Route::Settings && fluxa_ui::collection_input(node).is_some() => {
            state.settings.collection_fields[fluxa_ui::collection_input(node)?].clone()
        }
        _ if route == Route::Settings && fluxa_ui::badge_input(node).is_some() => {
            state.settings.badge_fields[fluxa_ui::badge_input(node)?].clone()
        }
        _ if route == Route::Settings && fluxa_ui::plugin_config_input(node).is_some() => {
            state.settings.plugin_config_fields[fluxa_ui::plugin_config_input(node)?].clone()
        }
        _ if route == Route::Settings => {
            state.settings.poster_fields[fluxa_ui::poster_field(node)?].clone()
        }
        _ => return None,
    })
}

fn save_left_fields(state: &mut RendererState) {
    let focused = state.ui.focused();
    for (index, field) in fluxa_ui::POSTER_FIELDS.iter().enumerate() {
        let value = state.settings.poster_fields[index].trim().to_owned();
        if focused == Some(field.input)
            || state.settings.str_value(field.key).unwrap_or("") == value
        {
            continue;
        }
        if let Some(values) = state.settings.values.as_object_mut() {
            values.insert(field.key.to_owned(), Value::String(value.clone()));
        }
        state
            .pending_native_actions
            .push(NativeAction::CoreCommand {
                command: json!({"type":"settingsChanged", "key":field.key, "value":value}),
            });
    }
}

pub(super) fn remember_actions(state: &mut RendererState, actions: Vec<UiAction>) {
    save_left_fields(state);
    recent_searches::save_left_query(state);
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
        let minimized = state.player.as_ref().is_some_and(|player| player.minimized);
        if state.player.is_some() && (!minimized || node.is_some_and(player::is_mini_node)) {
            if let Some(node) = node {
                player::activate(state, node);
            } else if matches!(action, UiAction::Back) {
                player::leave(state);
            }
            continue;
        }
        if let Some(node) = node {
            let episode = state.route == Route::Detail
                && (fluxa_ui::NODE_DETAIL_EPISODE_BASE..fluxa_ui::NODE_DETAIL_EPISODE_BASE + 1000)
                    .contains(&node);
            if (episode
                || matches!(
                    node,
                    fluxa_ui::NODE_PLAY | fluxa_ui::NODE_DETAIL_PLAY | fluxa_ui::NODE_SHORTS_PLAY
                ))
                && playback_blocked(state)
            {
                show_notice(state, "playback.no_sources");
                continue;
            }
            if profiles::activate_node(state, node)
                || shortcuts::activate_node(state, node)
                || updates::activate_node(state, node)
                || stream_badges::activate_node(state, node)
                || collections::activate_node(state, node)
                || watch_together::activate_node(state, node)
                || ambient::activate_node(state, node)
                || hero_catalogs::activate_node(state, node)
                || detail_sections::activate_node(state, node)
                || downloads::activate_node(state, node)
            {
                continue;
            }
            let request = state
                .rendered_layout
                .as_ref()
                .filter(|(route, _)| *route == state.route)
                .and_then(|(_, layout)| layout.choices.iter().find(|(id, _)| *id == node))
                .map(|(_, request)| request.clone());
            if let Some(request) = request {
                card_menu::open_choice(state, request);
                continue;
            }
            if state.route == Route::Home && node == fluxa_ui::NODE_HOME_ADD_ADDONS {
                state.settings.active_section = fluxa_ui::addons_page();
                state.settings.section_open = true;
                state.settings.page_open = true;
                state.settings.search.clear();
                state.screen_scroll_offsets.remove(&Route::Settings);
                state.pending_native_actions.push(NativeAction::Navigate {
                    destination: Route::Settings,
                });
                reset_ui(state);
                continue;
            }
            if node == fluxa_ui::NODE_LIBRARY {
                state.library_downloads = false;
                state.library_download_group = None;
                refresh_library_view(state);
            }
            if node == fluxa_ui::NODE_SIDEBAR_DOWNLOADS {
                state.library_downloads = true;
                state.library_download_group = None;
                refresh_library_view(state);
            }
            if (fluxa_ui::NODE_LIBRARY_SECTION_BASE..fluxa_ui::NODE_LIBRARY_SECTION_BASE + 3)
                .contains(&node)
            {
                let section = node - fluxa_ui::NODE_LIBRARY_SECTION_BASE;
                state.library_downloads = section == 2;
                state.library_download_group = None;
                refresh_library_view(state);
                let destination = if section == 1 {
                    Route::Calendar
                } else {
                    Route::Library
                };
                if destination != state.route {
                    state
                        .pending_native_actions
                        .push(NativeAction::Navigate { destination });
                }
                state.screen_scroll_offsets.remove(&Route::Library);
                reset_ui(state);
                continue;
            }
            if state.route == Route::Library && node == fluxa_ui::NODE_LIBRARY_VIEW {
                state.library_list = !state.library_list;
                refresh_library_view(state);
                state.screen_scroll_offsets.remove(&Route::Library);
                reset_ui(state);
                continue;
            }
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
            if state.route == Route::Folder
                && node >= fluxa_ui::NODE_FOLDER_TAB_BASE
                && let index = (node - fluxa_ui::NODE_FOLDER_TAB_BASE) as usize
                && index < state.folder.tabs.len()
            {
                state.folder_tab = index;
                state.screen_scroll_offsets.remove(&Route::Folder);
                reset_ui(state);
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
                && let Some(color) = node
                    .checked_sub(fluxa_ui::NODE_SETTINGS_SWATCH_BASE)
                    .and_then(|slot| fluxa_ui::accent_swatches().get(slot as usize))
            {
                state
                    .pending_native_actions
                    .push(NativeAction::SettingsChange {
                        key: "accentColorArgb".to_owned(),
                        value: Value::String((*color).to_owned()),
                    });
                continue;
            }
            if state.route == Route::Settings
                && let Some(row) = node
                    .checked_sub(fluxa_ui::NODE_SETTINGS_ROW_BASE)
                    .and_then(|index| fluxa_ui::settings_row_by_index(index as usize))
                && (row.key == "appIcon"
                    || logical_viewport(state).is_compact()
                    || logical_viewport(state).is_tv())
                && !row.options.is_empty()
                && row.key != "accentColorArgb"
            {
                card_menu::open_setting(state, row);
                continue;
            }
            if state.route == Route::Settings
                && (logical_viewport(state).is_compact() || logical_viewport(state).is_tv())
                && let Some(index) = node.checked_sub(fluxa_ui::NODE_SETTINGS_ACCOUNT_SOURCE_BASE)
                && (index < 2 || index == 3)
            {
                card_menu::open_source(state, index as usize);
                continue;
            }
            if state.route == Route::Settings
                && (fluxa_ui::NODE_SETTINGS_PLUGIN_CONFIG_BASE
                    ..fluxa_ui::NODE_SETTINGS_PLUGIN_CONFIG_BASE + 30)
                    .contains(&node)
            {
                let index = (node - fluxa_ui::NODE_SETTINGS_PLUGIN_CONFIG_BASE) as usize;
                open_plugin_config(state, index);
                reset_ui(state);
                continue;
            }
            if state.route == Route::Settings
                && let Some(url) = fluxa_ui::settings_link_url(node)
            {
                state.open_url = Some(url.to_owned());
                continue;
            }
            if state.route == Route::Settings
                && node == fluxa_ui::NODE_SETTINGS_BACK
                && (state.settings.section_open || state.settings.page_open)
            {
                close_settings_section(state);
                continue;
            }
            if state.route == Route::Settings
                && let Some(index) = fluxa_ui::settings_page_for_node(node)
            {
                state.settings.active_section = index;
                state.settings.section_open = true;
                state.settings.page_open = node < fluxa_ui::NODE_SETTINGS_SECTION_BASE;
                state.settings.search.clear();
                state.screen_scroll_offsets.remove(&Route::Settings);
                reset_ui(state);
                continue;
            }
            if state.route == Route::Settings
                && let Some(action_json) = settings_action_json(node, &state.settings)
            {
                if node == fluxa_ui::NODE_SETTINGS_ADDON_INSTALL {
                    state.addon_install = Some(state.settings.addon_url.trim().to_owned());
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
            if state.route == Route::Detail
                && (fluxa_ui::NODE_DETAIL_SEASON_BASE..fluxa_ui::NODE_DETAIL_EPISODE_BASE)
                    .contains(&node)
            {
                state.detail.selected_season =
                    Some((node - fluxa_ui::NODE_DETAIL_SEASON_BASE) as i64);
                state.detail.row_scroll_offsets[1] = 0.0;
                reset_ui(state);
                continue;
            }
            if state.route == Route::Detail && node == fluxa_ui::NODE_DETAIL_SHUFFLE {
                if let Some(item) = player::start_shuffle(state) {
                    state
                        .pending_native_actions
                        .push(NativeAction::StartPlayback { item });
                }
                continue;
            }
            let (node, hero) = if state.route == Route::Shorts {
                let node = match node {
                    fluxa_ui::NODE_SHORTS_PLAY => NODE_PLAY,
                    fluxa_ui::NODE_SHORTS_WATCHLIST => fluxa_ui::NODE_HERO_WATCHLIST,
                    fluxa_ui::NODE_SHORTS_INFO => NODE_MORE_INFO,
                    other => other,
                };
                (node, shorts::current(state))
            } else {
                (
                    node,
                    state
                        .gpu
                        .as_ref()
                        .and_then(|gpu| fluxa_ui::active_home_hero(&gpu.egui_context, &state.home)),
                )
            };
            if let Some(native_action) = native_action_for_node(
                node,
                &state.home,
                &state.library,
                state.library_tab,
                &state.discover,
                &state.folder,
                state.folder_tab,
                &state.calendar,
                &state.detail,
                &state.settings,
                state.route,
                hero,
            ) {
                let native_action = match native_action {
                    NativeAction::Detail {
                        id,
                        item_type,
                        preview,
                    } if state.route != Route::Detail
                        && state.home.form_factor == fluxa_ui::UiFormFactorJson::Desktop =>
                    {
                        NativeAction::DetailModal {
                            id,
                            item_type,
                            preview,
                        }
                    }
                    action => action,
                };
                if let NativeAction::Detail { preview, .. }
                | NativeAction::DetailModal { preview, .. } = &native_action
                {
                    state.backdrop_prefetch = ["background", "poster"]
                        .into_iter()
                        .find_map(|key| preview.get(key).and_then(Value::as_str))
                        .map(ToOwned::to_owned);
                }
                state.pending_native_actions.push(native_action);
            }
        }
        if matches!(action, UiAction::Back) {
            if state.profiles.is_some() {
                profiles::back(state);
            } else if state.route == Route::Calendar && state.calendar.selected_day.is_some() {
                state.calendar.selected_day = None;
                reset_ui(state);
            } else if state.route == Route::Settings
                && (state.settings.section_open || state.settings.page_open)
            {
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
    let live = session.snapshot();
    let snapshot = crate::preview::fixture_snapshot(&live).unwrap_or(live);
    let unchanged = state
        .core_snapshot
        .as_deref()
        .is_some_and(|old| !projection_inputs_changed(old, &snapshot));
    if let Some(old) = state.core_snapshot.replace(snapshot) {
        retire(old);
    }
    if unchanged {
        return;
    }
    state.core_snapshot_revision = state.core_snapshot_revision.wrapping_add(1);
    sync_home_from_core_snapshot(state);
}

const UNPROJECTED_SLICES: [&str; 5] = ["trailer", "lookup", "auth", "watchTogether", "_runtime"];

fn projection_inputs_changed(old: &Value, new: &Value) -> bool {
    let (Some(old), Some(new)) = (old.as_object(), new.as_object()) else {
        return true;
    };
    let relevant = |key: &&String| !UNPROJECTED_SLICES.contains(&key.as_str());
    old.keys().filter(relevant).count() != new.keys().filter(relevant).count()
        || new
            .iter()
            .filter(|(key, _)| relevant(key))
            .any(|(key, value)| old.get(key) != Some(value))
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

pub(super) fn folder_commands(
    snapshot: Option<&Value>,
    folder_id: &str,
    profile: &Value,
) -> Vec<Value> {
    let mut commands = vec![json!({
        "type": "navigationRequested",
        "route": Route::Folder.as_str(),
        "params": {"folderId": folder_id},
    })];
    let categories = snapshot
        .and_then(|snapshot| snapshot.pointer("/home/categories"))
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default();
    for category in categories {
        let is_source = category.get("type").and_then(Value::as_str)
            == Some("collection_folder_source")
            && category.get("folderId").and_then(Value::as_str) == Some(folder_id);
        let loaded = category
            .get("items")
            .and_then(Value::as_array)
            .is_some_and(|items| !items.is_empty());
        if !is_source || loaded {
            continue;
        }
        commands.push(json!({
            "type": "catalogPageRequested",
            "categoryId": category.get("id"),
            "transportUrl": category.get("transportUrl"),
            "contentType": category.get("contentType"),
            "catalogId": category.get("catalogId"),
            "skip": 0,
            "genre": category.get("genre"),
            "remoteSource": category.get("remoteSource"),
            "profile": profile,
        }));
    }
    commands
}

pub(super) fn navigation(route: Route) -> Value {
    json!({"type": "navigationRequested", "route": route.as_str(), "params": {}})
}

pub(super) fn session_commands(action: &NativeAction, profile: &Value) -> Option<Vec<Value>> {
    let commands = match action {
        NativeAction::CoreCommand { command } => vec![command.clone()],
        NativeAction::LoadMore { .. } => Vec::new(),
        NativeAction::Folder { .. }
        | NativeAction::DetailModal { .. }
        | NativeAction::OpenDetailPage => {
            return None;
        }
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
            Route::Shorts => vec![navigation(Route::Shorts)],
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
            json!({
                "type": "detailSecondaryRequested",
                "id": id,
                "contentType": item_type,
                "language": profile_language(profile),
                "profile": profile,
            }),
        ],
        NativeAction::ToggleWatchlist { item } => {
            vec![json!({"type": "toggleWatchlistRequested", "item": item, "profile": profile})]
        }
        NativeAction::ToggleFavorite { item } => {
            vec![json!({"type": "toggleLibraryStatusRequested", "list": "liked", "item": item})]
        }
        NativeAction::SettingsChange { key, value } => {
            vec![json!({"type": "settingsChanged", "key": key, "value": value})]
        }
        NativeAction::StartPlayback { item } => {
            vec![player::direct_playback_command(item, profile)]
        }
        NativeAction::PlayTrailer { .. }
        | NativeAction::SettingsSection { .. }
        | NativeAction::AccountToggle { .. }
        | NativeAction::LogOut
        | NativeAction::MediaServer { .. }
        | NativeAction::MediaCommand { .. }
        | NativeAction::OpenFile { .. }
        | NativeAction::OauthCallback { .. }
        | NativeAction::OpenLink { .. }
        | NativeAction::Search { .. } => return None,
    };
    Some(commands)
}

pub(super) fn route_actions_to_session(state: &mut RendererState) {
    let Some(session) = state.session.as_ref() else {
        start_playback_without_session(state);
        return;
    };
    if home_refresh_due(state) {
        let profile = session.active_profile();
        let command = json!({
            "type": "refreshContinueWatchingRequested",
            "profile": profile,
            "language": profile_language(&profile),
            "source": "poll",
        });
        state.home_refreshed_at = Instant::now();
        if let Err(error) = session.dispatch(command) {
            host_log(format!("core dispatch failed: {error}"));
        }
    }
    if state.pending_native_actions.is_empty() {
        return;
    }
    let profile = session.active_profile();
    let mut unhandled = Vec::new();
    let mut open_profiles = false;
    let mut log_out = false;
    let mut toggles = Vec::new();
    let mut servers = Vec::new();
    let mut callbacks = Vec::new();
    let mut files = Vec::new();
    let mut media = Vec::new();
    let mut searched = false;
    for action in std::mem::take(&mut state.pending_native_actions) {
        let action = match action {
            NativeAction::OpenLink { url } => {
                match core_value("deepLinkAction", json!({"url": url}))
                    .and_then(|action| serde_json::from_value(action).ok())
                {
                    Some(action) => action,
                    None => continue,
                }
            }
            NativeAction::Search { query } => {
                let command = json!({
                    "type": "searchRequested",
                    "query": query.trim(),
                    "language": state.discover.language,
                });
                state.discover.query = query;
                searched = true;
                state
                    .pending_native_actions
                    .push(NativeAction::CoreCommand { command });
                NativeAction::Navigate {
                    destination: Route::Discover,
                }
            }
            action => action,
        };
        if let NativeAction::MediaCommand { command, value } = action {
            media.push((command, value));
            continue;
        }
        if let NativeAction::AccountToggle { provider } = action {
            toggles.push(provider);
            continue;
        }
        if matches!(action, NativeAction::LogOut) {
            log_out = true;
            continue;
        }
        if let NativeAction::MediaServer { node } = action {
            servers.push(node);
            continue;
        }
        if let NativeAction::OauthCallback { url } = action {
            callbacks.push(url);
            continue;
        }
        if matches!(&action, NativeAction::Navigate { destination } if *destination == Route::Profiles)
        {
            open_profiles = true;
            continue;
        }
        if let NativeAction::PlayTrailer { item, urls } = &action {
            if let Some(video) = state.video.as_mut() {
                video.stop();
            }
            state.player = Some(player::PlayerSession::trailer(item.clone(), urls));
            state.screen_scroll_offsets.remove(&Route::Player);
            state.ui = UiTree::default();
            continue;
        }
        if let NativeAction::Folder { id } = &action {
            let commands = folder_commands(state.core_snapshot.as_deref(), id, &profile);
            state.folder_tab = 0;
            state.screen_scroll_offsets.remove(&Route::Folder);
            for command in commands {
                if let Err(error) = session.dispatch(command) {
                    host_log(format!("core dispatch failed: {error}"));
                }
            }
            continue;
        }
        if let NativeAction::DetailModal {
            id,
            item_type,
            preview,
        } = &action
        {
            let load = NativeAction::Detail {
                id: id.clone(),
                item_type: item_type.clone(),
                preview: preview.clone(),
            };
            state.detail_modal = true;
            for command in session_commands(&load, &profile)
                .unwrap_or_default()
                .into_iter()
                .skip(1)
            {
                if let Err(error) = session.dispatch(command) {
                    host_log(format!("core dispatch failed: {error}"));
                }
            }
            continue;
        }
        if matches!(action, NativeAction::OpenDetailPage) {
            state.detail_modal = false;
            if let Err(error) = session.dispatch(navigation(Route::Detail)) {
                host_log(format!("core dispatch failed: {error}"));
            }
            continue;
        }
        if let NativeAction::OpenFile { uri, title } = action {
            files.push((uri, title));
            continue;
        }
        if let NativeAction::StartPlayback { item } = &action {
            player::keep_shuffle_for(&mut state.shuffle, item);
            let mut player =
                player::PlayerSession::new(item.clone()).with_logo_hint(known_logo(state, item));
            player.pick_manually(state.core_snapshot.as_deref());
            player.cast_target = state.cast_next.take();
            player.stale = session.snapshot().get("player").cloned();
            if let Some(video) = state.video.as_mut() {
                video.stop();
            }
            state.player = Some(player);
            state.screen_scroll_offsets.remove(&Route::Player);
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
    if searched {
        recent_searches::refresh(state);
        reset_ui(state);
        state.focus_hint = None;
    }
    for (command, value) in media {
        player::media_command(state, &command, value);
    }
    for provider in toggles {
        accounts::toggle(state, &provider);
    }
    for node in servers {
        accounts::media_server(state, node);
    }
    for url in callbacks {
        accounts::finish_redirect(state, &url);
    }
    for (uri, title) in files {
        let command = player::open_file(state, &uri, &title);
        if let Some(session) = state.session.as_ref()
            && let Err(error) = session.dispatch(command)
        {
            host_log(format!("core dispatch failed: {error}"));
        }
    }
    if open_profiles {
        reset_ui(state);
        profiles::open(state);
    }
    if log_out {
        reset_ui(state);
        welcome::log_out(state);
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
        let mut player =
            player::PlayerSession::new(item.clone()).with_logo_hint(known_logo(state, item));
        player.pick_manually(state.core_snapshot.as_deref());
        player.cast_target = state.cast_next.take();
        player.stale = state
            .core_snapshot
            .as_ref()
            .and_then(|snapshot| snapshot.get("player"))
            .cloned();
        state.player = Some(player);
        state.screen_scroll_offsets.remove(&Route::Player);
        reset_ui(state);
        state.pending_native_actions[index] = NativeAction::CoreCommand { command };
    }
}

fn close_settings_section(state: &mut RendererState) {
    if state.settings.page_open {
        state.settings.page_open = false;
        state.settings.active_section = fluxa_ui::category_pages(state.settings.active_section)
            .next()
            .unwrap_or_default();
    } else {
        state.settings.section_open = false;
    }
    state.screen_scroll_offsets.remove(&Route::Settings);
    reset_ui(state);
}

pub(super) fn home_refresh_due(state: &RendererState) -> bool {
    state.window_focused
        && state.route == Route::Home
        && state.player.is_none()
        && state.home_refreshed_at.elapsed() >= HOME_REFRESH_INTERVAL
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trailer_resolution_does_not_reproject() {
        let old = json!({"home": {"categories": [1, 2]}, "trailer": {"resolutions": {}}});
        let new = json!({"home": {"categories": [1, 2]}, "trailer": {"resolutions": {"a": 1}}});
        assert!(!projection_inputs_changed(&old, &new));
    }

    #[test]
    fn new_catalog_page_reprojects() {
        let old = json!({"home": {"categories": [1]}, "trailer": {}});
        let new = json!({"home": {"categories": [1, 2]}, "trailer": {}});
        assert!(projection_inputs_changed(&old, &new));
        assert!(projection_inputs_changed(&old, &json!({"trailer": {}})));
    }
}
