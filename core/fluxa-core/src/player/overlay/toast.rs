use serde::Deserialize;
use serde_json::{Value, json};

const HOLD_MS: u64 = 1200;
const LEVEL_HOLD_MS: u64 = 900;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Input {
    kind: String,
    #[serde(default)]
    value: f64,
    #[serde(default)]
    label: Option<String>,
}

fn trim(value: f64) -> String {
    if value.fract().abs() < 0.005 {
        format!("{}", value.round() as i64)
    } else {
        format!("{value:.2}").trim_end_matches('0').to_owned()
    }
}

pub(crate) fn player_toast_plan_json(args_json: &str) -> Option<String> {
    let input: Input = serde_json::from_str(args_json).ok()?;
    let plan = match input.kind.as_str() {
        "seek" if input.value != 0.0 => {
            let key = if input.value > 0.0 {
                "player.toast.seek_forward"
            } else {
                "player.toast.seek_back"
            };
            json!({"key": key, "value": trim(input.value.abs()), "holdMs": HOLD_MS})
        }
        "speed" => {
            json!({"key": "player.toast.speed", "value": trim(input.value), "holdMs": HOLD_MS})
        }
        "volume" => {
            let percent = input.value.clamp(0.0, 100.0);
            json!({
                "key": "player.toast.volume",
                "value": trim(percent.round()),
                "level": percent / 100.0,
                "holdMs": LEVEL_HOLD_MS,
            })
        }
        "brightness" => {
            let percent = (input.value * 100.0).clamp(0.0, 100.0);
            json!({
                "key": "player.toast.brightness",
                "value": trim(percent.round()),
                "level": percent / 100.0,
                "holdMs": LEVEL_HOLD_MS,
            })
        }
        "muted" => {
            let key = if input.value > 0.0 {
                "player.toast.muted"
            } else {
                "player.toast.unmuted"
            };
            json!({"key": key, "holdMs": HOLD_MS})
        }
        "audio" => json!({"key": "player.toast.audio", "value": input.label?, "holdMs": HOLD_MS}),
        "subtitle" => match input.label {
            Some(label) => {
                json!({"key": "player.toast.subtitle", "value": label, "holdMs": HOLD_MS})
            }
            None => json!({"key": "player.toast.subtitle_off", "holdMs": HOLD_MS}),
        },
        _ => return None,
    };
    Some(Value::to_string(&plan))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn plan(input: Value) -> Value {
        serde_json::from_str(&player_toast_plan_json(&input.to_string()).unwrap()).unwrap()
    }

    #[test]
    fn seek_direction_picks_the_key_and_drops_the_sign() {
        let forward = plan(json!({"kind": "seek", "value": 10.0}));
        assert_eq!(forward["key"], "player.toast.seek_forward");
        assert_eq!(forward["value"], "10");
        let back = plan(json!({"kind": "seek", "value": -30.0}));
        assert_eq!(back["key"], "player.toast.seek_back");
        assert_eq!(back["value"], "30");
    }

    #[test]
    fn speed_keeps_fractions() {
        assert_eq!(
            plan(json!({"kind": "speed", "value": 1.25}))["value"],
            "1.25"
        );
        assert_eq!(plan(json!({"kind": "speed", "value": 2.0}))["value"], "2");
    }

    #[test]
    fn levels_are_clamped_and_carry_a_fraction() {
        let loud = plan(json!({"kind": "volume", "value": 140.0}));
        assert_eq!(loud["level"], 1.0);
        assert_eq!(loud["value"], "100");
        assert_eq!(
            plan(json!({"kind": "brightness", "value": 0.5}))["value"],
            "50"
        );
    }

    #[test]
    fn unknown_kinds_and_zero_seeks_have_no_toast() {
        assert!(player_toast_plan_json(r#"{"kind":"seek","value":0}"#).is_none());
        assert!(player_toast_plan_json(r#"{"kind":"nope"}"#).is_none());
    }

    #[test]
    fn missing_subtitle_label_means_off() {
        assert_eq!(
            plan(json!({"kind": "subtitle"}))["key"],
            "player.toast.subtitle_off"
        );
    }
}
