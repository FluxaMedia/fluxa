use std::sync::Arc;

use fluxa_ui::{
    CalendarModel, DetailModel, DiscoverModel, FolderModel, HomeModel, LibraryModel, LibraryTab,
    SettingsModel, UiFormFactorJson, detail_model_from_core_snapshot,
    discover_model_from_core_snapshot, home_model_with_rows, library_model_from_core_snapshot,
    refresh_discover_model_from_core_snapshot, settings_model_from_core_snapshot,
};
use serde_json::{Value, json};

use crate::{Route, Snapshot, core_value};

#[derive(Clone)]
pub(crate) struct Request {
    pub revision: u64,
    pub snapshot: Snapshot,
    pub form_factor: UiFormFactorJson,
    pub library_tab: LibraryTab,
    pub library_query: String,
    pub library_sort: String,
    pub library_type: String,
    pub library_list: bool,
    pub library_downloads: bool,
    pub library_download_group: Option<String>,
    pub hero_trailers: Arc<Value>,
    pub hero_page: u64,
    pub generations: Option<Arc<fluxa_effects::SliceGenerations>>,
}

pub(crate) struct Projection {
    pub revision: u64,
    pub library: LibraryModel,
    pub discover: DiscoverModel,
    pub folder: FolderModel,
    pub calendar: CalendarModel,
    pub detail: DetailModel,
    pub settings: SettingsModel,
    pub stream_providers: bool,
}

pub(crate) struct HomeProjection {
    pub revision: u64,
    pub home: HomeModel,
    pub trailer_targets: Vec<Value>,
}

#[derive(Default)]
pub(crate) struct Projected {
    pub light: Option<Projection>,
    pub home: Option<HomeProjection>,
}

#[cfg(not(target_arch = "wasm32"))]
enum Part {
    Light(Box<Projection>),
    Home(Box<HomeProjection>),
    HomeUnchanged(u64),
}

const HOME_INPUT_SLICES: [&str; 5] = ["home", "billboard", "profile", "settings", "library"];

#[derive(PartialEq)]
struct HomeKey {
    generations: Vec<u64>,
    form_factor: UiFormFactorJson,
    hero_page: u64,
    fetched: Option<Value>,
    ids: Option<Value>,
}

#[derive(Default)]
struct HomeMemo(Option<HomeKey>);

impl HomeMemo {
    fn key(request: &Request) -> Option<HomeKey> {
        let generations = request.generations.as_ref()?;
        Some(HomeKey {
            generations: HOME_INPUT_SLICES
                .iter()
                .map(|slice| generations.get(*slice).copied().unwrap_or(0))
                .collect(),
            form_factor: request.form_factor,
            hero_page: request.hero_page,
            fetched: request.hero_trailers.get("fetched").cloned(),
            ids: request.hero_trailers.get("ids").cloned(),
        })
    }
}

#[derive(Default)]
struct ProviderCache(Option<(Value, bool)>);

impl ProviderCache {
    fn has_stream_providers(&mut self, snapshot: &Snapshot) -> bool {
        let addons = snapshot
            .pointer("/addons/installed")
            .cloned()
            .unwrap_or_else(|| json!([]));
        let plugin_names = snapshot
            .pointer("/plugins/scrapers")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter(|scraper| scraper.get("enabled").and_then(Value::as_bool) != Some(false))
            .filter_map(|scraper| scraper.get("name").cloned())
            .collect::<Vec<_>>();
        let request = json!({"addons": addons, "pluginNames": plugin_names});
        if let Some((cached, result)) = &self.0
            && *cached == request
        {
            return *result;
        }
        let result = core_value("providerAvailabilityPlan", request.clone())
            .and_then(|plan| plan.get("hasStreamProviders").and_then(Value::as_bool))
            .unwrap_or(true);
        self.0 = Some((request, result));
        result
    }
}

#[derive(Default)]
struct HeroCache(Option<HeroInputs>);

struct HeroInputs {
    rows: Vec<Arc<Value>>,
    rest: Value,
    plan: Option<Value>,
}

