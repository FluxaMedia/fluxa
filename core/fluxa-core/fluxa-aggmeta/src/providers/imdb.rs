use serde_json::Value;

use super::{nonempty, text};
use crate::tmdb::slug;
use crate::types::{AppExtras, BehaviorHints, CastMember, Meta};

pub fn suggestion_url(imdb: &str) -> String {
    let first = imdb.chars().next().unwrap_or('t');
    format!("https://v3.sg.media-imdb.com/suggestion/{first}/{imdb}.json")
}

pub fn build_meta(response: &Value, imdb: &str, movie: bool) -> Option<Meta> {
    let entry = response
        .get("d")
        .and_then(Value::as_array)?
        .iter()
        .find(|e| text(e, "id") == imdb)?;
    let name = nonempty(text(entry, "l"))?;
    let kind = if movie { "movie" } else { "series" };
    let year = entry
        .get("y")
        .and_then(Value::as_u64)
        .map(|y| y.to_string())
        .unwrap_or_default();
    let cast = text(entry, "s")
        .split(", ")
        .filter(|n| !n.is_empty())
        .map(|n| CastMember {
            name: n.to_string(),
            ..Default::default()
        })
        .collect();
    Some(Meta {
        id: imdb.to_string(),
        kind: kind.into(),
        name: name.clone(),
        imdb_id: Some(imdb.to_string()),
        slug: slug(kind, &name, Some(imdb), ""),
        release_info: year.clone(),
        year,
        imdb_rating: "N/A".into(),
        poster: entry
            .pointer("/i/imageUrl")
            .and_then(Value::as_str)
            .and_then(nonempty),
        videos: (!movie).then(Vec::new),
        behavior_hints: BehaviorHints {
            default_video_id: movie.then(|| imdb.to_string()),
            has_scheduled_videos: !movie,
        },
        app_extras: AppExtras {
            cast,
            ..Default::default()
        },
        meta_provider: Some("imdb".into()),
        p_imdb: Some(imdb.to_string()),
        ..Default::default()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn only_the_matching_suggestion_is_used() {
        let response = json!({"d": [
            {"id": "tt2", "l": "Other"},
            {"id": "tt1", "l": "Example Film", "y": 2001, "s": "A Person, B Person", "i": {"imageUrl": "https://img.example/p.jpg"}}
        ]});
        let meta = build_meta(&response, "tt1", true).unwrap();
        assert_eq!(meta.name, "Example Film");
        assert_eq!(meta.year, "2001");
        assert_eq!(meta.app_extras.cast.len(), 2);
        assert!(build_meta(&response, "tt9", true).is_none());
    }

    #[test]
    fn suggestion_url_uses_the_first_letter_bucket() {
        assert!(suggestion_url("tt1").ends_with("/suggestion/t/tt1.json"));
    }
}
