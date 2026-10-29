use super::streams::now_ms;
use super::*;
use fluxa_aggmeta::idtables::{AnimeIdTable, Resolver};
use fluxa_aggmeta::service::{
    FetchError, Fetcher, HttpRequest, HttpResponse, KvCache, MetaConfig, MetaService,
};
use fluxa_aggmeta::types::{ContentType, Meta};
use fluxa_core::stream_pipeline;
use fluxa_streamkit::types::RequestContext;

struct ClientFetcher(Client);

impl Fetcher for ClientFetcher {
    async fn fetch(&self, request: HttpRequest) -> Result<HttpResponse, FetchError> {
        let builder = match request.post_body {
            Some(body) => self.0.post(&request.url).body(body),
            None => self.0.get(&request.url),
        };
        let builder = request
            .headers
            .iter()
            .fold(builder, |builder, (name, value)| {
                builder.header(name, value)
            });
        let response = builder
            .send()
            .await
            .map_err(|error| FetchError::Network(error.to_string()))?;
        let status = response.status().as_u16();
        let body = response
            .text()
            .await
            .map_err(|error| FetchError::Network(error.to_string()))?;
        Ok(HttpResponse { status, body })
    }
}

struct StorageCache(Storage);

fn cache_key(key: &str) -> String {
    format!("aggmeta_{key}")
}

impl KvCache for StorageCache {
    async fn get(&self, key: &str) -> Option<String> {
        let stored = self.0.read_json(&cache_key(key)).ok()??;
        let expires = stored.get("expires")?.as_u64()?;
        if u128::from(expires) < now_ms() {
            return None;
        }
        stored.get("value")?.as_str().map(ToOwned::to_owned)
    }

    async fn set(&self, key: &str, value: String, ttl: Duration) {
        let expires = now_ms() + ttl.as_millis();
        let _ = self.0.write_json(
            &cache_key(key),
            &json!({"value": value, "expires": u64::try_from(expires).unwrap_or(u64::MAX)}),
        );
    }

    async fn delete(&self, key: &str) {
        let _ = self.0.write_json(&cache_key(key), &Value::Null);
    }
}

const ANIME_LIST_URL: &str =
    "https://raw.githubusercontent.com/Fribb/anime-lists/master/anime-list-full.json";
const ANIME_LIST_TTL: Duration = Duration::from_secs(7 * 24 * 60 * 60);

async fn anime_resolver(client: &Client, cache: &StorageCache) -> Resolver {
    let body = match cache.get("anime-list").await {
        Some(body) => body,
        None => match client.get(ANIME_LIST_URL).send().await {
            Ok(response) if response.status().is_success() => {
                let Ok(body) = response.text().await else {
                    return Resolver::default();
                };
                cache.set("anime-list", body.clone(), ANIME_LIST_TTL).await;
                body
            }
            _ => return Resolver::default(),
        },
    };
    Resolver {
        anime: AnimeIdTable::from_fribb(&body).ok(),
        ..Resolver::default()
    }
}

impl EffectExecutor {
    pub(super) fn stream_prefs(&self) -> Value {
        self.current_profile_id()
            .ok()
            .flatten()
            .and_then(|id| self.profile_prefs(&id).ok().flatten())
            .unwrap_or(Value::Null)
    }

    pub(super) async fn aggregated_meta(
        &self,
        prefs: &Value,
        kind: &str,
        id: &str,
    ) -> Option<Meta> {
        let api_key = prefs.get("tmdbApiKey")?.as_str()?.trim();
        if api_key.is_empty() {
            return None;
        }
        let client = Client::builder()
            .bundled_roots()
            .user_agent("Fluxa/1.0")
            .native_timeout(Duration::from_secs(15))
            .build()
            .ok()?;
        let config = MetaConfig {
            tmdb_api_key: api_key.to_owned(),
            language: prefs
                .get("metadatakitLanguage")
                .and_then(Value::as_str)
                .filter(|language| !language.is_empty())
                .unwrap_or("en-US")
                .to_owned(),
            ..MetaConfig::default()
        };
        let cache = StorageCache(self.storage.clone());
        let resolver = if prefs
            .get("metadatakitAnimeMapping")
            .and_then(Value::as_bool)
            .unwrap_or(true)
        {
            anime_resolver(&client, &cache).await
        } else {
            Resolver::default()
        };
        let service = MetaService::new(ClientFetcher(client), cache, config, resolver);
        let content = if matches!(kind, "series" | "tv" | "show") {
            ContentType::Series
        } else {
            ContentType::Movie
        };
        let base = id.split(':').next().unwrap_or(id);
        service
            .meta(content, base, chrono::Utc::now())
            .await
            .ok()
            .map(|outcome| outcome.meta)
    }

    pub(super) async fn stream_context(&self, prefs: &Value, request: &Value) -> RequestContext {
        let kind = request
            .get("contentType")
            .and_then(Value::as_str)
            .unwrap_or("movie");
        let id = request
            .get("requestIds")
            .and_then(Value::as_array)
            .and_then(|ids| ids.first())
            .and_then(Value::as_str)
            .unwrap_or_default();
        let detail = request
            .get("detail")
            .filter(|detail| detail.is_object())
            .and_then(|detail| serde_json::from_value::<Meta>(detail.clone()).ok())
            .filter(|meta| !meta.name.trim().is_empty());
        let meta = match detail {
            Some(meta) => Some(meta),
            None if stream_pipeline::metadatakit_enabled(prefs) => {
                self.aggregated_meta(prefs, kind, id).await
            }
            None => None,
        };
        stream_pipeline::request_context(kind, id, meta.as_ref())
    }

    pub(super) fn run_stream_kit(
        &self,
        context: &RequestContext,
        parts: Vec<Vec<Value>>,
    ) -> stream_pipeline::Outcome {
        let prefs = self.stream_prefs();
        let parsed = parts
            .into_iter()
            .flat_map(|streams| {
                let addon = streams
                    .first()
                    .and_then(|stream| stream.get("addonName")?.as_str())
                    .unwrap_or("Addon")
                    .to_owned();
                stream_pipeline::parse_values(streams, &addon)
            })
            .collect();
        stream_pipeline::run(&prefs, context, parsed)
    }
}
