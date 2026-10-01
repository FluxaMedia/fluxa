use crate::storage::{Storage, sanitize_key};
use reqwest::{Client, ClientBuilder};
use serde_json::{Value, json};
use std::sync::OnceLock;
use std::sync::mpsc::Sender;
use std::time::Duration;

pub(crate) mod account;
mod addons;
mod catalog;
mod home;
mod http;
mod library;
mod mediaserver;
mod providers;
mod streams;
mod tmdb;
mod torrent;

use account::*;
use addons::*;
use catalog::*;
pub use mediaserver::{media_servers, remove_media_server};
use tmdb::*;
use torrent::*;

trait NativeTimeout {
    fn native_timeout(self, timeout: Duration) -> Self;
}

impl NativeTimeout for ClientBuilder {
    #[cfg(not(target_arch = "wasm32"))]
    fn native_timeout(self, timeout: Duration) -> Self {
        self.timeout(timeout)
    }

    #[cfg(target_arch = "wasm32")]
    fn native_timeout(self, _timeout: Duration) -> Self {
        self
    }
}

fn chrono_unix_seconds() -> i64 {
    chrono::Utc::now().timestamp()
}

#[derive(Clone)]
pub struct EffectExecutor {
    storage: Storage,
}

pub struct EffectCompletion {
    pub effect_id: String,
    pub effect_type: String,
    pub status: &'static str,
    pub value: Value,
    pub error: Value,
}

impl EffectExecutor {
    pub fn new(storage: Storage) -> Self {
        let _ = TORRENT_CACHE_DIR.set(storage.dir().join("torrent-cache"));
        mediaserver::init(&storage);
        Self { storage }
    }

    pub fn spawn(&self, effect: Value, sender: Sender<EffectCompletion>) {
        let executor = self.clone();
        let task = async move {
            let effect_id = effect
                .get("id")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_owned();
            let effect_type = effect
                .get("type")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_owned();
            let completion = match executor.execute(&effect).await {
                Ok(value) => EffectCompletion {
                    effect_id,
                    effect_type,
                    status: "ok",
                    value,
                    error: Value::Null,
                },
                Err(message) => EffectCompletion {
                    effect_id,
                    effect_type,
                    status: "error",
                    value: Value::Null,
                    error: json!({"code": "native_effect_error", "message": message}),
                },
            };
            let _ = sender.send(completion);
        };
        #[cfg(target_arch = "wasm32")]
        wasm_bindgen_futures::spawn_local(task);
        #[cfg(not(target_arch = "wasm32"))]
        runtime().spawn(task);
    }

