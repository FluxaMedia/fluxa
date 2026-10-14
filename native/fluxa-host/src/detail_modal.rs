use super::*;
use fluxa_ui::DetailModalOutcome;

pub(super) fn apply(state: &mut RendererState) {
    let Some(outcome) = state
        .gpu
        .as_ref()
        .and_then(|gpu| fluxa_ui::take_detail_modal_outcome(&gpu.egui_context))
    else {
        return;
    };
    if matches!(
        outcome,
        DetailModalOutcome::Play
            | DetailModalOutcome::Episode(_)
            | DetailModalOutcome::SimilarPlay(_)
    ) && crate::sync::playback_blocked(state)
    {
        crate::sync::show_notice(state, "playback.no_sources");
        return;
    }
    match outcome {
        DetailModalOutcome::Close => close(state),
        DetailModalOutcome::OpenPage => state
            .pending_native_actions
            .push(NativeAction::OpenDetailPage),
        DetailModalOutcome::Watchlist => {
            state
                .pending_native_actions
                .push(NativeAction::ToggleWatchlist {
                    item: state.detail.item.clone(),
                })
        }
        DetailModalOutcome::Episode(index) => {
            let Some(video) = state
                .detail
                .episodes
                .get(index)
                .map(|episode| episode.id.clone())
            else {
                return;
            };
            let item = playback_item(&state.detail.item, None, Some(video));
            close(state);
            state
                .pending_native_actions
                .push(NativeAction::StartPlayback { item });
        }
        DetailModalOutcome::Similar(index) => {
            let Some(card) = state.detail.similar.get(index) else {
                return;
            };
            let (Some(id), Some(item_type)) = (card.id.clone(), card.item_type.clone()) else {
                return;
            };
            let preview = card.raw.clone();
            state
                .pending_native_actions
                .push(NativeAction::DetailModal {
                    id,
                    item_type,
                    preview,
                });
        }
        DetailModalOutcome::SimilarPlay(index) => {
            let Some(card) = state.detail.similar.get(index) else {
                return;
            };
            let series = matches!(card.item_type.as_deref(), Some("series" | "tv" | "show"));
            let first = card
                .id
                .as_ref()
                .filter(|_| series)
                .map(|id| format!("{id}:1:1"));
            let item = playback_item(&card.raw, None, first);
            close(state);
            state
                .pending_native_actions
                .push(NativeAction::StartPlayback { item });
        }
        DetailModalOutcome::SimilarWatchlist(index) => {
            let Some(card) = state.detail.similar.get(index) else {
                return;
            };
            let item = card.raw.clone();
            state
                .pending_native_actions
                .push(NativeAction::ToggleWatchlist { item });
        }
        DetailModalOutcome::Play => {
            let id = state.detail.id.clone();
            let series = state.detail.is_series();
            let item = playback_item(
                &state.detail.item,
                state.detail.resume.as_ref(),
                series.then(|| format!("{id}:1:1")),
            );
            close(state);
            state
                .pending_native_actions
                .push(NativeAction::StartPlayback { item });
        }
    }
}

pub(super) fn close(state: &mut RendererState) {
    state.detail_modal = false;
}

pub(super) fn tick(state: &mut RendererState) {
    if state.detail_modal && (state.route == Route::Detail || state.player.is_some()) {
        close(state);
    }
}

pub(super) fn key(state: &mut RendererState, input: KeyInput) -> bool {
    if !state.detail_modal {
        return false;
    }
    if matches!(
        input,
        KeyInput::Key(Key::Back | Key::Escape) | KeyInput::Gamepad(GamepadButton::East)
    ) {
        close(state);
    }
    true
}
