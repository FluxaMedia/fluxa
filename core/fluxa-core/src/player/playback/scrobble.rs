const SCROBBLE_STOP_PROGRESS_PERCENT: f32 = 80.0;
const PERIODIC_PROGRESS_SAVE_MS: i64 = 30_000;
const DISPOSAL_PROGRESS_SAVE_MIN_MS: i64 = 5_000;

pub(crate) fn progress_percent(position_ms: i64, duration_ms: i64) -> f32 {
    if duration_ms <= 0 {
        return 0.0;
    }
    ((position_ms as f32 / duration_ms as f32) * 100.0).clamp(0.0, 100.0)
}

pub(crate) fn should_mark_stopped(has_scrobbled_stop: bool, progress: f32) -> bool {
    !has_scrobbled_stop && progress >= SCROBBLE_STOP_PROGRESS_PERCENT
}


pub(crate) fn should_save_periodic_progress(
    is_playing: bool,
    now_ms: i64,
    last_saved_at_ms: i64,
) -> bool {
    is_playing && now_ms - last_saved_at_ms > PERIODIC_PROGRESS_SAVE_MS
}

pub(crate) fn should_save_event_progress(now_ms: i64, last_saved_at_ms: i64) -> bool {
    now_ms - last_saved_at_ms >= PERIODIC_PROGRESS_SAVE_MS / 2
}

pub(crate) fn should_save_on_dispose(position_ms: i64) -> bool {
    position_ms > DISPOSAL_PROGRESS_SAVE_MIN_MS
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



}
