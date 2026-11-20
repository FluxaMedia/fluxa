use super::*;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

const MIN_REFRESH: u64 = 30;

static CURRENT: LazyLock<Mutex<String>> = LazyLock::new(Default::default);
static DATA: LazyLock<Mutex<serde_json::Map<String, Value>>> = LazyLock::new(Default::default);
static VERSION: AtomicU64 = AtomicU64::new(0);

pub(super) fn version() -> u64 {
    VERSION.load(Ordering::Relaxed)
}

pub(super) fn snapshot() -> Value {
    DATA.lock()
        .map_or(Value::Null, |data| Value::Object(data.clone()))
}

fn current(tag: &str) -> bool {
    CURRENT.lock().is_ok_and(|current| *current == tag)
}

pub(super) fn start(skin: &Skin) {
    if skin.scripts().is_empty() {
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
    for script in skin.scripts() {
        let extension = script.extension.clone();
        let source = script.source.clone();
        let tag = tag.clone();
        let spawned = std::thread::Builder::new()
            .name(format!("fluxa-skin-script-{}", extension.id))
            .spawn(move || {
                while current(&tag) {
                    let begun = std::time::Instant::now();
                    let outcome = fluxa_effects::run_skin_script(
                        &source,
                        &extension.id,
                        "data",
                        &extension.network_hosts,
                        Vec::new(),
                    );
                    match outcome {
                        Ok(value) if current(&tag) => {
                            if let Ok(mut data) = DATA.lock() {
                                data.insert(extension.id.clone(), value);
                            }
                            VERSION.fetch_add(1, Ordering::Relaxed);
                            eprintln!("skin script {}: ready in {:?}", extension.id, begun.elapsed());
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

    #[test]
    fn script_result_lands_in_ext_data() {
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
        let before = version();
        start(&skin);
        let deadline = std::time::Instant::now() + Duration::from_secs(10);
        while version() == before && std::time::Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(20));
        }
        assert_eq!(snapshot()["agenda"]["cells"][0]["label"], "Mon");
    }
}
