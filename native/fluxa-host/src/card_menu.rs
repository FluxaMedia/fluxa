use super::*;
use fluxa_ui::{ActionMenuItem, HomeCard, localized};

const LONG_PRESS: Duration = Duration::from_millis(450);
pub(super) const ROW_NODE_BASE: u64 = 0x3FFF_F000;

#[derive(Clone, Copy, PartialEq)]
enum Entry {
    Play,
    PlayManual,
    PlayOnTv,
    Details,
    Watchlist { saved: bool },
    Watched { watched: bool },
    EpisodeWatched,
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
    play: Option<Value>,
}

fn manual_item(mut item: Value) -> Value {
    if let Some(fields) = item.as_object_mut() {
        fields.insert("pickSource".to_owned(), true.into());
    }
    item
}

fn manual_entry(item: Value) -> MarkEntry {
    MarkEntry {
        icon: "List",
        label: "home.play_manually",
        video_ids: Vec::new(),
        watched: false,
        play: Some(manual_item(item)),
    }
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
            Target::Setting(row) => row.choices().len(),
            Target::Source { count, .. } => *count,
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
}

pub(crate) struct MenuView {
    pub(super) title: String,
    pub(super) items: Vec<ActionMenuItem>,
    pub(super) selected: Option<usize>,
    pub(super) anchor: Pos2,
}

fn skin_card(state: &RendererState, node: u64) -> Option<HomeCard> {
    let id = super::skins::item_for_node(node)?;
    let home = &state.home;
    let library = [
        fluxa_ui::LibraryTab::Watchlist,
        fluxa_ui::LibraryTab::Watching,
        fluxa_ui::LibraryTab::Completed,
        fluxa_ui::LibraryTab::Dropped,
        fluxa_ui::LibraryTab::OnHold,
        fluxa_ui::LibraryTab::Liked,
    ]
    .into_iter()
    .flat_map(|tab| state.library.cards(tab).iter());
    home.cards
        .iter()
        .chain(home.rows.iter().flat_map(|row| row.cards.iter()))
        .chain(state.discover.results.iter())
        .chain(library)
        .find(|card| card.id.as_deref() == Some(id.as_str()))
        .cloned()
}

