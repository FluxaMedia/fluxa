mod addons;
mod auth;
mod calendar;
mod complete_effect;
mod contracts;
mod detail;
mod discover;
mod dispatch;
mod effect_bookkeeping;
mod helpers;
mod home;
mod library;
mod navigation;
mod offline;
mod player;
mod plugins;
mod profile;
mod search;
mod settings;
mod state;
mod sync;
mod trailer;
#[cfg(feature = "plugin-js-engine")]
mod youtube_cipher;

use crate::core_error::{CoreError, LogAndDiscard};
use crate::runtime::{EffectEnvelope, EffectKind};
use contracts::{AppAction, DispatchResult};
use serde::Serialize;
use state::EngineState;
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Duration;
use web_time::Instant;

pub(crate) use contracts::EffectResultInput;

// Drops effects the platform never completed; real in-flight work finishes well inside it.
const EFFECT_EXPIRY: Duration = Duration::from_secs(300);

#[derive(Debug, Default)]
struct HeadlessEngine {
    state: EngineState,
    next_effect_id: u64,
    revision: u64,
    // Keeps the drain fallback in resolve_visible_effects from re-running in-flight effects.
    delivered_effect_ids: HashSet<String>,
    effect_created_at: HashMap<String, Instant>,
    // Runtime-only effect registry. Effect payloads can contain credentials and must never
    // be included in a UI state snapshot or StatePatch.
    pending_effects: Vec<EffectEnvelope>,
}

pub struct Engine(HeadlessEngine);

pub struct Update {
    pub revision: u64,
    pub state: serde_json::Map<String, serde_json::Value>,
    pub effects: Vec<EffectEnvelope>,
}

pub struct PageUpdate {
    pub revision: u64,
    pub delta: serde_json::Value,
    pub effects: Vec<EffectEnvelope>,
}

impl Engine {
    pub fn new(initial: serde_json::Value) -> Result<Self, String> {
        serde_json::from_value(initial)
            .map(|state| Self(HeadlessEngine::new(state)))
            .map_err(|error| error.to_string())
    }

    pub fn snapshot(&self) -> serde_json::Value {
        serde_json::to_value(&self.0.state).unwrap_or_default()
    }

    pub fn dispatch(&mut self, action: serde_json::Value) -> Result<Update, String> {
        let action = serde_json::from_value(action).map_err(|error| error.to_string())?;
        Ok(Update::from(self.0.apply(action)))
    }

    pub fn dispatch_page_request(
        &mut self,
        action: serde_json::Value,
    ) -> Result<(u64, Vec<EffectEnvelope>), String> {
        let action = serde_json::from_value(action).map_err(|error| error.to_string())?;
        self.0.apply_page_request(action)
    }

    pub fn complete(&mut self, result: serde_json::Value) -> Result<Update, String> {
        let result = serde_json::from_value(result).map_err(|error| error.to_string())?;
        Ok(Update::from(self.0.complete(result)))
    }

    pub fn complete_page(&mut self, result: serde_json::Value) -> Result<PageUpdate, String> {
        let result = serde_json::from_value(result).map_err(|error| error.to_string())?;
        let page = self.0.complete_page(result)?;
        Ok(PageUpdate {
            revision: page.revision,
            delta: serde_json::to_value(&page.delta).unwrap_or_default(),
            effects: page.effects,
        })
    }
}

