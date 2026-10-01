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

fn set_or_null(map: &mut Map<String, Value>, key: &str, value: Option<String>) {
    map.insert(
        key.to_string(),
        value.map(Value::String).unwrap_or(Value::Null),
    );
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
