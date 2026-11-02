use chrono::{DateTime, Utc};
use serde_json::{Value, json};

use super::{Episode, nonempty, strip_html, text, video, year_of};
use crate::service::HttpRequest;
use crate::tmdb::{format_runtime, release_info, release_timestamp, slug};
use crate::types::{AppExtras, BehaviorHints, CastMember, Meta, Trailer, Video};

pub const API_BASE: &str = "https://api4.thetvdb.com/v4";
pub const MAX_EPISODE_PAGES: usize = 30;

pub fn login_request(api_key: &str) -> HttpRequest {
    HttpRequest {
        url: format!("{API_BASE}/login"),
        post_body: Some(json!({ "apikey": api_key }).to_string()),
        headers: vec![
            ("Content-Type".into(), "application/json".into()),
            ("Accept".into(), "application/json".into()),
        ],
    }
}

pub fn token(login: &Value) -> Option<String> {
    login
        .pointer("/data/token")
        .and_then(Value::as_str)
        .and_then(nonempty)
}

pub fn get(path: &str, token: &str) -> HttpRequest {
    HttpRequest {
        url: format!("{API_BASE}{path}"),
        post_body: None,
        headers: vec![
            ("Authorization".into(), format!("Bearer {token}")),
            ("Accept".into(), "application/json".into()),
        ],
    }
}

pub fn detail_path(movie: bool, id: u64) -> String {
    if movie {
        format!("/movies/{id}/extended")
    } else {
        format!("/series/{id}/extended")
    }
}

pub fn episodes_path(id: u64, order: &str, page: usize) -> String {
    let order = match order {
        "dvd" | "absolute" | "alternate" => order,
        _ => "default",
    };
    format!("/series/{id}/episodes/{order}?page={page}")
}

pub const SEARCH_PAGE: usize = 20;

pub fn search_path(movie: bool, query: &str, page: u32) -> String {
    format!(
        "/search?query={}&type={}&limit={SEARCH_PAGE}&offset={}",
        crate::search::encode_component(query),
        if movie { "movie" } else { "series" },
        (page.max(1) as usize - 1) * SEARCH_PAGE
    )
}

pub fn parse_search(movie: bool, body: &Value) -> crate::catalog::ProviderPage<Meta> {
    let rows = body
        .get("data")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let items = rows
        .iter()
        .filter_map(|row| {
            let id = text(row, "tvdb_id").parse::<u64>().ok()?;
            let name = nonempty(text(row, "name"))?;
            let year = text(row, "year").to_owned();
            Some(Meta {
                id: format!("tvdb:{id}"),
                kind: if movie { "movie" } else { "series" }.into(),
                name,
                description: strip_html(text(row, "overview")),
                release_info: year.clone(),
                year,
                poster: nonempty(text(row, "image_url")),
                p_tvdb: Some(id),
                ..Meta::default()
            })
        })
        .collect();
    crate::catalog::ProviderPage {
        items,
        has_next_page: rows.len() >= SEARCH_PAGE,
    }
}

pub fn filter_path(movie: bool, sort: &str, page: u32) -> String {
    filter_path_with(movie, sort, "", page)
}

pub fn filter_path_with(movie: bool, sort: &str, extra: &str, page: u32) -> String {
    let extra = if extra.is_empty() {
        String::new()
    } else {
        format!("&{extra}")
    };
    format!(
        "/{}/filter?sort={sort}&sortType=desc&lang=eng&country=usa{extra}&page={}",
        if movie { "movies" } else { "series" },
        (page.max(1) - 1) as usize / FILTER_CHUNKS
    )
}

pub fn list_path(id: u64) -> String {
    format!("/lists/{id}/extended")
}

pub fn base_path(movie: bool, id: u64) -> String {
    format!("/{}/{id}", if movie { "movies" } else { "series" })
}

pub fn list_entities(body: &Value, movie: bool) -> Vec<u64> {
    let key = if movie { "movieId" } else { "seriesId" };
    let mut rows: Vec<(u64, u64)> = body
        .pointer("/data/entities")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|e| {
            Some((
                e.get("order").and_then(Value::as_u64).unwrap_or(0),
                e.get(key).and_then(Value::as_u64)?,
            ))
        })
        .collect();
    rows.sort_by_key(|(order, _)| *order);
    rows.into_iter().map(|(_, id)| id).collect()
}

pub fn parse_list_rows(movie: bool, body: &Value) -> crate::catalog::ProviderPage<Meta> {
    let items = body
        .get("data")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|row| row_meta(movie, row))
        .collect();
    crate::catalog::ProviderPage {
        items,
        has_next_page: body.get("more").and_then(Value::as_bool).unwrap_or(false),
    }
}

fn row_meta(movie: bool, row: &Value) -> Option<Meta> {
    let id = row.get("id").and_then(Value::as_u64)?;
    let name = nonempty(text(row, "name"))?;
    let year = text(row, "year").to_owned();
    Some(Meta {
        id: format!("tvdb:{id}"),
        kind: if movie { "movie" } else { "series" }.into(),
        name,
        description: strip_html(text(row, "overview")),
        release_info: year.clone(),
        year,
        poster: nonempty(text(row, "image")),
        p_tvdb: Some(id),
        ..Meta::default()
    })
}

