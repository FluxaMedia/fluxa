use std::sync::Arc;

use fluxa_ui::{
    CalendarModel, DetailModel, DiscoverModel, HomeModel, LibraryModel, LibraryTab, SettingsModel,
    UiFormFactorJson, detail_model_from_core_snapshot, discover_model_from_core_snapshot,
    home_model_from_core_snapshot, library_model_from_core_snapshot,
    refresh_discover_model_from_core_snapshot, settings_model_from_core_snapshot,
};
use serde_json::{Value, json};

use crate::core_value;

pub(crate) struct Request {
    pub revision: u64,
    pub snapshot: Arc<Value>,
    pub form_factor: UiFormFactorJson,
    pub library_tab: LibraryTab,
    pub library_query: String,
    pub library_sort: String,
    pub hero_trailers: Arc<Value>,
}

pub(crate) struct Projection {
    pub revision: u64,
    pub route: String,
    pub home: HomeModel,
    pub library: LibraryModel,
    pub discover: DiscoverModel,
    pub calendar: CalendarModel,
    pub detail: DetailModel,
    pub settings: SettingsModel,
    pub trailer_targets: Vec<Value>,
}

#[derive(Default)]
struct HeroCache(Option<(Value, Option<Value>)>);

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

fn project(
    request: Request,
    hero_cache: &mut HeroCache,
    discover_cache: &mut DiscoverCache,
) -> Projection {
    let snapshot = &request.snapshot;
    let hero_inputs = json!({
        "categories": snapshot.pointer("/home/categories").cloned().unwrap_or_else(|| json!([])),
        "billboard": snapshot.pointer("/home/billboard").cloned().unwrap_or(Value::Null),
        "prefs": snapshot.pointer("/settings/values").cloned().unwrap_or_else(|| json!({})),
        "fetchedTrailers": request.hero_trailers.get("fetched").cloned().unwrap_or_else(|| json!({})),
        "fetchedIds": request.hero_trailers.get("ids").cloned().unwrap_or_else(|| json!([])),
        "fetchedLogos": {},
        "fetchedLogoIds": [],
    });
    let hero_plan = match &hero_cache.0 {
        Some((inputs, plan)) if *inputs == hero_inputs => plan.clone(),
        _ => {
            let plan = core_value("homeHeroPlan", hero_inputs.clone());
            hero_cache.0 = Some((hero_inputs, plan.clone()));
            plan
        }
    };
    let trailer_targets = hero_plan
        .as_ref()
        .and_then(|plan| plan.get("trailerTargets"))
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let home = match hero_plan {
        Some(hero_plan) if snapshot.get("home").is_some_and(Value::is_object) => {
            let mut home_snapshot = json!({});
            if let (Some(target), Some(source)) =
                (home_snapshot.as_object_mut(), snapshot.as_object())
            {
                for key in ["billboard", "library", "profile", "settings"] {
                    if let Some(value) = source.get(key) {
                        target.insert(key.to_owned(), value.clone());
                    }
                }
                let mut home = snapshot["home"].clone();
                if let Some(billboard) = hero_plan.get("billboard") {
                    home["billboard"] = billboard.clone();
                }
                if let Some(slides) = hero_plan.get("slides") {
                    home["heroSlides"] = slides.clone();
                }
                target.insert("home".to_owned(), home);
            }
            home_model_from_core_snapshot(&home_snapshot, request.form_factor)
        }
        _ => home_model_from_core_snapshot(snapshot, request.form_factor),
    };
    let route = snapshot
        .pointer("/navigation/route")
        .and_then(Value::as_str)
        .unwrap_or("home")
        .to_owned();
    let mut library = library_model_from_core_snapshot(snapshot);
    library.query = request.library_query.clone();
    library.sort_by = request.library_sort.clone();
    if route == "library" {
        let source = snapshot.get("library").cloned().unwrap_or_else(|| json!({}));
        if let Some(plan) = core_value(
            "libraryViewPlan",
            json!({
                "watchlist": source.get("watchlist"),
                "watching": source.get("continueWatching"),
                "completed": source.get("completed"),
                "dropped": source.get("dropped"),
                "favorites": source.get("liked"),
                "progress": source.get("progress"),
                "tab": request.library_tab.core_tab_key(),
                "query": request.library_query,
                "sortBy": request.library_sort,
            }),
        ) {
            library.apply_core_plan(&plan, request.library_tab);
        }
    }
    Projection {
        revision: request.revision,
        route,
        home,
        library,
        discover: discover_cache.project(snapshot),
        calendar: fluxa_ui::calendar_model_from_core_snapshot(snapshot),
        detail: detail_model_from_core_snapshot(snapshot),
        settings: settings_model_from_core_snapshot(snapshot),
        trailer_targets,
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) struct Projector {
    requests: std::sync::mpsc::Sender<Request>,
    results: std::sync::mpsc::Receiver<Projection>,
    in_flight: bool,
}

#[cfg(not(target_arch = "wasm32"))]
impl Projector {
    pub fn new() -> Self {
        let (requests, request_rx) = std::sync::mpsc::channel::<Request>();
        let (result_tx, results) = std::sync::mpsc::channel();
        std::thread::Builder::new()
            .name("fluxa-projector".to_owned())
            .spawn(move || {
                let mut hero_cache = HeroCache::default();
                let mut discover_cache = DiscoverCache::default();
                while let Ok(mut request) = request_rx.recv() {
                    while let Ok(newer) = request_rx.try_recv() {
                        request = newer;
                    }
                    if result_tx.send(project(request, &mut hero_cache, &mut discover_cache)).is_err() {
                        break;
                    }
                }
            })
            .expect("spawn projector thread");
        Self {
            requests,
            results,
            in_flight: false,
        }
    }

    pub fn submit(&mut self, request: Request) {
        self.in_flight = self.requests.send(request).is_ok();
    }

    pub fn take(&mut self) -> Option<Projection> {
        let mut latest = None;
        while let Ok(projection) = self.results.try_recv() {
            latest = Some(projection);
        }
        latest
    }

    pub fn pending(&self) -> bool {
        self.in_flight
    }

    pub fn settle(&mut self, revision: u64, latest: u64) {
        if revision == latest {
            self.in_flight = false;
        }
    }
}

#[cfg(target_arch = "wasm32")]
pub(crate) struct Projector {
    hero_cache: HeroCache,
    discover_cache: DiscoverCache,
    ready: Option<Projection>,
}

#[cfg(target_arch = "wasm32")]
impl Projector {
    pub fn new() -> Self {
        Self {
            hero_cache: HeroCache::default(),
            discover_cache: DiscoverCache::default(),
            ready: None,
        }
    }

    pub fn submit(&mut self, request: Request) {
        self.ready = Some(project(request, &mut self.hero_cache, &mut self.discover_cache));
    }

    pub fn take(&mut self) -> Option<Projection> {
        self.ready.take()
    }

    pub fn pending(&self) -> bool {
        false
    }

    pub fn settle(&mut self, _revision: u64, _latest: u64) {}
}
