use super::*;
use crate::presence::Presence;
use serde_json::json;
use std::collections::BTreeMap;
use std::sync::{LazyLock, Mutex};

static LIBRARY: LazyLock<Mutex<(u64, Value)>> =
    LazyLock::new(|| Mutex::new((u64::MAX, Value::Null)));

fn library(state: &RendererState) -> Value {
    let Ok(mut cached) = LIBRARY.lock() else {
        return Value::Null;
    };
    if cached.0 != state.core_snapshot_revision {
        let watched: Vec<&str> = state
            .core_snapshot
            .as_ref()
            .and_then(|snapshot| snapshot.pointer("/library/watched"))
            .and_then(Value::as_object)
            .map(|map| {
                map.iter()
                    .filter(|(_, value)| value.as_bool() == Some(true))
                    .map(|(id, _)| id.as_str())
                    .collect()
            })
            .unwrap_or_default();
        *cached = (state.core_snapshot_revision, json!({"watched": watched}));
    }
    cached.1.clone()
}

fn public_settings(state: &RendererState) -> Value {
    let mut values = state.settings.values.clone();
    if let Some(map) = values.as_object_mut() {
        map.retain(|key, _| {
            let key = key.to_ascii_lowercase();
            !["token", "secret", "password", "apikey", "key"]
                .iter()
                .any(|word| key.contains(word))
        });
    }
    values
}

pub(crate) fn sync(state: &RendererState) {
    let player = state
        .player
        .as_ref()
        .and_then(|player| match player.presence() {
            Presence::Playing {
                title,
                paused,
                position,
                duration,
                ..
            } => Some(json!({
                "title": title,
                "paused": paused,
                "position": position,
                "duration": duration,
                "volume": player.status.volume,
            })),
            _ => None,
        })
        .unwrap_or(Value::Null);
    let profile = state
        .core_snapshot
        .as_ref()
        .and_then(|snapshot| snapshot.pointer("/profile/active"))
        .map_or(
            Value::Null,
            |active| json!({"id": active.get("id"), "name": active.get("name")}),
        );
    skins::script_sync(json!({
        "player": player,
        "profile": profile,
        "library": library(state),
        "settings": public_settings(state),
        "navigation": {"route": state.route.as_str()},
    }));
}

pub(crate) fn apply(state: &mut RendererState) {
    for request in skins::script_requests() {
        let number = |key: &str| request.args.get(key).and_then(Value::as_f64);
        let paused = state.player.as_ref().map(|player| player.status.paused);
        match request.name.as_str() {
            "player.volume" => {
                if let Some(value) = number("value") {
                    player::set_volume(state, value);
                }
            }
            "player.seek" => match (number("to"), number("seconds")) {
                (Some(to), _) => player::command(state, VideoCommand::SeekTo(to)),
                (None, Some(by)) => player::command(state, VideoCommand::Seek(by)),
                _ => {}
            },
            "player.pause" if paused == Some(false) => {
                player::command(state, VideoCommand::TogglePause)
            }
            "player.play" if paused == Some(true) => {
                player::command(state, VideoCommand::TogglePause)
            }
            "player.toggle" if paused.is_some() => {
                player::command(state, VideoCommand::TogglePause)
            }
            "page" => {
                if let Some(name) = request.args.get("name").and_then(Value::as_str) {
                    skins::open_page(name);
                }
            }
            name => {
                let args: BTreeMap<String, Value> = request
                    .args
                    .as_object()
                    .map(|map| map.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
                    .unwrap_or_default();
                let action = fluxa_skin::ResolvedAction {
                    name: name.to_owned(),
                    args,
                };
                if let Some(action) = skins::native_action(&action) {
                    state.pending_native_actions.push(action);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sync_shares_route_and_drops_secret_settings() {
        let _serial = skins::SCRIPT_SERIAL
            .lock()
            .unwrap_or_else(|held| held.into_inner());
        let host = FluxaHost::new(1.0, None);
        host.peek(|state| {
            state.settings.values = json!({"theme": "dark", "traktApiKey": "x", "authToken": "y"});
            state.route = Route::Library;
            sync(state);
        });
        let app = skins::script_app();
        assert_eq!(app["navigation"]["route"], "library");
        assert_eq!(app["settings"]["theme"], "dark");
        assert!(app["settings"].get("traktApiKey").is_none());
        assert!(app["settings"].get("authToken").is_none());
        assert_eq!(app["player"], Value::Null);
    }

    #[test]
    fn requests_become_native_actions_and_unknown_ones_are_dropped() {
        let _serial = skins::SCRIPT_SERIAL
            .lock()
            .unwrap_or_else(|held| held.into_inner());
        let host = FluxaHost::new(1.0, None);
        skins::script_requests();
        for (name, args) in [
            ("navigate", json!({"screen": "library"})),
            ("search", json!({"query": "dune"})),
            ("bogus", json!({})),
            ("player.pause", json!({})),
            ("player.volume", json!({"value": 10})),
        ] {
            skins::script_enqueue(skins::ScriptRequest {
                name: name.to_owned(),
                args,
            });
        }
        host.peek(|state| {
            apply(state);
            assert_eq!(state.pending_native_actions.len(), 2);
            assert!(matches!(
                state.pending_native_actions[0],
                NativeAction::Navigate { .. }
            ));
            assert!(matches!(
                state.pending_native_actions[1],
                NativeAction::Search { .. }
            ));
        });
    }
}
