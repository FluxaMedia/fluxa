use std::cell::RefCell;
use std::collections::HashMap;
use std::future::Future;
use std::task::{Context, Poll, Waker};
use std::time::Duration;

use chrono::{TimeZone, Utc};
use fluxa_aggmeta::age::AgeGate;
use fluxa_aggmeta::idtables::Resolver;
use fluxa_aggmeta::service::{
    FetchError, Fetcher, HttpRequest, HttpResponse, KvCache, MetaConfig, MetaError, MetaService,
};
use fluxa_aggmeta::types::ContentType;

fn block_on<T>(fut: impl Future<Output = T>) -> T {
    let mut fut = Box::pin(fut);
    let mut cx = Context::from_waker(Waker::noop());
    loop {
        if let Poll::Ready(v) = fut.as_mut().poll(&mut cx) {
            return v;
        }
    }
}

struct FakeFetcher {
    routes: Vec<(&'static str, u16, String)>,
    hits: RefCell<Vec<String>>,
}

impl Fetcher for FakeFetcher {
    async fn fetch(&self, req: HttpRequest) -> Result<HttpResponse, FetchError> {
        self.hits.borrow_mut().push(req.url.clone());
        for (needle, status, body) in &self.routes {
            if req.url.contains(needle) {
                return Ok(HttpResponse {
                    status: *status,
                    body: body.clone(),
                });
            }
        }
        Ok(HttpResponse {
            status: 404,
            body: String::new(),
        })
    }
}

#[derive(Default)]
struct MemCache {
    map: RefCell<HashMap<String, String>>,
}

impl KvCache for MemCache {
    async fn get(&self, key: &str) -> Option<String> {
        self.map.borrow().get(key).cloned()
    }
    async fn set(&self, key: &str, value: String, _ttl: Duration) {
        self.map.borrow_mut().insert(key.to_string(), value);
    }
    async fn delete(&self, key: &str) {
        self.map.borrow_mut().remove(key);
    }
}

const MOVIE: &str = r#"{"id":603,"title":"The Matrix","original_title":"The Matrix","original_language":"en","overview":"x","release_date":"1999-03-30","runtime":136,"external_ids":{"imdb_id":"tt0133093"},"release_dates":{"results":[{"iso_3166_1":"US","release_dates":[{"certification":"R"}]}]}}"#;

fn now() -> chrono::DateTime<Utc> {
    Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap()
}

fn service(
    routes: Vec<(&'static str, u16, String)>,
    gate: AgeGate,
) -> MetaService<FakeFetcher, MemCache> {
    let config = MetaConfig {
        age_gate: gate,
        ..Default::default()
    };
    MetaService::new(
        FakeFetcher {
            routes,
            hits: RefCell::new(Vec::new()),
        },
        MemCache::default(),
        config,
        Resolver::default(),
    )
}

#[test]
fn movie_meta_is_fetched_then_served_from_cache() {
    let svc = service(vec![("/movie/603", 200, MOVIE.into())], AgeGate::default());
    let first = block_on(svc.meta(ContentType::Movie, "tmdb:603", now())).unwrap();
    assert_eq!(first.meta.id, "tt0133093");
    assert!(!first.degraded);
    let calls = svc.fetcher.hits.borrow().len();
    block_on(svc.meta(ContentType::Movie, "tmdb:603", now())).unwrap();
    assert_eq!(svc.fetcher.hits.borrow().len(), calls);
}

#[test]
fn age_gate_blocks_restricted_titles() {
    let gate = AgeGate {
        cap: Some("PG-13".into()),
        allow_unrated: true,
    };
    let svc = service(vec![("/movie/603", 200, MOVIE.into())], gate);
    assert_eq!(
        block_on(svc.meta(ContentType::Movie, "tmdb:603", now())).err(),
        Some(MetaError::Blocked)
    );
}

#[test]
fn missing_title_reports_not_found() {
    let svc = service(Vec::new(), AgeGate::default());
    assert_eq!(
        block_on(svc.meta(ContentType::Movie, "tmdb:1", now())).err(),
        Some(MetaError::NotFound)
    );
}

#[test]
fn server_errors_are_retried() {
    let svc = service(vec![("/movie/603", 503, String::new())], AgeGate::default());
    let err = block_on(svc.meta(ContentType::Movie, "tmdb:603", now())).err();
    assert!(matches!(err, Some(MetaError::Unavailable(_))));
    assert!(svc.fetcher.hits.borrow().len() >= 3);
}