    async fn execute(&self, effect: &Value) -> Result<Value, String> {
        let effect_type = effect
            .get("type")
            .and_then(Value::as_str)
            .ok_or_else(|| "effect is missing type".to_owned())?;
        let payload = effect.get("payload").cloned().unwrap_or_else(|| json!({}));
        let effect_kind = fluxa_core::runtime::EffectKind::from_str(effect_type)
            .ok_or_else(|| format!("unknown Core effect type: {effect_type}"))?;
        match effect_kind {
            fluxa_core::runtime::EffectKind::ReadHomeBootstrap => {
                self.read_home_bootstrap(&payload).await
            }
            fluxa_core::runtime::EffectKind::RefreshContinueWatching => {
                self.refresh_continue_watching(&payload).await
            }
            fluxa_core::runtime::EffectKind::ReadLibraryState => {
                self.read_library_state(&payload).await
            }
            fluxa_core::runtime::EffectKind::WriteLibraryCommand => {
                self.write_library_command(&payload).await
            }
            fluxa_core::runtime::EffectKind::RunAuthFlow => self.run_auth_flow(&payload).await,
            fluxa_core::runtime::EffectKind::ExchangeAuthCode => {
                self.exchange_auth_code(&payload).await
            }
            fluxa_core::runtime::EffectKind::RefreshAuthToken => {
                self.refresh_auth_token(&payload).await
            }
            fluxa_core::runtime::EffectKind::SyncWatchedState => {
                self.sync_watched_state(&payload).await
            }
            fluxa_core::runtime::EffectKind::EnqueueTraktScrobble => self.scrobble(&payload).await,
            fluxa_core::runtime::EffectKind::ReadPlaybackProgress => {
                self.read_playback_progress(&payload).await
            }
            fluxa_core::runtime::EffectKind::ReadDetailLocalState => {
                self.read_detail_local_state(&payload).await
            }
            fluxa_core::runtime::EffectKind::ReadDiscoverCatalogFilters => {
                self.read_discover_catalog_filters(&payload).await
            }
            fluxa_core::runtime::EffectKind::RunDiscover => self.run_discover(&payload).await,
            fluxa_core::runtime::EffectKind::RunSearch => self.run_search(&payload).await,
            fluxa_core::runtime::EffectKind::FetchCatalogPage => {
                self.fetch_catalog_page(&payload).await
            }
            fluxa_core::runtime::EffectKind::FetchDiscoverPage => {
                self.fetch_discover_page(&payload).await
            }
            fluxa_core::runtime::EffectKind::FetchAddonManifest => {
                self.fetch_addon_manifest(&payload).await
            }
            fluxa_core::runtime::EffectKind::FetchAddonResource => {
                self.fetch_addon_resource(&payload).await
            }
            fluxa_core::runtime::EffectKind::RefreshInstalledAddons => {
                self.read_discover_catalog_filters(&payload).await
            }
            fluxa_core::runtime::EffectKind::FetchSubtitles => {
                let stream = payload.get("stream").cloned().unwrap_or_else(|| json!({}));
                core_value("streamSubtitlesResult", stream)
                    .ok_or_else(|| "Fluxa Core could not resolve stream subtitles".to_owned())
            }
            fluxa_core::runtime::EffectKind::FetchMetaDetail => {
                let meta = self.fetch_meta_detail(&payload).await?;
                Ok(json!({"meta": meta, "mdblistRatings": null}))
            }
            fluxa_core::runtime::EffectKind::FetchMetaDetailLookup => {
                self.fetch_meta_detail(&payload).await
            }
            fluxa_core::runtime::EffectKind::FetchDetailStreams => {
                self.fetch_detail_streams(&payload).await
            }
            fluxa_core::runtime::EffectKind::PrepareDirectPlayback => {
                self.prepare_direct_playback(&payload).await
            }
            fluxa_core::runtime::EffectKind::LoadStreams => self.load_streams(&payload).await,
            fluxa_core::runtime::EffectKind::PrefetchNextEpisodeStreams => {
                self.prefetch_next_episode_streams(&payload).await
            }
            fluxa_core::runtime::EffectKind::StartTorrentStream => {
                self.start_torrent_stream(&payload)
            }
            fluxa_core::runtime::EffectKind::StopTorrent => self.stop_torrent(),
            fluxa_core::runtime::EffectKind::ReadCalendarMonth => {
                self.read_calendar_month(&payload).await
            }
            fluxa_core::runtime::EffectKind::WriteSettings => self.write_settings(&payload),
            fluxa_core::runtime::EffectKind::WritePlaybackProgress => {
                self.write_playback_progress(&payload).await
            }
            fluxa_core::runtime::EffectKind::FetchYoutubeTrailerWatchConfig
            | fluxa_core::runtime::EffectKind::FetchYoutubeTrailerPlayer
            | fluxa_core::runtime::EffectKind::FetchYoutubeTrailerPlayerScript => {
                youtube_request(&payload).await
            }
            _ => Err(format!(
                "effect {effect_type} has no native implementation yet"
            )),
        }
    }

