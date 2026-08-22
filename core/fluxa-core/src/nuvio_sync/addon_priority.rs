use super::helpers::parse;
use serde_json::{Value, json};

pub(crate) fn sort_addons_by_priority_json(args_json: &str) -> Option<String> {
    let args = parse(args_json)?;
    let mut addons = args.get("addons")?.as_array()?.clone();
    addons.sort_by_key(|a| a.get("sort_order").and_then(Value::as_i64).unwrap_or(0));
    serde_json::to_string(&addons).ok()
}

pub(crate) fn addon_state_json(args_json: &str) -> Option<String> {
    let args = parse(args_json)?;
    let mut addons = args.get("addons")?.as_array()?.clone();
    addons.sort_by_key(|addon| {
        addon
            .get("sortOrder")
            .or_else(|| addon.get("sort_order"))
            .and_then(Value::as_i64)
            .unwrap_or(0)
    });
    let mut installed_urls = Vec::new();
    let mut disabled_urls = Vec::new();
    for addon in addons {
        let Some(url) = addon
            .get("url")
            .and_then(Value::as_str)
            .filter(|url| !url.is_empty())
        else {
            continue;
        };
        if !installed_urls.iter().any(|item| item == url) {
            installed_urls.push(url.to_string());
        }
        if addon.get("enabled").and_then(Value::as_bool) == Some(false)
            && !disabled_urls.iter().any(|item| item == url)
        {
            disabled_urls.push(url.to_string());
        }
    }
    serde_json::to_string(&json!({
        "installedUrls": installed_urls,
        "disabledUrls": disabled_urls,
    }))
    .ok()
}
