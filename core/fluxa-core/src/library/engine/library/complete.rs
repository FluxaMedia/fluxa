use super::*;

pub(crate) fn complete(
    engine: &mut HeadlessEngine,
    effect_type: &str,
    generation: u64,
    result: &EffectResultInput,
) -> Vec<EffectEnvelope> {
    match effect_type {
        "readLibraryState" => {
            if generation == engine.state.runtime.get(GenerationKey::Library) {
                engine.state.library.is_loading = false;
                if result.status.is_ok() {
                    let library_result = serde_json::from_str::<Value>(
                        &crate::library::state::normalize_library_read_result_json(
                            &result.value.to_string(),
                        ),
                    )
                    .unwrap_or_else(|_| serde_json::json!({}));
                    engine.state.library.watchlist = library_result
                        .get("watchlist")
                        .cloned()
                        .unwrap_or_else(|| serde_json::json!([]));
                    engine.state.library.continue_watching = library_result
                        .get("continueWatching")
                        .cloned()
                        .unwrap_or_else(|| serde_json::json!([]));
                    engine.state.library.liked = library_result
                        .get("liked")
                        .cloned()
                        .unwrap_or_else(|| serde_json::json!([]));
                    engine.state.library.watched = library_result
                        .get("watched")
                        .cloned()
                        .unwrap_or_else(|| serde_json::json!({}));
                    engine.state.library.dropped = library_result
                        .get("dropped")
                        .cloned()
                        .unwrap_or_else(|| serde_json::json!([]));
                    engine.state.library.on_hold = library_result
                        .get("onHold")
                        .cloned()
                        .unwrap_or_else(|| serde_json::json!([]));
                    engine.state.library.completed = library_result
                        .get("completed")
                        .cloned()
                        .unwrap_or_else(|| serde_json::json!([]));
                    engine.state.library.error = Value::Null;
                } else {
                    engine.state.library.error = normalize_error(result.error.clone());
                }
            }
        }
        "writeLibraryCommand" => {
            if generation == engine.state.runtime.get(GenerationKey::Library) {
                if result.status.is_ok() {
                    engine.state.library.last_write = result.value.clone();
                    engine.state.library.last_write_error = Value::Null;
                    if let Some(value) = engine
                        .state
                        .library
                        .last_write
                        .get("isInWatchlist")
                        .cloned()
                    {
                        detail::set_is_in_watchlist(engine, value);
                    }
                    if let Some(value) = engine
                        .state
                        .library
                        .last_write
                        .get("localWatchedVideoIds")
                        .cloned()
                    {
                        detail::set_local_watched_video_ids(engine, value);
                    }
                } else {
                    engine.state.library.last_write_error = normalize_error(result.error.clone());
                }
            }
        }
        "writeFeedback" => {
            if generation == engine.state.runtime.get(GenerationKey::Library) {
                if result.status.is_ok() {
                    detail::set_feedback(
                        engine,
                        result.value.get("feedback").cloned().unwrap_or(Value::Null),
                    );
                    engine.state.library.last_write_error = Value::Null;
                } else {
                    engine.state.library.last_write_error = normalize_error(result.error.clone());
                }
            }
        }
        "clearPlaybackProgress" => {
            if generation == engine.state.runtime.get(GenerationKey::Library) {
                if result.status.is_ok() {
                    detail::clear_saved_playback(engine);
                    engine.state.library.last_write_error = Value::Null;
                    // Remove the dropped item from home.continueWatching so stale state
                    // doesn't reappear when the user navigates back to the home screen.
                    if let Some(dropped_id) = result.value.get("droppedId").and_then(Value::as_str)
                    {
                        home::remove_from_continue_watching(engine, dropped_id);
                    }
                } else {
                    engine.state.library.last_write_error = normalize_error(result.error.clone());
                }
            }
        }
        "writePlaybackProgress" => {
            if generation == engine.state.runtime.get(GenerationKey::Library) {
                if result.status.is_ok() {
                    engine.state.library.saved_playback_progress =
                        engine.state.library.pending_playback_progress.clone();
                    engine.state.library.pending_playback_progress = Value::Null;
                    engine.state.library.last_write_error = Value::Null;
                } else {
                    engine.state.library.last_write_error = normalize_error(result.error.clone());
                }
            }
        }
        "syncWatchedState" => {
            if generation == engine.state.runtime.get(GenerationKey::Library) {
                if result.status.is_ok() {
                    engine.state.library.last_watched_sync = result.value.clone();
                    engine.state.library.last_watched_sync_error = Value::Null;
                } else {
                    engine.state.library.last_watched_sync_error =
                        normalize_error(result.error.clone());
                }
            }
        }
        _ => {}
    }
    vec![]
}
