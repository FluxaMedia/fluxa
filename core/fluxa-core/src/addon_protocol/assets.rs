use super::url::{base_url, is_http_url, normalize_manifest_url, prefer_https_asset_url};
use serde_json::{Map, Value};

pub(crate) fn string_array(json: &Value, key: &str) -> Vec<Value> {
    json.get(key)
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(|item| {
                    item.as_str()
                        .filter(|text| !text.is_empty())
                        .map(|text| Value::String(text.to_string()))
                })
                .collect()
        })
        .unwrap_or_default()
}

pub(crate) fn first_text(
    json: &Value,
    behavior_hints: Option<&Value>,
    keys: &[&str],
) -> Option<String> {
    keys.iter().find_map(|key| {
        json.get(*key)
            .and_then(Value::as_str)
            .filter(|text| !text.is_empty())
            .or_else(|| {
                behavior_hints
                    .and_then(|hints| hints.get(*key))
                    .and_then(Value::as_str)
                    .filter(|text| !text.is_empty())
            })
            .map(str::to_string)
    })
}

pub(crate) fn resolve_asset_url(asset: Option<String>, manifest_url: &str) -> Option<String> {
    let secure = prefer_https_asset_url(asset?.as_str())?;
    if is_http_url(&secure) {
        return Some(secure);
    }
    if secure.starts_with('/') {
        let base = base_url(manifest_url);
        let scheme_end = base.find("://").map(|index| index + 3)?;
        let host_end = base[scheme_end..]
            .find('/')
            .map(|index| scheme_end + index)
            .unwrap_or(base.len());
        return prefer_https_asset_url(&format!("{}{}", &base[..host_end], secure));
    }
    prefer_https_asset_url(&format!("{}{}", base_url(manifest_url), secure))
}

fn text_value<'a>(value: &'a Value, key: &str) -> Option<&'a str> {
    value
        .get(key)
        .and_then(Value::as_str)
        .filter(|text| !text.is_empty())
}

fn non_empty_array(value: Option<&Value>) -> Option<Value> {
    value
        .and_then(Value::as_array)
        .filter(|items| !items.is_empty())
        .cloned()
        .map(Value::Array)
}

fn set_or_null(map: &mut Map<String, Value>, key: &str, value: Option<String>) {
    map.insert(
        key.to_string(),
        value.map(Value::String).unwrap_or(Value::Null),
    );
}

fn value_has_content(value: &Value) -> bool {
    match value {
        Value::Null => false,
        Value::String(text) => !text.trim().is_empty(),
        Value::Array(items) => !items.is_empty(),
        Value::Object(map) => !map.is_empty(),
        Value::Bool(_) | Value::Number(_) => true,
    }
}

pub(crate) fn resolve_manifest_assets_json(descriptor_json: &str) -> Option<String> {
    let mut descriptor: Value = serde_json::from_str(descriptor_json).ok()?;
    let transport_url = descriptor
        .get("transportUrl")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    let normalized_transport_url = normalize_manifest_url(&transport_url);
    descriptor.as_object_mut()?.insert(
        "transportUrl".to_string(),
        Value::String(normalized_transport_url.clone()),
    );

    let manifest = descriptor.get_mut("manifest")?.as_object_mut()?;
    let logo = text_value(&Value::Object(manifest.clone()), "logo").map(str::to_string);
    let background = text_value(&Value::Object(manifest.clone()), "background").map(str::to_string);
    let resolved_background = resolve_asset_url(background.clone(), &normalized_transport_url);
    let resolved_logo =
        resolve_asset_url(logo, &normalized_transport_url).or_else(|| resolved_background.clone());
    let description = manifest
        .get("description")
        .and_then(Value::as_str)
        .filter(|text| !text.is_empty())
        .map(str::to_string);

    set_or_null(manifest, "description", description);
    set_or_null(manifest, "logo", resolved_logo);
    set_or_null(manifest, "background", resolved_background);
    serde_json::to_string(&descriptor).ok()
}

pub(crate) fn merge_live_manifest_json(
    descriptor_json: &str,
    live_json: Option<&str>,
    unknown_name: &str,
) -> Option<String> {
    let Some(live_json) = live_json.filter(|value| !value.trim().is_empty()) else {
        return resolve_manifest_assets_json(descriptor_json);
    };
    let mut descriptor: Value = serde_json::from_str(descriptor_json).ok()?;
    let live: Value = serde_json::from_str(live_json).ok()?;
    let transport_url = descriptor
        .get("transportUrl")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    let normalized_transport_url = normalize_manifest_url(&transport_url);
    descriptor.as_object_mut()?.insert(
        "transportUrl".to_string(),
        Value::String(normalized_transport_url.clone()),
    );

    let current_manifest_snapshot = descriptor.get("manifest")?.clone();
    let current_manifest = current_manifest_snapshot.as_object()?;
    let live_manifest = live.get("manifest")?.as_object()?;
    let manifest = descriptor.get_mut("manifest")?.as_object_mut()?;

    if let Some(id) = text_value(&Value::Object(live_manifest.clone()), "id") {
        manifest.insert("id".to_string(), Value::String(id.to_string()));
    }
    if let Some(name) = text_value(&Value::Object(live_manifest.clone()), "name")
        .filter(|name| *name != unknown_name)
    {
        manifest.insert("name".to_string(), Value::String(name.to_string()));
    }
    if let Some(description) = text_value(&Value::Object(live_manifest.clone()), "description") {
        manifest.insert(
            "description".to_string(),
            Value::String(description.to_string()),
        );
    }
    for (key, value) in live_manifest {
        if matches!(
            key.as_str(),
            "id" | "name" | "description" | "logo" | "background"
        ) {
            continue;
        }
        if matches!(
            key.as_str(),
            "resources" | "types" | "catalogs" | "idPrefixes"
        ) {
            if let Some(value) = non_empty_array(Some(value)) {
                manifest.insert(key.to_string(), value);
            }
            continue;
        }
        if value_has_content(value) {
            manifest.insert(key.to_string(), value.clone());
        }
    }

    let current_logo = current_manifest
        .get("logo")
        .and_then(Value::as_str)
        .map(str::to_string);
    let current_background = current_manifest
        .get("background")
        .and_then(Value::as_str)
        .map(str::to_string);
    let live_logo = live_manifest
        .get("logo")
        .and_then(Value::as_str)
        .filter(|text| !text.is_empty())
        .map(str::to_string);
    let live_background = live_manifest
        .get("background")
        .and_then(Value::as_str)
        .filter(|text| !text.is_empty())
        .map(str::to_string);

    let resolved_current_logo = resolve_asset_url(current_logo, &normalized_transport_url);
    let resolved_current_background =
        resolve_asset_url(current_background, &normalized_transport_url);
    let logo = live_logo
        .or(resolved_current_logo)
        .or_else(|| live_background.clone())
        .or_else(|| resolved_current_background.clone());
    let background = live_background.or(resolved_current_background);
    set_or_null(manifest, "logo", logo);
    set_or_null(manifest, "background", background);

    serde_json::to_string(&descriptor).ok()
}
