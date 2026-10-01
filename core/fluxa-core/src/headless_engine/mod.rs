pub(crate) mod complete_effect;
pub(crate) mod contracts;
pub(crate) mod dispatch;
pub(crate) mod effect_bookkeeping;
pub(crate) mod helpers;
pub(crate) mod manifest;
pub(crate) mod navigation;
pub(crate) mod routes;
pub(crate) mod state;

pub(crate) use crate::accounts::engine::auth;
pub(crate) use crate::addons::engine::installed as addons;
pub(crate) use crate::addons::engine::plugins;
pub(crate) use crate::catalog::engine::detail;
pub(crate) use crate::catalog::engine::discover;
pub(crate) use crate::catalog::engine::search;
pub(crate) use crate::home::engine as home;
pub(crate) use crate::library::engine::calendar;
pub(crate) use crate::library::engine::library;
pub(crate) use crate::library::engine::offline;
pub(crate) use crate::player::engine as player;
pub(crate) use crate::player::engine::trailer;
pub(crate) use crate::profile::engine as profile;
pub(crate) use crate::settings::engine as settings;

use crate::runtime::{EffectEnvelope, EffectKind};
use contracts::{AppAction, DispatchResult};
use serde::Serialize;
use state::EngineState;
use std::collections::{HashMap, HashSet};
use std::time::Duration;
use web_time::Instant;

pub(crate) use contracts::EffectResultInput;

// Drops effects the platform never completed; real in-flight work finishes well inside it.
const EFFECT_EXPIRY: Duration = Duration::from_secs(300);

#[derive(Debug, Default)]
pub(crate) struct HeadlessEngine {
    pub(crate) state: EngineState,
    pub(crate) next_effect_id: u64,
    pub(crate) revision: u64,
    // Keeps the drain fallback in resolve_visible_effects from re-running in-flight effects.
    pub(crate) delivered_effect_ids: HashSet<String>,
    pub(crate) effect_created_at: HashMap<String, Instant>,
    // Runtime-only effect registry. Effect payloads can contain credentials and must never
    // be included in a UI state snapshot or StatePatch.
    pub(crate) pending_effects: Vec<EffectEnvelope>,
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
struct PageCompletionResult {
    pub(crate) revision: u64,
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

#[cfg(test)]
pub(crate) mod tests;