impl HeroCache {
    fn plan(&mut self, snapshot: &Snapshot, trailers: &Value, page: u64) -> Option<Value> {
        let rows = snapshot.home_rows();
        let rest = json!({
            "heroPage": page,
            "billboard": snapshot.pointer("/home/billboard"),
            "prefs": snapshot.pointer("/settings/values"),
            "fetchedTrailers": trailers.get("fetched"),
            "fetchedIds": trailers.get("ids"),
        });
        if let Some(cached) = &self.0
            && cached.rest == rest
            && cached.rows.len() == rows.len()
            && cached
                .rows
                .iter()
                .zip(rows)
                .all(|(a, b)| Arc::ptr_eq(a, b) || a == b)
        {
            return cached.plan.clone();
        }
        let plan = plan_hero(&rest, rows);
        self.0 = Some(HeroInputs {
            rows: rows.to_vec(),
            rest,
            plan: plan.clone(),
        });
        plan
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn plan_hero(request: &Value, rows: &[Arc<Value>]) -> Option<Value> {
    Some(fluxa_core::home_hero_plan_over(
        request,
        rows.iter().map(|row| &**row),
    ))
}

#[cfg(target_arch = "wasm32")]
fn plan_hero(request: &Value, rows: &[Arc<Value>]) -> Option<Value> {
    let mut request = request.clone();
    request["categories"] = Value::Array(rows.iter().map(|row| (**row).clone()).collect());
    core_value("homeHeroPlan", request)
}

#[derive(Default)]
struct RootCache {
    snapshot: Snapshot,
    slices: serde_json::Map<String, Value>,
}

impl RootCache {
    fn take_except(&mut self, snapshot: &Snapshot, excluded: &[&str]) -> Value {
        let mut slices = std::mem::take(&mut self.slices);
        let root: serde_json::Map<String, Value> = snapshot
            .names()
            .filter(|name| !excluded.contains(name))
            .filter_map(|name| {
                let value = if self.snapshot.slice_shared(snapshot, name) {
                    slices.remove(name).or_else(|| snapshot.get(name).cloned())
                } else {
                    snapshot.get(name).cloned()
                };
                Some((name.to_owned(), value?))
            })
            .collect();
        self.snapshot = snapshot.clone();
        Value::Object(root)
    }

    fn put_back(&mut self, root: Value, excluded: &[&str]) {
        if let Value::Object(mut slices) = root {
            for name in excluded {
                slices.remove(*name);
            }
            self.slices = slices;
        }
    }
}

#[derive(Default)]
struct DiscoverCache(Option<DiscoverModel>);

impl DiscoverCache {
    fn project(&mut self, snapshot: &Value) -> DiscoverModel {
        let refreshed = self
            .0
            .as_mut()
            .is_some_and(|model| refresh_discover_model_from_core_snapshot(snapshot, model));
        if !refreshed {
            self.0 = Some(discover_model_from_core_snapshot(snapshot));
        }
        self.0.clone().unwrap_or_default()
    }
}

fn project_home(
    request: &Request,
    hero_cache: &mut HeroCache,
    home_memo: &mut HomeMemo,
) -> Option<HomeProjection> {
    let key = HomeMemo::key(request);
    if key.is_some() && home_memo.0 == key {
        return None;
    }
    let snapshot = &request.snapshot;
    let hero_plan = hero_cache.plan(snapshot, &request.hero_trailers, request.hero_page);
    let trailer_targets = hero_plan
        .as_ref()
        .and_then(|plan| plan.get("trailerTargets"))
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let mut root = snapshot.view(&["settings", "profile", "billboard"]);
    if let (Some(home), Some(object)) = (snapshot.home(), root.as_object_mut()) {
        object.insert("home".to_owned(), home.rest().clone());
    }
    let home = home_model_with_rows(
        &root,
        snapshot.home_rows(),
        request.form_factor,
        hero_plan.as_ref().and_then(|plan| plan.get("billboard")),
        hero_plan.as_ref().and_then(|plan| plan.get("slides")),
    );
    home_memo.0 = key;
    Some(HomeProjection {
        revision: request.revision,
        home,
        trailer_targets,
    })
}

fn project_light(
    request: Request,
    discover_cache: &mut DiscoverCache,
    provider_cache: &mut ProviderCache,
    root_cache: &mut RootCache,
) -> Projection {
    let route = request
        .snapshot
        .pointer("/navigation/route")
        .and_then(Value::as_str)
        .map_or(Route::Home, Route::parse);
    let mut root = root_cache.take_except(&request.snapshot, &["home"]);
    if let (Some(home), Some(object)) = (request.snapshot.home(), root.as_object_mut()) {
        let home = if route == Route::Folder {
            home.value()
        } else {
            home.rest()
        };
        object.insert("home".to_owned(), home.clone());
    }
    let snapshot = &root;
    let mut library = library_model_from_core_snapshot(snapshot);
    library.query = request.library_query.clone();
    library.sort_by = request.library_sort.clone();
    library.content_type = request.library_type.clone();
    library.list_view = request.library_list;
    library.downloads_open = request.library_downloads;
    library.downloads_group = request.library_download_group.clone();
    if route == Route::Library {
        let source = snapshot
            .get("library")
            .cloned()
            .unwrap_or_else(|| json!({}));
        if let Some(plan) = core_value(
            "libraryViewPlan",
            json!({
                "watchlist": source.get("watchlist"),
                "watching": source.get("continueWatching"),
                "completed": source.get("completed"),
                "dropped": source.get("dropped"),
                "onHold": source.get("onHold"),
                "favorites": source.get("liked"),
                "progress": source.get("progress"),
                "tab": request.library_tab.core_tab_key(),
                "query": request.library_query,
                "sortBy": request.library_sort,
                "type": request.library_type,
                "source": library.source.clone(),
            }),
        ) {
            library.apply_core_plan(&plan, request.library_tab);
        }
    }
    let discover = discover_cache.project(snapshot);
    let projection = Projection {
        revision: request.revision,
        library,
        discover,
        folder: if route == Route::Folder {
            fluxa_ui::folder_model_from_core_snapshot(snapshot)
        } else {
            FolderModel::default()
        },
        calendar: crate::preview::calendar(fluxa_ui::calendar_model_from_core_snapshot(snapshot)),
        detail: detail_model(snapshot),
        settings: settings_model_from_core_snapshot(snapshot),
        stream_providers: provider_cache.has_stream_providers(&request.snapshot),
    };
    root_cache.put_back(root, &["home"]);
    projection
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) struct Projector {
    light_requests: std::sync::mpsc::Sender<Request>,
    home_requests: std::sync::mpsc::Sender<Request>,
    results: std::sync::mpsc::Receiver<Part>,
    submitted: Option<u64>,
    light_done: Option<u64>,
    home_done: Option<u64>,
    done: std::sync::Arc<std::sync::atomic::AtomicBool>,
}

#[cfg(not(target_arch = "wasm32"))]
fn newest(requests: &std::sync::mpsc::Receiver<Request>) -> Option<Request> {
    let mut request = requests.recv().ok()?;
    while let Ok(newer) = requests.try_recv() {
        request = newer;
    }
    Some(request)
}

#[cfg(not(target_arch = "wasm32"))]
impl Projector {
    pub fn new() -> Self {
        let (light_requests, light_rx) = std::sync::mpsc::channel::<Request>();
        let (home_requests, home_rx) = std::sync::mpsc::channel::<Request>();
        let (result_tx, results) = std::sync::mpsc::channel();
        let done = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let light_tx = result_tx.clone();
        let light_signal = done.clone();
        std::thread::Builder::new()
            .name("fluxa-projector".to_owned())
            .spawn(move || {
                let mut discover_cache = DiscoverCache::default();
                let mut provider_cache = ProviderCache::default();
                let mut root_cache = RootCache::default();
                while let Some(request) = newest(&light_rx) {
                    let part = Part::Light(Box::new(project_light(
                        request,
                        &mut discover_cache,
                        &mut provider_cache,
                        &mut root_cache,
                    )));
                    if light_tx.send(part).is_err() {
                        break;
                    }
                    light_signal.store(true, std::sync::atomic::Ordering::Release);
                }
            })
            .expect("spawn projector thread");
        let home_signal = done.clone();
        std::thread::Builder::new()
            .name("fluxa-projector-home".to_owned())
            .spawn(move || {
                let mut hero_cache = HeroCache::default();
                let mut home_memo = HomeMemo::default();
                while let Some(request) = newest(&home_rx) {
                    let revision = request.revision;
                    let part = match project_home(&request, &mut hero_cache, &mut home_memo) {
                        Some(home) => Part::Home(Box::new(home)),
                        None => Part::HomeUnchanged(revision),
                    };
                    if result_tx.send(part).is_err() {
                        break;
                    }
                    home_signal.store(true, std::sync::atomic::Ordering::Release);
                }
            })
            .expect("spawn home projector thread");
        Self {
            light_requests,
            home_requests,
            results,
            submitted: None,
            light_done: None,
            home_done: None,
            done,
        }
    }

    pub fn submit(&mut self, request: Request) {
        self.submitted = Some(request.revision);
        let _ = self.home_requests.send(request.clone());
        let _ = self.light_requests.send(request);
    }

    pub fn ready(&self) -> bool {
        self.done.load(std::sync::atomic::Ordering::Acquire)
    }

    pub fn take(&mut self) -> Option<Projected> {
        self.done.store(false, std::sync::atomic::Ordering::Release);
        let mut projected = Projected::default();
        let mut any = false;
        while let Ok(part) = self.results.try_recv() {
            any = true;
            match part {
                Part::Light(light) => {
                    self.light_done = Some(light.revision);
                    projected.light = Some(*light);
                }
                Part::Home(home) => {
                    self.home_done = Some(home.revision);
                    projected.home = Some(*home);
                }
                Part::HomeUnchanged(revision) => self.home_done = Some(revision),
            }
        }
        any.then_some(projected)
    }

    pub fn pending(&self) -> bool {
        self.submitted.is_some()
            && (self.light_done != self.submitted || self.home_done != self.submitted)
    }
}

#[cfg(target_arch = "wasm32")]
pub(crate) struct Projector {
    hero_cache: HeroCache,
    home_memo: HomeMemo,
    discover_cache: DiscoverCache,
    provider_cache: ProviderCache,
    root_cache: RootCache,
    ready: Option<Projected>,
}

#[cfg(target_arch = "wasm32")]
impl Projector {
    pub fn new() -> Self {
        Self {
            hero_cache: HeroCache::default(),
            home_memo: HomeMemo::default(),
            discover_cache: DiscoverCache::default(),
            provider_cache: ProviderCache::default(),
            root_cache: RootCache::default(),
            ready: None,
        }
    }

    pub fn submit(&mut self, request: Request) {
        let home = project_home(&request, &mut self.hero_cache, &mut self.home_memo);
        let light = project_light(
            request,
            &mut self.discover_cache,
            &mut self.provider_cache,
            &mut self.root_cache,
        );
        let mut next = self.ready.take().unwrap_or_default();
        next.light = Some(light);
        if home.is_some() {
            next.home = home;
        }
        self.ready = Some(next);
    }

    pub fn take(&mut self) -> Option<Projected> {
        self.ready.take()
    }

    pub fn pending(&self) -> bool {
        false
    }

    pub fn ready(&self) -> bool {
        self.ready.is_some()
    }
}

pub(crate) fn detail_model(snapshot: &Value) -> fluxa_ui::DetailModel {
    let mut detail = detail_model_from_core_snapshot(snapshot);
    let values = snapshot.pointer("/settings/values").unwrap_or(&Value::Null);
    detail.sections = Some(
        fluxa_core::detail_sections::visible(values)
            .into_iter()
            .filter_map(fluxa_ui::DetailSection::from_key)
            .collect(),
    );
    detail
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(generations: &[(&str, u64)]) -> Request {
        Request {
            revision: 1,
            snapshot: Snapshot::from_value(json!({
                "home": {"categories": [{"id": "popular", "title": "Popular", "items": [{"id": "tt1", "name": "One"}]}]},
                "detail": {"id": "tt1"},
            })),
            form_factor: UiFormFactorJson::Desktop,
            library_tab: LibraryTab::default(),
            library_query: String::new(),
            library_sort: String::new(),
            library_type: String::new(),
            library_list: false,
            library_downloads: false,
            library_download_group: None,
            hero_trailers: Arc::new(json!({})),
            hero_page: 0,
            generations: Some(Arc::new(
                generations
                    .iter()
                    .map(|(slice, generation)| ((*slice).to_owned(), *generation))
                    .collect(),
            )),
        }
    }

    struct Caches {
        hero: HeroCache,
        memo: HomeMemo,
    }

    impl Caches {
        fn new() -> Self {
            Self {
                hero: HeroCache::default(),
                memo: HomeMemo::default(),
            }
        }

        fn home(&mut self, request: &Request) -> Option<HomeProjection> {
            project_home(request, &mut self.hero, &mut self.memo)
        }
    }

    #[test]
    fn opening_a_detail_does_not_rebuild_the_home_model() {
        let mut caches = Caches::new();
        assert!(
            caches
                .home(&request(&[("home", 4), ("detail", 1)]))
                .is_some()
        );
        assert!(
            caches
                .home(&request(&[("home", 4), ("detail", 2)]))
                .is_none()
        );
    }

    #[test]
    fn a_new_catalog_page_rebuilds_the_home_model() {
        let mut caches = Caches::new();
        caches.home(&request(&[("home", 4)]));
        assert!(caches.home(&request(&[("home", 5)])).is_some());
    }

    #[test]
    fn fixture_previews_always_rebuild_the_home_model() {
        let mut caches = Caches::new();
        let mut preview = request(&[]);
        preview.generations = None;
        assert!(caches.home(&preview).is_some());
        assert!(caches.home(&preview).is_some());
    }

    #[test]
    fn changed_hero_page_rebuilds_the_home_model() {
        let mut caches = Caches::new();
        caches.home(&request(&[("home", 4)]));
        let mut wrapped = request(&[("home", 4)]);
        wrapped.hero_page = 1;
        assert!(caches.home(&wrapped).is_some());
    }

    #[test]
    fn a_changed_slice_is_reprojected_while_the_cache_is_warm() {
        let (mut discover, mut providers, mut roots) = (
            DiscoverCache::default(),
            ProviderCache::default(),
            RootCache::default(),
        );
        let first = request(&[("home", 4), ("detail", 1)]);
        let mut second = first.clone();
        second.snapshot.set("detail", json!({"id": "tt2"}));
        let light = project_light(first, &mut discover, &mut providers, &mut roots);
        assert_eq!(light.detail.id, "tt1");
        let light = project_light(second, &mut discover, &mut providers, &mut roots);
        assert_eq!(light.detail.id, "tt2");
    }

    #[test]
    fn detail_is_projected_without_waiting_for_the_home_model() {
        let light = project_light(
            request(&[("home", 4), ("detail", 2)]),
            &mut DiscoverCache::default(),
            &mut ProviderCache::default(),
            &mut RootCache::default(),
        );
        assert_eq!(light.detail.id, "tt1");
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn card(row: usize, index: usize) -> Value {
        json!({
            "id": format!("tt{row:02}{index:04}"),
            "type": "movie",
            "name": format!("Title {row} {index}"),
            "poster": format!("https://img.example.test/poster/{row}/{index}.jpg"),
            "background": format!("https://img.example.test/bg/{row}/{index}.jpg"),
            "logo": format!("https://img.example.test/logo/{row}/{index}.png"),
            "description": "A long enough description to look like real catalog metadata, with a few sentences of plot summary.",
            "releaseInfo": "2024",
            "imdbRating": "7.4",
            "genres": ["Drama", "Thriller", "Mystery"],
            "runtime": "118 min",
            "cast": ["Actor One", "Actor Two", "Actor Three", "Actor Four"],
        })
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn loaded_snapshot() -> Snapshot {
        let mut engine = fluxa_core::Engine::new(json!({})).expect("engine");
        let load = engine
            .dispatch(json!({"type": "homeLoadRequested", "profile": {"id": "p1"}, "language": "en", "force": false}))
            .expect("load");
        let ids = |update: &fluxa_core::Update, kind: &str| -> Vec<String> {
            update
                .effects
                .iter()
                .filter_map(|effect| serde_json::to_value(effect).ok())
                .filter(|effect| effect["type"] == kind)
                .map(|effect| effect["id"].as_str().unwrap().to_owned())
                .collect()
        };
        let bootstrap = ids(&load, "readHomeBootstrap");
        let feeds: Vec<Value> = (0..20).map(|i| json!({"key": format!("row{i}")})).collect();
        let started = engine
            .complete(json!({"effectId": bootstrap[0], "status": "ok", "value": {"plan": {"feeds": feeds}}, "error": null}))
            .expect("bootstrap");
        let mut snapshot = engine.shared_snapshot();
        for (index, id) in ids(&started, "fetchHomeFeed").iter().enumerate() {
            let items: Vec<Value> = (0..100).map(|i| card(index, i)).collect();
            let value = json!({"row": {"id": format!("row{index}"), "name": format!("Row {index}"), "type": "movie", "items": items}});
            let update = engine
                .complete(json!({"effectId": id, "status": "ok", "value": value, "error": null}))
                .expect("feed");
            for (name, value) in update.state {
                snapshot.set(&name, value);
            }
            if let Some(home) = update.home {
                snapshot.set_home(home);
            }
        }
        let list = |n: usize| (0..n).map(|i| card(30, i)).collect::<Vec<_>>();
        snapshot.set(
            "library",
            json!({"isLoading": false, "watchlist": list(300), "continueWatching": list(20), "completed": list(200), "dropped": [], "liked": list(100)}),
        );
        snapshot
    }


    #[cfg(not(target_arch = "wasm32"))]
    mod counting {
        use std::alloc::{GlobalAlloc, Layout, System};
        use std::sync::atomic::{AtomicUsize, Ordering::Relaxed};

        pub static LIVE: AtomicUsize = AtomicUsize::new(0);
        pub static PEAK: AtomicUsize = AtomicUsize::new(0);

        pub struct Counting;

        unsafe impl GlobalAlloc for Counting {
            unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
                let live = LIVE.fetch_add(layout.size(), Relaxed) + layout.size();
                PEAK.fetch_max(live, Relaxed);
                unsafe { System.alloc(layout) }
            }

            unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
                LIVE.fetch_sub(layout.size(), Relaxed);
                unsafe { System.dealloc(ptr, layout) }
            }

            unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new: usize) -> *mut u8 {
                let live = LIVE.fetch_add(new, Relaxed) + new - layout.size();
                LIVE.fetch_sub(layout.size(), Relaxed);
                PEAK.fetch_max(live, Relaxed);
                unsafe { System.realloc(ptr, layout, new) }
            }
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[global_allocator]
    static ALLOC: counting::Counting = counting::Counting;

    #[cfg(not(target_arch = "wasm32"))]
    fn live_mb() -> f64 {
        counting::LIVE.load(std::sync::atomic::Ordering::Relaxed) as f64 / 1048576.0
    }

    #[test]
    #[ignore = "run with: cargo test --release -p fluxa-host --lib projection::tests::projection_cost -- --ignored --nocapture"]
    #[cfg(not(target_arch = "wasm32"))]
    fn projection_cost() {
        let before = live_mb();
        let snapshot = loaded_snapshot();
        println!("snapshot live: {:.1} MB", live_mb() - before);
        let mut req = request(&[("home", 4), ("library", 2)]);
        req.snapshot = snapshot;
        let time = |label: &str, mut f: Box<dyn FnMut() + '_>| {
            f();
            let started = std::time::Instant::now();
            for _ in 0..20 {
                f();
            }
            println!("{label}: {:?}", started.elapsed() / 20);
        };
        let before = live_mb();
        let held = project_home(&req, &mut HeroCache::default(), &mut HomeMemo::default());
        println!("home projection live: {:.1} MB", live_mb() - before);
        drop(held);
        let (mut discover, mut providers, mut roots) = (
            DiscoverCache::default(),
            ProviderCache::default(),
            RootCache::default(),
        );
        time(
            "project_light",
            Box::new(|| drop(project_light(req.clone(), &mut discover, &mut providers, &mut roots))),
        );
        time(
            "project_home cold",
            Box::new(|| drop(project_home(&req, &mut HeroCache::default(), &mut HomeMemo::default()))),
        );
        let (mut hero, mut memo) = (HeroCache::default(), HomeMemo::default());
        time(
            "project_home memo hit",
            Box::new(|| drop(project_home(&req, &mut hero, &mut memo))),
        );
        let mut rows = req.clone();
        rows.generations = None;
        let mut hero2 = HeroCache::default();
        time(
            "project_home hero cached",
            Box::new(|| drop(project_home(&rows, &mut hero2, &mut HomeMemo::default()))),
        );
    }
}
