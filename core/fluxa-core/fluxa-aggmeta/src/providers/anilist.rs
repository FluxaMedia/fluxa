use chrono::{DateTime, Utc};
use serde_json::{Value, json};

use super::{Episode, nonempty, rating, strip_html, text, video, year_of};
use crate::catalog::ProviderPage;
use crate::ids::IdSet;
use crate::service::HttpRequest;
use crate::tmdb::{format_runtime, release_info, slug};
use crate::tmdb_catalog::AnimeList;
use crate::types::{AppExtras, BehaviorHints, Meta, Trailer, Video};

pub const ENDPOINT: &str = "https://graphql.anilist.co";

const QUERY: &str = "query($id:Int,$mal:Int){Media(id:$id,idMal:$mal,type:ANIME){id idMal title{romaji english native} description(asHtml:false) genres tags{name} format status episodes duration startDate{year month day} endDate{year month day} countryOfOrigin averageScore coverImage{extraLarge} bannerImage trailer{id site} studios(isMain:true){nodes{name}} streamingEpisodes{title thumbnail} nextAiringEpisode{episode}}}";

pub fn request(ids: &IdSet) -> Option<HttpRequest> {
    let variables = match (ids.anilist, ids.mal) {
        (Some(id), _) => json!({ "id": id }),
        (None, Some(mal)) => json!({ "mal": mal }),
        _ => return None,
    };
    Some(HttpRequest {
        url: ENDPOINT.into(),
        post_body: Some(json!({ "query": QUERY, "variables": variables }).to_string()),
        headers: vec![
            ("Content-Type".into(), "application/json".into()),
            ("Accept".into(), "application/json".into()),
        ],
    })
}

fn date(v: Option<&Value>) -> String {
    let Some(v) = v else { return String::new() };
    let part = |key: &str| v.get(key).and_then(Value::as_u64);
    match (part("year"), part("month"), part("day")) {
        (Some(y), Some(m), Some(d)) => format!("{y:04}-{m:02}-{d:02}"),
        (Some(y), Some(m), None) => format!("{y:04}-{m:02}-01"),
        (Some(y), None, _) => format!("{y:04}-01-01"),
        _ => String::new(),
    }
}

fn status(raw: &str) -> &'static str {
    match raw {
        "FINISHED" => "Ended",
        "RELEASING" => "Returning Series",
        "NOT_YET_RELEASED" => "Planned",
        "CANCELLED" => "Canceled",
        _ => "Ended",
    }
}

fn episode_titles(media: &Value) -> Vec<(u32, String, Option<String>)> {
    media
        .get("streamingEpisodes")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|e| {
            let title = text(e, "title");
            let rest = title.strip_prefix("Episode ")?;
            let (number, name) = rest.split_once(" - ").unwrap_or((rest, ""));
            Some((
                number.trim().parse().ok()?,
                name.trim().to_string(),
                nonempty(text(e, "thumbnail")),
            ))
        })
        .collect()
}

pub fn build_videos(media: &Value, show_id: &str, now: DateTime<Utc>) -> Vec<Video> {
    let next = media
        .pointer("/nextAiringEpisode/episode")
        .and_then(Value::as_u64)
        .map(|n| n as u32);
    let total = media
        .get("episodes")
        .and_then(Value::as_u64)
        .map(|n| n as u32)
        .or_else(|| next.map(|n| n.saturating_sub(1)))
        .unwrap_or(0);
    let minutes = media
        .get("duration")
        .and_then(Value::as_u64)
        .map(|m| m as u32);
    let titles = episode_titles(media);
    (1..=total)
        .map(|number| {
            let found = titles.iter().find(|(n, _, _)| *n == number);
            let mut v = video(
                Episode {
                    show_id,
                    season: 1,
                    number,
                    anime: true,
                    title: found.map_or("", |(_, t, _)| t.as_str()),
                    overview: "",
                    thumbnail: found.and_then(|(_, _, t)| t.as_deref()),
                    date: "",
                    minutes,
                },
                now,
            );
            v.available = next.is_none_or(|n| number < n);
            v
        })
        .collect()
}

