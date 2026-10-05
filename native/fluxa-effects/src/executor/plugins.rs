use super::*;
use fluxa_plugin_runtime::{
    PLUGIN_MAX_RESPONSE_BODY_BYTES, PluginHttpClient, PluginHttpRequest, PluginHttpResponse,
    execute_scraper, plugin_http_request_error, plugin_network_address_allowed,
};
use std::collections::HashMap;
use std::net::ToSocketAddrs;
use std::sync::{Arc, Mutex};

const SCRAPER_TIMEOUT: Duration = Duration::from_secs(25);
const MAX_REDIRECTS: usize = 5;

fn tmdb_id_of(meta: &Value) -> Option<String> {
    ["moviedb_id", "_tmdbId", "tmdbId", "tmdb_id"]
        .iter()
        .find_map(|key| match meta.get(*key)? {
            Value::Number(number) => number.as_i64().map(|id| id.to_string()),
            Value::String(text) => text.parse::<i64>().ok().map(|id| id.to_string()),
            _ => None,
        })
}

pub(super) struct PluginTarget {
    tmdb_id: String,
    media_type: String,
    season: Option<i32>,
    episode: Option<i32>,
}

impl EffectExecutor {
    pub(super) async fn fetch_plugin_manifest(&self, payload: &Value) -> Result<Value, String> {
        let manifest_url = payload
            .get("manifestUrl")
            .and_then(Value::as_str)
            .filter(|url| !url.trim().is_empty())
            .ok_or_else(|| "plugin repository is missing manifestUrl".to_owned())?;
        let client = Client::builder()
            .user_agent("Fluxa/1.0")
            .native_timeout(Duration::from_secs(20))
            .build()
            .map_err(|error| error.to_string())?;
        let (status, body) = fetch_text(&client, manifest_url).await?;
        let body = body
            .filter(|_| (200..300).contains(&status))
            .ok_or_else(|| format!("plugin repository returned HTTP {status}"))?;
        let manifest: Value = serde_json::from_str(&body)
            .map_err(|_| "plugin manifest is not valid JSON".to_owned())?;
        if !manifest.get("scrapers").is_some_and(Value::is_array) {
            return Err("plugin manifest has no scrapers".to_owned());
        }
        Ok(json!({"manifestUrl": manifest_url, "manifest": manifest}))
    }

    pub(super) async fn enabled_scrapers(&self, request: &Value) -> Vec<Value> {
        let media_type = request
            .get("contentType")
            .and_then(Value::as_str)
            .unwrap_or("movie");
        let Ok(Some(profile_id)) = self.current_profile_id() else {
            return Vec::new();
        };
        let mut scrapers = self
            .storage
            .read_json(&Storage::plugins_key(&profile_id))
            .ok()
            .flatten()
            .and_then(|state| state.get("scrapers").and_then(Value::as_array).cloned())
            .unwrap_or_default();
        for scraper in self.nuvio_scrapers(&profile_id).await {
            let key = |value: &Value| {
                (
                    value.get("repositoryUrl").cloned(),
                    value.get("id").cloned(),
                )
            };
            if !scrapers.iter().any(|known| key(known) == key(&scraper)) {
                scrapers.push(scraper);
            }
        }
        scrapers
            .into_iter()
            .filter(|scraper| {
                fluxa_core::addons::plugin_streams::plugin_scraper_enabled_for(scraper, media_type)
            })
            .collect()
    }

    async fn nuvio_scrapers(&self, profile_id: &str) -> Vec<Value> {
        static CACHE: OnceLock<Mutex<HashMap<String, (i64, Vec<Value>)>>> = OnceLock::new();
        let cache = CACHE.get_or_init(Default::default);
        let now = chrono_unix_seconds();
        if let Some((at, scrapers)) = cache
            .lock()
            .ok()
            .and_then(|cache| cache.get(profile_id).cloned())
            && now - at < 600
        {
            return scrapers;
        }
        let scrapers = self.fetch_nuvio_scrapers().await.unwrap_or_else(|error| {
            crate::log!("[fluxa-native] Nuvio plugin sync failed: {error}");
            Vec::new()
        });
        if let Ok(mut cache) = cache.lock() {
            cache.insert(profile_id.to_owned(), (now, scrapers.clone()));
        }
        scrapers
    }

