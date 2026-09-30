use super::helpers::parse;
use serde_json::{Value, json};

pub(crate) fn addon_snapshot_plan_json(args_json: &str) -> Option<String> {
    let args = parse(args_json)?;
    let mut rows = args.get("rows")?.as_array()?.clone();
    rows.sort_by_key(|row| row.get("sort_order").and_then(Value::as_i64).unwrap_or(0));
    let cached = args.get("snapshot").unwrap_or(&Value::Null);
    let cached_addon = |url: &str| {
        cached
            .get("addons")
            .and_then(Value::as_array)?
            .iter()
            .find(|addon| addon.get("transportUrl").and_then(Value::as_str) == Some(url))
    };
    let manifests = args.get("manifests").unwrap_or(&Value::Null);
    let mut fetch = Vec::new();
    let mut addons = Vec::new();
    for row in &rows {
        let Some(url) = row
            .get("url")
            .and_then(Value::as_str)
            .filter(|url| !url.is_empty())
        else {
            continue;
        };
        let Some(manifest) = manifests
            .get(url)
            .or_else(|| cached_addon(url).and_then(|addon| addon.get("manifest")))
        else {
            fetch.push(url.to_owned());
            continue;
        };
        addons.push(json!({
            "transportUrl": url,
            "manifest": manifest,
            "name": row.get("name").cloned().unwrap_or(Value::Null),
            "enabled": row.get("enabled").and_then(Value::as_bool).unwrap_or(true),
        }));
    }
    serde_json::to_string(&json!({
        "changed": cached.get("rows") != Some(&Value::Array(rows.clone())),
        "rows": rows,
        "fetch": fetch,
        "addons": addons,
    }))
    .ok()
}
