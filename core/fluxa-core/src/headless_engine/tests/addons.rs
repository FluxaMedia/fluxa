use super::super::*;
use crate::runtime::EffectKind;
use serde_json::json;

#[test]
fn install_survives_a_refresh_started_after_it() {
    let mut engine = HeadlessEngine::new(Default::default());
    let install = engine
        .apply(
            serde_json::from_value(json!({
                "type": "addonInstallRequested",
                "transportUrl": "https://addon.example.com/manifest.json",
            }))
            .unwrap(),
        )
        .effects;
    let fetch = install
        .iter()
        .find(|effect| effect.kind == EffectKind::FetchAddonManifest)
        .unwrap();
    engine.apply(serde_json::from_value(json!({"type": "addonsRefreshRequested"})).unwrap());

    engine.complete_effect(
        serde_json::from_value(json!({
            "effectId": fetch.id,
            "status": "ok",
            "value": {
                "transportUrl": "https://addon.example.com/manifest.json",
                "manifest": {"id": "example.addon", "resources": ["catalog"], "types": ["movie"]},
            },
        }))
        .unwrap(),
    );

    assert_eq!(engine.state.addons.installed.as_array().unwrap().len(), 1);
}
