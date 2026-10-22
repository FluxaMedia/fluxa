use serde_json::{Map, Value, json};
use std::collections::HashSet;
use std::sync::OnceLock;

const MANIFEST: &str = include_str!("../../../../settings/settings-manifest.json");
const SYNCED_CATEGORIES: [&str; 5] = ["appearance", "general", "playback", "subtitles", "account"];
const CREDENTIALS: [(&str, &str, &str); 3] = [
    ("mdblist", "api_key", "mdblistApiKey"),
    ("introdb", "api_key", "introDbApiKey"),
    ("animeskip", "client_id", "animeSkipClientId"),
];

fn is_secret(key: &str) -> bool {
    key.ends_with("ApiKey") || key.ends_with("ClientId") || key == "mdblistLibraryListId"
}

fn synced_keys() -> &'static HashSet<String> {
    static KEYS: OnceLock<HashSet<String>> = OnceLock::new();
    KEYS.get_or_init(|| {
        let manifest: Value = serde_json::from_str(MANIFEST).unwrap_or(Value::Null);
        manifest
            .get("settings")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter(|setting| {
                setting
                    .get("category")
                    .and_then(Value::as_str)
                    .is_some_and(|category| SYNCED_CATEGORIES.contains(&category))
            })
            .filter_map(|setting| setting.get("key").and_then(Value::as_str))
            .filter(|key| !is_secret(key))
            .map(ToOwned::to_owned)
            .collect()
    })
}

fn parse(args_json: &str) -> Option<Value> {
    serde_json::from_str(args_json).ok()
}

fn text(value: Option<&Value>) -> &str {
    value.and_then(Value::as_str).unwrap_or_default().trim()
}

pub(crate) fn settings_blob_json(args_json: &str) -> Option<String> {
    let args = parse(args_json)?;
    let values = args.get("values")?.as_object()?;
    let prefs: Map<String, Value> = values
        .iter()
        .filter(|(key, _)| synced_keys().contains(key.as_str()))
        .map(|(key, value)| (key.clone(), value.clone()))
        .collect();
    serde_json::to_string(&json!({"version": 1, "prefs": prefs})).ok()
}

pub(crate) fn settings_changes_json(args_json: &str) -> Option<String> {
    let args = parse(args_json)?;
    let remote = args.get("prefs")?.as_object()?;
    let local = args.get("values").and_then(Value::as_object);
    let changes: Vec<Value> = remote
        .iter()
        .filter(|(key, _)| synced_keys().contains(key.as_str()))
        .filter(|(key, value)| local.and_then(|local| local.get(key.as_str())) != Some(*value))
        .map(|(key, value)| json!({"key": key, "value": value}))
        .collect();
    serde_json::to_string(&changes).ok()
}

fn remote_value<'a>(rows: &'a [Value], provider: &str, field: &str) -> &'a str {
    rows.iter()
        .find(|row| row.get("provider").and_then(Value::as_str) == Some(provider))
        .and_then(|row| row.pointer(&format!("/credential_json/{field}")))
        .and_then(Value::as_str)
        .unwrap_or_default()
        .trim()
}

pub(crate) fn credential_plan_json(args_json: &str) -> Option<String> {
    let args = parse(args_json)?;
    let values = args.get("values").unwrap_or(&Value::Null);
    let remote = args
        .get("remote")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let local_wins = args.get("localWins").and_then(Value::as_bool) == Some(true);
    let mut rows = remote.clone();
    let mut changes = Vec::new();
    let mut push = false;
    for (provider, field, key) in CREDENTIALS {
        let local = text(values.get(key));
        let remote_text = remote_value(&remote, provider, field);
        if !local_wins && !remote_text.is_empty() && remote_text != local {
            changes.push(json!({"key": key, "value": remote_text}));
        } else if remote_text != local && (local_wins || !local.is_empty()) {
            push = true;
            let row = json!({"provider": provider, "credential_json": {field: local}});
            match rows
                .iter_mut()
                .find(|row| row.get("provider").and_then(Value::as_str) == Some(provider))
            {
                Some(existing) => *existing = row,
                None => rows.push(row),
            }
        }
    }
    let rows: Vec<Value> = rows
        .into_iter()
        .map(|row| {
            json!({
                "provider": row.get("provider"),
                "credential_json": row.get("credential_json"),
            })
        })
        .collect();
    serde_json::to_string(&json!({"changes": changes, "push": push, "rows": rows})).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn plan(args: Value) -> Value {
        serde_json::from_str(&credential_plan_json(&args.to_string()).unwrap()).unwrap()
    }

    #[test]
    fn remote_key_fills_an_empty_local_one() {
        let result = plan(json!({
            "values": {"mdblistApiKey": ""},
            "remote": [{"provider": "mdblist", "credential_json": {"api_key": "abc"}}],
        }));
        assert_eq!(
            result["changes"],
            json!([{"key": "mdblistApiKey", "value": "abc"}])
        );
        assert_eq!(result["push"], false);
    }

    #[test]
    fn local_key_is_pushed_without_dropping_other_providers() {
        let result = plan(json!({
            "values": {"introDbApiKey": "mine"},
            "remote": [{"provider": "debrid:torbox", "credential_json": {"api_key": "tb"}}],
        }));
        assert_eq!(result["push"], true);
        assert_eq!(result["rows"].as_array().map(Vec::len), Some(2));
        assert_eq!(result["rows"][0]["provider"], "debrid:torbox");
    }

    #[test]
    fn local_wins_pushes_a_cleared_key() {
        let result = plan(json!({
            "localWins": true,
            "values": {"mdblistApiKey": ""},
            "remote": [{"provider": "mdblist", "credential_json": {"api_key": "old"}}],
        }));
        assert_eq!(result["changes"], json!([]));
        assert_eq!(result["rows"][0]["credential_json"]["api_key"], "");
    }

    #[test]
    fn blob_leaves_out_secrets_and_device_settings() {
        let blob: Value = serde_json::from_str(
            &settings_blob_json(
                &json!({"values": {"accentColorArgb": "#fff", "mdblistApiKey": "k", "downloadDir": "/x"}})
                    .to_string(),
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(blob["prefs"], json!({"accentColorArgb": "#fff"}));
    }
}
