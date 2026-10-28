use chrono::{DateTime, Duration, NaiveDate, TimeZone, Utc};
use serde_json::Value;

use crate::art::{self, ArtPolicy, ImageCandidate, Translation};
use crate::types::{BehaviorHints, CastMember, Meta, Trailer, Video};

pub const API_BASE: &str = "https://api.themoviedb.org/3";
const APPEND: &str = "videos,credits,external_ids,images,translations,watch/providers,keywords";

pub fn detail_url(kind: &str, id: u64, language: &str, extra_append: &str) -> String {
    let short = language.split('-').next().unwrap_or(language);
    let append = if extra_append.is_empty() {
        APPEND.to_string()
    } else {
        format!("{APPEND},{extra_append}")
    };
    format!(
        "{API_BASE}/{kind}/{id}?language={language}&append_to_response={append}&include_image_language={short},en,null&include_video_language={short},en,null"
    )
}

pub fn season_append_batches(seasons: &[u32]) -> Vec<String> {
    seasons
        .chunks(20)
        .map(|c| {
            c.iter()
                .map(|s| format!("season/{s}"))
                .collect::<Vec<_>>()
                .join(",")
        })
        .collect()
}

pub fn find_url(external_id: &str, source: &str) -> String {
    format!("{API_BASE}/find/{external_id}?external_source={source}")
}

pub fn should_retry(status: u16) -> bool {
    !matches!(status, 400 | 401 | 403 | 404 | 422)
}

pub fn format_runtime(minutes: u32) -> String {
    match (minutes / 60, minutes % 60) {
        (0, m) => format!("{m}min"),
        (h, 0) => format!("{h}h"),
        (h, m) => format!("{h}h {m}min"),
    }
}

pub fn slug(kind: &str, name: &str, imdb: Option<&str>, fallback_id: &str) -> String {
    let tail = imdb
        .map(|i| i.trim_start_matches("tt").to_string())
        .unwrap_or_else(|| fallback_id.to_string());
    format!("{kind}/{}-{tail}", name.to_lowercase().replace(' ', "-"))
}

pub fn release_info(kind: &str, first: &str, last: &str, status: &str) -> String {
    let year = |d: &str| d.get(..4).unwrap_or("").to_string();
    let start = year(first);
    if kind == "movie" || start.is_empty() {
        return start;
    }
    let ongoing =
        matches!(status, "Returning Series" | "In Production" | "Planned") || last.is_empty();
    let end = year(last);
    if ongoing {
        format!("{start}-")
    } else if end.is_empty() || end == start {
        start
    } else {
        format!("{start}-{end}")
    }
}

fn utc_offset_hours(country: &str) -> f64 {
    match country.to_lowercase().as_str() {
        "jp" | "kr" => 9.0,
        "cn" | "hk" | "tw" | "sg" | "ph" => 8.0,
        "in" => 5.5,
        "gb" | "ie" | "pt" => 0.0,
        "fr" | "de" | "it" | "es" | "nl" | "be" | "se" | "no" | "dk" | "pl" => 1.0,
        "tr" | "ru" => 3.0,
        "au" => 10.0,
        "nz" => 12.0,
        "br" | "ar" => -3.0,
        "mx" => -6.0,
        "ca" | "us" => -5.0,
        _ => 0.0,
    }
}

pub fn release_timestamp(date: &str, origin_country: Option<&str>) -> Option<String> {
    let day = NaiveDate::parse_from_str(date.get(..10)?, "%Y-%m-%d").ok()?;
    let local_midnight = Utc.from_utc_datetime(&day.and_hms_opt(0, 0, 0)?);
    let offset = origin_country.map_or(0.0, utc_offset_hours);
    let utc = local_midnight - Duration::minutes((offset * 60.0) as i64);
    Some(utc.to_rfc3339_opts(chrono::SecondsFormat::Millis, true))
}

pub fn is_available(released: Option<&str>, now: DateTime<Utc>) -> bool {
    released
        .and_then(|r| DateTime::parse_from_rfc3339(r).ok())
        .is_some_and(|r| r.with_timezone(&Utc) <= now)
}

