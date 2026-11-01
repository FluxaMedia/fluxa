use chrono::{DateTime, Utc};
use roxmltree::{Document, Node};

use super::{Episode, nonempty, rating, strip_html, video, year_of};
use crate::tmdb::{release_info, release_timestamp, slug};
use crate::types::{BehaviorHints, Meta};

const API_BASE: &str = "http://api.anidb.net:9001/httpapi";
const IMAGE_BASE: &str = "https://cdn.anidb.net/images/main";

pub fn request_url(client: &str, id: u64) -> Option<String> {
    let client = client.trim();
    if client.is_empty() {
        return None;
    }
    Some(format!(
        "{API_BASE}?request=anime&client={client}&clientver=1&protover=1&aid={id}"
    ))
}

fn child<'a>(node: Node<'a, 'a>, name: &str) -> Option<Node<'a, 'a>> {
    node.children().find(|c| c.has_tag_name(name))
}

fn child_text<'a>(node: Node<'a, 'a>, name: &str) -> &'a str {
    child(node, name).and_then(|c| c.text()).unwrap_or("")
}

fn title_of(node: Node, kind: &str, lang: Option<&str>) -> Option<String> {
    let titles = child(node, "titles")?;
    titles
        .children()
        .filter(|t| t.has_tag_name("title"))
        .find(|t| {
            t.attribute("type") == Some(kind)
                && lang.is_none_or(|l| t.attributes().any(|a| a.name() == "lang" && a.value() == l))
        })
        .and_then(|t| t.text())
        .and_then(nonempty)
}

fn status(end: &str) -> &'static str {
    if end.is_empty() {
        "Returning Series"
    } else {
        "Ended"
    }
}

pub fn build_meta(xml: &str, requested_id: &str, now: DateTime<Utc>) -> Option<Meta> {
    let doc = Document::parse(xml).ok()?;
    let root = doc.root_element();
    if !root.has_tag_name("anime") {
        return None;
    }
    let anidb: u64 = root.attribute("id")?.parse().ok()?;
    let name = title_of(root, "official", Some("en")).or_else(|| title_of(root, "main", None))?;
    let kind_raw = child_text(root, "type");
    let movie = kind_raw.eq_ignore_ascii_case("movie");
    let kind = if movie { "movie" } else { "series" };
    let first = child_text(root, "startdate");
    let last = child_text(root, "enddate");
    let score = child(root, "ratings")
        .map(|r| {
            ["permanent", "temporary"]
                .iter()
                .find_map(|tag| child_text(r, tag).parse::<f64>().ok())
                .unwrap_or(0.0)
        })
        .unwrap_or(0.0);
    let genres = child(root, "tags")
        .map(|tags| {
            tags.children()
                .filter(|t| t.has_tag_name("tag"))
                .filter_map(|t| child(t, "name").and_then(|n| n.text()))
                .take(8)
                .map(String::from)
                .collect()
        })
        .unwrap_or_default();
    let id = requested_id.to_string();
    let videos = child(root, "episodes").map(|eps| {
        let mut out: Vec<_> = eps
            .children()
            .filter(|e| e.has_tag_name("episode"))
            .filter_map(|e| {
                let epno = child(e, "epno")?;
                if epno.attribute("type") != Some("1") {
                    return None;
                }
                let number: u32 = epno.text()?.trim().parse().ok()?;
                let title = e
                    .children()
                    .filter(|t| t.has_tag_name("title"))
                    .find(|t| {
                        t.attributes()
                            .any(|a| a.name() == "lang" && a.value() == "en")
                    })
                    .and_then(|t| t.text())
                    .unwrap_or("");
                let minutes = child_text(e, "length").parse().ok();
                Some(video(
                    Episode {
                        show_id: &id,
                        season: 1,
                        number,
                        anime: true,
                        title,
                        overview: "",
                        thumbnail: None,
                        date: child_text(e, "airdate"),
                        minutes,
                    },
                    now,
                ))
            })
            .collect();
        out.sort_by_key(|v| v.episode);
        out
    });
    let runtime = videos
        .as_ref()
        .and_then(|v| v.first())
        .and_then(|v| v.runtime.clone())
        .unwrap_or_default();
    Some(Meta {
        id: id.clone(),
        kind: kind.into(),
        name: name.clone(),
        description: strip_html(child_text(root, "description")),
        slug: slug(kind, &name, None, &anidb.to_string()),
        genres,
        year: year_of(first),
        released: release_timestamp(first, Some("jp")),
        release_info: release_info(kind, first, last, status(last)),
        runtime,
        status: (!movie).then(|| status(last).to_string()),
        imdb_rating: rating(score),
        poster: nonempty(child_text(root, "picture")).map(|p| format!("{IMAGE_BASE}/{p}")),
        videos: if movie { None } else { videos },
        behavior_hints: BehaviorHints {
            default_video_id: movie.then(|| id.clone()),
            has_scheduled_videos: !movie,
        },
        meta_provider: Some("anidb".into()),
        p_anidb: Some(anidb),
        ..Default::default()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const XML: &str = r#"<anime id="7" restricted="false">
        <type>TV Series</type><episodecount>2</episodecount>
        <startdate>2025-04-05</startdate><enddate>2025-06-21</enddate>
        <titles>
          <title xml:lang="x-jat" type="main">Example Romaji</title>
          <title xml:lang="en" type="official">Example Show</title>
        </titles>
        <description>A tale of sand.</description>
        <ratings><permanent count="10">8.3</permanent></ratings>
        <picture>1234.jpg</picture>
        <tags><tag id="1"><name>drama</name></tag></tags>
        <episodes>
          <episode id="1"><epno type="1">1</epno><length>24</length><airdate>2025-04-05</airdate><title xml:lang="en">First</title></episode>
          <episode id="2"><epno type="1">2</epno><length>24</length><airdate>2025-04-12</airdate><title xml:lang="en">Second</title></episode>
          <episode id="3"><epno type="2">S1</epno><title xml:lang="en">Special</title></episode>
        </episodes>
    </anime>"#;

    fn now() -> DateTime<Utc> {
        "2026-01-01T00:00:00Z".parse().unwrap()
    }

    #[test]
    fn requests_need_a_client_name() {
        assert!(request_url("  ", 7).is_none());
        assert!(request_url("example", 7).unwrap().contains("aid=7"));
    }

    #[test]
    fn english_title_wins_and_specials_are_dropped() {
        let meta = build_meta(XML, "anidb:7", now()).unwrap();
        assert_eq!(meta.name, "Example Show");
        let videos = meta.videos.unwrap();
        assert_eq!(videos.len(), 2);
        assert_eq!(videos[1].id, "anidb:7:2");
        assert_eq!(videos[1].title, "Second");
        assert_eq!(
            meta.poster.as_deref(),
            Some("https://cdn.anidb.net/images/main/1234.jpg")
        );
        assert_eq!(meta.imdb_rating, "8.3");
    }

    #[test]
    fn error_documents_are_rejected() {
        assert!(build_meta("<error code=\"500\">banned</error>", "anidb:7", now()).is_none());
    }
}
