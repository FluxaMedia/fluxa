use crate::action_contract::{
    MarkWatchedAction, SavePlaybackProgressAction, mark_watched_action_value,
    save_playback_progress_action_value,
};
use crate::library_state::{UP_NEXT_DURATION_SECONDS, UP_NEXT_POSITION_SECONDS};
use serde_json::{Value, json};

pub(crate) fn playback_close_plan_json(input: &str) -> Option<String> {
    let value: Value = serde_json::from_str(input).ok()?;
    let meta = value.get("meta")?;
    let episode = value.get("episode").filter(|value| !value.is_null());
    let stream = value.get("stream").filter(|value| !value.is_null());
    let next_episode = value.get("nextEpisode").filter(|value| !value.is_null());
    let time_pos = value
        .get("timePos")
        .and_then(Value::as_f64)
        .unwrap_or_default();
    let duration = value
        .get("duration")
        .and_then(Value::as_f64)
        .unwrap_or_default();
    let playback_started = value
        .get("playbackStarted")
        .and_then(Value::as_bool)
        .unwrap_or(true);
    let prefs = value.get("prefs").cloned().unwrap_or_else(|| json!({}));
    let safe_prefs: Value = crate::profile_prefs::profile_safe_prefs_json(&prefs.to_string())
        .and_then(|json| serde_json::from_str(&json).ok())
        .unwrap_or_else(|| json!({"watchedThresholdPercent": 90.0}));
    let threshold = safe_prefs
        .get("watchedThresholdPercent")
        .and_then(Value::as_f64)
        .filter(|value| *value > 0.0)
        .unwrap_or(90.0)
        / 100.0;
    let scrobble_pause = value
        .get("scrobbleTraktPause")
        .and_then(Value::as_bool)
        .unwrap_or(true);
    let meaningful = playback_started && time_pos > 30.0 && duration > 0.0;
    let watched = meaningful && time_pos / duration >= threshold;
    let text_field = |source: Option<&Value>, names: &[&str]| {
        names
            .iter()
            .find_map(|name| source.and_then(|source| source.get(*name)))
            .and_then(Value::as_str)
            .map(str::to_string)
    };
    let number_field = |source: Option<&Value>, names: &[&str]| {
        names
            .iter()
            .find_map(|name| source.and_then(|source| source.get(*name)))
            .and_then(Value::as_i64)
    };
    let progress = |target: Option<&Value>,
                    position: i64,
                    target_duration: i64,
                    scrobble: bool,
                    include_stream: bool| {
        save_playback_progress_action_value(&SavePlaybackProgressAction {
            profile: None,
            meta: meta.clone(),
            time_offset: position,
            duration: target_duration,
            last_video_id: text_field(target, &["id"]),
            last_stream_index: include_stream
                .then(|| {
                    value
                        .get("streamIndex")
                        .and_then(Value::as_i64)
                        .and_then(|index| i32::try_from(index).ok())
                })
                .flatten(),
            last_episode_name: text_field(target, &["name", "title"]),
            last_episode_season: number_field(target, &["season"]),
            last_episode_number: number_field(target, &["episode", "number"]),
            last_episode_thumbnail: text_field(target, &["thumbnail"]),
            last_stream_url: include_stream
                .then(|| text_field(stream, &["playableUrl", "url"]))
                .flatten(),
            last_stream_title: include_stream
                .then(|| text_field(stream, &["title", "name"]))
                .flatten(),
            last_audio_language: None,
            last_subtitle_language: None,
            scrobble_trakt_pause: Some(scrobble),
            refresh_external_continue_watching: Some(scrobble && meaningful),
        })
    };
    let progress_action = playback_started.then(|| {
        progress(
            episode,
            if meaningful {
                time_pos.floor() as i64
            } else {
                1
            },
            if duration > 0.0 {
                duration.floor() as i64
            } else {
                0
            },
            scrobble_pause,
            true,
        )
    });
    let mark_watched_action = watched.then(|| {
        mark_watched_action_value(&MarkWatchedAction {
            series_id: text_field(Some(meta), &["id"]).unwrap_or_default(),
            video_ids: text_field(episode.or(Some(meta)), &["id"])
                .into_iter()
                .collect(),
            watched: Some(true),
            meta: Some(meta.clone()),
            episodes: episode.map(|episode| {
                vec![json!({
                    "id": text_field(Some(episode), &["id"]),
                    "name": text_field(Some(episode), &["name", "title"]),
                    "season": number_field(Some(episode), &["season"]),
                    "number": number_field(Some(episode), &["episode", "number"]),
                    "thumbnail": text_field(Some(episode), &["thumbnail"]),
                })]
            }),
            profile: None,
        })
    });
    let up_next_action = (watched
        && meta.get("type").and_then(Value::as_str) == Some("series")
        && next_episode.is_some())
    .then(|| {
        progress(
            next_episode,
            UP_NEXT_POSITION_SECONDS,
            UP_NEXT_DURATION_SECONDS,
            false,
            false,
        )
    });
    serde_json::to_string(&json!({"shouldScrobble": meaningful, "progressAction": progress_action, "markWatchedAction": mark_watched_action, "upNextAction": up_next_action, "reloadHome": meaningful})).ok()
}

pub(crate) fn playback_preferences_plan_json(input: &str) -> Option<String> {
    let prefs: Value = serde_json::from_str(input).ok()?;
    let safe: Value = crate::profile_prefs::profile_safe_prefs_json(input)
        .and_then(|json| serde_json::from_str(&json).ok())?;
    serde_json::to_string(&json!({
        "watchedThresholdPercent": safe.get("watchedThresholdPercent"),
        "nextEpisodeThresholdPercent": safe.get("nextEpisodeThresholdPercent"),
        "autoPlayNextEpisode": safe.get("autoPlayNextEpisode"),
        "autoSkipIntro": safe.get("autoSkipIntro"),
        "autoPlayCountdownSecs": prefs.get("autoPlayCountdownSecs").and_then(Value::as_i64).unwrap_or(7).clamp(1, 60),
        "useSkipSegments": safe.get("useSkipSegments"),
        "useAnimeSkip": prefs.get("useAnimeSkip").and_then(Value::as_bool).unwrap_or(true),
        "animeSkipClientId": prefs.get("animeSkipClientId").and_then(Value::as_str).unwrap_or(""),
    })).ok()
}
