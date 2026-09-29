use crate::addons::sources::protocol::{identity, normalize_manifest_url};
use serde_json::{Value, json};

fn text(value: Option<&Value>) -> String {
    value
        .and_then(Value::as_str)
        .unwrap_or_default()
        .trim()
        .to_string()
}

fn fallback_name(url: &str) -> String {
    let host = url
        .split("//")
        .nth(1)
        .unwrap_or(url)
        .split('/')
        .next()
        .unwrap_or("")
        .split(':')
        .next()
        .unwrap_or("")
        .trim_start_matches("www.")
        .split('.')
        .next()
        .unwrap_or("");
    if host.is_empty() {
        "Stremio Addon".to_string()
    } else {
        host.replace('-', " ")
            .split_whitespace()
            .map(|part| {
                let mut chars = part.chars();
                chars
                    .next()
                    .map(|first| first.to_uppercase().collect::<String>() + chars.as_str())
                    .unwrap_or_default()
            })
            .collect::<Vec<_>>()
            .join(" ")
    }
}

pub(crate) fn addon_store_entries_plan_json(args_json: &str) -> Option<String> {
    let args: Value = serde_json::from_str(args_json).ok()?;
    let repository = args.get("repositoryAddons")?.as_array()?;
    let local_urls = args.get("localUrls")?.as_array()?;
    let disabled = args.get("disabledKeys")?.as_array()?;
    let is_nuvio = args
        .get("isNuvioProfile")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let local_loaded = args
        .get("localLoaded")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let refreshing = args
        .get("refreshingUrl")
        .and_then(Value::as_str)
        .map(identity);

    let local_order: Vec<String> = local_urls
        .iter()
        .filter_map(Value::as_str)
        .map(normalize_manifest_url)
        .collect();
    let local_keys: Vec<String> = local_order.iter().map(|url| identity(url)).collect();
    let disabled_keys: Vec<String> = disabled
        .iter()
        .filter_map(Value::as_str)
        .map(identity)
        .collect();

    let mut entries = Vec::new();
    let mut repository_keys = Vec::new();
    for addon in repository {
        let raw_url = text(addon.get("transportUrl"));
        if raw_url.is_empty() {
            continue;
        }
        let url = normalize_manifest_url(&raw_url);
        let key = identity(&url);
        if repository_keys.iter().any(|known| known == &key) {
            continue;
        }
        repository_keys.push(key.clone());
        let manifest = addon.get("manifest").unwrap_or(&Value::Null);
        let name = text(manifest.get("name")).if_empty_then(|| fallback_name(&url));
        let local_index = local_keys.iter().position(|local_key| local_key == &key);
        let can_remove = addon
            .get("isManaged")
            .and_then(Value::as_bool)
            .unwrap_or(false)
            || (!is_nuvio && local_index.is_some());
        entries.push(json!({
            "url": url,
            "identity": key,
            "name": name,
            "description": text(manifest.get("description")),
            "logoUrl": manifest.get("logo").and_then(Value::as_str),
            "version": manifest.get("version").and_then(Value::as_str),
            "configurable": manifest.get("configurable").and_then(Value::as_bool).unwrap_or(false),
            "isEnabled": if is_nuvio { addon.get("isEnabled").and_then(Value::as_bool).unwrap_or(true) } else { !disabled_keys.contains(&key) },
            "canRemove": can_remove,
            "canMoveUp": can_remove && local_index.is_some_and(|index| index > 0),
            "canMoveDown": can_remove && local_index.is_some_and(|index| index < local_order.len().saturating_sub(1)),
            "isRefreshing": refreshing.as_deref() == Some(key.as_str()),
            "order": local_index.unwrap_or(usize::MAX),
        }));
    }

    if local_loaded && !is_nuvio {
        for (index, url) in local_order.iter().enumerate() {
            let key = local_keys[index].clone();
            if repository_keys.contains(&key) {
                continue;
            }
            let can_remove = true;
            entries.push(json!({
                "url": url,
                "identity": key,
                "name": fallback_name(url),
                "description": "",
                "logoUrl": Value::Null,
                "version": Value::Null,
                "configurable": false,
                "isEnabled": !disabled_keys.contains(&key),
                "canRemove": can_remove,
                "canMoveUp": index > 0,
                "canMoveDown": index < local_order.len().saturating_sub(1),
                "isRefreshing": refreshing.as_deref() == Some(key.as_str()),
                "order": index,
            }));
        }
    }

    entries.sort_by(|left, right| {
        left["order"]
            .as_u64()
            .unwrap_or(u64::MAX)
            .cmp(&right["order"].as_u64().unwrap_or(u64::MAX))
            .then_with(|| {
                left["name"]
                    .as_str()
                    .unwrap_or("")
                    .to_ascii_lowercase()
                    .cmp(&right["name"].as_str().unwrap_or("").to_ascii_lowercase())
            })
    });
    serde_json::to_string(&entries).ok()
}

trait EmptyTextFallback {
    fn if_empty_then(self, fallback: impl FnOnce() -> String) -> String;
}

impl EmptyTextFallback for String {
    fn if_empty_then(self, fallback: impl FnOnce() -> String) -> String {
        if self.is_empty() { fallback() } else { self }
    }
}
