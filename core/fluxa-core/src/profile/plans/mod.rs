use crate::runtime::constants::GUEST_PROFILE_ID;
use serde::Deserialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

mod settings_migration;
mod tokens;

pub(crate) use settings_migration::*;
pub(crate) use tokens::*;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ActiveProfileRequest {
    #[serde(default)]
    profiles: Vec<Value>,
    #[serde(default)]
    stored_active_id: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TokenMergeRequest {
    profile: Value,
    auth_result: Value,
    provider: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AuthProvider {
    Trakt,
    Simkl,
    Anilist,
    Mdblist,
    Stremio,
    Unknown,
}

impl From<&str> for AuthProvider {
    fn from(value: &str) -> Self {
        match value {
            "trakt" => AuthProvider::Trakt,
            "simkl" => AuthProvider::Simkl,
            "anilist" => AuthProvider::Anilist,
            "mdblist" => AuthProvider::Mdblist,
            "stremio" | "account" => AuthProvider::Stremio,
            _ => AuthProvider::Unknown,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SettingsMigrationRequest {
    #[serde(default)]
    raw: Value,
    #[serde(default)]
    schema_version: Option<i32>,
}

pub(crate) fn active_profile_plan_json(request_json: &str) -> Option<String> {
    let request = serde_json::from_str::<ActiveProfileRequest>(request_json).ok()?;
    if request.profiles.is_empty() {
        return serde_json::to_string(&json!({
            "activeId": GUEST_PROFILE_ID,
            "shouldCreateDefault": true,
            "activeProfile": Value::Null
        }))
        .ok();
    }
    let stored = request.stored_active_id.as_deref().unwrap_or("").trim();
    let active = if stored.is_empty() || stored == GUEST_PROFILE_ID {
        request.profiles.first().cloned().unwrap_or(Value::Null)
    } else {
        request
            .profiles
            .iter()
            .find(|p| p.get("id").and_then(Value::as_str) == Some(stored))
            .cloned()
            .or_else(|| request.profiles.first().cloned())
            .unwrap_or(Value::Null)
    };
    let active_id = active
        .get("id")
        .and_then(Value::as_str)
        .unwrap_or(GUEST_PROFILE_ID)
        .to_string();
    serde_json::to_string(&json!({
        "activeId": active_id,
        "shouldCreateDefault": false,
        "activeProfile": active
    }))
    .ok()
}

pub(crate) fn primary_profile_id_json(profiles_json: &str) -> Option<String> {
    let profiles: Vec<Value> = serde_json::from_str(profiles_json).ok()?;
    profiles
        .first()?
        .get("id")
        .and_then(Value::as_str)
        .map(str::to_string)
}

#[expect(
    clippy::indexing_slicing,
    reason = "index comes from a preceding position lookup in the same profile vector"
)]
pub(crate) fn profile_mutation_plan_json(request_json: &str) -> Option<String> {
    let request: Value = serde_json::from_str(request_json).ok()?;
    let operation = request.get("operation")?.as_str()?;
    let mut profiles = request
        .get("profiles")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    match operation {
        "save" => {
            let profile = request.get("profile")?.clone();
            let id = profile.get("id")?.as_str()?;
            if let Some(index) = profiles
                .iter()
                .position(|item| item.get("id").and_then(Value::as_str) == Some(id))
            {
                profiles[index] = profile;
            } else {
                profiles.push(profile);
            }
        }
        "delete" => {
            let id = request.get("id")?.as_str()?;
            profiles.retain(|item| item.get("id").and_then(Value::as_str) != Some(id));
        }
        _ => return None,
    }
    serde_json::to_string(&profiles).ok()
}

pub(crate) fn create_profile_plan_json(request_json: &str) -> Option<String> {
    let request: Value = serde_json::from_str(request_json).ok()?;
    Some(json!({"id": request.get("id")?.as_str()?, "name": request.get("name").and_then(Value::as_str).unwrap_or("").trim(), "color": request.get("color").and_then(Value::as_str).unwrap_or("#E85D3F")}).to_string())
}

pub(crate) fn profile_pin_hash(pin: &str) -> String {
    let digest = Sha256::digest(pin.as_bytes());
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

pub(crate) fn profile_pin_matches(profile_json: &str, pin: &str) -> bool {
    let profile: Value = serde_json::from_str(profile_json).unwrap_or(Value::Null);
    profile
        .get("pinHash")
        .and_then(Value::as_str)
        .is_none_or(|hash| hash == profile_pin_hash(pin))
}

pub(crate) fn account_source_json(request_json: &str) -> Option<String> {
    let request: Value = serde_json::from_str(request_json).ok()?;
    let profile = request.get("profile").unwrap_or(&Value::Null);
    let entity = request
        .get("entity")
        .and_then(Value::as_str)
        .unwrap_or("addons");
    let text = |key: &str| {
        profile
            .get(key)
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
    };
    let source = match text("nuvioAccessToken") {
        Some(_) => {
            let account = text("nuvioUserId").or_else(|| text("nuvioEmail"))?;
            let index = profile
                .get("nuvioProfileIndex")
                .and_then(Value::as_i64)
                .filter(|index| *index > 0)
                .unwrap_or(1);
            let account: String = account
                .chars()
                .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
                .collect();
            json!({
                "backend": "nuvio",
                "profileIndex": index,
                "snapshotKey": format!("remote_{entity}_nuvio_{account}_{index}"),
            })
        }
        None => json!({"backend": "local"}),
    };
    Some(source.to_string())
}

#[cfg(test)]
mod tests;
