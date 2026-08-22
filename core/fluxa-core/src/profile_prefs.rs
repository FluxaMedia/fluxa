use crate::constants::DEFAULT_LANGUAGE;
use serde::Serialize;
use serde_json::Value;

pub(crate) fn safe_player_buffer_cache_mb(value: Option<i32>) -> i32 {
    value.unwrap_or(100).clamp(100, 2000)
}

pub(crate) fn safe_dolby_vision_fallback_mode(
    mode: Option<&str>,
    legacy_dv7_fallback: Option<bool>,
    legacy_dv7_to_dv8_fallback: Option<bool>,
) -> &'static str {
    match mode {
        Some("auto") => "auto",
        Some("convert_dv81") => "convert_dv81",
        Some("hdr10") => "hdr10",
        Some("dv8") => "dv8",
        Some("off") => "off",
        _ if legacy_dv7_to_dv8_fallback == Some(true) => "dv8",
        _ if legacy_dv7_fallback != Some(false) => "hdr10",
        _ => "off",
    }
}

pub(crate) fn safe_stream_source_selection_mode(mode: Option<&str>) -> &'static str {
    match mode {
        Some("first") => "first",
        Some("regex") => "regex",
        _ => "manual",
    }
}

pub(crate) fn safe_audio_processing_mode(mode: Option<&str>) -> &'static str {
    match mode {
        Some("balanced") => "balanced",
        Some("night") => "night",
        _ => "reference",
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ProfileSafePrefs {
    language: String,
    subtitle_size_percent: f32,
    subtitle_size: f32,
    subtitle_color: i64,
    subtitle_background_color: i64,
    subtitle_outline_color: i64,
    subtitle_text_opacity: f32,
    subtitle_background_opacity: f32,
    subtitle_outline_opacity: f32,
    preferred_subtitle_language: String,
    preferred_audio_language: String,
    secondary_subtitle_language: String,
    secondary_audio_language: String,
    audio_processing_mode: String,
    stable_volume: bool,
    ambient_light: bool,
    force_software_audio: bool,
    preferred_player: String,
    continue_watching_source: String,
    card_layout: String,
    continue_watching_layout: String,
    continue_watching_artwork: String,
    continue_watching_enabled: bool,
    upcoming_row_enabled: bool,
    resolved_continue_watching_layout: String,
    subtitle_shadow: bool,
    auto_enable_subtitles: bool,
    auto_skip_intro: bool,
    auto_play_next_episode: bool,
    next_episode_threshold_percent: f32,
    watched_threshold_percent: f32,
    seek_forward_seconds: i64,
    seek_backward_seconds: i64,
    player_buffer_cache_mb: i32,
    player_forward_buffer_seconds: i64,
    player_back_buffer_seconds: i64,
    timezone_conversion_enabled: bool,
    torrent_wifi_only: bool,
    torrent_max_connections: i64,
    torrent_speed_preset: String,
    torrent_cache_preset: String,
    app_theme: String,
    accent_color_argb: i64,
    card_corner_preset: String,
    interface_density: String,
    amoled_mode: bool,
    poster_width_preset: String,
    poster_landscape_mode: bool,
    poster_hide_titles: bool,
    detail_episode_view_mode: String,
    animations_enabled: bool,
    reduce_motion: bool,
    start_page: String,
    notifications_enabled: bool,
    alert_new_episodes: bool,
    automatic_updates: bool,
    background_playback: bool,
    picture_in_picture: bool,
    playback_speed: f32,
    hold_to_speed_enabled: bool,
    hold_speed: f32,
    dolby_vision_fallback_mode: String,
    dv7_to_dv8_fallback: bool,
    dv7_fallback: bool,
    tunneled_playback: bool,
    use_skip_segments: bool,
    default_quality: String,
    mobile_data_usage: String,
    hdr_playback: bool,
    resume_playback: bool,
    autoplay_mode: String,
    stream_source_selection_mode: String,
    stream_source_regex_pattern: String,
    try_binge_group: bool,
    show_hero_section: bool,
    trakt_token_expires_at: i64,
    trakt_last_sync_at: i64,
    trakt_last_synced_items: i64,
    trakt_last_continue_watching_count: i64,
    trakt_last_watchlist_count: i64,
}

pub(crate) fn profile_safe_prefs_json(profile_json: &str) -> Option<String> {
    let profile = serde_json::from_str::<Value>(profile_json).ok()?;
    serde_json::to_string(&profile_safe_prefs(&profile)).ok()
}

fn profile_safe_prefs(profile: &Value) -> ProfileSafePrefs {
    let subtitle_size_percent =
        safe_subtitle_size_percent(number(profile, "subtitleSize").unwrap_or(100.0) as f32);
    let card_layout = safe_card_layout(text(profile, "cardLayout"));
    let continue_watching_layout =
        safe_continue_watching_layout(text(profile, "continueWatchingLayout"));
    let dolby_mode = safe_dolby_vision_fallback_mode(
        text(profile, "dolbyVisionFallbackMode"),
        bool_value(profile, "dv7Fallback"),
        bool_value(profile, "dv7ToDv8Fallback"),
    )
    .to_string();

    ProfileSafePrefs {
        language: text(profile, "language")
            .unwrap_or(DEFAULT_LANGUAGE)
            .to_string(),
        subtitle_size_percent,
        subtitle_size: 20.0 * (subtitle_size_percent / 100.0),
        subtitle_color: int(profile, "subtitleColor").unwrap_or(0xFFFF_FFFFu32 as i32 as i64),
        subtitle_background_color: int(profile, "subtitleBackgroundColor")
            .unwrap_or(0x8000_0000u32 as i32 as i64),
        subtitle_outline_color: int(profile, "subtitleOutlineColor")
            .unwrap_or(0xFF00_0000u32 as i32 as i64),
        subtitle_text_opacity: number(profile, "subtitleTextOpacity").unwrap_or(1.0) as f32,
        subtitle_background_opacity: number(profile, "subtitleBackgroundOpacity").unwrap_or(0.5)
            as f32,
        subtitle_outline_opacity: number(profile, "subtitleOutlineOpacity").unwrap_or(1.0) as f32,
        preferred_subtitle_language: text(profile, "preferredSubtitleLanguage")
            .unwrap_or("none")
            .to_string(),
        preferred_audio_language: text(profile, "preferredAudioLanguage")
            .unwrap_or("none")
            .to_string(),
        secondary_subtitle_language: text(profile, "secondarySubtitleLanguage")
            .unwrap_or("none")
            .to_string(),
        secondary_audio_language: text(profile, "secondaryAudioLanguage")
            .unwrap_or("none")
            .to_string(),
        audio_processing_mode: safe_audio_processing_mode(text(profile, "audioProcessingMode"))
            .to_string(),
        stable_volume: bool_value(profile, "stableVolume").unwrap_or(false),
        ambient_light: bool_value(profile, "ambientLight").unwrap_or(true),
        force_software_audio: bool_value(profile, "forceSoftwareAudio").unwrap_or(false),
        preferred_player: safe_preferred_player(text(profile, "preferredPlayer")).to_string(),
        continue_watching_source: safe_continue_watching_source(text(
            profile,
            "continueWatchingSource",
        ))
        .to_string(),
        card_layout: card_layout.clone(),
        continue_watching_layout: continue_watching_layout.clone(),
        continue_watching_artwork: text(profile, "continueWatchingArtwork")
            .unwrap_or("episode")
            .to_string(),
        continue_watching_enabled: bool_value(profile, "continueWatchingEnabled").unwrap_or(true),
        upcoming_row_enabled: bool_value(profile, "upcomingRowEnabled").unwrap_or(false),
        resolved_continue_watching_layout: if continue_watching_layout == "inherit" {
            card_layout
        } else {
            continue_watching_layout
        },
        subtitle_shadow: bool_value(profile, "subtitleShadow").unwrap_or(true),
        auto_enable_subtitles: bool_value(profile, "autoEnableSubtitles").unwrap_or(true),
        auto_skip_intro: bool_value(profile, "autoSkipIntro").unwrap_or(false),
        auto_play_next_episode: bool_value(profile, "autoPlayNextEpisode").unwrap_or(true),
        next_episode_threshold_percent: number(profile, "nextEpisodeThresholdPercent")
            .unwrap_or(90.0)
            .clamp(50.0, 99.0) as f32,
        watched_threshold_percent: number(profile, "watchedThresholdPercent")
            .unwrap_or(80.0)
            .clamp(50.0, 99.0) as f32,
        seek_forward_seconds: int(profile, "seekForwardSeconds").unwrap_or(10),
        seek_backward_seconds: int(profile, "seekBackwardSeconds").unwrap_or(10),
        player_buffer_cache_mb: safe_player_buffer_cache_mb(
            int(profile, "playerBufferCacheMb").map(|v| v as i32),
        ),
        player_forward_buffer_seconds: int(profile, "playerForwardBufferSeconds")
            .unwrap_or(120)
            .clamp(30, 600),
        player_back_buffer_seconds: int(profile, "playerBackBufferSeconds")
            .unwrap_or(30)
            .clamp(0, 300),
        timezone_conversion_enabled: true,
        torrent_wifi_only: bool_value(profile, "torrentWifiOnly").unwrap_or(false),
        torrent_max_connections: int(profile, "torrentMaxConnections").unwrap_or(60),
        torrent_speed_preset: text(profile, "torrentSpeedPreset")
            .unwrap_or("default")
            .to_string(),
        torrent_cache_preset: text(profile, "torrentCachePreset")
            .unwrap_or("auto")
            .to_string(),
        app_theme: text(profile, "appTheme").unwrap_or("dark").to_string(),
        accent_color_argb: int(profile, "accentColorArgb").unwrap_or(0xFFFF_FFFFu32 as i32 as i64),
        card_corner_preset: text(profile, "cardCornerPreset")
            .unwrap_or("medium")
            .to_string(),
        interface_density: text(profile, "interfaceDensity")
            .unwrap_or("medium")
            .to_string(),
        amoled_mode: bool_value(profile, "amoledMode").unwrap_or(false),
        poster_width_preset: text(profile, "posterWidthPreset")
            .unwrap_or("medium")
            .to_string(),
        poster_landscape_mode: bool_value(profile, "posterLandscapeMode").unwrap_or(false),
        poster_hide_titles: bool_value(profile, "posterHideTitles").unwrap_or(false),
        detail_episode_view_mode: safe_detail_episode_view_mode(text(
            profile,
            "detailEpisodeViewMode",
        ))
        .to_string(),
        animations_enabled: bool_value(profile, "animationsEnabled").unwrap_or(true),
        reduce_motion: bool_value(profile, "reduceMotion").unwrap_or(false),
        start_page: text(profile, "startPage").unwrap_or("home").to_string(),
        notifications_enabled: bool_value(profile, "notificationsEnabled").unwrap_or(true),
        alert_new_episodes: bool_value(profile, "alertNewEpisodes").unwrap_or(true),
        automatic_updates: bool_value(profile, "automaticUpdates").unwrap_or(true),
        background_playback: bool_value(profile, "backgroundPlayback").unwrap_or(false),
        picture_in_picture: bool_value(profile, "pictureInPicture").unwrap_or(true),
        playback_speed: number(profile, "playbackSpeed").unwrap_or(1.0) as f32,
        hold_to_speed_enabled: bool_value(profile, "holdToSpeedEnabled").unwrap_or(true),
        hold_speed: number(profile, "holdSpeed").unwrap_or(2.0) as f32,
        dv7_to_dv8_fallback: dolby_mode == "dv8",
        dv7_fallback: dolby_mode == "hdr10",
        dolby_vision_fallback_mode: dolby_mode,
        tunneled_playback: bool_value(profile, "tunneledPlayback").unwrap_or(false),
        use_skip_segments: bool_value(profile, "useSkipSegments").unwrap_or(true),
        default_quality: text(profile, "defaultQuality")
            .unwrap_or("1080p")
            .to_string(),
        mobile_data_usage: text(profile, "mobileDataUsage")
            .unwrap_or("medium")
            .to_string(),
        hdr_playback: bool_value(profile, "hdrPlayback").unwrap_or(true),
        resume_playback: bool_value(profile, "resumePlayback").unwrap_or(true),
        autoplay_mode: text(profile, "autoplayMode")
            .unwrap_or("next_episode")
            .to_string(),
        stream_source_selection_mode: safe_stream_source_selection_mode(text(
            profile,
            "streamSourceSelectionMode",
        ))
        .to_string(),
        stream_source_regex_pattern: text(profile, "streamSourceRegexPattern")
            .unwrap_or("")
            .to_string(),
        try_binge_group: bool_value(profile, "tryBingeGroup").unwrap_or(false),
        show_hero_section: bool_value(profile, "showHeroSection").unwrap_or(true),
        trakt_token_expires_at: int(profile, "traktTokenExpiresAt").unwrap_or(0),
        trakt_last_sync_at: int(profile, "traktLastSyncAt").unwrap_or(0),
        trakt_last_synced_items: int(profile, "traktLastSyncedItems").unwrap_or(0),
        trakt_last_continue_watching_count: int(profile, "traktLastContinueWatchingCount")
            .unwrap_or(0),
        trakt_last_watchlist_count: int(profile, "traktLastWatchlistCount").unwrap_or(0),
    }
}

fn safe_subtitle_size_percent(value: f32) -> f32 {
    if value <= 40.0 {
        ((value / 20.0) * 100.0).clamp(50.0, 200.0)
    } else {
        value.clamp(50.0, 200.0)
    }
}

fn safe_preferred_player(value: Option<&str>) -> &'static str {
    match value {
        Some("mpv") => "mpv",
        _ => "exoplayer",
    }
}

