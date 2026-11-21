use super::*;
use fluxa_effects::{SkinHost, SkinOutcome};
use fluxa_skin::Extension;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

const MIN_REFRESH: u64 = 30;
const PROGRESS_STEP: f64 = 15.0;
const TRACKER_METHODS: [&str; 8] = [
    "auth",
    "status",
    "disconnect",
    "scrobbleStart",
    "scrobblePause",
    "scrobbleStop",
    "watched",
    "library",
];

static CURRENT: LazyLock<Mutex<String>> = LazyLock::new(Default::default);
static DATA: LazyLock<Mutex<serde_json::Map<String, Value>>> = LazyLock::new(Default::default);
static SCRIPTS: LazyLock<Mutex<Vec<fluxa_skin::Script>>> = LazyLock::new(Default::default);
static APP: LazyLock<Mutex<Value>> = LazyLock::new(|| Mutex::new(json!({})));
static REQUESTS: LazyLock<Mutex<Vec<Request>>> = LazyLock::new(Default::default);
static REPORTED: LazyLock<Mutex<f64>> = LazyLock::new(Default::default);
static TRACKER_RESULTS: LazyLock<Mutex<HashMap<String, Value>>> = LazyLock::new(Default::default);
static VERSION: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Request {
    pub name: String,
    pub args: Value,
}

pub(super) fn version() -> u64 {
    VERSION.load(Ordering::Relaxed)
}

fn bump() {
    VERSION.fetch_add(1, Ordering::Relaxed);
}

pub(super) fn snapshot() -> Value {
    DATA.lock()
        .map_or(Value::Null, |data| Value::Object(data.clone()))
}

fn choices(setting: &fluxa_skin::Setting) -> Vec<Value> {
    if setting.kind == fluxa_skin::SettingKind::Toggle {
        return vec![
            json!({"value": "true", "label": "On"}),
            json!({"value": "false", "label": "Off"}),
        ];
    }
    setting
        .choices
        .iter()
        .map(|choice| json!({"value": choice.value, "label": choice.label}))
        .collect()
}

pub(crate) fn app() -> Value {
    let mut app = APP.lock().map_or(json!({}), |app| app.clone());
    let scripts = SCRIPTS
        .lock()
        .map(|scripts| scripts.clone())
        .unwrap_or_default();
    let mut extensions = serde_json::Map::new();
    let mut trackers = Vec::new();
    for script in &scripts {
        let extension = &script.extension;
        let values = store::settings(extension);
        let settings: Vec<Value> = extension
            .settings
            .iter()
            .map(|setting| {
                json!({
                    "key": setting.key,
                    "title": setting.title,
                    "description": setting.description,
                    "kind": format!("{:?}", setting.kind).to_lowercase(),
                    "value": values[&setting.key],
                    "choices": choices(setting),
                    "min": setting.min,
                    "max": setting.max,
                })
            })
            .collect();
        extensions.insert(
            extension.id.clone(),
            json!({"name": extension.name, "settings": settings}),
        );
        if tracker_script(extension) {
            let results = TRACKER_RESULTS
                .lock()
                .map(|all| {
                    TRACKER_METHODS
                        .iter()
                        .filter_map(|method| {
                            let found = all.get(&format!("{}/{method}", extension.id))?;
                            Some(((*method).to_owned(), found.clone()))
                        })
                        .collect::<serde_json::Map<String, Value>>()
                })
                .unwrap_or_default();
            let results = Value::Object(results);
            let connected = match &results["status"] {
                status if status["ok"] == true => status["value"]["connected"] == true,
                _ => results["auth"]["value"]["connected"] == true,
            };
            for entry in &extension.contributes.trackers {
                trackers.push(json!({
                    "connected": connected,
                    "extension": extension.id,
                    "id": entry.id,
                    "title": entry.title,
                    "results": results,
                }));
            }
        }
    }
    app["extensions"] = Value::Object(extensions);
    app["trackers"] = Value::Array(trackers);
    app
}

pub(super) fn stop() {
    if let Ok(mut running) = CURRENT.lock() {
        running.clear();
    }
    if let Ok(mut data) = DATA.lock() {
        data.clear();
    }
    if let Ok(mut scripts) = SCRIPTS.lock() {
        scripts.clear();
    }
    if let Ok(mut requests) = REQUESTS.lock() {
        requests.clear();
    }
    bump();
}

fn tracker_script(extension: &Extension) -> bool {
    extension.allows("provider.tracker") && !extension.contributes.trackers.is_empty()
}