const FILTER_PAGE: usize = 500;
const FILTER_CHUNKS: usize = FILTER_PAGE / crate::tmdb_catalog::PAGE_SIZE;

pub fn parse_filter(movie: bool, body: &Value, page: u32) -> crate::catalog::ProviderPage<Meta> {
    let rows = body
        .get("data")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let size = crate::tmdb_catalog::PAGE_SIZE;
    let start = (page.max(1) as usize - 1) % FILTER_CHUNKS * size;
    let items = rows
        .iter()
        .skip(start)
        .take(size)
        .filter_map(|row| row_meta(movie, row))
        .collect();
    crate::catalog::ProviderPage {
        items,
        has_next_page: rows.len() > start + size || has_next_page(body),
    }
}

pub fn has_next_page(page: &Value) -> bool {
    page.pointer("/links/next")
        .is_some_and(|next| !next.is_null())
}

fn artwork(data: &Value, kinds: &[u64]) -> Option<String> {
    data.get("artworks")
        .and_then(Value::as_array)?
        .iter()
        .filter(|a| {
            a.get("type")
                .and_then(Value::as_u64)
                .is_some_and(|t| kinds.contains(&t))
        })
        .max_by_key(|a| a.get("score").and_then(Value::as_i64).unwrap_or(0))
        .and_then(|a| a.get("image").and_then(Value::as_str))
        .and_then(nonempty)
}

fn status(raw: &str) -> &'static str {
    match raw {
        "Continuing" => "Returning Series",
        "Upcoming" => "Planned",
        _ => "Ended",
    }
}

pub fn build_videos(pages: &[Value], show_id: &str, now: DateTime<Utc>) -> Vec<Video> {
    let mut out: Vec<Video> = pages
        .iter()
        .flat_map(|p| {
            p.pointer("/data/episodes")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
        })
        .filter_map(|ep| {
            let season = ep.get("seasonNumber").and_then(Value::as_u64)? as u32;
            let number = ep.get("number").and_then(Value::as_u64)? as u32;
            (season > 0).then(|| {
                video(
                    Episode {
                        show_id,
                        season,
                        number,
                        anime: false,
                        title: text(ep, "name"),
                        overview: text(ep, "overview"),
                        thumbnail: ep.get("image").and_then(Value::as_str),
                        date: text(ep, "aired"),
                        minutes: ep.get("runtime").and_then(Value::as_u64).map(|m| m as u32),
                    },
                    now,
                )
            })
        })
        .collect();
    out.sort_by_key(|v| (v.season, v.episode));
    out
}

