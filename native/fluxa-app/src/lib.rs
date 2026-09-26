use fluxa_core::FluxaCore;
use serde_json::{Value, json};
use std::sync::Arc;

pub struct RuntimeUpdate {
    pub effects: Vec<Value>,
}

/// Host-independent commands. JSON remains the wire format at the Core
/// boundary, but hosts can migrate away from ad-hoc action objects one command
/// at a time.
#[derive(Clone, Debug)]
pub enum AppCommand {
    Navigate {
        route: String,
        params: Value,
    },
    HomeLoad {
        profile: Value,
        language: String,
        force: bool,
    },
    LibraryHydrate {
        profile_id: Option<String>,
    },
    DiscoverRequest {
        content_type: String,
        filters: Value,
        load_catalog_filters: bool,
        profile: Option<Value>,
        language: Option<String>,
    },
    CalendarMonth {
        year: i32,
        month: i32,
    },
}

impl AppCommand {
    fn into_json(self) -> Value {
        match self {
            Self::Navigate { route, params } => {
                json!({ "type": "navigationRequested", "route": route, "params": params })
            }
            Self::HomeLoad {
                profile,
                language,
                force,
            } => {
                json!({ "type": "homeLoadRequested", "profile": profile, "language": language, "force": force })
            }
            Self::LibraryHydrate { profile_id } => {
                json!({ "type": "libraryHydrateRequested", "profileId": profile_id })
            }
            Self::DiscoverRequest {
                content_type,
                filters,
                load_catalog_filters,
                profile,
                language,
            } => {
                json!({
                    "type": "discoverRequested",
                    "contentType": content_type,
                    "filters": filters,
                    "loadCatalogFilters": load_catalog_filters,
                    "profile": profile,
                    "language": language,
                })
            }
            Self::CalendarMonth { year, month } => {
                json!({ "type": "calendarMonthRequested", "year": year, "month": month })
            }
        }
    }
}

pub struct FluxaRuntime {
    handle: u64,
    snapshot: Arc<Value>,
    revision: u64,
    home_hero_plan: Option<Value>,
    home_hero_plan_inputs: Option<(Value, Value, Value)>,
}

impl FluxaRuntime {
    pub fn library_view_plan(&self, tab: &str, query: &str, sort_by: &str) -> Option<Value> {
        let library = self.snapshot.get("library")?;
        let raw = fluxa_core::ffi::core_invoke(
            "libraryViewPlan",
            &serde_json::json!({
                "watchlist": library.get("watchlist"),
                "watching": library.get("continueWatching"),
                "completed": library.get("completed"),
                "dropped": library.get("dropped"),
                "favorites": library.get("liked"),
                "progress": library.get("progress"),
            "tab": tab,
            "query": query,
            "sortBy": sort_by,
            })
            .to_string(),
        );
        let response: Value = serde_json::from_str(&raw).ok()?;
        (response.get("ok").and_then(Value::as_bool) == Some(true))
            .then(|| response.get("value").cloned())
            .flatten()
    }
    pub fn new(initial_state: Value) -> Result<Self, String> {
        let handle = FluxaCore::create_headless_engine(&initial_state.to_string());
        if handle == 0 {
            return Err("failed to create Fluxa core engine".to_owned());
        }
        let snapshot = FluxaCore::headless_engine_snapshot_json(handle)
            .ok_or_else(|| "failed to read initial Fluxa core snapshot".to_owned())?;
        let snapshot = serde_json::from_str(&snapshot).map_err(|error| error.to_string())?;
        Ok(Self {
            handle,
            snapshot: Arc::new(snapshot),
            revision: 0,
            home_hero_plan: None,
            home_hero_plan_inputs: None,
        })
    }

    pub fn snapshot(&self) -> &Value {
        self.snapshot.as_ref()
    }

    /// Shares the immutable Core snapshot with a renderer without cloning
    /// the entire JSON tree on every redraw.
    pub fn snapshot_shared(&self) -> Arc<Value> {
        Arc::clone(&self.snapshot)
    }

    /// Monotonically changes whenever a Core result replaces the snapshot.
    pub fn revision(&self) -> u64 {
        self.revision
    }

