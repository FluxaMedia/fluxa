use serde_json::Value;

use crate::age::AgeGate;
use crate::catalog::ProviderPage;
use crate::tmdb::API_BASE;
use crate::types::{ContentType, Meta};

pub const PAGE_SIZE: usize = 20;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Source {
    Trending,
    Popular,
    TopRated,
}

#[derive(Clone, Copy, Debug)]
pub struct CatalogDef {
    pub id: &'static str,
    pub kind: ContentType,
    pub name: &'static str,
    pub source: Source,
}

pub const CATALOGS: [CatalogDef; 6] = [
    CatalogDef {
        id: "tmdb.trending",
        kind: ContentType::Movie,
        name: "Trending",
        source: Source::Trending,
    },
    CatalogDef {
        id: "tmdb.popular",
        kind: ContentType::Movie,
        name: "Popular",
        source: Source::Popular,
    },
    CatalogDef {
        id: "tmdb.top",
        kind: ContentType::Movie,
        name: "Top rated",
        source: Source::TopRated,
    },
    CatalogDef {
        id: "tmdb.trending",
        kind: ContentType::Series,
        name: "Trending",
        source: Source::Trending,
    },
    CatalogDef {
        id: "tmdb.popular",
        kind: ContentType::Series,
        name: "Popular",
        source: Source::Popular,
    },
    CatalogDef {
        id: "tmdb.top",
        kind: ContentType::Series,
        name: "Top rated",
        source: Source::TopRated,
    },
];

pub fn find(kind: ContentType, id: &str) -> Option<&'static CatalogDef> {
    CATALOGS.iter().find(|def| def.kind == kind && def.id == id)
}

fn media(kind: ContentType) -> &'static str {
    if kind == ContentType::Movie {
        "movie"
    } else {
        "tv"
    }
}

pub fn page_url(def: &CatalogDef, page: u32, language: &str, gate: &AgeGate) -> String {
    let media = media(def.kind);
    let cap = (def.kind == ContentType::Movie)
        .then(|| gate.movie_cap())
        .flatten();
    match (def.source, cap) {
        (Source::Trending, _) => {
            format!("{API_BASE}/trending/{media}/week?language={language}&page={page}")
        }
        (Source::Popular, None) => {
            format!("{API_BASE}/{media}/popular?language={language}&page={page}")
        }
        (Source::TopRated, None) => {
            format!("{API_BASE}/{media}/top_rated?language={language}&page={page}")
        }
        (source, Some(cap)) => {
            let sort = if source == Source::Popular {
                "popularity.desc"
            } else {
                "vote_average.desc&vote_count.gte=300"
            };
            format!(
                "{API_BASE}/discover/movie?language={language}&page={page}&sort_by={sort}&include_adult=false&certification_country=US&certification.lte={cap}"
            )
        }
    }
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
        let def = find(ContentType::Movie, "tmdb.popular").unwrap();
        let url = page_url(def, 2, "en-US", &gate(Some("PG-13")));
        assert!(url.contains("/discover/movie"), "{url}");
        assert!(url.contains("certification.lte=PG-13"), "{url}");
        assert!(url.contains("page=2"), "{url}");
        let open = page_url(def, 1, "en-US", &gate(None));
        assert!(open.contains("/movie/popular"), "{open}");
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
