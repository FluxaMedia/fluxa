use serde_json::{Map, Value, json};

pub(crate) fn sanitize_profile_json(
    profile_json: &str,
    mirrored_addons_json: &str,
    merge_mirrored_addons: bool,
) -> Option<String> {
    let mut profile: Value = serde_json::from_str(profile_json).ok()?;
    let mirrored_addons: Vec<String> = if merge_mirrored_addons {
        serde_json::from_str(mirrored_addons_json).unwrap_or_default()
    } else {
        Vec::new()
    };

    let mut base_addons = string_list_field(&profile, "localAddons");
    base_addons.extend(mirrored_addons);
    let cleaned_addons = normalize_distinct_addons(base_addons);
    let cleaned_ids = cleaned_addons
        .iter()
        .map(|addon| crate::addon_protocol::identity(addon))
        .collect::<std::collections::HashSet<_>>();
    let cleaned_disabled_addons =
        normalize_distinct_addons(string_list_field(&profile, "disabledLocalAddons"))
            .into_iter()
            .filter(|addon| cleaned_ids.contains(&crate::addon_protocol::identity(addon)))
            .collect::<Vec<_>>();

    let object = profile.as_object_mut()?;
    object.insert("localAddons".to_string(), json!(cleaned_addons));
    object.insert(
        "disabledLocalAddons".to_string(),
        json!(cleaned_disabled_addons),
    );
    fill_structured_settings(object);
    serde_json::to_string(&profile).ok()
}

pub(crate) fn addon_profile_mutation_plan_json(args_json: &str) -> Option<String> {
    let args: Value = serde_json::from_str(args_json).ok()?;
    let mut profile = args.get("profile")?.clone();
    let command = args.get("command")?.as_str()?;
    let addon_key = args.get("addonKey")?.as_str()?;
    let settings = profile.get("addonSettings");
    let mut local = settings
        .and_then(|value| value.get("localAddons"))
        .and_then(Value::as_array)
        .or_else(|| profile.get("localAddons").and_then(Value::as_array))
        .cloned()
        .unwrap_or_default();
    let mut disabled = settings
        .and_then(|value| value.get("disabledLocalAddons"))
        .and_then(Value::as_array)
        .or_else(|| profile.get("disabledLocalAddons").and_then(Value::as_array))
        .cloned()
        .unwrap_or_default();
    let target = crate::addon_protocol::identity(addon_key);
    match command {
        "install" => {
            if !local.iter().any(|value| {
                value
                    .as_str()
                    .is_some_and(|url| crate::addon_protocol::identity(url) == target)
            }) {
                local.push(Value::String(addon_key.to_string()));
            }
        }
        "remove" => {
            local.retain(|value| {
                value
                    .as_str()
                    .is_none_or(|url| crate::addon_protocol::identity(url) != target)
            });
            disabled.retain(|value| {
                value
                    .as_str()
                    .is_none_or(|url| crate::addon_protocol::identity(url) != target)
            });
        }
        "toggle" => {
            if disabled.iter().any(|value| {
                value
                    .as_str()
                    .is_some_and(|url| crate::addon_protocol::identity(url) == target)
            }) {
                disabled.retain(|value| {
                    value
                        .as_str()
                        .is_none_or(|url| crate::addon_protocol::identity(url) != target)
                });
            } else {
                disabled.push(Value::String(addon_key.to_string()));
            }
        }
        _ => return None,
    }
    let object = profile.as_object_mut()?;
    object.insert("localAddons".to_string(), Value::Array(local.clone()));
    let settings = object
        .entry("addonSettings")
        .or_insert_with(|| json!({}))
        .as_object_mut()?;
    settings.insert("localAddons".to_string(), Value::Array(local));
    settings.insert("disabledLocalAddons".to_string(), Value::Array(disabled));
    serde_json::to_string(&profile).ok()
}

fn normalize_distinct_addons(addons: Vec<String>) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    addons
        .into_iter()
        .map(|addon| crate::addon_protocol::normalize_manifest_url(&addon))
        .filter(|addon| !addon.trim().is_empty())
        .filter(|addon| seen.insert(crate::addon_protocol::identity(addon)))
        .collect()
}

