use chrono::{DateTime, Utc};
use serde_json::Value;

use super::{Episode, nonempty, rating, strip_html, text, video, year_of};
use crate::tmdb::{format_runtime, release_info, release_timestamp, slug};
use crate::types::{BehaviorHints, Meta, Trailer, Video};

pub const API_BASE: &str = "https://kitsu.io/api/edge";
pub const EPISODE_PAGE: usize = 20;
pub const MAX_EPISODE_PAGES: usize = 30;

pub fn detail_url(id: u64) -> String {
    format!("{API_BASE}/anime/{id}?include=genres")
}

pub fn episodes_url(id: u64, offset: usize) -> String {
    format!("{API_BASE}/anime/{id}/episodes?page[limit]={EPISODE_PAGE}&page[offset]={offset}")
}

pub fn has_next_page(page: &Value) -> bool {
    page.pointer("/links/next").is_some()
}

fn status(raw: &str) -> &'static str {
    match raw {
        "current" => "Returning Series",
        "upcoming" | "unreleased" | "tba" => "Planned",
        _ => "Ended",
    }
}

pub fn build_videos(pages: &[Value], show_id: &str, now: DateTime<Utc>) -> Vec<Video> {
    let mut out: Vec<Video> = pages
        .iter()
        .flat_map(|p| {
            p.get("data")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
        })
        .filter_map(|ep| {
            let a = ep.get("attributes")?;
            let number = a.get("number").and_then(Value::as_u64)? as u32;
            Some(video(
                Episode {
                    show_id,
                    season: 1,
                    number,
                    anime: true,
                    title: text(a, "canonicalTitle"),
                    overview: &strip_html(text(a, "synopsis")),
                    thumbnail: a.pointer("/thumbnail/original").and_then(Value::as_str),
                    date: text(a, "airdate"),
                    minutes: a.get("length").and_then(Value::as_u64).map(|m| m as u32),
                },
                now,
            ))
        })
        .collect();
    out.sort_by_key(|v| v.episode);
    out.dedup_by_key(|v| v.episode);
    out
}

pub fn build_meta(doc: &Value, requested_id: &str, episodes: Vec<Video>) -> Option<Meta> {
    let data = doc.get("data")?;
    let kitsu: u64 = text(data, "id").parse().ok()?;
    let a = data.get("attributes")?;
    let titles = a.get("titles").unwrap_or(&Value::Null);
    let name = nonempty(text(titles, "en"))
        .or_else(|| nonempty(text(a, "canonicalTitle")))
        .or_else(|| nonempty(text(titles, "en_jp")))
        .unwrap_or_default();
    let movie = text(a, "subtype") == "movie";
    let kind = if movie { "movie" } else { "series" };
    let first = text(a, "startDate");
    let last = text(a, "endDate");
    let raw_status = text(a, "status");
    let minutes = a
        .get("episodeLength")
        .and_then(Value::as_u64)
        .map(|m| m as u32);
    let id = requested_id.to_string();
    let genres = doc
        .get("included")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter(|i| text(i, "type") == "genres")
                .filter_map(|i| i.pointer("/attributes/name").and_then(Value::as_str))
                .map(String::from)
                .collect()
        })
        .unwrap_or_default();
    let score = text(a, "averageRating").parse::<f64>().unwrap_or(0.0) / 10.0;
    let youtube = text(a, "youtubeVideoId");
    Some(Meta {
        id: id.clone(),
        kind: kind.into(),
        name: name.clone(),
        description: strip_html(text(a, "synopsis")),
        slug: slug(kind, &name, None, &kitsu.to_string()),
        genres,
        year: year_of(first),
        released: release_timestamp(first, Some("jp")),
        release_info: release_info(kind, first, last, status(raw_status)),
        runtime: minutes.map(format_runtime).unwrap_or_default(),
        status: (!movie).then(|| status(raw_status).to_string()),
        imdb_rating: rating(score),
        poster: a
            .pointer("/posterImage/original")
            .and_then(Value::as_str)
            .and_then(nonempty),
        background: a
            .pointer("/coverImage/original")
            .and_then(Value::as_str)
            .and_then(nonempty),
        trailers: nonempty(youtube)
            .map(|yt| {
                vec![Trailer {
                    source: yt.clone(),
                    kind: "Trailer".into(),
                    name: "Trailer".into(),
                    yt_id: yt,
                    lang: None,
                }]
            })
            .unwrap_or_default(),
        videos: (!movie).then_some(episodes),
        behavior_hints: BehaviorHints {
            default_video_id: movie.then(|| id.clone()),
            has_scheduled_videos: !movie,
        },
        meta_provider: Some("kitsu".into()),
        p_kitsu: Some(kitsu),
        ..Default::default()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use serde_json::json;

    fn now() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2025, 6, 1, 0, 0, 0).unwrap()
    }

    #[test]
    fn episodes_are_ordered_and_deduplicated_across_pages() {
        let page = |numbers: &[u64]| {
            json!({"data": numbers.iter().map(|n| json!({
                "attributes": {"number": n, "canonicalTitle": format!("Part {n}"), "airdate": "2025-01-02"}
            })).collect::<Vec<_>>()})
        };
        let videos = build_videos(&[page(&[3, 4]), page(&[1, 2, 3])], "kitsu:7", now());
        let numbers: Vec<_> = videos.iter().map(|v| v.episode).collect();
        assert_eq!(numbers, vec![1, 2, 3, 4]);
        assert!(videos[0].available);
        assert_eq!(videos[0].id, "kitsu:7:1");
    }

    #[test]
    fn rating_is_rescaled_and_genres_come_from_included_items() {
        let doc = json!({
            "data": {"id": "7", "attributes": {
                "canonicalTitle": "Example Show", "titles": {"en": "", "en_jp": "Example"},
                "synopsis": "Sand.", "startDate": "2024-04-01", "status": "finished",
                "subtype": "TV", "averageRating": "83.00", "episodeLength": 24,
                "posterImage": {"original": "https://img.example/p.jpg"}
            }},
            "included": [{"type": "genres", "attributes": {"name": "Drama"}}]
        });
        let meta = build_meta(&doc, "kitsu:7", Vec::new()).unwrap();
        assert_eq!(meta.name, "Example Show");
        assert_eq!(meta.imdb_rating, "8.3");
        assert_eq!(meta.genres, vec!["Drama"]);
        assert_eq!(meta.status.as_deref(), Some("Ended"));
    }
}
