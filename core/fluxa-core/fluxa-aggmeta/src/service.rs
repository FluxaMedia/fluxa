use std::collections::HashMap;
use std::future::Future;
use std::time::Duration;

use chrono::{DateTime, Utc};
use serde_json::Value;

use crate::age::AgeGate;
use crate::art::ArtPolicy;
use crate::art::select_image;
use crate::cache::{MetaKey, NegativeKind, Stored, TtlClass, store_ttl};
use crate::catalog::ProviderPage;
use crate::ids::{IdSet, normalise_request_id};
use crate::idtables::Resolver;
use crate::merge;
use crate::numbering::{self, SeasonNumbering};
use crate::providers::{
    anidb, anilist, fanart, gemini, imdb, jikan, kitsu, mal_list, mdblist, rpdb, tvdb, tvmaze,
};
use crate::search::{self, SearchHit};
use crate::tmdb::{self, BuildOptions};
use crate::tmdb_catalog;
use crate::types::{CatalogPage, ContentType, Meta, Provider, default_provider, fallback_chain};

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
    pub fanart_first: bool,
    pub anime_renumber: bool,
    pub tmdb_api_key: String,
    pub tvdb_api_key: String,
    pub jikan_url: String,
    pub anidb_client: String,
    pub fanart_api_key: String,
    pub rpdb_api_key: String,
    pub gemini_api_key: String,
    pub mdblist_api_key: String,
    pub anilist_token: String,
    pub tvdb_search: Vec<ContentType>,
    pub cache_hours: Option<u64>,
    pub ai_search: bool,
    pub catalogs: Vec<tmdb_catalog::CatalogDef>,
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
            fanart_first: true,
            anime_renumber: false,
            tmdb_api_key: String::new(),
            tvdb_api_key: String::new(),
            jikan_url: jikan::DEFAULT_BASE.into(),
            anidb_client: String::new(),
            fanart_api_key: String::new(),
            rpdb_api_key: String::new(),
            gemini_api_key: String::new(),
            mdblist_api_key: String::new(),
            anilist_token: String::new(),
            tvdb_search: Vec::new(),
            cache_hours: None,
            ai_search: false,
            catalogs: tmdb_catalog::defaults(),
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
        self.send_json(HttpRequest {
            url: full,
            post_body: None,
            headers: Vec::new(),
        })
        .await
    }

    async fn plain_json(&self, url: String) -> Result<Value, MetaError> {
        self.send_json(HttpRequest {
            url,
            post_body: None,
            headers: vec![("Accept".into(), "application/json".into())],
        })
        .await
    }

    async fn anilist_viewer(&self) -> Result<String, MetaError> {
        use std::hash::{Hash, Hasher};
        let token = self.config.anilist_token.trim();
        if token.is_empty() {
            return Err(MetaError::NotFound);
        }
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        token.hash(&mut hasher);
        let key = format!("anilist-viewer|{:x}", hasher.finish());
        if let Some(name) = self.cache.get(&key).await {
            return Ok(name);
        }
        let body = self.send_json(anilist::viewer_request(token)).await?;
        let name = anilist::viewer_name(&body).ok_or(MetaError::NotFound)?;
        self.cache
            .set(&key, name.clone(), Duration::from_secs(86_400))
            .await;
        Ok(name)
    }

    async fn send_json(&self, request: HttpRequest) -> Result<Value, MetaError> {
        let mut last = MetaError::Unavailable("no attempt".into());
        for _ in 0..3 {
            match self.fetcher.fetch(request.clone()).await {
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

    async fn tvdb_token(&self) -> Result<String, MetaError> {
        let key = self.config.tvdb_api_key.trim();
        if key.is_empty() {
            return Err(MetaError::NotFound);
        }
        if let Some(token) = self.cache.get("tvdb-token").await {
            return Ok(token);
        }
        let login = self.send_json(tvdb::login_request(key)).await?;
        let token = tvdb::token(&login)
            .ok_or_else(|| MetaError::Unavailable("tvdb login rejected".into()))?;
        self.cache
            .set(
                "tvdb-token",
                token.clone(),
                Duration::from_secs(25 * 86_400),
            )
            .await;
        Ok(token)
    }

    async fn tvdb_meta(
        &self,
        kind: ContentType,
        ids: &IdSet,
        requested: &str,
        now: DateTime<Utc>,
    ) -> Result<Meta, MetaError> {
        let id = ids.tvdb.ok_or(MetaError::NotFound)?;
        let movie = kind == ContentType::Movie;
        let token = self.tvdb_token().await?;
        let detail = self
            .send_json(tvdb::get(&tvdb::detail_path(movie, id), &token))
            .await?;
        let mut meta =
            tvdb::build_meta(&detail, movie, requested, Vec::new()).ok_or(MetaError::NotFound)?;
        if !movie {
            let mut pages = Vec::new();
            for page in 0..tvdb::MAX_EPISODE_PAGES {
                let body = self
                    .send_json(tvdb::get(
                        &tvdb::episodes_path(id, &self.config.episode_order, page),
                        &token,
                    ))
                    .await?;
                let more = tvdb::has_next_page(&body);
                pages.push(body);
                if !more {
                    break;
                }
            }
            meta.videos = Some(tvdb::build_videos(&pages, &meta.id, now));
        }
        Ok(meta)
    }

    async fn tvmaze_meta(
        &self,
        kind: ContentType,
        ids: &IdSet,
        requested: &str,
        now: DateTime<Utc>,
    ) -> Result<Meta, MetaError> {
        if kind == ContentType::Movie {
            return Err(MetaError::NotFound);
        }
        let url = tvmaze::show_url(ids).ok_or(MetaError::NotFound)?;
        let mut show = self.plain_json(url).await?;
        if tvmaze::needs_embed(&show) {
            let id = show
                .get("id")
                .and_then(Value::as_u64)
                .ok_or(MetaError::NotFound)?;
            show = self.plain_json(tvmaze::embedded_url(id)).await?;
        }
        tvmaze::build_meta(&show, requested, now).ok_or(MetaError::NotFound)
    }

    async fn imdb_meta(
        &self,
        kind: ContentType,
        ids: &IdSet,
        _requested: &str,
    ) -> Result<Meta, MetaError> {
        let id = ids.imdb.as_deref().ok_or(MetaError::NotFound)?;
        let body = self.plain_json(imdb::suggestion_url(id)).await?;
        imdb::build_meta(&body, id, kind == ContentType::Movie).ok_or(MetaError::NotFound)
    }

    async fn anilist_meta(
        &self,
        ids: &IdSet,
        requested: &str,
        now: DateTime<Utc>,
    ) -> Result<Meta, MetaError> {
        let request = anilist::request(ids).ok_or(MetaError::NotFound)?;
        let body = self.send_json(request).await?;
        anilist::build_meta(&body, requested, now).ok_or(MetaError::NotFound)
    }

    async fn kitsu_meta(
        &self,
        ids: &IdSet,
        requested: &str,
        now: DateTime<Utc>,
    ) -> Result<Meta, MetaError> {
        let id = ids.kitsu.ok_or(MetaError::NotFound)?;
        let doc = self.plain_json(kitsu::detail_url(id)).await?;
        let mut meta = kitsu::build_meta(&doc, requested, Vec::new()).ok_or(MetaError::NotFound)?;
        if meta.kind == "series" {
            let mut pages = Vec::new();
            for page in 0..kitsu::MAX_EPISODE_PAGES {
                let body = self
                    .plain_json(kitsu::episodes_url(id, page * kitsu::EPISODE_PAGE))
                    .await?;
                let more = kitsu::has_next_page(&body);
                pages.push(body);
                if !more {
                    break;
                }
            }
            meta.videos = Some(kitsu::build_videos(&pages, &meta.id, now));
        }
        Ok(meta)
    }

    async fn mal_meta(
        &self,
        ids: &IdSet,
        requested: &str,
        now: DateTime<Utc>,
    ) -> Result<Meta, MetaError> {
        let id = ids.mal.ok_or(MetaError::NotFound)?;
        let base = &self.config.jikan_url;
        let doc = self.plain_json(jikan::detail_url(base, id)).await?;
        let mut meta = jikan::build_meta(&doc, requested, Vec::new()).ok_or(MetaError::NotFound)?;
        if meta.kind == "series" {
            let mut pages = Vec::new();
            for page in 1..=jikan::MAX_EPISODE_PAGES {
                let body = self.plain_json(jikan::episodes_url(base, id, page)).await?;
                let more = jikan::has_next_page(&body);
                pages.push(body);
                if !more {
                    break;
                }
            }
            let runtime = doc
                .pointer("/data/duration")
                .and_then(Value::as_str)
                .and_then(jikan::minutes);
            meta.videos = Some(jikan::build_videos(&pages, &meta.id, runtime, now));
        }
        Ok(meta)
    }

    async fn anidb_meta(
        &self,
        ids: &IdSet,
        requested: &str,
        now: DateTime<Utc>,
    ) -> Result<Meta, MetaError> {
        let id = ids.anidb.ok_or(MetaError::NotFound)?;
        let url = anidb::request_url(&self.config.anidb_client, id).ok_or(MetaError::NotFound)?;
        let response = self
            .fetcher
            .fetch(HttpRequest {
                url,
                post_body: None,
                headers: Vec::new(),
            })
            .await
            .map_err(|FetchError::Network(e)| MetaError::Unavailable(e))?;
        match response.status {
            200 => anidb::build_meta(&response.body, requested, now).ok_or(MetaError::NotFound),
            404 => Err(MetaError::NotFound),
            status => Err(MetaError::Unavailable(format!("status {status}"))),
        }
    }

    async fn fanart_artwork(&self, movie: bool, ids: &IdSet) -> Option<fanart::Artwork> {
        let key = self.config.fanart_api_key.trim();
        if key.is_empty() {
            return None;
        }
        let id = if movie {
            ids.tmdb.map(|n| n.to_string()).or_else(|| ids.imdb.clone())
        } else {
            ids.tvdb.map(|n| n.to_string())
        }?;
        let cache_key = format!("fanart|{}|{id}", if movie { "m" } else { "t" });
        let doc = match self.cache.get(&cache_key).await {
            Some(raw) => serde_json::from_str(&raw).ok()?,
            None => {
                let doc = self
                    .plain_json(fanart::url(movie, &id, key))
                    .await
                    .unwrap_or(Value::Null);
                self.cache
                    .set(&cache_key, doc.to_string(), Duration::from_secs(3 * 86_400))
                    .await;
                doc
            }
        };
        Some(fanart::artwork(&doc, movie))
    }

    async fn apply_art(&self, meta: &mut Meta, kind: ContentType, ids: &IdSet) {
        let movie = kind == ContentType::Movie;
        if let Some(art) = self.fanart_artwork(movie, ids).await {
            let policy = &self.config.art;
            let pick = |items: &[crate::art::ImageCandidate]| {
                select_image(items, policy).map(|c| c.file_path.clone())
            };
            let keep = |current: &Option<String>, found: Option<String>| {
                if self.config.fanart_first || current.is_none() {
                    found
                } else {
                    None
                }
            };
            let poster = keep(&meta.poster, pick(&art.posters));
            let background = keep(&meta.background, pick(&art.backgrounds));
            let logo = keep(&meta.logo, pick(&art.logos));
            merge::overlay_art(meta, poster, background, logo, None);
        }
        let imdb = meta
            .imdb_id
            .clone()
            .or_else(|| ids.imdb.clone())
            .unwrap_or_default();
        if let Some(poster) = rpdb::poster_url(&self.config.rpdb_api_key, &imdb) {
            meta.poster = Some(poster);
        }
    }

    async fn card_art(&self, metas: &mut [Meta], kind: ContentType) {
        if self.config.fanart_api_key.trim().is_empty()
            && self.config.rpdb_api_key.trim().is_empty()
        {
            return;
        }
        for meta in metas {
            let ids = self
                .resolver
                .resolve_all(IdSet::from_prefixed(&meta.id), kind == ContentType::Movie)
                .ids;
            let mut card = meta.clone();
            self.apply_art(&mut card, kind, &ids).await;
            meta.poster = card.poster;
        }
    }

    async fn provider_meta(
        &self,
        provider: Provider,
        kind: ContentType,
        ids: &IdSet,
        requested: &str,
        now: DateTime<Utc>,
    ) -> Result<Meta, MetaError> {
        match provider {
            Provider::Tmdb => self.tmdb_meta(kind, ids, requested, now).await,
            Provider::Tvdb => self.tvdb_meta(kind, ids, requested, now).await,
            Provider::Tvmaze => self.tvmaze_meta(kind, ids, requested, now).await,
            Provider::Imdb => self.imdb_meta(kind, ids, requested).await,
            Provider::Anilist => self.anilist_meta(ids, requested, now).await,
            Provider::Kitsu => self.kitsu_meta(ids, requested, now).await,
            Provider::Mal => self.mal_meta(ids, requested, now).await,
            Provider::Anidb => self.anidb_meta(ids, requested, now).await,
        }
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

    pub async fn catalog(
        &self,
        kind: ContentType,
        id: &str,
        skip: usize,
        limit: usize,
    ) -> Result<CatalogPage, MetaError> {
        let def = tmdb_catalog::find(&self.config.catalogs, kind, id).ok_or(MetaError::NotFound)?;
        let size = tmdb_catalog::PAGE_SIZE;
        let viewer = match &def.source {
            tmdb_catalog::Source::AnilistList { user, .. } if user == anilist::VIEWER_USER => {
                Some(self.anilist_viewer().await?)
            }
            _ => None,
        };
        let discover_key;
        let region = match &def.source {
            tmdb_catalog::Source::Service { region, .. } => region.as_str(),
            tmdb_catalog::Source::AnilistList { user, .. } => {
                viewer.as_deref().unwrap_or(user.as_str())
            }
            tmdb_catalog::Source::Discover { genres, from, to } => {
                discover_key = format!("{genres}:{from:?}:{to:?}");
                discover_key.as_str()
            }
            tmdb_catalog::Source::AnimeCustom {
                genre,
                season,
                year,
                tag,
            } => {
                discover_key = format!("{genre}:{season}:{year:?}:{tag}");
                discover_key.as_str()
            }
            tmdb_catalog::Source::MalList { user, .. } => user.as_str(),
            _ => "",
        };
        let personal = matches!(
            def.source,
            tmdb_catalog::Source::AnilistList { .. }
                | tmdb_catalog::Source::MdblistList(_)
                | tmdb_catalog::Source::MalList { .. }
        );
        let ttl = Duration::from_secs(if personal {
            10 * 60
        } else {
            self.config.cache_hours.unwrap_or(24) * 60 * 60
        });
        let mut page = (skip / size) as u32 + 1;
        let mut offset = skip % size;
        let mut metas = Vec::with_capacity(limit);
        let mut has_more = true;
        while metas.len() < limit && has_more {
            let key = format!(
                "cat:{id}:{}:{}:{}:{region}:{page}",
                kind.as_str(),
                self.config.language,
                self.config.age_gate.movie_cap().unwrap_or("none")
            );
            let body = match self.cache.get(&key).await {
                Some(cached) => serde_json::from_str(&cached).unwrap_or(Value::Null),
                None => {
                    let body = match &def.source {
                        tmdb_catalog::Source::Anime(list) => {
                            self.send_json(anilist::catalog_request(*list, page))
                                .await?
                        }
                        tmdb_catalog::Source::AnilistList { user, status } => {
                            let name = viewer.as_deref().unwrap_or(user.as_str());
                            let token = if viewer.is_some() {
                                self.config.anilist_token.trim()
                            } else {
                                ""
                            };
                            self.send_json(anilist::user_list_request(name, status, token))
                                .await?
                        }
                        tmdb_catalog::Source::AnimeCustom {
                            genre,
                            season,
                            year,
                            tag,
                        } => {
                            self.send_json(anilist::custom_request(genre, season, *year, tag, page))
                                .await?
                        }
                        tmdb_catalog::Source::MalList { user, status } => {
                            self.send_json(mal_list::request(user, *status)).await?
                        }
                        tmdb_catalog::Source::MdblistList(list) => {
                            let key = self.config.mdblist_api_key.trim();
                            if key.is_empty() {
                                return Err(MetaError::NotFound);
                            }
                            self.plain_json(mdblist::items_url(list, key, page)).await?
                        }
                        _ => {
                            let url = tmdb_catalog::page_url(
                                def,
                                page,
                                &self.config.language,
                                &self.config.age_gate,
                            )
                            .ok_or(MetaError::NotFound)?;
                            self.get_json(url).await?
                        }
                    };
                    self.cache.set(&key, body.to_string(), ttl).await;
                    body
                }
            };
            let fetched = match def.source {
                tmdb_catalog::Source::Anime(_) | tmdb_catalog::Source::AnimeCustom { .. } => {
                    anilist::parse_catalog(&body)
                }
                tmdb_catalog::Source::MalList { .. } => mal_list::parse(&body, page),
                tmdb_catalog::Source::AnilistList { .. } => anilist::parse_user_list(&body, page),
                tmdb_catalog::Source::MdblistList(_) => mdblist::parse_items(kind, &body),
                _ => tmdb_catalog::parse_page(kind, &body),
            };
            has_more = fetched.has_next_page;
            let room = limit - metas.len();
            metas.extend(fetched.items.into_iter().skip(offset).take(room));
            offset = 0;
            page += 1;
        }
        self.card_art(&mut metas, kind).await;
        Ok(CatalogPage { metas, has_more })
    }

    fn uses_tvdb_search(&self, kind: ContentType) -> bool {
        !self.config.tvdb_api_key.trim().is_empty() && self.config.tvdb_search.contains(&kind)
    }

    async fn search_page(
        &self,
        anime: bool,
        kind: ContentType,
        query: &str,
        page: u32,
    ) -> Result<ProviderPage<Meta>, MetaError> {
        let key = format!(
            "search:{}{}:{}:{}:{}:{page}",
            if anime { "anime" } else { kind.as_str() },
            if !anime && self.uses_tvdb_search(kind) {
                "+tvdb"
            } else {
                ""
            },
            self.config.language,
            self.config.age_gate.movie_cap().unwrap_or("none"),
            search::cache_query(query)
        );
        let body = match self.cache.get(&key).await {
            Some(cached) => serde_json::from_str(&cached).unwrap_or(Value::Null),
            None => {
                let body = if anime {
                    self.send_json(anilist::search_request(query, page)).await?
                } else if self.uses_tvdb_search(kind) {
                    let token = self.tvdb_token().await?;
                    let movie = kind == ContentType::Movie;
                    self.send_json(tvdb::get(&tvdb::search_path(movie, query, page), &token))
                        .await?
                } else {
                    let url = tmdb_catalog::search_url(kind, query, page, &self.config.language);
                    self.get_json(url).await?
                };
                self.cache
                    .set(&key, body.to_string(), Duration::from_secs(60 * 60))
                    .await;
                body
            }
        };
        Ok(if anime {
            anilist::parse_catalog(&body)
        } else if self.uses_tvdb_search(kind) {
            tvdb::parse_search(kind == ContentType::Movie, &body)
        } else {
            tmdb_catalog::parse_page(kind, &body)
        })
    }

    async fn ai_hits(&self, anime: bool, kind: ContentType, query: &str) -> Vec<SearchHit> {
        let key = format!(
            "ai:{}:{}",
            if anime { "anime" } else { kind.as_str() },
            search::cache_query(query)
        );
        let titles: Vec<String> = match self.cache.get(&key).await {
            Some(cached) => serde_json::from_str(&cached).unwrap_or_default(),
            None => {
                let request = gemini::request(
                    self.config.gemini_api_key.trim(),
                    query,
                    anime,
                    kind == ContentType::Movie,
                );
                let titles = match self.send_json(request).await {
                    Ok(body) => gemini::titles(&body),
                    Err(_) => return Vec::new(),
                };
                if let Ok(raw) = serde_json::to_string(&titles) {
                    self.cache
                        .set(&key, raw, Duration::from_secs(24 * 60 * 60))
                        .await;
                }
                titles
            }
        };
        let mut hits = Vec::new();
        for title in titles {
            if let Ok(page) = self.search_page(anime, kind, &title, 1).await {
                hits.extend(page.items.into_iter().take(1).map(|meta| SearchHit {
                    ids: IdSet::from_prefixed(&meta.id),
                    meta,
                }));
            }
        }
        hits
    }

    pub async fn search(
        &self,
        anime: bool,
        kind: ContentType,
        query: &str,
        skip: usize,
        limit: usize,
    ) -> Result<CatalogPage, MetaError> {
        let query = query.trim();
        if query.is_empty() {
            return Ok(CatalogPage {
                metas: Vec::new(),
                has_more: false,
            });
        }
        let ai = if skip == 0
            && self.config.ai_search
            && !self.config.gemini_api_key.trim().is_empty()
        {
            self.ai_hits(anime, kind, query).await
        } else {
            Vec::new()
        };
        let size = tmdb_catalog::PAGE_SIZE;
        let mut page = (skip / size) as u32 + 1;
        let mut offset = skip % size;
        let mut plain = Vec::new();
        let mut has_more = true;
        let mut failure = None;
        while plain.len() < limit && has_more {
            match self.search_page(anime, kind, query, page).await {
                Ok(fetched) => {
                    has_more = fetched.has_next_page;
                    let room = limit - plain.len();
                    plain.extend(fetched.items.into_iter().skip(offset).take(room));
                }
                Err(error) => {
                    failure = Some(error);
                    has_more = false;
                }
            }
            offset = 0;
            page += 1;
        }
        if let (Some(error), true) = (failure, plain.is_empty() && ai.is_empty()) {
            return Err(error);
        }
        let plain_hits = plain
            .into_iter()
            .map(|meta| SearchHit {
                ids: IdSet::from_prefixed(&meta.id),
                meta,
            })
            .collect();
        let mut metas = search::merge_results(vec![ai, plain_hits]);
        if !anime && self.config.age_gate.cap.is_some() {
            let now = chrono::Utc::now();
            let mut allowed = Vec::with_capacity(metas.len());
            for meta in metas {
                let blocked = meta.id.starts_with("tmdb:")
                    && self.meta(kind, &meta.id, now).await.err() == Some(MetaError::Blocked);
                if !blocked {
                    allowed.push(meta);
                }
            }
            metas = allowed;
        }
        self.card_art(&mut metas, kind).await;
        Ok(CatalogPage { metas, has_more })
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
                    match self
                        .provider_meta(provider, effective, &ids, &requested, now)
                        .await
                    {
                        Ok(mut m) => {
                            m.season_numbering = None;
                            if index == 0 && provider != Provider::Tmdb && lacks_basics(&m) {
                                if let Ok(secondary) =
                                    self.tmdb_meta(effective, &ids, &requested, now).await
                                {
                                    merge::fill_missing(&mut m, &secondary);
                                }
                            }
                            if let (Some(ttl), Ok(raw)) = (
                                store_ttl(TtlClass::Meta, degraded).map(|ttl| {
                                    self.config
                                        .cache_hours
                                        .map_or(ttl, |hours| Duration::from_secs(hours * 3600))
                                }),
                                serde_json::to_string(&m),
                            ) {
                                self.cache.set(&key, raw, ttl).await;
                            }
                            m
                        }
                        Err(e) => {
                            if e != MetaError::NotFound || last_err == MetaError::NotFound {
                                last_err = e;
                            }
                            continue;
                        }
                    }
                }
            };
            let mut meta = meta;
            self.apply_art(&mut meta, effective, &ids).await;
            if self.config.anime_renumber && !matches!(provider, Provider::Tmdb | Provider::Tvdb) {
                if let (Some(lists), Some(anidb)) = (&self.resolver.anime_lists, ids.anidb) {
                    lists.renumber(anidb, &mut meta);
                }
            }
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

fn lacks_basics(meta: &Meta) -> bool {
    meta.description.trim().is_empty() || meta.poster.is_none() || meta.genres.is_empty()
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
