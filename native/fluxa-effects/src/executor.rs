use crate::storage::{Storage, sanitize_key};
use reqwest::{Client, ClientBuilder};
use serde_json::{Value, json};
use std::sync::OnceLock;
use std::sync::mpsc::Sender;
use std::time::Duration;

mod providers;

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

#[cfg(not(target_arch = "wasm32"))]
static TORRENT_SERVER: OnceLock<std::sync::Mutex<Option<Value>>> = OnceLock::new();

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
        Self { storage }
    }

    #[cfg(target_arch = "wasm32")]
    pub fn warm_torrent_engine(&self) {}

    #[cfg(not(target_arch = "wasm32"))]
    pub fn warm_torrent_engine(&self) {
        std::thread::spawn(|| match ensure_torrent_server() {
            Ok(_) => crate::log!("[fluxa-native] torrent engine warmed up"),
            Err(error) => crate::log!("[fluxa-native] torrent engine warm-up failed: {error}"),
        });
    }

    #[cfg(target_arch = "wasm32")]
    pub fn poll_torrent_status(
        &self,
        _link: String,
        _file_id: Option<usize>,
    ) -> std::sync::mpsc::Receiver<Value> {
        std::sync::mpsc::channel().1
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn poll_torrent_status(
        &self,
        link: String,
        file_id: Option<usize>,
    ) -> std::sync::mpsc::Receiver<Value> {
        let (sender, receiver) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let Ok(server) = ensure_torrent_server() else {
                return;
            };
            let Some(base_url) = server.get("url").and_then(Value::as_str) else {
                return;
            };
            let url = format!("{}/torrents", base_url.trim_end_matches('/'));
            let Ok(runtime) = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
            else {
                return;
            };
            let client = Client::new();
            loop {
                let response: Result<Value, String> = runtime
                    .block_on(async {
                        client
                            .post(&url)
                            .json(&json!({ "action": "get", "link": link, "file_id": file_id }))
                            .timeout(Duration::from_secs(3))
                            .send()
                            .await?
                            .error_for_status()?
                            .json::<Value>()
                            .await
                    })
                    .map_err(|error: reqwest::Error| error.to_string());
                let status = response
                    .unwrap_or_else(|error| json!({"phase": "status_unavailable", "error": error}));
                if sender.send(status).is_err() {
                    break;
                }
                std::thread::sleep(Duration::from_millis(700));
            }
        });
        receiver
    }

    pub fn fetch_json(&self, url: String) -> std::sync::mpsc::Receiver<Option<Value>> {
        let (sender, receiver) = std::sync::mpsc::channel();
        let task = async move {
            let response = async {
                Client::builder()
                    .native_timeout(Duration::from_secs(10))
                    .build()?
                    .get(&url)
                    .header("User-Agent", "Fluxa")
                    .send()
                    .await?
                    .error_for_status()?
                    .json::<Value>()
                    .await
            }
            .await;
            let _ = sender.send(response.ok());
        };
        #[cfg(target_arch = "wasm32")]
        wasm_bindgen_futures::spawn_local(task);
        #[cfg(not(target_arch = "wasm32"))]
        std::thread::spawn(move || {
            if let Ok(runtime) = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
            {
                runtime.block_on(task);
            }
        });
        receiver
    }

    pub fn fetch_trailers(
        &self,
        items: Vec<Value>,
        api_key: String,
        language: String,
    ) -> std::sync::mpsc::Receiver<Vec<(String, Value)>> {
        let (sender, receiver) = std::sync::mpsc::channel();
        let task = async move {
            let mut found = Vec::new();
            if let Ok(client) = Client::builder()
                .native_timeout(Duration::from_secs(10))
                .build()
            {
                for item in items {
                    let Some(id) = item.get("id").and_then(Value::as_str) else {
                        continue;
                    };
                    let trailers = tmdb_trailers(&client, &item, &api_key, &language).await;
                    found.push((id.to_owned(), trailers.unwrap_or_else(|| json!([]))));
                }
            }
            let _ = sender.send(found);
        };
        #[cfg(target_arch = "wasm32")]
        wasm_bindgen_futures::spawn_local(task);
        #[cfg(not(target_arch = "wasm32"))]
        std::thread::spawn(move || {
            if let Ok(runtime) = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
            {
                runtime.block_on(task);
            }
        });
        receiver
    }

    pub fn fetch_similar(
        &self,
        item: Value,
        api_key: String,
        language: String,
        endpoints: Vec<&'static str>,
    ) -> std::sync::mpsc::Receiver<Vec<Value>> {
        let (sender, receiver) = std::sync::mpsc::channel();
        let task = async move {
            let items = match Client::builder()
                .native_timeout(Duration::from_secs(10))
                .build()
            {
                Ok(client) => tmdb_similar(&client, &item, &api_key, &language, &endpoints)
                    .await
                    .unwrap_or_default(),
                Err(_) => Vec::new(),
            };
            let _ = sender.send(items);
        };
        #[cfg(target_arch = "wasm32")]
        wasm_bindgen_futures::spawn_local(task);
        #[cfg(not(target_arch = "wasm32"))]
        std::thread::spawn(move || {
            if let Ok(runtime) = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
            {
                runtime.block_on(task);
            }
        });
        receiver
    }

    pub fn fetch_trending(&self, api_key: String) -> std::sync::mpsc::Receiver<Vec<Value>> {
        detached(async move {
            let Ok(client) = Client::builder().native_timeout(Duration::from_secs(10)).build() else {
                return Vec::new();
            };
            let mut results = Vec::new();
            for page in 1..=2 {
                let url = format!(
                    "https://api.themoviedb.org/3/trending/all/week?api_key={api_key}&page={page}"
                );
                let Ok(body) = fetch_json(&client, &url).await else {
                    break;
                };
                results.extend(body.get("results").and_then(Value::as_array).cloned().unwrap_or_default());
            }
            results
        })
    }

    pub fn fetch_mdblist_media(
        &self,
        api_key: String,
        media_type: &'static str,
        ids: Vec<String>,
    ) -> std::sync::mpsc::Receiver<Vec<Value>> {
        detached(async move {
            let Some(plan) = core_value(
                "mdblistMediaInfoBatchPlan",
                json!({"provider": "imdb", "mediaType": media_type, "ids": ids}),
            ) else {
                return Vec::new();
            };
            let Some(url) = plan.get("url").and_then(Value::as_str) else {
                return Vec::new();
            };
            let Ok(client) = Client::builder().native_timeout(Duration::from_secs(15)).build() else {
                return Vec::new();
            };
            let response = client
                .post(format!("{url}?apikey={api_key}"))
                .header("Content-Type", "application/json")
                .body(plan.get("body").cloned().unwrap_or_default().to_string())
                .send()
                .await;
            let Ok(response) = response.and_then(|response| response.error_for_status()) else {
                return Vec::new();
            };
            response
                .json::<Value>()
                .await
                .ok()
                .and_then(|body| body.as_array().cloned())
                .unwrap_or_default()
        })
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
        std::thread::spawn(move || {
            match tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
            {
                Ok(runtime) => runtime.block_on(task),
                Err(error) => crate::log!("[fluxa-effects] tokio runtime failed: {error}"),
            }
        });
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
            fluxa_core::runtime::EffectKind::ReadCalendarMonth => {
                self.read_calendar_month(&payload)
            }
            fluxa_core::runtime::EffectKind::WriteSettings => self.write_settings(&payload),
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

    async fn read_home_bootstrap(&self, payload: &Value) -> Result<Value, String> {
        let language = payload
            .get("language")
            .and_then(Value::as_str)
            .unwrap_or("en");
        let profile_id = payload
            .get("profileId")
            .and_then(Value::as_str)
            .unwrap_or("guest");
        let cache_key = format!(
            "home_bootstrap_v1_{}_{}",
            Storage::library_key(profile_id),
            sanitize_key(language)
        );
        if payload.get("force").and_then(Value::as_bool) != Some(true) {
            if let Some(Value::Object(mut cached)) = self.storage.read_json(&cache_key)? {
                cached.insert("stale".to_owned(), Value::Bool(true));
                return Ok(Value::Object(cached));
            }
        }

        let profiles = self
            .storage
            .read_json("profiles")?
            .unwrap_or_else(|| json!([]));
        let active_id = self
            .storage
            .read_json("active_profile_id")?
            .and_then(|value| value.as_str().map(ToOwned::to_owned))
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| profile_id.to_owned());
        let profile = profiles
            .as_array()
            .and_then(|items| {
                items
                    .iter()
                    .find(|item| item.get("id").and_then(Value::as_str) == Some(active_id.as_str()))
            })
            .cloned()
            .unwrap_or_else(|| json!({}));
        let prefs = self
            .storage
            .read_json(&Storage::prefs_key(&active_id))?
            .or_else(|| self.storage.read_json("prefs").ok().flatten())
            .unwrap_or_else(|| json!({}));
        let profiles_request = json!({"profiles": profiles, "activeProfileId": active_id});
        let owner = core_value("effectiveAddonsOwnerId", profiles_request)
            .and_then(|value| value.as_str().map(ToOwned::to_owned))
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| active_id.clone());
        let mut addons = self
            .storage
            .read_json(&Storage::addons_key(&owner))?
            .or_else(|| {
                self.storage
                    .read_json(&Storage::addons_key(&active_id))
                    .ok()
                    .flatten()
            })
            .unwrap_or_else(|| json!([]));
        if !addons.is_array() {
            addons = json!([]);
        }
        addons = normalize_enabled_addons(addons)?;
        if let Some(api_key) = prefs
            .get("tmdbApiKey")
            .and_then(Value::as_str)
            .filter(|key| !key.trim().is_empty())
        {
            if let Some(manifest) = core_value("tmdbBuiltinManifest", json!({})).and_then(|value| {
                value
                    .as_str()
                    .and_then(|raw| serde_json::from_str::<Value>(raw).ok())
            }) {
                if let Some(items) = addons.as_array_mut() {
                    let descriptor = json!({"transportUrl": "tmdb://builtin", "manifest": manifest, "apiKey": api_key});
                    if prefs.get("tmdbPreferOverAddons").and_then(Value::as_bool) == Some(true) {
                        items.insert(0, descriptor);
                    } else {
                        items.push(descriptor);
                    }
                }
            }
        }

        // Discover already resolves add-ons from the signed-in Nuvio account,
        // but Home used only the local add-on cache. Consequently the user's
        // collection shelves loaded while their catalog shelves (and catalog-
        // sourced hero candidates) silently vanished. Merge the same remote
        // manifests into this bootstrap before deriving metadata feeds.
        self.merge_nuvio_addons(&profile, &mut addons).await;

        let addon_json = addons.to_string();
        let feeds =
            core_value("buildMetadataFeedOptions", addons.clone()).unwrap_or_else(|| json!([]));
        let feed_count = feeds.as_array().map_or(0, Vec::len);
        let visible_feeds = core_value(
            "homeMetadataFeedPlan",
            json!({
                "feeds": feeds,
                "selectedKeys": prefs.get("homeFeedToggles").unwrap_or(&Value::Null),
                "order": prefs.get("homeFeedOrder").unwrap_or(&Value::Null),
            }),
        )
        .and_then(|value| value.as_array().cloned())
        .ok_or_else(|| "Fluxa Core could not resolve selected home metadata feeds".to_owned())?;
        let metadata_feeds = visible_feeds.clone();
        crate::log!(
            "[fluxa-native] home bootstrap: addons={} feeds={} visible_feeds={}",
            addons.as_array().map_or(0, Vec::len),
            feed_count,
            visible_feeds.len()
        );

        let client = Client::builder()
            .user_agent("Fluxa/1.0")
            .native_timeout(Duration::from_secs(20))
            .build()
            .map_err(|error| error.to_string())?;
        let mut categories = Vec::new();
        for feed in visible_feeds {
            let content_type = feed.get("type").and_then(Value::as_str).unwrap_or("movie");
            let catalog_id = feed.get("id").and_then(Value::as_str).unwrap_or_default();
            let transport_url = feed
                .get("transportUrl")
                .and_then(Value::as_str)
                .unwrap_or_default();
            let extra = json!({});
            let data = if transport_url == "tmdb://builtin" {
                let Some(api_key) = prefs.get("tmdbApiKey").and_then(Value::as_str) else {
                    continue;
                };
                let url = core_value(
                    "tmdbBuiltinCatalogUrl",
                    json!({"contentType": content_type, "extra": extra, "apiKey": api_key, "language": language}),
                )
                .and_then(|value| value.as_str().map(ToOwned::to_owned));
                let Some(url) = url else { continue };
                let response = match fetch_json(&client, &url).await {
                    Ok(response) => response,
                    Err(error) => {
                        crate::log!(
                            "[fluxa-native] TMDB catalog request failed for {}: {error}",
                            url_without_query(&url)
                        );
                        continue;
                    }
                };
                let results = response
                    .get("results")
                    .cloned()
                    .unwrap_or_else(|| json!([]));
                core_value(
                    "tmdbBulkMetas",
                    json!({"itemsJson": results.to_string(), "requestedType": content_type, "language": language}),
                )
                .unwrap_or_else(|| json!([]))
            } else {
                let Some(url) = core_value(
                    "buildResourceUrl",
                    json!({
                        "transportUrl": transport_url,
                        "resource": "catalog",
                        "contentType": content_type,
                        "id": catalog_id,
                        "extraJson": extra.to_string()
                    }),
                )
                .and_then(|value| value.as_str().map(ToOwned::to_owned)) else {
                    continue;
                };
                let (status_code, body) = match fetch_text(&client, &url).await {
                    Ok(response) => response,
                    Err(error) => {
                        crate::log!(
                            "[fluxa-native] catalog request failed for {}: {error}",
                            url_without_query(&url)
                        );
                        continue;
                    }
                };
                match parse_catalog_items(&url, status_code, body.as_deref(), "discover") {
                    Ok(items) => Value::Array(items),
                    Err(error) => {
                        crate::log!("[fluxa-native] catalog response rejected by Core: {error}");
                        continue;
                    }
                }
            };
            let items = annotate_catalog_items(
                data.as_array().map(Vec::as_slice).unwrap_or_default(),
                transport_url,
                content_type,
            )?;
            if items.is_empty() {
                continue;
            }
            let label = feed
                .get("label")
                .and_then(Value::as_str)
                .unwrap_or("Catalog");
            let home_title = feed
                .get("homeTitle")
                .and_then(Value::as_str)
                .unwrap_or(label);
            categories.push(json!({
                "id": feed.get("key"),
                "name": home_title,
                "semanticName": home_title,
                "type": content_type,
                "items": items,
                "addonName": label.split(" - ").next().unwrap_or(label),
                "transportUrl": transport_url,
                "catalogId": catalog_id,
                "canLoadMore": true
            }));
        }
        crate::log!(
            "[fluxa-native] home bootstrap: populated_catalog_shelves={}",
            categories.len()
        );

        // The Web home pipeline hydrates Nuvio's cloud collections before
        // asking Core to build collection shelves. Do the same here instead
        // of silently rendering the profile's stale local copy.
        let mut collection_profile = profile.clone();
        if let (Some(base_url), Some(api_key), Some(token)) = (
            option_env!("FLUXA_NUVIO_SUPABASE_URL"),
            option_env!("FLUXA_NUVIO_SUPABASE_KEY"),
            profile
                .get("nuvioAccessToken")
                .and_then(Value::as_str)
                .filter(|token| !token.is_empty()),
        ) {
            let profile_index = profile
                .get("nuvioProfileIndex")
                .and_then(Value::as_i64)
                .filter(|index| *index > 0)
                .unwrap_or(1);
            let endpoint = format!(
                "{}/rest/v1/rpc/sync_pull_collections",
                base_url.trim_end_matches('/')
            );
            let remote = client
                .post(endpoint)
                .header("apikey", api_key)
                .bearer_auth(token)
                .json(&json!({"p_profile_id": profile_index}))
                .send()
                .await
                .and_then(reqwest::Response::error_for_status);
            if let Ok(response) = remote
                && let Ok(rows) = response.json::<Value>().await
                && let Some(collections) = rows
                    .as_array()
                    .and_then(|rows| rows.first())
                    .and_then(|row| row.get("collections_json"))
                    .and_then(Value::as_array)
                && let Some(mapped) = core_value(
                    "nuvioMapCollections",
                    json!({"collections": collections, "profileIndex": profile_index}),
                )
                && let Some(fields) = collection_profile.as_object_mut()
            {
                fields.insert("libraryCollections".to_owned(), mapped);
            }
        }
        let shelves = core_value(
            "buildHomeCollectionShelves",
            json!({"profileJson": collection_profile.to_string(), "addonsJson": addon_json}),
        )
        .unwrap_or_else(
            || json!({"pinnedShelves": [], "regularShelves": [], "hiddenFolderCategories": []}),
        );
        let mut all_categories = shelves
            .get("pinnedShelves")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        all_categories.extend(categories);
        all_categories.extend(
            shelves
                .get("regularShelves")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default(),
        );
        all_categories.extend(
            shelves
                .get("hiddenFolderCategories")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default(),
        );
        let continue_watching = self
            .continue_watching_for_source(&active_id, &profile, &prefs, None)
            .await?;
        let billboard = all_categories.iter().find_map(|category| {
            category
                .get("items")
                .and_then(Value::as_array)
                .and_then(|items| items.first())
                .cloned()
        });
        if all_categories.is_empty() {
            if let Some(Value::Object(mut cached)) = self.storage.read_json(&cache_key)? {
                cached.insert("stale".to_owned(), Value::Bool(false));
                return Ok(Value::Object(cached));
            }
        }
        let bootstrap = json!({
            "categories": all_categories,
            "continueWatching": continue_watching,
            "metadataFeeds": metadata_feeds,
            "billboard": billboard
        });
        self.storage.write_json(&cache_key, &bootstrap)?;
        Ok(bootstrap)
    }

    async fn nuvio_addons_profile_index(
        &self,
        client: &Client,
        base_url: &str,
        api_key: &str,
        token: &str,
        profile_index: i64,
    ) -> i64 {
        if profile_index == 1 {
            return 1;
        }
        let response = client
            .post(format!(
                "{}/rest/v1/rpc/sync_pull_profiles",
                base_url.trim_end_matches('/')
            ))
            .header("apikey", api_key)
            .bearer_auth(token)
            .json(&json!({}))
            .send()
            .await;
        let remote_profiles = match response {
            Ok(response) if response.status().is_success() => {
                match response.json::<Vec<Value>>().await {
                    Ok(profiles) => profiles,
                    Err(error) => {
                        crate::log!("[fluxa-native] Nuvio profile scopes decode failed: {error}");
                        return profile_index;
                    }
                }
            }
            Ok(response) => {
                crate::log!(
                    "[fluxa-native] Nuvio profile scopes request failed: HTTP {}",
                    response.status()
                );
                return profile_index;
            }
            Err(error) => {
                crate::log!("[fluxa-native] Nuvio profile scopes request failed: {error}");
                return profile_index;
            }
        };
        core_value(
            "nuvioEffectiveProfileScopes",
            json!({"profileIndex": profile_index, "profiles": remote_profiles}),
        )
        .and_then(|scopes| scopes.get("addons").and_then(Value::as_i64))
        .unwrap_or(profile_index)
    }

    async fn merge_nuvio_addons(&self, profile: &Value, addons: &mut Value) {
        let (Some(base_url), Some(api_key), Some(token)) = (
            option_env!("FLUXA_NUVIO_SUPABASE_URL"),
            option_env!("FLUXA_NUVIO_SUPABASE_KEY"),
            profile
                .get("nuvioAccessToken")
                .and_then(Value::as_str)
                .filter(|token| !token.is_empty()),
        ) else {
            return;
        };
        let profile_index = profile
            .get("nuvioProfileIndex")
            .and_then(Value::as_i64)
            .filter(|index| *index > 0)
            .unwrap_or(1);
        let client = match Client::builder()
            .user_agent("Fluxa/1.0")
            .native_timeout(Duration::from_secs(12))
            .build()
        {
            Ok(client) => client,
            Err(error) => {
                crate::log!("[fluxa-native] Nuvio add-on client unavailable: {error}");
                return;
            }
        };
        let addon_profile_index = self
            .nuvio_addons_profile_index(&client, base_url, api_key, token, profile_index)
            .await;
        let endpoint = format!(
            "{}/rest/v1/addons?select=*&profile_id=eq.{}&order=sort_order",
            base_url.trim_end_matches('/'),
            addon_profile_index
        );
        let remote_addons = match client
            .get(endpoint)
            .header("apikey", api_key)
            .bearer_auth(token)
            .send()
            .await
            .and_then(reqwest::Response::error_for_status)
        {
            Ok(response) => match response.json::<Vec<Value>>().await {
                Ok(addons) => addons,
                Err(error) => {
                    crate::log!("[fluxa-native] Nuvio add-on list response invalid: {error}");
                    return;
                }
            },
            Err(error) => {
                crate::log!("[fluxa-native] Nuvio add-on list request failed: {error}");
                return;
            }
        };
        let Some(items) = addons.as_array_mut() else {
            return;
        };
        for remote in remote_addons {
            if remote.get("enabled").and_then(Value::as_bool) == Some(false) {
                continue;
            }
            let Some(url) = remote.get("url").and_then(Value::as_str) else {
                continue;
            };
            if items
                .iter()
                .any(|item| item.get("transportUrl").and_then(Value::as_str) == Some(url))
            {
                continue;
            }
            let manifest = match client.get(url).send().await {
                Ok(response) => match response.error_for_status() {
                    Ok(response) => match response.json::<Value>().await {
                        Ok(manifest) => manifest,
                        Err(error) => {
                            crate::log!("[fluxa-native] Nuvio add-on manifest invalid: {error}");
                            continue;
                        }
                    },
                    Err(error) => {
                        crate::log!("[fluxa-native] Nuvio add-on manifest request failed: {error}");
                        continue;
                    }
                },
                Err(error) => {
                    crate::log!("[fluxa-native] Nuvio add-on manifest request failed: {error}");
                    continue;
                }
            };
            let descriptor = json!({
                "transportUrl": url,
                "manifest": manifest,
                "name": remote.get("name").cloned().unwrap_or(Value::Null),
                "enabled": true,
            });
            if let Some(descriptor) = core_value(
                "normalizeAddonDescriptor",
                json!({"addonJson": descriptor.to_string()}),
            ) {
                items.push(descriptor);
            }
        }
    }

    fn start_torrent_stream(&self, payload: &Value) -> Result<Value, String> {
        let stream = payload
            .get("stream")
            .filter(|value| value.is_object())
            .cloned()
            .ok_or_else(|| "torrent stream payload is missing stream".to_owned())?;
        let link = payload
            .get("url")
            .and_then(Value::as_str)
            .filter(|value| !value.trim().is_empty())
            .ok_or_else(|| "torrent stream payload is missing url".to_owned())?;
        let title = payload
            .get("title")
            .and_then(Value::as_str)
            .filter(|value| !value.trim().is_empty())
            .unwrap_or("Fluxa stream");
        let server = ensure_torrent_server()?;
        let base_url = server
            .get("url")
            .and_then(Value::as_str)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| "torrent server did not return url".to_owned())?;
        let runtime_request = json!({
            "link": link,
            "title": title,
            "requestedFileIdx": payload.get("fileIdx").cloned().unwrap_or(Value::Null),
            "preferredFilename": payload.get("preferredFilename").cloned().unwrap_or(Value::Null),
            "sources": payload.get("sources").cloned().unwrap_or_else(|| json!([])),
            "fileStats": [],
            "rejectedIndex": null,
            "baseUrl": base_url,
            "play": true,
            "stat": false,
            "durationMs": null,
        });
        let runtime = core_value("torrentRuntimeInfo", runtime_request)
            .ok_or_else(|| "torrent runtime info could not be resolved".to_owned())?;
        let stream_url = runtime
            .get("streamUrl")
            .and_then(Value::as_str)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| "torrent runtime did not return streamUrl".to_owned())?;
        let normalized_link = runtime
            .get("normalizedLink")
            .and_then(Value::as_str)
            .unwrap_or(link)
            .to_owned();
        let file_id = runtime.get("selectedFileIdx").and_then(Value::as_i64);
        crate::log!(
            "[fluxa-native] torrent stream ready base={} url={}",
            base_url,
            stream_url
        );
        start_torrent_add(base_url.to_owned(), normalized_link, file_id);
        Ok(json!({
            "url": stream_url,
            "baseUrl": base_url,
            "generation": server.get("generation").cloned().unwrap_or(Value::Null),
            "stream": stream,
        }))
    }

    async fn refresh_continue_watching(&self, payload: &Value) -> Result<Value, String> {
        let profile_id = payload
            .get("profileId")
            .and_then(Value::as_str)
            .unwrap_or("guest");
        let profile = match payload.get("profile").filter(|profile| profile.is_object()) {
            Some(profile) => profile.clone(),
            None => self
                .storage
                .read_json("profiles")?
                .and_then(|profiles| {
                    profiles
                        .as_array()?
                        .iter()
                        .find(|item| item.get("id").and_then(Value::as_str) == Some(profile_id))
                        .cloned()
                })
                .unwrap_or_else(|| json!({})),
        };
        let prefs = self
            .storage
            .read_json(&Storage::prefs_key(profile_id))?
            .or_else(|| self.storage.read_json("prefs").ok().flatten())
            .unwrap_or_else(|| json!({}));
        let source = payload.get("source").and_then(Value::as_str);
        let items = self
            .continue_watching_for_source(profile_id, &profile, &prefs, source)
            .await?;
        Ok(json!({"continueWatching": items}))
    }

    async fn continue_watching_for_source(
        &self,
        profile_id: &str,
        profile: &Value,
        prefs: &Value,
        source: Option<&str>,
    ) -> Result<Value, String> {
        let nuvio_connected = profile
            .get("nuvioAccessToken")
            .and_then(Value::as_str)
            .is_some_and(|token| !token.is_empty());
        let requested = source
            .or_else(|| prefs.get("continueWatchingSource")?.as_str())
            .unwrap_or("local");
        let provider = core_value(
            "continueWatchingSourcePlan",
            json!({"source": requested, "nuvioConnected": nuvio_connected}),
        )
        .and_then(|plan| plan.get("provider")?.as_str().map(ToOwned::to_owned));
        match provider.as_deref() {
            Some("nuvio") => {
                let snapshot = self
                    .read_nuvio_library(profile_id, Some(profile).filter(|p| p.is_object()))
                    .await
                    .inspect_err(|error| {
                        crate::log!("[fluxa-native] Nuvio continue watching failed: {error}")
                    })
                    .ok()
                    .flatten();
                return Ok(snapshot
                    .and_then(|snapshot| snapshot.get("continueWatching").cloned())
                    .unwrap_or_else(|| json!([])));
            }
            Some(other) if providers::is_provider(other) => {
                let snapshot = self
                    .read_provider_library(other, profile_id, profile)
                    .await
                    .inspect_err(|error| {
                        crate::log!("[fluxa-native] {other} continue watching failed: {error}")
                    })
                    .ok();
                return Ok(snapshot
                    .and_then(|snapshot| snapshot.get("continueWatching").cloned())
                    .unwrap_or_else(|| json!([])));
            }
            Some(other) => {
                crate::log!(
                    "[fluxa-native] continue watching source '{other}' has no native client yet"
                );
                return Ok(json!([]));
            }
            None => {}
        }
        let library = self
            .storage
            .read_json(&Storage::library_key(profile_id))?
            .unwrap_or_else(|| json!({}));
        let progress = library
            .get("progress")
            .cloned()
            .unwrap_or_else(|| json!({}));
        Ok(core_value("buildContinueWatchingFromProgress", progress).unwrap_or_else(|| json!([])))
    }

    async fn read_library_state(&self, payload: &Value) -> Result<Value, String> {
        let profile_id = payload
            .get("profileId")
            .and_then(Value::as_str)
            .filter(|value| !value.is_empty())
            .unwrap_or("guest");
        let effect_profile = payload.get("profile").filter(|profile| profile.is_object());
        let source = payload
            .get("source")
            .and_then(Value::as_str)
            .or_else(|| effect_profile?.get("integrationLibrarySource")?.as_str())
            .unwrap_or("local");
        if source == "nuvio" {
            return self
                .read_nuvio_library(profile_id, effect_profile)
                .await?
                .ok_or_else(|| "Nuvio account is not connected to the active profile".to_owned());
        }
        if providers::is_provider(source) {
            let profile = match effect_profile {
                Some(profile) => profile.clone(),
                None => self.stored_profile(profile_id),
            };
            let snapshot = self.read_provider_library(source, profile_id, &profile).await?;
            return core_value("normalizeLibraryDocument", snapshot)
                .ok_or_else(|| "Fluxa Core could not normalize the library document".to_owned());
        }
        let stored_library = self
            .storage
            .read_json(&Storage::library_key(profile_id))?
            .or_else(|| self.storage.read_json("library").ok().flatten())
            .unwrap_or_else(|| json!({}));
        let should_derive_continue_watching = stored_library
            .get("continueWatching")
            .is_none_or(Value::is_null);
        let mut library = core_value("normalizeLibraryDocument", stored_library)
            .ok_or_else(|| "Fluxa Core could not normalize the library document".to_owned())?;
        if should_derive_continue_watching {
            let progress = library
                .get("progress")
                .cloned()
                .unwrap_or_else(|| json!({}));
            let continue_watching = core_value("buildContinueWatchingFromProgress", progress)
                .ok_or_else(|| "Fluxa Core could not build continue watching items".to_owned())?;
            if let Some(object) = library.as_object_mut() {
                object.insert("continueWatching".to_owned(), continue_watching);
            }
        }
        Ok(library)
    }

    fn stored_profile(&self, profile_id: &str) -> Value {
        self.storage
            .read_json("profiles")
            .ok()
            .flatten()
            .and_then(|profiles| {
                profiles
                    .as_array()?
                    .iter()
                    .find(|item| item.get("id").and_then(Value::as_str) == Some(profile_id))
                    .cloned()
            })
            .unwrap_or_else(|| json!({}))
    }

    async fn write_library_command(&self, payload: &Value) -> Result<Value, String> {
        let profile_id = payload
            .get("profileId")
            .and_then(Value::as_str)
            .filter(|value| !value.is_empty())
            .unwrap_or("guest");
        let source = payload
            .get("source")
            .and_then(Value::as_str)
            .unwrap_or("local")
            .trim()
            .to_lowercase();
        let provider = providers::is_provider(&source).then_some(source.as_str());
        if provider.is_none() && !matches!(source.as_str(), "" | "local" | "fluxa") {
            return Err(format!(
                "desktop library writes for provider '{source}' are not implemented"
            ));
        }
        let command = payload
            .get("command")
            .filter(|value| value.is_object())
            .ok_or_else(|| "library command is missing".to_owned())?;
        let key = Storage::library_key(profile_id);
        let profile = self.stored_profile(profile_id);
        let library = match provider {
            Some(provider) => match self.cached_provider_library(provider, profile_id) {
                Some(library) => library,
                None => self.read_provider_library(provider, profile_id, &profile).await?,
            },
            None => self
                .storage
                .read_json(&key)?
                .or_else(|| self.storage.read_json("library").ok().flatten())
                .unwrap_or_else(|| json!({})),
        };
        let library = core_value("normalizeLibraryDocument", library)
            .ok_or_else(|| "Fluxa Core could not normalize the library document".to_owned())?;
        let plan = core_value(
            "libraryCommandPlan",
            json!({
                "library": library,
                "command": command,
                "nowIso": chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
            }),
        )
        .ok_or_else(|| "Fluxa Core could not plan the library command".to_owned())?;
        let updated = plan
            .get("library")
            .filter(|value| value.is_object())
            .cloned()
            .ok_or_else(|| "Fluxa Core returned an invalid library command plan".to_owned())?;
        match provider {
            Some(provider) => {
                let mut remote = command.clone();
                if let Some(id) = command.pointer("/item/id").and_then(Value::as_str) {
                    remote["remove"] = json!(!in_watchlist(&updated, id));
                }
                self.push_provider_command(provider, &profile, &remote).await?;
                self.store_provider_library(provider, profile_id, &updated);
            }
            None => self.storage.write_json(&key, &updated)?,
        }
        let kind = command
            .get("type")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let result = match kind {
            "toggleWatchlist" => {
                let id = command
                    .pointer("/item/id")
                    .and_then(Value::as_str)
                    .unwrap_or_default();
                let is_in_watchlist = in_watchlist(&updated, id);
                json!({
                    "watchlist": updated.get("watchlist").cloned().unwrap_or_else(|| json!([])),
                    "isInWatchlist": is_in_watchlist,
                })
            }
            "toggleLibraryStatus" => json!({
                "watchlist": updated.get("watchlist").cloned().unwrap_or_else(|| json!([])),
                "completed": updated.get("completed").cloned().unwrap_or_else(|| json!([])),
                "dropped": updated.get("dropped").cloned().unwrap_or_else(|| json!([])),
            }),
            "markWatched" => {
                let watched = command
                    .get("watched")
                    .and_then(Value::as_bool)
                    .unwrap_or(true);
                let local_watched_video_ids = command
                    .get("videoIds")
                    .and_then(Value::as_array)
                    .into_iter()
                    .flatten()
                    .filter_map(Value::as_str)
                    .filter(|id| {
                        watched
                            && updated
                                .pointer(&format!("/watched/{id}"))
                                .and_then(Value::as_bool)
                                == Some(true)
                    })
                    .collect::<Vec<_>>();
                json!({
                    "watchlist": updated.get("watchlist").cloned().unwrap_or_else(|| json!([])),
                    "localWatchedVideoIds": local_watched_video_ids,
                })
            }
            _ => return Err(format!("unsupported library command: {kind}")),
        };
        Ok(result)
    }

    async fn read_playback_progress(&self, payload: &Value) -> Result<Value, String> {
        let id = payload
            .get("id")
            .and_then(Value::as_str)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| "playback progress request is missing id".to_owned())?;
        let profile_id = self.active_profile_id(None)?;
        let library = self
            .read_library_state(&json!({"profileId": profile_id}))
            .await?;
        Ok(library
            .get("progress")
            .and_then(|progress| progress.get(id))
            .map(progress_meta)
            .unwrap_or(Value::Null))
    }

    async fn read_detail_local_state(&self, payload: &Value) -> Result<Value, String> {
        let profile = payload.get("profile").filter(|value| value.is_object());
        let profile_id = self.active_profile_id(profile)?;
        let library = self
            .read_library_state(&json!({"profileId": profile_id, "profile": profile}))
            .await?;
        let primary_id = payload
            .get("primaryId")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let fallback_id = payload.get("fallbackId").and_then(Value::as_str);
        let progress = [Some(primary_id), fallback_id]
            .into_iter()
            .flatten()
            .find_map(|id| library.get("progress").and_then(|items| items.get(id)));
        let ids = [Some(primary_id), fallback_id]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>();
        let is_in_watchlist = library
            .get("watchlist")
            .and_then(Value::as_array)
            .is_some_and(|items| {
                items.iter().any(|item| {
                    item.get("id")
                        .and_then(Value::as_str)
                        .is_some_and(|id| ids.contains(&id))
                })
            });
        let local_watched_video_ids = if payload
            .get("contentType")
            .and_then(Value::as_str)
            .is_some_and(|value| matches!(value, "series" | "tv" | "anime"))
        {
            library
                .get("watched")
                .and_then(Value::as_object)
                .map(|watched| {
                    watched
                        .iter()
                        .filter(|(id, value)| {
                            value.as_bool() == Some(true)
                                && ids
                                    .iter()
                                    .any(|series_id| id.starts_with(&format!("{series_id}:")))
                        })
                        .map(|(id, _)| Value::String(id.clone()))
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default()
        } else {
            Vec::new()
        };
        let addons = self.load_enabled_addons()?;
        let provider_availability = core_value(
            "providerAvailabilityPlan",
            json!({"addons": addons.clone(), "pluginNames": []}),
        )
        .ok_or_else(|| "Fluxa Core could not resolve stream providers".to_owned())?;
        let feedback = library
            .get("feedback")
            .and_then(Value::as_object)
            .and_then(|values| ids.iter().find_map(|id| values.get(*id)))
            .cloned()
            .unwrap_or(Value::Null);

        Ok(json!({
            "savedPlayback": progress.map(progress_meta).unwrap_or(Value::Null),
            "localWatchedVideoIds": local_watched_video_ids,
            "isInWatchlist": is_in_watchlist,
            "feedback": feedback,
            "hasStreamProviders": provider_availability.get("hasStreamProviders").and_then(Value::as_bool).unwrap_or(false),
            "userAddons": addons
        }))
    }

    fn active_profile_id(&self, profile: Option<&Value>) -> Result<String, String> {
        if let Some(id) = profile
            .and_then(|value| value.get("id"))
            .and_then(Value::as_str)
            .filter(|value| !value.is_empty())
        {
            return Ok(id.to_owned());
        }
        if let Some(id) = self
            .storage
            .read_json("active_profile_id")?
            .and_then(|value| value.as_str().map(ToOwned::to_owned))
            .filter(|value| !value.is_empty())
        {
            return Ok(id);
        }
        Ok("guest".to_owned())
    }

    async fn read_nuvio_library(
        &self,
        profile_id: &str,
        effect_profile: Option<&Value>,
    ) -> Result<Option<Value>, String> {
        let Some(base_url) = option_env!("FLUXA_NUVIO_SUPABASE_URL") else {
            return Err("Nuvio API is not configured in this native build".to_owned());
        };
        let Some(api_key) = option_env!("FLUXA_NUVIO_SUPABASE_KEY") else {
            return Err("Nuvio API is not configured in this native build".to_owned());
        };
        let profiles = self
            .storage
            .read_json("profiles")?
            .unwrap_or_else(|| json!([]));
        let stored_profile = profiles.as_array().and_then(|items| {
            items
                .iter()
                .find(|item| item.get("id").and_then(Value::as_str) == Some(profile_id))
        });
        let Some(profile) = effect_profile.or(stored_profile) else {
            return Err(format!(
                "Nuvio library could not find the active profile ({profile_id})"
            ));
        };
        let Some(mut token) = profile
            .get("nuvioAccessToken")
            .and_then(Value::as_str)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned)
        else {
            return Err(
                "Nuvio library is selected, but this profile has no Nuvio session".to_owned(),
            );
        };

        let client = Client::builder()
            .native_timeout(Duration::from_secs(20))
            .build()
            .map_err(|error| error.to_string())?;
        let expires_at = profile
            .get("nuvioTokenExpiresAt")
            .and_then(Value::as_i64)
            .unwrap_or_default();
        if expires_at <= chrono_unix_seconds() + 60 {
            if let Some(refresh_token) = profile
                .get("nuvioRefreshToken")
                .and_then(Value::as_str)
                .filter(|value| !value.is_empty())
            {
                let refreshed = client
                    .post(format!(
                        "{}/auth/v1/token?grant_type=refresh_token",
                        base_url.trim_end_matches('/')
                    ))
                    .header("apikey", api_key)
                    .json(&json!({"refresh_token": refresh_token}))
                    .send()
                    .await
                    .map_err(|error| format!("Nuvio session refresh failed: {error}"))?
                    .error_for_status()
                    .map_err(|error| format!("Nuvio session refresh failed: {error}"))?
                    .json::<Value>()
                    .await
                    .map_err(|error| {
                        format!("Nuvio session refresh response was invalid: {error}")
                    })?;
                if let Some(access_token) = refreshed.get("access_token").and_then(Value::as_str) {
                    token = access_token.to_owned();
                    let mut updated_profile = profile.clone();
                    if let Some(object) = updated_profile.as_object_mut() {
                        object.insert("nuvioAccessToken".into(), json!(access_token));
                        if let Some(refresh) = refreshed.get("refresh_token") {
                            object.insert("nuvioRefreshToken".into(), refresh.clone());
                        }
                        let expires_in = refreshed
                            .get("expires_in")
                            .and_then(Value::as_i64)
                            .unwrap_or(3600);
                        object.insert(
                            "nuvioTokenExpiresAt".into(),
                            json!(chrono_unix_seconds() + expires_in),
                        );
                    }
                    if let Some(items) = profiles.as_array() {
                        let updated = items
                            .iter()
                            .map(|item| {
                                if item.get("id").and_then(Value::as_str) == Some(profile_id) {
                                    updated_profile.clone()
                                } else {
                                    item.clone()
                                }
                            })
                            .collect::<Vec<_>>();
                        self.storage.write_json("profiles", &json!(updated))?;
                    }
                }
            }
        }
        let nuvio_profile_id = profile
            .get("nuvioProfileIndex")
            .and_then(Value::as_i64)
            .filter(|value| *value > 0)
            .unwrap_or(1);
        let endpoint = format!("{}/rest/v1/rpc/", base_url.trim_end_matches('/'));
        let headers = |request: reqwest::RequestBuilder| {
            request.header("apikey", api_key).bearer_auth(&token)
        };
        let (library, progress) = futures::try_join!(
            async {
                headers(client.post(format!("{endpoint}sync_pull_library")))
                    .json(&json!({"p_profile_id": nuvio_profile_id, "p_limit": 500, "p_offset": 0}))
                    .send()
                    .await
                    .map_err(|error| format!("Nuvio library request failed: {error}"))?
                    .error_for_status()
                    .map_err(|error| format!("Nuvio library request failed: {error}"))?
                    .json::<Value>()
                    .await
                    .map_err(|error| format!("Nuvio library response was invalid: {error}"))
            },
            async {
                headers(client.post(format!("{endpoint}sync_pull_watch_progress")))
                    .json(&json!({"p_profile_id": nuvio_profile_id, "p_limit": 1000}))
                    .send()
                    .await
                    .map_err(|error| format!("Nuvio watch progress request failed: {error}"))?
                    .error_for_status()
                    .map_err(|error| format!("Nuvio watch progress request failed: {error}"))?
                    .json::<Value>()
                    .await
                    .map_err(|error| format!("Nuvio watch progress response was invalid: {error}"))
            }
        )?;
        crate::log!(
            "[fluxa-native] Nuvio library sync: library_rows={} progress_rows={}",
            library.as_array().map_or(0, Vec::len),
            progress.as_array().map_or(0, Vec::len)
        );
        core_value(
            "nuvioProviderLibrarySnapshot",
            json!({"library": library, "progress": progress}),
        )
        .map(Some)
        .ok_or_else(|| "Fluxa Core could not build the Nuvio library snapshot".to_owned())
    }

    async fn read_discover_catalog_filters(&self, payload: &Value) -> Result<Value, String> {
        let effect_profile = payload
            .get("profile")
            .filter(|value| value.is_object())
            .cloned();
        let profile_id = payload
            .get("profileId")
            .and_then(Value::as_str)
            .filter(|value| !value.is_empty())
            .or_else(|| effect_profile.as_ref()?.get("id").and_then(Value::as_str))
            .unwrap_or("guest");
        let mut profiles = self
            .storage
            .read_json("profiles")?
            .unwrap_or_else(|| json!([]));
        let stored_active_id = self
            .storage
            .read_json("active_profile_id")?
            .and_then(|value| value.as_str().map(ToOwned::to_owned))
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| profile_id.to_owned());
        let active_id = effect_profile
            .as_ref()
            .and_then(|profile| profile.get("id"))
            .and_then(Value::as_str)
            .filter(|value| !value.is_empty())
            .unwrap_or(&stored_active_id)
            .to_owned();
        if let Some(profile) = effect_profile.as_ref()
            && let Some(items) = profiles.as_array_mut()
        {
            if let Some(existing) = items.iter_mut().find(|item| {
                item.get("id").and_then(Value::as_str) == profile.get("id").and_then(Value::as_str)
            }) {
                *existing = profile.clone();
            } else {
                items.push(profile.clone());
            }
        }
        let owner = core_value(
            "effectiveAddonsOwnerId",
            json!({"profiles": profiles, "activeProfileId": active_id}),
        )
        .and_then(|value| value.as_str().map(ToOwned::to_owned))
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| profile_id.to_owned());
        let mut addons = payload
            .get("addons")
            .filter(|value| value.as_array().is_some_and(|items| !items.is_empty()))
            .cloned()
            .or(self
                .storage
                .read_json(&Storage::addons_key(&owner))?
                .or_else(|| {
                    self.storage
                        .read_json(&Storage::addons_key(profile_id))
                        .ok()
                        .flatten()
                })
                .or_else(|| self.storage.read_json("addons").ok().flatten()))
            .unwrap_or_else(|| json!([]));
        if !addons.is_array() {
            addons = json!([]);
        }
        // Home normalizes cached/legacy descriptors before deriving catalog
        // feeds. Discover must use the same Core-normalized input or installed
        // addons can appear on Home while this screen has no selectable catalogs.
        addons = normalize_enabled_addons(addons)?;
        let cached_addon_count = addons.as_array().map_or(0, Vec::len);

        // Home and Discover must use the same effective add-on inventory.
        // Refresh a stale Nuvio token here, then reuse Home's single remote
        // profile/add-on/manifest hydration path instead of maintaining a
        // second implementation that could silently produce an empty list.
        if let Some(profile) = profiles.as_array().and_then(|items| {
            items.iter().find(|item| {
                item.get("id").and_then(Value::as_str) == Some(active_id.as_str())
                    || item.get("id").and_then(Value::as_str) == Some(profile_id)
            })
        }) {
            let mut profile_for_addons = profile.clone();
            let mut session = profile
                .get("nuvioAccessToken")
                .and_then(Value::as_str)
                .filter(|value| !value.is_empty())
                .map(ToOwned::to_owned);
            if let (Some(base_url), Some(api_key)) = (
                option_env!("FLUXA_NUVIO_SUPABASE_URL"),
                option_env!("FLUXA_NUVIO_SUPABASE_KEY"),
            ) {
                let client = Client::builder()
                    .user_agent("Fluxa/1.0")
                    .native_timeout(Duration::from_secs(12))
                    .build()
                    .map_err(|error| error.to_string())?;
                let expires_at = profile
                    .get("nuvioTokenExpiresAt")
                    .and_then(Value::as_i64)
                    .unwrap_or_default();
                if expires_at <= chrono_unix_seconds() + 60
                    && let Some(refresh_token) = profile
                        .get("nuvioRefreshToken")
                        .and_then(Value::as_str)
                        .filter(|value| !value.is_empty())
                    && let Ok(response) = client
                        .post(format!(
                            "{}/auth/v1/token?grant_type=refresh_token",
                            base_url.trim_end_matches('/')
                        ))
                        .header("apikey", api_key)
                        .json(&json!({"refresh_token": refresh_token}))
                        .send()
                        .await
                        .and_then(reqwest::Response::error_for_status)
                    && let Ok(refreshed) = response.json::<Value>().await
                    && let Some(access_token) =
                        refreshed.get("access_token").and_then(Value::as_str)
                {
                    session = Some(access_token.to_owned());
                    let mut updated_profile = profile.clone();
                    if let Some(object) = updated_profile.as_object_mut() {
                        object.insert("nuvioAccessToken".into(), json!(access_token));
                        if let Some(refresh) = refreshed.get("refresh_token") {
                            object.insert("nuvioRefreshToken".into(), refresh.clone());
                        }
                        let expires_in = refreshed
                            .get("expires_in")
                            .and_then(Value::as_i64)
                            .unwrap_or(3600);
                        object.insert(
                            "nuvioTokenExpiresAt".into(),
                            json!(chrono_unix_seconds() + expires_in),
                        );
                    }
                    profile_for_addons = updated_profile.clone();
                    if let Some(items) = profiles.as_array() {
                        let updated = items
                            .iter()
                            .map(|item| {
                                if item.get("id").and_then(Value::as_str)
                                    == profile.get("id").and_then(Value::as_str)
                                {
                                    updated_profile.clone()
                                } else {
                                    item.clone()
                                }
                            })
                            .collect::<Vec<_>>();
                        self.storage.write_json("profiles", &json!(updated))?;
                    }
                }
                if let Some(session) = session {
                    if let Some(object) = profile_for_addons.as_object_mut() {
                        object.insert("nuvioAccessToken".into(), json!(session));
                    }
                }
            }
            self.merge_nuvio_addons(&profile_for_addons, &mut addons)
                .await;
        }
        crate::log!(
            "[fluxa-native] Discover catalogs: cached_addons={} hydrated_remote_addons={}",
            cached_addon_count,
            addons
                .as_array()
                .map_or(0, Vec::len)
                .saturating_sub(cached_addon_count)
        );
        if let Some(items) = addons.as_array_mut() {
            for item in items {
                if item
                    .get("manifest")
                    .and_then(|manifest| manifest.get("id"))
                    .and_then(Value::as_str)
                    .is_none()
                    && let Some(normalized) = core_value(
                        "normalizeAddonDescriptor",
                        json!({"addonJson": item.to_string()}),
                    )
                {
                    *item = normalized;
                }
            }
        }
        Ok(json!({"addons": addons}))
    }

    async fn prepare_direct_playback(&self, payload: &Value) -> Result<Value, String> {
        let meta = payload
            .get("meta")
            .filter(|value| value.is_object())
            .ok_or_else(|| "direct playback is missing meta".to_owned())?;
        let content_type = meta.get("type").and_then(Value::as_str).unwrap_or("movie");
        let id = meta
            .get("lastVideoId")
            .and_then(Value::as_str)
            .filter(|id| !id.is_empty())
            .or_else(|| meta.get("id").and_then(Value::as_str))
            .or_else(|| meta.get("imdb_id").and_then(Value::as_str))
            .ok_or_else(|| "direct playback meta is missing id".to_owned())?;
        let request_ids = stream_request_ids(content_type, id, Some(meta))?;
        let mut request = json!({
            "kind": "streams",
            "resource": "stream",
            "contentType": content_type,
            "requestIds": request_ids,
            "addons": self.load_enabled_addons()?,
        });
        request["detail"] = meta.clone();
        let streams = self.fetch_streams(&request).await?;
        let has_stream_providers = !streams.is_empty();
        let available_addons = core_value(
            "detailStreamResultPlan",
            json!({
                "attempts": [{"requestId": id, "streams": streams.clone()}],
                "hasStreamProviders": has_stream_providers
            }),
        )
        .and_then(|plan| plan.get("availableAddons").cloned())
        .unwrap_or_else(|| json!([]));
        Ok(json!({
            "streams": streams,
            "availableAddons": available_addons,
            "failedAddons": [],
            "hasStreamProviders": has_stream_providers,
        }))
    }

    async fn load_streams(&self, payload: &Value) -> Result<Value, String> {
        if payload.get("useInitialStreams").and_then(Value::as_bool) == Some(true) {
            let streams = payload
                .get("initialStreams")
                .and_then(Value::as_array)
                .filter(|streams| !streams.is_empty())
                .ok_or_else(|| "Core requested initial streams but supplied none".to_owned())?;
            return Ok(Value::Array(streams.clone()));
        }

        let content_type = payload
            .get("contentType")
            .and_then(Value::as_str)
            .unwrap_or("movie");
        let id = payload
            .get("id")
            .and_then(Value::as_str)
            .ok_or_else(|| "loadStreams is missing id".to_owned())?;
        let request_ids = stream_request_ids(content_type, id, None)?;
        let streams = self
            .fetch_streams(&json!({
                "kind": "streams",
                "resource": "stream",
                "contentType": content_type,
                "requestIds": request_ids,
                "addons": self.load_enabled_addons()?,
            }))
            .await?;
        Ok(Value::Array(streams))
    }

    async fn prefetch_next_episode_streams(&self, payload: &Value) -> Result<Value, String> {
        let content_type = payload
            .get("contentType")
            .and_then(Value::as_str)
            .unwrap_or("series");
        let next_video_id = payload
            .get("nextVideoId")
            .and_then(Value::as_str)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| "prefetchNextEpisodeStreams is missing nextVideoId".to_owned())?;
        let request_ids = stream_request_ids(content_type, next_video_id, None)?;
        let streams = self
            .fetch_streams(&json!({
                "kind": "streams",
                "resource": "stream",
                "contentType": content_type,
                "requestIds": request_ids,
                "addons": self.load_enabled_addons()?,
            }))
            .await?;
        Ok(json!({ "streams": streams }))
    }

    fn load_enabled_addons(&self) -> Result<Value, String> {
        let profiles = self
            .storage
            .read_json("profiles")?
            .unwrap_or_else(|| json!([]));
        let active_id = self
            .storage
            .read_json("active_profile_id")?
            .and_then(|value| value.as_str().map(ToOwned::to_owned))
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| "guest".to_owned());
        let owner = core_value(
            "effectiveAddonsOwnerId",
            json!({"profiles": profiles, "activeProfileId": active_id}),
        )
        .and_then(|value| value.as_str().map(ToOwned::to_owned))
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| active_id.clone());
        let mut addons = self
            .storage
            .read_json(&Storage::addons_key(&owner))?
            .or_else(|| {
                self.storage
                    .read_json(&Storage::addons_key(&active_id))
                    .ok()
                    .flatten()
            })
            .or_else(|| self.storage.read_json("addons").ok().flatten())
            .unwrap_or_else(|| json!([]));
        if !addons.is_array() {
            addons = json!([]);
        }
        addons = core_value(
            "filterEnabledAddons",
            json!({"addons": addons, "disabledKeys": []}),
        )
        .ok_or_else(|| "Fluxa Core could not filter disabled addons".to_owned())?;
        if let Some(items) = addons.as_array_mut() {
            for item in items {
                if item
                    .get("manifest")
                    .and_then(|manifest| manifest.get("id"))
                    .and_then(Value::as_str)
                    .is_none()
                    && let Some(normalized) = core_value(
                        "normalizeAddonDescriptor",
                        json!({"addonJson": item.to_string()}),
                    )
                {
                    *item = normalized;
                }
            }
        }
        Ok(addons)
    }

    async fn fetch_streams(&self, request: &Value) -> Result<Vec<Value>, String> {
        let policy = core_value("resourceFetchExecutionPolicy", request.clone())
            .ok_or_else(|| "Fluxa Core could not create a stream fetch plan".to_owned())?;
        let requests = policy
            .get("requests")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        if requests.is_empty() {
            return Ok(Vec::new());
        }
        let client = Client::builder()
            .user_agent("Fluxa/1.0")
            .native_timeout(Duration::from_secs(60))
            .build()
            .map_err(|error| error.to_string())?;
        // Stream providers are independent. The old loop waited for every
        // addon serially, so one dead provider delayed the entire player by
        // its full timeout. Fan them out and flatten in request order.
        let mut jobs = Vec::new();
        for (index, item) in requests.into_iter().enumerate() {
            let client = client.clone();
            let request_resource = request.get("resource").cloned();
            jobs.push(async move {
                let Some(url) = item.get("url").and_then(Value::as_str) else {
                    return (index, Vec::new());
                };
                let kind = item
                    .get("kind")
                    .and_then(Value::as_str)
                    .unwrap_or("streams");
                let resource = core_value(
                    "resourceKindToResource",
                    json!({
                        "kind": kind,
                        "requestResource": request_resource,
                        "itemResource": item.get("resource"),
                    }),
                )
                .and_then(|value| value.as_str().map(ToOwned::to_owned))
                .unwrap_or_else(|| "stream".to_owned());
                let (status_code, body) = match fetch_text(&client, url).await {
                    Ok(response) => response,
                    Err(error) => {
                        crate::log!(
                            "[fluxa-native] stream request failed for {}: {error}",
                            url_without_query(url)
                        );
                        (0, None)
                    }
                };
                let streams = core_value(
                    "parseAndPlanAddonResource",
                    json!({
                        "resource": resource,
                        "url": url,
                        "statusCode": status_code,
                        "body": body,
                        "kind": kind,
                        "addonName": item.get("addonName"),
                        "season": Value::Null,
                    }),
                )
                .and_then(|parsed| {
                    (parsed.get("kind").and_then(Value::as_str) == Some("success")).then(|| {
                        parsed
                            .get("value")
                            .and_then(|value| value.get("streams"))
                            .and_then(Value::as_array)
                            .cloned()
                            .unwrap_or_default()
                    })
                })
                .unwrap_or_default();
                (index, streams)
            });
        }

        let mut results = futures::future::join_all(jobs).await;
        results.sort_by_key(|(index, _)| *index);
        Ok(results
            .into_iter()
            .flat_map(|(_, streams)| streams)
            .collect())
    }

    async fn run_discover(&self, payload: &Value) -> Result<Value, String> {
        let requests = core_value("discoverSourceRequests", payload.clone())
            .and_then(|value| value.as_array().cloned())
            .ok_or_else(|| "Fluxa Core could not plan Discover requests".to_owned())?;
        let mut sources = Vec::with_capacity(requests.len());
        for request in requests {
            let transport_url = request
                .get("transportUrl")
                .and_then(Value::as_str)
                .ok_or_else(|| "Core Discover request is missing transportUrl".to_owned())?;
            let request_type = request
                .get("type")
                .and_then(Value::as_str)
                .ok_or_else(|| "Core Discover request is missing type".to_owned())?;
            let catalog_id = request
                .get("catalogId")
                .and_then(Value::as_str)
                .ok_or_else(|| "Core Discover request is missing catalogId".to_owned())?;
            let extra = request.get("extra").cloned().unwrap_or_else(|| json!({}));
            // Match Android: a failing media type must not discard successful sibling results.
            let items = self
                .fetch_addon_catalog_items(
                    transport_url,
                    request_type,
                    catalog_id,
                    &extra,
                    "discover",
                )
                .await
                .unwrap_or_default();
            sources.push(json!({
                "transportUrl": transport_url,
                "catalogId": catalog_id,
                "type": request_type,
                "genre": request.get("genre"),
                "items": items
            }));
        }
        core_value("mergeDiscoverSources", json!({"sources": sources}))
            .ok_or_else(|| "Fluxa Core could not merge Discover sources".to_owned())
    }

    async fn run_search(&self, payload: &Value) -> Result<Value, String> {
        let query = payload
            .get("query")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .trim();
        if query.is_empty() {
            return Ok(json!({"results": [], "categories": [], "grouping": {
                "groups": [], "totalCount": 0, "query": query
            }}));
        }

        let addons = self.load_enabled_addons()?;
        let plan = core_value(
            "resourceFetchPlan",
            json!({"kind": "search", "query": query, "addons": addons}),
        )
        .ok_or_else(|| "Fluxa Core could not plan search resources".to_owned())?;
        let requests = plan
            .get("requests")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        let client = Client::builder()
            .user_agent("Fluxa/1.0")
            .native_timeout(Duration::from_secs(20))
            .build()
            .map_err(|error| error.to_string())?;
        let fetches = requests.iter().map(|request| {
            let client = &client;
            async move {
                let url = request.get("url").and_then(Value::as_str)?;
                let transport_url = request
                    .get("transportUrl")
                    .and_then(Value::as_str)
                    .unwrap_or_default();
                let catalog_type = request
                    .get("catalogType")
                    .and_then(Value::as_str)
                    .unwrap_or_default();
                let items = if transport_url.starts_with("tmdb://builtin")
                    || url.starts_with("tmdb://builtin")
                {
                    match self
                        .fetch_builtin_tmdb_items(catalog_type, &json!({"search": query}))
                        .await
                    {
                        Ok(items) => items,
                        Err(error) => {
                            crate::log!("[fluxa-native] builtin search failed: {error}");
                            return None;
                        }
                    }
                } else {
                    let (status_code, body) = match fetch_text(client, url).await {
                        Ok(response) => response,
                        Err(error) => {
                            crate::log!(
                                "[fluxa-native] search request failed for {}: {error}",
                                url_without_query(url)
                            );
                            return None;
                        }
                    };
                    let items =
                        match parse_catalog_items(url, status_code, body.as_deref(), "search") {
                            Ok(items) => items,
                            Err(error) => {
                                crate::log!(
                                    "[fluxa-native] search response rejected by Core: {error}"
                                );
                                return None;
                            }
                        };
                    annotate_catalog_items(&items, transport_url, catalog_type).ok()?
                };
                if items.is_empty() {
                    return None;
                }
                let source_name = request
                    .get("categoryName")
                    .or_else(|| request.get("addonName"))
                    .cloned()
                    .unwrap_or(Value::Null);
                Some(json!({
                    "id": request.get("categoryId").cloned().unwrap_or_else(|| json!(url)),
                    "name": source_name,
                    "semanticName": source_name,
                    "type": request.get("catalogType"),
                    "items": items,
                    "addonName": request.get("addonName"),
                    "catalogId": request.get("catalogId")
                }))
            }
        });
        let sources: Vec<Value> = futures::future::join_all(fetches)
            .await
            .into_iter()
            .flatten()
            .collect();

        let merged = core_value("mergeSearchSources", Value::Array(sources))
            .ok_or_else(|| "Fluxa Core could not merge search results".to_owned())?;
        let results = merged.get("results").cloned().unwrap_or_else(|| json!([]));
        let grouping = core_value(
            "searchResultGrouping",
            json!({"query": query, "results": results}),
        )
        .unwrap_or_else(|| json!({"groups": [], "totalCount": 0, "query": query}));
        Ok(json!({
            "results": results,
            "categories": merged.get("categories").cloned().unwrap_or_else(|| json!([])),
            "grouping": grouping
        }))
    }

    async fn fetch_catalog_page(&self, payload: &Value) -> Result<Value, String> {
        if payload
            .get("remoteSource")
            .is_some_and(|source| !source.is_null() && source != &json!([]))
        {
            return Err("desktop remote collection paging is not implemented".to_owned());
        }
        let transport_url = payload
            .get("transportUrl")
            .and_then(Value::as_str)
            .ok_or_else(|| "catalog page is missing transportUrl".to_owned())?;
        let content_type = payload
            .get("contentType")
            .and_then(Value::as_str)
            .unwrap_or("movie");
        let catalog_id = payload
            .get("catalogId")
            .and_then(Value::as_str)
            .ok_or_else(|| "catalog page is missing catalogId".to_owned())?;
        let mut extra = serde_json::Map::new();
        if let Some(skip) = payload.get("skip").and_then(Value::as_i64) {
            extra.insert("skip".to_owned(), json!(skip.max(0)));
        }
        if let Some(genre) = payload.get("genre").and_then(Value::as_str) {
            extra.insert("genre".to_owned(), json!(genre));
        }
        if let Some(search) = payload.get("search").and_then(Value::as_str) {
            if !search.trim().is_empty() {
                extra.insert("search".to_owned(), json!(search));
            }
        }
        let items = self
            .fetch_addon_catalog_items(
                transport_url,
                content_type,
                catalog_id,
                &Value::Object(extra),
                "catalogPage",
            )
            .await?;
        Ok(json!({"items": items}))
    }

    async fn fetch_discover_page(&self, payload: &Value) -> Result<Value, String> {
        let transport_url = payload
            .get("transportUrl")
            .and_then(Value::as_str)
            .ok_or_else(|| "discover page is missing transportUrl".to_owned())?;
        let content_type = payload
            .get("contentType")
            .and_then(Value::as_str)
            .unwrap_or("movie");
        let catalog_id = payload
            .get("catalogId")
            .and_then(Value::as_str)
            .ok_or_else(|| "discover page is missing catalogId".to_owned())?;
        let mut extra = serde_json::Map::new();
        if let Some(skip) = payload.get("skip").and_then(Value::as_i64) {
            extra.insert("skip".to_owned(), json!(skip.max(0)));
        }
        if let Some(genre) = payload.get("genre").and_then(Value::as_str) {
            extra.insert("genre".to_owned(), json!(genre));
        }
        if let Some(search) = payload
            .get("search")
            .and_then(Value::as_str)
            .filter(|value| !value.trim().is_empty())
        {
            extra.insert("search".to_owned(), json!(search));
        }
        let items = self
            .fetch_addon_catalog_items(
                transport_url,
                content_type,
                catalog_id,
                &Value::Object(extra),
                "catalogPage",
            )
            .await?;
        Ok(json!({"items": items}))
    }

    async fn fetch_detail_streams(&self, payload: &Value) -> Result<Value, String> {
        let content_type = payload
            .get("contentType")
            .and_then(Value::as_str)
            .unwrap_or("movie");
        let request_ids = payload
            .get("requestIds")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .filter(|id| !id.trim().is_empty())
            .collect::<Vec<_>>();
        let addons = self.load_enabled_addons()?;
        let provider_plan = core_value(
            "providerAvailabilityPlan",
            json!({"addons": addons.clone(), "pluginNames": []}),
        )
        .ok_or_else(|| "Fluxa Core could not resolve stream provider availability".to_owned())?;
        let mut attempts = Vec::new();
        for request_id in request_ids {
            let mut request = json!({
                "kind": "streams",
                "resource": "stream",
                "contentType": content_type,
                "requestIds": [request_id],
                "addons": addons.clone()
            });
            if let Some(detail) = payload.get("detail") {
                request["detail"] = detail.clone();
            }
            let streams = self.fetch_streams(&request).await?;
            let found_streams = !streams.is_empty();
            attempts.push(json!({"requestId": request_id, "streams": streams}));
            if found_streams {
                break;
            }
        }
        core_value(
            "detailStreamResultPlan",
            json!({
                "attempts": attempts,
                "hasStreamProviders": provider_plan
                    .get("hasStreamProviders")
                    .and_then(Value::as_bool)
                    .unwrap_or(false)
            }),
        )
        .ok_or_else(|| "Fluxa Core could not build the detail stream result".to_owned())
    }

    async fn fetch_meta_detail(&self, payload: &Value) -> Result<Value, String> {
        let content_type = payload
            .get("contentType")
            .and_then(Value::as_str)
            .ok_or_else(|| "meta detail request is missing contentType".to_owned())?;
        let id = payload
            .get("id")
            .and_then(Value::as_str)
            .ok_or_else(|| "meta detail request is missing id".to_owned())?;
        let addons = self.load_enabled_addons()?;
        let plan = core_value(
            "resourceFetchPlan",
            json!({
                "kind": "metaDetail",
                "resource": "meta",
                "contentType": content_type,
                "id": id,
                "transportUrl": payload.get("sourceAddonTransportUrl"),
                "addons": addons
            }),
        )
        .ok_or_else(|| "Fluxa Core could not plan the meta detail request".to_owned())?;
        let requests = plan
            .get("requests")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        let client = Client::builder()
            .user_agent("Fluxa/1.0")
            .native_timeout(Duration::from_secs(20))
            .build()
            .map_err(|error| error.to_string())?;
        for request in requests {
            let Some(url) = request.get("url").and_then(Value::as_str) else {
                continue;
            };
            let (status_code, body) = match fetch_text(&client, url).await {
                Ok(response) => response,
                Err(error) => {
                    crate::log!(
                        "[fluxa-native] meta request failed for {}: {error}",
                        url_without_query(url)
                    );
                    continue;
                }
            };
            let parsed = core_value(
                "parseAndPlanAddonResource",
                json!({
                    "resource": "meta",
                    "url": url,
                    "statusCode": status_code,
                    "body": body,
                    "kind": "metaDetail",
                    "addonName": request.get("addonName"),
                    "season": null
                }),
            );
            if parsed
                .as_ref()
                .and_then(|value| value.get("kind"))
                .and_then(Value::as_str)
                == Some("success")
            {
                return Ok(parsed
                    .and_then(|value| value.pointer("/value/meta").cloned())
                    .unwrap_or(Value::Null));
            }
        }
        Ok(Value::Null)
    }

    async fn fetch_addon_catalog_items(
        &self,
        transport_url: &str,
        content_type: &str,
        catalog_id: &str,
        extra: &Value,
        parse_kind: &str,
    ) -> Result<Vec<Value>, String> {
        if transport_url == "tmdb://builtin" {
            return self.fetch_builtin_tmdb_items(content_type, extra).await;
        }
        let url = core_value(
            "buildResourceUrl",
            json!({
                "transportUrl": transport_url,
                "resource": "catalog",
                "contentType": content_type,
                "id": catalog_id,
                "extraJson": extra.to_string()
            }),
        )
        .and_then(|value| value.as_str().map(ToOwned::to_owned))
        .ok_or_else(|| "Fluxa Core could not build the catalog page URL".to_owned())?;
        let client = Client::builder()
            .user_agent("Fluxa/1.0")
            .native_timeout(Duration::from_secs(20))
            .build()
            .map_err(|error| error.to_string())?;
        let (status_code, body) = fetch_text(&client, &url).await?;
        let items = parse_catalog_items(&url, status_code, body.as_deref(), parse_kind)?;
        annotate_catalog_items(&items, transport_url, content_type)
    }

    async fn fetch_builtin_tmdb_items(
        &self,
        content_type: &str,
        extra: &Value,
    ) -> Result<Vec<Value>, String> {
        let active_id = self
            .storage
            .read_json("active_profile_id")?
            .and_then(|value| value.as_str().map(ToOwned::to_owned))
            .unwrap_or_default();
        let prefs = self
            .storage
            .read_json(&Storage::prefs_key(&active_id))?
            .or_else(|| self.storage.read_json("prefs").ok().flatten())
            .unwrap_or_else(|| json!({}));
        let api_key = prefs
            .get("tmdbApiKey")
            .and_then(Value::as_str)
            .ok_or_else(|| "TMDB API key is not configured".to_owned())?;
        let language = self
            .storage
            .read_json("profiles")?
            .and_then(|profiles| {
                profiles.as_array()?.iter().find_map(|profile| {
                    (profile.get("id").and_then(Value::as_str) == Some(active_id.as_str()))
                        .then(|| profile.get("language")?.as_str().map(ToOwned::to_owned))
                        .flatten()
                })
            })
            .unwrap_or_else(|| "en".to_owned());
        let url = core_value(
            "tmdbBuiltinCatalogUrl",
            json!({"contentType": content_type, "extra": extra, "apiKey": api_key, "language": language}),
        )
        .and_then(|value| value.as_str().map(ToOwned::to_owned))
        .ok_or_else(|| "Fluxa Core could not build the TMDB catalog URL".to_owned())?;
        let client = Client::builder()
            .user_agent("Fluxa/1.0")
            .native_timeout(Duration::from_secs(20))
            .build()
            .map_err(|error| error.to_string())?;
        let response = fetch_json(&client, &url).await?;
        let results = response
            .get("results")
            .cloned()
            .unwrap_or_else(|| json!([]));
        let metas = core_value(
            "tmdbBulkMetas",
            json!({"itemsJson": results.to_string(), "requestedType": content_type, "language": language}),
        )
        .unwrap_or_else(|| json!([]));
        annotate_catalog_items(
            metas.as_array().map(Vec::as_slice).unwrap_or_default(),
            "tmdb://builtin",
            content_type,
        )
    }

    async fn fetch_addon_manifest(&self, payload: &Value) -> Result<Value, String> {
        let transport_url = payload
            .get("transportUrl")
            .and_then(Value::as_str)
            .filter(|url| !url.trim().is_empty())
            .ok_or_else(|| "manifest request is missing transportUrl".to_owned())?;
        let plan = core_value("manifestFetchPlan", json!({"url": transport_url}))
            .ok_or_else(|| "Fluxa Core could not plan manifest requests".to_owned())?;
        let candidates = plan
            .get("candidateUrls")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        let client = Client::builder()
            .user_agent("Fluxa/1.0")
            .native_timeout(Duration::from_secs(20))
            .build()
            .map_err(|error| error.to_string())?;
        for candidate in candidates.iter().filter_map(Value::as_str) {
            let Ok((status_code, Some(body))) = fetch_text(&client, candidate).await else {
                continue;
            };
            if !(200..300).contains(&status_code) {
                continue;
            }
            let host = reqwest::Url::parse(candidate)
                .ok()
                .and_then(|url| url.host_str().map(ToOwned::to_owned))
                .unwrap_or_else(|| "Unknown Addon".to_owned());
            let Some(descriptor) = core_value(
                "parseManifest",
                json!({
                    "body": body,
                    "transportUrl": plan.get("normalizedTransportUrl").and_then(Value::as_str).unwrap_or(transport_url),
                    "unknownName": host,
                }),
            ) else {
                continue;
            };
            return core_value("resolveManifestAssets", descriptor.clone())
                .or(Some(descriptor))
                .ok_or_else(|| "Fluxa Core could not resolve manifest assets".to_owned());
        }
        Err("addon manifest could not be fetched or parsed".to_owned())
    }

    async fn fetch_addon_resource(&self, payload: &Value) -> Result<Value, String> {
        let transport_url = payload
            .get("transportUrl")
            .and_then(Value::as_str)
            .filter(|value| !value.trim().is_empty())
            .ok_or_else(|| "addon resource is missing transportUrl".to_owned())?;
        let resource = payload
            .get("resource")
            .and_then(Value::as_str)
            .unwrap_or("catalog");
        let content_type = payload
            .get("contentType")
            .and_then(Value::as_str)
            .unwrap_or("movie");
        let id = payload
            .get("id")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let extra = payload.get("extra").cloned().unwrap_or(Value::Null);
        let extra_args = extra
            .as_object()
            .map(|values| {
                values
                    .iter()
                    .filter_map(|(key, value)| {
                        let text = value
                            .as_str()
                            .map(ToOwned::to_owned)
                            .or_else(|| value.as_i64().map(|number| number.to_string()))
                            .or_else(|| value.as_bool().map(|boolean| boolean.to_string()))?;
                        Some((key.clone(), Value::String(text)))
                    })
                    .collect::<serde_json::Map<_, _>>()
            })
            .unwrap_or_default();
        let extra_raw = payload
            .get("extraArgs")
            .and_then(Value::as_str)
            .or_else(|| extra.as_str())
            .unwrap_or_default();
        let request_plan = core_value(
            "addonResourceRequestPlan",
            json!({
                "transportUrl": transport_url,
                "resource": resource,
                "contentType": content_type,
                "id": id,
                "extraArgs": extra_args,
                "extraRaw": extra_raw,
            }),
        )
        .ok_or_else(|| "Fluxa Core could not plan addon resource URLs".to_owned())?;
        let urls = request_plan
            .get("urls")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        let kind = match resource {
            "catalog" | "metas" => "catalogPage",
            "meta" => "metaDetail",
            "stream" | "streams" => "streams",
            "subtitle" | "subtitles" => "subtitles",
            other => other,
        };
        let addon_name = payload.get("addonName").and_then(Value::as_str);
        let season = payload.get("season").and_then(Value::as_i64);
        let client = Client::builder()
            .user_agent("Fluxa/1.0")
            .native_timeout(Duration::from_secs(20))
            .build()
            .map_err(|error| error.to_string())?;
        for url in urls.iter().filter_map(Value::as_str) {
            let Ok((status_code, body)) = fetch_text(&client, url).await else {
                continue;
            };
            let Some(parsed) = core_value(
                "parseAndPlanAddonResource",
                json!({
                    "resource": resource,
                    "url": url,
                    "statusCode": status_code,
                    "body": body,
                    "kind": kind,
                    "addonName": addon_name,
                    "season": season,
                }),
            ) else {
                continue;
            };
            if parsed.get("kind").and_then(Value::as_str) == Some("success") {
                if let Some(value_json) = parsed.get("valueJson").and_then(Value::as_str) {
                    if let Ok(value) = serde_json::from_str(value_json) {
                        return Ok(value);
                    }
                }
            }
        }
        Ok(if kind == "metaDetail" {
            Value::Null
        } else {
            Value::Array(Vec::new())
        })
    }

    fn read_calendar_month(&self, payload: &Value) -> Result<Value, String> {
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
        let external_items = library
            .get("externalCalendarItems")
            .cloned()
            .unwrap_or_else(|| json!([]));
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

fn in_watchlist(library: &Value, id: &str) -> bool {
    library
        .get("watchlist")
        .and_then(Value::as_array)
        .is_some_and(|items| {
            items
                .iter()
                .any(|item| item.get("id").and_then(Value::as_str) == Some(id))
        })
}

fn progress_meta(progress: &Value) -> Value {
    let mut meta = progress
        .get("meta")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();
    if let Some(values) = progress.as_object() {
        for (key, value) in values {
            if key != "meta" {
                meta.insert(key.clone(), value.clone());
            }
        }
    }
    Value::Object(meta)
}

#[cfg(target_arch = "wasm32")]
fn ensure_torrent_server() -> Result<Value, String> {
    Err("torrent streaming is not available on this platform".to_owned())
}

#[cfg(target_arch = "wasm32")]
fn start_torrent_add(_base_url: String, _link: String, _file_id: Option<i64>) {}

static TORRENT_CACHE_DIR: OnceLock<std::path::PathBuf> = OnceLock::new();

#[cfg(not(target_arch = "wasm32"))]
fn ensure_torrent_server() -> Result<Value, String> {
    let slot = TORRENT_SERVER.get_or_init(|| std::sync::Mutex::new(None));
    let mut guard = slot
        .lock()
        .map_err(|_| "torrent server state is poisoned".to_owned())?;
    if let Some(server) = guard.as_ref() {
        return Ok(server.clone());
    }
    let cache_dir = TORRENT_CACHE_DIR
        .get()
        .cloned()
        .ok_or_else(|| "torrent cache directory is not configured".to_owned())?;
    let server_raw =
        fluxa_streaming_engine::start_torrent_server(&cache_dir.to_string_lossy(), 0, "")
            .ok_or_else(|| "failed to start torrent server".to_owned())?;
    let server: Value = serde_json::from_str(&server_raw)
        .map_err(|error| format!("invalid torrent server response: {error}"))?;
    *guard = Some(server.clone());
    Ok(server)
}

#[cfg(not(target_arch = "wasm32"))]
fn start_torrent_add(base_url: String, link: String, file_id: Option<i64>) {
    std::thread::spawn(move || {
        let info_hash = link
            .split("xt=urn:btih:")
            .nth(1)
            .map(|tail| {
                tail.chars()
                    .take_while(char::is_ascii_alphanumeric)
                    .collect::<String>()
            })
            .unwrap_or_default();
        crate::log!(
            "[fluxa-native] torrent metadata add started hash={info_hash} file={file_id:?}"
        );
        let Ok(runtime) = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
        else {
            return;
        };
        let result: Result<Value, String> = runtime.block_on(async {
            let response = Client::new()
                .post(format!("{}/torrents", base_url.trim_end_matches('/')))
                .json(&json!({ "action": "add", "link": link, "file_id": file_id }))
                .timeout(Duration::from_secs(95))
                .send()
                .await
                .map_err(|error| error.to_string())?;
            let status = response.status();
            let body = response.text().await.map_err(|error| error.to_string())?;
            if !status.is_success() {
                return Err(format!("HTTP {status}: {body}"));
            }
            serde_json::from_str(&body).map_err(|error| error.to_string())
        });
        match result {
            Ok(status) => crate::log!(
                "[fluxa-native] torrent metadata ready hash={} peers={}",
                status.get("hash").and_then(Value::as_str).unwrap_or(""),
                status
                    .get("active_peers")
                    .and_then(Value::as_u64)
                    .unwrap_or(0)
            ),
            Err(error) => crate::log!("[fluxa-native] torrent add failed: {error}"),
        }
    });
}

async fn tmdb_similar(
    client: &Client,
    item: &Value,
    api_key: &str,
    language: &str,
    endpoints: &[&str],
) -> Option<Vec<Value>> {
    if api_key.trim().is_empty() || endpoints.is_empty() {
        return None;
    }
    let content_type = item.get("type").and_then(Value::as_str).unwrap_or("movie");
    let request = |extra: Value| {
        let mut args = json!({
            "contentType": content_type,
            "contentId": item.get("id"),
            "language": language,
            "apiKey": api_key,
            "endpoints": endpoints,
        });
        if let (Some(args), Some(extra)) = (args.as_object_mut(), extra.as_object()) {
            args.extend(extra.clone());
        }
        args
    };
    let mut plan = core_value("tmdbDetailRequestPlan", request(json!({})))?;
    if plan.get("urls").is_none() {
        let find_url = plan.get("findUrl")?.as_str()?.to_owned();
        let find = fetch_json(client, &find_url).await.ok()?;
        plan = core_value(
            "tmdbDetailRequestUrlsFromFind",
            request(json!({"find": find})),
        )?;
    }
    for endpoint in endpoints {
        let Some(url) = plan
            .pointer(&format!("/urls/{endpoint}"))
            .and_then(Value::as_str)
        else {
            continue;
        };
        let Ok(response) = fetch_json(client, url).await else {
            continue;
        };
        let Some(results) = response
            .get("results")
            .filter(|value| value.as_array().is_some_and(|items| !items.is_empty()))
        else {
            continue;
        };
        let metas = core_value(
            "tmdbBulkMetas",
            json!({"itemsJson": results.to_string(), "requestedType": content_type, "language": language}),
        )
        .and_then(|value| value.as_array().cloned())
        .unwrap_or_default();
        if !metas.is_empty() {
            return Some(metas);
        }
    }
    None
}

async fn tmdb_trailers(
    client: &Client,
    item: &Value,
    api_key: &str,
    language: &str,
) -> Option<Value> {
    let request = |extra: Value| {
        let mut args = json!({
            "contentType": item.get("type"),
            "contentId": item.get("id"),
            "language": language,
            "apiKey": api_key,
            "endpoints": ["videos"],
        });
        if let (Some(args), Some(extra)) = (args.as_object_mut(), extra.as_object()) {
            args.extend(extra.clone());
        }
        args
    };
    let mut plan = core_value("tmdbDetailRequestPlan", request(json!({})))?;
    if plan.get("urls").is_none() {
        let find_url = plan.get("findUrl")?.as_str()?.to_owned();
        let find = fetch_json(client, &find_url).await.ok()?;
        plan = core_value(
            "tmdbDetailRequestUrlsFromFind",
            request(json!({"find": find})),
        )?;
    }
    let url = plan.pointer("/urls/videos")?.as_str()?.to_owned();
    let videos = fetch_json(client, &url).await.ok()?;
    let results = videos
        .get("results")
        .filter(|value| value.as_array().is_some_and(|items| !items.is_empty()))?;
    core_value("tmdbBulkVideosToTrailers", results.clone())
}

async fn youtube_request(payload: &Value) -> Result<Value, String> {
    let url = payload
        .get("url")
        .and_then(Value::as_str)
        .ok_or_else(|| "missing trailer request URL".to_owned())?;
    let client = Client::builder()
        .native_timeout(Duration::from_secs(15))
        .build()
        .map_err(|error| error.to_string())?;
    let mut request = match payload.get("method").and_then(Value::as_str) {
        Some("POST") => client.post(url),
        _ => client.get(url),
    };
    for (name, value) in payload
        .get("headers")
        .and_then(Value::as_object)
        .into_iter()
        .flatten()
    {
        if let Some(value) = value.as_str() {
            request = request.header(name.as_str(), value);
        }
    }
    if let Some(body) = payload.get("body").filter(|body| !body.is_null()) {
        request = request
            .header("Content-Type", "application/json")
            .body(body.to_string());
    }
    let response = request.send().await.map_err(|error| error.to_string())?;
    let status = response.status().as_u16();
    let body = response.text().await.map_err(|error| error.to_string())?;
    Ok(json!({"statusCode": status, "body": body}))
}

#[cfg(not(target_arch = "wasm32"))]
fn detached<T: Send + 'static>(
    task: impl std::future::Future<Output = T> + Send + 'static,
) -> std::sync::mpsc::Receiver<T> {
    let (sender, receiver) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        if let Ok(runtime) = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
        {
            let _ = sender.send(runtime.block_on(task));
        }
    });
    receiver
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
    let response = client
        .get(parsed)
        .send()
        .await
        .map_err(|error| error.to_string())?;
    let status = i32::from(response.status().as_u16());
    let body = response.text().await.map_err(|error| error.to_string())?;
    Ok((status, Some(body)))
}

