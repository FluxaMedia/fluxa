use crate::headless_engine::contracts::actions::{
    MarkWatchedAction, SavePlaybackProgressAction, mark_watched_action_value,
    save_playback_progress_action_value,
};
use crate::library::state::{UP_NEXT_DURATION_SECONDS, UP_NEXT_POSITION_SECONDS};
use serde_json::{Value, json};

const WATCHED_FRACTION: f64 = 0.9;
const MIN_REAL_DURATION_SECONDS: f64 = 121.0;

fn is_placeholder_duration(duration: f64) -> bool {
    duration > 0.0 && duration < MIN_REAL_DURATION_SECONDS
}

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
    let scrobble_pause = value
        .get("scrobbleTraktPause")
        .and_then(Value::as_bool)
        .unwrap_or(true);
    let meaningful = playback_started && time_pos > 30.0 && duration > 0.0;
    let watched = meaningful
        && !is_placeholder_duration(duration)
        && time_pos / duration >= WATCHED_FRACTION;
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
            rewatch: false,
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
    let safe: Value = crate::profile::prefs::profile_safe_prefs_json(input)
        .and_then(|json| serde_json::from_str(&json).ok())?;
    serde_json::to_string(&json!({
        "nextEpisodeThresholdPercent": safe.get("nextEpisodeThresholdPercent"),
        "autoPlayNextEpisode": safe.get("autoPlayNextEpisode"),
        "autoSkipIntro": safe.get("autoSkipIntro"),
        "autoPlayCountdownSecs": prefs.get("autoPlayCountdownSecs").and_then(Value::as_i64).unwrap_or(7).clamp(1, 60),
        "useSkipSegments": safe.get("useSkipSegments"),
        "useAnimeSkip": prefs.get("useAnimeSkip").and_then(Value::as_bool).unwrap_or(true),
        "animeSkipClientId": prefs.get("animeSkipClientId").and_then(Value::as_str).unwrap_or(""),
    })).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn plan(time_pos: f64, duration: f64) -> Value {
        let input = json!({
            "meta": {"id": "tt1", "type": "movie", "name": "A"},
            "timePos": time_pos,
            "duration": duration,
        });
        serde_json::from_str(&playback_close_plan_json(&input.to_string()).unwrap()).unwrap()
    }

    #[test]
    fn marks_watched_at_ninety_percent() {
        assert!(!plan(5500.0, 6000.0)["markWatchedAction"].is_null());
        assert!(plan(5000.0, 6000.0)["markWatchedAction"].is_null());
    }

    #[test]
    fn placeholder_clip_is_never_watched() {
        assert!(plan(100.0, 110.0)["markWatchedAction"].is_null());
    }
}
