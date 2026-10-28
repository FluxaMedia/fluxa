use std::collections::HashMap;
use std::future::Future;
use std::time::Duration;

use chrono::{DateTime, Utc};
use serde_json::Value;

use crate::age::AgeGate;
use crate::art::ArtPolicy;
use crate::cache::{MetaKey, NegativeKind, Stored, TtlClass, store_ttl};
use crate::ids::{IdSet, normalise_request_id};
use crate::idtables::Resolver;
use crate::numbering::{self, SeasonNumbering};
use crate::tmdb::{self, BuildOptions};
use crate::types::{ContentType, Meta, Provider, default_provider, fallback_chain};

#[derive(Clone, Debug)]
pub struct HttpRequest {
    pub url: String,
    pub post_body: Option<String>,
    pub headers: Vec<(String, String)>,
}

#[derive(Clone, Debug)]
pub struct HttpResponse {
    pub status: u16,
    pub body: String,
}

#[derive(Clone, Debug)]
pub enum FetchError {
    Network(String),
}

pub trait Fetcher {
    fn fetch(&self, req: HttpRequest) -> impl Future<Output = Result<HttpResponse, FetchError>>;
}

pub trait KvCache {
    fn get(&self, key: &str) -> impl Future<Output = Option<String>>;
    fn set(&self, key: &str, value: String, ttl: Duration) -> impl Future<Output = ()>;
    fn delete(&self, key: &str) -> impl Future<Output = ()>;
}

#[derive(Clone, Debug)]
pub struct MetaConfig {
    pub language: String,
    pub providers: HashMap<ContentType, Provider>,
    pub art: ArtPolicy,
    pub tmdb_api_key: String,
    pub age_gate: AgeGate,
    pub season_numbering: SeasonNumbering,
    pub per_show_numbering: HashMap<String, SeasonNumbering>,
    pub episode_order: String,
}

