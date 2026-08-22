use crate::headless_engine;
use crate::runtime::EffectKind;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Stable domain-brain contract for platform shells.
///
/// `CoreBrainSession` wraps the existing headless engine while the app migrates
/// from JSON-only calls to typed UniFFI contracts. Dynamic provider payloads
/// stay as `serde_json::Value`; action/effect/state ownership is explicit.
#[derive(Debug)]
pub struct CoreBrainSession {
    handle: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CoreAction {
    #[serde(rename = "type")]
    pub action_type: String,
    #[serde(default)]
    pub payload: Value,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CoreState {
    #[serde(default)]
    pub value: Value,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CoreEffect {
    pub id: String,
    #[serde(rename = "type")]
    pub effect_type: String,
    #[serde(default)]
    pub generation: u64,
    #[serde(default)]
    pub payload: Value,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CoreEffectResult {
    pub effect_id: String,
    pub status: String,
    #[serde(default)]
    pub value: Value,
    #[serde(default)]
    pub error: Value,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CoreDispatchResult {
    #[serde(default)]
    pub revision: u64,
    pub state: CoreState,
    #[serde(default)]
    pub effects: Vec<CoreEffect>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CoreCapabilitySet {
    pub http: bool,
    pub storage: bool,
    pub auth: bool,
    pub player: bool,
    pub plugins: bool,
    pub torrent: bool,
    pub local_stream: bool,
    pub notifications: bool,
}

impl CoreCapabilitySet {
    pub fn android_default() -> Self {
        Self {
            http: true,
            storage: true,
            auth: true,
            player: true,
            plugins: true,
            torrent: false,
            local_stream: false,
            notifications: true,
        }
    }

    pub fn portable_minimum() -> Self {
        Self {
            http: true,
            storage: true,
            auth: true,
            player: true,
            plugins: false,
            torrent: false,
            local_stream: false,
            notifications: false,
        }
    }
}

impl CoreBrainSession {
    pub fn new(initial_state: Value) -> Self {
        let initial_json =
            serde_json::to_string(&initial_state).unwrap_or_else(|_| "{}".to_string());
        Self {
            handle: headless_engine::create_headless_engine(&initial_json),
        }
    }

    pub fn snapshot(&self) -> Option<CoreState> {
        headless_engine::headless_engine_snapshot_json(self.handle)
            .and_then(|json| serde_json::from_str::<Value>(&json).ok())
            .map(|value| CoreState { value })
    }

    pub fn dispatch_json(&self, action_json: &str) -> Option<CoreDispatchResult> {
        headless_engine::headless_engine_dispatch_json(self.handle, action_json)
            .and_then(|json| parse_dispatch_result(&json))
    }

    pub fn complete_json(&self, result_json: &str) -> Option<CoreDispatchResult> {
        headless_engine::headless_engine_complete_effect_json(self.handle, result_json)
            .and_then(|json| parse_dispatch_result(&json))
    }
}

impl Drop for CoreBrainSession {
    fn drop(&mut self) {
        headless_engine::destroy_headless_engine(self.handle);
    }
}

pub fn core_capabilities_json(portable: bool) -> String {
    let capabilities = if portable {
        CoreCapabilitySet::portable_minimum()
    } else {
        CoreCapabilitySet::android_default()
    };
    serde_json::to_string(&capabilities).unwrap_or_else(|_| "{}".to_string())
}

/// Machine-readable source for generated bindings, docs and contract drift tests.
pub fn core_contract_manifest_json() -> String {
    serde_json::json!({
        "version": env!("CARGO_PKG_VERSION"),
        "methods": ["engine.create", "engine.snapshot", "engine.dispatch", "engine.completeEffect", "engine.destroy", "app.create", "app.state", "app.dispatch", "app.destroy"],
        "effectEnvelope": {"required": ["id", "type", "generation", "payload"], "optional": ["groupId", "priority", "dedupeKey", "cachePolicy", "timeoutMs"]},
        "effectTypes": EffectKind::ALL.iter().map(|kind| kind.as_str()).collect::<Vec<_>>(),
        "capabilities": {"native": CoreCapabilitySet::android_default(), "portable": CoreCapabilitySet::portable_minimum()},
    }).to_string()
}

fn parse_dispatch_result(json: &str) -> Option<CoreDispatchResult> {
    let raw = serde_json::from_str::<Value>(json).ok()?;
    let state = raw.get("state").cloned().unwrap_or(Value::Null);
    let effects = raw
        .get("effects")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(|item| serde_json::from_value::<CoreEffect>(item.clone()).ok())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    Some(CoreDispatchResult {
        revision: raw
            .get("revision")
            .and_then(Value::as_u64)
            .unwrap_or_default(),
        state: CoreState { value: state },
        effects,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn typed_session_wraps_headless_state_and_effects() {
        let session = CoreBrainSession::new(json!({"profile":{"activeProfileId":"p1"}}));
        let result = session
            .dispatch_json(r#"{"type":"searchRequested","query":"matrix","language":"en"}"#)
            .expect("dispatch result");

        assert_eq!(result.effects.len(), 1);
        assert_eq!(result.effects[0].effect_type, "runSearch");
        assert_eq!(result.effects[0].payload["profileId"], "p1");
        assert!(session.snapshot().is_some());
    }

    #[test]
    fn capability_sets_make_native_and_portable_differences_explicit() {
        let native =
            serde_json::from_str::<CoreCapabilitySet>(&core_capabilities_json(false)).unwrap();
        let portable =
            serde_json::from_str::<CoreCapabilitySet>(&core_capabilities_json(true)).unwrap();

        assert!(!native.torrent);
        assert!(!native.local_stream);
        assert!(!portable.torrent);
        assert!(!portable.local_stream);
        assert!(!portable.plugins);
    }
}