    async fn read_calendar_month(&self, payload: &Value) -> Result<Value, String> {
        let profile_id = payload
            .get("profileId")
            .and_then(Value::as_str)
            .filter(|value| !value.is_empty())
            .unwrap_or("guest");
        let year = payload.get("year").and_then(Value::as_i64).unwrap_or(1970);
        let month = payload
            .get("month")
            .and_then(Value::as_i64)
            .unwrap_or(1)
            .clamp(1, 12);
        let month_prefix = format!("{year:04}-{month:02}");
        let library = self
            .storage
            .read_json(&Storage::library_key(profile_id))?
            .unwrap_or_else(|| json!({}));
        let mut planned_items = payload
            .get("plannedItems")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        if let Some(items) = library
            .pointer(&format!("/calendarMonthItems/{month_prefix}"))
            .and_then(Value::as_array)
        {
            planned_items.extend(items.iter().cloned());
        }
        let library_items = [
            library.get("watchlist").and_then(Value::as_array),
            library.get("continueWatching").and_then(Value::as_array),
        ]
        .into_iter()
        .flatten()
        .flatten()
        .cloned()
        .collect::<Vec<_>>();
        let profile = payload.get("profile").cloned().unwrap_or(Value::Null);
        let external_items = if profile["integrationLibrarySource"] == "simkl" {
            self.simkl_calendar_items(profile_id, &profile, year, month)
                .await
        } else if let Some(provider) = profile["integrationLibrarySource"]
            .as_str()
            .filter(|source| providers::has_capability(source, "calendar"))
        {
            self.provider_calendar_items(provider, &profile, year, month)
                .await
        } else {
            library
                .get("externalCalendarItems")
                .cloned()
                .unwrap_or_else(|| json!([]))
        };
        Ok(core_value(
            "desktopCalendarReadPlan",
            json!({
                "monthPrefix": month_prefix,
                "plannedItems": planned_items,
                "libraryItems": library_items,
                "externalItems": external_items,
            }),
        )
        .unwrap_or_else(|| json!({"items": [], "localItems": [], "externalItems": []})))
    }

    fn write_settings(&self, payload: &Value) -> Result<Value, String> {
        let key = payload
            .get("key")
            .and_then(Value::as_str)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| "settings write is missing key".to_owned())?;
        let value = payload.get("value").cloned().unwrap_or(Value::Null);
        let mut settings = self
            .storage
            .read_json("prefs")?
            .unwrap_or_else(|| json!({}));
        if !settings.is_object() {
            settings = json!({});
        }
        settings
            .as_object_mut()
            .expect("settings was normalized to an object")
            .insert(key.to_owned(), value.clone());
        self.storage.write_json("prefs", &settings)?;
        let active_id = self
            .storage
            .read_json("active_profile_id")?
            .and_then(|value| value.as_str().map(ToOwned::to_owned))
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| "guest".to_owned());
        let profiles = self
            .storage
            .read_json("profiles")?
            .unwrap_or_else(|| json!([]));
        if profiles.as_array().is_some_and(|items| {
            items
                .iter()
                .any(|profile| profile.get("id").and_then(Value::as_str) == Some(&active_id))
        }) {
            let profile_key = Storage::prefs_key(&active_id);
            let mut profile_settings = self
                .storage
                .read_json(&profile_key)?
                .unwrap_or_else(|| json!({}));
            if !profile_settings.is_object() {
                profile_settings = json!({});
            }
            profile_settings
                .as_object_mut()
                .expect("profile settings was normalized to an object")
                .insert(key.to_owned(), value);
            self.storage.write_json(&profile_key, &profile_settings)?;
        }
        Ok(settings)
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn detached<T: Send + 'static>(
    task: impl std::future::Future<Output = T> + Send + 'static,
) -> std::sync::mpsc::Receiver<T> {
    let (sender, receiver) = std::sync::mpsc::channel();
    runtime().spawn(async move {
        let _ = sender.send(task.await);
    });
    receiver
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn runtime() -> &'static tokio::runtime::Runtime {
    static RUNTIME: std::sync::OnceLock<tokio::runtime::Runtime> = std::sync::OnceLock::new();
    RUNTIME.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .worker_threads(4)
            .thread_name("fluxa-effects")
            .enable_all()
            .build()
            .expect("tokio runtime")
    })
}

#[cfg(target_arch = "wasm32")]
fn detached<T: 'static>(
    task: impl std::future::Future<Output = T> + 'static,
) -> std::sync::mpsc::Receiver<T> {
    let (sender, receiver) = std::sync::mpsc::channel();
    wasm_bindgen_futures::spawn_local(async move {
        let _ = sender.send(task.await);
    });
    receiver
}

async fn fetch_json(client: &Client, url: &str) -> Result<Value, String> {
    let parsed = reqwest::Url::parse(url).map_err(|error| error.to_string())?;
    if !matches!(parsed.scheme(), "http" | "https") {
        return Err(format!(
            "unsupported resource URL scheme: {}",
            parsed.scheme()
        ));
    }
    let response = client
        .get(parsed)
        .send()
        .await
        .map_err(|error| error.to_string())?
        .error_for_status()
        .map_err(|error| error.to_string())?;
    let body = response.text().await.map_err(|error| error.to_string())?;
    serde_json::from_str(&body).map_err(|error| error.to_string())
}

