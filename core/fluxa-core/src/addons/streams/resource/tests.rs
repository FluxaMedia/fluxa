use super::{
    normalize_addon_subtitles_json 
    
};
use serde_json::Value;





#[test]
fn subtitle_payload_is_resolved_without_reordering_valid_entries() {
    let subtitles = normalize_addon_subtitles_json(
        r#"[
            {"id":"one","url":"/subs/one.vtt","lang":"en","attributes":{"languages":[]}},
            {"id":"drop","attributes":{}},
            {"id":"two","attributes":{"url":"two.srt"}}
        ]"#,
        "https://addon.example/subtitles/movie/tt1.json",
    );
    let subtitles: Value = serde_json::from_str(&subtitles).expect("subtitles");

    assert_eq!(subtitles.as_array().map(Vec::len), Some(2));
    assert_eq!(subtitles[0]["id"].as_str(), Some("one"));
    assert_eq!(
        subtitles[0]["url"].as_str(),
        Some("https://addon.example/subs/one.vtt")
    );
    assert_eq!(
        subtitles[0]["attributes"]["languages"][0].as_str(),
        Some("en")
    );
    assert_eq!(subtitles[1]["id"].as_str(), Some("two"));
    assert_eq!(
        subtitles[1]["url"].as_str(),
        Some("https://addon.example/subtitles/movie/two.srt")
    );
}
