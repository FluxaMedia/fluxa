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
    persisted_plugins: Value,
    persisted_downloads: Vec<(String, String)>,
    executor: EffectExecutor,
    sender: Sender<EffectCompletion>,
    outstanding: usize,
    plugin_queue: Vec<String>,
}

impl AppSession {
    fn open(storage: Storage, sender: Sender<EffectCompletion>) -> Result<Self, String> {
        let state = persisted_runtime_state(&storage, json!({}));
        let persisted_addons = state["addons"]["installed"].clone();
        let persisted_plugins = state["plugins"].clone();
        let active_id = state["profile"]["activeProfileId"]
            .as_str()
            .unwrap_or("guest")
            .to_owned();
        let mut runtime = FluxaRuntime::new(state)?;
        let stored_downloads = storage
            .read_json(&Storage::downloads_key(&active_id))
            .ok()
            .flatten()
            .filter(Value::is_array);
        if let Some(items) = stored_downloads {
            let _ = runtime.dispatch(json!({"type": "offlineDownloadsRestored", "items": items}));
        }
        let executor = EffectExecutor::bound_to(storage.clone(), &active_id);
        executor.warm_torrent_engine();
        Ok(Self {
            runtime,
            storage,
            persisted_addons,
            persisted_plugins,
            persisted_downloads: Vec::new(),
            executor,
            sender,
            outstanding: 0,
            plugin_queue: Vec::new(),
        })
    }

