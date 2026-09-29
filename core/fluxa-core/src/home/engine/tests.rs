use super::add_continue_watching_episode_labels;
use serde_json::json;

#[test]
fn continue_watching_episode_label_includes_locator_and_episode_name() {
    let items = add_continue_watching_episode_labels(json!([{
        "lastEpisodeName": "The Day I Become a Shinigami",
        "lastVideoId": "show-id:1:1",
    }]));

    assert_eq!(
        items[0]["episodeLabel"],
        "S1:E1 The Day I Become a Shinigami"
    );
}
