use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppCoreState {
    #[serde(default)]
    pub home: HomeState,
    #[serde(default)]
    pub home_search: HomeSearchState,
    #[serde(default)]
    pub billboard: BillboardState,
    #[serde(default)]
    pub discover: DiscoverState,
    #[serde(default)]
    pub calendar: CalendarState,
    #[serde(default)]
    pub library: LibraryState,
    #[serde(default)]
    pub player: PlayerCoreState,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BillboardState {
    #[serde(default)]
    pub error: Value,
    #[serde(default)]
    pub pool: Value,
    #[serde(default)]
    pub index: i64,
    #[serde(default)]
    pub movie: Value,
    #[serde(default)]
    pub logo: Value,
    #[serde(default)]
    pub watchlist: bool,
    #[serde(default)]
    pub next_episode: Value,
    #[serde(default)]
    pub trailer_url: Value,
}

impl Default for BillboardState {
    fn default() -> Self {
        Self {
            error: Value::Null,
            pool: json!([]),
            index: 0,
            movie: Value::Null,
            logo: Value::Null,
            watchlist: false,
            next_episode: Value::Null,
            trailer_url: Value::Null,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscoverState {
    #[serde(default)]
    pub results: Value,
    #[serde(default)]
    pub is_loading: bool,
    #[serde(default)]
    pub genres: Value,
    #[serde(default)]
    pub catalogs: Value,
}

impl Default for DiscoverState {
    fn default() -> Self {
        Self {
            results: json!([]),
            is_loading: false,
            genres: json!([]),
            catalogs: json!([]),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CalendarState {
    #[serde(default)]
    pub items: Value,
    #[serde(default)]
    pub is_loading: bool,
}

impl Default for CalendarState {
    fn default() -> Self {
        Self {
            items: json!([]),
            is_loading: false,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryState {
    #[serde(default)]
    pub ui_state: Value,
}

impl Default for LibraryState {
    fn default() -> Self {
        Self {
            ui_state: json!({}),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HomeState {
    #[serde(default)]
    pub categories: Value,
    #[serde(default)]
    pub is_loading: bool,
    #[serde(default = "default_home_filter")]
    pub current_filter: String,
    #[serde(default)]
    pub is_direct_loading: bool,
    #[serde(default)]
    pub trakt_continue_watching_last_updated_at: i64,
    #[serde(default)]
    pub user_addons: Value,
    #[serde(default)]
    pub watchlist: Value,
    #[serde(default)]
    pub liked_items: Value,
    #[serde(default)]
    pub active_profile: Value,
    #[serde(default)]
    pub current_watchlist: Value,
    #[serde(default)]
    pub external_continue_watching: Value,
    #[serde(default)]
    pub trakt_watched_state: Value,
}

impl Default for HomeState {
    fn default() -> Self {
        Self {
            categories: json!([]),
            is_loading: false,
            current_filter: default_home_filter(),
            is_direct_loading: false,
            trakt_continue_watching_last_updated_at: 0,
            user_addons: json!([]),
            watchlist: json!([]),
            liked_items: json!([]),
            active_profile: Value::Null,
            current_watchlist: json!([]),
            external_continue_watching: json!([]),
            trakt_watched_state: json!({}),
        }
    }
}

fn default_home_filter() -> String {
    "all".to_string()
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HomeSearchState {
    #[serde(default)]
    pub search_results: Value,
    #[serde(default)]
    pub search_rows: Value,
    #[serde(default)]
    pub search_history: Value,
    #[serde(default)]
    pub focused_movie: Value,
    #[serde(default)]
    pub focused_movie_trailer_url: Value,
    #[serde(default)]
    pub preview_url: Value,
}

impl Default for HomeSearchState {
    fn default() -> Self {
        Self {
            search_results: json!([]),
            search_rows: json!([]),
            search_history: json!([]),
            focused_movie: Value::Null,
            focused_movie_trailer_url: Value::Null,
            preview_url: Value::Null,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerCoreState {
    #[serde(default)]
    pub current_video_id: Value,
    #[serde(default)]
    pub current_stream_index: i64,
    #[serde(default)]
    pub last_saved_position: i64,
    #[serde(default)]
    pub should_apply_initial_progress: bool,
    #[serde(default)]
    pub playback_ended: bool,
    #[serde(default)]
    pub has_started_playing: bool,
    #[serde(default)]
    pub is_video_rendered: bool,
    #[serde(default = "default_buffering")]
    pub is_buffering: bool,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

impl Default for PlayerCoreState {
    fn default() -> Self {
        Self {
            current_video_id: Value::Null,
            current_stream_index: 0,
            last_saved_position: 0,
            should_apply_initial_progress: false,
            playback_ended: false,
            has_started_playing: false,
            is_video_rendered: false,
            is_buffering: true,
            extra: HashMap::new(),
        }
    }
}

fn default_buffering() -> bool {
    true
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AppCoreAction {
    #[serde(rename = "type")]
    action_type: String,
    #[serde(default)]
    value: Value,
    #[serde(default)]
    video_id: Value,
}

static NEXT_HANDLE: AtomicU64 = AtomicU64::new(1);
static STORE: OnceLock<Mutex<HashMap<u64, Arc<Mutex<AppCoreState>>>>> = OnceLock::new();

fn store() -> &'static Mutex<HashMap<u64, Arc<Mutex<AppCoreState>>>> {
    STORE.get_or_init(|| Mutex::new(HashMap::new()))
}

// See headless_engine::lock_engines — recovering from poison keeps this store
// usable after a single caught panic instead of going dark for every handle.
fn lock_store() -> std::sync::MutexGuard<'static, HashMap<u64, Arc<Mutex<AppCoreState>>>> {
    store()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

pub fn create_app_core_state(initial_json: &str) -> u64 {
    let state = match serde_json::from_str(initial_json) {
        Ok(state) => state,
        Err(error) => {
            crate::log_sink::record("create_app_core_state", &error.to_string());
            return 0;
        }
    };
    let mut states = lock_store();
    let handle = NEXT_HANDLE.fetch_add(1, Ordering::Relaxed);
    states.insert(handle, Arc::new(Mutex::new(state)));
    handle
}

pub fn destroy_app_core_state(handle: u64) -> bool {
    lock_store().remove(&handle).is_some()
}

pub fn app_core_state_json(handle: u64) -> Option<String> {
    let state = lock_store().get(&handle)?.clone();
    let state = lock_app_state(&state)?;
    serde_json::to_string(&*state).ok()
}

pub fn app_core_dispatch_json(handle: u64, action_json: &str) -> Option<String> {
    let action: AppCoreAction = serde_json::from_str(action_json)
        .map_err(|error| {
            crate::log_sink::record("app_core_dispatch_json", &error.to_string());
        })
        .ok()?;
    let state = lock_store().get(&handle)?.clone();
    let mut state = lock_app_state(&state)?;
    if !reduce(&mut state, action) {
        crate::log_sink::record("app_core_dispatch_json", "unknown action");
        return None;
    }
    serde_json::to_string(&*state).ok()
}

pub fn app_core_dispatch_delta_json(handle: u64, action_json: &str) -> Option<String> {
    let action: AppCoreAction = serde_json::from_str(action_json).ok()?;
    let action_type = action.action_type.clone();
    let state = lock_store().get(&handle)?.clone();
    let mut state = lock_app_state(&state)?;
    if !reduce(&mut state, action) {
        return None;
    }
    let patch = action_patch(&action_type, &state);
    Some(json!({ "patch": patch }).to_string())
}

fn action_patch(action_type: &str, state: &AppCoreState) -> Value {
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

pub fn app_core_set_player_position(handle: u64, position_ms: i64) -> bool {
    update_player(handle, |player| player.last_saved_position = position_ms)
}

pub fn app_core_set_player_buffering(handle: u64, buffering: bool) -> bool {
    update_player(handle, |player| player.is_buffering = buffering)
}

pub fn app_core_set_player_stream_index(handle: u64, stream_index: i64) -> bool {
    update_player(handle, |player| player.current_stream_index = stream_index)
}

pub fn app_core_set_player_playback_ended(handle: u64, ended: bool) -> bool {
    update_player(handle, |player| player.playback_ended = ended)
}

pub fn app_core_set_player_video_rendered(handle: u64, rendered: bool) -> bool {
    update_player(handle, |player| player.is_video_rendered = rendered)
}

pub fn app_core_set_player_started(handle: u64, started: bool) -> bool {
    update_player(handle, |player| player.has_started_playing = started)
}

pub fn app_core_update_player(
    handle: u64,
    position_ms: i64,
    stream_index: i64,
    buffering: bool,
    playback_ended: bool,
    started: bool,
    rendered: bool,
) -> bool {
    let Some(state) = lock_store().get(&handle).cloned() else {
        return false;
    };
    let Some(mut state) = lock_app_state(&state) else {
        return false;
    };
    state.player.last_saved_position = position_ms;
    state.player.current_stream_index = stream_index;
    state.player.is_buffering = buffering;
    state.player.playback_ended = playback_ended;
    state.player.has_started_playing = started;
    state.player.is_video_rendered = rendered;
    true
}

fn update_player(handle: u64, update: impl FnOnce(&mut PlayerCoreState)) -> bool {
    let Some(state) = lock_store().get(&handle).cloned() else {
        return false;
    };
    let Some(mut state) = lock_app_state(&state) else {
        return false;
    };
    update(&mut state.player);
    true
}

fn lock_app_state(
    state: &Arc<Mutex<AppCoreState>>,
) -> Option<std::sync::MutexGuard<'_, AppCoreState>> {
    match state.lock() {
        Ok(guard) => Some(guard),
        Err(_) => {
            crate::log_sink::record("app_core_state", "poisoned handle; recreate the app state");
            None
        }
    }
}

fn reduce(state: &mut AppCoreState, action: AppCoreAction) -> bool {
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

fn array_or_empty(value: Value) -> Value {
    if value.is_array() { value } else { json!([]) }
}

fn reset_player_for_episode(player: &mut PlayerCoreState, video_id: Value) {
    player.current_video_id = video_id;
    player.current_stream_index = 0;
    player.last_saved_position = 0;
    player.should_apply_initial_progress = false;
    player.playback_ended = false;
    player.has_started_playing = false;
    player.is_video_rendered = false;
    player.is_buffering = true;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reducer_updates_home_search_state_without_reordering_payloads() {
        let handle = create_app_core_state(
            r#"{"home":{"categories":[{"id":"c1"}]},"homeSearch":{"searchHistory":[{"id":"tt1"}]}}"#,
        );

        let snapshot = app_core_dispatch_json(
            handle,
            r#"{"type":"setSearchResults","value":[{"id":"tt2"},{"id":"tt1"}]}"#,
        )
        .unwrap();
        let value: Value = serde_json::from_str(&snapshot).unwrap();

        assert_eq!(
            value["homeSearch"]["searchResults"],
            json!([{"id":"tt2"},{"id":"tt1"}])
        );
        assert_eq!(value["home"]["categories"], json!([{"id":"c1"}]));
        assert_eq!(value["homeSearch"]["searchHistory"], json!([{"id":"tt1"}]));
        assert!(destroy_app_core_state(handle));
    }

    #[test]
    fn reducer_owns_home_shell_state() {
        let handle = create_app_core_state("{}");

        let snapshot = app_core_dispatch_json(
            handle,
            r#"{"type":"setHomeCategories","value":[{"id":"continue_watching"},{"id":"popular"}]}"#,
        )
        .unwrap();
        let value: Value = serde_json::from_str(&snapshot).unwrap();
        assert_eq!(
            value["home"]["categories"],
            json!([{"id":"continue_watching"},{"id":"popular"}])
        );

        let snapshot = app_core_dispatch_json(
            handle,
            r#"{"type":"setHomeCurrentFilter","value":"movies"}"#,
        )
        .unwrap();
        let value: Value = serde_json::from_str(&snapshot).unwrap();
        assert_eq!(value["home"]["currentFilter"], json!("movies"));

        let snapshot =
            app_core_dispatch_json(handle, r#"{"type":"setHomeLoading","value":true}"#).unwrap();
        let value: Value = serde_json::from_str(&snapshot).unwrap();
        assert_eq!(value["home"]["isLoading"], json!(true));
        assert_eq!(value["home"]["currentFilter"], json!("movies"));
        assert!(destroy_app_core_state(handle));
    }

    #[test]
    fn reducer_owns_home_feature_state_branches() {
        let handle = create_app_core_state("{}");

        let snapshot =
            app_core_dispatch_json(handle, r#"{"type":"setBillboardIndex","value":2}"#).unwrap();
        let value: Value = serde_json::from_str(&snapshot).unwrap();
        assert_eq!(value["billboard"]["index"], json!(2));

        let snapshot = app_core_dispatch_json(
            handle,
            r#"{"type":"setDiscoverResults","value":[{"id":"tt1"},{"id":"tt2"}]}"#,
        )
        .unwrap();
        let value: Value = serde_json::from_str(&snapshot).unwrap();
        assert_eq!(
            value["discover"]["results"],
            json!([{"id":"tt1"},{"id":"tt2"}])
        );

        let snapshot = app_core_dispatch_json(
            handle,
            r#"{"type":"setCalendarItems","value":[{"title":"Episode"}]}"#,
        )
        .unwrap();
        let value: Value = serde_json::from_str(&snapshot).unwrap();
        assert_eq!(value["calendar"]["items"], json!([{"title":"Episode"}]));

        let snapshot = app_core_dispatch_json(
            handle,
            r#"{"type":"setLibraryUiState","value":{"isLoading":false,"lastLoadedProfileKey":"profile"}}"#,
        )
        .unwrap();
        let value: Value = serde_json::from_str(&snapshot).unwrap();
        assert_eq!(
            value["library"]["uiState"]["lastLoadedProfileKey"],
            json!("profile")
        );
        assert!(destroy_app_core_state(handle));
    }

    #[test]
    fn reducer_resets_player_episode_state_like_kotlin_state_holder() {
        let handle = create_app_core_state(
            r#"{"player":{"currentStreamIndex":3,"lastSavedPosition":9200,"playbackEnded":true,"hasStartedPlaying":true,"isVideoRendered":true,"isBuffering":false}}"#,
        );

        let snapshot = app_core_dispatch_json(
            handle,
            r#"{"type":"playerResetForEpisode","videoId":"tt123:1:2"}"#,
        )
        .unwrap();
        let value: Value = serde_json::from_str(&snapshot).unwrap();

        assert_eq!(value["player"]["currentVideoId"], json!("tt123:1:2"));
        assert_eq!(value["player"]["currentStreamIndex"], json!(0));
        assert_eq!(value["player"]["lastSavedPosition"], json!(0));
        assert_eq!(value["player"]["shouldApplyInitialProgress"], json!(false));
        assert_eq!(value["player"]["playbackEnded"], json!(false));
        assert_eq!(value["player"]["hasStartedPlaying"], json!(false));
        assert_eq!(value["player"]["isVideoRendered"], json!(false));
        assert_eq!(value["player"]["isBuffering"], json!(true));
        assert!(destroy_app_core_state(handle));
    }

    #[test]
    fn primitive_player_updates_avoid_json_dispatch() {
        let handle = create_app_core_state("{}");
        assert!(app_core_update_player(
            handle, 12_345, 2, false, true, true, true
        ));
        let value: Value = serde_json::from_str(&app_core_state_json(handle).unwrap()).unwrap();
        assert_eq!(value["player"]["lastSavedPosition"], json!(12_345));
        assert_eq!(value["player"]["isBuffering"], json!(false));
        assert_eq!(value["player"]["currentStreamIndex"], json!(2));
        assert_eq!(value["player"]["playbackEnded"], json!(true));
        assert_eq!(value["player"]["isVideoRendered"], json!(true));
        assert_eq!(value["player"]["hasStartedPlaying"], json!(true));
        assert!(destroy_app_core_state(handle));
    }

    #[test]
    fn delta_dispatch_returns_the_small_action_payload() {
        let handle = create_app_core_state("{}");
        let delta =
            app_core_dispatch_delta_json(handle, r#"{"type":"setHomeLoading","value":true}"#)
                .unwrap();
        assert_eq!(delta, r#"{"patch":{"home":{"isLoading":true}}}"#);
        assert!(destroy_app_core_state(handle));
    }
}
