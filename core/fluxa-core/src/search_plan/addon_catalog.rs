use crate::{addon_protocol, content_identity};
use serde_json::{Value, json};

fn manifest_value(addon: &Value) -> Option<&Value> {
    addon.get("manifest").or(Some(addon))
}

fn addon_transport_url(addon: &Value) -> &str {
    addon
        .get("transportUrl")
        .and_then(Value::as_str)
        .unwrap_or("")
}

fn addon_manifest_name(addon: &Value) -> String {
    let manifest = manifest_value(addon).unwrap_or(addon);
    manifest
        .get("name")
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .or_else(|| manifest.get("id").and_then(Value::as_str))
        .unwrap_or("Metadata")
        .to_string()
}

fn title_label(value: &str) -> String {
    let label = value
        .split(['_', '-', ' '])
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => format!("{}{}", first.to_uppercase(), chars.as_str()),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ");
    if label.is_empty() {
        value.to_string()
    } else {
        label
    }
}

fn metadata_feed_home_title(label: &str) -> String {
    let parts = label
        .split(" - ")
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>();
    match parts.len() {
        0 => label.to_string(),
        1 => parts.first().unwrap_or(&label).to_string(),
        2 => parts.get(1).unwrap_or(&label).to_string(),
        _ => parts.get(1..).unwrap_or_default().join(" "),
    }
}

fn discover_catalog_label(raw_name: Option<&str>, id: &str) -> String {
    let fallback = title_label(id);
    let base = raw_name
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(&fallback);
    let mut label = base
        .split(['-', ':', '|', '/'])
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join(" ");
    for word in [
        "cinemeta", "movie", "movies", "film", "films", "series", "shows", "tv",
    ] {
        label = label
            .split_whitespace()
            .filter(|part| !part.eq_ignore_ascii_case(word))
            .collect::<Vec<_>>()
            .join(" ");
    }
    if label.trim().is_empty() {
        fallback
    } else {
        label
    }
}

fn catalog_extras(catalog: &Value) -> Vec<Value> {
    catalog
        .get("extra")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|extra| {
            let name = extra.get("name").and_then(Value::as_str)?.trim();
            if name.is_empty() {
                return None;
            }
            let options = extra
                .get("options")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .filter_map(|option| option.as_str().map(str::trim).map(str::to_string))
                .filter(|option| !option.is_empty())
                .collect::<Vec<_>>();
            let default_value = extra.get("default").and_then(Value::as_str);
            (!options.is_empty()).then(|| {
                json!({
                    "name": name,
                    "options": options,
                    "default": default_value,
                    "isRequired": catalog_requires_extra(catalog, name)
                })
            })
        })
        .collect()
}

fn manifest_supports_catalog(manifest: &Value) -> bool {
    serde_json::to_string(manifest)
        .ok()
        .is_some_and(|json| addon_protocol::supports_resource(&json, "catalog", None, None))
}

fn catalog_has_required_extra_except(catalog: &Value, allowed: &[&str]) -> bool {
    let allowed_json =
        serde_json::to_string(&allowed.iter().map(|s| s.to_string()).collect::<Vec<_>>())
            .unwrap_or_else(|_| "[]".to_string());
    serde_json::to_string(catalog)
        .ok()
        .is_some_and(|json| addon_protocol::catalog_has_required_extra_except(&json, &allowed_json))
}

fn catalog_requires_extra(catalog: &Value, extra_name: &str) -> bool {
    serde_json::to_string(catalog)
        .ok()
        .is_some_and(|json| addon_protocol::catalog_requires_extra(&json, extra_name))
}

pub(crate) fn build_metadata_feed_options_json(addons_json: &str) -> Option<String> {
    let addons = serde_json::from_str::<Vec<Value>>(addons_json).ok()?;
    let mut feeds = Vec::new();
    for addon in addons {
        let Some(manifest) = manifest_value(&addon) else {
            continue;
        };
        if !manifest_supports_catalog(manifest) {
            continue;
        }
        let addon_name = addon_manifest_name(&addon);
        let transport_url = addon_transport_url(&addon);
        let source_key = manifest
            .get("id")
            .and_then(Value::as_str)
            .filter(|value| !value.is_empty())
            .unwrap_or(transport_url);
        for catalog in manifest
            .get("catalogs")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            let Some(type_value) = catalog
                .get("type")
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
            else {
                continue;
            };
            let Some(id) = catalog
                .get("id")
                .and_then(Value::as_str)
                .filter(|value| !value.trim().is_empty())
            else {
                continue;
            };
            if catalog_has_required_extra_except(catalog, &[]) {
                continue;
            }
            let name = catalog
                .get("name")
                .and_then(Value::as_str)
                .filter(|value| !value.trim().is_empty())
                .map(str::to_string)
                .unwrap_or_else(|| title_label(id));
            let key = format!(
                "addon:{}:{}:{}",
                content_identity::stable_feed_part(source_key),
                content_identity::stable_feed_part(type_value),
                content_identity::stable_feed_part(id)
            );
            let label = format!("{addon_name} - {name}");
            feeds.push(json!({
                "key": key,
                "label": label,
                "homeTitle": metadata_feed_home_title(&label),
                "transportUrl": transport_url,
                "type": type_value,
                "id": id,
                "genre": Value::Null
            }));
        }
    }
    serde_json::to_string(&feeds).ok()
}

