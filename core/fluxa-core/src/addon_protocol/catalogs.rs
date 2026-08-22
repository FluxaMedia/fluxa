use super::assets::string_array;
use serde_json::{Map, Value, json};
use std::collections::HashSet;

pub(crate) fn parse_catalogs(json: &Value) -> Vec<Value> {
    json.get("catalogs")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(|item| {
                    let object = item.as_object()?;
                    let mut map = object.clone();
                    let extra = object
                        .get("extra")
                        .and_then(Value::as_array)
                        .map(|extras| {
                            extras
                                .iter()
                                .filter_map(|extra| {
                                    let extra_object = extra.as_object()?;
                                    let mut map = Map::new();
                                    if let Some(name) = extra_object
                                        .get("name")
                                        .and_then(Value::as_str)
                                        .filter(|text| !text.is_empty())
                                    {
                                        map.insert(
                                            "name".to_string(),
                                            Value::String(name.to_string()),
                                        );
                                    }
                                    let options = string_array(extra, "options");
                                    if !options.is_empty() {
                                        map.insert("options".to_string(), Value::Array(options));
                                    }
                                    if let Some(is_required) =
                                        extra_object.get("isRequired").and_then(Value::as_bool)
                                    {
                                        map.insert(
                                            "isRequired".to_string(),
                                            Value::Bool(is_required),
                                        );
                                    }
                                    if let Some(options_limit) =
                                        extra_object.get("optionsLimit").and_then(Value::as_i64)
                                    {
                                        map.insert(
                                            "optionsLimit".to_string(),
                                            json!(options_limit as i32),
                                        );
                                    }
                                    if let Some(default_value) =
                                        extra_object.get("default").and_then(Value::as_str)
                                    {
                                        map.insert(
                                            "default".to_string(),
                                            Value::String(default_value.to_string()),
                                        );
                                    }
                                    Some(Value::Object(map))
                                })
                                .collect::<Vec<_>>()
                        })
                        .unwrap_or_default();
                    if !extra.is_empty() {
                        map.insert("extra".to_string(), Value::Array(extra));
                    }
                    let extras = map
                        .get("extra")
                        .and_then(Value::as_array)
                        .cloned()
                        .unwrap_or_default();
                    let extra_supported =
                        string_array(&Value::Object(map.clone()), "extraSupported");
                    let supports_initial_load = !extras.iter().any(|extra| {
                        extra
                            .get("isRequired")
                            .and_then(Value::as_bool)
                            .unwrap_or(false)
                    });
                    let supports_search = extra_supported.iter().any(|extra| {
                        extra
                            .as_str()
                            .is_some_and(|name| name.eq_ignore_ascii_case("search"))
                    }) || extras.iter().any(|extra| {
                        extra
                            .get("name")
                            .and_then(Value::as_str)
                            .is_some_and(|name| name.eq_ignore_ascii_case("search"))
                    });
                    let has_required_extra_except_genre = extras.iter().any(|extra| {
                        extra
                            .get("isRequired")
                            .and_then(Value::as_bool)
                            .unwrap_or(false)
                            && !extra
                                .get("name")
                                .and_then(Value::as_str)
                                .is_some_and(|name| name.eq_ignore_ascii_case("genre"))
                    });
                    map.insert(
                        "supportsInitialLoad".to_string(),
                        Value::Bool(supports_initial_load),
                    );
                    map.insert("supportsSearch".to_string(), Value::Bool(supports_search));
                    map.insert(
                        "hasRequiredExtraExceptGenre".to_string(),
                        Value::Bool(has_required_extra_except_genre),
                    );
                    Some(Value::Object(map))
                })
                .collect()
        })
        .unwrap_or_default()
}

pub(crate) fn canonical_resource_name(value: &str) -> String {
    match value.to_ascii_lowercase().trim_end_matches('s') {
        "metadata" => "meta".to_string(),
        "subtitle" => "subtitle".to_string(),
        other => other.to_string(),
    }
}

pub(crate) fn to_string_vec(value: Option<&Value>) -> Vec<String> {
    match value {
        Some(Value::Array(items)) => items
            .iter()
            .filter_map(|item| item.as_str().map(str::to_string))
            .filter(|item| !item.is_empty())
            .collect(),
        Some(Value::String(text)) if !text.is_empty() => vec![text.to_string()],
        _ => Vec::new(),
    }
}

