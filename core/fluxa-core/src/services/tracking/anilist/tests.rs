use super::anilist_entries_to_sync;
use serde_json::{Value, json};

fn entry(status: &str, progress: i64, episodes: Option<i64>) -> Value {
    json!({
        "status": status,
        "progress": progress,
        "updatedAt": 1_700_000_000,
        "media": {"id": 7, "episodes": episodes, "title": {"english": "Show"}}
    })
}

fn watching(entries: &[Value]) -> Vec<Value> {
    anilist_entries_to_sync(entries, 0, None, false)["watching"]
        .as_array()
        .cloned()
        .unwrap_or_default()
}

#[test]
fn continue_watching_points_at_the_next_episode_without_a_progress_bar() {
    let items = watching(&[entry("CURRENT", 8, Some(12))]);
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["lastVideoId"], "anilist:7:1:9");
    assert_eq!(items[0]["lastEpisodeNumber"], 9);
    assert_eq!(items[0]["continueWatchingBadge"], "upNext");
    assert!(items[0].get("timeOffset").is_none());
    assert!(items[0].get("duration").is_none());
}

#[test]
fn finished_series_stays_out_of_continue_watching() {
    assert!(watching(&[entry("CURRENT", 12, Some(12))]).is_empty());
}

#[test]
fn unknown_episode_total_still_offers_the_next_episode() {
    let items = watching(&[entry("CURRENT", 1000, None)]);
    assert_eq!(items[0]["lastEpisodeNumber"], 1001);
}

#[test]
fn caught_up_airing_show_is_marked_upcoming_with_its_air_time() {
    let mut caught_up = entry("CURRENT", 8, None);
    caught_up["media"]["nextAiringEpisode"] = json!({"episode": 9, "airingAt": 1_800_000_000});
    let items = watching(&[caught_up]);
    assert_eq!(items[0]["lastEpisodeNumber"], 9);
    assert_eq!(items[0]["upcoming"], true);
    assert_eq!(items[0]["airsAt"], 1_800_000_000_000_i64);

    let mut behind = entry("CURRENT", 5, None);
    behind["media"]["nextAiringEpisode"] = json!({"episode": 9, "airingAt": 1_800_000_000});
    let items = watching(&[behind]);
    assert_eq!(items[0]["lastEpisodeNumber"], 6);
    assert!(items[0].get("upcoming").is_none());
}
