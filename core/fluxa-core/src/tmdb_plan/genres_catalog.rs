use super::helpers::{tmdb_api_url, tmdb_content_type};
use serde_json::{Value, json};

const MOVIE_GENRES: &[(u32, &str)] = &[
    (28, "Action"),
    (12, "Adventure"),
    (16, "Animation"),
    (35, "Comedy"),
    (80, "Crime"),
    (99, "Documentary"),
    (18, "Drama"),
    (10751, "Family"),
    (14, "Fantasy"),
    (36, "History"),
    (27, "Horror"),
    (10402, "Music"),
    (9648, "Mystery"),
    (10749, "Romance"),
    (878, "Science Fiction"),
    (10770, "TV Movie"),
    (53, "Thriller"),
    (10752, "War"),
    (37, "Western"),
];
const TV_GENRES: &[(u32, &str)] = &[
    (10759, "Action & Adventure"),
    (16, "Animation"),
    (35, "Comedy"),
    (80, "Crime"),
    (99, "Documentary"),
    (18, "Drama"),
    (10751, "Family"),
    (10762, "Kids"),
    (9648, "Mystery"),
    (10763, "News"),
    (10764, "Reality"),
    (10765, "Sci-Fi & Fantasy"),
    (10766, "Soap"),
    (10767, "Talk"),
    (10768, "War & Politics"),
    (37, "Western"),
];
fn genre_table(content_type: &str) -> &'static [(u32, &'static str)] {
    if content_type == "series" {
        TV_GENRES
    } else {
        MOVIE_GENRES
    }
}
fn tmdb_genre_id(content_type: &str, name: &str) -> Option<u32> {
    genre_table(content_type)
        .iter()
        .find(|(_, n)| n.eq_ignore_ascii_case(name))
        .map(|(id, _)| *id)
}
fn genre_names(content_type: &str) -> Vec<&'static str> {
    genre_table(content_type).iter().map(|(_, n)| *n).collect()
}
pub(crate) fn tmdb_builtin_manifest_json() -> String {
    let catalog = |content_type: &str, id: &str, name: &str| {
        json!({
            "type": content_type,
            "id": id,
            "name": name,
            "extra": [
                { "name": "search" },
                { "name": "genre", "options": genre_names(content_type) },
                { "name": "skip" },
            ],
        })
    };
    json!({
        "id": "com.fluxa.tmdb-builtin",
        "name": "TMDB",
        "description": "Built-in metadata sourced directly from TMDB",
        "version": "1.0.0",
        "resources": ["catalog", "meta"],
        "types": ["movie", "series"],
        "idPrefixes": ["tt", "tmdb:"],
        "catalogs": [
            catalog("movie", "tmdb.movies", "TMDB Movies"),
            catalog("series", "tmdb.series", "TMDB Series"),
        ],
    })
    .to_string()
}
pub(crate) fn tmdb_builtin_catalog_url(
    content_type: &str,
    extra: &Value,
    api_key: &str,
    language: &str,
) -> String {
    let tmdb_type = tmdb_content_type(content_type);
    let skip = extra
        .get("skip")
        .and_then(|v| {
            v.as_u64()
                .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
        })
        .unwrap_or(0);
    let page = (skip / 20) + 1;
    let page_str = page.to_string();

    if let Some(search) = extra
        .get("search")
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
    {
        return tmdb_api_url(
            &format!("3/search/{tmdb_type}"),
            api_key,
            language,
            &[("query", search), ("page", &page_str)],
        );
    }

    if let Some(genre_name) = extra
        .get("genre")
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
        && let Some(genre_id) = tmdb_genre_id(content_type, genre_name)
    {
        let genre_id_str = genre_id.to_string();
        return tmdb_api_url(
            &format!("3/discover/{tmdb_type}"),
            api_key,
            language,
            &[
                ("with_genres", genre_id_str.as_str()),
                ("sort_by", "popularity.desc"),
                ("page", &page_str),
            ],
        );
    }

    tmdb_api_url(
        &format!("3/{tmdb_type}/popular"),
        api_key,
        language,
        &[("page", &page_str)],
    )
}