    pub fn home_hero_plan(&mut self) -> Option<&Value> {
        let home = self.snapshot.get("home").unwrap_or(&Value::Null);
        let raw_billboard = home.get("billboard").unwrap_or(&Value::Null);
        let raw_categories = home.get("categories").unwrap_or(&Value::Null);
        let raw_prefs = self
            .snapshot
            .pointer("/settings/values")
            .unwrap_or(&Value::Null);
        let inputs_match = self.home_hero_plan.is_some()
            && self
                .home_hero_plan_inputs
                .as_ref()
                .is_some_and(|(billboard, categories, prefs)| {
                    billboard == raw_billboard && categories == raw_categories && prefs == raw_prefs
                });
        if !inputs_match {
            let billboard = raw_billboard.clone();
            let categories = if raw_categories.is_array() {
                raw_categories.clone()
            } else {
                json!([])
            };
            let prefs = if raw_prefs.is_object() {
                raw_prefs.clone()
            } else {
                json!({})
            };
            let raw = fluxa_core::ffi::core_invoke(
                "homeHeroPlan",
                &json!({
                    "categories": categories,
                    "billboard": billboard,
                    "prefs": prefs,
                    "fetchedTrailers": {},
                    "fetchedIds": [],
                    "fetchedLogos": {},
                    "fetchedLogoIds": [],
                })
                .to_string(),
            );
            self.home_hero_plan = serde_json::from_str::<Value>(&raw)
                .ok()
                .filter(|envelope| envelope.get("ok").and_then(Value::as_bool) == Some(true))
                .and_then(|envelope| envelope.get("value").cloned());
            self.home_hero_plan_inputs = Some((
                raw_billboard.clone(),
                raw_categories.clone(),
                raw_prefs.clone(),
            ));
        }
        self.home_hero_plan.as_ref()
    }

    pub fn dispatch(&mut self, action: Value) -> Result<RuntimeUpdate, String> {
        let result = FluxaCore::headless_engine_dispatch_json(self.handle, &action.to_string())
            .ok_or_else(|| "Fluxa core rejected the action".to_owned())?;
        self.apply_result(&result)
    }

    /// Fast path for Discover pagination: the renderer only needs the emitted
    /// fetch effect immediately. The accumulated result snapshot is refreshed
    /// when that effect completes, avoiding a full Core snapshot/JSON roundtrip
    /// in the scroll frame.
    pub fn dispatch_discover_page(&mut self, action: Value) -> Result<RuntimeUpdate, String> {
        let result = FluxaCore::headless_engine_dispatch_effects_json(
            self.handle,
            &action.to_string(),
        )
        .ok_or_else(|| "Fluxa Core rejected the Discover page request".to_owned())?;
        let result: Value = serde_json::from_str(&result).map_err(|error| error.to_string())?;
        self.revision = result
            .get("revision")
            .and_then(Value::as_u64)
            .unwrap_or(self.revision);
        Ok(RuntimeUpdate {
            effects: result
                .get("effects")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default(),
        })
    }

    pub fn dispatch_command(&mut self, command: AppCommand) -> Result<RuntimeUpdate, String> {
        self.dispatch(command.into_json())
    }

    #[allow(dead_code)]
    pub fn complete_effect(
        &mut self,
        effect_id: &str,
        status: &str,
        value: Value,
    ) -> Result<RuntimeUpdate, String> {
        self.complete_effect_result(effect_id, status, value, Value::Null)
    }

    pub fn complete_effect_result(
        &mut self,
        effect_id: &str,
        status: &str,
        value: Value,
        error: Value,
    ) -> Result<RuntimeUpdate, String> {
        let result = FluxaCore::headless_engine_complete_effect_json(
            self.handle,
            &json!({
                "effectId": effect_id,
                "status": status,
                "value": value,
                "error": error,
            })
            .to_string(),
        )
        .ok_or_else(|| "Fluxa core rejected the effect result".to_owned())?;
        self.apply_result(&result)
    }