impl Default for MetaConfig {
    fn default() -> Self {
        Self {
            language: "en-US".into(),
            providers: HashMap::new(),
            art: ArtPolicy::default(),
            tmdb_api_key: String::new(),
            age_gate: AgeGate::default(),
            season_numbering: SeasonNumbering::Sequential,
            per_show_numbering: HashMap::new(),
            episode_order: "official".into(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum MetaError {
    NotFound,
    Blocked,
    Unavailable(String),
}

pub struct MetaOutcome {
    pub meta: Meta,
    pub degraded: bool,
    pub provider: Provider,
}

pub struct MetaService<F, K> {
    pub fetcher: F,
    pub cache: K,
    pub config: MetaConfig,
    pub resolver: Resolver,
}

impl<F: Fetcher, K: KvCache> MetaService<F, K> {
    pub fn new(fetcher: F, cache: K, config: MetaConfig, resolver: Resolver) -> Self {
        Self {
            fetcher,
            cache,
            config,
            resolver,
        }
    }

    fn primary_for(&self, kind: ContentType) -> Provider {
        self.config
            .providers
            .get(&kind)
            .copied()
            .filter(|p| p.allowed_for(kind))
            .unwrap_or_else(|| default_provider(kind))
    }

    async fn get_json(&self, url: String) -> Result<Value, MetaError> {
        let sep = if url.contains('?') { '&' } else { '?' };
        let full = format!("{url}{sep}api_key={}", self.config.tmdb_api_key);
        let mut last = MetaError::Unavailable("no attempt".into());
        for _ in 0..3 {
            match self
                .fetcher
                .fetch(HttpRequest {
                    url: full.clone(),
                    post_body: None,
                    headers: Vec::new(),
                })
                .await
            {
                Ok(r) if r.status == 200 => {
                    return serde_json::from_str(&r.body)
                        .map_err(|e| MetaError::Unavailable(e.to_string()));
                }
                Ok(r) if r.status == 404 => return Err(MetaError::NotFound),
                Ok(r) if !tmdb::should_retry(r.status) => {
                    return Err(MetaError::Unavailable(format!("status {}", r.status)));
                }
                Ok(r) => last = MetaError::Unavailable(format!("status {}", r.status)),
                Err(FetchError::Network(e)) => last = MetaError::Unavailable(e),
            }
        }
        Err(last)
    }

    async fn tmdb_meta(
        &self,
        kind: ContentType,
        ids: &IdSet,
        requested: &str,
        now: DateTime<Utc>,
    ) -> Result<Meta, MetaError> {
        let media = if kind == ContentType::Movie {
            "movie"
        } else {
            "tv"
        };
        let tmdb_id = match ids.tmdb {
            Some(id) => id,
            None => {
                let imdb = ids.imdb.as_deref().ok_or(MetaError::NotFound)?;
                let found = self.get_json(tmdb::find_url(imdb, "imdb_id")).await?;
                let key = if media == "movie" {
                    "movie_results"
                } else {
                    "tv_results"
                };
                found
                    .get(key)
                    .and_then(Value::as_array)
                    .and_then(|a| a.first())
                    .and_then(|r| r.get("id"))
                    .and_then(Value::as_u64)
                    .ok_or(MetaError::NotFound)?
            }
        };
        let append = if media == "movie" {
            "release_dates"
        } else {
            "content_ratings"
        };
        let detail = self
            .get_json(tmdb::detail_url(
                media,
                tmdb_id,
                &self.config.language,
                append,
            ))
            .await?;
        let opts = BuildOptions {
            language: self.config.language.clone(),
            art: self.config.art.clone(),
            requested_id: requested.to_string(),
        };
        let mut meta = tmdb::build_meta(media, &detail, &opts);
        meta.p_tmdb = Some(tmdb_id);
        meta.app_extras.certification = certification(&detail, media);
        if media == "tv" {
            let numbers: Vec<u32> = detail
                .get("seasons")
                .and_then(Value::as_array)
                .map(|s| {
                    s.iter()
                        .filter(|x| tmdb::season_is_valid(x))
                        .filter_map(|x| x.get("season_number").and_then(Value::as_u64))
                        .map(|n| n as u32)
                        .collect()
                })
                .unwrap_or_default();
            let mut seasons = Vec::new();
            for batch in tmdb::season_append_batches(&numbers) {
                let url = format!(
                    "{}/tv/{tmdb_id}?language={}&append_to_response={batch}",
                    tmdb::API_BASE,
                    self.config.language
                );
                let body = self.get_json(url).await?;
                for n in &numbers {
                    if let Some(s) = body.get(format!("season/{n}")) {
                        seasons.push(s.clone());
                    }
                }
            }
            let origin = detail
                .get("origin_country")
                .and_then(Value::as_array)
                .and_then(|a| a.first())
                .and_then(Value::as_str);
            let show_id = if meta.id.is_empty() {
                requested.to_string()
            } else {
                meta.id.clone()
            };
            meta.videos = Some(tmdb::build_videos(
                &show_id,
                &seasons,
                meta.background.as_deref(),
                origin,
                now,
            ));
        }
        Ok(meta)
    }

    pub async fn meta(
        &self,
        kind: ContentType,
        id: &str,
        now: DateTime<Utc>,
    ) -> Result<MetaOutcome, MetaError> {
        let requested = normalise_request_id(id);
        let mut ids = IdSet::from_prefixed(&requested);
        let resolved = self
            .resolver
            .resolve_all(ids.clone(), kind == ContentType::Movie);
        ids = resolved.ids;
        let effective = if ids.is_anime_id() && kind == ContentType::Series {
            ContentType::Anime
        } else {
            kind
        };
        let primary = self.primary_for(effective);
        let show_key = ids.tvdb.map(|n| format!("tvdb:{n}")).unwrap_or_default();
        let numbering = numbering::resolve_numbering(
            &show_key,
            &self.config.per_show_numbering,
            self.config.season_numbering,
        );

        let mut last_err = MetaError::NotFound;
        for (index, provider) in fallback_chain(primary).into_iter().enumerate() {
            let key = MetaKey {
                kind: effective.as_str(),
                id: &requested,
                language: &self.config.language,
                provider: provider.as_str(),
                episode_order: &self.config.episode_order,
                season_numbering: numbering.as_str(),
                signature: "",
            }
            .build();
            let cached = match self.cache.get(&key).await {
                Some(raw) => match serde_json::from_str::<Meta>(&raw) {
                    Ok(m) if !numbering::needs_refetch(&m, numbering) => Some(m),
                    Ok(_) => None,
                    Err(_) => {
                        self.cache.delete(&key).await;
                        None
                    }
                },
                None => None,
            };
            let degraded = index > 0;
            let meta = match cached {
                Some(m) => m,
                None => {
                    if provider != Provider::Tmdb {
                        continue;
                    }
                    match self.tmdb_meta(effective, &ids, &requested, now).await {
                        Ok(mut m) => {
                            m.season_numbering = None;
                            if let (Some(ttl), Ok(raw)) = (
                                store_ttl(TtlClass::Meta, degraded),
                                serde_json::to_string(&m),
                            ) {
                                self.cache.set(&key, raw, ttl).await;
                            }
                            m
                        }
                        Err(e) => {
                            last_err = e;
                            continue;
                        }
                    }
                }
            };
            let mut meta = meta;
            numbering::project(&mut meta, numbering);
            let certification = meta.app_extras.certification.clone().unwrap_or_default();
            if !self
                .config
                .age_gate
                .allows(&certification, effective != ContentType::Movie)
            {
                return Err(MetaError::Blocked);
            }
            return Ok(MetaOutcome {
                meta,
                degraded,
                provider,
            });
        }
        let negative = match last_err {
            MetaError::NotFound => NegativeKind::NotFound,
            _ => NegativeKind::TemporaryError,
        };
        let key = format!("neg|{requested}");
        if let Ok(raw) = serde_json::to_string(&matches!(negative, NegativeKind::NotFound)) {
            self.cache.set(&key, raw, negative.ttl()).await;
        }
        let _ = Stored::<()>::Negative(negative);
        Err(last_err)
    }
}

fn certification(detail: &Value, media: &str) -> Option<String> {
    if media == "movie" {
        detail
            .get("release_dates")
            .and_then(|r| r.get("results"))
            .and_then(Value::as_array)?
            .iter()
            .find(|c| c.get("iso_3166_1").and_then(Value::as_str) == Some("US"))?
            .get("release_dates")
            .and_then(Value::as_array)?
            .iter()
            .filter_map(|d| d.get("certification").and_then(Value::as_str))
            .find(|c| !c.is_empty())
            .map(String::from)
    } else {
        detail
            .get("content_ratings")
            .and_then(|r| r.get("results"))
            .and_then(Value::as_array)?
            .iter()
            .find(|c| c.get("iso_3166_1").and_then(Value::as_str) == Some("US"))?
            .get("rating")
            .and_then(Value::as_str)
            .map(String::from)
    }
}
