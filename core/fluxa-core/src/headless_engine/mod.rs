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
use crate::runtime::EffectEnvelope;
use contracts::{AppAction, DispatchResult, StatePatch};
use state::EngineState;
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Duration;
use web_time::Instant;

pub(crate) use contracts::EffectResultInput;

// If the platform never calls complete_effect for an effect (a transient IPC failure on
// the completion call, a swallowed exception, etc.), it would otherwise sit in
// pending_effects/delivered_effect_ids forever for the life of the engine instance.
// Anything genuinely still in flight completes well within this window.
const EFFECT_EXPIRY: Duration = Duration::from_secs(300);

#[derive(Debug, Default)]
struct HeadlessEngine {
    state: EngineState,
    next_effect_id: u64,
    revision: u64,
    // Ids handed to the platform at least once, awaiting their complete_effect call.
    // Never serialized — purely tracks delivery so the "drain the queue" fallback in
    // resolve_visible_effects doesn't hand out an effect that's already in flight as
    // if it were fresh work (which used to make an unrelated dispatch while a slow
    // effect was still running re-trigger a full duplicate execution of it).
    delivered_effect_ids: HashSet<String>,
    // When each pending effect was created, for expire_stale_pending_effects. Never
    // serialized — Instant isn't a portable wall-clock value, just an internal timer.
    effect_created_at: HashMap<String, Instant>,
    // Runtime-only effect registry. Effect payloads can contain credentials and must never
    // be included in a UI state snapshot or StatePatch.
    pending_effects: Vec<EffectEnvelope>,
}

static ENGINE_COUNTER: AtomicU64 = AtomicU64::new(1);
static ENGINES: OnceLock<Mutex<HashMap<u64, Arc<Mutex<HeadlessEngine>>>>> = OnceLock::new();

pub fn create_headless_engine(initial_json: &str) -> u64 {
    let mut engine = HeadlessEngine {
        next_effect_id: 1,
        ..HeadlessEngine::default()
    };
    let initial_state = match serde_json::from_str::<EngineState>(initial_json) {
        Ok(state) => state,
        Err(error) => {
            crate::log_sink::record("create_headless_engine", &error.to_string());
            return 0;
        }
    };
    engine.state = initial_state;
    let mut map = lock_engines();
    let handle = ENGINE_COUNTER.fetch_add(1, Ordering::Relaxed);
    map.insert(handle, Arc::new(Mutex::new(engine)));
    handle
}

pub fn destroy_headless_engine(handle: u64) -> bool {
    lock_engines().remove(&handle).is_some()
}

pub fn headless_engine_snapshot_json(handle: u64) -> Option<String> {
    let state = {
        let map = lock_engines();
        map.get(&handle)?.clone()
    };
    let state = lock_engine(&state)?.state.clone();
    serde_json::to_string(&state).ok()
}

pub fn headless_engine_dispatch_json(handle: u64, action_json: &str) -> Option<String> {
    let action: AppAction = serde_json::from_str(action_json)
        .map_err(|e| CoreError::BadInput {
            context: "headless_engine_dispatch_json",
            detail: e.to_string(),
        })
        .log_discard()?;
    let (revision, patch, visible_effects) = {
        let engine = match lock_engines().get(&handle) {
            Some(engine) => Arc::clone(engine),
            None => {
                return CoreError::NotFound {
                    context: "headless_engine_dispatch_json",
                }
                .log_and_none();
            }
        };
        let mut engine = lock_engine(&engine).or_else(|| {
            CoreError::NotFound {
                context: "headless_engine_dispatch_json (poisoned handle)",
            }
            .log_and_none()
        })?;
        engine.expire_stale_pending_effects(Instant::now());
        let effects = engine.dispatch(action);
        let visible_effects = engine.resolve_visible_effects(effects);
        engine.revision = engine.revision.saturating_add(1);
        (engine.revision, engine.state.diff_dirty(), visible_effects)
    };
    result_patch_json(revision, patch, visible_effects)
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
    let Some(engine) = lock_engines().get(&handle).cloned() else {
        return false;
    };
    let Some(mut engine) = lock_engine(&engine) else {
        return false;
    };
    update(&mut engine);
    true
}

pub fn headless_engine_complete_effect_json(handle: u64, result_json: &str) -> Option<String> {
    let result: EffectResultInput = serde_json::from_str(result_json)
        .map_err(|e| CoreError::BadInput {
            context: "headless_engine_complete_effect_json",
            detail: e.to_string(),
        })
        .log_discard()?;
    let (revision, patch, visible_effects) = {
        let engine = match lock_engines().get(&handle) {
            Some(engine) => Arc::clone(engine),
            None => {
                return CoreError::NotFound {
                    context: "headless_engine_complete_effect_json",
                }
                .log_and_none();
            }
        };
        let mut engine = lock_engine(&engine).or_else(|| {
            CoreError::NotFound {
                context: "headless_engine_complete_effect_json (poisoned handle)",
            }
            .log_and_none()
        })?;
        engine.expire_stale_pending_effects(Instant::now());
        let effects = engine.complete_effect(result);
        let visible_effects = engine.resolve_visible_effects(effects);
        engine.revision = engine.revision.saturating_add(1);
        (engine.revision, engine.state.diff_dirty(), visible_effects)
    };
    result_patch_json(revision, patch, visible_effects)
}

// Deliberately takes owned before/after snapshots rather than a reference to the locked
// engine: diffing and serializing a large state (e.g. a big discover catalog) can take
// over a second. Callers clone what they need and drop the per-engine lock before calling
// this, so unrelated engine handles continue to make progress.
fn result_patch_json(
    revision: u64,
    state: StatePatch,
    effects: Vec<EffectEnvelope>,
) -> Option<String> {
    serde_json::to_string(&DispatchResult {
        revision,
        state,
        effects,
    })
    .ok()
}

fn engines() -> &'static Mutex<HashMap<u64, Arc<Mutex<HeadlessEngine>>>> {
    ENGINES.get_or_init(|| Mutex::new(HashMap::new()))
}

fn lock_engine(
    engine: &Arc<Mutex<HeadlessEngine>>,
) -> Option<std::sync::MutexGuard<'_, HeadlessEngine>> {
    match engine.lock() {
        Ok(guard) => Some(guard),
        Err(_) => {
            crate::log_sink::record("headless_engine", "poisoned handle; recreate the engine");
            None
        }
    }
}

// A panic while a request held the registry lock poisons it; recover so a caught panic
// does not make every handle inaccessible. A poisoned engine itself is isolated to its
// own per-handle lock.
fn lock_engines() -> std::sync::MutexGuard<'static, HashMap<u64, Arc<Mutex<HeadlessEngine>>>> {
    engines()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

#[cfg(test)]
mod tests;