impl From<DispatchResult> for Update {
    fn from(result: DispatchResult) -> Self {
        let state = match serde_json::to_value(&result.state) {
            Ok(serde_json::Value::Object(state)) => state,
            _ => serde_json::Map::new(),
        };
        Self {
            revision: result.revision,
            state,
            effects: result.effects,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct EffectsOnlyResult {
    revision: u64,
    effects: Vec<EffectEnvelope>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct PageCompletionResult {
    revision: u64,
    delta: discover::DiscoverPageDelta,
    effects: Vec<EffectEnvelope>,
}

impl HeadlessEngine {
    fn new(state: EngineState) -> Self {
        Self {
            state,
            next_effect_id: 1,
            ..Self::default()
        }
    }

    fn step<T>(&mut self, run: impl FnOnce(&mut Self) -> T) -> (u64, T) {
        self.expire_stale_pending_effects(Instant::now());
        let value = run(self);
        self.revision = self.revision.saturating_add(1);
        (self.revision, value)
    }

    fn apply(&mut self, action: AppAction) -> DispatchResult {
        let (revision, (state, effects)) = self.step(|engine| {
            let effects = engine.dispatch(action);
            let effects = engine.resolve_visible_effects(effects);
            (engine.state.diff_dirty(), effects)
        });
        DispatchResult {
            revision,
            state,
            effects,
        }
    }

    fn apply_page_request(
        &mut self,
        action: AppAction,
    ) -> Result<(u64, Vec<EffectEnvelope>), String> {
        if !matches!(&action, AppAction::DiscoverPageRequested { .. }) {
            return Err("effects-only dispatch is reserved for Discover page requests".to_owned());
        }
        Ok(self.step(|engine| {
            let effects = engine.dispatch(action);
            let effects = engine.resolve_visible_effects(effects);
            drop(engine.state.diff_dirty());
            effects
        }))
    }

    fn complete(&mut self, result: EffectResultInput) -> DispatchResult {
        let (revision, (state, effects)) = self.step(|engine| {
            let effects = engine.complete_effect(result);
            let effects = engine.resolve_visible_effects(effects);
            (engine.state.diff_dirty(), effects)
        });
        DispatchResult {
            revision,
            state,
            effects,
        }
    }

    fn complete_page(&mut self, result: EffectResultInput) -> Result<PageCompletionResult, String> {
        self.expire_stale_pending_effects(Instant::now());
        if !self.pending_effects.iter().any(|effect| {
            effect.id == result.effect_id && effect.kind == EffectKind::FetchDiscoverPage
        }) {
            return Err("effect id is not a pending Discover page request".to_owned());
        }
        let (revision, (delta, effects)) = self.step(|engine| {
            let effects = engine.complete_effect(result);
            let effects = engine.resolve_visible_effects(effects);
            let delta = discover::take_page_delta(engine);
            drop(engine.state.diff_dirty());
            (delta, effects)
        });
        Ok(PageCompletionResult {
            revision,
            delta,
            effects,
        })
    }
}

static ENGINE_COUNTER: AtomicU64 = AtomicU64::new(1);
static ENGINES: OnceLock<Mutex<HashMap<u64, Arc<Mutex<HeadlessEngine>>>>> = OnceLock::new();

pub fn create_headless_engine(initial_json: &str) -> u64 {
    let state = match serde_json::from_str::<EngineState>(initial_json) {
        Ok(state) => state,
        Err(error) => {
            crate::log_sink::record("create_headless_engine", &error.to_string());
            return 0;
        }
    };
    let handle = ENGINE_COUNTER.fetch_add(1, Ordering::Relaxed);
    lock_engines().insert(handle, Arc::new(Mutex::new(HeadlessEngine::new(state))));
    handle
}

pub fn destroy_headless_engine(handle: u64) -> bool {
    lock_engines().remove(&handle).is_some()
}

pub fn headless_engine_snapshot_json(handle: u64) -> Option<String> {
    let state = with_engine(handle, "headless_engine_snapshot_json", |engine| {
        engine.state.clone()
    })?;
    serde_json::to_string(&state).ok()
}

pub fn headless_engine_dispatch_json(handle: u64, action_json: &str) -> Option<String> {
    const CONTEXT: &str = "headless_engine_dispatch_json";
    let action: AppAction = parse(CONTEXT, action_json)?;
    let result = with_engine(handle, CONTEXT, |engine| engine.apply(action))?;
    serde_json::to_string(&result).ok()
}

pub fn headless_engine_dispatch_effects_json(handle: u64, action_json: &str) -> Option<String> {
    const CONTEXT: &str = "headless_engine_dispatch_effects_json";
    let action: AppAction = parse(CONTEXT, action_json)?;
    let (revision, effects) =
        with_engine(handle, CONTEXT, |engine| engine.apply_page_request(action))?
            .map_err(|detail| CoreError::BadInput {
                context: CONTEXT,
                detail,
            })
            .log_discard()?;
    serde_json::to_string(&EffectsOnlyResult { revision, effects }).ok()
}

pub fn headless_engine_set_player_buffering(handle: u64, buffering: bool) -> bool {
    update_player(handle, |engine| player::set_buffering(engine, buffering))
}

pub fn headless_engine_set_player_stream_index(handle: u64, stream_index: i64) -> bool {
    update_player(handle, |engine| {
        player::set_stream_index(engine, stream_index)
    })
}

pub fn headless_engine_set_player_position(handle: u64, position_ms: i64) -> bool {
    update_player(handle, |engine| player::set_position(engine, position_ms))
}

fn update_player(handle: u64, update: impl FnOnce(&mut HeadlessEngine)) -> bool {
    with_engine(handle, "headless_engine_update_player", update).is_some()
}

pub fn headless_engine_complete_effect_json(handle: u64, result_json: &str) -> Option<String> {
    const CONTEXT: &str = "headless_engine_complete_effect_json";
    let result: EffectResultInput = parse(CONTEXT, result_json)?;
    let result = with_engine(handle, CONTEXT, |engine| engine.complete(result))?;
    serde_json::to_string(&result).ok()
}

pub fn headless_engine_complete_discover_page_json(
    handle: u64,
    result_json: &str,
) -> Option<String> {
    const CONTEXT: &str = "headless_engine_complete_discover_page_json";
    let result: EffectResultInput = parse(CONTEXT, result_json)?;
    let page = with_engine(handle, CONTEXT, |engine| engine.complete_page(result))?
        .map_err(|detail| CoreError::BadInput {
            context: CONTEXT,
            detail,
        })
        .log_discard()?;
    serde_json::to_string(&page).ok()
}

fn parse<T: serde::de::DeserializeOwned>(context: &'static str, json: &str) -> Option<T> {
    serde_json::from_str(json)
        .map_err(|error| CoreError::BadInput {
            context,
            detail: error.to_string(),
        })
        .log_discard()
}

fn with_engine<T>(
    handle: u64,
    context: &'static str,
    run: impl FnOnce(&mut HeadlessEngine) -> T,
) -> Option<T> {
    let Some(engine) = lock_engines().get(&handle).cloned() else {
        return CoreError::NotFound { context }.log_and_none();
    };
    let mut engine = match engine.lock() {
        Ok(engine) => engine,
        Err(_) => {
            crate::log_sink::record(context, "poisoned handle; recreate the engine");
            return None;
        }
    };
    Some(run(&mut engine))
}

// A panic while a request held the registry lock poisons it; recover so a caught panic
// does not make every handle inaccessible.
fn engines() -> &'static Mutex<HashMap<u64, Arc<Mutex<HeadlessEngine>>>> {
    ENGINES.get_or_init(|| Mutex::new(HashMap::new()))
}

fn lock_engines() -> std::sync::MutexGuard<'static, HashMap<u64, Arc<Mutex<HeadlessEngine>>>> {
    engines()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

#[cfg(test)]
mod tests;