fn state_for(extension: &Extension) -> Value {
    let app = APP.lock().map_or(json!({}), |app| app.clone());
    let mut state = serde_json::Map::new();
    for slice in ["player", "profile", "library", "settings", "navigation"] {
        if extension.allows(&format!("state.{slice}")) {
            state.insert(
                slice.to_owned(),
                app.get(slice).cloned().unwrap_or(Value::Null),
            );
        }
    }
    Value::Object(state)
}

fn permitted(extension: &Extension, name: &str) -> bool {
    match name {
        "play" | "player.volume" | "player.seek" | "player.pause" | "player.play"
        | "player.toggle" => extension.allows("actions.player"),
        "navigate" | "page" | "back" | "search" | "open_detail" => {
            extension.allows("actions.navigate")
        }
        _ => false,
    }
}

fn commit(extension: &Extension, outcome: &SkinOutcome) {
    if extension.allows("storage")
        && !store::apply(&extension.id, outcome.stored.clone(), &outcome.removed)
    {
        eprintln!("skin script {}: storage limit reached", extension.id);
    }
    let Ok(mut queue) = REQUESTS.lock() else {
        return;
    };
    for request in &outcome.requests {
        if permitted(extension, &request.name) {
            queue.push(Request {
                name: request.name.clone(),
                args: request.args.clone(),
            });
        } else {
            eprintln!(
                "skin script {}: request {} is not permitted",
                extension.id, request.name
            );
        }
    }
}

#[cfg(test)]
pub(crate) static SERIAL: Mutex<()> = Mutex::new(());

#[cfg(test)]
pub(crate) fn enqueue(request: Request) {
    REQUESTS.lock().unwrap().push(request);
}

pub(crate) fn take_requests() -> Vec<Request> {
    REQUESTS
        .lock()
        .map(|mut queue| std::mem::take(&mut *queue))
        .unwrap_or_default()
}

fn run(script: &fluxa_skin::Script, method: &str, args: Vec<Value>) -> Result<SkinOutcome, String> {
    let extension = &script.extension;
    let host = SkinHost {
        settings: store::settings(extension),
        storage: if extension.allows("storage") {
            store::storage(&extension.id)
        } else {
            Default::default()
        },
        state: state_for(extension),
        storage_limit: store::LIMIT,
    };
    let outcome = fluxa_effects::run_skin_script(
        &script.source,
        &extension.id,
        method,
        &extension.network_hosts,
        Vec::new(),
        args,
        &host,
    )?;
    commit(extension, &outcome);
    Ok(outcome)
}

fn script(id: &str) -> Option<fluxa_skin::Script> {
    SCRIPTS
        .lock()
        .ok()?
        .iter()
        .find(|script| script.extension.id == id)
        .cloned()
}

fn refresh(script: &fluxa_skin::Script) {
    if let Ok(outcome) = run(script, "data", Vec::new())
        && let Ok(mut data) = DATA.lock()
    {
        data.insert(script.extension.id.clone(), outcome.value);
        drop(data);
        bump();
    }
}

fn coerce(extension: &Extension, key: &str, value: Value) -> Value {
    use fluxa_skin::SettingKind;
    let Some(setting) = extension.settings.iter().find(|setting| setting.key == key) else {
        return value;
    };
    let Some(text) = value.as_str() else {
        return value;
    };
    match setting.kind {
        SettingKind::Toggle => text.parse::<bool>().map_or(value.clone(), Value::Bool),
        SettingKind::Number => text
            .parse::<f64>()
            .map_or(value.clone(), |number| json!(number)),
        _ => value,
    }
}

fn change_setting(script: &fluxa_skin::Script, args: &Value) {
    let extension = &script.extension;
    let Some(key) = args.get("key").and_then(Value::as_str) else {
        return;
    };
    let value = coerce(
        extension,
        key,
        args.get("value").cloned().unwrap_or_default(),
    );
    if !store::set_setting(extension, key, value.clone()) {
        eprintln!("skin script {}: rejected setting {key}", extension.id);
        return;
    }
    bump();
    let _ = run(script, "onSetting", vec![json!(key), value]);
    refresh(script);
}

pub(crate) fn act(id: &str, method: &str, args: Value) {
    let Some(script) = script(id) else {
        return;
    };
    let method = method.to_owned();
    let _ = std::thread::Builder::new()
        .name(format!("fluxa-skin-action-{id}"))
        .spawn(move || {
            if let Some(op) = method.strip_prefix("tracker.") {
                run_tracker(&script.extension.id, op, args);
                if matches!(op, "auth" | "disconnect") {
                    run_tracker(&script.extension.id, "status", json!({}));
                }
                return;
            }
            if method == "setting" {
                change_setting(&script, &args);
                return;
            }
            match run(&script, &method, vec![args]) {
                Ok(_) => refresh(&script),
                Err(error) => eprintln!("skin script {}: {method}: {error}", script.extension.id),
            }
        });
}

