use std::sync::{
    Arc, Mutex,
    atomic::{AtomicUsize, Ordering},
    mpsc::{self, Receiver, Sender},
};

use fluxa_app::FluxaRuntime;
use serde_json::{Value, json};

use crate::{EffectCompletion, EffectExecutor, Storage, executor::core_value};

struct AppSession {
    runtime: FluxaRuntime,
    storage: Storage,
    persisted_addons: Value,
    executor: EffectExecutor,
    sender: Sender<EffectCompletion>,
    outstanding: usize,
}

impl AppSession {
    fn open(storage: Storage, sender: Sender<EffectCompletion>) -> Result<Self, String> {
        let state = persisted_runtime_state(&storage, json!({}));
        let persisted_addons = state["addons"]["installed"].clone();
        let runtime = FluxaRuntime::new(state)?;
        let executor = EffectExecutor::new(storage.clone());
        executor.warm_torrent_engine();
        Ok(Self {
            runtime,
            storage,
            persisted_addons,
            executor,
            sender,
            outstanding: 0,
        })
    }

    fn dispatch(&mut self, action: Value) -> Result<(), String> {
        let update = self.runtime.dispatch(action)?;
        self.persist_addons();
        self.schedule(update.effects);
        Ok(())
    }

    fn complete(&mut self, completion: EffectCompletion) {
        self.outstanding = self.outstanding.saturating_sub(1);
        if completion.status != "ok" {
            crate::log!(
                "[fluxa-session] effect {} failed: {}",
                completion.effect_type,
                completion.error
            );
        }
        let result = if completion.effect_type == "fetchDiscoverPage" {
            self.runtime.complete_discover_page_result(
                &completion.effect_id,
                completion.status,
                completion.value,
                completion.error,
            )
        } else {
            self.runtime.complete_effect_result(
                &completion.effect_id,
                completion.status,
                completion.value,
                completion.error,
            )
        };
        match result {
            Ok(update) => {
                self.persist_addons();
                self.schedule(update.effects);
            }
            Err(error) => crate::log!(
                "[fluxa-session] effect {} completion rejected: {error}",
                completion.effect_id
            ),
        }
    }

    fn persist_addons(&mut self) {
        let snapshot = self.runtime.snapshot();
        let Some(installed) = snapshot
            .pointer("/addons/installed")
            .filter(|value| value.is_array())
        else {
            return;
        };
        if *installed == self.persisted_addons {
            return;
        }
        let active_id = snapshot
            .pointer("/profile/activeProfileId")
            .and_then(Value::as_str)
            .unwrap_or("guest")
            .to_owned();
        let profiles = self
            .storage
            .read_json("profiles")
            .ok()
            .flatten()
            .unwrap_or_else(|| json!([]));
        let owner = core_value(
            "effectiveAddonsOwnerId",
            json!({"profiles": profiles, "activeProfileId": active_id}),
        )
        .and_then(|value| value.as_str().map(ToOwned::to_owned))
        .unwrap_or(active_id);
        let installed = installed.clone();
        match self
            .storage
            .write_json(&Storage::addons_key(&owner), &installed)
        {
            Ok(()) => self.persisted_addons = installed,
            Err(error) => crate::log!("[fluxa-session] saving add-ons failed: {error}"),
        }
    }

    fn snapshot(&self) -> Arc<Value> {
        self.runtime.snapshot_shared()
    }

    fn revision(&self) -> u64 {
        self.runtime.revision()
    }

    fn has_outstanding_effects(&self) -> bool {
        self.outstanding > 0
    }

    fn schedule(&mut self, effects: Vec<Value>) {
        for effect in effects {
            self.outstanding += 1;
            self.executor.spawn(effect, self.sender.clone());
        }
    }
}

enum Command {
    Dispatch(Value),
    Completed(EffectCompletion),
    Stop,
}

struct Published {
    revision: u64,
    snapshot: Arc<Value>,
    outstanding: bool,
}

pub struct SessionHandle {
    commands: Sender<Command>,
    published: Arc<Mutex<Published>>,
    queued: Arc<AtomicUsize>,
    storage: Storage,
    executor: EffectExecutor,
}

impl SessionHandle {
    pub fn open(storage: Storage) -> Result<Self, String> {
        let (completions_tx, completions) = mpsc::channel();
        let session = AppSession::open(storage.clone(), completions_tx)?;
        let executor = session.executor.clone();
        let published = Arc::new(Mutex::new(Published {
            revision: session.revision(),
            snapshot: session.snapshot(),
            outstanding: false,
        }));
        let queued = Arc::new(AtomicUsize::new(0));
        let (commands, inbox) = mpsc::channel();
        let forward = commands.clone();
        std::thread::Builder::new()
            .name("fluxa-session-effects".to_owned())
            .spawn(move || {
                for completion in completions {
                    if forward.send(Command::Completed(completion)).is_err() {
                        break;
                    }
                }
            })
            .map_err(|error| error.to_string())?;
        let shared = published.clone();
        let pending = queued.clone();
        std::thread::Builder::new()
            .name("fluxa-session".to_owned())
            .spawn(move || run(session, inbox, shared, pending))
            .map_err(|error| error.to_string())?;
        Ok(Self {
            commands,
            published,
            queued,
            storage,
            executor,
        })
    }

