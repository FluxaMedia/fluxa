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

fn list_page(page: u32, total: u32, ids: std::ops::Range<u32>) -> String {
    let results: Vec<_> = ids
        .map(|id| format!(r#"{{"id":{id},"title":"Title {id}","poster_path":"/p{id}.jpg"}}"#))
        .collect();
    format!(
        r#"{{"page":{page},"total_pages":{total},"results":[{}]}}"#,
        results.join(",")
    )
}

#[test]
fn catalog_pages_follow_skip_across_provider_pages() {
    let svc = service(
        vec![
            (
                "/movie/popular?language=en-US&page=1",
                200,
                list_page(1, 2, 1..21),
            ),
            (
                "/movie/popular?language=en-US&page=2",
                200,
                list_page(2, 2, 21..31),
            ),
        ],
        AgeGate::default(),
    );
    let first = block_on(svc.catalog(ContentType::Movie, "tmdb.popular", 0, 20)).unwrap();
    assert_eq!(first.metas.len(), 20);
    assert_eq!(first.metas[0].id, "tmdb:1");
    assert!(first.has_more);

    let second = block_on(svc.catalog(ContentType::Movie, "tmdb.popular", 20, 20)).unwrap();
    assert_eq!(second.metas.len(), 10);
    assert_eq!(second.metas[0].id, "tmdb:21");
    assert!(!second.has_more);
}

#[test]
fn catalog_with_an_age_cap_uses_certified_discover() {
    let gate = AgeGate {
        cap: Some("PG".into()),
        allow_unrated: true,
    };
    let svc = service(vec![("/discover/movie", 200, list_page(1, 1, 1..4))], gate);
    let page = block_on(svc.catalog(ContentType::Movie, "tmdb.top", 0, 20)).unwrap();
    assert_eq!(page.metas.len(), 3);
    let hits = svc.fetcher.hits.borrow();
    assert!(hits[0].contains("certification.lte=PG"), "{}", hits[0]);
}

#[test]
fn unknown_catalog_is_not_found() {
    let svc = service(Vec::new(), AgeGate::default());
    assert_eq!(
        block_on(svc.catalog(ContentType::Movie, "mal.airing", 0, 20)).err(),
        Some(MetaError::NotFound)
    );
}

fn configured(
    routes: Vec<(&'static str, u16, String)>,
    config: MetaConfig,
) -> MetaService<FakeFetcher, MemCache> {
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

const KITSU_SHOW: &str = r#"{"data":{"id":"7","attributes":{"canonicalTitle":"Example Show","titles":{"en":"Example Show"},"synopsis":"","startDate":"2024-04-01","status":"finished","subtype":"TV","averageRating":"83.00"}},"included":[]}"#;
const KITSU_EPISODES: &str = r#"{"data":[{"attributes":{"number":1,"canonicalTitle":"One","airdate":"2024-04-01"}}],"links":{}}"#;

#[test]
fn anime_ids_are_served_by_the_anime_provider_with_blanks_filled_from_tmdb() {
    let mut config = MetaConfig::default();
    config
        .providers
        .insert(ContentType::Anime, fluxa_aggmeta::types::Provider::Kitsu);
    let svc = configured(
        vec![
            ("/anime/7/episodes", 200, KITSU_EPISODES.into()),
            ("/anime/7", 200, KITSU_SHOW.into()),
        ],
        config,
    );
    let out = block_on(svc.meta(ContentType::Series, "kitsu:7", now())).unwrap();
    assert_eq!(out.provider, fluxa_aggmeta::types::Provider::Kitsu);
    assert_eq!(out.meta.name, "Example Show");
    assert_eq!(out.meta.videos.as_ref().map(Vec::len), Some(1));
    assert!(!out.degraded);
}

#[test]
fn tvdb_is_skipped_without_a_key_and_tmdb_answers_instead() {
    let mut config = MetaConfig::default();
    config
        .providers
        .insert(ContentType::Series, fluxa_aggmeta::types::Provider::Tvdb);
    let tv = r#"{"id":1396,"name":"Example Show","original_name":"Example Show","original_language":"en","overview":"x","first_air_date":"2008-01-20","external_ids":{"imdb_id":"tt0903747"},"seasons":[]}"#;
    let svc = configured(vec![("/tv/1396", 200, tv.into())], config);
    let out = block_on(svc.meta(ContentType::Series, "tmdb:1396", now())).unwrap();
    assert_eq!(out.provider, fluxa_aggmeta::types::Provider::Tmdb);
    assert!(out.degraded);
}

#[test]
fn tvdb_logs_in_once_and_reuses_the_token() {
    let mut config = MetaConfig::default();
    config.tvdb_api_key = "key".into();
    config
        .providers
        .insert(ContentType::Series, fluxa_aggmeta::types::Provider::Tvdb);
    let svc = configured(
        vec![
            ("/login", 200, r#"{"data":{"token":"tok"}}"#.into()),
            (
                "/episodes/default",
                200,
                r#"{"data":{"episodes":[]},"links":{"next":null}}"#.into(),
            ),
            (
                "/series/81189/extended",
                200,
                r#"{"data":{"id":81189,"name":"Example Show","overview":"Sand.","firstAired":"2008-01-20","genres":[{"name":"Drama"}],"artworks":[{"type":2,"image":"https://img.example/p.jpg","score":1}],"status":{"name":"Ended"}}}"#.into(),
            ),
        ],
        config,
    );
    block_on(svc.meta(ContentType::Series, "tvdb:81189", now())).unwrap();
    block_on(svc.meta(ContentType::Series, "tvdb:81190", now())).ok();
    let logins = svc
        .fetcher
        .hits
        .borrow()
        .iter()
        .filter(|url| url.ends_with("/login"))
        .count();
    assert_eq!(logins, 1);
}

const FANART_MOVIE: &str = r#"{"movieposter":[{"url":"https://img.example/p.jpg","lang":"en","likes":"5"}],"moviebackground":[{"url":"https://img.example/b.jpg","lang":"00","likes":"5"}],"hdmovielogo":[{"url":"https://img.example/l.png","lang":"en","likes":"5"}]}"#;

#[test]
fn fanart_art_replaces_the_provider_art_and_is_cached() {
    let config = MetaConfig {
        fanart_api_key: "fk".into(),
        ..Default::default()
    };
    let svc = configured(
        vec![
            ("/movie/603", 200, MOVIE.into()),
            (
                "webservice.fanart.tv/v3/movies/603",
                200,
                FANART_MOVIE.into(),
            ),
        ],
        config,
    );
    let first = block_on(svc.meta(ContentType::Movie, "tmdb:603", now())).unwrap();
    assert_eq!(
        first.meta.poster.as_deref(),
        Some("https://img.example/p.jpg")
    );
    assert_eq!(
        first.meta.logo.as_deref(),
        Some("https://img.example/l.png")
    );
    block_on(svc.meta(ContentType::Movie, "tmdb:603", now())).unwrap();
    let fanart_calls = svc
        .fetcher
        .hits
        .borrow()
        .iter()
        .filter(|url| url.contains("fanart.tv"))
        .count();
    assert_eq!(fanart_calls, 1);
}

#[test]
fn provider_art_wins_when_fanart_is_second() {
    let config = MetaConfig {
        fanart_api_key: "fk".into(),
        fanart_first: false,
        ..Default::default()
    };
    let svc = configured(
        vec![
            (
                "/movie/603",
                200,
                MOVIE.replace("\"overview\"", "\"poster_path\":\"/own.jpg\",\"overview\""),
            ),
            (
                "webservice.fanart.tv/v3/movies/603",
                200,
                FANART_MOVIE.into(),
            ),
        ],
        config,
    );
    let out = block_on(svc.meta(ContentType::Movie, "tmdb:603", now())).unwrap();
    assert!(out.meta.poster.as_deref().unwrap().ends_with("/own.jpg"));
    assert_eq!(out.meta.logo.as_deref(), Some("https://img.example/l.png"));
}

#[test]
fn catalog_cards_get_fanart_posters() {
    let config = MetaConfig {
        fanart_api_key: "fk".into(),
        ..Default::default()
    };
    let svc = configured(
        vec![
            (
                "/movie/popular?language=en-US&page=1",
                200,
                list_page(1, 1, 603..604),
            ),
            (
                "webservice.fanart.tv/v3/movies/603",
                200,
                FANART_MOVIE.into(),
            ),
        ],
        config,
    );
    let page = block_on(svc.catalog(ContentType::Movie, "tmdb.popular", 0, 20)).unwrap();
    assert_eq!(
        page.metas[0].poster.as_deref(),
        Some("https://img.example/p.jpg")
    );
}

#[test]
fn rpdb_poster_is_used_when_a_key_is_set() {
    let config = MetaConfig {
        rpdb_api_key: "rk".into(),
        ..Default::default()
    };
    let svc = configured(vec![("/movie/603", 200, MOVIE.into())], config);
    let out = block_on(svc.meta(ContentType::Movie, "tmdb:603", now())).unwrap();
    assert_eq!(
        out.meta.poster.as_deref(),
        Some("https://api.ratingposterdb.com/rk/imdb/poster-default/tt0133093.jpg?fallback=true")
    );
}

#[test]
fn enabled_service_catalogs_are_served_and_disabled_ones_are_not_found() {
    let defaults = MetaConfig::default();
    let config = MetaConfig {
        catalogs: fluxa_aggmeta::tmdb_catalog::enabled(
            |key, on| key == "catalogNetflix" || on,
            "GB",
        ),
        ..Default::default()
    };
    let body = r#"{"page":1,"total_pages":1,"results":[{"id":9,"title":"Streamed","release_date":"2024-01-01"}]}"#;
    let svc = configured(vec![("with_watch_providers=8", 200, body.into())], config);
    let page = block_on(svc.catalog(ContentType::Movie, "svc.netflix", 0, 20)).unwrap();
    assert_eq!(page.metas[0].id, "tmdb:9");
    assert!(svc.fetcher.hits.borrow()[0].contains("watch_region=GB"));

    let plain = configured(vec![], defaults);
    assert_eq!(
        block_on(plain.catalog(ContentType::Movie, "svc.netflix", 0, 20)).err(),
        Some(MetaError::NotFound)
    );
}

const TMDB_SEARCH: &str = r#"{"page":1,"total_pages":1,"results":[{"id":603,"title":"The Matrix","release_date":"1999-03-30"},{"id":604,"title":"Matrix Two","release_date":"2003-05-15"}]}"#;

#[test]
fn search_returns_tmdb_results_for_the_requested_type() {
    let svc = configured(
        vec![("/search/movie", 200, TMDB_SEARCH.into())],
        MetaConfig::default(),
    );
    let page = block_on(svc.search(false, ContentType::Movie, "matrix", 0, 20)).unwrap();
    assert_eq!(page.metas.len(), 2);
    assert_eq!(page.metas[0].id, "tmdb:603");
    assert!(svc.fetcher.hits.borrow()[0].contains("query=matrix"));
}

#[test]
fn ai_titles_are_searched_first_and_duplicates_dropped() {
    let config = MetaConfig {
        gemini_api_key: "gk".into(),
        ai_search: true,
        ..Default::default()
    };
    let gemini = r#"{"candidates":[{"content":{"parts":[{"text":"[\"Matrix Two\"]"}]}}]}"#;
    let svc = configured(
        vec![
            ("generativelanguage.googleapis.com", 200, gemini.into()),
            ("/search/movie", 200, TMDB_SEARCH.into()),
        ],
        config,
    );
    let page = block_on(svc.search(
        false,
        ContentType::Movie,
        "sequel about a simulated world",
        0,
        20,
    ))
    .unwrap();
    let ids: Vec<_> = page.metas.iter().map(|m| m.id.as_str()).collect();
    assert_eq!(ids, vec!["tmdb:603", "tmdb:604"]);
}

#[test]
fn ai_search_stays_off_without_a_key() {
    let config = MetaConfig {
        ai_search: true,
        ..Default::default()
    };
    let svc = configured(vec![("/search/movie", 200, TMDB_SEARCH.into())], config);
    block_on(svc.search(false, ContentType::Movie, "matrix", 0, 20)).unwrap();
    assert!(
        svc.fetcher
            .hits
            .borrow()
            .iter()
            .all(|url| !url.contains("googleapis"))
    );
}

#[test]
fn mdblist_catalogs_need_the_key_and_read_the_kind_array() {
    let defs = fluxa_aggmeta::tmdb_catalog::from_prefs(
        &serde_json::json!({"metadatakitMdblistLists": "77"}),
    );
    let body = r#"{"movies":[{"title":"Listed","ids":{"imdb":"tt9"}}],"shows":[]}"#;
    let with_key = MetaConfig {
        catalogs: defs.clone(),
        mdblist_api_key: "mk".into(),
        ..Default::default()
    };
    let svc = configured(vec![("/lists/77/items", 200, body.into())], with_key);
    let page = block_on(svc.catalog(ContentType::Movie, "mdblist.77", 0, 20)).unwrap();
    assert_eq!(page.metas[0].id, "tt9");

    let without_key = MetaConfig {
        catalogs: defs,
        ..Default::default()
    };
    let svc = configured(vec![], without_key);
    assert_eq!(
        block_on(svc.catalog(ContentType::Movie, "mdblist.77", 0, 20)).err(),
        Some(MetaError::NotFound)
    );
}

#[test]
fn anilist_user_lists_are_paged_locally() {
    let defs = fluxa_aggmeta::tmdb_catalog::from_prefs(&serde_json::json!({
        "metadatakitAnilistUser": "example", "catalogAnilistWatching": true
    }));
    let body = r#"{"data":{"MediaListCollection":{"lists":[{"entries":[{"media":{"id":5,"title":{"romaji":"Listed Anime","english":null},"startDate":{"year":2024}}}]}]}}}"#;
    let config = MetaConfig {
        catalogs: defs,
        ..Default::default()
    };
    let svc = configured(vec![("graphql.anilist.co", 200, body.into())], config);
    let page = block_on(svc.catalog(ContentType::Series, "anilist.list.current", 0, 20)).unwrap();
    assert_eq!(page.metas[0].id, "anilist:5");
    assert!(!page.has_more);
}

#[test]
fn anilist_token_serves_the_private_lists_of_the_signed_in_user() {
    let prefs = serde_json::json!({
        "anilistToken": "tok", "catalogAnilistWatching": true
    });
    let body = r#"{"data":{"Viewer":{"name":"signed_in"},"MediaListCollection":{"lists":[{"entries":[{"media":{"id":9,"title":{"romaji":"Private Anime","english":null},"startDate":{"year":2024}}}]}]}}}"#;
    let config = MetaConfig {
        catalogs: fluxa_aggmeta::tmdb_catalog::from_prefs(&prefs),
        anilist_token: "tok".into(),
        ..Default::default()
    };
    let svc = configured(vec![("graphql.anilist.co", 200, body.into())], config);
    let page = block_on(svc.catalog(ContentType::Series, "anilist.list.current", 0, 20)).unwrap();
    assert_eq!(page.metas[0].id, "anilist:9");
}

#[test]
fn mal_list_catalog_reads_the_public_list() {
    let prefs = serde_json::json!({
        "metadatakitMalUser": "example", "catalogMalWatching": true
    });
    let body = r#"[{"anime_id":7,"anime_title":"Example Anime","anime_image_path":"https://cdn.example/a.jpg"}]"#;
    let config = MetaConfig {
        catalogs: fluxa_aggmeta::tmdb_catalog::from_prefs(&prefs),
        ..Default::default()
    };
    let svc = configured(
        vec![("myanimelist.net/animelist/example", 200, body.into())],
        config,
    );
    let page = block_on(svc.catalog(ContentType::Series, "mal.list.watching", 0, 20)).unwrap();
    assert_eq!(page.metas[0].id, "mal:7");
}

#[test]
fn tvdb_search_is_used_when_chosen_and_keyed() {
    let config = MetaConfig {
        tvdb_api_key: "key".into(),
        tvdb_search: vec![ContentType::Series],
        ..Default::default()
    };
    let svc = configured(
        vec![
            ("/login", 200, r#"{"data":{"token":"tok"}}"#.into()),
            (
                "/search?query=show",
                200,
                r#"{"data":[{"tvdb_id":"5","name":"Example Show","year":"2008"}]}"#.into(),
            ),
        ],
        config,
    );
    let page = block_on(svc.search(false, ContentType::Series, "show", 0, 20)).unwrap();
    assert_eq!(page.metas[0].id, "tvdb:5");
}

#[test]
fn search_drops_titles_above_the_age_cap() {
    let gate = AgeGate {
        cap: Some("PG".into()),
        allow_unrated: true,
    };
    let results = r#"{"page":1,"total_pages":1,"results":[{"id":603,"title":"The Matrix"}]}"#;
    let svc = service(
        vec![
            ("/search/movie", 200, results.into()),
            ("/movie/603", 200, MOVIE.into()),
        ],
        gate,
    );
    let page = block_on(svc.search(false, ContentType::Movie, "matrix", 0, 20)).unwrap();
    assert!(page.metas.is_empty());
}

#[test]
fn people_search_lists_the_credits_of_the_first_match() {
    let config = MetaConfig {
        people_search: true,
        ..Default::default()
    };
    let credits = r#"{"cast":[{"id":7,"media_type":"movie","title":"Credited Film","popularity":9.0},{"id":8,"media_type":"tv","name":"Credited Show","popularity":5.0}],"crew":[]}"#;
    let svc = configured(
        vec![
            ("/search/person", 200, r#"{"results":[{"id":42}]}"#.into()),
            ("/person/42/combined_credits", 200, credits.into()),
            ("/search/movie", 200, r#"{"results":[]}"#.into()),
        ],
        config,
    );
    let page = block_on(svc.search(false, ContentType::Movie, "some actor", 0, 20)).unwrap();
    assert_eq!(page.metas.len(), 1);
    assert_eq!(page.metas[0].name, "Credited Film");
}