pub(crate) fn discover_catalog_options_json(
    addons_json: &str,
    selected_type: &str,
) -> Option<String> {
    let addons = serde_json::from_str::<Vec<Value>>(addons_json).ok()?;
    let lower_selected_type = selected_type.to_lowercase();
    let normalized_type = content_identity::normalize_content_type(&lower_selected_type)
        .map(str::to_string)
        .unwrap_or(lower_selected_type);
    let mut options = Vec::new();
    for addon in addons {
        let Some(manifest) = manifest_value(&addon) else {
            continue;
        };
        if !manifest_supports_catalog(manifest) {
            continue;
        }
        let transport_url = addon_transport_url(&addon);
        for catalog in manifest
            .get("catalogs")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            let Some(type_value) = catalog
                .get("type")
                .and_then(Value::as_str)
                .and_then(content_identity::normalize_content_type)
            else {
                continue;
            };
            if normalized_type != "all" && normalized_type != type_value {
                continue;
            }
            let Some(id) = catalog
                .get("id")
                .and_then(Value::as_str)
                .filter(|value| !value.trim().is_empty())
            else {
                continue;
            };
            let extras = catalog_extras(catalog);
            let supported_extra_names = extras
                .iter()
                .filter_map(|extra| extra.get("name").and_then(Value::as_str))
                .collect::<Vec<_>>();
            if catalog_has_required_extra_except(catalog, &supported_extra_names) {
                continue;
            }
            let catalog_label =
                discover_catalog_label(catalog.get("name").and_then(Value::as_str), id);
            let genre_extra = extras
                .iter()
                .find(|extra| extra.get("name").and_then(Value::as_str) == Some("genre"));
            let genres: Vec<&str> = genre_extra
                .and_then(|extra| extra.get("options"))
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .filter_map(Value::as_str)
                .collect();
            let requires_genre = genre_extra
                .and_then(|extra| extra.get("isRequired"))
                .and_then(Value::as_bool)
                .unwrap_or(false);
            let default_genre = catalog
                .get("extra")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .find(|extra| extra.get("name").and_then(Value::as_str) == Some("genre"))
                .and_then(|extra| extra.get("default"))
                .and_then(Value::as_str);
            options.push(json!({
                "key": format!(
                    "discover:{}:{}:{}",
                    content_identity::stable_feed_part(transport_url),
                    content_identity::stable_feed_part(type_value),
                    content_identity::stable_feed_part(id)
                ),
                "label": catalog_label,
                "transportUrl": transport_url,
                "type": type_value,
                "id": id,
                "genres": genres,
                "requiresGenre": requires_genre,
                "defaultGenre": default_genre,
                "extras": extras
            }));
        }
    }
    serde_json::to_string(&options).ok()
}

pub(crate) fn discover_content_types_json(addons_json: &str) -> Option<String> {
    let options: Vec<Value> =
        serde_json::from_str(&discover_catalog_options_json(addons_json, "all")?).ok()?;
    let mut types = vec!["movie".to_string(), "series".to_string()];
    for option in &options {
        let extras = option
            .get("extras")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        let search_required = extras.iter().any(|extra| {
            extra.get("name").and_then(Value::as_str) == Some("search")
                && extra.get("isRequired").and_then(Value::as_bool) == Some(true)
        });
        let has_browsable_extra = extras.iter().any(|extra| {
            !matches!(
                extra.get("name").and_then(Value::as_str),
                Some("search" | "skip")
            ) && extra
                .get("options")
                .and_then(Value::as_array)
                .is_some_and(|options| !options.is_empty())
        });
        if search_required && !has_browsable_extra {
            continue;
        }
        let Some(content_type) = option.get("type").and_then(Value::as_str) else {
            continue;
        };
        if !types.iter().any(|value| value == content_type) {
            types.push(content_type.to_string());
        }
    }
    serde_json::to_string(&types).ok()
}

