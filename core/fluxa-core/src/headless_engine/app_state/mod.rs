use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock};

mod player;
mod reducer;

#[cfg(test)]
use player::*;
pub use reducer::*;

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
            crate::runtime::log_sink::record("create_app_core_state", &error.to_string());
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
            crate::runtime::log_sink::record("app_core_dispatch_json", &error.to_string());
        })
        .ok()?;
    let state = lock_store().get(&handle)?.clone();
    let mut state = lock_app_state(&state)?;
    if !reduce(&mut state, action) {
        crate::runtime::log_sink::record("app_core_dispatch_json", "unknown action");
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

fn lock_app_state(
    state: &Arc<Mutex<AppCoreState>>,
) -> Option<std::sync::MutexGuard<'_, AppCoreState>> {
    match state.lock() {
        Ok(guard) => Some(guard),
        Err(_) => {
            crate::runtime::log_sink::record(
                "app_core_state",
                "poisoned handle; recreate the app state",
            );
            None
        }
    }
}

#[cfg(test)]
mod tests;