fn text<'a>(v: &'a Value, key: &str) -> &'a str {
    v.get(key).and_then(Value::as_str).unwrap_or("")
}

fn images(v: &Value, key: &str) -> Vec<ImageCandidate> {
    v.get("images")
        .and_then(|i| i.get(key))
        .and_then(Value::as_array)
        .map(|list| {
            list.iter()
                .map(|i| ImageCandidate {
                    file_path: text(i, "file_path").to_string(),
                    language: i.get("iso_639_1").and_then(Value::as_str).map(String::from),
                    country: i
                        .get("iso_3166_1")
                        .and_then(Value::as_str)
                        .map(String::from),
                    vote_average: i.get("vote_average").and_then(Value::as_f64).unwrap_or(0.0),
                    width: i.get("width").and_then(Value::as_u64).unwrap_or(0) as u32,
                })
                .collect()
        })
        .unwrap_or_default()
}

fn translations(v: &Value, title_key: &str) -> Vec<Translation> {
    v.get("translations")
        .and_then(|t| t.get("translations"))
        .and_then(Value::as_array)
        .map(|list| {
            list.iter()
                .map(|t| Translation {
                    language: text(t, "iso_639_1").to_string(),
                    country: text(t, "iso_3166_1").to_string(),
                    title: t
                        .get("data")
                        .map(|d| text(d, title_key).to_string())
                        .unwrap_or_default(),
                })
                .collect()
        })
        .unwrap_or_default()
}

pub fn pick_trailers(detail: &Value, language: &str) -> Vec<Trailer> {
    let short = language.split('-').next().unwrap_or(language);
    let all: Vec<Trailer> = detail
        .get("videos")
        .and_then(|v| v.get("results"))
        .and_then(Value::as_array)
        .map(|list| {
            list.iter()
                .filter(|v| text(v, "site") == "YouTube" && text(v, "type") == "Trailer")
                .map(|v| Trailer {
                    source: text(v, "key").to_string(),
                    kind: "Trailer".into(),
                    name: text(v, "name").to_string(),
                    yt_id: text(v, "key").to_string(),
                    lang: v.get("iso_639_1").and_then(Value::as_str).map(String::from),
                })
                .collect()
        })
        .unwrap_or_default();
    let by = |lang: &str| -> Vec<Trailer> {
        all.iter()
            .filter(|t| {
                t.lang
                    .as_deref()
                    .is_some_and(|l| l.eq_ignore_ascii_case(lang))
            })
            .cloned()
            .collect()
    };
    let own = by(short);
    if !own.is_empty() {
        return own;
    }
    let en = by("en");
    if !en.is_empty() { en } else { all }
}

fn people(detail: &Value) -> (Vec<String>, Vec<String>, Vec<CastMember>) {
    let credits = detail.get("credits");
    let crew = credits
        .and_then(|c| c.get("crew"))
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let mut directors = Vec::new();
    let mut writers = Vec::new();
    for p in &crew {
        let name = text(p, "name").to_string();
        let push = |list: &mut Vec<String>| {
            if !list.contains(&name) {
                list.push(name.clone());
            }
        };
        if text(p, "job") == "Director" {
            push(&mut directors);
        }
        if text(p, "department") == "Writing" || text(p, "job") == "Creator" {
            push(&mut writers);
        }
    }
    let cast = credits
        .and_then(|c| c.get("cast"))
        .and_then(Value::as_array)
        .map(|list| {
            list.iter()
                .map(|p| CastMember {
                    name: text(p, "name").to_string(),
                    character: text(p, "character").to_string(),
                    photo: p
                        .get("profile_path")
                        .and_then(Value::as_str)
                        .map(art::profile_url),
                    tmdb_id: p.get("id").and_then(Value::as_u64),
                })
                .collect()
        })
        .unwrap_or_default();
    (directors, writers, cast)
}

