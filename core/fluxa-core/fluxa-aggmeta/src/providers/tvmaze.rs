use chrono::{DateTime, Utc};
use serde_json::Value;

use super::{Episode, nonempty, rating, strip_html, text, video, year_of};
use crate::ids::IdSet;
use crate::tmdb::{format_runtime, release_info, release_timestamp, slug};
use crate::types::{AppExtras, BehaviorHints, CastMember, Meta};

pub const API_BASE: &str = "https://api.tvmaze.com";
const EMBED: &str = "embed[]=episodes&embed[]=cast";

pub fn show_url(ids: &IdSet) -> Option<String> {
    if let Some(id) = ids.tvmaze {
        return Some(format!("{API_BASE}/shows/{id}?{EMBED}"));
    }
    if let Some(imdb) = &ids.imdb {
        return Some(format!("{API_BASE}/lookup/shows?imdb={imdb}"));
    }
    ids.tvdb
        .map(|id| format!("{API_BASE}/lookup/shows?thetvdb={id}"))
}

pub fn embedded_url(id: u64) -> String {
    format!("{API_BASE}/shows/{id}?{EMBED}")
}

pub fn needs_embed(show: &Value) -> bool {
    show.pointer("/_embedded/episodes").is_none()
}

fn status(raw: &str) -> &'static str {
    match raw {
        "Running" => "Returning Series",
        "In Development" | "To Be Determined" => "Planned",
        _ => "Ended",
    }
}

pub fn build_meta(show: &Value, requested_id: &str, now: DateTime<Utc>) -> Option<Meta> {
    let tvmaze = show.get("id").and_then(Value::as_u64)?;
    let imdb = nonempty(
        show.pointer("/externals/imdb")
            .and_then(Value::as_str)
            .unwrap_or(""),
    );
    let name = text(show, "name").to_string();
    let first = text(show, "premiered");
    let last = text(show, "ended");
    let raw_status = text(show, "status");
    let id = imdb.clone().unwrap_or_else(|| requested_id.to_string());
    let minutes = show
        .get("runtime")
        .and_then(Value::as_u64)
        .or_else(|| show.get("averageRuntime").and_then(Value::as_u64))
        .map(|m| m as u32);
    let episodes = show
        .pointer("/_embedded/episodes")
        .and_then(Value::as_array)
        .map(|list| {
            list.iter()
                .filter_map(|ep| {
                    let number = ep.get("number").and_then(Value::as_u64)? as u32;
                    let season = ep.get("season").and_then(Value::as_u64)? as u32;
                    let date = text(ep, "airstamp");
                    let date = if date.is_empty() {
                        text(ep, "airdate")
                    } else {
                        date
                    };
                    Some(video(
                        Episode {
                            show_id: &id,
                            season,
                            number,
                            anime: false,
                            title: text(ep, "name"),
                            overview: &strip_html(text(ep, "summary")),
                            thumbnail: ep.pointer("/image/original").and_then(Value::as_str),
                            date,
                            minutes: ep.get("runtime").and_then(Value::as_u64).map(|m| m as u32),
                        },
                        now,
                    ))
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let cast = show
        .pointer("/_embedded/cast")
        .and_then(Value::as_array)
        .map(|list| {
            list.iter()
                .map(|c| CastMember {
                    name: c
                        .pointer("/person/name")
                        .and_then(Value::as_str)
                        .unwrap_or("")
                        .into(),
                    character: c
                        .pointer("/character/name")
                        .and_then(Value::as_str)
                        .unwrap_or("")
                        .into(),
                    photo: c
                        .pointer("/person/image/original")
                        .and_then(Value::as_str)
                        .map(String::from),
                    tmdb_id: None,
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    Some(Meta {
        id: id.clone(),
        kind: "series".into(),
        name: name.clone(),
        description: strip_html(text(show, "summary")),
        imdb_id: imdb.clone(),
        slug: slug("series", &name, imdb.as_deref(), &tvmaze.to_string()),
        genres: show
            .get("genres")
            .and_then(Value::as_array)
            .map(|g| {
                g.iter()
                    .filter_map(Value::as_str)
                    .map(String::from)
                    .collect()
            })
            .unwrap_or_default(),
        year: year_of(first),
        released: release_timestamp(first, None),
        release_info: release_info(
            "series",
            first,
            last,
            if raw_status == "Running" {
                "Returning Series"
            } else {
                "Ended"
            },
        ),
        runtime: minutes.map(format_runtime).unwrap_or_default(),
        country: show
            .pointer("/network/country/name")
            .or_else(|| show.pointer("/webChannel/country/name"))
            .and_then(Value::as_str)
            .unwrap_or("")
            .into(),
        status: Some(status(raw_status).to_string()),
        imdb_rating: rating(
            show.pointer("/rating/average")
                .and_then(Value::as_f64)
                .unwrap_or(0.0),
        ),
        poster: show
            .pointer("/image/original")
            .and_then(Value::as_str)
            .and_then(nonempty),
        videos: Some(episodes),
        behavior_hints: BehaviorHints {
            default_video_id: None,
            has_scheduled_videos: true,
        },
        app_extras: AppExtras {
            cast,
            ..Default::default()
        },
        meta_provider: Some("tvmaze".into()),
        p_imdb: imdb,
        p_tvdb: show.pointer("/externals/thetvdb").and_then(Value::as_u64),
        ..Default::default()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use serde_json::json;

    #[test]
    fn lookup_prefers_a_known_tvmaze_id() {
        let ids = IdSet {
            tvmaze: Some(5),
            imdb: Some("tt1".into()),
            ..Default::default()
        };
        assert!(show_url(&ids).unwrap().contains("/shows/5?"));
        let ids = IdSet {
            tvdb: Some(9),
            ..Default::default()
        };
        assert!(show_url(&ids).unwrap().ends_with("thetvdb=9"));
        assert!(show_url(&IdSet::default()).is_none());
    }

    #[test]
    fn episodes_use_season_numbering_and_future_ones_are_unavailable() {
        let now = Utc.with_ymd_and_hms(2025, 6, 1, 0, 0, 0).unwrap();
        let show = json!({
            "id": 5, "name": "Example Show", "summary": "<p>Sand.</p>", "status": "Running",
            "premiered": "2024-01-01", "externals": {"imdb": "tt7", "thetvdb": 11},
            "_embedded": {"episodes": [
                {"season": 1, "number": 1, "name": "One", "airstamp": "2024-01-01T20:00:00+00:00"},
                {"season": 1, "number": 2, "name": "Two", "airstamp": "2030-01-01T20:00:00+00:00"}
            ]}
        });
        let meta = build_meta(&show, "tvmaze:5", now).unwrap();
        let videos = meta.videos.unwrap();
        assert_eq!(meta.id, "tt7");
        assert_eq!(videos[0].id, "tt7:1:1");
        assert!(videos[0].available);
        assert!(!videos[1].available);
        assert_eq!(meta.p_tvdb, Some(11));
    }
}
