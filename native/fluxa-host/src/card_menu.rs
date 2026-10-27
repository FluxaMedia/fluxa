use super::*;
use fluxa_ui::{ActionMenuItem, ActionMenuOutcome, HomeCard, localized};

const LONG_PRESS: Duration = Duration::from_millis(450);

#[derive(Clone, Copy, PartialEq)]
enum Entry {
    Play,
    PlayOnTv,
    Details,
    Watchlist { saved: bool },
    Watched { watched: bool },
    Rewatch,
    ClearProgress,
}

pub(super) enum Then {
    Stream(Value),
    Card(HomeCard),
}

struct MarkEntry {
    icon: &'static str,
    label: &'static str,
    video_ids: Vec<String>,
    watched: bool,
}

enum Target {
    Mark {
        title: String,
        entries: Vec<MarkEntry>,
    },
    Card {
        card: HomeCard,
        entries: Vec<Entry>,
    },
    Setting(&'static fluxa_ui::SettingsRow),
    Source {
        key: &'static str,
        title: &'static str,
        count: usize,
    },
    Disconnect(String),
    QuitDownloads,
    Stream {
        stream: Value,
        index: usize,
        entries: Vec<crate::stream_menu::Entry>,
    },
    Choice(fluxa_ui::ChoiceRequest),
    CastDevices {
        then: Then,
        count: usize,
    },
}

impl Target {
    fn len(&self) -> usize {
        match self {
            Target::Card { entries, .. } => entries.len(),
            Target::Mark { entries, .. } => entries.len(),
            Target::Setting(row) => row.options.len(),
            Target::Source { count, .. } => *count,
            Target::Disconnect(_) => 1,
            Target::QuitDownloads => 2,
            Target::Stream { entries, .. } => entries.len(),
            Target::Choice(request) => request.options.len(),
            Target::CastDevices { count, .. } => (*count).max(1),
        }
    }
}

pub(super) struct CardMenu {
    target: Target,
    selected: Option<usize>,
    anchor: Pos2,
    scroll: f32,
    drag: Option<(f32, f32)>,
    serial: u64,
}

pub(super) struct MenuView {
    pub(super) title: String,
    pub(super) items: Vec<ActionMenuItem>,
    pub(super) selected: Option<usize>,
    pub(super) anchor: Pos2,
    pub(super) scroll: f32,
    pub(super) serial: u64,
}

fn card_for_node(state: &RendererState, node: u64) -> Option<HomeCard> {
    let index = node.checked_sub(NODE_CARD_BASE)? as usize;
    let card = match state.route {
        Route::Home => state.home.card_at(index),
        Route::Library => state.library.cards(state.library_tab).get(index),
        Route::Discover => state.discover.results.get(index),
        Route::Folder => state.folder.card_at(state.folder_tab, index),
        Route::Detail => {
            let range = fluxa_ui::NODE_DETAIL_SIMILAR_BASE..fluxa_ui::NODE_DETAIL_CAST_BASE;
            if !range.contains(&node) {
                return None;
            }
            state
                .detail
                .similar
                .get((node - fluxa_ui::NODE_DETAIL_SIMILAR_BASE) as usize)
        }
        _ => None,
    }?;
    card.id.as_ref()?;
    Some(card.clone())
}

pub(super) fn open(state: &mut RendererState, node: u64, anchor: Pos2, keyboard: bool) -> bool {
    if state.profiles.is_some() || state.welcome.is_some() {
        return false;
    }
    if state.player_full() {
        return open_stream(state, node, anchor, keyboard);
    }
    if state.route == Route::Detail && open_mark(state, node, anchor, keyboard) {
        return true;
    }
    let Some(card) = card_for_node(state, node) else {
        return false;
    };
    let id = card.id.clone().unwrap_or_default();
    let personal = state.library.personal.get(&id);
    let mut entries = vec![Entry::Play];
    if cfg!(not(target_arch = "wasm32")) {
        entries.push(Entry::PlayOnTv);
    }
    entries.push(Entry::Details);
    entries.push(Entry::Watchlist {
        saved: personal.is_some_and(|personal| personal.saved),
    });
    if card.item_type.as_deref() == Some("movie") {
        entries.push(Entry::Watched {
            watched: personal.is_some_and(|personal| personal.watched),
        });
    }
    let simkl_rewatch = state
        .settings
        .value("integrationLibrarySource")
        .and_then(Value::as_str)
        == Some("simkl");
    let simkl_rewatch = simkl_rewatch
        && !state.session.as_ref().is_some_and(|session| {
            let profile = session.active_profile();
            fluxa_effects::simkl_plan_lacks_rewatch(
                session.storage(),
                profile.get("id").and_then(Value::as_str).unwrap_or("guest"),
            )
        });
    if simkl_rewatch && personal.is_some_and(|personal| personal.watched) {
        entries.push(Entry::Rewatch);
    }
    if card.progress > 0.0 || personal.is_some_and(|personal| personal.progress > 0.0) {
        entries.push(Entry::ClearProgress);
    }
    state.menu_serial += 1;
    state.card_menu = Some(CardMenu {
        target: Target::Card { card, entries },
        selected: keyboard.then_some(0),
        anchor,
        scroll: 0.0,
        drag: None,
        serial: state.menu_serial,
    });
    true
}

fn open_mark(state: &mut RendererState, node: u64, anchor: Pos2, keyboard: bool) -> bool {
    let detail = &state.detail;
    if !detail.is_series() {
        return false;
    }
    let ids = |keep: &dyn Fn(&fluxa_ui::DetailEpisode) -> bool| -> Vec<String> {
        detail
            .episodes
            .iter()
            .filter(|episode| keep(episode))
            .map(|episode| episode.id.clone())
            .collect()
    };
    let (title, entries) = if (fluxa_ui::NODE_DETAIL_EPISODE_BASE
        ..fluxa_ui::NODE_DETAIL_EPISODE_BASE + 1000)
        .contains(&node)
    {
        let Some(episode) = detail
            .episodes
            .get((node - fluxa_ui::NODE_DETAIL_EPISODE_BASE) as usize)
        else {
            return false;
        };
        let season = episode.season;
        let number = episode.number;
        let through =
            ids(&|other| other.season > 0 && (other.season, other.number) <= (season, number));
        let mut entries = vec![MarkEntry {
            icon: if episode.watched {
                "EyeOff"
            } else {
                "CircleCheck"
            },
            label: if episode.watched {
                "menu.mark_unwatched"
            } else {
                "menu.mark_watched"
            },
            video_ids: vec![episode.id.clone()],
            watched: !episode.watched,
        }];
        entries.push(MarkEntry {
            icon: "CircleCheck",
            label: "detail.mark_episode_through_watched",
            video_ids: through.clone(),
            watched: true,
        });
        entries.push(MarkEntry {
            icon: "EyeOff",
            label: "detail.mark_episode_through_unwatched",
            video_ids: through,
            watched: false,
        });
        (episode.title.clone(), entries)
    } else if (fluxa_ui::NODE_DETAIL_SEASON_BASE..fluxa_ui::NODE_DETAIL_EPISODE_BASE)
        .contains(&node)
    {
        let season = if node == fluxa_ui::NODE_DETAIL_SEASON_BASE {
            detail.current_season()
        } else {
            (node - fluxa_ui::NODE_DETAIL_SEASON_BASE) as i64
        };
        let all_watched = detail
            .episodes
            .iter()
            .filter(|episode| episode.season == season)
            .all(|episode| episode.watched);
        let in_season = ids(&|episode| episode.season == season);
        let through = ids(&|episode| episode.season > 0 && episode.season <= season);
        let mut entries = vec![MarkEntry {
            icon: if all_watched { "EyeOff" } else { "CircleCheck" },
            label: if all_watched {
                "detail.mark_season_unwatched"
            } else {
                "detail.mark_season_watched"
            },
            video_ids: in_season,
            watched: !all_watched,
        }];
        if season > 1 {
            entries.push(MarkEntry {
                icon: "CircleCheck",
                label: "detail.mark_through_season_watched",
                video_ids: through.clone(),
                watched: true,
            });
            entries.push(MarkEntry {
                icon: "EyeOff",
                label: "detail.mark_through_season_unwatched",
                video_ids: through,
                watched: false,
            });
        }
        (
            localized("format.season_number", state.settings.language())
                .replace("%s", &season.to_string()),
            entries,
        )
    } else {
        return false;
    };
    if entries.iter().all(|entry| entry.video_ids.is_empty()) {
        return false;
    }
    state.menu_serial += 1;
    state.card_menu = Some(CardMenu {
        target: Target::Mark { title, entries },
        selected: keyboard.then_some(0),
        anchor,
        scroll: 0.0,
        drag: None,
        serial: state.menu_serial,
    });
    true
}

fn open_stream(state: &mut RendererState, node: u64, anchor: Pos2, keyboard: bool) -> bool {
    let range = fluxa_ui::NODE_PLAYER_SOURCE_BASE..fluxa_ui::NODE_PLAYER_SOURCE_BASE + 1000;
    if !range.contains(&node) {
        return false;
    }
    let index = (node - fluxa_ui::NODE_PLAYER_SOURCE_BASE) as usize;
    let Some(stream) = state
        .player
        .as_ref()
        .and_then(|player| player.listed_stream(index))
    else {
        return false;
    };
    let entries = crate::stream_menu::entries(&stream);
    if entries.is_empty() {
        return false;
    }
    state.menu_serial += 1;
    state.card_menu = Some(CardMenu {
        target: Target::Stream {
            stream,
            index,
            entries,
        },
        selected: keyboard.then_some(0),
        anchor,
        scroll: 0.0,
        drag: None,
        serial: state.menu_serial,
    });
    true
}

pub(super) fn play_on_tv(state: &mut RendererState, then: Then, anchor: Pos2, keyboard: bool) {
    let saved = state
        .session
        .as_ref()
        .and_then(|session| cast::Device::saved(session.storage()));
    if let Some(device) = saved {
        start_cast(state, then, device);
        return;
    }
    state.cast_scan.scan();
    state.menu_serial += 1;
    state.card_menu = Some(CardMenu {
        target: Target::CastDevices { then, count: 0 },
        selected: keyboard.then_some(0),
        anchor,
        scroll: 0.0,
        drag: None,
        serial: state.menu_serial,
    });
}

fn start_cast(state: &mut RendererState, then: Then, device: cast::Device) {
    match then {
        Then::Stream(stream) => crate::stream_menu::cast_to(state, &stream, device),
        Then::Card(card) => {
            state.cast_next = Some(device);
            state
                .pending_native_actions
                .push(play_action(card.raw.clone(), card));
            crate::sync::show_notice(state, "sources.play_on_tv_preparing");
        }
    }
}

fn play_action(raw: Value, card: HomeCard) -> NativeAction {
    let id = card.id.unwrap_or_default();
    let item_type = card.item_type.unwrap_or_default();
    if raw.get("lastVideoId").is_some() {
        NativeAction::StartPlayback { item: raw }
    } else {
        NativeAction::Play { id, item_type }
    }
}

fn initial_scroll(state: &RendererState, count: usize, current: Option<usize>) -> f32 {
    current.map_or(0.0, |index| {
        let viewport = logical_viewport(state);
        let layout = fluxa_ui::action_menu_layout(viewport, count, Pos2::ZERO, 0.0, 0.0);
        let centered = layout
            .rows
            .get(index)
            .map_or(0.0, |row| row.center().y - layout.body.center().y);
        centered.clamp(0.0, layout.max_scroll)
    })
}

pub(super) fn open_setting(state: &mut RendererState, row: &'static fluxa_ui::SettingsRow) {
    state.menu_serial += 1;
    let current = state
        .settings
        .value(row.key)
        .and_then(Value::as_str)
        .and_then(|value| row.options.iter().position(|option| *option == value));
    state.card_menu = Some(CardMenu {
        target: Target::Setting(row),
        selected: state.keyboard_focus_visible.then(|| current.unwrap_or(0)),
        anchor: Pos2::ZERO,
        scroll: initial_scroll(state, row.options.len(), current),
        drag: None,
        serial: state.menu_serial,
    });
}

pub(super) fn open_source(state: &mut RendererState, index: usize) {
    let Some((title, key)) = fluxa_ui::account_source(index) else {
        return;
    };
    let (options, current, _) = fluxa_ui::account_source_state(&state.settings, key);
    if options.is_empty() {
        return;
    }
    let current = options.iter().position(|(value, _)| *value == current);
    state.menu_serial += 1;
    let scroll = initial_scroll(state, options.len(), current);
    state.card_menu = Some(CardMenu {
        target: Target::Source {
            key,
            title,
            count: options.len(),
        },
        selected: state.keyboard_focus_visible.then(|| current.unwrap_or(0)),
        anchor: Pos2::ZERO,
        scroll,
        drag: None,
        serial: state.menu_serial,
    });
}

pub(super) fn open_choice(state: &mut RendererState, request: fluxa_ui::ChoiceRequest) {
    let current = request
        .options
        .iter()
        .position(|(value, _)| *value == request.selected);
    state.menu_serial += 1;
    let scroll = initial_scroll(state, request.options.len(), current);
    state.card_menu = Some(CardMenu {
        target: Target::Choice(request),
        selected: state.keyboard_focus_visible.then(|| current.unwrap_or(0)),
        anchor: Pos2::ZERO,
        scroll,
        drag: None,
        serial: state.menu_serial,
    });
}

pub(crate) fn open_disconnect(state: &mut RendererState, provider: &str) {
    state.menu_serial += 1;
    state.card_menu = Some(CardMenu {
        target: Target::Disconnect(provider.to_owned()),
        selected: state.keyboard_focus_visible.then_some(0),
        anchor: Pos2::ZERO,
        scroll: 0.0,
        drag: None,
        serial: state.menu_serial,
    });
}

pub(crate) fn open_quit_downloads(state: &mut RendererState) {
    state.menu_serial += 1;
    state.card_menu = Some(CardMenu {
        target: Target::QuitDownloads,
        selected: state.keyboard_focus_visible.then_some(0),
        anchor: Pos2::ZERO,
        scroll: 0.0,
        drag: None,
        serial: state.menu_serial,
    });
}

pub(crate) fn quit_downloads_open(state: &RendererState) -> bool {
    matches!(
        state.card_menu.as_ref().map(|menu| &menu.target),
        Some(Target::QuitDownloads)
    )
}

pub(super) fn view(state: &RendererState) -> Option<MenuView> {
    let menu = state.card_menu.as_ref()?;
    let language = state.settings.language();
    let (card, entries) = match &menu.target {
        Target::Card { card, entries } => (card, entries),
        Target::Mark { title, entries } => {
            let language = state.settings.language();
            return Some(MenuView {
                title: title.clone(),
                items: entries
                    .iter()
                    .map(|entry| ActionMenuItem {
                        icon: entry.icon,
                        label: localized(entry.label, language),
                        app_icon: None,
                    })
                    .collect(),
                selected: menu.selected,
                anchor: menu.anchor,
                scroll: menu.scroll,
                serial: menu.serial,
            });
        }
        Target::Setting(row) => {
            let current = state.settings.value(row.key).and_then(Value::as_str);
            return Some(MenuView {
                title: fluxa_ui::settings_row_label(row, language),
                items: row
                    .options
                    .iter()
                    .map(|option| ActionMenuItem {
                        icon: if current == Some(*option) {
                            "Check"
                        } else {
                            ""
                        },
                        label: fluxa_ui::option_label(row.key, option, language),
                        app_icon: (row.key == "appIcon").then_some(*option),
                    })
                    .collect(),
                selected: menu.selected,
                anchor: menu.anchor,
                scroll: menu.scroll,
                serial: menu.serial,
            });
        }
        Target::Choice(request) => {
            return Some(MenuView {
                title: request.title.clone(),
                items: request
                    .options
                    .iter()
                    .map(|(value, label)| ActionMenuItem {
                        icon: if *value == request.selected {
                            "Check"
                        } else {
                            ""
                        },
                        label: label.clone(),
                        app_icon: None,
                    })
                    .collect(),
                selected: menu.selected,
                anchor: menu.anchor,
                scroll: menu.scroll,
                serial: menu.serial,
            });
        }
        Target::Disconnect(provider) => {
            return Some(MenuView {
                title: localized("settings.disconnect_confirm_title", language)
                    .replace("%s", &provider_name(provider)),
                items: vec![ActionMenuItem {
                    icon: "Close",
                    label: localized("settings.account_disconnect", language),
                    app_icon: None,
                }],
                selected: menu.selected,
                anchor: menu.anchor,
                scroll: menu.scroll,
                serial: menu.serial,
            });
        }
        Target::QuitDownloads => {
            return Some(MenuView {
                title: localized("downloads.quit_title", language),
                items: vec![
                    ActionMenuItem {
                        icon: "Downloads",
                        label: localized("downloads.quit_keep", language),
                        app_icon: None,
                    },
                    ActionMenuItem {
                        icon: "Close",
                        label: localized("downloads.quit_stop", language),
                        app_icon: None,
                    },
                ],
                selected: menu.selected,
                anchor: menu.anchor,
                scroll: menu.scroll,
                serial: menu.serial,
            });
        }
        Target::Stream {
            stream, entries, ..
        } => {
            return Some(MenuView {
                title: crate::stream_menu::title(stream),
                items: entries
                    .iter()
                    .map(|entry| {
                        let (icon, label) = crate::stream_menu::item(*entry, language);
                        ActionMenuItem {
                            icon,
                            label,
                            app_icon: None,
                        }
                    })
                    .collect(),
                selected: menu.selected,
                anchor: menu.anchor,
                scroll: menu.scroll,
                serial: menu.serial,
            });
        }
        Target::CastDevices { then, .. } => {
            let mut items: Vec<ActionMenuItem> = state
                .cast_scan
                .listing()
                .into_iter()
                .map(|(name, protocol)| ActionMenuItem {
                    icon: "Cast",
                    label: format!("{name} · {protocol}"),
                    app_icon: None,
                })
                .collect();
            if items.is_empty() {
                let key = if state.cast_scan.scanning() {
                    "player.cast_searching"
                } else {
                    "player.cast_no_devices"
                };
                items.push(ActionMenuItem {
                    icon: "",
                    label: localized(key, language),
                    app_icon: None,
                });
            }
            let title = match then {
                Then::Stream(stream) => crate::stream_menu::title(stream),
                Then::Card(card) => card.title.clone(),
            };
            return Some(MenuView {
                title,
                items,
                selected: menu.selected,
                anchor: menu.anchor,
                scroll: menu.scroll,
                serial: menu.serial,
            });
        }
        Target::Source { key, title, .. } => {
            let (options, current, _) = fluxa_ui::account_source_state(&state.settings, key);
            return Some(MenuView {
                title: localized(title, language),
                items: options
                    .into_iter()
                    .map(|(value, label)| ActionMenuItem {
                        icon: if value == current { "Check" } else { "" },
                        label,
                        app_icon: None,
                    })
                    .collect(),
                selected: menu.selected,
                anchor: menu.anchor,
                scroll: menu.scroll,
                serial: menu.serial,
            });
        }
    };
    let resume = card.progress > 0.0;
    let items = entries
        .iter()
        .map(|entry| {
            let (icon, key) = match entry {
                Entry::Play if resume => ("PlayFilled", "menu.resume"),
                Entry::Play => ("PlayFilled", "menu.play"),
                Entry::PlayOnTv => ("Cast", "sources.play_on_tv"),
                Entry::Details => ("Info", "menu.details"),
                Entry::Watchlist { saved: true } => ("Check", "menu.remove_watchlist"),
                Entry::Watchlist { saved: false } => ("Plus", "menu.add_watchlist"),
                Entry::Watched { watched: true } => ("EyeOff", "menu.mark_unwatched"),
                Entry::Watched { watched: false } => ("CircleCheck", "menu.mark_watched"),
                Entry::Rewatch => ("Refresh", "menu.rewatch"),
                Entry::ClearProgress => ("Close", "menu.remove_continue"),
            };
            ActionMenuItem {
                icon,
                label: localized(key, language),
                app_icon: None,
            }
        })
        .collect();
    Some(MenuView {
        title: card.title.clone(),
        items,
        selected: menu.selected,
        anchor: menu.anchor,
        scroll: menu.scroll,
        serial: menu.serial,
    })
}

fn provider_name(provider: &str) -> String {
    let mut chars = provider.chars();
    chars
        .next()
        .map(|first| first.to_uppercase().chain(chars).collect())
        .unwrap_or_default()
}

pub(super) fn apply(state: &mut RendererState, outcome: ActionMenuOutcome) {
    match outcome {
        ActionMenuOutcome::Pick(index) => pick(state, index),
        ActionMenuOutcome::Dismiss => state.card_menu = None,
    }
}

fn pick(state: &mut RendererState, index: usize) {
    let Some(menu) = state.card_menu.take() else {
        return;
    };
    let (card, entries) = match menu.target {
        Target::Card { card, entries } => (card, entries),
        Target::Mark { entries, .. } => {
            if let Some(entry) = entries.into_iter().nth(index) {
                let meta = state.detail.item.clone();
                let series_id = meta
                    .get("id")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_owned();
                let episodes: Vec<Value> = state
                    .detail
                    .episodes
                    .iter()
                    .filter(|episode| entry.video_ids.contains(&episode.id))
                    .map(|episode| {
                        json!({
                            "id": episode.id,
                            "name": episode.title,
                            "season": episode.season,
                            "number": episode.number,
                            "thumbnail": episode.thumbnail,
                        })
                    })
                    .collect();
                state
                    .pending_native_actions
                    .push(NativeAction::CoreCommand {
                        command: json!({
                            "type": "markWatchedRequested",
                            "seriesId": series_id,
                            "videoIds": entry.video_ids,
                            "watched": entry.watched,
                            "meta": meta,
                            "episodes": episodes,
                        }),
                    });
            }
            return;
        }
        Target::Setting(row) => {
            if let Some(option) = row.options.get(index) {
                state
                    .pending_native_actions
                    .push(NativeAction::SettingsChange {
                        key: row.key.to_owned(),
                        value: Value::String((*option).to_owned()),
                    });
            }
            return;
        }
        Target::Choice(request) => {
            if let Some((value, _)) = request.options.into_iter().nth(index) {
                let route = state.route;
                crate::input::apply_choice(state, route, request.key, value);
            }
            return;
        }
        Target::QuitDownloads => {
            if index == 0 {
                state.quit_when_idle = true;
                state.minimize_requested = true;
            } else {
                state.quit_requested = true;
            }
            return;
        }
        Target::Stream {
            stream,
            index: source,
            entries,
        } => {
            match entries.get(index).copied() {
                Some(crate::stream_menu::Entry::PlayOnTv) => {
                    play_on_tv(
                        state,
                        Then::Stream(stream),
                        menu.anchor,
                        menu.selected.is_some(),
                    );
                }
                Some(entry) => crate::stream_menu::run(state, &stream, source, entry),
                None => {}
            }
            return;
        }
        Target::CastDevices { then, .. } => {
            if let Some(device) = state.cast_scan.device(index) {
                if let Some(session) = state.session.as_ref() {
                    device.save(session.storage());
                }
                start_cast(state, then, device);
            }
            return;
        }
        Target::Disconnect(provider) => {
            if index == 0 {
                crate::accounts::disconnect(state, &provider);
            }
            return;
        }
        Target::Source { key, .. } => {
            let (options, _, _) = fluxa_ui::account_source_state(&state.settings, key);
            if let Some((value, _)) = options.into_iter().nth(index) {
                state
                    .pending_native_actions
                    .push(NativeAction::SettingsChange {
                        key: key.to_owned(),
                        value: Value::String(value),
                    });
            }
            return;
        }
    };
    let Some(entry) = entries.get(index).copied() else {
        return;
    };
    let id = card.id.clone().unwrap_or_default();
    let item_type = card.item_type.clone().unwrap_or_default();
    if matches!(entry, Entry::Rewatch)
        && let Some(session) = state.session.as_ref()
    {
        let profile = session.active_profile();
        fluxa_effects::set_simkl_rewatch_intent(
            session.storage(),
            profile.get("id").and_then(Value::as_str).unwrap_or("guest"),
            &id,
        );
    }
    if entry == Entry::PlayOnTv {
        play_on_tv(
            state,
            Then::Card(card),
            menu.anchor,
            menu.selected.is_some(),
        );
        return;
    }
    let action = match entry {
        Entry::PlayOnTv => return,
        Entry::Play | Entry::Rewatch if card.raw.get("lastVideoId").is_some() => {
            NativeAction::StartPlayback { item: card.raw }
        }
        Entry::Play | Entry::Rewatch => NativeAction::Play { id, item_type },
        Entry::Details => NativeAction::Detail {
            id,
            item_type,
            preview: card.raw,
        },
        Entry::Watchlist { .. } => NativeAction::ToggleWatchlist { item: card.raw },
        Entry::Watched { watched } => NativeAction::CoreCommand {
            command: json!({
                "type": "markWatchedRequested",
                "seriesId": id,
                "videoIds": [id],
                "watched": !watched,
                "meta": card.raw,
            }),
        },
        Entry::ClearProgress => NativeAction::CoreCommand {
            command: json!({"type": "clearPlaybackProgressRequested", "meta": card.raw}),
        },
    };
    state.pending_native_actions.push(action);
}

pub(super) fn key(state: &mut RendererState, input: KeyInput) -> bool {
    let viewport = logical_viewport(state);
    let Some(menu) = state.card_menu.as_mut() else {
        return false;
    };
    let count = menu.target.len();
    let selected = menu.selected.unwrap_or(0);
    match input {
        KeyInput::Key(Key::Up)
        | KeyInput::Gamepad(GamepadButton::DPadUp)
        | KeyInput::Key(Key::ShiftTab) => {
            menu.selected = Some(selected.saturating_sub(1));
        }
        KeyInput::Key(Key::Down)
        | KeyInput::Gamepad(GamepadButton::DPadDown)
        | KeyInput::Key(Key::Tab) => {
            menu.selected = Some((selected + 1).min(count - 1));
        }
        KeyInput::Key(Key::Enter)
        | KeyInput::Gamepad(GamepadButton::South | GamepadButton::Start) => match menu.selected {
            Some(index) => pick(state, index),
            None => menu.selected = Some(0),
        },
        KeyInput::Key(Key::Back | Key::Escape)
        | KeyInput::Gamepad(GamepadButton::East | GamepadButton::Select | GamepadButton::West) => {
            state.card_menu = None;
        }
        _ => {}
    }
    let width = menu_width(state, viewport);
    if let Some(menu) = state.card_menu.as_mut()
        && let Some(index) = menu.selected
    {
        menu.scroll = fluxa_ui::action_menu_scroll_to(
            viewport,
            count,
            menu.anchor,
            index,
            menu.scroll,
            width,
        );
    }
    true
}

fn menu_width(state: &RendererState, viewport: fluxa_ui::Viewport) -> f32 {
    let items = view(state).map(|view| view.items).unwrap_or_default();
    fluxa_ui::action_menu_width(
        viewport,
        fluxa_ui::UiMetrics::for_viewport(viewport),
        &items,
    )
}

pub(super) fn scroll_by(state: &mut RendererState, delta: f32) -> bool {
    let viewport = logical_viewport(state);
    let width = menu_width(state, viewport);
    let Some(menu) = state.card_menu.as_mut() else {
        return false;
    };
    let layout = fluxa_ui::action_menu_layout(viewport, menu.target.len(), menu.anchor, 0.0, width);
    menu.scroll = (menu.scroll + delta).clamp(0.0, layout.max_scroll);
    true
}

pub(super) fn open_from_focus(state: &mut RendererState) -> bool {
    rebuild_current_ui(state);
    let Some(node) = state.ui.focused() else {
        return false;
    };
    let anchor = state
        .ui
        .node(node)
        .map(|node| {
            Pos2::new(
                node.bounds.x + node.bounds.width * 0.5,
                node.bounds.y + node.bounds.height * 0.5,
            )
        })
        .unwrap_or(Pos2::ZERO);
    open(state, node, anchor, true)
}

pub(super) fn open_at(state: &mut RendererState, position: Pos2) -> bool {
    rebuild_current_ui(state);
    let Some(node) = state.ui.hit_test([position.x, position.y]) else {
        return false;
    };
    open(state, node, position, false)
}

pub(super) fn pointer(state: &mut RendererState, phase: PointerPhase, position: [f32; 2]) -> bool {
    if state.card_menu.is_none() {
        return false;
    }
    let viewport = logical_viewport(state);
    let point = Pos2::new(position[0], position[1]);
    let width = menu_width(state, viewport);
    match phase {
        PointerPhase::Down => {
            if let Some(menu) = state.card_menu.as_mut() {
                let layout = fluxa_ui::action_menu_layout(
                    viewport,
                    menu.target.len(),
                    menu.anchor,
                    0.0,
                    width,
                );
                menu.drag = layout
                    .body
                    .contains(point)
                    .then_some((point.y, 0.0))
                    .filter(|_| layout.max_scroll > 0.0);
            }
            return true;
        }
        PointerPhase::Move => {
            let delta = state.card_menu.as_mut().and_then(|menu| {
                let (last, travelled) = menu.drag?;
                menu.drag = Some((point.y, travelled + (last - point.y).abs()));
                Some(last - point.y)
            });
            if let Some(delta) = delta {
                scroll_by(state, delta);
            }
            return true;
        }
        PointerPhase::Up => {}
    }
    let dragged = state
        .card_menu
        .as_mut()
        .and_then(|menu| menu.drag.take())
        .is_some_and(|(_, travelled)| travelled > 8.0);
    if state.touch_menu_opened {
        state.touch_menu_opened = false;
        return true;
    }
    if dragged {
        return true;
    }
    let Some(menu) = state.card_menu.as_ref() else {
        return true;
    };
    let layout =
        fluxa_ui::action_menu_layout(viewport, menu.target.len(), menu.anchor, menu.scroll, width);
    match fluxa_ui::action_menu_hit(&layout, point) {
        Some(index) => pick(state, index),
        None if !layout.panel.contains(point) => state.card_menu = None,
        None => {}
    }
    true
}

pub(super) fn tick(state: &mut RendererState) {
    state.cast_scan.poll();
    if let Some(menu) = state.card_menu.as_mut()
        && let Target::CastDevices { count, .. } = &mut menu.target
    {
        *count = state.cast_scan.device_count();
    }
    let (Some(start), Some(down_at)) = (state.touch_start, state.touch_down_at) else {
        return;
    };
    if state.touch_scrolled || state.card_menu.is_some() || down_at.elapsed() < LONG_PRESS {
        return;
    }
    state.touch_down_at = None;
    if open_at(state, Pos2::new(start[0], start[1])) {
        state.touch_menu_opened = true;
        fluxa_ui::buzz(fluxa_ui::Haptic::LongPress);
    }
}
