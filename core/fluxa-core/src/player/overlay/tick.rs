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
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct TickPlan {
    scrobble: Option<&'static str>,
    save_progress: bool,
    progress_percent: f32,
}

pub(crate) fn player_tick_plan_json(input: &str) -> Option<String> {
    let input: TickInput = serde_json::from_str(input).ok()?;
    let percent = scrobble::progress_percent(input.position_ms, input.duration_ms);
    let wanted = if !input.has_frame {
        None
    } else if input.ended || (input.paused && scrobble::should_mark_stopped(false, percent)) {
        Some("stop")
    } else if input.paused {
        Some("pause")
    } else {
        Some("start")
    };
    let scrobble_action = wanted.filter(|action| input.last_scrobble.as_deref() != Some(*action));
    let playing = input.has_frame && !input.paused;
    let save_progress = input.duration_ms > 0
        && scrobble::should_save_on_dispose(input.position_ms)
        && (scrobble::should_save_periodic_progress(playing, input.now_ms, input.last_saved_at_ms)
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
}
