use super::helpers::{parse, str_field};
use serde_json::{Value, json};

const AVATAR_STORAGE_BASE: &str = "https://api.nuvio.tv/storage/v1/object/public/avatars/";
fn avatar_url_for(profile: &Value, avatar_catalog: &[Value]) -> Option<String> {
    if let Some(url) = str_field(profile, "avatar_url").filter(|s| !s.is_empty()) {
        return Some(url.to_string());
    }
    let avatar_id = profile.get("avatar_id").filter(|v| !v.is_null())?;
    let entry = avatar_catalog
        .iter()
        .find(|a| a.get("id") == Some(avatar_id))?;
    let storage_path = str_field(entry, "storage_path").filter(|s| !s.is_empty())?;
    Some(format!("{AVATAR_STORAGE_BASE}{storage_path}"))
}

pub(crate) fn apply_remote_profiles_json(args_json: &str) -> Option<String> {
    let args = parse(args_json)?;
    let session = args.get("sessionProfile")?;
    let remote = args.get("nuvioProfiles").and_then(Value::as_array)?;
    let avatar_catalog = args
        .get("avatarCatalog")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let mut profiles = args.get("profiles").and_then(Value::as_array)?.clone();
    let account = |profile: &Value| {
        ["nuvioUserId", "nuvioEmail", "email"]
            .iter()
            .find_map(|key| {
                str_field(profile, key)
                    .filter(|s| !s.is_empty())
                    .map(str::to_owned)
            })
    };
    let owner = account(session)?;
    let mut changed = false;
    for profile in &mut profiles {
        if account(profile).as_deref() != Some(owner.as_str()) {
            continue;
        }
        let index = profile
            .get("nuvioProfileIndex")
            .and_then(Value::as_i64)
            .unwrap_or(1);
        let Some(source) = remote
            .iter()
            .find(|entry| entry.get("profile_index").and_then(Value::as_i64) == Some(index))
        else {
            continue;
        };
        let Some(fields) = profile.as_object_mut() else {
            continue;
        };
        let mut set = |key: &str, value: Value| {
            if fields.get(key) != Some(&value) {
                fields.insert(key.into(), value);
                changed = true;
            }
        };
        if let Some(name) = str_field(source, "name").filter(|s| !s.trim().is_empty()) {
            set("name", Value::String(name.trim().to_owned()));
        }
        if let Some(url) = avatar_url_for(source, &avatar_catalog) {
            set("avatarUrl", Value::String(url));
        }
        set("nuvioProfileIndex", json!(index));
    }
    changed.then(|| Value::Array(profiles).to_string())
}

pub(crate) fn effective_profile_scopes_json(args_json: &str) -> Option<String> {
    let args = parse(args_json)?;
    let profile_index = args
        .get("profileIndex")
        .and_then(Value::as_i64)
        .filter(|index| *index > 0)
        .unwrap_or(1);
    let profiles = args.get("profiles").and_then(Value::as_array)?;
    let profile = profiles.iter().find(|profile| {
        profile
            .get("profile_index")
            .or_else(|| profile.get("profileIndex"))
            .and_then(Value::as_i64)
            == Some(profile_index)
    });
    let uses_primary = |snake: &str, camel: &str| {
        profile
            .and_then(|value| value.get(snake).or_else(|| value.get(camel)))
            .and_then(Value::as_bool)
            == Some(true)
    };
    Some(
        json!({
            "addons": if profile_index != 1 && uses_primary("uses_primary_addons", "usesPrimaryAddons") { 1 } else { profile_index },
            "plugins": if profile_index != 1 && uses_primary("uses_primary_plugins", "usesPrimaryPlugins") { 1 } else { profile_index },
        })
        .to_string(),
    )
}
