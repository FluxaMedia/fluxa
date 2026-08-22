use super::*;

pub(super) fn route_player_scrobble(method: &str, args_json: &str) -> Outcome {
    match method {
        "playerScrobbleLifecycleAction" => {
            opt_json(player_scrobble::lifecycle_action_json(args_json))
        }
        "scrobbleMediaContext" => opt_json(player_scrobble::scrobble_media_context_json(args_json)),
        "scrobbleCloseAction" => {
            let args = object(args_json)?;
            let time_pos = field(&args, "timePosSec")?
                .as_f64()
                .ok_or_else(|| fail(ErrorKind::InvalidArgs, "timePosSec must be a number"))?;
            let duration = field(&args, "durationSec")?
                .as_f64()
                .ok_or_else(|| fail(ErrorKind::InvalidArgs, "durationSec must be a number"))?;
            Ok(Value::String(
                player_scrobble::scrobble_close_action(time_pos, duration).to_string(),
            ))
        }
        "playerProgressPercent" => {
            let args = object(args_json)?;
            let position_ms = field(&args, "positionMs")?
                .as_i64()
                .ok_or_else(|| fail(ErrorKind::InvalidArgs, "positionMs must be a number"))?;
            let duration_ms = field(&args, "durationMs")?
                .as_i64()
                .ok_or_else(|| fail(ErrorKind::InvalidArgs, "durationMs must be a number"))?;
            Ok(json!(player_scrobble::progress_percent(
                position_ms,
                duration_ms,
            )))
        }
        "playerShouldSendScrobbleStart" => {
            let args = object(args_json)?;
            let token = args.get("token").and_then(Value::as_str);
            let is_playing = field(&args, "isPlaying")?
                .as_bool()
                .ok_or_else(|| fail(ErrorKind::InvalidArgs, "isPlaying must be bool"))?;
            let has_scrobbled_start = field(&args, "hasScrobbledStart")?
                .as_bool()
                .ok_or_else(|| fail(ErrorKind::InvalidArgs, "hasScrobbledStart must be bool"))?;
            let progress = field(&args, "progress")?
                .as_f64()
                .ok_or_else(|| fail(ErrorKind::InvalidArgs, "progress must be a number"))?
                as f32;
            Ok(json!(player_scrobble::should_send_start(
                token,
                is_playing,
                has_scrobbled_start,
                progress,
            )))
        }
        "playerShouldMarkScrobbleStopped" => {
            let args = object(args_json)?;
            let has_scrobbled_stop = field(&args, "hasScrobbledStop")?
                .as_bool()
                .ok_or_else(|| fail(ErrorKind::InvalidArgs, "hasScrobbledStop must be bool"))?;
            let progress = field(&args, "progress")?
                .as_f64()
                .ok_or_else(|| fail(ErrorKind::InvalidArgs, "progress must be a number"))?
                as f32;
            Ok(json!(player_scrobble::should_mark_stopped(
                has_scrobbled_stop,
                progress,
            )))
        }
        "playerShouldQueueScrobblePause" => {
            let args = object(args_json)?;
            let token = args.get("token").and_then(Value::as_str);
            let was_play_when_ready = field(&args, "wasPlayWhenReady")?
                .as_bool()
                .ok_or_else(|| fail(ErrorKind::InvalidArgs, "wasPlayWhenReady must be bool"))?;
            let has_scrobbled_start = field(&args, "hasScrobbledStart")?
                .as_bool()
                .ok_or_else(|| fail(ErrorKind::InvalidArgs, "hasScrobbledStart must be bool"))?;
            let has_scrobbled_stop = field(&args, "hasScrobbledStop")?
                .as_bool()
                .ok_or_else(|| fail(ErrorKind::InvalidArgs, "hasScrobbledStop must be bool"))?;
            Ok(json!(player_scrobble::should_queue_pause(
                token,
                was_play_when_ready,
                has_scrobbled_start,
                has_scrobbled_stop,
            )))
        }
        "playerShouldEnqueueDurableScrobble" => {
            let args = object(args_json)?;
            let token = args.get("token").and_then(Value::as_str);
            let progress = field(&args, "progress")?
                .as_f64()
                .ok_or_else(|| fail(ErrorKind::InvalidArgs, "progress must be a number"))?
                as f32;
            Ok(json!(player_scrobble::should_enqueue_durable(
                field_str(&args, "action")?,
                token,
                progress,
            )))
        }
        "playerShouldSavePeriodicProgress" => {
            let args = object(args_json)?;
            let is_playing = field(&args, "isPlaying")?
                .as_bool()
                .ok_or_else(|| fail(ErrorKind::InvalidArgs, "isPlaying must be bool"))?;
            let now_ms = field(&args, "nowMs")?
                .as_i64()
                .ok_or_else(|| fail(ErrorKind::InvalidArgs, "nowMs must be a number"))?;
            let last_saved_at_ms = field(&args, "lastSavedAtMs")?
                .as_i64()
                .ok_or_else(|| fail(ErrorKind::InvalidArgs, "lastSavedAtMs must be a number"))?;
            Ok(json!(player_scrobble::should_save_periodic_progress(
                is_playing,
                now_ms,
                last_saved_at_ms,
            )))
        }
        "playerShouldSaveOnDispose" => {
            let args = object(args_json)?;
            let position_ms = field(&args, "positionMs")?
                .as_i64()
                .ok_or_else(|| fail(ErrorKind::InvalidArgs, "positionMs must be a number"))?;
            Ok(json!(player_scrobble::should_save_on_dispose(position_ms)))
        }

        _ => Err(unknown_method()),
    }
}
