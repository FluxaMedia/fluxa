use serde::Deserialize;
use serde_json::{Map, Value, json};

struct Action {
    id: &'static str,
    player: bool,
    defaults: &'static [&'static str],
}

const ACTIONS: &[Action] = &[
    Action {
        id: "nav_home",
        player: false,
        defaults: &["1"],
    },
    Action {
        id: "nav_library",
        player: false,
        defaults: &["2"],
    },
    Action {
        id: "nav_discover",
        player: false,
        defaults: &["3"],
    },
    Action {
        id: "nav_calendar",
        player: false,
        defaults: &["4"],
    },
    Action {
        id: "nav_settings",
        player: false,
        defaults: &["5", "ctrl+comma", "meta+comma"],
    },
    Action {
        id: "focus_search",
        player: false,
        defaults: &["ctrl+f", "/"],
    },
    Action {
        id: "go_back",
        player: false,
        defaults: &["alt+left"],
    },
    Action {
        id: "toggle_window_fullscreen",
        player: false,
        defaults: &["f11"],
    },
    Action {
        id: "player_play_pause",
        player: true,
        defaults: &["k", "space"],
    },
    Action {
        id: "player_seek_back",
        player: true,
        defaults: &["left", "j"],
    },
    Action {
        id: "player_seek_forward",
        player: true,
        defaults: &["right", "l"],
    },
    Action {
        id: "player_seek_big_back",
        player: true,
        defaults: &["shift+left"],
    },
    Action {
        id: "player_seek_big_forward",
        player: true,
        defaults: &["shift+right"],
    },
    Action {
        id: "player_seek_start",
        player: true,
        defaults: &["home"],
    },
    Action {
        id: "player_seek_end",
        player: true,
        defaults: &["end"],
    },
    Action {
        id: "player_seek_percent_0",
        player: true,
        defaults: &["0"],
    },
    Action {
        id: "player_seek_percent_1",
        player: true,
        defaults: &["1"],
    },
    Action {
        id: "player_seek_percent_2",
        player: true,
        defaults: &["2"],
    },
    Action {
        id: "player_seek_percent_3",
        player: true,
        defaults: &["3"],
    },
    Action {
        id: "player_seek_percent_4",
        player: true,
        defaults: &["4"],
    },
    Action {
        id: "player_seek_percent_5",
        player: true,
        defaults: &["5"],
    },
    Action {
        id: "player_seek_percent_6",
        player: true,
        defaults: &["6"],
    },
    Action {
        id: "player_seek_percent_7",
        player: true,
        defaults: &["7"],
    },
    Action {
        id: "player_seek_percent_8",
        player: true,
        defaults: &["8"],
    },
    Action {
        id: "player_seek_percent_9",
        player: true,
        defaults: &["9"],
    },
    Action {
        id: "player_frame_step",
        player: true,
        defaults: &["."],
    },
    Action {
        id: "player_frame_back",
        player: true,
        defaults: &["comma"],
    },
    Action {
        id: "player_cycle_audio",
        player: true,
        defaults: &["a"],
    },
    Action {
        id: "player_cycle_subtitles",
        player: true,
        defaults: &["c"],
    },
    Action {
        id: "player_subtitle_delay_decrease",
        player: true,
        defaults: &["z"],
    },
    Action {
        id: "player_subtitle_delay_increase",
        player: true,
        defaults: &["x"],
    },
    Action {
        id: "player_volume_up",
        player: true,
        defaults: &["up"],
    },
    Action {
        id: "player_volume_down",
        player: true,
        defaults: &["down"],
    },
    Action {
        id: "player_mute",
        player: true,
        defaults: &["m"],
    },
    Action {
        id: "player_speed_decrease",
        player: true,
        defaults: &["["],
    },
    Action {
        id: "player_speed_increase",
        player: true,
        defaults: &["]"],
    },
    Action {
        id: "player_skip_active",
        player: true,
        defaults: &["s"],
    },
    Action {
        id: "player_next_episode",
        player: true,
        defaults: &["n"],
    },
    Action {
        id: "player_fullscreen",
        player: true,
        defaults: &["f"],
    },
    Action {
        id: "player_anime4k_toggle",
        player: true,
        defaults: &["u"],
    },
    Action {
        id: "player_anime4k_mode_next",
        player: true,
        defaults: &["shift+u"],
    },
    Action {
        id: "player_anime4k_mode_prev",
        player: true,
        defaults: &[],
    },
    Action {
        id: "player_boost_toggle",
        player: true,
        defaults: &["b"],
    },
    Action {
        id: "player_previous_episode",
        player: true,
        defaults: &["p"],
    },
    Action {
        id: "player_mark_segment",
        player: true,
        defaults: &["shift+i"],
    },
    Action {
        id: "player_toggle_stats",
        player: true,
        defaults: &["i"],
    },
];

const NAMED_KEYS: &[&str] = &[
    "left",
    "right",
    "up",
    "down",
    "space",
    "enter",
    "escape",
    "tab",
    "backspace",
    "delete",
    "home",
    "end",
    "pageup",
    "pagedown",
    "insert",
    "comma",
    "plus",
    "minus",
];

