use std::sync::Arc;

use fluxa_ui::{
    CalendarModel, DetailModel, DiscoverModel, FolderModel, HomeModel, LibraryModel, LibraryTab,
    SettingsModel, UiFormFactorJson, detail_model_from_core_snapshot,
    discover_model_from_core_snapshot, home_model_with_hero, library_model_from_core_snapshot,
    refresh_discover_model_from_core_snapshot, settings_model_from_core_snapshot,
};
use serde_json::{Value, json};

use crate::{Route, core_value};

pub(crate) struct Request {
    pub revision: u64,
    pub snapshot: Arc<Value>,
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
}

pub(crate) struct Projection {
    pub revision: u64,
    pub home: HomeModel,
    pub library: LibraryModel,
    pub discover: DiscoverModel,
    pub folder: FolderModel,
    pub calendar: CalendarModel,
    pub detail: DetailModel,
    pub settings: SettingsModel,
    pub trailer_targets: Vec<Value>,
    pub stream_providers: bool,
}

#[derive(Default)]
struct ProviderCache(Option<(Value, bool)>);

impl ProviderCache {
    fn has_stream_providers(&mut self, snapshot: &Value) -> bool {
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
struct HeroCache(Option<(Value, Option<Value>)>);

impl HeroCache {
    fn plan(&mut self, snapshot: &Value, trailers: &Value, page: u64) -> Option<Value> {
        let page = Value::from(page);
        let inputs = [
            ("heroPage", Some(&page)),
            ("categories", snapshot.pointer("/home/categories")),
            ("billboard", snapshot.pointer("/home/billboard")),
            ("prefs", snapshot.pointer("/settings/values")),
            ("fetchedTrailers", trailers.get("fetched")),
            ("fetchedIds", trailers.get("ids")),
        ];
        if let Some((cached, plan)) = &self.0
            && inputs
                .iter()
                .all(|(key, value)| cached.get(key) == Some(value.unwrap_or(&Value::Null)))
        {
            return plan.clone();
        }
        let request = Value::Object(
            inputs
                .into_iter()
                .map(|(key, value)| (key.to_owned(), value.cloned().unwrap_or(Value::Null)))
                .collect(),
        );
        let plan = plan_hero(&request);
        self.0 = Some((request, plan.clone()));
        plan
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn plan_hero(request: &Value) -> Option<Value> {
    Some(fluxa_core::home_hero_plan(request))
}

#[cfg(target_arch = "wasm32")]
fn plan_hero(request: &Value) -> Option<Value> {
    core_value("homeHeroPlan", request.clone())
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

fn project(
    request: Request,
    hero_cache: &mut HeroCache,
    discover_cache: &mut DiscoverCache,
    provider_cache: &mut ProviderCache,
) -> Projection {
    let snapshot = &request.snapshot;
    let hero_plan = hero_cache.plan(snapshot, &request.hero_trailers, request.hero_page);
    let trailer_targets = hero_plan
        .as_ref()
        .and_then(|plan| plan.get("trailerTargets"))
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let home = home_model_with_hero(
        snapshot,
        request.form_factor,
        hero_plan.as_ref().and_then(|plan| plan.get("billboard")),
        hero_plan.as_ref().and_then(|plan| plan.get("slides")),
    );
    let route = snapshot
        .pointer("/navigation/route")
        .and_then(Value::as_str)
        .map_or(Route::Home, Route::parse);
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
    Projection {
        revision: request.revision,
        home,
        library,
        discover: discover_cache.project(snapshot),
        folder: if route == Route::Folder {
            fluxa_ui::folder_model_from_core_snapshot(snapshot)
        } else {
            FolderModel::default()
        },
        calendar: crate::preview::calendar(fluxa_ui::calendar_model_from_core_snapshot(snapshot)),
        detail: detail_model(snapshot),
        settings: settings_model_from_core_snapshot(snapshot),
        trailer_targets,
        stream_providers: provider_cache.has_stream_providers(snapshot),
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) struct Projector {
    requests: std::sync::mpsc::Sender<Request>,
    results: std::sync::mpsc::Receiver<Projection>,
    in_flight: bool,
    done: std::sync::Arc<std::sync::atomic::AtomicBool>,
}

#[cfg(not(target_arch = "wasm32"))]
impl Projector {
    pub fn new() -> Self {
        let (requests, request_rx) = std::sync::mpsc::channel::<Request>();
        let (result_tx, results) = std::sync::mpsc::channel();
        let done = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let signal = done.clone();
        std::thread::Builder::new()
            .name("fluxa-projector".to_owned())
            .spawn(move || {
                let mut hero_cache = HeroCache::default();
                let mut discover_cache = DiscoverCache::default();
                let mut provider_cache = ProviderCache::default();
                while let Ok(mut request) = request_rx.recv() {
                    while let Ok(newer) = request_rx.try_recv() {
                        request = newer;
                    }
                    if result_tx
                        .send(project(
                            request,
                            &mut hero_cache,
                            &mut discover_cache,
                            &mut provider_cache,
                        ))
                        .is_err()
                    {
                        break;
                    }
                    signal.store(true, std::sync::atomic::Ordering::Release);
                }
            })
            .expect("spawn projector thread");
        Self {
            requests,
            results,
            in_flight: false,
            done,
        }
    }

    pub fn submit(&mut self, request: Request) {
        self.in_flight = self.requests.send(request).is_ok();
    }

    pub fn ready(&self) -> bool {
        self.done.load(std::sync::atomic::Ordering::Acquire)
    }

    pub fn take(&mut self) -> Option<Projection> {
        self.done.store(false, std::sync::atomic::Ordering::Release);
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
    provider_cache: ProviderCache,
    ready: Option<Projection>,
}

#[cfg(target_arch = "wasm32")]
impl Projector {
    pub fn new() -> Self {
        Self {
            hero_cache: HeroCache::default(),
            discover_cache: DiscoverCache::default(),
            provider_cache: ProviderCache::default(),
            ready: None,
        }
    }

    pub fn submit(&mut self, request: Request) {
        self.ready = Some(project(
            request,
            &mut self.hero_cache,
            &mut self.discover_cache,
            &mut self.provider_cache,
        ));
    }

    pub fn take(&mut self) -> Option<Projection> {
        self.ready.take()
    }

    pub fn pending(&self) -> bool {
        false
    }

    pub fn ready(&self) -> bool {
        self.ready.is_some()
    }

    pub fn settle(&mut self, _revision: u64, _latest: u64) {}
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
