use super::helpers::{parse, str_field};
use serde_json::{Map, Value, json};

const AVATAR_STORAGE_BASE: &str = "https://api.nuvio.tv/storage/v1/object/public/avatars/";
fn safe_id_part(value: &str) -> String {
    let cleaned: String = value
        .trim()
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '_' || c == '-' {
                c
            } else {
                '_'
            }
        })
        .collect();
    if cleaned.is_empty() {
        "user".to_string()
    } else {
        cleaned
    }
}

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

pub(crate) fn build_local_profiles_json(args_json: &str) -> Option<String> {
    let args = parse(args_json)?;
    let session = args.get("sessionProfile")?;
    let remote_profiles = args
        .get("nuvioProfiles")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let avatar_catalog = args
        .get("avatarCatalog")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let existing_profiles = args
        .get("existingProfiles")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();

    let session_user_id = session.get("nuvioUserId").cloned().unwrap_or(Value::Null);
    let remote_profiles = if remote_profiles.is_empty() {
        vec![json!({
            "profile_index": 1,
            "name": str_field(session, "name").filter(|s| !s.is_empty()).unwrap_or("Primary"),
            "avatar_color_hex": Value::Null,
            "avatar_id": Value::Null,
            "avatar_url": Value::Null,
        })]
    } else {
        remote_profiles
    };

    let mut by_nuvio_index: Map<String, Value> = Map::new();
    for p in &existing_profiles {
        let matches_user =
            !session_user_id.is_null() && p.get("nuvioUserId") == Some(&session_user_id);
        if let (true, Some(index)) = (
            matches_user,
            p.get("nuvioProfileIndex").and_then(Value::as_i64),
        ) {
            by_nuvio_index.insert(index.to_string(), p.clone());
        }
    }

    let fallback_id_part = str_field(session, "nuvioUserId")
        .or_else(|| str_field(session, "nuvioEmail"))
        .or_else(|| str_field(session, "email"))
        .unwrap_or("user");

    let mut imported_ids: Vec<Value> = Vec::new();
    let mut imported: Vec<Value> = Vec::new();
    for remote in &remote_profiles {
        let index = remote
            .get("profile_index")
            .and_then(Value::as_i64)
            .unwrap_or(1);
        let existing = by_nuvio_index.get(&index.to_string());
        let mut out = existing
            .and_then(|e| e.as_object().cloned())
            .unwrap_or_default();
        let id = existing
            .and_then(|e| str_field(e, "id"))
            .map(str::to_string)
            .unwrap_or_else(|| format!("nuvio_{}_{index}", safe_id_part(fallback_id_part)));
        imported_ids.push(Value::String(id.clone()));
        out.insert("id".into(), Value::String(id));

        let name = str_field(remote, "name")
            .filter(|s| !s.is_empty())
            .map(str::to_string)
            .or_else(|| {
                existing
                    .and_then(|e| str_field(e, "name"))
                    .map(str::to_string)
            })
            .unwrap_or_else(|| format!("Profile {index}"));
        out.insert("name".into(), Value::String(name));

        if let Some(url) = avatar_url_for(remote, &avatar_catalog) {
            out.insert("avatarUrl".into(), Value::String(url));
        }
        if let Some(color) = remote.get("avatar_color_hex").filter(|v| !v.is_null()) {
            out.insert("color".into(), color.clone());
        }
        for (dst, src) in [
            ("authKey", "authKey"),
            ("email", "email"),
            ("nuvioAccessToken", "nuvioAccessToken"),
            ("nuvioRefreshToken", "nuvioRefreshToken"),
            ("nuvioTokenExpiresAt", "nuvioTokenExpiresAt"),
            ("nuvioUserId", "nuvioUserId"),
            ("nuvioEmail", "nuvioEmail"),
        ] {
            match session.get(src) {
                Some(v) if !v.is_null() => {
                    out.insert(dst.into(), v.clone());
                }
                _ => {
                    out.remove(dst);
                }
            }
        }
        out.insert("nuvioProfileIndex".into(), json!(index));
        if let Some(v) = remote.get("uses_primary_addons") {
            out.insert("usesPrimaryAddons".into(), v.clone());
        }
        if let Some(v) = remote.get("uses_primary_plugins") {
            out.insert("usesPrimaryPlugins".into(), v.clone());
        }
        if let Some(v) = remote.get("pin_enabled") {
            out.insert("nuvioPinEnabled".into(), v.clone());
        }
        if let Some(v) = remote.get("pin_locked_until") {
            out.insert("nuvioPinLockedUntil".into(), v.clone());
        }
        if let Some(v) = remote.get("updated_at") {
            out.insert("nuvioProfileUpdatedAt".into(), v.clone());
        }
        imported.push(Value::Object(out));
    }

    let mut result: Vec<Value> = existing_profiles
        .into_iter()
        .filter(|p| {
            p.get("id")
                .map(|id| !imported_ids.contains(id))
                .unwrap_or(true)
        })
        .collect();
    result.extend(imported);
    Some(Value::Array(result).to_string())
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