    async fn fetch_nuvio_scrapers(&self) -> Result<Vec<Value>, String> {
        let (_, profile, _) = self.active_profile()?;
        let Some(session) = self.nuvio_session(&profile).await? else {
            return Ok(Vec::new());
        };
        let scope = self.nuvio_profile_scope(&session, "plugins").await;
        let rows = session
            .rest(&format!(
                "plugins?select=*&profile_id=eq.{scope}&order=sort_order"
            ))
            .send()
            .await
            .and_then(reqwest::Response::error_for_status)
            .map_err(|error| format!("plugin list request failed: {error}"))?
            .json::<Value>()
            .await
            .map_err(|error| format!("plugin list response invalid: {error}"))?;
        let urls = rows
            .as_array()
            .into_iter()
            .flatten()
            .filter(|row| row.get("enabled").and_then(Value::as_bool) != Some(false))
            .filter_map(|row| row.get("url").and_then(Value::as_str))
            .map(ToOwned::to_owned)
            .collect::<Vec<_>>();
        let client = &session.client;
        let manifests = futures::future::join_all(urls.iter().map(|url| async move {
            let manifest = client
                .get(url)
                .send()
                .await
                .and_then(reqwest::Response::error_for_status)
                .map_err(|error| error.to_string())?
                .json::<Value>()
                .await
                .map_err(|error| error.to_string());
            if let Err(error) = &manifest {
                crate::log!(
                    "[fluxa-native] plugin manifest failed for {}: {error}",
                    url_without_query(url)
                );
            }
            manifest.map(|manifest| (url.clone(), manifest))
        }))
        .await;
        let mut scrapers = Vec::new();
        for (url, manifest) in manifests.into_iter().flatten() {
            for mut scraper in manifest
                .get("scrapers")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default()
            {
                if let Some(fields) = scraper.as_object_mut() {
                    fields.insert("repositoryUrl".to_owned(), json!(url));
                    fields.entry("settings").or_insert_with(|| json!({}));
                }
                scrapers.push(scraper);
            }
        }
        Ok(scrapers)
    }

    pub(super) async fn plugin_target(&self, request: &Value) -> Option<PluginTarget> {
        let content_type = request.get("contentType").and_then(Value::as_str)?;
        let id = request
            .get("requestIds")
            .and_then(Value::as_array)?
            .first()?
            .as_str()?;
        let mut parts = id.split(':');
        let imdb = parts.next()?.to_owned();
        let series = matches!(content_type, "series" | "tv" | "show");
        let season = parts.next().and_then(|part| part.parse().ok());
        let episode = parts.next().and_then(|part| part.parse().ok());
        let detail = request.get("detail");
        let tmdb_id = detail.and_then(tmdb_id_of);
        let tmdb_id = match tmdb_id {
            Some(id) => id,
            None => match self.tmdb_id_from_meta(content_type, &imdb).await {
                Some(id) => id,
                None => self.tmdb_id_for_imdb(&imdb, series).await?,
            },
        };
        Some(PluginTarget {
            tmdb_id,
            media_type: if series { "tv" } else { "movie" }.to_owned(),
            season: season.filter(|_| series),
            episode: episode.filter(|_| series),
        })
    }

    async fn tmdb_id_from_meta(&self, content_type: &str, imdb: &str) -> Option<String> {
        static CACHE: OnceLock<Mutex<HashMap<String, String>>> = OnceLock::new();
        let cache = CACHE.get_or_init(Default::default);
        if let Some(id) = cache.lock().ok().and_then(|cache| cache.get(imdb).cloned()) {
            return Some(id);
        }
        let meta = self
            .fetch_meta_detail(&json!({"contentType": content_type, "id": imdb}))
            .await
            .ok()?;
        let id = tmdb_id_of(&meta)?;
        if let Ok(mut cache) = cache.lock() {
            cache.insert(imdb.to_owned(), id.clone());
        }
        Some(id)
    }

    async fn tmdb_id_for_imdb(&self, imdb: &str, series: bool) -> Option<String> {
        if !imdb.starts_with("tt") {
            return None;
        }
        let profile_id = self.current_profile_id().ok().flatten()?;
        let prefs = self.profile_prefs(&profile_id).ok().flatten()?;
        let api_key = prefs.get("tmdbApiKey").and_then(Value::as_str)?.trim();
        if api_key.is_empty() {
            return None;
        }
        let client = Client::builder()
            .user_agent("Fluxa/1.0")
            .native_timeout(Duration::from_secs(10))
            .build()
            .ok()?;
        let url = format!(
            "https://api.themoviedb.org/3/find/{imdb}?external_source=imdb_id&api_key={api_key}"
        );
        let found = fetch_json(&client, &url).await.ok()?;
        let list = if series {
            "tv_results"
        } else {
            "movie_results"
        };
        found
            .get(list)?
            .as_array()?
            .first()?
            .get("id")?
            .as_i64()
            .map(|id| id.to_string())
    }