    fn dispatch(&mut self, action: Value) -> Result<(), String> {
        let update = self.runtime.dispatch(action)?;
        self.persist_addons();
        self.persist_plugins();
        self.persist_downloads();
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
                self.persist_plugins();
                self.persist_downloads();
                self.schedule(update.effects);
            }
            Err(error) => crate::log!(
                "[fluxa-session] effect {} completion rejected: {error}",
                completion.effect_id
            ),
        }
    }

    fn queue_plugin_repositories(&mut self, urls: Vec<String>) {
        let known = self
            .runtime
            .snapshot()
            .pointer("/plugins/repositories")
            .and_then(Value::as_array)
            .map(|repos| {
                repos
                    .iter()
                    .filter_map(|repo| repo.get("manifestUrl").and_then(Value::as_str))
                    .map(|url| url.trim_end_matches('/').to_owned())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        self.plugin_queue = urls
            .into_iter()
            .filter(|url| !known.contains(&url.trim_end_matches('/').to_owned()))
            .collect();
        self.add_next_plugin_repository();
    }

    fn add_next_plugin_repository(&mut self) {
        if self.plugin_queue.is_empty() {
            return;
        }
        let adding = self
            .runtime
            .snapshot()
            .pointer("/plugins/addingRepositoryUrl")
            .is_some_and(|value| !value.is_null());
        if adding {
            return;
        }
        let url = self.plugin_queue.remove(0);
        if let Err(error) = self.dispatch(json!({
            "type": "pluginRepositoryAddRequested",
            "manifestUrl": url,
        })) {
            crate::log!("[fluxa-session] plugin sync failed: {error}");
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

    fn persist_plugins(&mut self) {
        let snapshot = self.runtime.snapshot();
        let Some(plugins) = snapshot.get("plugins").filter(|value| value.is_object()) else {
            return;
        };
        if *plugins == self.persisted_plugins {
            return;
        }
        let active_id = snapshot
            .pointer("/profile/activeProfileId")
            .and_then(Value::as_str)
            .unwrap_or("guest");
        let stored = json!({
            "repositories": plugins.get("repositories"),
            "scrapers": plugins.get("scrapers"),
        });
        match self
            .storage
            .write_json(&Storage::plugins_key(active_id), &stored)
        {
            Ok(()) => self.persisted_plugins = plugins.clone(),
            Err(error) => crate::log!("[fluxa-session] saving plugins failed: {error}"),
        }
    }

    fn persist_downloads(&mut self) {
        let snapshot = self.runtime.snapshot();
        let Some(items) = snapshot
            .pointer("/offline/items")
            .and_then(Value::as_array)
        else {
            return;
        };
        let signature = items
            .iter()
            .map(|item| {
                (
                    item["id"].as_str().unwrap_or_default().to_owned(),
                    item["status"].as_str().unwrap_or_default().to_owned(),
                )
            })
            .collect::<Vec<_>>();
        if signature == self.persisted_downloads {
            return;
        }
        let active_id = snapshot
            .pointer("/profile/activeProfileId")
            .and_then(Value::as_str)
            .unwrap_or("guest");
        match self
            .storage
            .write_json(&Storage::downloads_key(active_id), &Value::Array(items.clone()))
        {
            Ok(()) => self.persisted_downloads = signature,
            Err(error) => crate::log!("[fluxa-session] saving downloads failed: {error}"),
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
    NuvioPlugins(Vec<String>),
    Stop,
}

struct Published {
    revision: u64,
    snapshot: Arc<Value>,
    outstanding: bool,
}

#[cfg(target_arch = "wasm32")]
struct Local {
    session: Mutex<AppSession>,
    inbox: Mutex<Receiver<Command>>,
    completions: Mutex<Receiver<EffectCompletion>>,
}

pub struct SessionHandle {
    commands: Sender<Command>,
    #[cfg(target_arch = "wasm32")]
    local: Local,
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
        #[cfg(target_arch = "wasm32")]
        let local = Local {
            session: Mutex::new(session),
            inbox: Mutex::new(inbox),
            completions: Mutex::new(completions),
        };
        #[cfg(not(target_arch = "wasm32"))]
        {
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
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let sink = commands.clone();
            let pending = queued.clone();
            let forward: crate::ActionSink = Arc::new(move |action| {
                pending.fetch_add(1, Ordering::AcqRel);
                if sink.send(Command::Dispatch(action)).is_err() {
                    pending.fetch_sub(1, Ordering::AcqRel);
                }
            });
            executor.set_watch_sink(forward.clone());
            executor.set_download_sink(forward);
        }
        #[cfg(not(target_arch = "wasm32"))]
        let sync = commands.clone();
        #[cfg(not(target_arch = "wasm32"))]
        let nuvio = executor.clone();
        #[cfg(not(target_arch = "wasm32"))]
        crate::executor::runtime().spawn(async move {
            match nuvio.nuvio_plugin_urls().await {
                Ok(urls) if !urls.is_empty() => {
                    let _ = sync.send(Command::NuvioPlugins(urls));
                }
                Ok(_) => {}
                Err(error) => crate::log!("[fluxa-session] Nuvio plugin sync failed: {error}"),
            }
        });
        Ok(Self {
            commands,
            #[cfg(target_arch = "wasm32")]
            local,
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

    #[cfg(target_arch = "wasm32")]
    fn pump(&self) {
        let (Ok(mut session), Ok(inbox), Ok(completions)) = (
            self.local.session.lock(),
            self.local.inbox.lock(),
            self.local.completions.lock(),
        ) else {
            return;
        };
        let mut dispatched = 0;
        loop {
            let command = match completions.try_recv() {
                Ok(completion) => Command::Completed(completion),
                Err(_) => match inbox.try_recv() {
                    Ok(command) => command,
                    Err(_) => break,
                },
            };
            if !step(&mut session, command, &mut dispatched) {
                break;
            }
        }
        publish(&session, &self.published);
        self.queued.fetch_sub(dispatched, Ordering::AcqRel);
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn pump(&self) {}

    pub fn snapshot(&self) -> Arc<Value> {
        self.pump();
        self.published
            .lock()
            .map(|published| published.snapshot.clone())
            .unwrap_or_default()
    }

    pub fn revision(&self) -> u64 {
        self.pump();
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
        self.pump();
        self.queued.load(Ordering::Acquire) > 0
    }

    pub fn has_outstanding_effects(&self) -> bool {
        self.pump();
        self.published
            .lock()
            .is_ok_and(|published| published.outstanding)
    }

    pub fn poll_torrent_status(&self, link: String, file_id: Option<usize>) -> Receiver<Value> {
        self.executor.poll_torrent_status(link, file_id)
    }

    pub fn deactivate_torrent(&self, link: String) {
        self.executor.deactivate_torrent(link);
    }

    pub fn fetch_json(&self, url: String) -> Receiver<Option<Value>> {
        self.executor.fetch_json(url)
    }

    pub fn fetch_bytes(&self, url: String) -> Receiver<Option<Vec<u8>>> {
        self.executor.fetch_bytes(url)
    }

    pub fn resolve_trickplay_source(&self, payload: Value) -> Receiver<Option<Value>> {
        self.executor.resolve_trickplay_source(payload)
    }

    pub fn request_json(&self, plan: Value) -> Receiver<Option<Value>> {
        self.executor.request_json(plan)
    }

    pub fn fetch_addon_subtitles(
        &self,
        content_type: String,
        id: String,
    ) -> Receiver<Option<Value>> {
        self.executor.fetch_addon_subtitles(content_type, id)
    }

    pub fn stream_feed_since(
        &self,
        opened_ms: u128,
        version: u64,
    ) -> Option<crate::executor::FeedSnapshot> {
        self.executor.stream_feed_since(opened_ms, version)
    }

    pub fn request_ok(&self, plan: Value) -> Receiver<bool> {
        self.executor.request_ok(plan)
    }

    pub fn take_watch_commands(&self) -> Vec<Value> {
        #[cfg(not(target_arch = "wasm32"))]
        return self.executor.take_watch_commands();
        #[cfg(target_arch = "wasm32")]
        Vec::new()
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
        self.executor.cancel();
        let _ = self.executor.stop_torrent();
        let _ = self.commands.send(Command::Stop);
    }
}

fn step(session: &mut AppSession, command: Command, dispatched: &mut usize) -> bool {
    match command {
        Command::Dispatch(action) => {
            *dispatched += 1;
            if let Err(error) = session.dispatch(action) {
                crate::log!("[fluxa-session] dispatch failed: {error}");
            }
        }
        Command::Completed(completion) => {
            session.complete(completion);
            session.add_next_plugin_repository();
        }
        Command::NuvioPlugins(urls) => session.queue_plugin_repositories(urls),
        Command::Stop => return false,
    }
    true
}

fn publish(session: &AppSession, published: &Mutex<Published>) {
    if let Ok(mut published) = published.lock() {
        let revision = session.revision();
        if revision != published.revision {
            published.revision = revision;
            published.snapshot = session.snapshot();
        }
        published.outstanding = session.has_outstanding_effects();
    }
}

#[cfg(not(target_arch = "wasm32"))]
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
            if !step(&mut session, command, &mut dispatched) {
                return;
            }
            next = inbox.try_recv().ok();
        }
        publish(&session, &published);
        queued.fetch_sub(dispatched, Ordering::AcqRel);
    }
}

fn adopt_legacy_documents(storage: &Storage, active_id: &str) {
    for (legacy, target) in [
        ("library", Storage::library_key(active_id)),
        ("addons", Storage::addons_key(active_id)),
    ] {
        let Some(document) = storage
            .read_json(legacy)
            .ok()
            .flatten()
            .filter(|value| !value.is_null())
        else {
            continue;
        };
        if storage.read_json(&target).ok().flatten().is_none() {
            let _ = storage.write_json(&target, &document);
        }
        let _ = storage.write_json(legacy, &Value::Null);
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
    adopt_legacy_documents(storage, &active_id);
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
            .filter(Value::is_array)
            .unwrap_or_else(|| json!([])),
    };
    json!({
        "settings": {"values": prefs},
        "profile": {"active": active_profile, "activeProfileId": active_id},
        "addons": {"installed": addons},
        "plugins": storage
            .read_json(&Storage::plugins_key(&active_id))
            .ok()
            .flatten()
            .filter(Value::is_object)
            .unwrap_or_else(|| json!({"repositories": [], "scrapers": []})),
    })
}
