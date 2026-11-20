use super::*;
use std::sync::atomic::{AtomicU64, Ordering};

static STARTED: LazyLock<Mutex<std::collections::HashSet<String>>> = LazyLock::new(Default::default);
static DATA: LazyLock<Mutex<serde_json::Map<String, Value>>> = LazyLock::new(Default::default);
static VERSION: AtomicU64 = AtomicU64::new(0);

pub(super) fn version() -> u64 {
    VERSION.load(Ordering::Relaxed)
}

pub(super) fn snapshot() -> Value {
    DATA.lock()
        .map_or(Value::Null, |data| Value::Object(data.clone()))
}

pub(super) fn start(skin: &Skin) {
    if skin.scripts().is_empty() {
        return;
    }
    let tag = format!("{}@{}", skin.manifest.id, skin.manifest.version);
    let Ok(mut started) = STARTED.lock() else {
        return;
    };
    if !started.insert(tag) {
        return;
    }
    for script in skin.scripts() {
        if !script.extension.allows("network") && !script.extension.network_hosts.is_empty() {
            continue;
        }
        let id = script.extension.id.clone();
        let source = script.source.clone();
        let hosts = script.extension.network_hosts.clone();
        let spawned = std::thread::Builder::new()
            .name(format!("fluxa-skin-script-{id}"))
            .spawn(move || {
                let begun = std::time::Instant::now();
                match fluxa_effects::run_skin_script(&source, &id, "data", &hosts, Vec::new()) {
                    Ok(value) => {
                        if let Ok(mut data) = DATA.lock() {
                            data.insert(id.clone(), value);
                        }
                        VERSION.fetch_add(1, Ordering::Relaxed);
                        eprintln!("skin script {id}: ready in {:?}", begun.elapsed());
                    }
                    Err(error) => eprintln!("skin script {id}: {error}"),
                }
            });
        if spawned.is_err() {
            eprintln!("skin script: could not start worker");
        }
    }
}
