use serde_json::{Value, json};

pub(crate) fn profile_local_addons_key_json(profile_json: &str) -> Option<String> {
    let profile: Value = serde_json::from_str(profile_json).ok()?;
    let id = string_field(&profile, "id").unwrap_or_default();
    let email = string_field(&profile, "email").unwrap_or_default();
    Some(format!(
        "local_addons_{}",
        if id.trim().is_empty() { email } else { id }
    ))
}

pub(super) fn effective_shared_owner_id(
    profiles_json: &str,
    active_profile_id: &str,
    flag_field: &str,
) -> Option<String> {
    let profiles: Vec<Value> = serde_json::from_str(profiles_json).ok()?;
    let active = profiles
        .iter()
        .find(|p| string_field(p, "id").as_deref() == Some(active_profile_id));
    let uses_primary = active
        .and_then(|p| p.get(flag_field))
        .and_then(Value::as_bool)
        .unwrap_or(false);
    if !uses_primary {
        return Some(active_profile_id.to_string());
    }
    profiles
        .first()
        .and_then(|p| string_field(p, "id"))
        .or_else(|| Some(active_profile_id.to_string()))
}

pub(crate) fn effective_addons_owner_id_json(args_json: &str) -> Option<String> {
    let args: Value = serde_json::from_str(args_json).ok()?;
    let profiles_json = args.get("profiles")?.to_string();
    let active_profile_id = args.get("activeProfileId").and_then(Value::as_str)?;
    effective_shared_owner_id(&profiles_json, active_profile_id, "usesPrimaryAddons")
}

pub(crate) fn effective_plugins_owner_id_json(args_json: &str) -> Option<String> {
    let args: Value = serde_json::from_str(args_json).ok()?;
    let profiles_json = args.get("profiles")?.to_string();
    let active_profile_id = args.get("activeProfileId").and_then(Value::as_str)?;
    effective_shared_owner_id(&profiles_json, active_profile_id, "usesPrimaryPlugins")
}

pub(crate) fn plugin_storage_fallback_json(args_json: &str) -> Option<String> {
    let args: Value = serde_json::from_str(args_json).ok()?;
    let repository_urls = args
        .get("scopedRepositoryUrls")
        .filter(|v| !v.is_null())
        .or_else(|| args.get("legacyRepositoryUrls"))
        .cloned()
        .unwrap_or_else(|| json!([]));
    let scraper_overrides = args
        .get("scopedScraperOverrides")
        .filter(|v| !v.is_null())
        .or_else(|| args.get("legacyScraperOverrides"))
        .cloned()
        .unwrap_or_else(|| json!({}));
    serde_json::to_string(&json!({
        "repositoryUrls": repository_urls,
        "scraperOverrides": scraper_overrides,
    }))
    .ok()
}

fn string_field(value: &Value, key: &str) -> Option<String> {
    value.get(key).and_then(Value::as_str).map(str::to_string)
}
