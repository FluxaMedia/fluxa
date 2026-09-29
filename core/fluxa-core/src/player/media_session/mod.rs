use serde::Deserialize;
use serde_json::{Value, json};

const SEEK_STEP_SECONDS: f64 = 10.0;
const HEADSET_TAPS: [&str; 3] = ["toggle", "next", "previous"];

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", default)]
struct Session {
    title: String,
    subtitle: Option<String>,
    poster: Option<String>,
    paused: bool,
    buffering: bool,
    position: f64,
    duration: f64,
    speed: f64,
    has_next: bool,
}

impl Default for Session {
    fn default() -> Self {
        Self {
            title: String::new(),
            subtitle: None,
            poster: None,
            paused: false,
            buffering: false,
            position: 0.0,
            duration: 0.0,
            speed: 1.0,
            has_next: false,
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", default)]
struct Command {
    command: String,
    value: f64,
    paused: bool,
    position: f64,
    duration: f64,
    has_next: bool,
}

impl Default for Command {
    fn default() -> Self {
        Self {
            command: String::new(),
            value: 0.0,
            paused: false,
            position: 0.0,
            duration: 0.0,
            has_next: false,
        }
    }
}

fn millis(seconds: f64) -> i64 {
    (seconds.max(0.0) * 1000.0).round() as i64
}

pub(crate) fn player_media_session_plan_json(args_json: &str) -> Option<String> {
    let session: Session = serde_json::from_str(args_json).ok()?;
    let state = if session.buffering {
        "buffering"
    } else if session.paused {
        "paused"
    } else {
        "playing"
    };
    let mut actions = vec!["play", "pause", "stop", "seekForward", "seekBackward"];
    if session.duration > 0.0 {
        actions.push("seekTo");
    }
    actions.push("previous");
    if session.has_next {
        actions.push("next");
    }
    let plan = json!({
        "state": state,
        "title": session.title,
        "subtitle": session.subtitle,
        "poster": session.poster,
        "positionMs": millis(session.position),
        "durationMs": millis(session.duration),
        "speed": if session.speed > 0.0 { session.speed } else { 1.0 },
        "actions": actions,
    });
    Some(Value::to_string(&plan))
}

fn act(action: &str, value: Option<f64>) -> Value {
    match value {
        Some(value) => json!({"action": action, "value": value}),
        None => json!({"action": action}),
    }
}

pub(crate) fn player_media_command_plan_json(args_json: &str) -> Option<String> {
    let input: Command = serde_json::from_str(args_json).ok()?;
    let command = match input.command.as_str() {
        "headset" => *HEADSET_TAPS.get((input.value as usize).checked_sub(1)?)?,
        other => other,
    };
    let plan = match command {
        "play" if input.paused => act("togglePause", None),
        "pause" if !input.paused => act("togglePause", None),
        "play" | "pause" => act("none", None),
        "toggle" => act("togglePause", None),
        "stop" => act("close", None),
        "next" if input.has_next => act("next", None),
        "next" => act("none", None),
        "previous" => act("seekTo", Some(0.0)),
        "seekForward" | "fastForward" => act("seekBy", Some(seek_step(input.value))),
        "seekBackward" | "rewind" => act("seekBy", Some(-seek_step(input.value))),
        "seekTo" => act(
            "seekTo",
            Some(input.value.clamp(0.0, input.duration.max(0.0))),
        ),
        "seekToPosition" => act(
            "seekTo",
            Some((input.position + input.value).clamp(0.0, input.duration.max(0.0))),
        ),
        _ => return None,
    };
    Some(Value::to_string(&plan))
}

fn seek_step(requested: f64) -> f64 {
    if requested > 0.0 {
        requested
    } else {
        SEEK_STEP_SECONDS
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn command(input: Value) -> Value {
        serde_json::from_str(&player_media_command_plan_json(&input.to_string()).unwrap()).unwrap()
    }

    #[test]
    fn play_and_pause_only_toggle_when_state_differs() {
        assert_eq!(
            command(json!({"command": "play", "paused": true}))["action"],
            "togglePause"
        );
        assert_eq!(
            command(json!({"command": "play", "paused": false}))["action"],
            "none"
        );
        assert_eq!(
            command(json!({"command": "pause", "paused": false}))["action"],
            "togglePause"
        );
    }

    #[test]
    fn headset_taps_map_to_toggle_next_previous() {
        assert_eq!(
            command(json!({"command": "headset", "value": 1}))["action"],
            "togglePause"
        );
        assert_eq!(
            command(json!({"command": "headset", "value": 2, "hasNext": true}))["action"],
            "next"
        );
        assert_eq!(
            command(json!({"command": "headset", "value": 3}))["action"],
            "seekTo"
        );
        assert!(player_media_command_plan_json(r#"{"command":"headset","value":4}"#).is_none());
    }

    #[test]
    fn next_without_a_following_episode_does_nothing() {
        assert_eq!(command(json!({"command": "next"}))["action"], "none");
    }

    #[test]
    fn absolute_seeks_stay_inside_the_media() {
        let plan = command(json!({"command": "seekTo", "value": 9999.0, "duration": 120.0}));
        assert_eq!(plan["value"], 120.0);
        let plan = command(
            json!({"command": "seekToPosition", "value": -50.0, "position": 20.0, "duration": 120.0}),
        );
        assert_eq!(plan["value"], 0.0);
    }

    #[test]
    fn default_step_applies_to_forward_and_back() {
        assert_eq!(command(json!({"command": "seekForward"}))["value"], 10.0);
        assert_eq!(
            command(json!({"command": "rewind", "value": 30.0}))["value"],
            -30.0
        );
    }

    #[test]
    fn session_plan_reports_state_and_next_action() {
        let plan: Value = serde_json::from_str(
            &player_media_session_plan_json(
                r#"{"title":"Show","paused":true,"position":12.5,"duration":60,"hasNext":true}"#,
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(plan["state"], "paused");
        assert_eq!(plan["positionMs"], 12500);
        assert!(
            plan["actions"]
                .as_array()
                .unwrap()
                .iter()
                .any(|a| a == "next")
        );
        assert_eq!(plan["speed"], 1.0);
    }
}