fn fill_structured_settings(profile: &mut Map<String, Value>) {
    insert_object_from_fields(
        profile,
        "externalAccounts",
        &[
            ("traktAccessToken", "traktAccessToken"),
            ("traktRefreshToken", "traktRefreshToken"),
            ("traktTokenExpiresAt", "traktTokenExpiresAt"),
            ("traktLastSyncAt", "traktLastSyncAt"),
            ("traktLastSyncedItems", "traktLastSyncedItems"),
            (
                "traktLastContinueWatchingCount",
                "traktLastContinueWatchingCount",
            ),
            ("traktLastWatchlistCount", "traktLastWatchlistCount"),
            ("malAccessToken", "malAccessToken"),
            ("malRefreshToken", "malRefreshToken"),
            ("simklAccessToken", "simklAccessToken"),
        ],
    );
    insert_object_from_fields(
        profile,
        "addonSettings",
        &[
            ("localAddons", "localAddons"),
            ("disabledLocalAddons", "disabledLocalAddons"),
        ],
    );
    insert_object_from_fields(
        profile,
        "subtitleSettings",
        &[
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
        ],
    );
    insert_object_from_fields(
        profile,
        "playbackSettings",
        &[
            ("preferredAudioLanguage", "preferredAudioLanguage"),
            ("secondaryAudioLanguage", "secondaryAudioLanguage"),
            ("audioProcessingMode", "audioProcessingMode"),
            ("stableVolume", "stableVolume"),
            ("ambientLight", "ambientLight"),
            ("forceSoftwareAudio", "forceSoftwareAudio"),
            ("preferredPlayer", "preferredPlayer"),
            ("autoSkipIntro", "autoSkipIntro"),
            ("autoPlayNextEpisode", "autoPlayNextEpisode"),
            ("nextEpisodeThresholdPercent", "nextEpisodeThresholdPercent"),
            ("watchedThresholdPercent", "watchedThresholdPercent"),
            ("seekForwardSeconds", "seekForwardSeconds"),
            ("seekBackwardSeconds", "seekBackwardSeconds"),
            ("playerBufferCacheMb", "playerBufferCacheMb"),
            ("playerForwardBufferSeconds", "playerForwardBufferSeconds"),
            ("playerBackBufferSeconds", "playerBackBufferSeconds"),
            ("backgroundPlayback", "backgroundPlayback"),
            ("pictureInPicture", "pictureInPicture"),
            ("playbackSpeed", "playbackSpeed"),
            ("holdToSpeedEnabled", "holdToSpeedEnabled"),
            ("holdSpeed", "holdSpeed"),
            ("dolbyVisionFallbackMode", "dolbyVisionFallbackMode"),
            ("dv7Fallback", "dv7Fallback"),
            ("dv7ToDv8Fallback", "dv7ToDv8Fallback"),
            ("tunneledPlayback", "tunneledPlayback"),
            ("useSkipSegments", "useSkipSegments"),
            ("defaultQuality", "defaultQuality"),
            ("mobileDataUsage", "mobileDataUsage"),
            ("hdrPlayback", "hdrPlayback"),
            ("resumePlayback", "resumePlayback"),
            ("autoplayMode", "autoplayMode"),
            ("streamSourceSelectionMode", "streamSourceSelectionMode"),
            ("streamSourceRegexPattern", "streamSourceRegexPattern"),
            ("tryBingeGroup", "tryBingeGroup"),
        ],
    );
    insert_object_from_fields(
        profile,
        "torrentSettings",
        &[
            ("wifiOnly", "torrentWifiOnly"),
            ("maxConnections", "torrentMaxConnections"),
            ("speedPreset", "torrentSpeedPreset"),
            ("cachePreset", "torrentCachePreset"),
        ],
    );
    insert_object_from_fields(
        profile,
        "appearanceSettings",
        &[
            ("language", "language"),
            ("cardLayout", "cardLayout"),
            ("continueWatchingLayout", "continueWatchingLayout"),
            ("continueWatchingArtwork", "continueWatchingArtwork"),
            ("continueWatchingEnabled", "continueWatchingEnabled"),
            ("appTheme", "appTheme"),
            ("accentColorArgb", "accentColorArgb"),
            ("cardCornerPreset", "cardCornerPreset"),
            ("interfaceDensity", "interfaceDensity"),
            ("amoledMode", "amoledMode"),
            ("posterWidthPreset", "posterWidthPreset"),
            ("posterLandscapeMode", "posterLandscapeMode"),
            ("posterHideTitles", "posterHideTitles"),
            ("detailEpisodeViewMode", "detailEpisodeViewMode"),
            ("detailSeasonSelectorMode", "detailSeasonSelectorMode"),
            ("detailSeasonPostersOnHero", "detailSeasonPostersOnHero"),
            ("homeSeasonPostersOnHero", "homeSeasonPostersOnHero"),
            ("animationsEnabled", "animationsEnabled"),
            ("reduceMotion", "reduceMotion"),
            ("startPage", "startPage"),
            ("continueWatchingSource", "continueWatchingSource"),
        ],
    );
    insert_object_from_fields(
        profile,
        "homeFeedSettings",
        &[
            ("heroFeedToggles", "heroFeedToggles"),
            ("homeFeedToggles", "homeFeedToggles"),
            ("topTenFeedToggles", "topTenFeedToggles"),
            ("heroFeedOrder", "heroFeedOrder"),
            ("homeFeedOrder", "homeFeedOrder"),
            ("showHeroSection", "showHeroSection"),
            ("libraryCollections", "libraryCollections"),
        ],
    );
}

fn insert_object_from_fields(
    profile: &mut Map<String, Value>,
    target: &str,
    fields: &[(&str, &str)],
) {
    let updates = fields
        .iter()
        .map(|(target_key, source_key)| {
            (
                (*target_key).to_string(),
                profile.get(*source_key).cloned().unwrap_or(Value::Null),
            )
        })
        .collect::<Vec<_>>();
    if profile.get(target).is_some_and(|value| !value.is_null()) {
        if let Some(target_object) = profile.get_mut(target).and_then(Value::as_object_mut) {
            for (key, value) in updates {
                target_object.insert(key, value);
            }
        }
        return;
    }
    let object = updates.into_iter().collect::<Map<_, _>>();
    profile.insert(target.to_string(), Value::Object(object));
}

fn string_list_field(value: &Value, key: &str) -> Vec<String> {
    value
        .get(key)
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(str::to_string)
        .collect()
}
