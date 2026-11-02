use chrono::{DateTime, Utc};
use serde_json::Value;

use crate::tmdb::{format_runtime, is_available, release_timestamp};
use crate::types::Video;

pub mod anidb;
pub mod anilist;
pub mod fanart;
pub mod gemini;
pub mod imdb;
pub mod jikan;
pub mod kitsu;
pub mod mal_list;
pub mod mdblist;
pub mod rpdb;
pub mod simkl;
pub mod trakt;
pub mod tvdb;
pub mod tvmaze;

pub(crate) fn text<'a>(v: &'a Value, key: &str) -> &'a str {
    v.get(key).and_then(Value::as_str).unwrap_or("")
}

pub(crate) fn nonempty(s: &str) -> Option<String> {
    let s = s.trim();
    (!s.is_empty()).then(|| s.to_string())
}

pub(crate) fn strip_html(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    let mut in_tag = false;
    let mut tag = String::new();
    for c in raw.chars() {
        match c {
            '<' => {
                in_tag = true;
                tag.clear();
            }
            '>' if in_tag => {
                in_tag = false;
                if tag.starts_with("br") || tag.starts_with("/p") {
                    out.push(' ');
                }
            }
            _ if in_tag => tag.push(c.to_ascii_lowercase()),
            _ => out.push(c),
        }
    }
    let decoded = out
        .replace("&amp;", "&")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&#039;", "'")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&nbsp;", " ");
    decoded.split_whitespace().collect::<Vec<_>>().join(" ")
}

pub(crate) fn year_of(date: &str) -> String {
    date.get(..4).unwrap_or("").to_string()
}

pub(crate) fn rating(score_out_of_ten: f64) -> String {
    if score_out_of_ten > 0.0 {
        format!("{score_out_of_ten:.1}")
    } else {
        "N/A".into()
    }
}

pub(crate) struct Episode<'a> {
    pub show_id: &'a str,
    pub season: u32,
    pub number: u32,
    pub anime: bool,
    pub title: &'a str,
    pub overview: &'a str,
    pub thumbnail: Option<&'a str>,
    pub date: &'a str,
    pub minutes: Option<u32>,
}

pub(crate) fn video(ep: Episode, now: DateTime<Utc>) -> Video {
    let released = release_timestamp(ep.date, None);
    Video {
        id: if ep.anime {
            crate::ids::anime_episode_id(ep.show_id, ep.number)
        } else {
            crate::ids::episode_id(ep.show_id, ep.season, ep.number)
        },
        title: nonempty(ep.title).unwrap_or_else(|| format!("Episode {}", ep.number)),
        season: ep.season,
        episode: ep.number,
        available: is_available(released.as_deref(), now),
        released,
        overview: ep.overview.trim().to_string(),
        thumbnail: ep.thumbnail.and_then(nonempty),
        runtime: ep.minutes.filter(|m| *m > 0).map(format_runtime),
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn html_is_stripped_and_entities_decoded() {
        assert_eq!(
            strip_html("<p>Fish &amp; chips<br>Second&nbsp;line</p>"),
            "Fish & chips Second line"
        );
    }

    #[test]
    fn missing_scores_read_as_not_available() {
        assert_eq!(rating(0.0), "N/A");
        assert_eq!(rating(7.84), "7.8");
    }
}
