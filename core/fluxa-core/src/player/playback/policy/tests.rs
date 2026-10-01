use super::*;
use serde_json::Value;


#[test]
fn stream_subtitles_result_has_one_shared_empty_and_populated_shape() {
    let populated: Value = serde_json::from_str(
        &stream_subtitles_result_json(
            r#"{"subtitles":[{"url":"https://sub.example/en.vtt","lang":"en"}]}"#,
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(populated["subtitles"][0]["lang"], "en");

    let empty: Value = serde_json::from_str(&stream_subtitles_result_json("{}").unwrap()).unwrap();
    assert_eq!(empty["subtitles"], serde_json::json!([]));
}