    pub(super) async fn run_plugin_scraper(
        &self,
        scraper: Value,
        target: Arc<PluginTarget>,
    ) -> Vec<Value> {
        let name = scraper
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or("Plugin")
            .to_owned();
        let result =
            tokio::time::timeout(SCRAPER_TIMEOUT, self.execute_plugin(&scraper, &target)).await;
        match result {
            Ok(Ok(raw)) => {
                let streams =
                    fluxa_core::addons::plugin_streams::plugin_scraper_streams(&raw, &name);
                crate::log!(
                    "[fluxa-native] plugin scraper {name} (tmdb {} {}): raw {} -> {} streams",
                    target.tmdb_id,
                    target.media_type,
                    raw.as_array().map_or(0, Vec::len),
                    streams.len()
                );
                streams
            }
            Ok(Err(error)) => {
                crate::log!("[fluxa-native] plugin scraper {name} failed: {error}");
                Vec::new()
            }
            Err(_) => {
                crate::log!("[fluxa-native] plugin scraper {name} timed out");
                Vec::new()
            }
        }
    }

    async fn execute_plugin(
        &self,
        scraper: &Value,
        target: &PluginTarget,
    ) -> Result<Value, String> {
        let repository_url = scraper
            .get("repositoryUrl")
            .and_then(Value::as_str)
            .ok_or_else(|| "scraper has no repository".to_owned())?
            .to_owned();
        let filename = scraper
            .get("filename")
            .and_then(Value::as_str)
            .ok_or_else(|| "scraper has no filename".to_owned())?;
        let scraper_id = scraper
            .get("id")
            .and_then(Value::as_str)
            .ok_or_else(|| "scraper has no id".to_owned())?
            .to_owned();
        install_plugin_logger();
        let code = scraper_code(&repository_url, filename).await?;
        let settings = scraper
            .get("settings")
            .map(Value::to_string)
            .unwrap_or_else(|| "{}".to_owned());
        let tmdb_id = target.tmdb_id.clone();
        let media_type = target.media_type.clone();
        let (season, episode) = (target.season, target.episode);
        let output = tokio::task::spawn_blocking(move || {
            execute_scraper(
                Arc::new(PluginClient::default()),
                code,
                repository_url,
                scraper_id,
                settings,
                tmdb_id,
                media_type,
                season,
                episode,
            )
        })
        .await
        .map_err(|error| error.to_string())??;
        serde_json::from_str(&output).map_err(|error| error.to_string())
    }
}

async fn scraper_code(repository_url: &str, filename: &str) -> Result<String, String> {
    static CACHE: OnceLock<Mutex<HashMap<String, String>>> = OnceLock::new();
    let base = repository_url
        .rsplit_once('/')
        .map_or(repository_url, |(base, _)| base);
    let url = format!("{base}/{filename}");
    let cache = CACHE.get_or_init(Default::default);
    if let Some(code) = cache.lock().ok().and_then(|cache| cache.get(&url).cloned()) {
        return Ok(code);
    }
    let client = Client::builder()
        .user_agent("Fluxa/1.0")
        .native_timeout(Duration::from_secs(20))
        .build()
        .map_err(|error| error.to_string())?;
    let (status, body) = fetch_text(&client, &url).await?;
    let code = body
        .filter(|_| (200..300).contains(&status))
        .ok_or_else(|| format!("scraper code returned HTTP {status}"))?;
    if let Ok(mut cache) = cache.lock() {
        cache.insert(url, code.clone());
    }
    Ok(code)
}

struct PluginLogger;

impl log::Log for PluginLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= log::Level::Debug
    }

    fn log(&self, record: &log::Record) {
        let text = record.args().to_string();
        if text.starts_with("[plugin]") {
            crate::log!("[fluxa-native] {text}");
        }
    }

    fn flush(&self) {}
}

fn install_plugin_logger() {
    static LOGGER: PluginLogger = PluginLogger;
    static ONCE: std::sync::Once = std::sync::Once::new();
    ONCE.call_once(|| {
        if log::set_logger(&LOGGER).is_ok() {
            log::set_max_level(log::LevelFilter::Debug);
        }
    });
}

const MAX_PLUGIN_REQUESTS: usize = 80;
const MAX_PLUGIN_BYTES: usize = 32 * 1024 * 1024;

