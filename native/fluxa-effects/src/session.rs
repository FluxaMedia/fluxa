use std::sync::{
    Arc,
    mpsc::{self, Receiver, Sender},
};

use fluxa_app::FluxaRuntime;
use serde_json::{Value, json};

use crate::{EffectCompletion, EffectExecutor, Storage};

pub struct AppSession {
    runtime: FluxaRuntime,
    executor: EffectExecutor,
    sender: Sender<EffectCompletion>,
    receiver: Receiver<EffectCompletion>,
}

impl AppSession {
    pub fn open(storage: Storage) -> Result<Self, String> {
        let state = persisted_runtime_state(&storage, json!({}));
        let runtime = FluxaRuntime::new(state)?;
        let executor = EffectExecutor::new(storage);
        executor.warm_torrent_engine();
        let (sender, receiver) = mpsc::channel();
        Ok(Self {
            runtime,
            executor,
            sender,
            receiver,
        })
    }

    pub fn dispatch(&mut self, action: Value) -> Result<(), String> {
        let update = self.runtime.dispatch(action)?;
        self.schedule(update.effects);
        Ok(())
    }

    pub fn pump(&mut self) -> bool {
        let mut follow_ups = Vec::new();
        let mut changed = false;
        while let Ok(completion) = self.receiver.try_recv() {
            changed = true;
            let result = if completion.effect_type == "fetchDiscoverPage" {
                self.runtime.complete_discover_page_result(
                    &completion.effect_id,
                    completion.status,
                    completion.value,
                    completion.error,
                )
            } else {
                self.runtime.complete_effect_result(
                    &completion.effect_id,
                    completion.status,
                    completion.value,
                    completion.error,
                )
            };
            match result {
                Ok(update) => follow_ups.extend(update.effects),
                Err(error) => eprintln!(
                    "[fluxa-session] effect {} completion rejected: {error}",
                    completion.effect_id
                ),
            }
        }
        self.schedule(follow_ups);
        changed
    }

    pub fn snapshot(&self) -> Arc<Value> {
        self.runtime.snapshot_shared()
    }

    pub fn revision(&self) -> u64 {
        self.runtime.revision()
    }

    pub fn active_profile(&self) -> Value {
        self.runtime
            .snapshot()
            .pointer("/profile/active")
            .cloned()
            .unwrap_or(Value::Null)
    }

    fn schedule(&self, effects: Vec<Value>) {
        for effect in effects {
            self.executor.spawn(effect, self.sender.clone());
        }
    }
}

pub fn persisted_runtime_state(storage: &Storage, default_prefs: Value) -> Value {
    let profiles = storage
        .read_json("profiles")
        .ok()
        .flatten()
        .unwrap_or_else(|| json!([]));
    let active_id = storage
        .read_json("active_profile_id")
        .ok()
        .flatten()
        .and_then(|value| value.as_str().map(ToOwned::to_owned))
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "guest".to_owned());
    let active_profile = profiles
        .as_array()
        .and_then(|items| {
            items.iter().find(|profile| {
                profile.get("id").and_then(Value::as_str) == Some(active_id.as_str())
            })
        })
        .cloned()
        .unwrap_or(Value::Null);
    let mut prefs = storage
        .read_json(&Storage::prefs_key(&active_id))
        .ok()
        .flatten()
        .unwrap_or(default_prefs);
    if active_profile
        .get("nuvioAccessToken")
        .and_then(Value::as_str)
        .is_some_and(|token| !token.is_empty())
        && let Some(values) = prefs.as_object_mut()
    {
        for key in ["integrationLibrarySource", "continueWatchingSource"] {
            if values
                .get(key)
                .and_then(Value::as_str)
                .is_none_or(|source| source == "local")
            {
                values.insert(key.to_owned(), Value::String("nuvio".to_owned()));
            }
        }
    }
    let owner = core_value(
        "effectiveAddonsOwnerId",
        json!({"profiles": profiles, "activeProfileId": active_id}),
    )
    .and_then(|value| value.as_str().map(ToOwned::to_owned))
    .unwrap_or_else(|| active_id.clone());
    let addons = storage
        .read_json(&Storage::addons_key(&owner))
        .ok()
        .flatten()
        .or_else(|| {
            storage
                .read_json(&Storage::addons_key(&active_id))
                .ok()
                .flatten()
        })
        .or_else(|| storage.read_json("addons").ok().flatten())
        .filter(Value::is_array)
        .unwrap_or_else(|| json!([]));
    json!({
        "settings": {"values": prefs},
        "profile": {"active": active_profile, "activeProfileId": active_id},
        "addons": {"installed": addons},
    })
}

fn core_value(method: &str, args: Value) -> Option<Value> {
    let response = fluxa_core::ffi::core_invoke(method, &args.to_string());
    let envelope: Value = serde_json::from_str(&response).ok()?;
    (envelope.get("ok").and_then(Value::as_bool) == Some(true))
        .then(|| envelope.get("value").cloned().unwrap_or(Value::Null))
}
