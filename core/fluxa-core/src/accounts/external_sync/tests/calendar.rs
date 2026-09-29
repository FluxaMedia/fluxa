use super::super::*;
use serde_json::{Value, json};

#[test]
fn trakt_calendar_items_include_episode_identity() {
    let request = json!({
        "provider": "trakt",
        "shows": [{
            "first_aired": "2026-07-27T03:00:00Z",
            "show": {
                "title": "Rick and Morty",
                "ids": {"imdb": "tt2861424"},
                "images": {"poster": ["walter-r2.trakt.tv/images/shows/poster.webp"]}
            },
            "episode": {
                "season": 9,
                "number": 10,
                "title": "Episode Title",
                "images": {"screenshot": ["walter-r2.trakt.tv/images/episodes/screenshot.webp"]}
            }
        }],
        "movies": []
    });
    let result: Value =
        serde_json::from_str(&provider_calendar_items_json(&request.to_string()).unwrap()).unwrap();
    assert_eq!(result[0]["seasonNumber"], 9);
    assert_eq!(result[0]["episodeNumber"], 10);
    assert_eq!(result[0]["metaType"], "series");
    assert_eq!(
        result[0]["episodePoster"],
        "https://walter-r2.trakt.tv/images/episodes/screenshot.webp"
    );
}

#[test]
fn simkl_calendar_items_accept_number_field_variants() {
    let request = json!({
        "provider": "simkl",
        "shows": [{
            "date": "2026-07-27T03:00:00Z",
            "show": {
                "title": "Rick and Morty",
                "ids": {"imdb": "tt2861424"}
            },
            "episode": {
                "season_number": 9,
                "episode_number": 10,
                "title": "Episode Title"
            }
        }],
        "movies": []
    });
    let result: Value =
        serde_json::from_str(&provider_calendar_items_json(&request.to_string()).unwrap()).unwrap();
    assert_eq!(result[0]["seasonNumber"], 9);
    assert_eq!(result[0]["episodeNumber"], 10);
}

#[test]
fn simkl_calendar_items_accept_v2_cdn_payloads() {
    let request = json!({
        "provider": "simkl",
        "shows": {
            "calendar": [{"simkl_id": 3437, "date": "2026-07-27T04:00:00Z", "episode": {"season": 15, "episode": 10, "title": "Propane Recall"}}],
            "metadata": {"3437": {"title": "King of the Hill", "poster": "https://example.test/poster.jpg", "ids": {"imdb": "tt0118375"}}}
        },
        "movies": {"calendar": [], "metadata": {}},
        "allowedContentIds": ["tt0118375"]
    });
    let result: Value =
        serde_json::from_str(&provider_calendar_items_json(&request.to_string()).unwrap()).unwrap();
    assert_eq!(result[0]["contentId"], "tt0118375");
    assert_eq!(result[0]["seasonNumber"], 15);
    assert_eq!(result[0]["episodeNumber"], 10);
    assert_eq!(result[0]["poster"], "https://example.test/poster.jpg");
}
