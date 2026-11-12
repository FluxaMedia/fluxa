use super::*;
use fluxa_ui::DetailModalOutcome;

pub(super) fn apply(state: &mut RendererState) {
    let Some(outcome) = state.gpu.as_ref().and_then(|gpu| {
        fluxa_ui::take_detail_modal_outcome(fluxa_ui::gfx::Context::from_backend(&gpu.egui_context))
    }) else {
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
            let preview = card.raw.cloned();
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
            let item = card.raw.cloned();
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

pub(super) fn begin_load(state: &mut RendererState, id: &str, item_type: &str, preview: &Value) {
    if state.detail.id != id {
        state.scroll_reset_rows(Route::Detail);
    }
    let placeholder = placeholder_detail(state.core_snapshot.as_ref(), id, item_type, preview);
    retire(std::mem::replace(&mut state.detail, placeholder));
    state.detail_pending = Some(id.to_owned());
}

pub(super) fn accept_projection(pending: &mut Option<String>, incoming: &str) -> bool {
    match pending {
        Some(id) if id != incoming => false,
        _ => {
            *pending = None;
            true
        }
    }
}

fn placeholder_detail(
    snapshot: Option<&Snapshot>,
    id: &str,
    item_type: &str,
    preview: &Value,
) -> fluxa_ui::DetailModel {
    let slice = |key: &str| {
        snapshot
            .and_then(|snapshot| snapshot.get(key))
            .cloned()
            .unwrap_or(Value::Null)
    };
    crate::projection::detail_model(&json!({
        "detail": {
            "id": id,
            "contentType": item_type,
            "isLoading": true,
            "meta": preview.clone(),
        },
        "profile": slice("profile"),
        "settings": slice("settings"),
    }))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn card_data_fills_the_detail_before_the_core_answers() {
        let preview =
            json!({"id": "tt2", "name": "Second Title", "poster": "https://img.example/2.jpg"});
        let detail = placeholder_detail(None, "tt2", "movie", &preview);
        assert_eq!(detail.id, "tt2");
        assert_eq!(detail.title, "Second Title");
        assert_eq!(
            detail.poster_url.as_deref(),
            Some("https://img.example/2.jpg")
        );
        assert!(detail.is_loading);
    }

    #[test]
    fn projection_of_the_previous_item_is_dropped_while_a_load_is_pending() {
        let mut pending = Some("tt2".to_owned());
        assert!(!accept_projection(&mut pending, "tt1"));
        assert_eq!(pending.as_deref(), Some("tt2"));
        assert!(accept_projection(&mut pending, "tt2"));
        assert_eq!(pending, None);
        assert!(accept_projection(&mut pending, "tt3"));
    }
}