fn run_tracker(id: &str, op: &str, args: Value) {
    let found = match tracker_call(id, op, vec![args]) {
        Ok(value) => json!({"ok": true, "value": value}),
        Err(error) => json!({"ok": false, "error": error}),
    };
    if let Ok(mut all) = TRACKER_RESULTS.lock() {
        all.insert(format!("{id}/{op}"), found);
    }
    bump();
}

pub(crate) fn tracker_rows(language: &str) -> Vec<Value> {
    app()["trackers"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|tracker| {
            let connected = tracker["connected"] == true;
            let failed = tracker["results"]["auth"]["error"].as_str();
            let hint = match failed {
                Some(error) if !connected => format!(
                    "{} {error}",
                    fluxa_ui::localized("settings.account_sign_in_failed", language)
                ),
                _ => fluxa_ui::localized(
                    if connected {
                        "settings.connected"
                    } else {
                        "settings.not_connected"
                    },
                    language,
                ),
            };
            json!({
                "kind": "tracker",
                "extension": tracker["extension"],
                "title": tracker["title"],
                "hint": hint,
                "button": fluxa_ui::localized(
                    if connected { "settings.account_disconnect" } else { "settings.account_connect" },
                    language,
                ),
                "primary": !connected,
                "op": if connected { "disconnect" } else { "auth" },
            })
        })
        .collect()
}

fn scrobble_method(event: &str) -> Option<&'static str> {
    match event {
        "playback.started" => Some("tracker.scrobbleStart"),
        "playback.paused" => Some("tracker.scrobblePause"),
        "playback.ended" => Some("tracker.scrobbleStop"),
        _ => None,
    }
}

pub(crate) fn emit(event: &str, payload: Value) {
    let scripts = SCRIPTS
        .lock()
        .map(|scripts| scripts.clone())
        .unwrap_or_default();
    for script in scripts {
        let extension = &script.extension;
        let listens = extension
            .contributes
            .events
            .iter()
            .any(|name| name == event);
        let scrobbles = tracker_script(extension) && scrobble_method(event).is_some();
        if !listens && !scrobbles {
            continue;
        }
        let event = event.to_owned();
        let payload = payload.clone();
        let _ = std::thread::Builder::new()
            .name(format!("fluxa-skin-event-{}", script.extension.id))
            .spawn(move || {
                if listens {
                    let _ = run(&script, "onEvent", vec![json!(event), payload.clone()]);
                }
                if scrobbles
                    && let Some(method) = scrobble_method(&event)
                    && let Err(error) = run(&script, method, vec![payload])
                {
                    eprintln!("skin script {}: {method}: {error}", script.extension.id);
                }
            });
    }
}

pub(crate) fn trackers() -> Vec<Value> {
    app()["trackers"].as_array().cloned().unwrap_or_default()
}

pub(crate) fn tracker_call(id: &str, method: &str, args: Vec<Value>) -> Result<Value, String> {
    if !TRACKER_METHODS.contains(&method) {
        return Err(format!("unknown tracker method {method}"));
    }
    let script = script(id).filter(|script| tracker_script(&script.extension));
    let script = script.ok_or_else(|| format!("no tracker extension {id}"))?;
    let value = run(&script, &format!("tracker.{method}"), args)?.value;
    Ok(value)
}

fn field<'a>(value: &'a Value, path: &str) -> &'a Value {
    value.pointer(path).unwrap_or(&Value::Null)
}

