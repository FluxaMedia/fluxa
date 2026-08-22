use crate::constants::{DEFAULT_LANGUAGE, GUEST_PROFILE_ID};
use serde::Deserialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

const DEFAULT_ADDON_URL: &str = "https://v3-cinemeta.strem.io/manifest.json";

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
    Stremio,
    Unknown,
}

impl From<&str> for AuthProvider {
    fn from(value: &str) -> Self {
        match value {
            "trakt" => AuthProvider::Trakt,
            "simkl" => AuthProvider::Simkl,
            "anilist" => AuthProvider::Anilist,
            "stremio" | "account" => AuthProvider::Stremio,
            _ => AuthProvider::Unknown,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DefaultProfileRequest {
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    email: Option<String>,
    #[serde(default)]
    auth_key: Option<String>,
    #[serde(default)]
    language: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SettingsMigrationRequest {
    #[serde(default)]
    raw: Value,
    #[serde(default)]
    schema_version: Option<i32>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AvatarDefaultRequest {
    #[serde(default)]
    profile: Value,
    #[serde(default)]
    catalog: Vec<Value>,
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

pub(crate) fn token_merge_plan_json(request_json: &str) -> Option<String> {
    let request = serde_json::from_str::<TokenMergeRequest>(request_json).ok()?;
    let mut profile = request.profile.clone();
    let auth = &request.auth_result;
    let provider = request.provider.as_str();

    if profile.is_null() || !profile.is_object() {
        profile = json!({});
    }

    let obj = profile.as_object_mut()?;

    match AuthProvider::from(provider) {
        AuthProvider::Trakt => {
            let token = auth.get("accessToken").or_else(|| auth.get("access_token"));
            let refresh = auth
                .get("refreshToken")
                .or_else(|| auth.get("refresh_token"));
            let expires_at = auth
                .get("expiresAt")
                .or_else(|| auth.get("expires_at"))
                .or_else(|| auth.get("traktTokenExpiresAt"));
            if let Some(t) = token {
                obj.insert("traktAccessToken".to_string(), t.clone());
            }
            if let Some(r) = refresh {
                obj.insert("traktRefreshToken".to_string(), r.clone());
            }
            if let Some(e) = expires_at {
                obj.insert("traktTokenExpiresAt".to_string(), e.clone());
            }
            obj.insert("traktLastSyncAt".to_string(), Value::Null);
        }
        AuthProvider::Simkl => {
            let token = auth.get("accessToken").or_else(|| auth.get("access_token"));
            if let Some(t) = token {
                obj.insert("simklAccessToken".to_string(), t.clone());
            }
        }
        AuthProvider::Anilist => {
            let token = auth.get("accessToken").or_else(|| auth.get("access_token"));
            let refresh = auth
                .get("refreshToken")
                .or_else(|| auth.get("refresh_token"));
            let expires_at = auth
                .get("expiresAt")
                .or_else(|| auth.get("expires_at"))
                .or_else(|| auth.get("anilistTokenExpiresAt"));
            if let Some(t) = token {
                obj.insert("anilistAccessToken".to_string(), t.clone());
            }
            if let Some(r) = refresh {
                obj.insert("anilistRefreshToken".to_string(), r.clone());
            }
            if let Some(e) = expires_at {
                obj.insert("anilistTokenExpiresAt".to_string(), e.clone());
            }
        }
        AuthProvider::Stremio => {
            if let Some(auth_key) = auth.get("authKey").or_else(|| auth.get("apiKey")) {
                obj.insert("authKey".to_string(), auth_key.clone());
            }
            if let Some(id) = auth.get("id") {
                obj.insert("id".to_string(), id.clone());
            }
            if let Some(email) = auth.get("email") {
                obj.insert("email".to_string(), email.clone());
            }
            obj.insert("isGuest".to_string(), json!(false));
        }
        AuthProvider::Unknown => {}
    }

    serde_json::to_string(&json!({
        "mergedProfile": profile,
        "provider": provider
    }))
    .ok()
}

pub(crate) fn profile_default_seed_json(request_json: &str) -> Option<String> {
    let request = serde_json::from_str::<DefaultProfileRequest>(request_json).ok()?;
    let id = request
        .id
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| GUEST_PROFILE_ID.to_string());
    let email = request.email.unwrap_or_default();
    let auth_key = request.auth_key.unwrap_or_default();
    let language = request
        .language
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| DEFAULT_LANGUAGE.to_string());
    let is_guest = id == GUEST_PROFILE_ID || auth_key.is_empty();
    serde_json::to_string(&json!({
        "id": id,
        "email": email,
        "authKey": auth_key,
        "isGuest": is_guest,
        "language": language,
        "localAddons": [DEFAULT_ADDON_URL],
        "disabledLocalAddons": []
    }))
    .ok()
}

pub(crate) fn profile_settings_migration_plan_json(request_json: &str) -> Option<String> {
    let request = serde_json::from_str::<SettingsMigrationRequest>(request_json).ok()?;
    let mut profile = request.raw.clone();
    let schema_version = request.schema_version.unwrap_or(0);

    if profile.is_null() || !profile.is_object() {
        return serde_json::to_string(&json!({
            "migratedProfile": Value::Null,
            "appliedMigrations": []
        }))
        .ok();
    }

    let obj = profile.as_object_mut()?;
    let mut applied = Vec::<String>::new();

    // Migration: flatten nested externalAccounts into top-level fields (v1 -> v2)
    if schema_version < 2
        && let Some(ext) = obj.remove("externalAccounts")
        && let Some(ext_obj) = ext.as_object()
    {
        for (k, v) in ext_obj {
            obj.entry(k.clone()).or_insert(v.clone());
        }
        applied.push("flatten_external_accounts".to_string());
    }

    // Migration: flatten nested addonSettings into top-level localAddons/disabledLocalAddons
    if schema_version < 2
        && let Some(addon_settings) = obj.remove("addonSettings")
        && let Some(addon_obj) = addon_settings.as_object()
    {
        if let Some(local) = addon_obj.get("localAddons") {
            obj.entry("localAddons".to_string())
                .or_insert(local.clone());
        }
        if let Some(disabled) = addon_obj.get("disabledLocalAddons") {
            obj.entry("disabledLocalAddons".to_string())
                .or_insert(disabled.clone());
        }
        applied.push("flatten_addon_settings".to_string());
    }

    // Migration: flatten nested subtitleSettings
    if schema_version < 2
        && let Some(sub_settings) = obj.remove("subtitleSettings")
        && let Some(sub_obj) = sub_settings.as_object()
    {
        let field_map = [
            ("size", "subtitleSize"),
            ("color", "subtitleColor"),
            ("backgroundColor", "subtitleBackgroundColor"),
            ("outlineColor", "subtitleOutlineColor"),
            ("textOpacity", "subtitleTextOpacity"),
            ("backgroundOpacity", "subtitleBackgroundOpacity"),
            ("outlineOpacity", "subtitleOutlineOpacity"),
            ("preferredLanguage", "preferredSubtitleLanguage"),
            ("secondaryLanguage", "secondarySubtitleLanguage"),
            ("shadow", "subtitleShadow"),
            ("autoEnable", "autoEnableSubtitles"),
        ];
        for (src, dst) in field_map {
            if let Some(v) = sub_obj.get(src) {
                obj.entry(dst.to_string()).or_insert(v.clone());
            }
        }
        applied.push("flatten_subtitle_settings".to_string());
    }

    // Migration: flatten nested playbackSettings
    if schema_version < 2
        && let Some(pb_settings) = obj.remove("playbackSettings")
        && let Some(pb_obj) = pb_settings.as_object()
    {
        for (k, v) in pb_obj {
            obj.entry(k.clone()).or_insert(v.clone());
        }
        applied.push("flatten_playback_settings".to_string());
    }

    // Migration: flatten nested torrentSettings
    if schema_version < 2
        && let Some(torr_settings) = obj.remove("torrentSettings")
        && let Some(torr_obj) = torr_settings.as_object()
    {
        let field_map = [
            ("wifiOnly", "torrentWifiOnly"),
            ("maxConnections", "torrentMaxConnections"),
            ("speedPreset", "torrentSpeedPreset"),
            ("cachePreset", "torrentCachePreset"),
        ];
        for (src, dst) in field_map {
            if let Some(v) = torr_obj.get(src) {
                obj.entry(dst.to_string()).or_insert(v.clone());
            }
        }
        applied.push("flatten_torrent_settings".to_string());
    }

    // Migration: flatten nested appearanceSettings
    if schema_version < 2
        && let Some(app_settings) = obj.remove("appearanceSettings")
        && let Some(app_obj) = app_settings.as_object()
    {
        for (k, v) in app_obj {
            obj.entry(k.clone()).or_insert(v.clone());
        }
        applied.push("flatten_appearance_settings".to_string());
    }

    // Migration: flatten nested homeFeedSettings
    if schema_version < 2
        && let Some(feed_settings) = obj.remove("homeFeedSettings")
        && let Some(feed_obj) = feed_settings.as_object()
    {
        for (k, v) in feed_obj {
            if k == "libraryCollections"
                && v.as_array().is_some_and(|items| !items.is_empty())
                && obj
                    .get(k)
                    .and_then(Value::as_array)
                    .is_none_or(|items| items.is_empty())
            {
                obj.insert(k.clone(), v.clone());
                continue;
            }
            obj.entry(k.clone()).or_insert(v.clone());
        }
        applied.push("flatten_home_feed_settings".to_string());
    }

    // Ensure localAddons always has at least the default addon
    {
        let has_local_addons = obj
            .get("localAddons")
            .and_then(Value::as_array)
            .is_some_and(|arr| !arr.is_empty());
        if !has_local_addons {
            obj.insert("localAddons".to_string(), json!([DEFAULT_ADDON_URL]));
            applied.push("ensure_default_addon".to_string());
        }
    }

    serde_json::to_string(&json!({
        "migratedProfile": profile,
        "appliedMigrations": applied,
        "schemaVersion": 2
    }))
    .ok()
}

pub(crate) fn profile_avatar_default_json(request_json: &str) -> Option<String> {
    let request = serde_json::from_str::<AvatarDefaultRequest>(request_json).ok()?;
    let existing = request
        .profile
        .get("avatarUrl")
        .and_then(Value::as_str)
        .filter(|s| !s.trim().is_empty());
    if let Some(url) = existing {
        return serde_json::to_string(&json!({
            "avatarUrl": url,
            "fromCatalog": false
        }))
        .ok();
    }
    let first_catalog = request
        .catalog
        .first()
        .and_then(|e| e.get("url").and_then(Value::as_str))
        .map(ToString::to_string);
    serde_json::to_string(&json!({
        "avatarUrl": first_catalog,
        "fromCatalog": first_catalog.is_some()
    }))
    .ok()
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

pub(crate) fn profile_connection_state_json(profile_json: &str, now_epoch_seconds: i64) -> String {
    let profile: Value = serde_json::from_str(profile_json).unwrap_or(Value::Null);
    let trakt = profile
        .get("traktAccessToken")
        .and_then(Value::as_str)
        .is_some_and(|token| !token.is_empty())
        && profile
            .get("traktTokenExpiresAt")
            .and_then(Value::as_i64)
            .is_none_or(|expires| expires >= now_epoch_seconds);
    let simkl = profile
        .get("simklAccessToken")
        .and_then(Value::as_str)
        .is_some_and(|token| !token.is_empty());
    json!({"trakt": trakt, "simkl": simkl}).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    #[test]
    fn active_profile_plan_returns_first_when_no_stored_id() {
        let result: Value = serde_json::from_str(
            &active_profile_plan_json(r#"{"profiles":[{"id":"p1"},{"id":"p2"}]}"#).unwrap(),
        )
        .unwrap();
        assert_eq!(result["activeId"], "p1");
        assert_eq!(result["shouldCreateDefault"], false);
    }

    #[test]
    fn primary_profile_id_returns_first_profile() {
        assert_eq!(
            primary_profile_id_json(r#"[{"id":"p1"},{"id":"p2"}]"#),
            Some("p1".to_string())
        );
        assert_eq!(primary_profile_id_json(r#"[]"#), None);
    }

    #[test]
    fn active_profile_plan_creates_default_when_profiles_empty() {
        let result: Value =
            serde_json::from_str(&active_profile_plan_json(r#"{"profiles":[]}"#).unwrap()).unwrap();
        assert_eq!(result["activeId"], "guest");
        assert_eq!(result["shouldCreateDefault"], true);
    }

    #[test]
    fn active_profile_plan_selects_stored_id() {
        let result: Value = serde_json::from_str(
            &active_profile_plan_json(
                r#"{"profiles":[{"id":"p1"},{"id":"p2"}],"storedActiveId":"p2"}"#,
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(result["activeId"], "p2");
    }

    #[test]
    fn token_merge_plan_merges_trakt_tokens_into_profile() {
        let result: Value = serde_json::from_str(
            &token_merge_plan_json(
                r#"{"profile":{"id":"p1","email":"u@example.com"},"authResult":{"accessToken":"tok","refreshToken":"ref","expiresAt":999},"provider":"trakt"}"#,
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(result["mergedProfile"]["traktAccessToken"], "tok");
        assert_eq!(result["mergedProfile"]["traktRefreshToken"], "ref");
        assert_eq!(result["mergedProfile"]["traktTokenExpiresAt"], 999);
    }

    #[test]
    fn settings_migration_flattens_nested_external_accounts() {
        let result: Value = serde_json::from_str(
            &profile_settings_migration_plan_json(
                r#"{"raw":{"id":"p1","externalAccounts":{"traktAccessToken":"tok"}},"schemaVersion":1}"#,
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(result["migratedProfile"]["traktAccessToken"], "tok");
        assert!(
            result["appliedMigrations"]
                .as_array()
                .unwrap()
                .iter()
                .any(|m| m == "flatten_external_accounts")
        );
    }

    #[test]
    fn settings_migration_keeps_nested_library_collections_when_top_level_is_empty() {
        let result: Value = serde_json::from_str(
            &profile_settings_migration_plan_json(
                r#"{"raw":{"id":"p1","libraryCollections":[],"homeFeedSettings":{"libraryCollections":[{"id":"c1","title":"Collection"}]}},"schemaVersion":1}"#,
            )
            .unwrap(),
        )
        .unwrap();

        assert_eq!(
            result["migratedProfile"]["libraryCollections"][0]["id"],
            "c1"
        );
    }

    #[test]
    fn default_seed_produces_guest_profile_with_default_addon() {
        let result: Value =
            serde_json::from_str(&profile_default_seed_json("{}").unwrap()).unwrap();
        assert_eq!(result["id"], "guest");
        assert_eq!(result["isGuest"], true);
        let addons = result["localAddons"].as_array().unwrap();
        assert!(!addons.is_empty());
    }
}