pub(crate) fn safe_continue_watching_source(value: Option<&str>) -> &'static str {
    crate::library_state::normalized_continue_watching_source(value)
}

fn safe_card_layout(value: Option<&str>) -> String {
    match value {
        Some("episode") => "horizontal".to_string(),
        None => "vertical".to_string(),
        Some(value) => value.to_string(),
    }
}

fn safe_continue_watching_layout(value: Option<&str>) -> String {
    match value {
        Some("episode") => "horizontal".to_string(),
        None => "horizontal".to_string(),
        Some(value) => value.to_string(),
    }
}

fn safe_detail_episode_view_mode(value: Option<&str>) -> &'static str {
    match value {
        Some("legacy") => "legacy",
        _ => "modern",
    }
}

fn text<'a>(value: &'a Value, key: &str) -> Option<&'a str> {
    value
        .get(key)
        .and_then(Value::as_str)
        .filter(|v| !v.is_empty())
}

fn bool_value(value: &Value, key: &str) -> Option<bool> {
    value.get(key).and_then(Value::as_bool)
}

fn int(value: &Value, key: &str) -> Option<i64> {
    value.get(key).and_then(Value::as_i64)
}

fn number(value: &Value, key: &str) -> Option<f64> {
    value.get(key).and_then(Value::as_f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn player_buffer_cache_is_clamped() {
        assert_eq!(safe_player_buffer_cache_mb(None), 100);
        assert_eq!(safe_player_buffer_cache_mb(Some(50)), 100);
        assert_eq!(safe_player_buffer_cache_mb(Some(500)), 500);
        assert_eq!(safe_player_buffer_cache_mb(Some(5000)), 2000);
    }

    #[test]
    fn dolby_vision_mode_keeps_explicit_and_migrates_legacy_flags() {
        assert_eq!(
            safe_dolby_vision_fallback_mode(Some("auto"), None, None),
            "auto"
        );
        assert_eq!(
            safe_dolby_vision_fallback_mode(None, Some(false), Some(true)),
            "dv8",
        );
        assert_eq!(safe_dolby_vision_fallback_mode(None, None, None), "hdr10");
        assert_eq!(
            safe_dolby_vision_fallback_mode(None, Some(false), Some(false)),
            "off",
        );
    }

    #[test]
    fn stream_source_mode_defaults_to_manual() {
        assert_eq!(safe_stream_source_selection_mode(Some("first")), "first");
        assert_eq!(safe_stream_source_selection_mode(Some("regex")), "regex");
        assert_eq!(safe_stream_source_selection_mode(Some("manual")), "manual");
        assert_eq!(safe_stream_source_selection_mode(None), "manual");
    }

    #[test]
    fn audio_processing_mode_is_closed_to_safe_values() {
        assert_eq!(safe_audio_processing_mode(Some("reference")), "reference");
        assert_eq!(safe_audio_processing_mode(Some("balanced")), "balanced");
        assert_eq!(safe_audio_processing_mode(Some("night")), "night");
        assert_eq!(safe_audio_processing_mode(Some("passthrough")), "reference");
        assert_eq!(safe_audio_processing_mode(None), "reference");
    }

    #[test]
    fn profile_safe_prefs_match_kotlin_defaults_and_migrations() {
        let json = profile_safe_prefs_json(
            r#"{
                "language":null,
                "subtitleSize":20,
                "preferredPlayer":"internal",
                "cardLayout":"episode",
                "continueWatchingLayout":"inherit",
                "playerBufferCacheMb":50,
                "playerForwardBufferSeconds":999,
                "playerBackBufferSeconds":-1,
                "detailEpisodeViewMode":"unknown",
                "dolbyVisionFallbackMode":null,
                "dv7Fallback":false,
                "dv7ToDv8Fallback":true,
                "streamSourceSelectionMode":"invalid"
            }"#,
        )
        .expect("profile safe prefs");
        let value: Value = serde_json::from_str(&json).unwrap();

        assert_eq!(value["language"], "en");
        assert_eq!(value["subtitleSizePercent"], 100.0);
        assert_eq!(value["preferredPlayer"], "exoplayer");
        assert_eq!(value["cardLayout"], "horizontal");
        assert_eq!(value["resolvedContinueWatchingLayout"], "horizontal");
        assert_eq!(value["playerBufferCacheMb"], 100);
        assert_eq!(value["playerForwardBufferSeconds"], 600);
        assert_eq!(value["playerBackBufferSeconds"], 0);
        assert_eq!(value["detailEpisodeViewMode"], "modern");
        assert_eq!(value["dolbyVisionFallbackMode"], "dv8");
        assert_eq!(value["streamSourceSelectionMode"], "manual");
        assert_eq!(value["audioProcessingMode"], "reference");
    }
}
