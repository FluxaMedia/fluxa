use serde_json::Value;

use crate::age::AgeGate;
use crate::catalog::ProviderPage;
use crate::tmdb::API_BASE;
use crate::types::{ContentType, Meta};

pub const PAGE_SIZE: usize = 20;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnimeList {
    Trending,
    Popular,
    Airing,
    TopRated,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Source {
    Trending,
    Popular,
    TopRated,
    NowPlaying,
    Service {
        provider: u32,
        region: String,
    },
    Anime(AnimeList),
    MdblistList(String),
    AnilistList {
        user: String,
        status: &'static str,
    },
    Discover {
        genres: String,
        from: Option<u32>,
        to: Option<u32>,
    },
}

#[derive(Clone, Debug)]
pub struct CatalogDef {
    pub id: String,
    pub kind: ContentType,
    pub name: String,
    pub source: Source,
}

fn def(id: &str, kind: ContentType, name: &str, source: Source) -> CatalogDef {
    CatalogDef {
        id: id.to_owned(),
        kind,
        name: name.to_owned(),
        source,
    }
}

const SERVICES: [(&str, &str, &str, u32); 7] = [
    ("catalogNetflix", "svc.netflix", "Netflix", 8),
    ("catalogPrime", "svc.prime", "Prime Video", 9),
    ("catalogDisney", "svc.disney", "Disney+", 337),
    ("catalogMax", "svc.max", "Max", 1899),
    ("catalogAppleTv", "svc.appletv", "Apple TV+", 350),
    ("catalogHulu", "svc.hulu", "Hulu", 15),
    ("catalogParamount", "svc.paramount", "Paramount+", 531),
];

const ANIME: [(&str, &str, &str, AnimeList); 4] = [
    (
        "catalogAnimeTrending",
        "anilist.trending",
        "Anime trending",
        AnimeList::Trending,
    ),
    (
        "catalogAnimePopular",
        "anilist.popular",
        "Anime popular",
        AnimeList::Popular,
    ),
    (
        "catalogAnimeAiring",
        "anilist.airing",
        "Anime airing",
        AnimeList::Airing,
    ),
    (
        "catalogAnimeTop",
        "anilist.top",
        "Anime top rated",
        AnimeList::TopRated,
    ),
];

pub const DEFAULT_REGION: &str = "US";

pub fn enabled(is_on: impl Fn(&str, bool) -> bool, region: &str) -> Vec<CatalogDef> {
    let region = if region.trim().is_empty() {
        DEFAULT_REGION
    } else {
        region.trim()
    };
    let mut out = Vec::new();
    let both = [ContentType::Movie, ContentType::Series];
    for (key, id, name, source, on) in [
        (
            "catalogTrending",
            "tmdb.trending",
            "Trending",
            Source::Trending,
            true,
        ),
        (
            "catalogPopular",
            "tmdb.popular",
            "Popular",
            Source::Popular,
            true,
        ),
        (
            "catalogTopRated",
            "tmdb.top",
            "Top rated",
            Source::TopRated,
            true,
        ),
        (
            "catalogNowPlaying",
            "tmdb.now",
            "Now playing",
            Source::NowPlaying,
            false,
        ),
    ] {
        if is_on(key, on) {
            out.extend(both.map(|kind| def(id, kind, name, source.clone())));
        }
    }
    for (key, id, name, provider) in SERVICES {
        if is_on(key, false) {
            let source = Source::Service {
                provider,
                region: region.to_owned(),
            };
            out.extend(both.map(|kind| def(id, kind, name, source.clone())));
        }
    }
    for (key, id, name, list) in ANIME {
        if is_on(key, false) {
            out.push(def(id, ContentType::Series, name, Source::Anime(list)));
        }
    }
    out
}

pub fn defaults() -> Vec<CatalogDef> {
    enabled(|_, on| on, DEFAULT_REGION)
}

const ANILIST_LISTS: [(&str, &str, &str, &str); 3] = [
    (
        "catalogAnilistWatching",
        "anilist.list.current",
        "My anime watching",
        "CURRENT",
    ),
    (
        "catalogAnilistPlanning",
        "anilist.list.planning",
        "My anime planned",
        "PLANNING",
    ),
    (
        "catalogAnilistCompleted",
        "anilist.list.completed",
        "My anime completed",
        "COMPLETED",
    ),
];

pub fn list_defs(
    is_on: impl Fn(&str, bool) -> bool,
    mdblist_ids: &str,
    anilist_user: &str,
    has_anilist_token: bool,
) -> Vec<CatalogDef> {
    let mut out = Vec::new();
    for list in mdblist_ids
        .split(',')
        .map(str::trim)
        .filter(|l| !l.is_empty())
    {
        let id = format!("mdblist.{list}");
        let name = format!("MDBList {list}");
        for kind in [ContentType::Movie, ContentType::Series] {
            out.push(def(&id, kind, &name, Source::MdblistList(list.to_owned())));
        }
    }
    let user = match anilist_user.trim() {
        "" if has_anilist_token => crate::providers::anilist::VIEWER_USER,
        user => user,
    };
    if !user.is_empty() {
        for (key, id, name, status) in ANILIST_LISTS {
            if is_on(key, false) {
                let source = Source::AnilistList {
                    user: user.to_owned(),
                    status,
                };
                out.push(def(id, ContentType::Series, name, source));
            }
        }
    }
    out
}

const GENRES: [(&str, u32, u32); 17] = [
    ("action", 28, 10759),
    ("adventure", 12, 10759),
    ("animation", 16, 16),
    ("comedy", 35, 35),
    ("crime", 80, 80),
    ("documentary", 99, 99),
    ("drama", 18, 18),
    ("family", 10751, 10751),
    ("fantasy", 14, 10765),
    ("history", 36, 36),
    ("horror", 27, 27),
    ("mystery", 9648, 9648),
    ("romance", 10749, 10749),
    ("science fiction", 878, 10765),
    ("sci-fi", 878, 10765),
    ("thriller", 53, 53),
    ("war", 10752, 10768),
];

fn genre_ids(genres: &str, kind: ContentType) -> String {
    let mut out: Vec<String> = Vec::new();
    for part in genres.split(',').map(|g| g.trim().to_lowercase()) {
        let id = part.parse::<u32>().ok().or_else(|| {
            GENRES
                .iter()
                .find(|(name, _, _)| *name == part)
                .map(|(_, movie, tv)| {
                    if kind == ContentType::Movie {
                        *movie
                    } else {
                        *tv
                    }
                })
        });
        if let Some(id) = id.map(|id| id.to_string()) {
            if !out.contains(&id) {
                out.push(id);
            }
        }
    }
    out.join(",")
}

pub fn custom_defs(text: &str) -> Vec<CatalogDef> {
    let mut out = Vec::new();
    for (index, line) in text.split(';').map(str::trim).enumerate() {
        let mut fields = line.split('|').map(str::trim);
        let name = fields.next().unwrap_or_default();
        let genres = fields.next().unwrap_or_default();
        let year = |field: Option<&str>| field.and_then(|f| f.parse::<u32>().ok());
        let from = year(fields.next());
        let to = year(fields.next());
        if name.is_empty() || (genres.is_empty() && from.is_none() && to.is_none()) {
            continue;
        }
        let source = Source::Discover {
            genres: genres.to_owned(),
            from,
            to,
        };
        let id = format!("custom.{index}");
        for kind in [ContentType::Movie, ContentType::Series] {
            out.push(def(&id, kind, name, source.clone()));
        }
    }
    out
}

pub fn from_prefs(prefs: &Value) -> Vec<CatalogDef> {
    let text = |key: &str| prefs.get(key).and_then(Value::as_str).unwrap_or("");
    let is_on =
        |key: &str, default: bool| prefs.get(key).and_then(Value::as_bool).unwrap_or(default);
    let mut defs = enabled(
        is_on,
        prefs
            .get("metadatakitRegion")
            .and_then(Value::as_str)
            .unwrap_or(DEFAULT_REGION),
    );
    defs.extend(list_defs(
        is_on,
        text("metadatakitMdblistLists"),
        text("metadatakitAnilistUser"),
        !text("anilistToken").is_empty(),
    ));
    defs.extend(custom_defs(text("metadatakitCustomCatalogs")));
    defs
}

pub const SEARCH_ID: &str = "tmdb.search";
pub const ANIME_SEARCH_ID: &str = "anilist.search";

pub fn search_url(kind: ContentType, query: &str, page: u32, language: &str) -> String {
    format!(
        "{API_BASE}/search/{}?language={language}&query={}&page={page}&include_adult=false",
        media(kind),
        crate::search::encode_component(query)
    )
}

pub fn search_defs(anime: bool) -> Vec<CatalogDef> {
    let mut out = vec![ContentType::Movie, ContentType::Series]
        .into_iter()
        .map(|kind| def(SEARCH_ID, kind, "Search", Source::Trending))
        .collect::<Vec<_>>();
    if anime {
        out.push(def(
            ANIME_SEARCH_ID,
            ContentType::Series,
            "Anime search",
            Source::Anime(AnimeList::Trending),
        ));
    }
    out
}

pub fn anime_search_enabled(prefs: &Value) -> bool {
    prefs
        .get("metadatakitSearchAnime")
        .and_then(Value::as_bool)
        .unwrap_or(false)
}

pub fn find<'a>(defs: &'a [CatalogDef], kind: ContentType, id: &str) -> Option<&'a CatalogDef> {
    defs.iter().find(|def| def.kind == kind && def.id == id)
}