pub struct BuildOptions {
    pub language: String,
    pub art: ArtPolicy,
    pub requested_id: String,
}

pub fn build_meta(kind: &str, detail: &Value, opts: &BuildOptions) -> Meta {
    let is_movie = kind == "movie";
    let title_key = if is_movie { "title" } else { "name" };
    let original_title = text(
        detail,
        if is_movie {
            "original_title"
        } else {
            "original_name"
        },
    );
    let original_language = text(detail, "original_language");
    let name = art::translate_title(
        &translations(detail, title_key),
        &opts.language,
        original_language,
        original_title,
    )
    .or_else(|| Some(text(detail, title_key).to_string()).filter(|t| !t.is_empty()))
    .unwrap_or_default();

    let imdb = detail
        .get("external_ids")
        .and_then(|e| e.get("imdb_id"))
        .and_then(Value::as_str)
        .or_else(|| detail.get("imdb_id").and_then(Value::as_str))
        .filter(|s| !s.is_empty())
        .map(String::from);

    let first = text(
        detail,
        if is_movie {
            "release_date"
        } else {
            "first_air_date"
        },
    );
    let last = text(detail, "last_air_date");
    let origin = detail
        .get("origin_country")
        .and_then(Value::as_array)
        .and_then(|a| a.first())
        .and_then(Value::as_str)
        .or_else(|| {
            detail
                .get("production_countries")
                .and_then(Value::as_array)
                .and_then(|a| a.first())
                .and_then(|c| c.get("iso_3166_1"))
                .and_then(Value::as_str)
        });
    let (directors, writers, cast) = people(detail);
    let runtime_minutes = if is_movie {
        detail.get("runtime").and_then(Value::as_u64)
    } else {
        detail
            .get("episode_run_time")
            .and_then(Value::as_array)
            .and_then(|a| a.first())
            .and_then(Value::as_u64)
    }
    .filter(|m| *m > 0)
    .map(|m| m as u32);

    let mut policy = opts.art.clone();
    policy.original_language = Some(original_language.to_string());
    let pick = |key: &str, size: &str| -> Option<String> {
        art::select_image(&images(detail, key), &policy)
            .map(|c| art::tmdb_image_url(size, &c.file_path))
    };
    let poster = pick("posters", "w600_and_h900_bestv2").or_else(|| {
        detail
            .get("poster_path")
            .and_then(Value::as_str)
            .map(|p| art::tmdb_image_url("w600_and_h900_bestv2", p))
    });
    let background = pick("backdrops", "original").or_else(|| {
        detail
            .get("backdrop_path")
            .and_then(Value::as_str)
            .map(|p| art::tmdb_image_url("original", p))
    });

    let id = imdb.clone().unwrap_or_else(|| opts.requested_id.clone());
    let tmdb_id = detail.get("id").and_then(Value::as_u64);
    let fallback_id = tmdb_id.map(|n| n.to_string()).unwrap_or_default();

    Meta {
        id,
        kind: if is_movie { "movie" } else { "series" }.into(),
        name: name.clone(),
        description: text(detail, "overview").to_string(),
        imdb_id: imdb.clone(),
        slug: slug(
            if is_movie { "movie" } else { "series" },
            &name,
            imdb.as_deref(),
            &fallback_id,
        ),
        genres: detail
            .get("genres")
            .and_then(Value::as_array)
            .map(|g| g.iter().map(|x| text(x, "name").to_string()).collect())
            .unwrap_or_default(),
        director: directors.clone(),
        writer: writers.clone(),
        year: first.get(..4).unwrap_or("").to_string(),
        released: release_timestamp(first, origin),
        release_info: release_info(
            if is_movie { "movie" } else { "series" },
            first,
            last,
            text(detail, "status"),
        ),
        runtime: runtime_minutes.map(format_runtime).unwrap_or_default(),
        country: detail
            .get("production_countries")
            .and_then(Value::as_array)
            .map(|c| {
                c.iter()
                    .map(|x| text(x, "name"))
                    .collect::<Vec<_>>()
                    .join(", ")
            })
            .unwrap_or_default(),
        status: (!is_movie).then(|| text(detail, "status").to_string()),
        imdb_rating: "N/A".into(),
        poster,
        background,
        trailers: pick_trailers(detail, &opts.language),
        videos: (!is_movie).then(Vec::new),
        behavior_hints: BehaviorHints {
            default_video_id: is_movie.then(|| imdb.clone()).flatten(),
            has_scheduled_videos: !is_movie,
        },
        keywords: detail
            .get("keywords")
            .and_then(|k| k.get("keywords").or_else(|| k.get("results")))
            .and_then(Value::as_array)
            .map(|k| k.iter().map(|x| text(x, "name").to_string()).collect())
            .unwrap_or_default(),
        app_extras: crate::types::AppExtras {
            cast,
            directors,
            writers,
            ..Default::default()
        },
        meta_provider: Some("tmdb".into()),
        p_tmdb: tmdb_id,
        p_imdb: imdb,
        ..Default::default()
    }
}

