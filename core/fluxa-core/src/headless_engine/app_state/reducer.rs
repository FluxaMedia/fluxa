use super::*;

pub(super) fn action_patch(action_type: &str, state: &AppCoreState) -> Value {
    match action_type {
        "setHomeCategories" => json!({"home": {"categories": state.home.categories}}),
        "setHomeLoading" => json!({"home": {"isLoading": state.home.is_loading}}),
        "setHomeCurrentFilter" => json!({"home": {"currentFilter": state.home.current_filter}}),
        "setHomeDirectLoading" => {
            json!({"home": {"isDirectLoading": state.home.is_direct_loading}})
        }
        "setTraktContinueWatchingLastUpdatedAt" => {
            json!({"home": {"traktContinueWatchingLastUpdatedAt": state.home.trakt_continue_watching_last_updated_at}})
        }
        "setUserAddons" => json!({"home": {"userAddons": state.home.user_addons}}),
        "setWatchlist" => json!({"home": {"watchlist": state.home.watchlist}}),
        "setLikedItems" => json!({"home": {"likedItems": state.home.liked_items}}),
        "setActiveProfile" => json!({"home": {"activeProfile": state.home.active_profile}}),
        "setCurrentWatchlist" => {
            json!({"home": {"currentWatchlist": state.home.current_watchlist}})
        }
        "setExternalContinueWatching" => {
            json!({"home": {"externalContinueWatching": state.home.external_continue_watching}})
        }
        "setTraktWatchedState" => {
            json!({"home": {"traktWatchedState": state.home.trakt_watched_state}})
        }
        "setSearchResults" => {
            json!({"homeSearch": {"searchResults": state.home_search.search_results}})
        }
        "setSearchRows" => json!({"homeSearch": {"searchRows": state.home_search.search_rows}}),
        "setSearchHistory" => {
            json!({"homeSearch": {"searchHistory": state.home_search.search_history}})
        }
        "setFocusedMovie" => {
            json!({"homeSearch": {"focusedMovie": state.home_search.focused_movie}})
        }
        "setFocusedMovieTrailerUrl" => {
            json!({"homeSearch": {"focusedMovieTrailerUrl": state.home_search.focused_movie_trailer_url}})
        }
        "setPreviewUrl" => json!({"homeSearch": {"previewUrl": state.home_search.preview_url}}),
        "setBillboardError" => json!({"billboard": {"error": state.billboard.error}}),
        "setBillboardPool" => json!({"billboard": {"pool": state.billboard.pool}}),
        "setBillboardIndex" => json!({"billboard": {"index": state.billboard.index}}),
        "setBillboardMovie" => json!({"billboard": {"movie": state.billboard.movie}}),
        "setBillboardLogo" => json!({"billboard": {"logo": state.billboard.logo}}),
        "setBillboardWatchlist" => json!({"billboard": {"watchlist": state.billboard.watchlist}}),
        "setBillboardNextEpisode" => {
            json!({"billboard": {"nextEpisode": state.billboard.next_episode}})
        }
        "setBillboardTrailerUrl" => {
            json!({"billboard": {"trailerUrl": state.billboard.trailer_url}})
        }
        "setDiscoverResults" => json!({"discover": {"results": state.discover.results}}),
        "setDiscoverLoading" => json!({"discover": {"isLoading": state.discover.is_loading}}),
        "setDiscoverGenres" => json!({"discover": {"genres": state.discover.genres}}),
        "setDiscoverCatalogs" => json!({"discover": {"catalogs": state.discover.catalogs}}),
        "setCalendarItems" => json!({"calendar": {"items": state.calendar.items}}),
        "setCalendarLoading" => json!({"calendar": {"isLoading": state.calendar.is_loading}}),
        "setLibraryUiState" => json!({"library": {"uiState": state.library.ui_state}}),
        "playerResetForEpisode" => {
            return json!({
                "player": {
                    "currentVideoId": state.player.current_video_id,
                    "currentStreamIndex": state.player.current_stream_index,
                    "lastSavedPosition": state.player.last_saved_position,
                    "shouldApplyInitialProgress": state.player.should_apply_initial_progress,
                    "playbackEnded": state.player.playback_ended,
                    "hasStartedPlaying": state.player.has_started_playing,
                    "isVideoRendered": state.player.is_video_rendered,
                    "isBuffering": state.player.is_buffering
                }
            });
        }
        _ => return json!({}),
    }
}