const ALWAYS_RESERVED: &[&str] = &["escape", "tab", "shift+tab", "enter", "backspace"];
const BARE_NAVIGATION: &[&str] = &["left", "right", "up", "down", "space"];

#[derive(Default)]
struct Chord {
    ctrl: bool,
    alt: bool,
    shift: bool,
    meta: bool,
    key: String,
}

impl Chord {
    fn parse(raw: &str) -> Option<Self> {
        let mut chord = Self::default();
        let lowered = raw.trim().to_lowercase();
        let (modifiers, key) = match lowered.rsplit_once('+') {
            Some((head, "")) => (head.strip_suffix('+').unwrap_or(head), "plus"),
            Some((head, tail)) => (head, tail),
            None => ("", lowered.as_str()),
        };
        for part in modifiers.split('+').filter(|part| !part.is_empty()) {
            match part {
                "ctrl" | "control" => chord.ctrl = true,
                "alt" | "option" => chord.alt = true,
                "shift" => chord.shift = true,
                "meta" | "cmd" | "super" | "win" => chord.meta = true,
                _ => return None,
            }
        }
        let is_function = key
            .strip_prefix('f')
            .and_then(|number| number.parse::<u8>().ok())
            .is_some_and(|number| (1..=24).contains(&number));
        let is_char = key.chars().count() == 1 && key != "+";
        if !(is_function || is_char || NAMED_KEYS.contains(&key)) {
            return None;
        }
        chord.key = key.to_owned();
        Some(chord)
    }

    fn has_command_modifier(&self) -> bool {
        self.ctrl || self.alt || self.meta
    }

    fn is_function(&self) -> bool {
        self.key.len() > 1 && self.key.starts_with('f') && self.key[1..].parse::<u8>().is_ok()
    }

    fn id(&self) -> String {
        let mut parts = Vec::new();
        for (on, name) in [
            (self.ctrl, "ctrl"),
            (self.alt, "alt"),
            (self.shift, "shift"),
            (self.meta, "meta"),
        ] {
            if on {
                parts.push(name);
            }
        }
        parts.push(&self.key);
        parts.join("+")
    }

    fn label(&self) -> String {
        let mut parts = Vec::new();
        for (on, name) in [
            (self.ctrl, "Ctrl"),
            (self.alt, "Alt"),
            (self.shift, "Shift"),
            (self.meta, "Meta"),
        ] {
            if on {
                parts.push(name.to_owned());
            }
        }
        parts.push(match self.key.as_str() {
            "left" => "←".to_owned(),
            "right" => "→".to_owned(),
            "up" => "↑".to_owned(),
            "down" => "↓".to_owned(),
            "comma" => ",".to_owned(),
            "plus" => "+".to_owned(),
            "minus" => "-".to_owned(),
            "escape" => "Esc".to_owned(),
            "pageup" => "Page Up".to_owned(),
            "pagedown" => "Page Down".to_owned(),
            key => {
                let mut chars = key.chars();
                chars.next().map_or_else(String::new, |first| {
                    first.to_uppercase().chain(chars).collect()
                })
            }
        });
        parts.join(" + ")
    }
}

fn normalize(raw: &str) -> Option<String> {
    Chord::parse(raw).map(|chord| chord.id())
}

fn effective(overrides: &Value, action: &Action) -> Vec<String> {
    match overrides.get(action.id).and_then(Value::as_array) {
        Some(list) => list
            .iter()
            .filter_map(|chord| normalize(chord.as_str()?))
            .collect(),
        None => action
            .defaults
            .iter()
            .map(|chord| (*chord).to_owned())
            .collect(),
    }
}

fn overrides_object(overrides: &Value) -> Map<String, Value> {
    overrides.as_object().cloned().unwrap_or_default()
}

pub(crate) fn bindings_json(args_json: &str) -> Option<String> {
    #[derive(Deserialize, Default)]
    #[serde(default)]
    struct Args {
        overrides: Value,
    }
    let args: Args = serde_json::from_str(args_json).ok()?;
    let actions = ACTIONS
        .iter()
        .map(|action| {
            let chords = effective(&args.overrides, action);
            let label = chords
                .iter()
                .filter_map(|chord| Chord::parse(chord))
                .map(|chord| chord.label())
                .collect::<Vec<_>>()
                .join(" / ");
            json!({
                "id": action.id,
                "scope": if action.player { "player" } else { "app" },
                "chords": chords,
                "label": label,
                "custom": args.overrides.get(action.id).is_some(),
            })
        })
        .collect::<Vec<_>>();
    Some(json!({ "actions": actions }).to_string())
}