fn media(kind: ContentType) -> &'static str {
    if kind == ContentType::Movie {
        "movie"
    } else {
        "tv"
    }
}

pub fn page_url(def: &CatalogDef, page: u32, language: &str, gate: &AgeGate) -> Option<String> {
    let media = media(def.kind);
    let cap = (def.kind == ContentType::Movie)
        .then(|| gate.movie_cap())
        .flatten();
    let certified = |sort: &str, extra: &str| {
        let cap = cap
            .map(|cap| format!("&certification_country=US&certification.lte={cap}"))
            .unwrap_or_default();
        format!(
            "{API_BASE}/discover/{media}?language={language}&page={page}&sort_by={sort}&include_adult=false{extra}{cap}"
        )
    };
    Some(match (&def.source, cap) {
        (Source::Trending, _) => {
            format!("{API_BASE}/trending/{media}/week?language={language}&page={page}")
        }
        (Source::Popular, None) => {
            format!("{API_BASE}/{media}/popular?language={language}&page={page}")
        }
        (Source::TopRated, None) => {
            format!("{API_BASE}/{media}/top_rated?language={language}&page={page}")
        }
        (Source::Popular, Some(_)) => certified("popularity.desc", ""),
        (Source::TopRated, Some(_)) => certified("vote_average.desc", "&vote_count.gte=300"),
        (Source::NowPlaying, _) => {
            let path = if def.kind == ContentType::Movie {
                "movie/now_playing"
            } else {
                "tv/on_the_air"
            };
            format!("{API_BASE}/{path}?language={language}&page={page}")
        }
        (Source::Service { provider, region }, _) => certified(
            "popularity.desc",
            &format!(
                "&with_watch_providers={provider}&watch_region={region}&with_watch_monetization_types=flatrate"
            ),
        ),
        (Source::Discover { genres, from, to }, _) => {
            let (gte, lte) = if def.kind == ContentType::Movie {
                ("primary_release_date.gte", "primary_release_date.lte")
            } else {
                ("first_air_date.gte", "first_air_date.lte")
            };
            let mut extra = String::new();
            let ids = genre_ids(genres, def.kind);
            if !ids.is_empty() {
                extra.push_str(&format!("&with_genres={ids}"));
            }
            if let Some(year) = from {
                extra.push_str(&format!("&{gte}={year}-01-01"));
            }
            if let Some(year) = to {
                extra.push_str(&format!("&{lte}={year}-12-31"));
            }
            certified("popularity.desc", &format!("{extra}&vote_count.gte=20"))
        }
        (Source::Anime(_) | Source::MdblistList(_) | Source::AnilistList { .. }, _) => return None,
    })
}

