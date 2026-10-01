use serde_json::Value;

fn addon_key(addon: &Value) -> String {
    addon
        .get("transportUrl")
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| {
            addon
                .get("manifest")
                .and_then(|manifest| manifest.get("id"))
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string()
        })
}

pub(crate) fn filter_enabled_addons_json(args_json: &str) -> Option<String> {
    let args: Value = serde_json::from_str(args_json).ok()?;
    let addons = args.get("addons")?.as_array()?.clone();
    let disabled_keys: Vec<&str> = args
        .get("disabledKeys")?
        .as_array()?
        .iter()
        .filter_map(Value::as_str)
        .collect();
    let filtered: Vec<Value> = addons
        .into_iter()
        .filter(|addon| {
            addon.get("enabled").and_then(Value::as_bool) != Some(false)
                && !disabled_keys.contains(&addon_key(addon).as_str())
        })
        .collect();
    serde_json::to_string(&filtered).ok()
}

#[cfg(test)]
mod tests {
    use super::filter_enabled_addons_json;
    use serde_json::{Value, json};

    #[test]
    fn enabled_addon_filter_applies_core_and_host_persisted_disable_flags() {
        let filtered = filter_enabled_addons_json(
            &json!({
                "addons": [
                    {"transportUrl": "https://enabled.test/manifest.json", "enabled": true},
                    {"transportUrl": "https://disabled.test/manifest.json", "enabled": false},
                    {"transportUrl": "https://profile-disabled.test/manifest.json"}
                ],
                "disabledKeys": ["https://profile-disabled.test/manifest.json"]
            })
            .to_string(),
        )
        .and_then(|value| serde_json::from_str::<Value>(&value).ok())
        .expect("filter should return JSON");

        assert_eq!(
            filtered,
            json!([{"transportUrl": "https://enabled.test/manifest.json", "enabled": true}])
        );
    }
}
