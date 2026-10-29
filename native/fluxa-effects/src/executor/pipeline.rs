use super::streams::now_ms;
use super::*;
use fluxa_aggmeta::service::{
    FetchError, Fetcher, HttpRequest, HttpResponse, KvCache, MetaConfig, MetaService,
};
use fluxa_aggmeta::types::{ContentType, Meta};
use fluxa_core::stream_pipeline;
use fluxa_streamkit::fetch::{Transport, TransportError, TransportResponse};
use fluxa_streamkit::types::RequestContext;

pub(super) struct ClientTransport(pub Client);

impl Transport for ClientTransport {
    async fn get(
        &self,
        url: &str,
        _timeout: Duration,
    ) -> Result<TransportResponse, TransportError> {
        let (status, body) = fetch_text(&self.0, url)
            .await
            .map_err(TransportError::Network)?;
        Ok(TransportResponse {
            status: u16::try_from(status).unwrap_or(0),
            body: body.unwrap_or_default(),
        })
    }
}

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

impl EffectExecutor {
    pub(super) fn stream_prefs(&self) -> Value {
        self.current_profile_id()
            .ok()
            .flatten()
            .and_then(|id| self.profile_prefs(&id).ok().flatten())
            .unwrap_or(Value::Null)
    }

    async fn aggregated_meta(&self, prefs: &Value, kind: &str, id: &str) -> Option<Meta> {
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
                .get("language")
                .and_then(Value::as_str)
                .unwrap_or("en-US")
                .to_owned(),
            ..MetaConfig::default()
        };
        let service = MetaService::new(
            ClientFetcher(client),
            StorageCache(self.storage.clone()),
            config,
            Default::default(),
        );
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
            None if stream_pipeline::enabled(prefs) => self.aggregated_meta(prefs, kind, id).await,
            None => None,
        };
        stream_pipeline::request_context(kind, id, meta.as_ref())
    }
}
