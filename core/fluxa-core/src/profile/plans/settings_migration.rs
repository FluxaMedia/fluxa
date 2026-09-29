use super::*;

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

    serde_json::to_string(&json!({
        "migratedProfile": profile,
        "appliedMigrations": applied,
        "schemaVersion": 2
    }))
    .ok()
}