fn image(path: Option<&str>, size: &str) -> Option<String> {
    path.filter(|path| !path.is_empty())
        .map(|path| format!("https://image.tmdb.org/t/p/{size}{path}"))
}

fn meta(kind: ContentType, item: &Value) -> Option<Meta> {
    let id = item.get("id").and_then(Value::as_u64)?;
    let text = |key: &str| item.get(key).and_then(Value::as_str).unwrap_or_default();
    let (name, date) = if kind == ContentType::Movie {
        (text("title"), text("release_date"))
    } else {
        (text("name"), text("first_air_date"))
    };
    if name.is_empty() {
        return None;
    }
    let rating = item
        .get("vote_average")
        .and_then(Value::as_f64)
        .filter(|rating| *rating > 0.0)
        .map(|rating| format!("{rating:.1}"))
        .unwrap_or_default();
    let year = date.get(..4).unwrap_or_default().to_owned();
    Some(Meta {
        id: format!("tmdb:{id}"),
        kind: kind.as_str().to_owned(),
        name: name.to_owned(),
        description: text("overview").to_owned(),
        release_info: year.clone(),
        year,
        imdb_rating: rating,
        poster: image(item.get("poster_path").and_then(Value::as_str), "w500"),
        background: image(item.get("backdrop_path").and_then(Value::as_str), "w1280"),
        p_tmdb: Some(id),
        ..Meta::default()
    })
}

