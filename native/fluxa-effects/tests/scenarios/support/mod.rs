mod proxy;
mod world;

#[allow(unused_imports)]
pub use world::{Recorded, Reply, World};

use fluxa_effects::{SessionHandle, Storage};
use serde_json::{Value, json};
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, OnceLock};
use std::time::{Duration, Instant};

static WORLD: OnceLock<Arc<World>> = OnceLock::new();
static SERIAL: Mutex<()> = Mutex::new(());
static APPS: AtomicUsize = AtomicUsize::new(0);

fn world() -> Arc<World> {
    WORLD
        .get_or_init(|| {
            let world = Arc::new(World::default());
            let port = proxy::spawn(world.clone());
            let proxy_url = format!("http://127.0.0.1:{port}");
            unsafe {
                std::env::set_var("SSL_CERT_FILE", proxy::CA_PEM);
                std::env::set_var("HTTPS_PROXY", &proxy_url);
                std::env::set_var("https_proxy", &proxy_url);
                std::env::set_var("HTTP_PROXY", &proxy_url);
                std::env::set_var("http_proxy", &proxy_url);
                std::env::remove_var("NO_PROXY");
                std::env::remove_var("no_proxy");
            }
            world
        })
        .clone()
}

#[allow(dead_code)]
pub struct Scenario {
    pub world: Arc<World>,
    _serial: MutexGuard<'static, ()>,
}

impl Scenario {
    pub fn start() -> Self {
        let serial = SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        let world = world();
        world.reset();
        Self {
            world,
            _serial: serial,
        }
    }

    pub fn app(&self) -> App {
        self.app_with_profile(json!(null))
    }

    pub fn app_with_profile(&self, profile: Value) -> App {
        let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
            .join(format!("app-{}-{}", std::process::id(), APPS.fetch_add(1, Ordering::Relaxed)));
        let _ = std::fs::remove_dir_all(&dir);
        let storage = Storage::open(dir.clone()).expect("storage");
        if profile.is_object() {
            let id = profile["id"].as_str().unwrap_or("guest").to_owned();
            storage.write_json("profiles", &json!([profile])).expect("profiles");
            storage.write_json("active_profile_id", &json!(id)).expect("active profile");
        }
        App {
            session: SessionHandle::open(storage).expect("session"),
            dir,
        }
    }
}

pub struct App {
    pub session: SessionHandle,
    dir: PathBuf,
}

impl App {
    pub fn dispatch(&self, action: Value) -> &Self {
        self.session.dispatch(action).expect("dispatch");
        self.settle();
        self
    }

    pub fn restart(self) -> App {
        let App { session, dir } = self;
        drop(session);
        let storage = Storage::open(dir.clone()).expect("storage");
        App {
            session: SessionHandle::open(storage).expect("session"),
            dir,
        }
    }

    pub fn settle(&self) {
        let deadline = Instant::now() + Duration::from_secs(30);
        let mut quiet = 0;
        while quiet < 3 {
            if Instant::now() > deadline {
                panic!("session did not settle");
            }
            std::thread::sleep(Duration::from_millis(15));
            if self.session.has_queued_dispatches() || self.session.has_outstanding_effects() {
                quiet = 0;
            } else {
                quiet += 1;
            }
        }
    }

    pub fn state(&self) -> Value {
        (*self.session.snapshot()).clone()
    }

    pub fn at(&self, pointer: &str) -> Value {
        self.state().pointer(pointer).cloned().unwrap_or(Value::Null)
    }
}

pub fn addon_manifest(id: &str, resources: Value, types: Value, catalogs: Value) -> Value {
    json!({"id": id, "version": "1.0.0", "name": id, "resources": resources, "types": types, "catalogs": catalogs})
}
