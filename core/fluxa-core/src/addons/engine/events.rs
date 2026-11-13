use crate::addons::sources::protocol::supports_resource;
use crate::headless_engine::HeadlessEngine;
use crate::headless_engine::state::GenerationKey;
use crate::runtime::{EffectEnvelope, EffectKind};
use serde::Serialize;
use serde_json::{Map, Value};

const EVENT_RESOURCES: [&str; 2] = ["player", "library"];
const WATCHED_CHUNK: usize = 100;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct EmitAddonEventPayload {
    requests: Vec<EventRequest>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct EventRequest {
    transport_url: String,
    resource: &'static str,
    content_type: String,
    id: String,
    extra: Value,
}

pub(crate) fn declares_events(addon: &Value) -> bool {
    let manifest = addon.get("manifest").unwrap_or(addon).to_string();
    EVENT_RESOURCES
        .into_iter()
        .any(|resource| supports_resource(&manifest, resource, None, None))
}

fn targets(installed: &Value, resource: &str, content_type: &str, id: &str) -> Vec<String> {
    installed
        .as_array()
        .into_iter()
        .flatten()
        .filter(|addon| addon.get("enabled").and_then(Value::as_bool) != Some(false))
        .filter(|addon| addon.get("trackingConsent").and_then(Value::as_bool) == Some(true))
        .filter(|addon| {
            let manifest = addon.get("manifest").unwrap_or(addon).to_string();
            supports_resource(&manifest, resource, Some(content_type), Some(id))
        })
        .filter_map(|addon| {
            addon
                .get("transportUrl")
                .or_else(|| addon.pointer("/manifest/transportUrl"))
                .and_then(Value::as_str)
                .map(ToOwned::to_owned)
        })
        .collect()
}

fn emit(
    engine: &mut HeadlessEngine,
    resource: &'static str,
    content_type: &str,
    id: &str,
    extras: Vec<Value>,
) -> Vec<EffectEnvelope> {
    if id.is_empty() || extras.is_empty() {
        return vec![];
    }
    let urls = targets(&engine.state.addons.installed, resource, content_type, id);
    if urls.is_empty() {
        return vec![];
    }
    let requests = urls
        .iter()
        .flat_map(|url| {
            extras.iter().map(|extra| EventRequest {
                transport_url: url.clone(),
                resource,
                content_type: content_type.to_owned(),
                id: id.to_owned(),
                extra: extra.clone(),
            })
        })
        .collect();
    let generation = engine.state.runtime.get(GenerationKey::Addon);
    vec![engine.effect(
        EffectKind::EmitAddonEvent,
        generation,
        EmitAddonEventPayload { requests },
    )]
}

pub(crate) fn player_event(
    engine: &mut HeadlessEngine,
    content_type: &str,
    video_id: &str,
    action: &str,
    position_ms: i64,
    duration_ms: i64,
) -> Vec<EffectEnvelope> {
    if !matches!(action, "start" | "pause" | "stop") || duration_ms <= 0 {
        return vec![];
    }
    let mut extra = Map::new();
    extra.insert("action".into(), action.into());
    extra.insert("currentTime".into(), position_ms.max(0).to_string().into());
    extra.insert("duration".into(), duration_ms.to_string().into());
    emit(
        engine,
        "player",
        content_type,
        video_id,
        vec![Value::Object(extra)],
    )
}

pub(crate) fn library_event(
    engine: &mut HeadlessEngine,
    content_type: &str,
    meta_id: &str,
    action: &str,
) -> Vec<EffectEnvelope> {
    let mut extra = Map::new();
    extra.insert("action".into(), action.into());
    emit(
        engine,
        "library",
        content_type,
        meta_id,
        vec![Value::Object(extra)],
    )
}

pub(crate) fn watched_event(
    engine: &mut HeadlessEngine,
    content_type: &str,
    meta_id: &str,
    video_ids: &[String],
    watched: bool,
) -> Vec<EffectEnvelope> {
    let action = if watched { "watched" } else { "unwatched" };
    let whole_item = video_ids.is_empty() || video_ids == [meta_id.to_owned()];
    let extras = if whole_item {
        let mut extra = Map::new();
        extra.insert("action".into(), action.into());
        vec![Value::Object(extra)]
    } else {
        video_ids
            .chunks(WATCHED_CHUNK)
            .map(|chunk| {
                let mut extra = Map::new();
                extra.insert("action".into(), action.into());
                extra.insert("videoId".into(), chunk.join(",").into());
                Value::Object(extra)
            })
            .collect()
    };
    emit(engine, "library", content_type, meta_id, extras)
}

pub(crate) fn dispatch_tracking_consent(
    engine: &mut HeadlessEngine,
    transport_url: &str,
    granted: bool,
) -> Vec<EffectEnvelope> {
    let Some(addon) = engine
        .state
        .addons
        .installed
        .as_array_mut()
        .and_then(|items| {
            items.iter_mut().find(|addon| {
                addon
                    .get("transportUrl")
                    .or_else(|| addon.pointer("/manifest/transportUrl"))
                    .and_then(Value::as_str)
                    == Some(transport_url)
            })
        })
    else {
        return vec![];
    };
    addon["trackingConsent"] = Value::Bool(granted);
    let addons = engine.state.addons.installed.clone();
    crate::headless_engine::home::set_user_addons(engine, addons);
    vec![]
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn tracker(consent: Option<bool>) -> Value {
        let mut addon = json!({
            "transportUrl": "https://tracker.example.com/manifest.json",
            "manifest": {
                "id": "example.tracker",
                "resources": [
                    {"name": "player", "types": ["movie", "series"], "idPrefixes": ["tt"]},
                    {"name": "library", "types": ["movie", "series"], "idPrefixes": ["tt"]}
                ],
                "types": ["movie", "series"]
            }
        });
        if let Some(consent) = consent {
            addon["trackingConsent"] = json!(consent);
        }
        addon
    }

    #[test]
    fn addon_without_consent_gets_nothing() {
        let installed = json!([tracker(None), tracker(Some(false))]);
        assert!(targets(&installed, "player", "movie", "tt1").is_empty());
    }

    #[test]
    fn consent_and_prefix_are_both_required() {
        let installed = json!([tracker(Some(true))]);
        assert_eq!(targets(&installed, "library", "series", "tt1").len(), 1);
        assert!(targets(&installed, "library", "series", "kitsu:1").is_empty());
        assert!(targets(&installed, "library", "tv", "tt1").is_empty());
    }

    #[test]
    fn disabled_addon_gets_nothing() {
        let mut addon = tracker(Some(true));
        addon["enabled"] = json!(false);
        assert!(targets(&json!([addon]), "player", "movie", "tt1").is_empty());
    }

    #[test]
    fn manifests_declaring_events_are_detected() {
        assert!(declares_events(&tracker(None)));
        let plain = json!({"manifest": {"id": "x", "resources": ["catalog"]}});
        assert!(!declares_events(&plain));
    }
}