fn card_for_node(state: &RendererState, node: u64) -> Option<HomeCard> {
    if node >= fluxa_ui::NODE_SKIN_BASE {
        return skin_card(state, node);
    }
    let index = node.checked_sub(NODE_CARD_BASE)? as usize;
    let card = match state.route {
        Route::Home => state.home.card_at(index),
        Route::Library => state.library.cards(state.library_tab).get(index),
        Route::Discover => state.discover.results.get(index),
        Route::Folder => state.folder.card_at(state.folder_tab, index),
        Route::Detail => state.detail.linked_card(node),
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
    let mut entries = vec![Entry::Play, Entry::PlayManual];
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
    if card.item_type.as_deref() != Some("movie")
        && card
            .raw
            .get("lastVideoId")
            .and_then(Value::as_str)
            .is_some()
    {
        entries.push(Entry::EpisodeWatched);
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
    state.card_menu = Some(CardMenu {
        target: Target::Card { card, entries },
        selected: keyboard.then_some(0),
        anchor,
    });
    true
}

fn open_mark(state: &mut RendererState, node: u64, anchor: Pos2, keyboard: bool) -> bool {
    let detail = &state.detail;
    if !detail.is_series() && node != fluxa_ui::NODE_DETAIL_PLAY {
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
            play: None,
        }];
        entries.push(MarkEntry {
            icon: "CircleCheck",
            label: "detail.mark_episode_through_watched",
            video_ids: through.clone(),
            watched: true,
            play: None,
        });
        entries.push(MarkEntry {
            icon: "EyeOff",
            label: "detail.mark_episode_through_unwatched",
            video_ids: through,
            watched: false,
            play: None,
        });
        let mut item = detail.item.clone();
        if let Some(fields) = item.as_object_mut() {
            fields.insert("lastVideoId".to_owned(), episode.id.clone().into());
        }
        entries.insert(0, manual_entry(item));
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
            play: None,
        }];
        if season > 1 {
            entries.push(MarkEntry {
                icon: "CircleCheck",
                label: "detail.mark_through_season_watched",
                video_ids: through.clone(),
                watched: true,
                play: None,
            });
            entries.push(MarkEntry {
                icon: "EyeOff",
                label: "detail.mark_through_season_unwatched",
                video_ids: through,
                watched: false,
                play: None,
            });
        }
        (
            localized("format.season_number", state.settings.language())
                .replace("%s", &season.to_string()),
            entries,
        )
    } else if node == fluxa_ui::NODE_DETAIL_PLAY && !detail.id.is_empty() {
        let item = crate::actions::playback_target(
            &detail.item,
            detail.resume.as_ref(),
            None,
            crate::actions::first_playable_episode(detail),
        );
        (detail.title.clone(), vec![manual_entry(item)])
    } else {
        return false;
    };
    if entries
        .iter()
        .all(|entry| entry.video_ids.is_empty() && entry.play.is_none())
    {
        return false;
    }
    state.card_menu = Some(CardMenu {
        target: Target::Mark { title, entries },
        selected: keyboard.then_some(0),
        anchor,
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
    state.card_menu = Some(CardMenu {
        target: Target::Stream {
            stream,
            index,
            entries,
        },
        selected: keyboard.then_some(0),
        anchor,
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
    state.card_menu = Some(CardMenu {
        target: Target::CastDevices { then, count: 0 },
        selected: keyboard.then_some(0),
        anchor,
    });
}

fn start_cast(state: &mut RendererState, then: Then, device: cast::Device) {
    match then {
        Then::Stream(stream) => crate::stream_menu::cast_to(state, &stream, device),
        Then::Card(card) => {
            state.cast_next = Some(device);
            state
                .pending_native_actions
                .push(play_action(card.raw.cloned(), card));
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

fn row_target(
    settings: &fluxa_ui::SettingsModel,
    row: &'static fluxa_ui::SettingsRow,
) -> (Target, usize) {
    let current = settings
        .value(row.key)
        .and_then(Value::as_str)
        .and_then(|value| row.choices().iter().position(|option| *option == value));
    (Target::Setting(row), current.unwrap_or(0))
}

fn source_target(
    settings: &fluxa_ui::SettingsModel,
    title: &'static str,
    key: &'static str,
) -> Option<(Target, usize)> {
    let (options, current, _) = fluxa_ui::account_source_state(settings, key);
    if options.is_empty() {
        return None;
    }
    let position = options.iter().position(|(value, _)| *value == current);
    let target = Target::Source {
        key,
        title,
        count: options.len(),
    };
    Some((target, position.unwrap_or(0)))
}

fn setting_target(settings: &fluxa_ui::SettingsModel, key: &str) -> Option<(Target, usize)> {
    let row = (0..)
        .map_while(fluxa_ui::settings_row_by_index)
        .find(|row| row.key == key && !row.choices().is_empty());
    if let Some(row) = row {
        return Some(row_target(settings, row));
    }
    let (title, key) = (0..)
        .map_while(fluxa_ui::account_source)
        .find(|(_, source)| *source == key)?;
    source_target(settings, title, key)
}

fn open_target(state: &mut RendererState, target: Target, current: usize) {
    state.card_menu = Some(CardMenu {
        target,
        selected: state.keyboard_focus_visible.then_some(current),
        anchor: Pos2::ZERO,
    });
}

pub(super) fn open_setting(state: &mut RendererState, row: &'static fluxa_ui::SettingsRow) {
    let (target, current) = row_target(&state.settings, row);
    open_target(state, target, current);
}

pub(super) fn open_setting_key(state: &mut RendererState, key: &str) {
    if let Some((target, current)) = setting_target(&state.settings, key) {
        open_target(state, target, current);
    }
}

pub(super) fn open_source(state: &mut RendererState, index: usize) {
    let Some((title, key)) = fluxa_ui::account_source(index) else {
        return;
    };
    if let Some((target, current)) = source_target(&state.settings, title, key) {
        open_target(state, target, current);
    }
}

pub(super) fn open_choice(state: &mut RendererState, request: fluxa_ui::ChoiceRequest) {
    let current = request
        .options
        .iter()
        .position(|(value, _)| *value == request.selected);
    state.card_menu = Some(CardMenu {
        target: Target::Choice(request),
        selected: state.keyboard_focus_visible.then(|| current.unwrap_or(0)),
        anchor: Pos2::ZERO,
    });
}

pub(crate) fn open_quit_downloads(state: &mut RendererState) {
    state.card_menu = Some(CardMenu {
        target: Target::QuitDownloads,
        selected: state.keyboard_focus_visible.then_some(0),
        anchor: Pos2::ZERO,
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
            });
        }
        Target::Setting(row) => {
            let current = state.settings.value(row.key).and_then(Value::as_str);
            return Some(MenuView {
                title: fluxa_ui::settings_row_label(row, language),
                items: row
                    .choices()
                    .iter()
                    .map(|option| ActionMenuItem {
                        icon: if current == Some(*option) {
                            "Check"
                        } else {
                            ""
                        },
                        label: fluxa_ui::option_label(row.key, option, language),
                        app_icon: (row.key == fluxa_ui::APP_ICON_KEY).then_some(*option),
                    })
                    .collect(),
                selected: menu.selected,
                anchor: menu.anchor,
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
                Entry::PlayManual => ("List", "home.play_manually"),
                Entry::PlayOnTv => ("Cast", "sources.play_on_tv"),
                Entry::Details => ("Info", "menu.details"),
                Entry::Watchlist { saved: true } => ("Check", "menu.remove_watchlist"),
                Entry::Watchlist { saved: false } => ("Plus", "menu.add_watchlist"),
                Entry::Watched { watched: true } => ("EyeOff", "menu.mark_unwatched"),
                Entry::Watched { watched: false } => ("CircleCheck", "menu.mark_watched"),
                Entry::EpisodeWatched => ("CircleCheck", "menu.mark_watched"),
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
    })
}

pub(crate) fn pick(state: &mut RendererState, index: usize) {
    let Some(menu) = state.card_menu.take() else {
        return;
    };
    let (card, entries) = match menu.target {
        Target::Card { card, entries } => (card, entries),
        Target::Mark { entries, .. } => {
            if let Some(entry) = entries.into_iter().nth(index) {
                if let Some(item) = entry.play {
                    state
                        .pending_native_actions
                        .push(NativeAction::StartPlayback { item });
                    return;
                }
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
            if let Some(option) = row.choices().get(index) {
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
                crate::input::apply_choice(state, route, &request.key, value);
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
        Entry::PlayManual => NativeAction::StartPlayback {
            item: manual_item(crate::actions::playback_target(
                &card.raw,
                state.home.resume_for(&id),
                None,
                crate::actions::series_start(&id, &item_type),
            )),
        },
        Entry::Play | Entry::Rewatch if card.raw.get("lastVideoId").is_some() => {
            NativeAction::StartPlayback {
                item: card.raw.cloned(),
            }
        }
        Entry::Play | Entry::Rewatch => NativeAction::Play { id, item_type },
        Entry::Details => NativeAction::Detail {
            id,
            item_type,
            preview: card.raw.cloned(),
        },
        Entry::Watchlist { .. } => NativeAction::ToggleWatchlist {
            item: card.raw.cloned(),
        },
        Entry::Watched { watched } => NativeAction::CoreCommand {
            command: json!({
                "type": "markWatchedRequested",
                "seriesId": id,
                "videoIds": [id],
                "watched": !watched,
                "meta": card.raw,
            }),
        },
        Entry::EpisodeWatched => {
            let raw = &card.raw;
            let video_id = raw
                .get("lastVideoId")
                .and_then(Value::as_str)
                .unwrap_or_default();
            NativeAction::CoreCommand {
                command: json!({
                    "type": "markWatchedRequested",
                    "seriesId": id,
                    "videoIds": [video_id],
                    "watched": true,
                    "meta": raw,
                    "episodes": [{
                        "id": video_id,
                        "name": raw.get("lastEpisodeName"),
                        "season": raw.get("lastEpisodeSeason"),
                        "number": raw.get("lastEpisodeNumber"),
                        "thumbnail": raw.get("lastEpisodeThumbnail"),
                    }],
                }),
            }
        }
        Entry::ClearProgress => NativeAction::CoreCommand {
            command: json!({"type": "clearPlaybackProgressRequested", "meta": card.raw}),
        },
    };
    state.pending_native_actions.push(action);
}

pub(super) fn key(state: &mut RendererState, input: KeyInput) -> bool {
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
    true
}

pub(crate) fn dismiss(state: &mut RendererState) {
    state.card_menu = None;
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn nuvio_only(values: Value) -> fluxa_ui::SettingsModel {
        fluxa_ui::SettingsModel {
            values,
            profile: json!({"nuvioAccessToken": "token"}),
            ..Default::default()
        }
    }

    #[test]
    fn library_source_dropdown_opens_with_the_connected_providers() {
        let settings = nuvio_only(json!({"integrationLibrarySource": "nuvio"}));
        let (target, current) = setting_target(&settings, "integrationLibrarySource").unwrap();
        assert!(matches!(target, Target::Source { count: 1, .. }));
        assert_eq!(current, 0);
    }

    #[test]
    fn stale_source_still_opens_so_it_can_be_replaced() {
        let settings = nuvio_only(json!({"continueWatchingSource": "simkl"}));
        let (target, current) = setting_target(&settings, "continueWatchingSource").unwrap();
        assert!(matches!(target, Target::Source { count: 1, .. }));
        assert_eq!(current, 0);
    }

    #[test]
    fn fixed_choice_settings_keep_their_own_menu() {
        let settings = nuvio_only(json!({}));
        let (target, _) = setting_target(&settings, "language").unwrap();
        assert!(matches!(target, Target::Setting(_)));
    }

    #[test]
    fn every_account_source_with_options_resolves_by_key() {
        let settings = fluxa_ui::SettingsModel {
            values: json!({}),
            profile: json!({
                "nuvioAccessToken": "a",
                "traktAccessToken": "b",
                "mdblistAccessToken": "c",
            }),
            ..Default::default()
        };
        for (_, key) in (0..).map_while(fluxa_ui::account_source) {
            let (options, _, _) = fluxa_ui::account_source_state(&settings, key);
            if !options.is_empty() {
                assert!(
                    setting_target(&settings, key).is_some(),
                    "{key} has options but its dropdown does not open"
                );
            }
        }
    }
}
