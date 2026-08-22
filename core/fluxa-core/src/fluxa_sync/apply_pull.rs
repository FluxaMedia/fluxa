use super::documents::{hash_payload, merged_with};
use serde_json::{Map, Value, json};

fn map_at(local: &Value, field: &str) -> Map<String, Value> {
    local
        .get(field)
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default()
}

fn remove_from_library(library: &mut Map<String, Value>, id: &str) {
    for (_, items) in library.iter_mut() {
        if let Some(list) = items.as_array_mut() {
            list.retain(|item| item.get("id").and_then(Value::as_str) != Some(id));
        }
    }
}

fn library_item_by_id(library: &Map<String, Value>, id: &str) -> Option<Value> {
    library.values().find_map(|items| {
        items.as_array()?.iter().find_map(|item| {
            (item.get("id").and_then(Value::as_str) == Some(id)
                || item.get("_id").and_then(Value::as_str) == Some(id))
                .then(|| item.clone())
        })
    })
}

fn merge_library_item(local: Option<Value>, remote: Value) -> Value {
    let mut merged = local.and_then(|value| value.as_object().cloned()).unwrap_or_default();
    if let Some(remote) = remote.as_object() {
        for (key, value) in remote {
            merged.insert(key.clone(), value.clone());
        }
    }
    Value::Object(merged)
}

fn collections_without(collections: &[Value], id: &str) -> Vec<Value> {
    collections
        .iter()
        .filter(|entry| entry.get("id").and_then(Value::as_str) != Some(id))
        .cloned()
        .collect()
}

fn merge_progress_entry(local: Option<&Value>, remote: &Value) -> Value {
    let mut merged = remote.as_object().cloned().unwrap_or_default();
    if let Some(local) = local.and_then(Value::as_object) {
        // Presentation metadata and stream selection remain device-local. The
        // compact remote payload only owns resume identity and timing.
        for field in [
            "meta",
            "lastEpisodeName",
            "lastEpisodeThumbnail",
            "lastStreamUrl",
            "lastStreamTitle",
            "lastStream",
            "continueWatchingBadge",
            "continueWatchingEpisodeResolved",
        ] {
            if let Some(value) = local.get(field) {
                merged.insert(field.to_string(), value.clone());
            }
        }
    }
    Value::Object(merged)
}

pub(crate) fn apply_pull_json(args_json: &str) -> Option<String> {
    let args: Value = serde_json::from_str(args_json).ok()?;
    let changes = args.get("changes")?.as_array()?;
    let local = args.get("local").cloned().unwrap_or(Value::Null);
    let defaults = args
        .get("settingsDefaults")
        .cloned()
        .unwrap_or(Value::Null);
    let mut known = args
        .get("known")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();

    let mut progress = map_at(&local, "progress");
    let mut watched = map_at(&local, "watched");
    let mut last_watched = map_at(&local, "lastWatched");
    let mut library = map_at(&local, "library");
    let mut collections = local
        .get("collections")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let mut addons = local.get("addons").cloned().unwrap_or(Value::Null);
    let mut plugins = local.get("plugins").cloned().unwrap_or(Value::Null);
    let mut settings = local.get("settings").cloned().unwrap_or(Value::Null);
    let mut profile = local.get("profile").cloned().unwrap_or(Value::Null);

    for document in changes {
        let (Some(entity), Some(key)) = (
            document.get("entity_type").and_then(Value::as_str),
            document.get("key").and_then(Value::as_str),
        ) else {
            continue;
        };
        let deleted = document.get("deleted").and_then(Value::as_bool) == Some(true);
        let payload = document.get("payload").cloned().unwrap_or(Value::Null);
        let revision = document
            .get("revision")
            .and_then(Value::as_i64)
            .unwrap_or_default();
        let name = format!("{entity}|{key}");

        match entity {
            "watch_progress" => {
                if deleted {
                    progress.remove(key);
                } else {
                    let previous = progress.get(key);
                    progress.insert(key.to_string(), merge_progress_entry(previous, &payload));
                }
            }
            "watched_history" => {
                if let Some(video) = key.strip_prefix("video:") {
                    if deleted {
                        watched.remove(video);
                    } else {
                        watched.insert(video.to_string(), json!(true));
                    }
                } else if let Some(series) = key.strip_prefix("series:") {
                    if deleted {
                        last_watched.remove(series);
                    } else {
                        last_watched.insert(series.to_string(), payload.clone());
                    }
                }
            }
            "library" => {
                let local_item = library_item_by_id(&library, key);
                remove_from_library(&mut library, key);
                if !deleted {
                    let status = payload
                        .get("status")
                        .and_then(Value::as_str)
                        .unwrap_or("watchlist")
                        .to_string();
                    let item = merge_library_item(local_item, payload.get("item").cloned().unwrap_or(Value::Null));
                    let slot = library
                        .entry(status)
                        .or_insert_with(|| Value::Array(Vec::new()))
                        .as_array_mut();
                    if let (false, Some(list)) = (item.is_null(), slot) {
                        list.push(item);
                    }
                }
            }
            "collections" => {
                collections = collections_without(&collections, key);
                if !deleted {
                    collections.push(payload.clone());
                }
            }
            "addons" => addons = if deleted { Value::Null } else { payload.clone() },
            "plugins" => plugins = if deleted { Value::Null } else { payload.clone() },
            "settings" if key == "profile" => {
                profile = if deleted { Value::Null } else { payload.clone() };
            }
            "settings" => {
                settings = if deleted {
                    defaults.clone()
                } else {
                    merged_with(&payload, &defaults)
                };
            }
            _ => continue,
        }

        if deleted {
            known.remove(&name);
        } else {
            known.insert(
                name,
                json!({ "revision": revision, "hash": hash_payload(&payload) }),
            );
        }
    }

    serde_json::to_string(&json!({
        "local": {
            "progress": Value::Object(progress),
            "watched": Value::Object(watched),
            "lastWatched": Value::Object(last_watched),
            "library": Value::Object(library),
            "collections": collections,
            "addons": addons,
            "plugins": plugins,
            "settings": settings,
            "profile": profile,
        },
        "known": Value::Object(known),
    }))
    .ok()
}