pub fn build_meta(
    response: &Value,
    movie: bool,
    requested_id: &str,
    episodes: Vec<Video>,
) -> Option<Meta> {
    let data = response.get("data")?;
    let tvdb = data.get("id").and_then(Value::as_u64)?;
    let kind = if movie { "movie" } else { "series" };
    let imdb = data
        .get("remoteIds")
        .and_then(Value::as_array)
        .and_then(|ids| {
            ids.iter()
                .find(|r| text(r, "sourceName").eq_ignore_ascii_case("imdb"))
        })
        .and_then(|r| r.get("id").and_then(Value::as_str))
        .and_then(nonempty);
    let first = if movie {
        data.get("releases")
            .and_then(Value::as_array)
            .and_then(|r| r.first())
            .map_or("", |r| text(r, "date"))
    } else {
        text(data, "firstAired")
    };
    let last = text(data, "lastAired");
    let raw_status = data
        .pointer("/status/name")
        .and_then(Value::as_str)
        .unwrap_or("");
    let minutes = data
        .get("runtime")
        .or_else(|| data.get("averageRuntime"))
        .and_then(Value::as_u64)
        .map(|m| m as u32);
    let name = nonempty(text(data, "name")).unwrap_or_default();
    let id = imdb.clone().unwrap_or_else(|| requested_id.to_string());
    let (poster, background, logo) = if movie {
        (
            artwork(data, &[14]),
            artwork(data, &[15]),
            artwork(data, &[25]),
        )
    } else {
        (
            artwork(data, &[2]),
            artwork(data, &[3]),
            artwork(data, &[23]),
        )
    };
    let cast = data
        .get("characters")
        .and_then(Value::as_array)
        .map(|list| {
            list.iter()
                .filter(|c| c.get("type").and_then(Value::as_u64) == Some(3))
                .map(|c| CastMember {
                    name: text(c, "personName").into(),
                    character: text(c, "name").into(),
                    photo: nonempty(text(c, "personImgURL")),
                    tmdb_id: None,
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let trailers = data
        .get("trailers")
        .and_then(Value::as_array)
        .map(|list| {
            list.iter()
                .filter_map(|t| {
                    let url = text(t, "url");
                    let yt = url.split("v=").nth(1).or_else(|| url.rsplit('/').next())?;
                    (url.contains("youtu")).then(|| Trailer {
                        source: yt.to_string(),
                        kind: "Trailer".into(),
                        name: text(t, "name").into(),
                        yt_id: yt.to_string(),
                        lang: nonempty(text(t, "language")),
                    })
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    Some(Meta {
        id: id.clone(),
        kind: kind.into(),
        name: name.clone(),
        description: strip_html(text(data, "overview")),
        imdb_id: imdb.clone(),
        slug: slug(kind, &name, imdb.as_deref(), &tvdb.to_string()),
        genres: data
            .get("genres")
            .and_then(Value::as_array)
            .map(|g| g.iter().map(|x| text(x, "name").to_string()).collect())
            .unwrap_or_default(),
        year: nonempty(&year_of(first)).unwrap_or_else(|| text(data, "year").to_string()),
        released: release_timestamp(first, None),
        release_info: release_info(kind, first, last, status(raw_status)),
        runtime: minutes.map(format_runtime).unwrap_or_default(),
        country: text(data, "originalCountry").to_string(),
        status: (!movie).then(|| status(raw_status).to_string()),
        imdb_rating: "N/A".into(),
        poster,
        background,
        logo,
        trailers,
        videos: (!movie).then_some(episodes),
        behavior_hints: BehaviorHints {
            default_video_id: movie.then(|| id.clone()),
            has_scheduled_videos: !movie,
        },
        app_extras: AppExtras {
            cast,
            ..Default::default()
        },
        meta_provider: Some("tvdb".into()),
        p_tvdb: Some(tvdb),
        p_imdb: imdb,
        ..Default::default()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn search_rows_become_cards() {
        let body = json!({"data": [{"tvdb_id": "81189", "name": "Example Show", "year": "2008", "image_url": "https://img.example/p.jpg"}]});
        let page = parse_search(false, &body);
        assert_eq!(page.items[0].id, "tvdb:81189");
        assert_eq!(page.items[0].year, "2008");
        assert!(!page.has_next_page);
        assert!(search_path(true, "a b", 2).contains("type=movie&limit=20&offset=20"));
    }

    #[test]
    fn filter_rows_become_cards_and_pages_start_at_zero() {
        let body = json!({"data": [{"id": 3, "name": "Example Show", "year": "2019", "image": "https://img.example/a.jpg"}], "links": {"next": null}});
        let page = parse_filter(false, &body, 1);
        assert_eq!(page.items[0].id, "tvdb:3");
        assert!(!page.has_next_page);
        assert!(filter_path(true, "score", 1).starts_with("/movies/filter?sort=score"));
        assert!(filter_path(false, "score", 3).ends_with("page=0"));
        assert!(filter_path(false, "score", 26).ends_with("page=1"));
    }

    #[test]
    fn unknown_episode_orders_fall_back_to_default() {
        assert_eq!(episodes_path(5, "dvd", 1), "/series/5/episodes/dvd?page=1");
        assert_eq!(
            episodes_path(5, "official", 0),
            "/series/5/episodes/default?page=0"
        );
    }

    fn now() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2025, 6, 1, 0, 0, 0).unwrap()
    }

    #[test]
    fn best_scored_artwork_of_the_right_type_wins() {
        let response = json!({"data": {
            "id": 9, "name": "Example Show", "firstAired": "2020-01-01",
            "status": {"name": "Continuing"},
            "artworks": [
                {"type": 2, "image": "https://img.example/low.jpg", "score": 10},
                {"type": 2, "image": "https://img.example/high.jpg", "score": 90},
                {"type": 3, "image": "https://img.example/back.jpg", "score": 1}
            ],
            "remoteIds": [{"id": "tt42", "sourceName": "IMDB"}]
        }});
        let meta = build_meta(&response, false, "tvdb:9", Vec::new()).unwrap();
        assert_eq!(meta.poster.as_deref(), Some("https://img.example/high.jpg"));
        assert_eq!(
            meta.background.as_deref(),
            Some("https://img.example/back.jpg")
        );
        assert_eq!(meta.id, "tt42");
        assert_eq!(meta.status.as_deref(), Some("Returning Series"));
    }

    #[test]
    fn specials_are_left_out_of_the_episode_list() {
        let page = json!({"data": {"episodes": [
            {"seasonNumber": 0, "number": 1, "name": "Special"},
            {"seasonNumber": 2, "number": 3, "name": "Three", "aired": "2024-02-02"}
        ]}});
        let videos = build_videos(&[page], "tt42", now());
        assert_eq!(videos.len(), 1);
        assert_eq!(videos[0].id, "tt42:2:3");
    }

    #[test]
    fn token_is_read_from_the_login_response() {
        assert_eq!(
            token(&json!({"data": {"token": "abc"}})).as_deref(),
            Some("abc")
        );
        assert_eq!(token(&json!({"status": "failure"})), None);
    }
    #[test]
    fn list_entities_keep_order_and_pick_one_kind() {
        let body = json!({"data": {"entities": [
            {"order": 2, "movieId": 20},
            {"order": 1, "movieId": 10},
            {"order": 3, "seriesId": 30}
        ]}});
        assert_eq!(list_entities(&body, true), [10, 20]);
        assert_eq!(list_entities(&body, false), [30]);
    }
}
