const SCROBBLE_START_PROGRESS_PERCENT: f32 = 0.2;
const SCROBBLE_STOP_PROGRESS_PERCENT: f32 = 80.0;
const DURABLE_SCROBBLE_MIN_PROGRESS_PERCENT: f32 = 1.0;
const PERIODIC_PROGRESS_SAVE_MS: i64 = 30_000;
const DISPOSAL_PROGRESS_SAVE_MIN_MS: i64 = 5_000;

pub(crate) fn progress_percent(position_ms: i64, duration_ms: i64) -> f32 {
    if duration_ms <= 0 {
        return 0.0;
    }
    ((position_ms as f32 / duration_ms as f32) * 100.0).clamp(0.0, 100.0)
}

pub(crate) fn should_send_start(
    token: Option<&str>,
    is_playing: bool,
    has_scrobbled_start: bool,
    progress: f32,
) -> bool {
    has_token(token)
        && is_playing
        && !has_scrobbled_start
        && progress > SCROBBLE_START_PROGRESS_PERCENT
}

pub(crate) fn should_mark_stopped(has_scrobbled_stop: bool, progress: f32) -> bool {
    !has_scrobbled_stop && progress >= SCROBBLE_STOP_PROGRESS_PERCENT
}

pub(crate) fn scrobble_close_action(time_pos_sec: f64, duration_sec: f64) -> &'static str {
    let percent = if duration_sec > 0.0 {
        (time_pos_sec / duration_sec) * 100.0
    } else {
        0.0
    };
    if percent >= SCROBBLE_STOP_PROGRESS_PERCENT as f64 {
        "stop"
    } else {
        "pause"
    }
}

pub(crate) fn should_queue_pause(
    token: Option<&str>,
    was_play_when_ready: bool,
    has_scrobbled_start: bool,
    has_scrobbled_stop: bool,
) -> bool {
    has_token(token) && was_play_when_ready && has_scrobbled_start && !has_scrobbled_stop
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ScrobbleAction {
    Start,
    Pause,
    Stop,
    Unknown,
}

impl From<&str> for ScrobbleAction {
    fn from(value: &str) -> Self {
        match value {
            "start" => ScrobbleAction::Start,
            "pause" => ScrobbleAction::Pause,
            "stop" => ScrobbleAction::Stop,
            _ => ScrobbleAction::Unknown,
        }
    }
}

pub(crate) fn should_enqueue_durable(action: &str, token: Option<&str>, progress: f32) -> bool {
    if !has_token(token) {
        return false;
    }
    !matches!(
        ScrobbleAction::from(action),
        ScrobbleAction::Pause | ScrobbleAction::Stop
    ) || progress >= DURABLE_SCROBBLE_MIN_PROGRESS_PERCENT
}

pub(crate) fn should_save_periodic_progress(
    is_playing: bool,
    now_ms: i64,
    last_saved_at_ms: i64,
) -> bool {
    is_playing && now_ms - last_saved_at_ms > PERIODIC_PROGRESS_SAVE_MS
}

pub(crate) fn should_save_on_dispose(position_ms: i64) -> bool {
    position_ms > DISPOSAL_PROGRESS_SAVE_MIN_MS
}

fn has_token(token: Option<&str>) -> bool {
    token.is_some_and(|value| !value.trim().is_empty())
}

pub(crate) fn lifecycle_action_json(args_json: &str) -> Option<String> {
    let args: serde_json::Value = serde_json::from_str(args_json).ok()?;
    let event = args.get("event")?.as_str()?;
    let has_started = args
        .get("hasStarted")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);
    let has_paused = args
        .get("hasPaused")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);
    let has_stopped = args
        .get("hasStopped")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);
    let progress = args
        .get("progress")
        .and_then(serde_json::Value::as_f64)
        .unwrap_or(0.0) as f32;
    let token = args.get("token").and_then(serde_json::Value::as_str);
    let action = match event {
        "start" if should_send_start(token, true, !has_paused && has_started, progress) => "start",
        "pause" if should_queue_pause(token, true, has_started, has_stopped) => "pause",
        "stop" if has_started && !has_stopped => "stop",
        _ => return None,
    };
    serde_json::to_string(&serde_json::json!({ "action": action })).ok()
}

pub(crate) fn scrobble_media_context_json(args_json: &str) -> Option<String> {
    let args: serde_json::Value = serde_json::from_str(args_json).ok()?;
    let meta = args.get("meta")?;
    let episode = args.get("episode").filter(|value| !value.is_null());
    let profile = args.get("profile");
    let now_seconds = args
        .get("nowSeconds")
        .and_then(serde_json::Value::as_i64)
        .unwrap_or(0);
    let trakt_enabled = profile
        .and_then(|value| value.get("traktAccessToken"))
        .and_then(serde_json::Value::as_str)
        .is_some_and(|value| !value.trim().is_empty())
        && profile
            .and_then(|value| value.get("traktTokenExpiresAt"))
            .and_then(serde_json::Value::as_i64)
            .is_none_or(|expires| now_seconds <= expires);
    let is_episode =
        meta.get("type").and_then(serde_json::Value::as_str) == Some("series") && episode.is_some();
    let season = episode
        .and_then(|value| value.get("season"))
        .and_then(serde_json::Value::as_i64)
        .unwrap_or(1);
    let number = episode
        .and_then(|value| value.get("episode").or_else(|| value.get("number")))
        .and_then(serde_json::Value::as_i64)
        .unwrap_or(1);
    serde_json::to_string(&serde_json::json!({
        "videoId": meta.get("id"),
        "isEpisode": is_episode,
        "simklType": if is_episode { "tv" } else { "movie" },
        "season": season,
        "episode": number,
        "releaseDate": episode.and_then(|value| value.get("released")).and_then(serde_json::Value::as_str).map(|value| value.chars().take(10).collect::<String>()),
        "episodeTitle": episode.and_then(|value| value.get("name").or_else(|| value.get("title"))).and_then(serde_json::Value::as_str).unwrap_or(""),
        "traktEnabled": trakt_enabled,
    }))
    .ok()
}