fn stream_request_ids(
    content_type: &str,
    id: &str,
    detail: Option<&Value>,
) -> Result<Vec<String>, String> {
    let detail_id = detail
        .and_then(|value| value.get("id"))
        .and_then(Value::as_str);
    let current_series_lookup_id = detail_id.and_then(|detail_id| {
        (content_type == "series" || content_type == "tv" || content_type == "show")
            .then(|| detail_id.to_owned())
    });
    core_value(
        "streamRequestIds",
        json!({
            "contentType": content_type,
            "id": id,
            "detailId": detail_id,
            "currentSeriesLookupId": current_series_lookup_id,
        }),
    )
    .and_then(|value| value.as_array().cloned())
    .map(|values| {
        values
            .into_iter()
            .filter_map(|value| value.as_str().map(ToOwned::to_owned))
            .collect::<Vec<_>>()
    })
    .filter(|values| !values.is_empty())
    .ok_or_else(|| "Fluxa Core could not resolve stream request IDs".to_owned())
}

fn parse_catalog_items(
    url: &str,
    status_code: i32,
    body: Option<&str>,
    kind: &str,
) -> Result<Vec<Value>, String> {
    let result = core_value(
        "parseAndPlanAddonResource",
        json!({
            "resource": "catalog",
            "url": url,
            "statusCode": status_code,
            "body": body,
            "kind": kind,
            "season": null,
        }),
    )
    .ok_or_else(|| "Core could not parse the catalog response".to_owned())?;
    match result.get("kind").and_then(Value::as_str) {
        Some("success") => {}
        Some("empty") => return Ok(Vec::new()),
        _ => {
            return Err(result
                .get("error")
                .and_then(Value::as_str)
                .map(ToOwned::to_owned)
                .unwrap_or_else(|| {
                    format!(
                        "Core returned {} for {url}",
                        result
                            .get("kind")
                            .and_then(Value::as_str)
                            .unwrap_or("an invalid response")
                    )
                }));
        }
    }
    Ok(result
        .pointer("/value/items")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default())
}

