use chrono::{DateTime, Utc};
use serde_json::Value;

use super::{nonempty, rating, strip_html, text, year_of};
use crate::tmdb::{format_runtime, release_info, release_timestamp, slug};
use crate::types::{BehaviorHints, Meta, Trailer, Video};

pub const DEFAULT_BASE: &str = "https://api.jikan.moe/v4";
pub const MAX_EPISODE_PAGES: usize = 20;

pub fn detail_url(base: &str, id: u64) -> String {
    format!("{}/anime/{id}/full", base.trim_end_matches('/'))
}

pub fn episodes_url(base: &str, id: u64, page: usize) -> String {
    format!(
        "{}/anime/{id}/episodes?page={page}",
        base.trim_end_matches('/')
    )
}

pub fn has_next_page(page: &Value) -> bool {
    page.pointer("/pagination/has_next_page")
        .and_then(Value::as_bool)
        .unwrap_or(false)
}

pub fn minutes(duration: &str) -> Option<u32> {
    let mut total = 0;
    let mut pending: Option<u32> = None;
    for word in duration.split_whitespace() {
        if let Ok(n) = word.parse::<u32>() {
            pending = Some(n);
        } else if let Some(n) = pending.take() {
            if word.starts_with("hr") {
                total += n * 60;
            } else if word.starts_with("min") {
                total += n;
            }
        }
    }
    (total > 0).then_some(total)
}

fn status(raw: &str) -> &'static str {
    match raw {
        "Currently Airing" => "Returning Series",
        "Not yet aired" => "Planned",
        _ => "Ended",
    }
}

pub fn build_videos(
    pages: &[Value],
    show_id: &str,
    runtime: Option<u32>,
    now: DateTime<Utc>,
) -> Vec<Video> {
    let mut out: Vec<Video> = pages
        .iter()
        .flat_map(|p| {
            p.get("data")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
        })
        .filter_map(|ep| {
            let number = ep.get("mal_id").and_then(Value::as_u64)? as u32;
            let aired = text(ep, "aired");
            let mut v = super::video(
                super::Episode {
                    show_id,
                    season: 1,
                    number,
                    anime: true,
                    title: text(ep, "title"),
                    overview: "",
                    thumbnail: None,
                    date: aired,
                    minutes: runtime,
                },
                now,
            );
            v.is_filler = ep.get("filler").and_then(Value::as_bool).filter(|f| *f);
            v.is_recap = ep.get("recap").and_then(Value::as_bool).filter(|f| *f);
            Some(v)
        })
        .collect();
    out.sort_by_key(|v| v.episode);
    out.dedup_by_key(|v| v.episode);
    out
}

pub fn build_meta(doc: &Value, requested_id: &str, episodes: Vec<Video>) -> Option<Meta> {
    let a = doc.get("data").unwrap_or(doc);
    let mal = a.get("mal_id").and_then(Value::as_u64)?;
    let name = nonempty(text(a, "title_english"))
        .or_else(|| nonempty(text(a, "title")))
        .unwrap_or_default();
    let movie = text(a, "type") == "Movie";
    let kind = if movie { "movie" } else { "series" };
    let first = a
        .pointer("/aired/from")
        .and_then(Value::as_str)
        .unwrap_or("");
    let last = a.pointer("/aired/to").and_then(Value::as_str).unwrap_or("");
    let first = first.get(..10).unwrap_or("");
    let last = last.get(..10).unwrap_or("");
    let raw_status = text(a, "status");
    let names = |key: &str| -> Vec<String> {
        a.get(key)
            .and_then(Value::as_array)
            .map(|items| items.iter().map(|g| text(g, "name").to_string()).collect())
            .unwrap_or_default()
    };
    let mut genres = names("genres");
    genres.extend(names("themes"));
    let id = requested_id.to_string();
    let youtube = a
        .pointer("/trailer/youtube_id")
        .and_then(Value::as_str)
        .unwrap_or("");
    Some(Meta {
        id: id.clone(),
        kind: kind.into(),
        name: name.clone(),
        description: strip_html(text(a, "synopsis")),
        slug: slug(kind, &name, None, &mal.to_string()),
        genres,
        year: year_of(first),
        released: release_timestamp(first, Some("jp")),
        release_info: release_info(kind, first, last, status(raw_status)),
        runtime: minutes(text(a, "duration"))
            .map(format_runtime)
            .unwrap_or_default(),
        status: (!movie).then(|| status(raw_status).to_string()),
        imdb_rating: rating(a.get("score").and_then(Value::as_f64).unwrap_or(0.0)),
        poster: a
            .pointer("/images/jpg/large_image_url")
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
        meta_provider: Some("mal".into()),
        p_mal: Some(mal),
        ..Default::default()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use serde_json::json;

    #[test]
    fn durations_with_hours_and_minutes_are_summed() {
        assert_eq!(minutes("1 hr 55 min"), Some(115));
        assert_eq!(minutes("24 min per ep"), Some(24));
        assert_eq!(minutes("Unknown"), None);
    }

    #[test]
    fn filler_and_recap_flags_are_carried_over() {
        let now = Utc.with_ymd_and_hms(2025, 6, 1, 0, 0, 0).unwrap();
        let page = json!({"data": [
            {"mal_id": 1, "title": "One", "aired": "2024-01-01T00:00:00+00:00", "filler": false, "recap": false},
            {"mal_id": 2, "title": "Two", "aired": "2024-01-08T00:00:00+00:00", "filler": true, "recap": false}
        ]});
        let videos = build_videos(&[page], "mal:5", Some(24), now);
        assert_eq!(videos[0].is_filler, None);
        assert_eq!(videos[1].is_filler, Some(true));
        assert_eq!(videos[1].runtime.as_deref(), Some("24min"));
    }

    #[test]
    fn base_url_trailing_slash_is_ignored() {
        assert_eq!(
            detail_url("https://jikan.example/v4/", 5),
            "https://jikan.example/v4/anime/5/full"
        );
    }
}
