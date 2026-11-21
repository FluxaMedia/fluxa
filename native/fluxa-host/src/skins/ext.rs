use super::*;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

const MIN_REFRESH: u64 = 30;

static CURRENT: LazyLock<Mutex<String>> = LazyLock::new(Default::default);
static DATA: LazyLock<Mutex<serde_json::Map<String, Value>>> = LazyLock::new(Default::default);
static SCRIPTS: LazyLock<Mutex<Vec<fluxa_skin::Script>>> = LazyLock::new(Default::default);
static STATE: LazyLock<Mutex<HashMap<String, Value>>> = LazyLock::new(Default::default);
static VERSION: AtomicU64 = AtomicU64::new(0);

pub(super) fn version() -> u64 {
    VERSION.load(Ordering::Relaxed)
}

pub(super) fn snapshot() -> Value {
    DATA.lock()
        .map_or(Value::Null, |data| Value::Object(data.clone()))
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
    VERSION.fetch_add(1, Ordering::Relaxed);
}

fn state_of(id: &str) -> Value {
    STATE
        .lock()
        .ok()
        .and_then(|state| state.get(id).cloned())
        .unwrap_or_else(|| json!({}))
}

fn keep_state(id: &str, result: &Value) {
    if let (Some(next), Ok(mut state)) = (result.get("state"), STATE.lock()) {
        state.insert(id.to_owned(), next.clone());
    }
}

fn run(script: &fluxa_skin::Script, method: &str, args: Vec<Value>) -> Result<Value, String> {
    let extension = &script.extension;
    fluxa_effects::run_skin_script(
        &script.source,
        &extension.id,
        method,
        &extension.network_hosts,
        Vec::new(),
        args,
        &state_of(&extension.id),
    )
}

fn script(id: &str) -> Option<fluxa_skin::Script> {
    SCRIPTS
        .lock()
        .ok()?
        .iter()
        .find(|script| script.extension.id == id)
        .cloned()
}

pub(crate) fn act(id: &str, method: &str, args: Value) {
    let Some(script) = script(id) else {
        return;
    };
    let method = method.to_owned();
    let _ = std::thread::Builder::new()
        .name(format!("fluxa-skin-action-{id}"))
        .spawn(move || {
            let extension = &script.extension;
            match run(&script, &method, vec![args]) {
                Ok(result) => keep_state(&extension.id, &result),
                Err(error) => {
                    eprintln!("skin script {}: {method}: {error}", extension.id);
                    return;
                }
            }
            if let Ok(value) = run(&script, "data", Vec::new())
                && let Ok(mut data) = DATA.lock()
            {
                data.insert(extension.id.clone(), value);
                VERSION.fetch_add(1, Ordering::Relaxed);
            }
        });
}

pub(crate) fn emit(event: &str, payload: Value) {
    let scripts = SCRIPTS
        .lock()
        .map(|scripts| scripts.clone())
        .unwrap_or_default();
    for script in scripts {
        if !script
            .extension
            .contributes
            .events
            .iter()
            .any(|name| name == event)
        {
            continue;
        }
        let event = event.to_owned();
        let payload = payload.clone();
        let _ = std::thread::Builder::new()
            .name(format!("fluxa-skin-event-{}", script.extension.id))
            .spawn(move || {
                if let Ok(result) = run(&script, "onEvent", vec![json!(event), payload]) {
                    keep_state(&script.extension.id, &result);
                }
            });
    }
}

#[derive(Default)]
struct Watch {
    playing: bool,
    paused: bool,
    title: String,
    reported: f64,
}

static WATCH: LazyLock<Mutex<Watch>> = LazyLock::new(Default::default);