pub(crate) fn supports_resource(
    manifest_json: &str,
    resource_name: &str,
    content_type: Option<&str>,
    id: Option<&str>,
) -> bool {
    let Ok(manifest) = serde_json::from_str::<Value>(manifest_json) else {
        return false;
    };
    let expected = canonical_resource_name(resource_name);
    let manifest_types = to_string_vec(manifest.get("types"));
    let manifest_prefixes = to_string_vec(manifest.get("idPrefixes"));
    manifest
        .get("resources")
        .and_then(Value::as_array)
        .map(|resources| {
            resources.iter().any(|resource| {
                let (name, types, prefixes) = match resource {
                    Value::String(name) => (
                        name.as_str(),
                        manifest_types.clone(),
                        manifest_prefixes.clone(),
                    ),
                    Value::Object(map) => {
                        let Some(name) = map.get("name").and_then(Value::as_str) else {
                            return false;
                        };
                        let types = to_string_vec(map.get("types"))
                            .into_iter()
                            .chain(to_string_vec(map.get("type")))
                            .collect::<Vec<_>>();
                        let prefixes = to_string_vec(map.get("idPrefixes"))
                            .into_iter()
                            .chain(to_string_vec(map.get("idPrefix")))
                            .collect::<Vec<_>>();
                        (
                            name,
                            if types.is_empty() {
                                manifest_types.clone()
                            } else {
                                types
                            },
                            if prefixes.is_empty() {
                                manifest_prefixes.clone()
                            } else {
                                prefixes
                            },
                        )
                    }
                    _ => return false,
                };
                if canonical_resource_name(name) != expected {
                    return false;
                }
                if let Some(content_type) = content_type
                    && !types.is_empty()
                    && !types
                        .iter()
                        .any(|item| item.eq_ignore_ascii_case(content_type))
                {
                    return false;
                }
                if let Some(id) = id
                    && canonical_resource_name(name) != "catalog"
                    && !prefixes.is_empty()
                    && !prefixes.iter().any(|prefix| id.starts_with(prefix))
                {
                    return false;
                }
                true
            })
        })
        .unwrap_or(false)
}

pub(crate) fn catalog_supports_extra(catalog_json: &str, extra_name: &str) -> bool {
    let Ok(catalog) = serde_json::from_str::<Value>(catalog_json) else {
        return false;
    };
    catalog
        .get("extraSupported")
        .and_then(Value::as_array)
        .is_some_and(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .any(|item| item.eq_ignore_ascii_case(extra_name))
        })
        || catalog
            .get("extra")
            .and_then(Value::as_array)
            .is_some_and(|items| {
                items.iter().any(|item| {
                    item.get("name")
                        .and_then(Value::as_str)
                        .is_some_and(|name| name.eq_ignore_ascii_case(extra_name))
                })
            })
}

pub(crate) fn catalog_requires_extra(catalog_json: &str, extra_name: &str) -> bool {
    let Ok(catalog) = serde_json::from_str::<Value>(catalog_json) else {
        return false;
    };
    catalog
        .get("extra")
        .and_then(Value::as_array)
        .is_some_and(|items| {
            items.iter().any(|item| {
                item.get("name")
                    .and_then(Value::as_str)
                    .is_some_and(|name| name.eq_ignore_ascii_case(extra_name))
                    && item.get("isRequired").and_then(Value::as_bool) == Some(true)
            })
        })
}

pub(crate) fn catalog_has_required_extra_except(
    catalog_json: &str,
    allowed_names_json: &str,
) -> bool {
    let Ok(catalog) = serde_json::from_str::<Value>(catalog_json) else {
        return false;
    };
    let allowed_names = serde_json::from_str::<Vec<String>>(allowed_names_json)
        .unwrap_or_default()
        .into_iter()
        .collect::<HashSet<_>>();
    catalog
        .get("extra")
        .and_then(Value::as_array)
        .is_some_and(|items| {
            items.iter().any(|item| {
                item.get("isRequired").and_then(Value::as_bool) == Some(true)
                    && item
                        .get("name")
                        .and_then(Value::as_str)
                        .map(|name| !allowed_names.contains(&name.to_ascii_lowercase()))
                        .unwrap_or(true)
            })
        })
}