pub(super) fn reduce(state: &mut AppCoreState, action: AppCoreAction) -> bool {
    match action.action_type.as_str() {
        "setHomeCategories" => state.home.categories = array_or_empty(action.value),
        "setHomeLoading" => state.home.is_loading = action.value.as_bool().unwrap_or(false),
        "setHomeCurrentFilter" => {
            state.home.current_filter = action
                .value
                .as_str()
                .filter(|value| !value.is_empty())
                .unwrap_or("all")
                .to_string()
        }
        "setHomeDirectLoading" => {
            state.home.is_direct_loading = action.value.as_bool().unwrap_or(false)
        }
        "setTraktContinueWatchingLastUpdatedAt" => {
            state.home.trakt_continue_watching_last_updated_at = action.value.as_i64().unwrap_or(0)
        }
        "setUserAddons" => state.home.user_addons = array_or_empty(action.value),
        "setWatchlist" => state.home.watchlist = array_or_empty(action.value),
        "setLikedItems" => state.home.liked_items = array_or_empty(action.value),
        "setActiveProfile" => state.home.active_profile = action.value,
        "setCurrentWatchlist" => state.home.current_watchlist = array_or_empty(action.value),
        "setExternalContinueWatching" => {
            state.home.external_continue_watching = array_or_empty(action.value)
        }
        "setTraktWatchedState" => state.home.trakt_watched_state = action.value,
        "setSearchResults" => state.home_search.search_results = array_or_empty(action.value),
        "setSearchRows" => state.home_search.search_rows = array_or_empty(action.value),
        "setSearchHistory" => state.home_search.search_history = array_or_empty(action.value),
        "setFocusedMovie" => state.home_search.focused_movie = action.value,
        "setFocusedMovieTrailerUrl" => state.home_search.focused_movie_trailer_url = action.value,
        "setPreviewUrl" => state.home_search.preview_url = action.value,
        "setBillboardError" => state.billboard.error = action.value,
        "setBillboardPool" => state.billboard.pool = array_or_empty(action.value),
        "setBillboardIndex" => state.billboard.index = action.value.as_i64().unwrap_or(0).max(0),
        "setBillboardMovie" => state.billboard.movie = action.value,
        "setBillboardLogo" => state.billboard.logo = action.value,
        "setBillboardWatchlist" => {
            state.billboard.watchlist = action.value.as_bool().unwrap_or(false)
        }
        "setBillboardNextEpisode" => state.billboard.next_episode = action.value,
        "setBillboardTrailerUrl" => state.billboard.trailer_url = action.value,
        "setDiscoverResults" => state.discover.results = array_or_empty(action.value),
        "setDiscoverLoading" => state.discover.is_loading = action.value.as_bool().unwrap_or(false),
        "setDiscoverGenres" => state.discover.genres = array_or_empty(action.value),
        "setDiscoverCatalogs" => state.discover.catalogs = array_or_empty(action.value),
        "setCalendarItems" => state.calendar.items = array_or_empty(action.value),
        "setCalendarLoading" => state.calendar.is_loading = action.value.as_bool().unwrap_or(false),
        "setLibraryUiState" => state.library.ui_state = action.value,
        "playerResetForEpisode" => reset_player_for_episode(&mut state.player, action.video_id),
        _ => return false,
    }
    true
}

pub(super) fn array_or_empty(value: Value) -> Value {
    if value.is_array() { value } else { json!([]) }
}

pub(super) fn reset_player_for_episode(player: &mut PlayerCoreState, video_id: Value) {
    player.current_video_id = video_id;
    player.current_stream_index = 0;
    player.last_saved_position = 0;
    player.should_apply_initial_progress = false;
    player.playback_ended = false;
    player.has_started_playing = false;
    player.is_video_rendered = false;
    player.is_buffering = true;
}