pub(crate) fn sync(mut next: Value) {
    for key in ["position", "duration"] {
        if let Some(seconds) = next["player"].get(key).and_then(Value::as_f64) {
            next["player"][key] = json!(seconds.floor());
        }
    }
    let previous = match APP.lock() {
        Ok(mut app) => {
            if *app == next {
                return;
            }
            std::mem::replace(&mut *app, next.clone())
        }
        Err(_) => return,
    };
    bump();
    let mut events = Vec::new();
    let (was, now) = (&previous["player"], &next["player"]);
    if now.is_object() {
        let position = field(now, "/position").as_f64().unwrap_or_default();
        if !was.is_object() || field(was, "/title") != field(now, "/title") {
            if let Ok(mut reported) = REPORTED.lock() {
                *reported = position;
            }
            events.push(("playback.started", now.clone()));
        } else if field(was, "/paused") != field(now, "/paused") {
            events.push(("playback.paused", now.clone()));
        } else if field(now, "/paused") == &json!(false)
            && let Ok(mut reported) = REPORTED.lock()
            && (position - *reported).abs() >= PROGRESS_STEP
        {
            *reported = position;
            events.push(("playback.progress", now.clone()));
        }
    } else if was.is_object() {
        events.push(("playback.ended", json!({"title": field(was, "/title")})));
    }
    let (was, now) = (field(&previous, "/profile/id"), field(&next, "/profile/id"));
    if !was.is_null() && was != now {
        events.push(("profile.changed", next["profile"].clone()));
    }
    let (was, now) = (
        field(&previous, "/navigation/route"),
        field(&next, "/navigation/route"),
    );
    if !was.is_null() && was != now {
        events.push(("navigation.changed", next["navigation"].clone()));
    }
    let watched = |app: &Value| -> Vec<Value> {
        app["library"]["watched"]
            .as_array()
            .cloned()
            .unwrap_or_default()
    };
    if previous["library"].is_object() {
        let known = watched(&previous);
        let fresh: Vec<Value> = watched(&next)
            .into_iter()
            .filter(|id| !known.contains(id))
            .collect();
        if !fresh.is_empty() {
            events.push(("playback.watched", json!({"ids": fresh})));
        }
    }
    for (event, payload) in events {
        emit(event, payload);
    }
}

fn current(tag: &str) -> bool {
    CURRENT.lock().is_ok_and(|current| *current == tag)
}

