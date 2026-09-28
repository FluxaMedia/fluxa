use super::*;
use fluxa_ui::{ActionMenuItem, ActionMenuOutcome, HomeCard, localized};

const LONG_PRESS: Duration = Duration::from_millis(450);

#[derive(Clone, Copy, PartialEq)]
enum Entry {
    Play,
    Details,
    Watchlist { saved: bool },
    Watched { watched: bool },
    ClearProgress,
}

pub(super) struct CardMenu {
    card: HomeCard,
    entries: Vec<Entry>,
    selected: Option<usize>,
    anchor: Pos2,
    serial: u64,
}

pub(super) struct MenuView {
    pub(super) title: String,
    pub(super) items: Vec<ActionMenuItem>,
    pub(super) selected: Option<usize>,
    pub(super) anchor: Pos2,
    pub(super) serial: u64,
}

fn card_for_node(state: &RendererState, node: u64) -> Option<HomeCard> {
    let index = node.checked_sub(NODE_CARD_BASE)? as usize;
    let card = match state.route {
        Route::Home => state.home.card_at(index),
        Route::Library => state.library.cards(state.library_tab).get(index),
        Route::Discover => state.discover.results.get(index),
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
    if state.player.is_some() || state.profiles.is_some() {
        return false;
    }
    let Some(card) = card_for_node(state, node) else {
        return false;
    };
    let id = card.id.clone().unwrap_or_default();
    let personal = state.library.personal.get(&id);
    let mut entries = vec![Entry::Play, Entry::Details];
    entries.push(Entry::Watchlist {
        saved: personal.is_some_and(|personal| personal.saved),
    });
    if card.item_type.as_deref() == Some("movie") {
        entries.push(Entry::Watched {
            watched: personal.is_some_and(|personal| personal.watched),
        });
    }
    if card.progress > 0.0 || personal.is_some_and(|personal| personal.progress > 0.0) {
        entries.push(Entry::ClearProgress);
    }
    state.menu_serial += 1;
    state.card_menu = Some(CardMenu {
        card,
        entries,
        selected: keyboard.then_some(0),
        anchor,
        serial: state.menu_serial,
    });
    true
}

pub(super) fn view(state: &RendererState) -> Option<MenuView> {
    let menu = state.card_menu.as_ref()?;
    let language = state.settings.language();
    let resume = menu.card.progress > 0.0;
    let items = menu
        .entries
        .iter()
        .map(|entry| {
            let (icon, key) = match entry {
                Entry::Play if resume => ("PlayFilled", "menu.resume"),
                Entry::Play => ("PlayFilled", "menu.play"),
                Entry::Details => ("Info", "menu.details"),
                Entry::Watchlist { saved: true } => ("Check", "menu.remove_watchlist"),
                Entry::Watchlist { saved: false } => ("Plus", "menu.add_watchlist"),
                Entry::Watched { watched: true } => ("EyeOff", "menu.mark_unwatched"),
                Entry::Watched { watched: false } => ("CircleCheck", "menu.mark_watched"),
                Entry::ClearProgress => ("Close", "menu.remove_continue"),
            };
            ActionMenuItem {
                icon,
                label: localized(key, language),
            }
        })
        .collect();
    Some(MenuView {
        title: menu.card.title.clone(),
        items,
        selected: menu.selected,
        anchor: menu.anchor,
        serial: menu.serial,
    })
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
    let Some(entry) = menu.entries.get(index).copied() else {
        return;
    };
    let card = menu.card;
    let id = card.id.clone().unwrap_or_default();
    let item_type = card.item_type.clone().unwrap_or_default();
    let action = match entry {
        Entry::Play if card.raw.get("lastVideoId").is_some() => {
            NativeAction::StartPlayback { item: card.raw }
        }
        Entry::Play => NativeAction::Play { id, item_type },
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
    let Some(menu) = state.card_menu.as_mut() else {
        return false;
    };
    let count = menu.entries.len();
    let selected = menu.selected.unwrap_or(0);
    match input {
        KeyInput::Key(Key::Up) | KeyInput::Gamepad(GamepadButton::DPadUp) | KeyInput::Key(Key::ShiftTab) => {
            menu.selected = Some(selected.saturating_sub(1));
        }
        KeyInput::Key(Key::Down) | KeyInput::Gamepad(GamepadButton::DPadDown) | KeyInput::Key(Key::Tab) => {
            menu.selected = Some((selected + 1).min(count - 1));
        }
        KeyInput::Key(Key::Enter) | KeyInput::Gamepad(GamepadButton::South | GamepadButton::Start) => {
            match menu.selected {
                Some(index) => pick(state, index),
                None => menu.selected = Some(0),
            }
        }
        KeyInput::Key(Key::Back | Key::Escape)
        | KeyInput::Gamepad(GamepadButton::East | GamepadButton::Select | GamepadButton::West) => {
            state.card_menu = None;
        }
        _ => {}
    }
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
        .map(|node| Pos2::new(node.bounds.x + node.bounds.width * 0.5, node.bounds.y + node.bounds.height * 0.5))
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
    if phase != PointerPhase::Up {
        return true;
    }
    if state.touch_menu_opened {
        state.touch_menu_opened = false;
        return true;
    }
    let viewport = logical_viewport(state);
    let Some(menu) = state.card_menu.as_ref() else {
        return true;
    };
    let layout = fluxa_ui::action_menu_layout(viewport, menu.entries.len(), menu.anchor);
    let position = Pos2::new(position[0], position[1]);
    match layout.rows.iter().position(|row| row.contains(position)) {
        Some(index) => pick(state, index),
        None if !layout.panel.contains(position) => state.card_menu = None,
        None => {}
    }
    true
}

pub(super) fn tick(state: &mut RendererState) {
    let (Some(start), Some(down_at)) = (state.touch_start, state.touch_down_at) else {
        return;
    };
    if state.touch_scrolled || state.card_menu.is_some() || down_at.elapsed() < LONG_PRESS {
        return;
    }
    state.touch_down_at = None;
    if open_at(state, Pos2::new(start[0], start[1])) {
        state.touch_menu_opened = true;
    }
}