pub(crate) fn resolve_json(args_json: &str) -> Option<String> {
    #[derive(Deserialize, Default)]
    #[serde(default)]
    struct Args {
        chord: String,
        overrides: Value,
        player: bool,
        typing: bool,
    }
    let args: Args = serde_json::from_str(args_json).ok()?;
    let chord = Chord::parse(&args.chord)?;
    if args.typing && !chord.has_command_modifier() && !chord.is_function() {
        return Some(json!({ "action": null }).to_string());
    }
    let id = chord.id();
    let action = ACTIONS
        .iter()
        .filter(|action| {
            if args.player {
                action.player || matches!(action.id, "go_back" | "toggle_window_fullscreen")
            } else {
                !action.player
            }
        })
        .find(|action| effective(&args.overrides, action).contains(&id))
        .map(|action| action.id);
    Some(json!({ "action": action }).to_string())
}

pub(crate) fn assign_json(args_json: &str) -> Option<String> {
    #[derive(Deserialize, Default)]
    #[serde(default)]
    struct Args {
        overrides: Value,
        action: String,
        chord: Option<String>,
        reset: bool,
    }
    let args: Args = serde_json::from_str(args_json).ok()?;
    let target = ACTIONS.iter().find(|action| action.id == args.action)?;
    let mut overrides = overrides_object(&args.overrides);
    if args.reset {
        overrides.remove(target.id);
        return Some(json!({ "ok": true, "overrides": overrides, "displaced": [] }).to_string());
    }
    let Some(raw) = args.chord else {
        overrides.insert(target.id.to_owned(), json!([]));
        return Some(json!({ "ok": true, "overrides": overrides, "displaced": [] }).to_string());
    };
    let Some(chord) = Chord::parse(&raw) else {
        return Some(json!({ "ok": false, "error": "invalid" }).to_string());
    };
    let id = chord.id();
    let reserved = ALWAYS_RESERVED.contains(&id.as_str())
        || (!target.player && BARE_NAVIGATION.contains(&id.as_str()));
    if reserved {
        return Some(json!({ "ok": false, "error": "reserved" }).to_string());
    }
    let mut displaced = Vec::new();
    for other in ACTIONS.iter().filter(|other| other.id != target.id) {
        let chords = effective(&args.overrides, other);
        if chords.contains(&id) {
            let kept = chords
                .into_iter()
                .filter(|chord| *chord != id)
                .collect::<Vec<_>>();
            overrides.insert(other.id.to_owned(), json!(kept));
            displaced.push(other.id);
        }
    }
    overrides.insert(target.id.to_owned(), json!([id]));
    Some(json!({ "ok": true, "overrides": overrides, "displaced": displaced }).to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn call(function: fn(&str) -> Option<String>, args: Value) -> Value {
        serde_json::from_str(&function(&args.to_string()).unwrap()).unwrap()
    }

    #[test]
    fn modifier_order_and_case_are_normalized() {
        assert_eq!(normalize("Shift+CTRL+K").as_deref(), Some("ctrl+shift+k"));
        assert_eq!(normalize("ctrl++").as_deref(), Some("ctrl+plus"));
        assert!(normalize("hyper+k").is_none());
    }

    #[test]
    fn typing_blocks_bare_keys_but_not_commands() {
        let resolve = |chord: &str| {
            call(resolve_json, json!({ "chord": chord, "typing": true }))["action"].clone()
        };
        assert_eq!(resolve("1"), Value::Null);
        assert_eq!(resolve("ctrl+f"), json!("focus_search"));
        assert_eq!(resolve("f11"), json!("toggle_window_fullscreen"));
    }

    #[test]
    fn player_actions_only_resolve_while_playing() {
        let resolve = |player: bool| {
            call(resolve_json, json!({ "chord": "k", "player": player }))["action"].clone()
        };
        assert_eq!(resolve(true), json!("player_play_pause"));
        assert_eq!(resolve(false), Value::Null);
    }

    #[test]
    fn assigning_a_taken_chord_displaces_the_old_holder() {
        let out = call(
            assign_json,
            json!({ "action": "nav_library", "chord": "m", "overrides": {} }),
        );
        assert_eq!(out["displaced"], json!(["player_mute"]));
        assert_eq!(out["overrides"]["player_mute"], json!([]));
        assert_eq!(out["overrides"]["nav_library"], json!(["m"]));
    }

    #[test]
    fn navigation_keys_cannot_be_bound_to_app_actions() {
        let out = call(
            assign_json,
            json!({ "action": "nav_home", "chord": "escape", "overrides": {} }),
        );
        assert_eq!(out["error"], "reserved");
        let out = call(
            assign_json,
            json!({ "action": "nav_home", "chord": "left", "overrides": {} }),
        );
        assert_eq!(out["error"], "reserved");
        let out = call(
            assign_json,
            json!({ "action": "player_mute", "chord": "left", "overrides": {} }),
        );
        assert_eq!(out["ok"], true);
    }

    #[test]
    fn reset_restores_defaults() {
        let out = call(
            assign_json,
            json!({ "action": "player_mute", "reset": true, "overrides": {"player_mute": ["x"]} }),
        );
        let bindings = call(bindings_json, json!({ "overrides": out["overrides"] }));
        let mute = bindings["actions"]
            .as_array()
            .unwrap()
            .iter()
            .find(|action| action["id"] == "player_mute")
            .unwrap();
        assert_eq!(mute["chords"], json!(["m"]));
        assert_eq!(mute["custom"], false);
    }
}
