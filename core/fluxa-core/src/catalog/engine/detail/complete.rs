use super::*;

pub(crate) fn complete(
    engine: &mut HeadlessEngine,
    effect_type: &str,
    generation: u64,
    result: &EffectResultInput,
) -> Vec<EffectEnvelope> {
    match effect_type {
        "fetchMetaDetail" => {
            if generation == engine.state.runtime.get(GenerationKey::Detail) {
                engine.state.detail.is_loading = false;
                if result.status.is_ok() {
                    let meta = crate::catalog::identity::split_channel_schedule(
                        result.value.get("meta").cloned().unwrap_or(Value::Null),
                    );
                    engine.state.detail.trailers = normalize_meta_trailers(&meta);
                    engine.state.detail.mdblist_ratings = result
                        .value
                        .get("mdblistRatings")
                        .cloned()
                        .unwrap_or(Value::Null);
                    engine.state.detail.meta = meta;
                    engine.state.detail.error = Value::Null;
                    remap_local_watched(engine);
                } else {
                    engine.state.detail.error = normalize_error(result.error.clone());
                }
            }
        }
        "readPlaybackProgress" => {
            if generation == engine.state.runtime.get(GenerationKey::Detail) {
                engine.state.detail.saved_playback = if result.status.is_ok() {
                    result.value.clone()
                } else {
                    Value::Null
                };
            }
        }
        "readDetailLocalState" => {
            if generation == engine.state.runtime.get(GenerationKey::Detail) {
                if result.status.is_ok() {
                    engine.state.detail.saved_playback = result
                        .value
                        .get("savedPlayback")
                        .cloned()
                        .unwrap_or(Value::Null);
                    engine.state.detail.local_watched_video_ids = result
                        .value
                        .get("localWatchedVideoIds")
                        .cloned()
                        .unwrap_or_else(|| serde_json::json!([]));
                    engine.state.detail.provider_watched_video_ids =
                        engine.state.detail.local_watched_video_ids.clone();
                    engine.state.detail.trakt_seasons = result
                        .value
                        .get("traktSeasons")
                        .cloned()
                        .unwrap_or(Value::Null);
                    remap_local_watched(engine);
                    engine.state.detail.is_in_watchlist = result
                        .value
                        .get("isInWatchlist")
                        .cloned()
                        .unwrap_or(Value::Bool(false));
                    engine.state.detail.feedback =
                        result.value.get("feedback").cloned().unwrap_or(Value::Null);
                    engine.state.detail.has_stream_providers = result
                        .value
                        .get("hasStreamProviders")
                        .cloned()
                        .unwrap_or(Value::Bool(false));
                    engine.state.detail.user_addons = result
                        .value
                        .get("userAddons")
                        .cloned()
                        .unwrap_or_else(|| serde_json::json!([]));
                } else {
                    engine.state.detail.error = normalize_error(result.error.clone());
                }
            }
        }
        "fetchDetailSecondary" => {
            if generation == engine.state.runtime.get(GenerationKey::Detail) {
                if result.status.is_ok() {
                    engine.state.detail.watched_video_ids = result
                        .value
                        .get("watchedVideoIds")
                        .cloned()
                        .unwrap_or_else(|| serde_json::json!([]));
                    engine.state.detail.similar_items = result
                        .value
                        .get("similarItems")
                        .cloned()
                        .unwrap_or_else(|| serde_json::json!([]));
                    if value_array_is_empty(&engine.state.detail.trailers) {
                        engine.state.detail.trailers = result
                            .value
                            .get("trailers")
                            .cloned()
                            .unwrap_or_else(|| serde_json::json!([]));
                    }
                    engine.state.detail.omdb_ratings = result
                        .value
                        .get("omdbRatings")
                        .cloned()
                        .unwrap_or(Value::Null);
                    engine.state.detail.fanart_artwork = result
                        .value
                        .get("fanartArtwork")
                        .cloned()
                        .unwrap_or(Value::Null);
                } else {
                    engine.state.detail.error = normalize_error(result.error.clone());
                }
            }
        }
        "prefetchDetailStreams" => {
            if generation == engine.state.runtime.get(GenerationKey::Detail) {
                if result.status.is_ok() {
                    engine.state.detail.last_prefetch = result.value.clone();
                } else {
                    engine.state.detail.last_prefetch_error = normalize_error(result.error.clone());
                }
            }
        }
        "fetchDetailStreams" => {
            if generation == engine.state.runtime.get(GenerationKey::DetailStreams) {
                engine.state.detail.is_loading_streams = false;
                if result.status.is_ok() {
                    engine.state.detail.streams = result
                        .value
                        .get("streams")
                        .cloned()
                        .unwrap_or_else(|| serde_json::json!([]));
                    engine.state.detail.visible_streams = visible_streams(
                        &engine.state.detail.streams,
                        engine.state.detail.selected_addon.as_str(),
                    );
                    engine.state.detail.available_addons = result
                        .value
                        .get("availableAddons")
                        .cloned()
                        .unwrap_or_else(|| serde_json::json!([]));
                    engine.state.detail.loading_addon_names = serde_json::json!([]);
                    engine.state.detail.failed_addons = result
                        .value
                        .get("failedAddons")
                        .cloned()
                        .unwrap_or_else(|| serde_json::json!([]));
                    engine.state.detail.resolved_request_id = result
                        .value
                        .get("resolvedRequestId")
                        .cloned()
                        .unwrap_or(Value::Null);
                    engine.state.detail.has_stream_providers = result
                        .value
                        .get("hasStreamProviders")
                        .cloned()
                        .unwrap_or(Value::Bool(false));
                    engine.state.detail.streams_error = Value::Null;
                } else {
                    engine.state.detail.streams_error = normalize_error(result.error.clone());
                    engine.state.detail.loading_addon_names = serde_json::json!([]);
                }
            }
        }
        "fetchMetaDetailLookup" => {
            if generation == engine.state.runtime.get(GenerationKey::Lookup) {
                if result.status.is_ok() {
                    *engine.state.lookup = LookupState {
                        trailers: normalize_meta_trailers(&result.value),
                        meta_detail: result.value.clone(),
                        error: Value::Null,
                    };
                } else {
                    *engine.state.lookup = LookupState {
                        trailers: serde_json::json!([]),
                        meta_detail: Value::Null,
                        error: normalize_error(result.error.clone()),
                    };
                }
            }
        }
        "fetchSeasonEpisodes" => {
            if generation == engine.state.runtime.get(GenerationKey::Detail) {
                engine.state.detail.season_loading = Value::Null;
                if result.status.is_ok() {
                    engine.state.detail.season_episodes = result
                        .value
                        .get("episodes")
                        .cloned()
                        .unwrap_or_else(|| result.value.clone());
                    engine.state.detail.error = Value::Null;
                } else {
                    engine.state.detail.error = normalize_error(result.error.clone());
                }
            }
        }
        _ => {}
    }
    vec![]
}
