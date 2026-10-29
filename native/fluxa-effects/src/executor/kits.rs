use super::*;
use fluxa_core::kits::{METADATAKIT_HOST, STREAMKIT_HOST};
use std::sync::OnceLock;

static STORAGE: OnceLock<Storage> = OnceLock::new();

pub(super) fn init(storage: &Storage) {
    let _ = STORAGE.set(storage.clone());
}

pub(super) fn is_kit_host(host: Option<&str>) -> bool {
    matches!(host, Some(STREAMKIT_HOST | METADATAKIT_HOST))
}

pub(super) async fn respond(url: &reqwest::Url) -> (i32, Option<String>) {
    match handle(url).await {
        Some(body) => (200, Some(body.to_string())),
        None => (404, Some("{}".to_owned())),
    }
}

async fn handle(url: &reqwest::Url) -> Option<Value> {
    let storage = STORAGE.get()?.clone();
    let mut segments = url.path_segments()?;
    let profile_id = segments.next()?.to_owned();
    let resource = segments.next()?;
    let executor = EffectExecutor::bound_to(storage, &profile_id);
    let prefs = executor.profile_prefs(&profile_id).ok().flatten()?;
    let host = url.host_str()?;
    let addon = fluxa_core::kits::addons(&prefs, &profile_id)
        .into_iter()
        .find(|addon| {
            addon["transportUrl"]
                .as_str()
                .is_some_and(|transport| transport.contains(host))
        })?;
    if resource.strip_suffix(".json") == Some("manifest") {
        return Some(addon["manifest"].clone());
    }
    let kind = segments.next()?.to_owned();
    let id = segments.next()?.trim_end_matches(".json").to_owned();
    match (host, resource) {
        (STREAMKIT_HOST, "stream") => Some(json!({"streams": []})),
        (METADATAKIT_HOST, "meta") => {
            let meta = executor.aggregated_meta(&prefs, &kind, &id).await?;
            Some(json!({"meta": meta}))
        }
        _ => None,
    }
}