    /// Apply a page-sized Discover delta instead of rereading and parsing the
    /// full Core snapshot after each pagination response.
    pub fn complete_discover_page_result(
        &mut self,
        effect_id: &str,
        status: &str,
        value: Value,
        error: Value,
    ) -> Result<RuntimeUpdate, String> {
        let result = FluxaCore::headless_engine_complete_discover_page_json(
            self.handle,
            &json!({
                "effectId": effect_id,
                "status": status,
                "value": value,
                "error": error,
            })
            .to_string(),
        )
        .ok_or_else(|| "Fluxa Core rejected the Discover page result".to_owned())?;
        let result: Value = serde_json::from_str(&result).map_err(|error| error.to_string())?;
        self.revision = result
            .get("revision")
            .and_then(Value::as_u64)
            .unwrap_or(self.revision);
        let delta = result.get("delta").unwrap_or(&Value::Null);
        let snapshot = Arc::make_mut(&mut self.snapshot);
        let discover = snapshot
            .get_mut("discover")
            .and_then(Value::as_object_mut)
            .ok_or_else(|| "Discover state is missing from the Core snapshot".to_owned())?;

        if let Some(appended) = delta.get("appended").and_then(Value::as_array) {
            discover
                .entry("results")
                .or_insert_with(|| json!([]))
                .as_array_mut()
                .ok_or_else(|| "Discover results are not an array".to_owned())?
                .extend(appended.iter().cloned());
        }
        if let Some(sources) = delta.get("sources").and_then(Value::as_object) {
            let target = discover
                .entry("resultSources")
                .or_insert_with(|| json!({}))
                .as_object_mut()
                .ok_or_else(|| "Discover result sources are not an object".to_owned())?;
            for (key, source) in sources {
                target.entry(key.clone()).or_insert_with(|| source.clone());
            }
        }
        let paging = discover
            .entry("paging")
            .or_insert_with(|| json!({}))
            .as_object_mut()
            .ok_or_else(|| "Discover paging state is not an object".to_owned())?;
        paging.insert(
            "isLoading".to_owned(),
            delta.get("isLoading").cloned().unwrap_or(Value::Bool(false)),
        );
        paging.insert(
            "nextSkip".to_owned(),
            delta.get("nextSkip").cloned().unwrap_or(Value::from(0)),
        );
        paging.insert(
            "hasMore".to_owned(),
            delta.get("hasMore").cloned().unwrap_or(Value::Bool(false)),
        );
        paging.insert(
            "error".to_owned(),
            delta.get("error").cloned().unwrap_or(Value::Null),
        );

        Ok(RuntimeUpdate {
            effects: result
                .get("effects")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default(),
        })
    }

    fn apply_result(&mut self, result: &str) -> Result<RuntimeUpdate, String> {
        let result: Value = serde_json::from_str(result).map_err(|error| {
            eprintln!("[fluxa-native] Fluxa Core returned invalid dispatch JSON: {error}");
            error.to_string()
        })?;
        if result.get("ok").and_then(Value::as_bool) == Some(false) {
            let error = result.get("error");
            let kind = error
                .and_then(|value| value.get("kind"))
                .and_then(Value::as_str)
                .unwrap_or("error");
            let message = error
                .and_then(|value| value.get("message"))
                .and_then(Value::as_str)
                .unwrap_or("");
            eprintln!(
                "[fluxa-native] Fluxa Core rejected the operation: {kind}{}",
                if message.is_empty() {
                    String::new()
                } else {
                    format!(": {message}")
                }
            );
        }
        if let Some(patch) = result.get("state").and_then(Value::as_object)
            && !patch.is_empty()
            && let Some(snapshot) = Arc::make_mut(&mut self.snapshot).as_object_mut()
        {
            for (key, value) in patch {
                snapshot.insert(key.clone(), value.clone());
            }
        }
        self.revision = self.revision.wrapping_add(1);
        let effects = result
            .get("effects")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        Ok(RuntimeUpdate { effects })
    }
}

impl Drop for FluxaRuntime {
    fn drop(&mut self) {
        FluxaCore::destroy_headless_engine(self.handle);
    }
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
    fn library_view_uses_the_shared_core_plan() {
        let runtime = FluxaRuntime::new(json!({"library": {
            "watchlist": [{"id":"tt1","type":"movie","name":"Saved film"}],
            "continueWatching": [], "completed": [], "dropped": [], "liked": []
        }}))
        .expect("create runtime");
        let plan = runtime
            .library_view_plan("watchlist", "Saved", "title")
            .expect("Core library plan");
        assert_eq!(plan["items"][0]["id"], "tt1");
        assert_eq!(plan["items"][0]["name"], "Saved film");
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
