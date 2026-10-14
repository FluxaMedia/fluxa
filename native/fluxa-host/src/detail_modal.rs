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