pub(crate) fn observe(presence: &crate::presence::Presence) {
    use crate::presence::Presence;
    let Ok(mut watch) = WATCH.lock() else {
        return;
    };
    let (event, payload) = match presence {
        Presence::Playing {
            title,
            paused,
            position,
            duration,
            ..
        } => {
            let payload = json!({"title": title, "position": position, "duration": duration});
            if !watch.playing || watch.title != *title {
                watch.title.clone_from(title);
                watch.reported = *position;
                ("playback.started", payload)
            } else if watch.paused != *paused {
                ("playback.paused", payload)
            } else if !*paused && (position - watch.reported).abs() >= 15.0 {
                watch.reported = *position;
                ("playback.progress", payload)
            } else {
                watch.playing = true;
                watch.paused = *paused;
                return;
            }
        }
        _ if watch.playing => ("playback.ended", json!({"title": watch.title})),
        _ => return,
    };
    watch.playing = matches!(presence, Presence::Playing { .. });
    watch.paused = matches!(presence, Presence::Playing { paused: true, .. });
    drop(watch);
    emit(event, payload);
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
        let extension = script.extension.clone();
        let script = script.clone();
        let tag = tag.clone();
        let spawned = std::thread::Builder::new()
            .name(format!("fluxa-skin-script-{}", extension.id))
            .spawn(move || {
                while current(&tag) {
                    let begun = std::time::Instant::now();
                    let outcome = run(&script, "data", Vec::new());
                    match outcome {
                        Ok(value) if current(&tag) => {
                            if let Ok(mut data) = DATA.lock() {
                                data.insert(extension.id.clone(), value);
                            }
                            VERSION.fetch_add(1, Ordering::Relaxed);
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

    static SERIAL: Mutex<()> = Mutex::new(());

    #[test]
    fn script_result_lands_in_ext_data() {
        let _serial = SERIAL.lock().unwrap_or_else(|held| held.into_inner());
        let skin = fluxa_skin::load_embedded(&[
            (
                "manifest.json",
                r#"{"schemaVersion":1,"id":"agenda","name":"A","version":"1.0.0","extensions":["ext/extension.json"]}"#,
            ),
            (
                "ext/extension.json",
                r#"{"schemaVersion":1,"id":"agenda","name":"A","version":"1.0.0","kind":"script","entry":"script.js"}"#,
            ),
            (
                "ext/script.js",
                r#"class Provider { async data() { return { cells: [{ label: "Mon" }] }; } }"#,
            ),
        ])
        .unwrap();
        MANAGER.lock().unwrap().allowed.push("agenda".to_owned());
        let before = version();
        start(&skin);
        let deadline = std::time::Instant::now() + Duration::from_secs(10);
        while version() == before && std::time::Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(20));
        }
        assert_eq!(snapshot()["agenda"]["cells"][0]["label"], "Mon");
    }

    #[test]
    fn actions_and_events_reach_the_script() {
        let _serial = SERIAL.lock().unwrap_or_else(|held| held.into_inner());
        let skin = fluxa_skin::load_embedded(&[
            (
                "manifest.json",
                r#"{"schemaVersion":1,"id":"counter","name":"C","version":"1.0.0","extensions":["ext/extension.json"]}"#,
            ),
            (
                "ext/extension.json",
                r#"{"schemaVersion":1,"id":"counter","name":"C","version":"1.0.0","kind":"script","entry":"script.js","permissions":["events.playback","background"],"contributes":{"events":["playback.started"]}}"#,
            ),
            (
                "ext/script.js",
                r#"class Provider {
                    async data() { return { count: SCRAPER_SETTINGS.count || 0, last: SCRAPER_SETTINGS.last || "" }; }
                    async bump(args) { return { state: { ...SCRAPER_SETTINGS, count: (SCRAPER_SETTINGS.count || 0) + args.by } }; }
                    async onEvent(name, payload) { return { state: { ...SCRAPER_SETTINGS, last: name + ":" + payload.title } }; }
                }"#,
            ),
        ])
        .unwrap();
        MANAGER.lock().unwrap().allowed.push("counter".to_owned());
        let wait = |until: &dyn Fn() -> bool| {
            let deadline = std::time::Instant::now() + Duration::from_secs(10);
            while !until() && std::time::Instant::now() < deadline {
                std::thread::sleep(Duration::from_millis(20));
            }
        };
        start(&skin);
        wait(&|| snapshot()["counter"]["count"] == 0);
        act("counter", "bump", json!({"by": 2}));
        wait(&|| snapshot()["counter"]["count"] == 2);
        assert_eq!(snapshot()["counter"]["count"], 2);
        emit("playback.started", json!({"title": "Dune"}));
        std::thread::sleep(Duration::from_millis(300));
        act("counter", "bump", json!({"by": 1}));
        wait(&|| snapshot()["counter"]["count"] == 3);
        assert_eq!(snapshot()["counter"]["last"], "playback.started:Dune");
    }

    #[test]
    fn example_agenda_page_toggles_through_its_script() {
        let _serial = SERIAL.lock().unwrap_or_else(|held| held.into_inner());
        let skin =
            fluxa_skin::load_skin(std::path::Path::new("../../shared/skins/examples/agenda"))
                .unwrap();
        MANAGER
            .lock()
            .unwrap()
            .allowed
            .push("example-agenda".to_owned());
        let wait = |until: &dyn Fn() -> bool| {
            let deadline = std::time::Instant::now() + Duration::from_secs(10);
            while !until() && std::time::Instant::now() < deadline {
                std::thread::sleep(Duration::from_millis(20));
            }
        };
        start(&skin);
        wait(&|| snapshot()["agenda"]["items"].is_array());
        act("agenda", "toggle", json!({"index": 1}));
        wait(&|| snapshot()["agenda"]["items"][1]["done"] == true);
        let data = json!({"ext": snapshot()});
        let tree = skin
            .resolve("page.agenda", &data, &mut fluxa_skin::ResolveCache::new())
            .unwrap()
            .unwrap();
        assert_eq!(tree[0].children.len(), 6);
        assert_eq!(snapshot()["agenda"]["items"][1]["done"], true);
    }
}
