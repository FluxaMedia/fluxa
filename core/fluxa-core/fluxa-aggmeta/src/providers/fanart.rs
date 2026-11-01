use serde_json::Value;

use super::text;
use crate::art::ImageCandidate;

const API_BASE: &str = "https://webservice.fanart.tv/v3";

#[derive(Debug, Default, PartialEq)]
pub struct Artwork {
    pub posters: Vec<ImageCandidate>,
    pub backgrounds: Vec<ImageCandidate>,
    pub logos: Vec<ImageCandidate>,
}

pub fn url(movie: bool, id: &str, key: &str) -> String {
    let section = if movie { "movies" } else { "tv" };
    format!("{API_BASE}/{section}/{id}?api_key={key}")
}

fn candidates(doc: &Value, names: &[&str]) -> Vec<ImageCandidate> {
    names
        .iter()
        .filter_map(|name| doc.get(name).and_then(Value::as_array))
        .flatten()
        .filter_map(|item| {
            let file = text(item, "url");
            if file.is_empty() {
                return None;
            }
            let language = text(item, "lang");
            Some(ImageCandidate {
                file_path: file.to_string(),
                language: (!language.is_empty() && language != "00").then(|| language.to_string()),
                vote_average: text(item, "likes").parse().unwrap_or(0.0),
                ..Default::default()
            })
        })
        .collect()
}

pub fn artwork(doc: &Value, movie: bool) -> Artwork {
    if movie {
        Artwork {
            posters: candidates(doc, &["movieposter"]),
            backgrounds: candidates(doc, &["moviebackground"]),
            logos: candidates(doc, &["hdmovielogo"]),
        }
    } else {
        Artwork {
            posters: candidates(doc, &["tvposter"]),
            backgrounds: candidates(doc, &["showbackground"]),
            logos: candidates(doc, &["hdtvlogo"]),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::art::{ArtPolicy, select_image};
    use serde_json::json;

    #[test]
    fn show_sections_are_read_and_textless_art_has_no_language() {
        let doc = json!({
            "tvposter": [
                {"url": "https://img.example/a.jpg", "lang": "en", "likes": "3"},
                {"url": "https://img.example/b.jpg", "lang": "00", "likes": "9"}
            ],
            "hdtvlogo": [{"url": "https://img.example/l.png", "lang": "en", "likes": "1"}]
        });
        let art = artwork(&doc, false);
        assert_eq!(art.posters.len(), 2);
        assert_eq!(art.posters[1].language, None);
        assert_eq!(art.logos[0].file_path, "https://img.example/l.png");
        assert!(art.backgrounds.is_empty());
    }

    #[test]
    fn most_liked_art_in_the_user_language_wins() {
        let doc = json!({"movieposter": [
            {"url": "https://img.example/en-low.jpg", "lang": "en", "likes": "2"},
            {"url": "https://img.example/en-high.jpg", "lang": "en", "likes": "8"},
            {"url": "https://img.example/de.jpg", "lang": "de", "likes": "99"}
        ]});
        let art = artwork(&doc, true);
        let pick = select_image(&art.posters, &ArtPolicy::default()).unwrap();
        assert_eq!(pick.file_path, "https://img.example/en-high.jpg");
    }
}