pub(super) fn start(skin: &Skin) {
    if skin.scripts().is_empty() || !consent::allowed(skin) {
        return;
    }
    let tag = format!("{}@{}", skin.manifest.id, skin.manifest.version);
    let Ok(mut running) = CURRENT.lock() else {
        return;
    };
    if *running == tag {
        return;
    }
    running.clone_from(&tag);
    drop(running);
    if let Ok(mut data) = DATA.lock() {
        data.clear();
    }
    if let Ok(mut scripts) = SCRIPTS.lock() {
        *scripts = skin.scripts().to_vec();
    }
    for script in skin.scripts() {
        if tracker_script(&script.extension) {
            act(&script.extension.id, "tracker.status", json!({}));
        }
        let extension = script.extension.clone();
        let script = script.clone();
        let tag = tag.clone();
        let spawned = std::thread::Builder::new()
            .name(format!("fluxa-skin-script-{}", extension.id))
            .spawn(move || {
                while current(&tag) {
                    let begun = std::time::Instant::now();
                    match run(&script, "data", Vec::new()) {
                        Ok(outcome) if current(&tag) => {
                            if let Ok(mut data) = DATA.lock() {
                                data.insert(extension.id.clone(), outcome.value);
                            }
                            bump();
                            eprintln!(
                                "skin script {}: ready in {:?}",
                                extension.id,
                                begun.elapsed()
                            );
                        }
                        Ok(_) => {}
                        Err(error) => eprintln!("skin script {}: {error}", extension.id),
                    }
                    let Some(seconds) = extension.refresh_seconds else {
                        break;
                    };
                    std::thread::sleep(Duration::from_secs(seconds.max(MIN_REFRESH)));
                }
            });
        if spawned.is_err() {
            eprintln!("skin script: could not start worker");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use super::SERIAL;

    fn wait(until: &dyn Fn() -> bool) {
        let deadline = std::time::Instant::now() + Duration::from_secs(10);
        while !until() && std::time::Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(20));
        }
    }

    fn data_dir() {
        let dir = std::path::PathBuf::from(std::env::var("HOME").unwrap())
            .join(".cache/claude-tmp/skin-host-tests");
        MANAGER.lock().unwrap().data_dir = Some(dir);
    }

    fn launch(id: &str, extension: Value, script: &str) -> Skin {
        let mut manifest = json!({
            "schemaVersion": 1, "id": id, "name": id, "version": "1.0.0",
            "kind": "script", "entry": "script.js",
        });
        manifest
            .as_object_mut()
            .unwrap()
            .extend(extension.as_object().unwrap().clone());
        let skin = fluxa_skin::load_embedded(&[
            (
                "manifest.json",
                &json!({"schemaVersion": 1, "id": id, "name": id, "version": "1.0.0", "extensions": ["ext/extension.json"]}).to_string(),
            ),
            ("ext/extension.json", &manifest.to_string()),
            ("ext/script.js", script),
        ])
        .unwrap();
        data_dir();
        MANAGER.lock().unwrap().allowed.push(id.to_owned());
        take_requests();
        start(&skin);
        skin
    }

    fn player(title: &str, paused: bool) -> Value {
        json!({"player": {"title": title, "paused": paused, "position": 5.0, "duration": 100.0, "volume": 80.0}})
    }

    #[test]
    fn script_result_lands_in_ext_data() {
        let _serial = SERIAL.lock().unwrap_or_else(|held| held.into_inner());
        let before = version();
        launch(
            "agenda-data",
            json!({}),
            r#"class Provider { async data() { return { cells: [{ label: "Mon" }] }; } }"#,
        );
        wait(&|| version() != before);
        assert_eq!(snapshot()["agenda-data"]["cells"][0]["label"], "Mon");
    }

    #[test]
    fn storage_survives_between_runs_and_lands_on_disk() {
        let _serial = SERIAL.lock().unwrap_or_else(|held| held.into_inner());
        let file = std::path::PathBuf::from(std::env::var("HOME").unwrap())
            .join(".cache/claude-tmp/skin-host-tests/skin-data/storage/counter-store.json");
        let _ = std::fs::remove_file(&file);
        launch(
            "counter-store",
            json!({"permissions": ["storage"]}),
            r#"class Provider {
                async data() { return { count: storage.get("count", 0) }; }
                async bump(args) { storage.set("count", storage.get("count", 0) + args.by); }
            }"#,
        );
        wait(&|| snapshot()["counter-store"]["count"] == 0);
        act("counter-store", "bump", json!({"by": 2}));
        wait(&|| snapshot()["counter-store"]["count"] == 2);
        act("counter-store", "bump", json!({"by": 3}));
        wait(&|| snapshot()["counter-store"]["count"] == 5);
        assert_eq!(snapshot()["counter-store"]["count"], 5);
        let raw = std::fs::read_to_string(&file).unwrap();
        assert_eq!(serde_json::from_str::<Value>(&raw).unwrap()["count"], 5);
    }

    #[test]
    fn storage_needs_its_permission() {
        let _serial = SERIAL.lock().unwrap_or_else(|held| held.into_inner());
        launch(
            "no-store",
            json!({}),
            r#"class Provider {
                async data() { return { count: storage.get("count", 0) }; }
                async bump() { storage.set("count", 9); }
            }"#,
        );
        wait(&|| snapshot()["no-store"]["count"] == 0);
        act("no-store", "bump", json!({}));
        std::thread::sleep(Duration::from_millis(500));
        assert_eq!(snapshot()["no-store"]["count"], 0);
    }

    #[test]
    fn state_follows_the_granted_slices() {
        let _serial = SERIAL.lock().unwrap_or_else(|held| held.into_inner());
        sync(player("Dune", true));
        launch(
            "state-reader",
            json!({"permissions": ["state.player"]}),
            r#"class Provider { async data() { return { title: state("player.title"), paused: state("player.paused"), profile: state("profile") === undefined }; } }"#,
        );
        wait(&|| snapshot()["state-reader"]["title"] == "Dune");
        assert_eq!(snapshot()["state-reader"]["paused"], true);
        assert_eq!(snapshot()["state-reader"]["profile"], true);
        assert_eq!(app()["player"]["volume"], 80.0);
    }

    #[test]
    fn state_is_hidden_without_permission() {
        let _serial = SERIAL.lock().unwrap_or_else(|held| held.into_inner());
        sync(player("Dune", true));
        launch(
            "state-blind",
            json!({}),
            r#"class Provider { async data() { return { hidden: state("player") === undefined }; } }"#,
        );
        wait(&|| snapshot()["state-blind"]["hidden"] == true);
        assert_eq!(snapshot()["state-blind"]["hidden"], true);
    }

    #[test]
    fn permitted_requests_are_queued_and_others_dropped() {
        let _serial = SERIAL.lock().unwrap_or_else(|held| held.into_inner());
        launch(
            "remote",
            json!({"permissions": ["actions.player"]}),
            r#"class Provider {
                async data() { return {}; }
                async go() { app.call("player.volume", { value: 30 }); app.call("navigate", { screen: "library" }); app.call("player.pause"); }
            }"#,
        );
        wait(&|| snapshot()["remote"].is_object());
        take_requests();
        act("remote", "go", json!({}));
        wait(&|| !REQUESTS.lock().unwrap().is_empty());
        std::thread::sleep(Duration::from_millis(200));
        let queued = take_requests();
        assert_eq!(
            queued,
            vec![
                Request {
                    name: "player.volume".into(),
                    args: json!({"value": 30})
                },
                Request {
                    name: "player.pause".into(),
                    args: json!({})
                },
            ]
        );
    }

    #[test]
    fn app_events_reach_listening_scripts() {
        let _serial = SERIAL.lock().unwrap_or_else(|held| held.into_inner());
        sync(json!({}));
        data_dir();
        store::forget("listener");
        launch(
            "listener",
            json!({
                "permissions": ["storage", "background", "events.playback", "events.profile", "events.navigation"],
                "contributes": {"events": ["playback.started", "playback.paused", "playback.ended", "playback.watched", "profile.changed", "navigation.changed"]}
            }),
            r#"class Provider {
                async data() { return { seen: storage.get("seen", []) }; }
                async onEvent(name) { storage.set("seen", [...storage.get("seen", []), name]); }
            }"#,
        );
        let step = |value: Value, count: usize| {
            sync(value);
            wait(&|| {
                store::storage("listener")
                    .get("seen")
                    .and_then(Value::as_array)
                    .map_or(0, Vec::len)
                    >= count
            });
        };
        let mut first = player("Dune", false);
        first["profile"] = json!({"id": "a", "name": "A"});
        first["navigation"] = json!({"route": "home"});
        step(first.clone(), 1);
        let mut paused = first.clone();
        paused["player"]["paused"] = json!(true);
        step(paused.clone(), 2);
        let mut moved = paused.clone();
        moved["navigation"] = json!({"route": "library"});
        step(moved.clone(), 3);
        let mut other = moved.clone();
        other["profile"] = json!({"id": "b", "name": "B"});
        step(other.clone(), 4);
        let mut ended = other.clone();
        ended["player"] = Value::Null;
        step(ended.clone(), 5);
        ended["library"] = json!({"watched": []});
        sync(ended.clone());
        ended["library"] = json!({"watched": ["tt1"]});
        step(ended, 6);
        assert_eq!(
            store::storage("listener")["seen"],
            json!([
                "playback.started",
                "playback.paused",
                "navigation.changed",
                "profile.changed",
                "playback.ended",
                "playback.watched"
            ])
        );
    }

    #[test]
    fn declared_settings_are_stored_validated_and_passed_on() {
        let _serial = SERIAL.lock().unwrap_or_else(|held| held.into_inner());
        let file = std::path::PathBuf::from(std::env::var("HOME").unwrap())
            .join(".cache/claude-tmp/skin-host-tests/skin-data/settings/prefs.json");
        let _ = std::fs::remove_file(&file);
        launch(
            "prefs",
            json!({
                "permissions": ["ui.settings"],
                "settings": [
                    {"key": "loud", "title": "Loud", "kind": "toggle", "default": false},
                    {"key": "level", "title": "Level", "kind": "number", "min": 1, "max": 5, "default": 2},
                    {"key": "mode", "title": "Mode", "kind": "choice", "default": "a",
                     "choices": [{"value": "a", "label": "A"}, {"value": "b", "label": "B"}]},
                    {"key": "name", "title": "Name", "kind": "text"}
                ]
            }),
            r#"class Provider { async data() { return { loud: settings.loud, level: settings.level, mode: settings.mode }; } }"#,
        );
        wait(&|| snapshot()["prefs"]["level"] == 2.0);
        assert_eq!(snapshot()["prefs"]["loud"], false);
        assert_eq!(
            app()["extensions"]["prefs"]["settings"][2]["choices"][1]["value"],
            "b"
        );
        act("prefs", "setting", json!({"key": "loud", "value": "true"}));
        wait(&|| snapshot()["prefs"]["loud"] == true);
        act("prefs", "setting", json!({"key": "level", "value": 99}));
        act("prefs", "setting", json!({"key": "mode", "value": "b"}));
        wait(&|| snapshot()["prefs"]["mode"] == "b");
        assert_eq!(snapshot()["prefs"]["level"], 2.0);
        let raw = std::fs::read_to_string(&file).unwrap();
        let saved: Value = serde_json::from_str(&raw).unwrap();
        assert_eq!(saved["loud"], true);
        assert!(saved.get("level").is_none());
    }

    #[test]
    fn tracker_extension_scrobbles_and_answers_queries() {
        let _serial = SERIAL.lock().unwrap_or_else(|held| held.into_inner());
        sync(json!({}));
        launch(
            "tracker-probe",
            json!({
                "permissions": ["storage", "provider.tracker", "events.playback"],
                "contributes": {"trackers": [{"id": "probe", "title": "Probe"}]}
            }),
            r#"class Provider {
                constructor() {
                    this.tracker = {
                        auth: async () => ({ connected: true }),
                        scrobbleStart: async (item) => storage.set("last", "start:" + item.title),
                        scrobblePause: async (item) => storage.set("last", "pause:" + item.title),
                        scrobbleStop: async (item) => storage.set("last", "stop:" + item.title),
                        watched: async () => [{ id: "tt1" }],
                        library: async () => [{ id: "tt2" }],
                    };
                }
                async data() { return {}; }
            }"#,
        );
        assert_eq!(trackers()[0]["id"], "probe");
        assert_eq!(
            tracker_call("tracker-probe", "auth", Vec::new()).unwrap(),
            json!({"connected": true})
        );
        assert_eq!(
            tracker_call("tracker-probe", "watched", Vec::new()).unwrap()[0]["id"],
            "tt1"
        );
        assert!(tracker_call("tracker-probe", "secret", Vec::new()).is_err());
        assert!(tracker_call("missing", "auth", Vec::new()).is_err());
        sync(player("Dune", false));
        wait(&|| store::storage("tracker-probe").get("last") == Some(&json!("start:Dune")));
        assert_eq!(store::storage("tracker-probe")["last"], "start:Dune");
        sync(json!({}));
        wait(&|| store::storage("tracker-probe").get("last") == Some(&json!("stop:Dune")));
        assert_eq!(store::storage("tracker-probe")["last"], "stop:Dune");
    }

    #[test]
    fn extension_without_tracker_permission_is_not_listed() {
        let _serial = SERIAL.lock().unwrap_or_else(|held| held.into_inner());
        launch(
            "plain",
            json!({}),
            r#"class Provider { async data() { return {}; } }"#,
        );
        assert!(trackers().is_empty());
        assert!(tracker_call("plain", "auth", Vec::new()).is_err());
    }

    #[test]
    fn example_skin_shows_state_storage_and_settings() {
        let _serial = SERIAL.lock().unwrap_or_else(|held| held.into_inner());
        data_dir();
        let root = std::path::PathBuf::from(std::env::var("HOME").unwrap())
            .join(".cache/claude-tmp/skin-host-tests/skin-data");
        for kind in ["storage", "settings"] {
            let _ = std::fs::remove_file(root.join(kind).join("agenda.json"));
        }
        let skin =
            fluxa_skin::load_skin(std::path::Path::new("../../shared/skins/examples/agenda"))
                .unwrap();
        MANAGER
            .lock()
            .unwrap()
            .allowed
            .push("example-agenda".to_owned());
        sync(player("Dune", false));
        start(&skin);
        wait(&|| snapshot()["agenda"]["items"].is_array());
        wait(&|| snapshot()["agenda"]["nowPlaying"] == "Dune");
        assert_eq!(snapshot()["agenda"]["nowPlaying"], "Dune");
        assert_eq!(snapshot()["agenda"]["items"].as_array().unwrap().len(), 5);
        act("agenda", "toggle", json!({"index": 1}));
        wait(&|| snapshot()["agenda"]["items"][1]["done"] == true);
        act("agenda", "setting", json!({"key": "days", "value": "7"}));
        wait(&|| snapshot()["agenda"]["items"].as_array().unwrap().len() == 7);
        act(
            "agenda",
            "setting",
            json!({"key": "quiet", "value": "true"}),
        );
        wait(&|| !REQUESTS.lock().unwrap().is_empty());
        let queued = take_requests();
        assert_eq!(queued[0].name, "player.volume");
        assert_eq!(queued[0].args["value"], 20.0);
        assert_eq!(trackers()[0]["id"], "example");
        let data = json!({"ext": snapshot(), "app": app()});
        let tree = skin
            .resolve("page.agenda", &data, &mut fluxa_skin::ResolveCache::new())
            .unwrap()
            .unwrap();
        assert!(!tree[0].children.is_empty());
        assert_eq!(snapshot()["agenda"]["items"][1]["done"], true);
    }

    #[test]
    fn settings_page_lists_extension_settings_and_routes_changes() {
        let _serial = SERIAL.lock().unwrap_or_else(|held| held.into_inner());
        launch(
            "page-prefs",
            json!({
                "permissions": ["ui.settings"],
                "settings": [{"key": "mode", "title": "Mode", "kind": "choice", "default": "a",
                    "choices": [{"value": "a", "label": "A"}, {"value": "b", "label": "B"}]}]
            }),
            r#"class Provider { async data() { return { mode: settings.mode }; } }"#,
        );
        wait(&|| snapshot()["page-prefs"]["mode"] == "a");
        let listed = manager_data("en_us")["extensions"].clone();
        assert_eq!(listed[0]["id"], "page-prefs");
        assert_eq!(listed[0]["settings"][0]["choices"][1]["value"], "b");
        let args = [
            ("id".to_owned(), json!("page-prefs")),
            ("key".to_owned(), json!("mode")),
            ("value".to_owned(), json!("b")),
        ]
        .into_iter()
        .collect();
        remember(SkinFrame {
            targets: vec![(
                40_000_100,
                SkinTarget::Action(Arc::new(fluxa_skin::ResolvedAction {
                    name: "ext_setting".to_owned(),
                    args,
                })),
            )],
            layout: HomeLayout::default(),
        });
        let Some(Local::Script(id, method, args)) = local_for_node(40_000_100) else {
            panic!("ext_setting did not route to a script");
        };
        act(&id, &method, args);
        wait(&|| snapshot()["page-prefs"]["mode"] == "b");
        assert_eq!(snapshot()["page-prefs"]["mode"], "b");
    }

    #[test]
    fn tracker_buttons_run_methods_and_show_the_outcome() {
        let _serial = SERIAL.lock().unwrap_or_else(|held| held.into_inner());
        launch(
            "tracker-page",
            json!({
                "permissions": ["provider.tracker"],
                "contributes": {"trackers": [{"id": "probe", "title": "Probe"}]}
            }),
            r#"class Provider {
                constructor() {
                    this.tracker = {
                        auth: async () => ({ connected: true }),
                        watched: async () => [{ id: "a" }, { id: "b" }],
                        library: async () => { throw new Error("offline"); },
                    };
                }
                async data() { return {}; }
            }"#,
        );
        let press = |op: &str| {
            let args = [
                ("id".to_owned(), json!("tracker-page")),
                ("op".to_owned(), json!(op)),
            ]
            .into_iter()
            .collect();
            remember(SkinFrame {
                targets: vec![(
                    40_000_200,
                    SkinTarget::Action(Arc::new(fluxa_skin::ResolvedAction {
                        name: "ext_tracker".to_owned(),
                        args,
                    })),
                )],
                layout: HomeLayout::default(),
            });
            let Some(Local::Script(id, method, args)) = local_for_node(40_000_200) else {
                panic!("ext_tracker did not route to a script");
            };
            act(&id, &method, args);
        };
        let listed = || manager_data("en_us")["trackers"][0].clone();
        press("auth");
        wait(&|| listed()["status"] != "");
        assert_eq!(listed()["status"], "Connected");
        press("watched");
        wait(&|| listed()["watched"] == 2);
        assert_eq!(listed()["watched"], 2);
        press("library");
        wait(&|| app()["trackers"][0]["results"]["library"]["ok"] == false);
        assert!(
            app()["trackers"][0]["results"]["library"]["error"]
                .as_str()
                .unwrap()
                .contains("offline")
        );
    }

    #[test]
    fn tracker_rows_follow_the_scripts_own_status() {
        let _serial = SERIAL.lock().unwrap_or_else(|held| held.into_inner());
        data_dir();
        store::forget("tracker-rows");
        launch(
            "tracker-rows",
            json!({
                "permissions": ["storage", "provider.tracker"],
                "contributes": {"trackers": [{"id": "probe", "title": "Probe"}]}
            }),
            r#"class Provider {
                constructor() {
                    this.tracker = {
                        status: async () => ({ connected: storage.get("token", "") !== "" }),
                        auth: async () => { storage.set("token", "t"); return { connected: true }; },
                        disconnect: async () => { storage.remove("token"); },
                    };
                }
                async data() { return {}; }
            }"#,
        );
        let row = || tracker_rows("english_us")[0].clone();
        wait(&|| app()["trackers"][0]["results"]["status"].is_object());
        assert_eq!(row()["kind"], "tracker");
        assert_eq!(row()["op"], "auth");
        assert_eq!(row()["primary"], true);
        let account = fluxa_ui::SETTINGS_SECTIONS
            .iter()
            .position(|section| section.title == "Account")
            .unwrap();
        let mut model = SettingsModel::default();
        model.active_section = account;
        let page =
            super::tree::prefs_data(&model, fluxa_ui::UiFormFactor::Desktop, &HashMap::new());
        let on_page: Vec<&Value> = page["blocks"]
            .as_array()
            .unwrap()
            .iter()
            .flat_map(|group| group["rows"].as_array().unwrap())
            .collect();
        let service = on_page.iter().position(|row| row["kind"] == "service");
        let tracker = on_page.iter().position(|row| row["kind"] == "tracker");
        assert!(service.is_some() && tracker > service);
        assert_eq!(on_page[tracker.unwrap()]["title"], "Probe");
        act("tracker-rows", "tracker.auth", json!({}));
        wait(&|| row()["op"] == "disconnect");
        assert_eq!(row()["op"], "disconnect");
        assert_eq!(row()["primary"], false);
        act("tracker-rows", "tracker.disconnect", json!({}));
        wait(&|| row()["op"] == "auth");
        assert_eq!(row()["op"], "auth");
    }
}