#[derive(Default)]
struct PluginClient {
    requests: std::sync::atomic::AtomicUsize,
    bytes: std::sync::atomic::AtomicUsize,
}

impl PluginHttpClient for PluginClient {
    fn fetch(&self, request: PluginHttpRequest) -> PluginHttpResponse {
        use std::sync::atomic::Ordering::Relaxed;
        let over_budget = self.requests.fetch_add(1, Relaxed) >= MAX_PLUGIN_REQUESTS
            || self.bytes.load(Relaxed) >= MAX_PLUGIN_BYTES;
        if over_budget {
            return PluginHttpResponse {
                error: Some("plugin request budget exceeded".to_owned()),
                ..Default::default()
            };
        }
        let (sender, receiver) = std::sync::mpsc::channel();
        let spawned = std::thread::Builder::new()
            .name("fluxa-plugin-http".to_owned())
            .spawn(move || {
                let response = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .map_err(|error| error.to_string())
                    .and_then(|runtime| runtime.block_on(guarded_fetch(request)));
                let _ = sender.send(response);
            });
        let failed = |error: String| PluginHttpResponse {
            error: Some(error),
            ..Default::default()
        };
        if spawned.is_err() {
            return failed("plugin network thread unavailable".to_owned());
        }
        match receiver.recv_timeout(Duration::from_secs(40)) {
            Ok(Ok(response)) => {
                self.bytes.fetch_add(response.body.len(), Relaxed);
                response
            }
            Ok(Err(error)) => failed(error),
            Err(_) => failed("plugin request timed out".to_owned()),
        }
    }
}

async fn guarded_fetch(mut request: PluginHttpRequest) -> Result<PluginHttpResponse, String> {
    for _ in 0..=MAX_REDIRECTS {
        if let Some(error) = plugin_http_request_error(&request) {
            return Err(error.to_owned());
        }
        let url = reqwest::Url::parse(&request.url).map_err(|error| error.to_string())?;
        let host = url
            .host_str()
            .ok_or("plugin URL is missing a host")?
            .to_owned();
        let port = url.port_or_known_default().unwrap_or(443);
        let addresses = (host.as_str(), port)
            .to_socket_addrs()
            .map_err(|error| error.to_string())?
            .collect::<Vec<_>>();
        if addresses.is_empty()
            || addresses
                .iter()
                .any(|address| !plugin_network_address_allowed(&address.ip().to_string()))
        {
            return Err("private or loopback plugin address is not allowed".to_owned());
        }
        let client = Client::builder()
            .user_agent("Fluxa/1.0")
            .redirect(reqwest::redirect::Policy::none())
            .resolve(&host, addresses[0])
            .native_timeout(Duration::from_secs(30))
            .build()
            .map_err(|error| error.to_string())?;
        let method = reqwest::Method::from_bytes(request.method.to_ascii_uppercase().as_bytes())
            .map_err(|error| error.to_string())?;
        let mut builder = client.request(method, url.clone());
        for (name, value) in &request.headers {
            builder = builder.header(name.as_str(), value.as_str());
        }
        if let Some(body) = &request.body {
            builder = builder.body(body.clone());
        }
        let response = builder.send().await.map_err(|error| error.to_string())?;
        let status = response.status();
        if status.is_redirection()
            && request.follow_redirects
            && let Some(location) = response
                .headers()
                .get(reqwest::header::LOCATION)
                .and_then(|value| value.to_str().ok())
        {
            request.url = url
                .join(location)
                .map_err(|error| error.to_string())?
                .to_string();
            if status != reqwest::StatusCode::TEMPORARY_REDIRECT
                && status != reqwest::StatusCode::PERMANENT_REDIRECT
            {
                request.method = "GET".to_owned();
                request.body = None;
            }
            continue;
        }
        let headers = response
            .headers()
            .iter()
            .filter_map(|(name, value)| {
                Some((name.as_str().to_owned(), value.to_str().ok()?.to_owned()))
            })
            .collect::<HashMap<_, _>>();
        let final_url = response.url().to_string();
        let bytes = response.bytes().await.map_err(|error| error.to_string())?;
        if bytes.len() > PLUGIN_MAX_RESPONSE_BODY_BYTES {
            return Err("plugin response exceeds limit".to_owned());
        }
        return Ok(PluginHttpResponse {
            status: status.as_u16(),
            url: final_url,
            headers,
            body: String::from_utf8_lossy(&bytes).into_owned(),
            ok: status.is_success(),
            error: None,
        });
    }
    Err("too many plugin redirects".to_owned())
}