fn annotate_catalog_items(
    items: &[Value],
    transport_url: &str,
    catalog_type: &str,
) -> Result<Vec<Value>, String> {
    core_value(
        "annotateCatalogItems",
        json!({
            "itemsJson": Value::Array(items.to_vec()).to_string(),
            "transportUrl": transport_url,
            "catalogType": catalog_type
        }),
    )
    .and_then(|value| value.as_array().cloned())
    .ok_or_else(|| "Fluxa Core could not annotate catalog source metadata".to_owned())
}

fn url_without_query(url: &str) -> &str {
    url.split_once('?').map_or(url, |(base, _)| base)
}

fn core_value(method: &str, args: Value) -> Option<Value> {
    let raw = fluxa_core::ffi::core_invoke(method, &args.to_string());
    let envelope: Value = match serde_json::from_str(&raw) {
        Ok(envelope) => envelope,
        Err(error) => {
            crate::log!("[fluxa-native] core method `{method}` returned invalid JSON: {error}");
            return None;
        }
    };
    if envelope.get("ok").and_then(Value::as_bool) != Some(true) {
        let error = envelope.get("error");
        let kind = error
            .and_then(|value| value.get("kind"))
            .and_then(Value::as_str)
            .unwrap_or("error");
        let message = error
            .and_then(|value| value.get("message"))
            .and_then(Value::as_str)
            .unwrap_or("");
        crate::log!(
            "[fluxa-native] core method `{method}` failed: {kind}{}",
            if message.is_empty() {
                String::new()
            } else {
                format!(": {message}")
            }
        );
        return None;
    }
    envelope.get("value").cloned()
}

fn normalize_enabled_addons(addons: Value) -> Result<Value, String> {
    let mut addons = core_value(
        "filterEnabledAddons",
        json!({"addons": addons, "disabledKeys": []}),
    )
    .ok_or_else(|| "Fluxa Core could not filter enabled add-ons".to_owned())?;
    if let Some(items) = addons.as_array_mut() {
        for item in items {
            if item
                .get("manifest")
                .and_then(|manifest| manifest.get("id"))
                .and_then(Value::as_str)
                .is_some()
            {
                continue;
            }
            if let Some(normalized) = core_value(
                "normalizeAddonDescriptor",
                json!({"addonJson": item.to_string()}),
            ) {
                *item = normalized;
            }
        }
    }
    Ok(addons)
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
