use crate::player::playback::scrobble;
use serde::{Deserialize, Serialize};

#[derive(Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
struct TickInput {
    paused: bool,
    has_frame: bool,
    position_ms: i64,
    duration_ms: i64,
    now_ms: i64,
    last_saved_at_ms: i64,
    last_scrobble: Option<String>,
    ended: bool,
    speed: f64,
    last_paused: Option<bool>,
    last_position_ms: Option<i64>,
    elapsed_ms: i64,
    watched_at_percent: Option<serde_json::Value>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct TickPlan {
    scrobble: Option<&'static str>,
    save_progress: bool,
    progress_percent: f32,
}

const SEEK_JUMP_SECONDS: f64 = 3.0;
const EVENT_SAVE_GAP_MS: i64 = 2000;

impl TickInput {
    fn seeked(&self) -> bool {
        let Some(last) = self.last_position_ms else {
            return false;
        };
        let drift = (self.position_ms - last) as f64 / 1000.0;
        if self.last_paused == Some(true) {
            return drift.abs() > SEEK_JUMP_SECONDS / 2.0;
        }
        let speed = if self.speed > 0.0 { self.speed } else { 1.0 };
        let expected = self.elapsed_ms as f64 / 1000.0 * speed;
        (drift - expected).abs() > SEEK_JUMP_SECONDS
    }

    fn toggled(&self) -> bool {
        self.last_paused.is_some_and(|paused| paused != self.paused)
    }
}

pub(crate) fn player_tick_plan_json(input: &str) -> Option<String> {
    let input: TickInput = serde_json::from_str(input).ok()?;
    let percent = scrobble::progress_percent(input.position_ms, input.duration_ms);
    let threshold = scrobble::watched_percent(input.watched_at_percent.as_ref());
    let wanted = if !input.has_frame {
        None
    } else if input.ended
        || (input.paused && scrobble::should_mark_stopped(false, percent, threshold))
    {
        Some("stop")
    } else if input.paused {
        Some("pause")
    } else {
        Some("start")
    };
    let scrobble_action = wanted.filter(|action| input.last_scrobble.as_deref() != Some(*action));
    let playing = input.has_frame && !input.paused;
    let on_event = input.has_frame
        && (input.toggled() || input.seeked())
        && input.now_ms - input.last_saved_at_ms > EVENT_SAVE_GAP_MS;
    let save_progress = input.duration_ms > 0
        && scrobble::should_save_on_dispose(input.position_ms)
        && (scrobble::should_save_periodic_progress(playing, input.now_ms, input.last_saved_at_ms)
            || on_event
            || (scrobble_action == Some("pause")
                && scrobble::should_save_event_progress(input.now_ms, input.last_saved_at_ms)));
    serde_json::to_string(&TickPlan {
        scrobble: scrobble_action,
        save_progress,
        progress_percent: percent,
    })
    .ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};

    #[test]
    fn tick_scrobbles_each_state_change_once() {
        let tick = |input: Value| -> Value {
            serde_json::from_str(&player_tick_plan_json(&input.to_string()).unwrap()).unwrap()
        };
        let started = tick(json!({"hasFrame": true, "positionMs": 5000, "durationMs": 100000}));
        assert_eq!(started["scrobble"], "start");
        let repeat = tick(json!({
            "hasFrame": true, "positionMs": 6000, "durationMs": 100000, "lastScrobble": "start"
        }));
        assert!(repeat["scrobble"].is_null());
        let paused = tick(json!({
            "hasFrame": true, "paused": true, "positionMs": 6000,
            "durationMs": 100000, "lastScrobble": "start"
        }));
        assert_eq!(paused["scrobble"], "pause");
    }

    #[test]
    fn stop_fires_at_the_configured_watched_percent() {
        let tick = |threshold: Value| -> Value {
            serde_json::from_str(
                &player_tick_plan_json(
                    &json!({
                        "hasFrame": true, "paused": true, "positionMs": 70000,
                        "durationMs": 100000, "lastScrobble": "start",
                        "watchedAtPercent": threshold
                    })
                    .to_string(),
                )
                .unwrap(),
            )
            .unwrap()
        };
        assert_eq!(tick(json!("70"))["scrobble"], "stop");
        assert_eq!(tick(json!("90"))["scrobble"], "pause");
        assert_eq!(tick(Value::Null)["scrobble"], "pause");
    }

    #[test]
    fn seek_and_pause_toggle_save_progress_once_per_gap() {
        let tick = |input: Value| -> Value {
            serde_json::from_str(&player_tick_plan_json(&input.to_string()).unwrap()).unwrap()
        };
        let base = json!({
            "hasFrame": true, "durationMs": 100000, "nowMs": 100000, "lastSavedAtMs": 90000,
            "lastScrobble": "start", "speed": 1.0, "lastPaused": false,
            "lastPositionMs": 20000, "elapsedMs": 250
        });
        let with = |extra: Value| {
            let mut input = base.clone();
            input
                .as_object_mut()
                .unwrap()
                .extend(extra.as_object().unwrap().clone());
            input
        };
        assert_eq!(
            tick(with(json!({"positionMs": 20250})))["saveProgress"],
            false
        );
        assert_eq!(
            tick(with(json!({"positionMs": 60000})))["saveProgress"],
            true
        );
        assert_eq!(
            tick(with(json!({"positionMs": 60000, "lastSavedAtMs": 99000})))["saveProgress"],
            false
        );
    }
}