pub fn season_is_valid(season: &Value) -> bool {
    season
        .get("season_number")
        .and_then(Value::as_u64)
        .unwrap_or(0)
        > 0
        && !text(season, "air_date").is_empty()
}

pub fn build_videos(
    show_id: &str,
    seasons: &[Value],
    show_background: Option<&str>,
    origin: Option<&str>,
    now: DateTime<Utc>,
) -> Vec<Video> {
    let mut out = Vec::new();
    for season in seasons {
        let number = season
            .get("season_number")
            .and_then(Value::as_u64)
            .unwrap_or(0) as u32;
        let poster = season
            .get("poster_path")
            .and_then(Value::as_str)
            .map(|p| art::tmdb_image_url("w500", p));
        for ep in season
            .get("episodes")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            let episode = ep
                .get("episode_number")
                .and_then(Value::as_u64)
                .unwrap_or(0) as u32;
            let released = release_timestamp(text(ep, "air_date"), origin);
            let available = is_available(released.as_deref(), now);
            let still = ep
                .get("still_path")
                .and_then(Value::as_str)
                .map(|p| art::tmdb_image_url("w500", p));
            let thumbnail = still.or_else(|| {
                if available {
                    show_background.map(String::from)
                } else {
                    show_background.map(String::from).or(poster.clone())
                }
            });
            let title = Some(text(ep, "name")).filter(|t| !t.trim().is_empty());
            out.push(Video {
                id: crate::ids::episode_id(show_id, number, episode),
                title: title.map_or_else(|| format!("Episode {episode}"), String::from),
                season: number,
                episode,
                released,
                available,
                overview: text(ep, "overview").to_string(),
                thumbnail,
                runtime: ep
                    .get("runtime")
                    .and_then(Value::as_u64)
                    .filter(|m| *m > 0)
                    .map(|m| format_runtime(m as u32)),
                ..Default::default()
            });
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn opts() -> BuildOptions {
        BuildOptions {
            language: "en-US".into(),
            art: ArtPolicy::default(),
            requested_id: "tmdb:603".into(),
        }
    }

    #[test]
    fn detail_url_requests_textless_images() {
        let url = detail_url("movie", 603, "fr-FR", "");
        assert!(url.contains("include_image_language=fr,en,null"));
        assert!(url.contains("append_to_response=videos,credits"));
    }

    #[test]
    fn seasons_are_requested_in_batches_of_twenty() {
        let seasons: Vec<u32> = (1..=45).collect();
        let batches = season_append_batches(&seasons);
        assert_eq!(batches.len(), 3);
        assert_eq!(batches[0].matches("season/").count(), 20);
        assert_eq!(batches[2].matches("season/").count(), 5);
    }

    #[test]
    fn client_errors_are_not_retried() {
        assert!(!should_retry(404));
        assert!(should_retry(503));
    }

    #[test]
    fn runtime_strings_follow_the_catalog_style() {
        assert_eq!(format_runtime(135), "2h 15min");
        assert_eq!(format_runtime(45), "45min");
        assert_eq!(format_runtime(120), "2h");
    }

    #[test]
    fn release_info_marks_ongoing_and_finished_shows() {
        assert_eq!(
            release_info("series", "2019-03-01", "", "Returning Series"),
            "2019-"
        );
        assert_eq!(
            release_info("series", "2010-01-01", "2014-05-01", "Ended"),
            "2010-2014"
        );
        assert_eq!(
            release_info("series", "2010-01-01", "2010-12-01", "Ended"),
            "2010"
        );
    }

    #[test]
    fn origin_country_shifts_the_release_midnight() {
        let ts = release_timestamp("2024-04-07", Some("JP")).unwrap();
        assert_eq!(ts, "2024-04-06T15:00:00.000Z");
        assert_eq!(
            release_timestamp("2024-04-07", None).unwrap(),
            "2024-04-07T00:00:00.000Z"
        );
    }

    #[test]
    fn movie_meta_uses_imdb_id_and_translated_title() {
        let detail = json!({
            "id": 603, "title": "The Matrix", "original_title": "The Matrix", "original_language": "en",
            "overview": "A hacker.", "release_date": "1999-03-30", "runtime": 136,
            "genres": [{"id": 28, "name": "Action"}],
            "production_countries": [{"iso_3166_1": "US", "name": "United States of America"}],
            "external_ids": {"imdb_id": "tt0133093"},
            "credits": {"crew": [{"name": "Lana Wachowski", "job": "Director", "department": "Directing"}], "cast": []},
            "videos": {"results": [{"site": "YouTube", "type": "Trailer", "key": "abc", "name": "T", "iso_639_1": "en"}]}
        });
        let meta = build_meta("movie", &detail, &opts());
        assert_eq!(meta.id, "tt0133093");
        assert_eq!(meta.runtime, "2h 16min");
        assert_eq!(meta.slug, "movie/the-matrix-0133093");
        assert_eq!(meta.director, vec!["Lana Wachowski"]);
        assert_eq!(
            meta.behavior_hints.default_video_id.as_deref(),
            Some("tt0133093")
        );
        assert_eq!(meta.trailers.len(), 1);
        assert!(meta.videos.is_none());
    }

    #[test]
    fn series_without_imdb_keeps_the_requested_id() {
        let detail = json!({"id": 1, "name": "Obscure", "original_name": "Obscure", "original_language": "en",
            "first_air_date": "2020-01-01", "status": "Ended", "last_air_date": "2021-01-01"});
        let meta = build_meta("tv", &detail, &opts());
        assert_eq!(meta.id, "tmdb:603");
        assert_eq!(meta.release_info, "2020-2021");
        assert_eq!(meta.videos, Some(Vec::new()));
        assert!(meta.behavior_hints.has_scheduled_videos);
    }

    #[test]
    fn episodes_are_unavailable_until_they_air() {
        let now = Utc.with_ymd_and_hms(2024, 1, 10, 0, 0, 0).unwrap();
        let seasons = vec![
            json!({"season_number": 1, "air_date": "2024-01-01", "episodes": [
                {"episode_number": 1, "name": "Pilot", "air_date": "2024-01-01"},
                {"episode_number": 2, "name": "", "air_date": "2024-02-01"}
            ]}),
        ];
        let videos = build_videos("tt1", &seasons, Some("/bg"), None, now);
        assert!(videos[0].available);
        assert!(!videos[1].available);
        assert_eq!(videos[1].title, "Episode 2");
        assert_eq!(videos[1].id, "tt1:1:2");
    }

    #[test]
    fn seasons_without_air_date_are_invalid() {
        assert!(season_is_valid(
            &json!({"season_number": 1, "air_date": "2020-01-01"})
        ));
        assert!(!season_is_valid(&json!({"season_number": 2})));
        assert!(!season_is_valid(
            &json!({"season_number": 0, "air_date": "2020-01-01"})
        ));
    }
}
