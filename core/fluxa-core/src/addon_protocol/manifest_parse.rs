use super::assets::{first_text, resolve_asset_url, string_array};
use super::catalogs::{canonical_resource_name, parse_catalogs};
use serde_json::{Value, json};

// pub rather than pub(crate): re-exported under fuzz_targets for the `fuzz/`
// crate (see lib.rs). Not part of the supported public API otherwise.
pub fn parse_manifest(body: &str, transport_url: &str, unknown_name: &str) -> Option<String> {
    let json: Value = serde_json::from_str(body).ok()?;
    let behavior_hints = json.get("behaviorHints");
    let logo = first_text(
        &json,
        behavior_hints,
        &["logo", "icon", "iconUrl", "poster", "posterUrl"],
    );
    let background = first_text(
        &json,
        behavior_hints,
        &["background", "backgroundUrl", "backdrop", "backdropUrl"],
    );
    let description = first_text(
        &json,
        behavior_hints,
        &[
            "description",
            "shortDescription",
            "longDescription",
            "summary",
        ],
    );
    let resources = json
        .get("resources")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let id_prefixes = string_array(&json, "idPrefixes");
    let mut manifest = json.as_object().cloned().unwrap_or_default();
    manifest.insert(
        "id".to_string(),
        Value::String(
            json.get("id")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string(),
        ),
    );
    manifest.insert(
        "name".to_string(),
        Value::String(
            json.get("name")
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|text| !text.is_empty())
                .unwrap_or(unknown_name)
                .to_string(),
        ),
    );
    manifest.insert(
        "description".to_string(),
        description.map(Value::String).unwrap_or(Value::Null),
    );
    manifest.insert(
        "version".to_string(),
        first_text(&json, behavior_hints, &["version"])
            .map(Value::String)
            .unwrap_or(Value::Null),
    );
    manifest.insert("resources".to_string(), Value::Array(resources));
    manifest.insert(
        "supportsCatalog".to_string(),
        Value::Bool(
            json.get("resources")
                .and_then(Value::as_array)
                .is_some_and(|resources| {
                    resources.iter().any(|resource| match resource {
                        Value::String(name) => canonical_resource_name(name) == "catalog",
                        Value::Object(resource) => resource
                            .get("name")
                            .and_then(Value::as_str)
                            .is_some_and(|name| canonical_resource_name(name) == "catalog"),
                        _ => false,
                    })
                }),
        ),
    );
    manifest.insert(
        "types".to_string(),
        Value::Array(string_array(&json, "types")),
    );
    manifest.insert("catalogs".to_string(), Value::Array(parse_catalogs(&json)));
    manifest.insert(
        "idPrefixes".to_string(),
        if id_prefixes.is_empty() {
            Value::Null
        } else {
            Value::Array(id_prefixes)
        },
    );
    manifest.insert(
        "logo".to_string(),
        resolve_asset_url(logo, transport_url)
            .map(Value::String)
            .unwrap_or(Value::Null),
    );
    manifest.insert(
        "background".to_string(),
        resolve_asset_url(background, transport_url)
            .map(Value::String)
            .unwrap_or(Value::Null),
    );
    manifest.insert(
        "configurable".to_string(),
        behavior_hints
            .and_then(|hints| hints.get("configurable"))
            .and_then(Value::as_bool)
            .map(Value::Bool)
            .unwrap_or(Value::Null),
    );

    let descriptor = json!({
        "manifest": Value::Object(manifest),
        "transportUrl": transport_url
    });
    serde_json::to_string(&descriptor).ok()
}

pub(crate) fn normalize_addon_descriptor_json(addon_json: &str) -> Option<String> {
    let addon: Value = serde_json::from_str(addon_json).ok()?;
    if addon.get("manifest").is_some_and(Value::is_object) {
        let mut normalized = addon.as_object()?.clone();
        normalized
            .entry("transportUrl")
            .or_insert_with(|| Value::String(String::new()));
        return serde_json::to_string(&normalized).ok();
    }
    let transport_url = addon
        .get("transportUrl")
        .or_else(|| addon.get("id"))
        .and_then(Value::as_str)
        .unwrap_or("");
    parse_manifest(addon_json, transport_url, "Unknown Addon")
}