pub fn parse_page(kind: ContentType, body: &Value) -> ProviderPage<Meta> {
    let items = body
        .get("results")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|item| meta(kind, item))
        .collect();
    let page = body.get("page").and_then(Value::as_u64).unwrap_or(1);
    let total = body.get("total_pages").and_then(Value::as_u64).unwrap_or(1);
    ProviderPage {
        items,
        has_next_page: page < total,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn gate(cap: Option<&str>) -> AgeGate {
        AgeGate {
            cap: cap.map(str::to_owned),
            allow_unrated: true,
        }
    }

    #[test]
    fn movie_cap_switches_to_certified_discover() {
        let defs = defaults();
        let def = find(&defs, ContentType::Movie, "tmdb.popular").unwrap();
        let url = page_url(def, 2, "en-US", &gate(Some("PG-13"))).unwrap();
        assert!(url.contains("/discover/movie"), "{url}");
        assert!(url.contains("certification.lte=PG-13"), "{url}");
        assert!(url.contains("page=2"), "{url}");
        let open = page_url(def, 1, "en-US", &gate(None)).unwrap();
        assert!(open.contains("/movie/popular"), "{open}");
    }

    #[test]
    fn custom_catalogs_build_discover_urls_per_media() {
        let defs = custom_defs("Eighties Action|Action,Adventure|1980|1989; Broken");
        assert_eq!(defs.len(), 2);
        let movie = page_url(&defs[0], 1, "en-US", &gate(None)).unwrap();
        assert!(movie.contains("with_genres=28,12"), "{movie}");
        assert!(
            movie.contains("primary_release_date.gte=1980-01-01"),
            "{movie}"
        );
        let tv = page_url(&defs[1], 1, "en-US", &gate(None)).unwrap();
        assert!(tv.contains("with_genres=10759"), "{tv}");
        assert!(tv.contains("first_air_date.lte=1989-12-31"), "{tv}");
    }

    #[test]
    fn defaults_keep_the_original_six_catalogs() {
        let ids: Vec<_> = defaults().iter().map(|d| (d.kind, d.id.clone())).collect();
        assert_eq!(ids.len(), 6);
        assert!(ids.contains(&(ContentType::Series, "tmdb.top".to_owned())));
    }

    #[test]
    fn enabled_services_get_movie_and_series_catalogs_in_the_region() {
        let defs = enabled(|key, on| key == "catalogNetflix" || on, "GB");
        let netflix: Vec<_> = defs.iter().filter(|d| d.id == "svc.netflix").collect();
        assert_eq!(netflix.len(), 2);
        let url = page_url(netflix[0], 1, "en-US", &gate(None)).unwrap();
        assert!(url.contains("with_watch_providers=8"), "{url}");
        assert!(url.contains("watch_region=GB"), "{url}");
    }

    #[test]
    fn personal_lists_come_from_ids_and_a_user_name() {
        let prefs = json!({
            "metadatakitMdblistLists": "12, 34",
            "metadatakitAnilistUser": "example",
            "catalogAnilistWatching": true
        });
        let defs = from_prefs(&prefs);
        assert_eq!(defs.iter().filter(|d| d.id == "mdblist.34").count(), 2);
        assert!(defs.iter().any(|d| d.id == "anilist.list.current"));
        assert!(defs.iter().all(|d| d.id != "anilist.list.planning"));
        let no_user = from_prefs(&json!({"catalogAnilistWatching": true}));
        assert!(no_user.iter().all(|d| !d.id.starts_with("anilist.list")));
    }

    #[test]
    fn prefs_toggle_catalogs() {
        let prefs = json!({"catalogTrending": false, "catalogAnimeAiring": true});
        let defs = from_prefs(&prefs);
        assert!(defs.iter().all(|d| d.id != "tmdb.trending"));
        assert!(defs.iter().any(|d| d.id == "anilist.airing"));
    }

    #[test]
    fn rows_without_a_title_are_dropped() {
        let body = json!({"page": 1, "total_pages": 3, "results": [
            {"id": 1, "title": "One", "poster_path": "/a.jpg", "release_date": "1999-01-02", "vote_average": 7.84},
            {"id": 2, "title": ""}
        ]});
        let page = parse_page(ContentType::Movie, &body);
        assert_eq!(page.items.len(), 1);
        assert_eq!(page.items[0].id, "tmdb:1");
        assert_eq!(page.items[0].year, "1999");
        assert_eq!(page.items[0].imdb_rating, "7.8");
        assert!(page.has_next_page);
    }
}
