use crate::runtime::EffectKind;
use serde::{Deserialize, Serialize};

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
        "methods": ["engine.create", "engine.snapshot", "engine.dispatch", "engine.completeEffect", "engine.destroy", "app.create", "app.state", "app.dispatch", "app.destroy", "deviceAuthStart", "deviceAuthTransition"],
        "effectEnvelope": {"required": ["id", "type", "generation", "payload"], "optional": ["groupId", "priority", "dedupeKey", "cachePolicy", "timeoutMs"]},
        "effectTypes": EffectKind::ALL.iter().map(|kind| kind.as_str()).collect::<Vec<_>>(),
        "capabilities": {"native": CoreCapabilitySet::android_default(), "portable": CoreCapabilitySet::portable_minimum()},
    }).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

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