/// Given a catalog source `{catalogId, type, addonId?}` and an array of addon
/// descriptors, returns the first matching `transportUrl`, or `null`.
pub(crate) fn resolve_transport_url_json(source_json: &str, addons_json: &str) -> Option<String> {
    let source: Value = serde_json::from_str(source_json).ok()?;
    let addons: Vec<Value> = serde_json::from_str(addons_json).ok()?;

    let src_addon_id = source
        .get("addonId")
        .and_then(Value::as_str)
        .map(normalized_addon_id);
    let src_catalog_id = source.get("catalogId").and_then(Value::as_str)?;
    let normalize_type = |v: &str| -> String {
        match v.trim().to_lowercase().as_str() {
            "movies" => "movie".to_string(),
            "series" | "tv" | "show" | "shows" => "series".to_string(),
            other => other.to_string(),
        }
    };
    let src_type = source
        .get("type")
        .and_then(Value::as_str)
        .map(normalize_type);

    let mut fallback_transport_url: Option<&str> = None;
    for addon in &addons {
        let Some(manifest) = addon.get("manifest") else {
            continue;
        };
        let addon_id = manifest
            .get("id")
            .and_then(Value::as_str)
            .map(normalized_addon_id)
            .unwrap_or_default();
        let t_url = addon
            .get("transportUrl")
            .and_then(Value::as_str)
            .unwrap_or("");
        if let Some(ref wanted_addon_id) = src_addon_id
            && !(addon_id == *wanted_addon_id
                || addon_id.ends_with(wanted_addon_id.as_str())
                || wanted_addon_id.ends_with(addon_id.as_str())
                || t_url.to_lowercase().contains(wanted_addon_id.as_str()))
        {
            continue;
        }
        if src_addon_id.is_some() {
            fallback_transport_url = Some(t_url);
        }
        let catalogs = manifest
            .get("catalogs")
            .and_then(Value::as_array)
            .map(Vec::as_slice)
            .unwrap_or(&[]);
        let matches = catalogs.iter().any(|cat| {
            cat.get("id").and_then(Value::as_str) == Some(src_catalog_id)
                && src_type.as_deref().is_none_or(|st| {
                    cat.get("type").and_then(Value::as_str).map(&normalize_type)
                        == Some(st.to_string())
                })
        });
        if matches {
            return serde_json::to_string(t_url).ok();
        }
    }
    fallback_transport_url.and_then(|url| serde_json::to_string(url).ok())
}

fn normalized_addon_id(value: &str) -> String {
    value
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

/// Resolves the effective genre for a metadata feed option by inspecting the
/// corresponding catalog's `extra` array for a `genre` field with a default or
/// first required value.
pub(crate) fn resolve_feed_option_genre_json(
    feed_option_json: &str,
    addons_json: &str,
) -> Option<String> {
    let option: Value = serde_json::from_str(feed_option_json).ok()?;
    let addons: Vec<Value> = serde_json::from_str(addons_json).ok()?;

    // If genre is already set on the option, return it.
    if let Some(genre) = option
        .get("genre")
        .and_then(Value::as_str)
        .filter(|s| !s.trim().is_empty())
    {
        return serde_json::to_string(genre).ok();
    }

    let transport_url = option.get("transportUrl").and_then(Value::as_str)?;
    let opt_type = option.get("type").and_then(Value::as_str)?;
    let opt_id = option.get("id").and_then(Value::as_str)?;

    let addon = addons
        .iter()
        .find(|a| a.get("transportUrl").and_then(Value::as_str) == Some(transport_url))?;
    let catalogs = addon
        .get("manifest")
        .and_then(|m| m.get("catalogs"))
        .and_then(Value::as_array)?;
    let catalog = catalogs.iter().find(|cat| {
        cat.get("type").and_then(Value::as_str) == Some(opt_type)
            && cat.get("id").and_then(Value::as_str) == Some(opt_id)
    })?;

    let extras = catalog.get("extra").and_then(Value::as_array)?;
    let genre_extra = extras
        .iter()
        .find(|e| e.get("name").and_then(Value::as_str) == Some("genre"))?;

    let default_genre = genre_extra
        .get("default")
        .and_then(Value::as_str)
        .filter(|s| !s.trim().is_empty());
    let is_required = genre_extra
        .get("isRequired")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let first_option = genre_extra
        .get("options")
        .and_then(Value::as_array)
        .and_then(|opts| opts.first())
        .and_then(Value::as_str);

    let resolved = default_genre.or(if is_required { first_option } else { None })?;
    serde_json::to_string(resolved).ok()
}