async fn fetch_text(client: &Client, url: &str) -> Result<(i32, Option<String>), String> {
    let parsed = reqwest::Url::parse(url).map_err(|error| error.to_string())?;
    if !matches!(parsed.scheme(), "http" | "https") {
        return Err(format!(
            "unsupported resource URL scheme: {}",
            parsed.scheme()
        ));
    }
    if parsed.host_str() == Some(mediaserver::HOST) {
        return Ok(mediaserver::respond(&parsed).await);
    }
    let response = client
        .get(parsed)
        .send()
        .await
        .map_err(|error| error.to_string())?;
    let status = i32::from(response.status().as_u16());
    let body = response.text().await.map_err(|error| error.to_string())?;
    Ok((status, Some(body)))
}

fn url_without_query(url: &str) -> &str {
    url.split_once('?').map_or(url, |(base, _)| base)
}

pub fn core_value(method: &str, args: Value) -> Option<Value> {
    fluxa_core::ffi::call(method, &args)
        .map_err(|error| crate::log!("[fluxa-native] core method `{method}` failed: {error}"))
        .ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::Storage;
    use std::path::PathBuf;

    fn temporary_directory() -> PathBuf {
        static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let counter = COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        std::env::temp_dir().join(format!(
            "fluxa-native-effect-{}-{counter}",
            std::process::id()
        ))
    }

    #[test]
    fn cached_legacy_addons_are_normalized_before_discover_catalog_options() {
        let addons = normalize_enabled_addons(json!([{
            "transportUrl": "https://catalog.example/manifest.json",
            "id": "fixture.catalog",
            "name": "Fixture Catalog",
            "resources": ["catalog"],
            "types": ["movie"],
            "catalogs": [{"type": "movie", "id": "popular", "name": "Popular"}]
        }]))
        .expect("normalize addons");

        let catalogs = core_value(
            "discoverCatalogOptions",
            json!({"addons": addons.to_string(), "selectedType": "movie"}),
        )
        .unwrap_or_else(|| panic!("Core should produce catalog options; normalized={addons:#}"));
        assert_eq!(catalogs.as_array().map_or(0, Vec::len), 1);
    }

    #[test]
    fn home_effect_reads_the_real_cached_bootstrap() {
        let directory = temporary_directory();
        let storage = Storage::open(directory.clone()).expect("open storage");
        let cache_key = format!("home_bootstrap_v1_{}_en", Storage::library_key("guest"));
        storage
            .write_json(
                &cache_key,
                &json!({
                    "categories": [{"id": "catalog", "items": [{"id": "tt1", "name": "Cached title"}]}],
                    "continueWatching": [],
                    "metadataFeeds": [],
                    "billboard": {"id": "tt1", "name": "Cached title"}
                }),
            )
            .expect("write cache");
        let executor = EffectExecutor::new(storage);
        let effect = json!({
            "type": "readHomeBootstrap",
            "payload": {"profileId": "guest", "language": "en", "force": false}
        });
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("create runtime");
        let value = runtime
            .block_on(executor.execute(&effect))
            .expect("execute cached effect");
        assert_eq!(value["stale"], true);
        assert_eq!(value["billboard"]["name"], "Cached title");
        let _ = std::fs::remove_dir_all(directory);
    }

    #[test]
    fn catalog_page_effect_uses_the_shared_catalog_executor_path() {
        let directory = temporary_directory();
        let executor = EffectExecutor::new(Storage::open(directory.clone()).expect("open storage"));
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("create runtime");
        let error = runtime
            .block_on(executor.execute(&json!({
                "type": "fetchCatalogPage",
                "payload": {}
            })))
            .expect_err("missing Core request-plan fields must be rejected");
        assert_eq!(error, "catalog page is missing transportUrl");
        let _ = std::fs::remove_dir_all(directory);
    }

    #[test]
    fn search_effect_uses_core_planning_and_returns_core_search_shape() {
        let directory = temporary_directory();
        let executor = EffectExecutor::new(Storage::open(directory.clone()).expect("open storage"));
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("create runtime");
        let value = runtime
            .block_on(executor.execute(&json!({
                "type": "runSearch",
                "payload": {"query": "  matrix  "}
            })))
            .expect("empty addon inventory should produce an empty search result");
        assert_eq!(value["results"], json!([]));
        assert_eq!(value["categories"], json!([]));
        assert_eq!(value["grouping"]["query"], "matrix");
        assert_eq!(value["grouping"]["totalCount"], 0);
        let _ = std::fs::remove_dir_all(directory);
    }

    #[test]
    fn desktop_search_uses_the_same_core_resource_plan_as_other_hosts() {
        let addons = json!([{
            "transportUrl": "https://search.example",
            "manifest": {
                "id": "fixture.search",
                "name": "Fixture Search",
                "resources": ["catalog"],
                "types": ["movie"],
                "catalogs": [{
                    "type": "movie",
                    "id": "quick_search",
                    "name": "Quick Search",
                    "extra": [{"name": "search", "isRequired": true}]
                }]
            }
        }]);
        let plan = core_value(
            "resourceFetchPlan",
            json!({"kind": "search", "query": "matrix", "addons": addons}),
        )
        .expect("Core must produce a search request plan");
        let requests = plan["requests"]
            .as_array()
            .expect("Core search plan should contain request descriptors");
        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0]["catalogType"], "movie");
        assert_eq!(requests[0]["catalogId"], "quick_search");
        assert!(
            requests[0]["url"]
                .as_str()
                .unwrap_or_default()
                .contains("search=matrix")
        );
    }

    #[test]
    fn refresh_installed_addons_returns_the_shared_addon_result_shape() {
        let directory = temporary_directory();
        let executor = EffectExecutor::new(Storage::open(directory.clone()).expect("open storage"));
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("create runtime");
        let value = runtime
            .block_on(executor.execute(&json!({
                "type": "refreshInstalledAddons",
                "payload": {"profile": {"id": "guest"}, "forceRefresh": true}
            })))
            .expect("empty addon inventory should still return the shared shape");
        assert_eq!(value["addons"], json!([]));
        let _ = std::fs::remove_dir_all(directory);
    }

    #[test]
    fn desktop_library_commands_use_the_core_plan_and_persist_the_result() {
        let directory = temporary_directory();
        let storage = Storage::open(directory.clone()).expect("open storage");
        let executor = EffectExecutor::new(storage.clone());
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("create runtime");
        let added = runtime
            .block_on(executor.execute(&json!({
                "type": "writeLibraryCommand",
                "payload": {
                    "profileId": "guest",
                    "source": "local",
                    "command": {
                        "type": "toggleWatchlist",
                        "item": {"id": "tt-local", "name": "Local title", "type": "movie"}
                    }
                }
            })))
            .expect("local watchlist command should complete");
        assert_eq!(added["isInWatchlist"], true);
        assert_eq!(added["watchlist"][0]["id"], "tt-local");

        let completed = runtime
            .block_on(executor.execute(&json!({
                "type": "writeLibraryCommand",
                "payload": {
                    "profileId": "guest",
                    "source": "local",
                    "command": {
                        "type": "toggleLibraryStatus",
                        "list": "completed",
                        "item": {"id": "tt-local", "name": "Local title", "type": "movie"}
                    }
                }
            })))
            .expect("local status command should complete");
        assert_eq!(completed["watchlist"], json!([]));
        assert_eq!(completed["completed"][0]["id"], "tt-local");
        let persisted = storage
            .read_json(&Storage::library_key("guest"))
            .expect("read persisted library")
            .expect("library should be stored");
        assert_eq!(persisted["completed"][0]["id"], "tt-local");
        assert!(
            persisted["completed"][0]["statusChangedAt"]
                .as_str()
                .is_some()
        );
        let _ = std::fs::remove_dir_all(directory);
    }

    #[test]
    fn desktop_does_not_apply_remote_library_commands_as_local_writes() {
        let directory = temporary_directory();
        let storage = Storage::open(directory.clone()).expect("open storage");
        let executor = EffectExecutor::new(storage.clone());
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("create runtime");
        let error = runtime
            .block_on(executor.execute(&json!({
                "type": "writeLibraryCommand",
                "payload": {
                    "profileId": "guest",
                    "source": "nuvio",
                    "command": {
                        "type": "toggleWatchlist",
                        "item": {"id": "tt-remote", "name": "Remote title", "type": "movie"}
                    }
                }
            })))
            .expect_err("remote writes must not silently become local-only");
        assert!(error.contains("provider 'nuvio'"));
        assert!(
            storage
                .read_json(&Storage::library_key("guest"))
                .expect("read library")
                .is_none()
        );
        let _ = std::fs::remove_dir_all(directory);
    }

    #[test]
    fn desktop_subtitle_effect_uses_the_shared_core_result_contract() {
        let directory = temporary_directory();
        let executor = EffectExecutor::new(Storage::open(directory.clone()).expect("open storage"));
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("create runtime");
        let value = runtime
            .block_on(executor.execute(&json!({
                "type": "fetchSubtitles",
                "payload": {"stream": {"subtitles": [{"lang": "en", "url": "https://sub.example/en.vtt"}]}}
            })))
            .expect("subtitle selection is a local Core operation");
        assert_eq!(value["subtitles"][0]["lang"], "en");
        let _ = std::fs::remove_dir_all(directory);
    }

    #[test]
    fn detail_stream_effect_returns_the_shared_core_result_contract() {
        let directory = temporary_directory();
        let executor = EffectExecutor::new(Storage::open(directory.clone()).expect("open storage"));
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("create runtime");
        let value = runtime
            .block_on(executor.execute(&json!({
                "type": "fetchDetailStreams",
                "payload": {
                    "contentType": "movie",
                    "requestIds": ["tt0133093"],
                    "detail": {"id": "tt0133093", "type": "movie"},
                    "seasonEpisodes": [],
                    "profile": {"id": "guest"}
                }
            })))
            .expect("empty addon inventory should yield an empty detail stream result");
        assert_eq!(value["streams"], json!([]));
        assert_eq!(value["availableAddons"], json!([]));
        assert_eq!(value["resolvedRequestId"], Value::Null);
        assert_eq!(value["hasStreamProviders"], false);
        let _ = std::fs::remove_dir_all(directory);
    }

    #[test]
    fn desktop_detail_local_effects_match_the_android_result_contract() {
        let directory = temporary_directory();
        let executor = EffectExecutor::new(Storage::open(directory.clone()).expect("open storage"));
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("create runtime");

        let progress = runtime
            .block_on(executor.execute(&json!({
                "type": "readPlaybackProgress",
                "payload": {"id": "tt-empty"}
            })))
            .expect("empty saved progress is valid");
        assert_eq!(progress, Value::Null);

        let detail = runtime
            .block_on(executor.execute(&json!({
                "type": "readDetailLocalState",
                "payload": {
                    "primaryId": "tt-empty",
                    "fallbackId": "tmdb:42",
                    "contentType": "series",
                    "profile": {"id": "guest"}
                }
            })))
            .expect("empty local detail state is valid");
        assert_eq!(detail["savedPlayback"], Value::Null);
        assert_eq!(detail["localWatchedVideoIds"], json!([]));
        assert_eq!(detail["isInWatchlist"], false);
        assert_eq!(detail["feedback"], Value::Null);
        assert_eq!(detail["hasStreamProviders"], false);
        assert_eq!(detail["userAddons"], json!([]));
        let _ = std::fs::remove_dir_all(directory);
    }

    #[test]
    fn metadata_effects_return_the_core_lookup_and_detail_shapes() {
        let directory = temporary_directory();
        let executor = EffectExecutor::new(Storage::open(directory.clone()).expect("open storage"));
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("create runtime");
        let payload = json!({"contentType": "movie", "id": "tt0133093"});
        let lookup = runtime
            .block_on(executor.execute(&json!({
                "type": "fetchMetaDetailLookup",
                "payload": payload
            })))
            .expect("lookup without configured providers should complete empty");
        assert_eq!(lookup, Value::Null);
        let detail = runtime
            .block_on(executor.execute(&json!({
                "type": "fetchMetaDetail",
                "payload": {"contentType": "movie", "id": "tt0133093"}
            })))
            .expect("detail without configured providers should complete empty");
        assert_eq!(detail["meta"], Value::Null);
        assert_eq!(detail["mdblistRatings"], Value::Null);
        let _ = std::fs::remove_dir_all(directory);
    }
}