pub(crate) fn trakt_scrobble_plan_json(
    ids_json: &str,
    is_episode: bool,
    season: Option<i64>,
    ep_number: Option<i64>,
    time_pos_sec: f64,
    duration_sec: f64,
    requested_action: Option<&str>,
) -> Option<String> {
    let ids: serde_json::Value = serde_json::from_str(ids_json).ok()?;
    if duration_sec <= 0.0 {
        return None;
    }
    let progress = ((time_pos_sec / duration_sec) * 100.0).clamp(0.0, 100.0);
    let action = match requested_action {
        Some("start") => "start",
        Some("pause") => "pause",
        Some("stop") => "stop",
        _ if progress as f32 >= SCROBBLE_STOP_PROGRESS_PERCENT => "stop",
        _ => "pause",
    };
    let body = if is_episode {
        serde_json::json!({
            "show": { "ids": ids },
            "episode": { "season": season.unwrap_or(1), "number": ep_number.unwrap_or(1) },
            "progress": progress
        })
    } else {
        serde_json::json!({ "movie": { "ids": ids }, "progress": progress })
    };
    serde_json::to_string(&serde_json::json!({ "action": action, "body": body })).ok()
}

pub(crate) fn simkl_scrobble_body_json(
    ids_json: &str,
    is_episode: bool,
    season: i64,
    ep_number: i64,
    time_pos_sec: f64,
    duration_sec: f64,
) -> Option<String> {
    let ids: serde_json::Value = serde_json::from_str(ids_json).ok()?;
    if duration_sec <= 0.0 {
        return None;
    }
    let progress = ((time_pos_sec / duration_sec) * 100.0).clamp(0.0, 100.0);
    let body = if is_episode {
        serde_json::json!({
            "show": { "ids": ids },
            "episode": { "season": season, "number": ep_number },
            "progress": progress
        })
    } else {
        serde_json::json!({ "movie": { "ids": ids }, "progress": progress })
    };
    serde_json::to_string(&body).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn progress_percent_is_clamped_and_zero_for_missing_duration() {
        assert_eq!(progress_percent(1_000, 0), 0.0);
        assert_eq!(progress_percent(5_000, 10_000), 50.0);
        assert_eq!(progress_percent(20_000, 10_000), 100.0);
    }

    #[test]
    fn start_requires_token_playing_and_initial_progress() {
        assert!(!should_send_start(None, true, false, 1.0));
        assert!(!should_send_start(Some("token"), false, false, 1.0));
        assert!(!should_send_start(Some("token"), true, true, 1.0));
        assert!(!should_send_start(Some("token"), true, false, 0.1));
        assert!(should_send_start(Some("token"), true, false, 0.3));
    }

    #[test]
    fn lifecycle_actions_follow_start_pause_stop_order() {
        assert_eq!(lifecycle_action_json(r#"{"event":"start","token":"token","hasStarted":false,"hasStopped":false,"progress":1}"#).as_deref(), Some(r#"{"action":"start"}"#));
        assert_eq!(lifecycle_action_json(r#"{"event":"start","token":"token","hasStarted":true,"hasPaused":true,"hasStopped":false,"progress":1}"#).as_deref(), Some(r#"{"action":"start"}"#));
        assert_eq!(lifecycle_action_json(r#"{"event":"pause","token":"token","hasStarted":true,"hasStopped":false,"progress":1}"#).as_deref(), Some(r#"{"action":"pause"}"#));
        assert_eq!(lifecycle_action_json(r#"{"event":"stop","token":"token","hasStarted":true,"hasStopped":false,"progress":1}"#).as_deref(), Some(r#"{"action":"stop"}"#));
    }

    #[test]
    fn pause_stop_and_save_thresholds_match_platform_contract() {
        assert!(!should_mark_stopped(false, 79.9));
        assert!(should_mark_stopped(false, 80.0));
        assert!(!should_mark_stopped(true, 90.0));
        assert!(!should_enqueue_durable("pause", Some("token"), 0.5));
        assert!(should_enqueue_durable("pause", Some("token"), 1.0));
        assert!(should_enqueue_durable("start", Some("token"), 0.1));
        assert!(!should_save_periodic_progress(true, 30_000, 0));
        assert!(should_save_periodic_progress(true, 30_001, 0));
        assert!(!should_save_on_dispose(5_000));
        assert!(should_save_on_dispose(5_001));
    }
}
