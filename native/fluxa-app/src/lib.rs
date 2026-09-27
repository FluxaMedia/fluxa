use fluxa_core::{Engine, Update};
use serde_json::{Value, json};
use std::sync::Arc;

pub struct RuntimeUpdate {
    pub effects: Vec<Value>,
}

pub struct FluxaRuntime {
    engine: Engine,
    snapshot: Arc<Value>,
    revision: u64,
}

impl FluxaRuntime {
    pub fn new(initial_state: Value) -> Result<Self, String> {
        let engine = Engine::new(initial_state)?;
        let snapshot = Arc::new(engine.snapshot());
        Ok(Self {
            engine,
            snapshot,
            revision: 0,
        })
    }

    pub fn snapshot(&self) -> &Value {
        &self.snapshot
    }

    pub fn snapshot_shared(&self) -> Arc<Value> {
        Arc::clone(&self.snapshot)
    }

    pub fn revision(&self) -> u64 {
        self.revision
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
        let update = self.engine.complete(completion(effect_id, status, value, error))?;
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
        let discover = Arc::make_mut(&mut self.snapshot)
            .get_mut("discover")
            .and_then(Value::as_object_mut)
            .ok_or_else(|| "Discover state is missing from the Core snapshot".to_owned())?;
        let Value::Object(mut delta) = page.delta else {
            return Err("Discover page delta is not an object".to_owned());
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
        Ok(RuntimeUpdate {
            effects: effect_values(page.effects),
        })
    }

    fn apply(&mut self, update: Update) -> RuntimeUpdate {
        if !update.state.is_empty()
            && let Some(snapshot) = Arc::make_mut(&mut self.snapshot).as_object_mut()
        {
            snapshot.extend(update.state);
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
