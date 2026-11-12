use fluxa_core::{Engine, Update};
pub use fluxa_core::{HomeSlice, Snapshot};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::sync::Arc;

pub type SliceGenerations = BTreeMap<String, u64>;

pub struct RuntimeUpdate {
    pub effects: Vec<Value>,
}

pub struct FluxaRuntime {
    engine: Engine,
    snapshot: Snapshot,
    generations: Arc<SliceGenerations>,
    revision: u64,
}

impl FluxaRuntime {
    pub fn new(initial_state: Value) -> Result<Self, String> {
        let engine = Engine::new(initial_state)?;
        let snapshot = engine.shared_snapshot();
        Ok(Self {
            engine,
            snapshot,
            generations: Arc::default(),
            revision: 0,
        })
    }

    pub fn snapshot(&self) -> &Snapshot {
        &self.snapshot
    }

    pub fn snapshot_shared(&self) -> Snapshot {
        self.snapshot.clone()
    }

    pub fn revision(&self) -> u64 {
        self.revision
    }

    pub fn generations(&self) -> Arc<SliceGenerations> {
        Arc::clone(&self.generations)
    }

    fn bump<'a>(&mut self, slices: impl IntoIterator<Item = &'a str>) {
        let generations = Arc::make_mut(&mut self.generations);
        for slice in slices {
            *generations.entry(slice.to_owned()).or_default() += 1;
        }
    }

    pub fn dispatch(&mut self, action: Value) -> Result<RuntimeUpdate, String> {
        let update = self.engine.dispatch(action)?;
        Ok(self.apply(update))
    }

    pub fn complete_effect_result(
        &mut self,
        effect_id: &str,
        status: &str,
        value: Value,
        error: Value,
    ) -> Result<RuntimeUpdate, String> {
        let update = self
            .engine
            .complete(completion(effect_id, status, value, error))?;
        Ok(self.apply(update))
    }

    pub fn complete_discover_page_result(
        &mut self,
        effect_id: &str,
        status: &str,
        value: Value,
        error: Value,
    ) -> Result<RuntimeUpdate, String> {
        let page = self
            .engine
            .complete_page(completion(effect_id, status, value, error))?;
        self.revision = page.revision;
        self.bump(["discover"]);
        let Value::Object(mut delta) = page.delta else {
            return Err("Discover page delta is not an object".to_owned());
        };
        let applied = self.snapshot.edit("discover", |discover| {
            let Some(discover) = discover.as_object_mut() else {
                return;
            };
            if let Some(Value::Array(appended)) = delta.remove("appended")
                && let Some(results) = discover
                    .entry("results")
                    .or_insert_with(|| json!([]))
                    .as_array_mut()
            {
                results.extend(appended);
            }
            if let Some(Value::Object(sources)) = delta.remove("sources")
                && let Some(target) = discover
                    .entry("resultSources")
                    .or_insert_with(|| json!({}))
                    .as_object_mut()
            {
                for (key, source) in sources {
                    target.entry(key).or_insert(source);
                }
            }
            if let Some(paging) = discover
                .entry("paging")
                .or_insert_with(|| json!({}))
                .as_object_mut()
            {
                for (key, fallback) in [
                    ("isLoading", Value::Bool(false)),
                    ("nextSkip", Value::from(0)),
                    ("hasMore", Value::Bool(false)),
                    ("error", Value::Null),
                ] {
                    paging.insert(key.to_owned(), delta.remove(key).unwrap_or(fallback));
                }
            }
        });
        if !applied {
            return Err("Discover is not available right now.".to_owned());
        }
        Ok(RuntimeUpdate {
            effects: effect_values(page.effects),
        })
    }

    fn apply(&mut self, update: Update) -> RuntimeUpdate {
        self.bump(
            update
                .state
                .keys()
                .map(String::as_str)
                .chain(update.home.is_some().then_some("home")),
        );
        for (name, value) in update.state {
            self.snapshot.set(&name, value);
        }
        if let Some(home) = update.home {
            self.snapshot.set_home(home);
        }
        self.revision = update.revision;
        RuntimeUpdate {
            effects: effect_values(update.effects),
        }
    }
}

fn completion(effect_id: &str, status: &str, value: Value, error: Value) -> Value {
    json!({ "effectId": effect_id, "status": status, "value": value, "error": error })
}

fn effect_values(effects: Vec<fluxa_core::runtime::EffectEnvelope>) -> Vec<Value> {
    effects
        .iter()
        .filter_map(|effect| serde_json::to_value(effect).ok())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runtime_reads_and_updates_the_fluxa_core_snapshot() {
        let mut runtime = FluxaRuntime::new(json!({})).expect("create runtime");
        assert_eq!(runtime.snapshot()["navigation"]["route"], "home");

        let update = runtime
            .dispatch(json!({
                "type": "navigationRequested",
                "route": "discover",
                "params": null,
            }))
            .expect("dispatch navigation");

        assert!(update.effects.is_empty());
        assert_eq!(runtime.snapshot()["navigation"]["route"], "discover");
    }

    #[test]
    fn only_dirty_slices_advance_their_generation() {
        let mut runtime = FluxaRuntime::new(json!({})).expect("create runtime");
        assert!(runtime.generations().is_empty());
        runtime
            .dispatch(json!({
                "type": "navigationRequested",
                "route": "discover",
                "params": null,
            }))
            .expect("dispatch navigation");
        let after_first = runtime.generations();
        assert_eq!(after_first.get("navigation"), Some(&1));
        assert_eq!(after_first.get("library"), None);
        runtime
            .dispatch(json!({
                "type": "navigationRequested",
                "route": "home",
                "params": null,
            }))
            .expect("dispatch navigation");
        assert_eq!(runtime.generations().get("navigation"), Some(&2));
        assert_eq!(after_first.get("navigation"), Some(&1));
    }

    #[test]
    fn native_ui_fixture_is_accepted_by_core() {
        let fixture: Value =
            serde_json::from_str(include_str!("../../fixtures/native-ui-fixture.json"))
                .expect("valid native UI fixture JSON");
        let runtime = FluxaRuntime::new(fixture).expect("fixture should create a Core runtime");
        assert_eq!(
            runtime.snapshot()["home"]["billboard"]["id"],
            "fixture-featured"
        );
        assert_eq!(
            runtime.snapshot()["settings"]["values"]["cacheArtwork"],
            true
        );
    }
}