    pub fn dispatch(&self, action: Value) -> Result<(), String> {
        self.queued.fetch_add(1, Ordering::AcqRel);
        self.commands.send(Command::Dispatch(action)).map_err(|_| {
            self.queued.fetch_sub(1, Ordering::AcqRel);
            "session stopped".to_owned()
        })
    }

    pub fn snapshot(&self) -> Arc<Value> {
        self.published
            .lock()
            .map(|published| published.snapshot.clone())
            .unwrap_or_default()
    }

    pub fn revision(&self) -> u64 {
        self.published
            .lock()
            .map_or(0, |published| published.revision)
    }

    pub fn active_profile(&self) -> Value {
        self.snapshot()
            .pointer("/profile/active")
            .cloned()
            .unwrap_or(Value::Null)
    }

    pub fn has_queued_dispatches(&self) -> bool {
        self.queued.load(Ordering::Acquire) > 0
    }

    pub fn has_outstanding_effects(&self) -> bool {
        self.published
            .lock()
            .is_ok_and(|published| published.outstanding)
    }

    pub fn poll_torrent_status(&self, link: String, file_id: Option<usize>) -> Receiver<Value> {
        self.executor.poll_torrent_status(link, file_id)
    }

    pub fn fetch_json(&self, url: String) -> Receiver<Option<Value>> {
        self.executor.fetch_json(url)
    }

    pub fn request_json(&self, plan: Value) -> Receiver<Option<Value>> {
        self.executor.request_json(plan)
    }

    pub fn executor(&self) -> EffectExecutor {
        self.executor.clone()
    }

    pub fn storage(&self) -> &Storage {
        &self.storage
    }
}

impl Drop for SessionHandle {
    fn drop(&mut self) {
        let _ = self.commands.send(Command::Stop);
    }
}

fn run(
    mut session: AppSession,
    inbox: Receiver<Command>,
    published: Arc<Mutex<Published>>,
    queued: Arc<AtomicUsize>,
) {
    while let Ok(command) = inbox.recv() {
        let mut dispatched = 0;
        let mut next = Some(command);
        while let Some(command) = next.take() {
            match command {
                Command::Dispatch(action) => {
                    dispatched += 1;
                    if let Err(error) = session.dispatch(action) {
                        crate::log!("[fluxa-session] dispatch failed: {error}");
                    }
                }
                Command::Completed(completion) => session.complete(completion),
                Command::Stop => return,
            }
            next = inbox.try_recv().ok();
        }
        if let Ok(mut published) = published.lock() {
            let revision = session.revision();
            if revision != published.revision {
                published.revision = revision;
                published.snapshot = session.snapshot();
            }
            published.outstanding = session.has_outstanding_effects();
        }
        queued.fetch_sub(dispatched, Ordering::AcqRel);
    }
}

pub fn persisted_runtime_state(storage: &Storage, default_prefs: Value) -> Value {
    let profiles = storage
        .read_json("profiles")
        .ok()
        .flatten()
        .unwrap_or_else(|| json!([]));
    let stored_active_id = storage
        .read_json("active_profile_id")
        .ok()
        .flatten()
        .and_then(|value| value.as_str().map(ToOwned::to_owned));
    let plan = core_value(
        "activeProfilePlan",
        json!({"profiles": profiles, "storedActiveId": stored_active_id}),
    )
    .unwrap_or(Value::Null);
    let active_id = plan
        .get("activeId")
        .and_then(Value::as_str)
        .unwrap_or("guest")
        .to_owned();
    let active_profile = plan.get("activeProfile").cloned().unwrap_or(Value::Null);
    let mut prefs = storage
        .read_json(&Storage::prefs_key(&active_id))
        .ok()
        .flatten()
        .unwrap_or(default_prefs);
    if active_profile
        .get("nuvioAccessToken")
        .and_then(Value::as_str)
        .is_some_and(|token| !token.is_empty())
        && let Some(values) = prefs.as_object_mut()
    {
        for key in ["integrationLibrarySource", "continueWatchingSource"] {
            if values
                .get(key)
                .and_then(Value::as_str)
                .is_none_or(|source| source == "local")
            {
                values.insert(key.to_owned(), Value::String("nuvio".to_owned()));
            }
        }
    }
    let owner = core_value(
        "effectiveAddonsOwnerId",
        json!({"profiles": profiles, "activeProfileId": active_id}),
    )
    .and_then(|value| value.as_str().map(ToOwned::to_owned))
    .unwrap_or_else(|| active_id.clone());
    let addons = match crate::executor::account::account_source(&active_profile, "addons")
        .get("snapshotKey")
        .and_then(Value::as_str)
    {
        Some(key) => crate::executor::account::snapshot_addons(
            &storage.read_json(key).ok().flatten().unwrap_or(Value::Null),
        ),
        None => storage
            .read_json(&Storage::addons_key(&owner))
            .ok()
            .flatten()
            .or_else(|| {
                storage
                    .read_json(&Storage::addons_key(&active_id))
                    .ok()
                    .flatten()
            })
            .or_else(|| storage.read_json("addons").ok().flatten())
            .filter(Value::is_array)
            .unwrap_or_else(|| json!([])),
    };
    json!({
        "settings": {"values": prefs},
        "profile": {"active": active_profile, "activeProfileId": active_id},
        "addons": {"installed": addons},
    })
}
