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

fn save(action: &str, episode: Option<i64>, lookup: Value) -> Option<Value> {
    let args = json!({
        "stage": "save", "mediaId": 7, "episode": episode, "lookup": lookup, "token": "t"
    });
    super::anilist_scrobble(&args, action, 90.0).map(|plan| plan["body"]["variables"].clone())
}

fn entry_lookup(status: &str, progress: i64, episodes: i64) -> Value {
    json!({"data": {"Media": {
        "episodes": episodes,
        "mediaListEntry": {"status": status, "progress": progress}
    }}})
}

#[test]
fn starting_a_new_show_marks_it_watching() {
    let vars = save(
        "start",
        Some(1),
        json!({"data": {"Media": {"episodes": 12, "mediaListEntry": null}}}),
    );
    assert_eq!(vars.unwrap()["status"], "CURRENT");
}

#[test]
fn starting_a_rewatch_leaves_a_completed_entry_alone() {
    assert!(save("start", Some(1), entry_lookup("COMPLETED", 12, 12)).is_none());
}

#[test]
fn finishing_an_episode_saves_its_number() {
    let vars = save("stop", Some(5), entry_lookup("CURRENT", 4, 12)).unwrap();
    assert_eq!(vars["progress"], 5);
    assert_eq!(vars["status"], "CURRENT");
}

#[test]
fn finishing_the_last_episode_completes_the_show() {
    let vars = save("stop", Some(12), entry_lookup("CURRENT", 11, 12)).unwrap();
    assert_eq!(vars["status"], "COMPLETED");
}

#[test]
fn repeating_stays_repeating_and_progress_never_goes_back() {
    let vars = save("stop", Some(3), entry_lookup("REPEATING", 2, 12)).unwrap();
    assert_eq!(vars["status"], "REPEATING");
    assert!(save("stop", Some(3), entry_lookup("REPEATING", 5, 12)).is_none());
}

#[test]
fn progress_is_clamped_to_the_episode_total() {
    let vars = save("stop", Some(30), entry_lookup("CURRENT", 11, 12)).unwrap();
    assert_eq!(vars["progress"], 12);
}

#[test]
fn finished_movie_is_completed_once() {
    let none = json!({"data": {"Media": {"episodes": 1, "mediaListEntry": null}}});
    assert_eq!(save("stop", None, none).unwrap()["status"], "COMPLETED");
    assert!(save("stop", None, entry_lookup("COMPLETED", 1, 1)).is_none());
}

#[test]
fn lookup_asks_for_the_media_and_the_viewers_entry() {
    let args = json!({"stage": "lookup", "mediaId": 7, "token": "t"});
    let plan = super::anilist_scrobble(&args, "stop", 90.0).unwrap();
    assert_eq!(plan["body"]["variables"]["id"], 7);
    assert!(
        plan["body"]["query"]
            .as_str()
            .unwrap()
            .contains("mediaListEntry")
    );
}