pub fn build_meta(response: &Value, requested_id: &str, now: DateTime<Utc>) -> Option<Meta> {
    let media = response.pointer("/data/Media").unwrap_or(response);
    let anilist = media.get("id").and_then(Value::as_u64)?;
    let title = media.get("title").unwrap_or(&Value::Null);
    let name = nonempty(text(title, "english"))
        .or_else(|| nonempty(text(title, "romaji")))
        .or_else(|| nonempty(text(title, "native")))
        .unwrap_or_default();
    let movie = text(media, "format") == "MOVIE";
    let kind = if movie { "movie" } else { "series" };
    let first = date(media.get("startDate"));
    let last = date(media.get("endDate"));
    let raw_status = text(media, "status");
    let minutes = media
        .get("duration")
        .and_then(Value::as_u64)
        .map(|m| m as u32);
    let trailer = media
        .get("trailer")
        .filter(|t| text(t, "site") == "youtube");
    let id = requested_id.to_string();
    let studios = media
        .pointer("/studios/nodes")
        .and_then(Value::as_array)
        .map(|nodes| {
            nodes
                .iter()
                .map(|n| text(n, "name").to_string())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    Some(Meta {
        id: id.clone(),
        kind: kind.into(),
        name: name.clone(),
        description: strip_html(text(media, "description")),
        slug: slug(kind, &name, None, &anilist.to_string()),
        genres: media
            .get("genres")
            .and_then(Value::as_array)
            .map(|g| {
                g.iter()
                    .filter_map(Value::as_str)
                    .map(String::from)
                    .collect()
            })
            .unwrap_or_default(),
        year: year_of(&first),
        released: crate::tmdb::release_timestamp(&first, Some("jp")),
        release_info: release_info(kind, &first, &last, status(raw_status)),
        runtime: minutes.map(format_runtime).unwrap_or_default(),
        country: text(media, "countryOfOrigin").to_string(),
        status: (!movie).then(|| status(raw_status).to_string()),
        imdb_rating: rating(
            media
                .get("averageScore")
                .and_then(Value::as_f64)
                .unwrap_or(0.0)
                / 10.0,
        ),
        poster: nonempty(
            media
                .pointer("/coverImage/extraLarge")
                .and_then(Value::as_str)
                .unwrap_or(""),
        ),
        background: nonempty(text(media, "bannerImage")),
        trailers: trailer
            .map(|t| {
                vec![Trailer {
                    source: text(t, "id").to_string(),
                    kind: "Trailer".into(),
                    name: "Trailer".into(),
                    yt_id: text(t, "id").to_string(),
                    lang: None,
                }]
            })
            .unwrap_or_default(),
        videos: (!movie).then(|| build_videos(media, &id, now)),
        behavior_hints: BehaviorHints {
            default_video_id: movie.then(|| id.clone()),
            has_scheduled_videos: !movie,
        },
        keywords: media
            .get("tags")
            .and_then(Value::as_array)
            .map(|t| t.iter().map(|x| text(x, "name").to_string()).collect())
            .unwrap_or_default(),
        app_extras: AppExtras {
            directors: studios,
            ..Default::default()
        },
        meta_provider: Some("anilist".into()),
        p_anilist: Some(anilist),
        p_mal: media.get("idMal").and_then(Value::as_u64),
        ..Default::default()
    })
}

const CATALOG_QUERY: &str = "query($page:Int,$sort:[MediaSort],$status:MediaStatus){Page(page:$page,perPage:20){pageInfo{hasNextPage} media(type:ANIME,sort:$sort,status:$status,isAdult:false){id title{romaji english} description(asHtml:false) averageScore startDate{year} coverImage{extraLarge} bannerImage}}}";

pub fn catalog_request(list: AnimeList, page: u32) -> HttpRequest {
    let (sort, status) = match list {
        AnimeList::Trending => ("TRENDING_DESC", None),
        AnimeList::Popular => ("POPULARITY_DESC", None),
        AnimeList::Airing => ("POPULARITY_DESC", Some("RELEASING")),
        AnimeList::TopRated => ("SCORE_DESC", None),
    };
    HttpRequest {
        url: ENDPOINT.into(),
        post_body: Some(
            json!({"query": CATALOG_QUERY, "variables": {"page": page, "sort": [sort], "status": status}})
                .to_string(),
        ),
        headers: vec![
            ("Content-Type".into(), "application/json".into()),
            ("Accept".into(), "application/json".into()),
        ],
    }
}

const SEARCH_QUERY: &str = "query($page:Int,$q:String){Page(page:$page,perPage:20){pageInfo{hasNextPage} media(type:ANIME,search:$q,sort:[SEARCH_MATCH],isAdult:false){id title{romaji english} description(asHtml:false) averageScore startDate{year} coverImage{extraLarge} bannerImage}}}";

pub fn search_request(query: &str, page: u32) -> HttpRequest {
    HttpRequest {
        url: ENDPOINT.into(),
        post_body: Some(
            json!({"query": SEARCH_QUERY, "variables": {"page": page, "q": query}}).to_string(),
        ),
        headers: vec![
            ("Content-Type".into(), "application/json".into()),
            ("Accept".into(), "application/json".into()),
        ],
    }
}

const USER_LIST_QUERY: &str = "query($user:String,$status:MediaListStatus){MediaListCollection(userName:$user,type:ANIME,status:$status){lists{entries{media{id title{romaji english} description(asHtml:false) averageScore startDate{year} coverImage{extraLarge} bannerImage}}}}}";

pub const VIEWER_USER: &str = "@me";

const VIEWER_QUERY: &str = "query{Viewer{name}}";

pub fn viewer_request(token: &str) -> HttpRequest {
    let mut request = HttpRequest {
        url: ENDPOINT.into(),
        post_body: Some(json!({"query": VIEWER_QUERY}).to_string()),
        headers: vec![
            ("Content-Type".into(), "application/json".into()),
            ("Accept".into(), "application/json".into()),
        ],
    };
    request
        .headers
        .push(("Authorization".into(), format!("Bearer {token}")));
    request
}

pub fn viewer_name(body: &Value) -> Option<String> {
    body.pointer("/data/Viewer/name")
        .and_then(Value::as_str)
        .filter(|name| !name.is_empty())
        .map(str::to_owned)
}

pub fn user_list_request(user: &str, status: &str, token: &str) -> HttpRequest {
    let mut request = user_list_public(user, status);
    if !token.is_empty() {
        request
            .headers
            .push(("Authorization".into(), format!("Bearer {token}")));
    }
    request
}

fn user_list_public(user: &str, status: &str) -> HttpRequest {
    HttpRequest {
        url: ENDPOINT.into(),
        post_body: Some(
            json!({"query": USER_LIST_QUERY, "variables": {"user": user, "status": status}})
                .to_string(),
        ),
        headers: vec![
            ("Content-Type".into(), "application/json".into()),
            ("Accept".into(), "application/json".into()),
        ],
    }
}

pub fn parse_user_list(body: &Value, page: u32) -> ProviderPage<Meta> {
    let media: Vec<Value> = body
        .pointer("/data/MediaListCollection/lists")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .flat_map(|list| {
            list.get("entries")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
        })
        .filter_map(|entry| entry.get("media").cloned())
        .collect();
    let size = crate::tmdb_catalog::PAGE_SIZE;
    let start = (page.max(1) as usize - 1) * size;
    let wrapped = json!({"data": {"Page": {
        "pageInfo": {"hasNextPage": media.len() > start + size},
        "media": media.iter().skip(start).take(size).collect::<Vec<_>>()
    }}});
    parse_catalog(&wrapped)
}

pub fn parse_catalog(body: &Value) -> ProviderPage<Meta> {
    let page = body.pointer("/data/Page");
    let items = page
        .and_then(|p| p.get("media"))
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|item| {
            let id = item.get("id").and_then(Value::as_u64)?;
            let title = item.get("title").unwrap_or(&Value::Null);
            let name =
                nonempty(text(title, "english")).or_else(|| nonempty(text(title, "romaji")))?;
            let year = item
                .pointer("/startDate/year")
                .and_then(Value::as_u64)
                .map(|y| y.to_string())
                .unwrap_or_default();
            let score = item
                .get("averageScore")
                .and_then(Value::as_f64)
                .unwrap_or(0.0)
                / 10.0;
            Some(Meta {
                id: format!("anilist:{id}"),
                kind: "series".into(),
                name,
                description: strip_html(text(item, "description")),
                release_info: year.clone(),
                year,
                imdb_rating: rating(score),
                poster: item
                    .pointer("/coverImage/extraLarge")
                    .and_then(Value::as_str)
                    .and_then(nonempty),
                background: nonempty(text(item, "bannerImage")),
                p_anilist: Some(id),
                ..Meta::default()
            })
        })
        .collect();
    ProviderPage {
        items,
        has_next_page: page
            .and_then(|p| p.pointer("/pageInfo/hasNextPage"))
            .and_then(Value::as_bool)
            .unwrap_or(false),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn now() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2025, 6, 1, 0, 0, 0).unwrap()
    }

    fn show() -> Value {
        json!({"data": {"Media": {
            "id": 10, "idMal": 20,
            "title": {"romaji": "Example Show", "english": null, "native": "x"},
            "description": "A <b>tale</b><br>of sand.",
            "genres": ["Drama"], "format": "TV", "status": "RELEASING",
            "episodes": null, "duration": 24,
            "startDate": {"year": 2025, "month": 4, "day": 5}, "endDate": {"year": null, "month": null, "day": null},
            "countryOfOrigin": "JP", "averageScore": 82,
            "coverImage": {"extraLarge": "https://img.example/p.jpg"}, "bannerImage": "https://img.example/b.jpg",
            "trailer": {"id": "abc123", "site": "youtube"},
            "studios": {"nodes": [{"name": "Example Studio"}]},
            "streamingEpisodes": [{"title": "Episode 2 - Second", "thumbnail": "https://img.example/2.jpg"}],
            "nextAiringEpisode": {"episode": 4}
        }}})
    }

    #[test]
    fn airing_show_lists_only_aired_episodes_as_available() {
        let meta = build_meta(&show(), "anilist:10", now()).unwrap();
        let videos = meta.videos.unwrap();
        assert_eq!(videos.len(), 3);
        assert!(videos.iter().all(|v| v.available));
        assert_eq!(videos[1].title, "Second");
        assert_eq!(videos[1].id, "anilist:10:2");
    }

    #[test]
    fn title_falls_back_to_romaji_and_markup_is_removed() {
        let meta = build_meta(&show(), "anilist:10", now()).unwrap();
        assert_eq!(meta.name, "Example Show");
        assert_eq!(meta.description, "A tale of sand.");
        assert_eq!(meta.imdb_rating, "8.2");
        assert_eq!(meta.p_mal, Some(20));
    }

    #[test]
    fn mal_id_is_used_when_no_anilist_id_is_known() {
        let ids = IdSet {
            mal: Some(20),
            ..Default::default()
        };
        let body = request(&ids).unwrap().post_body.unwrap();
        assert!(body.contains("\"mal\":20"));
    }

    #[test]
    fn catalog_rows_become_anilist_series_cards() {
        let body = json!({"data": {"Page": {
            "pageInfo": {"hasNextPage": true},
            "media": [
                {"id": 5, "title": {"romaji": "Example Romaji", "english": null},
                 "description": "A <b>tale</b>.", "averageScore": 81,
                 "startDate": {"year": 2024}, "coverImage": {"extraLarge": "https://img.example/c.jpg"}},
                {"id": 6, "title": {"romaji": null, "english": null}}
            ]
        }}});
        let page = parse_catalog(&body);
        assert_eq!(page.items.len(), 1);
        assert_eq!(page.items[0].id, "anilist:5");
        assert_eq!(page.items[0].name, "Example Romaji");
        assert_eq!(page.items[0].imdb_rating, "8.1");
        assert!(page.has_next_page);
    }
}
