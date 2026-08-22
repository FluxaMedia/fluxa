use serde::Deserialize;
use serde_json::Value;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Chapter {
    title: String,
    start_time: i64,
}

pub(crate) fn should_play_next_episode(has_next_episode: bool, auto_play: bool) -> bool {
    has_next_episode && auto_play
}

pub(crate) fn chapter_skip_segments_json(chapters_json: &str) -> String {
    let chapters: Vec<Chapter> = match serde_json::from_str(chapters_json) {
        Ok(chapters) => chapters,
        Err(_) => return "[]".to_string(),
    };
    let segments = chapters
        .iter()
        .enumerate()
        .filter_map(|(index, chapter)| {
            let segment_type = classify_chapter(&chapter.title)?;
            let end_time = chapters.get(index + 1)?.start_time;
            (end_time > chapter.start_time).then(|| {
                serde_json::json!({
                    "type": segment_type,
                    "startTime": chapter.start_time,
                    "endTime": end_time,
                })
            })
        })
        .collect::<Vec<Value>>();
    serde_json::to_string(&segments).unwrap_or_else(|_| "[]".to_string())
}

fn classify_chapter(title: &str) -> Option<&'static str> {
    let normalized = title.trim().to_lowercase();
    match normalized.as_str() {
        "op" | "opening" | "intro" | "introduction" | "op sequence" | "mixed-intro"
        | "opening sequence" | "opening theme" => return Some("intro"),
        "ed" | "ending" | "outro" | "credits" | "end credits" | "closing" | "ending theme"
        | "ending sequence" => return Some("outro"),
        "recap" | "previously" | "previously on" | "cold open" => return Some("recap"),
        _ => {}
    }
    if normalized.starts_with("op ")
        || normalized.starts_with("opening ")
        || normalized.contains("intro")
        || normalized.contains("opening")
    {
        return Some("intro");
    }
    if normalized.starts_with("ed ")
        || normalized.starts_with("ending ")
        || normalized.contains("ending")
        || normalized.contains("outro")
        || normalized.contains("credits")
    {
        return Some("outro");
    }
    if normalized.contains("recap") || normalized.contains("previously") {
        return Some("recap");
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn next_episode_requires_both_a_successor_and_autoplay() {
        assert!(should_play_next_episode(true, true));
        assert!(!should_play_next_episode(true, false));
        assert!(!should_play_next_episode(false, true));
    }

    #[test]
    fn derives_skip_segments_from_named_chapters() {
        let result = chapter_skip_segments_json(
            r#"[{"title":"Opening Theme","startTime":0},{"title":"Episode","startTime":90000},{"title":"Credits","startTime":120000},{"title":"End","startTime":140000}]"#,
        );
        let value: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(value[0]["type"], "intro");
        assert_eq!(value[0]["endTime"], 90000);
        assert_eq!(value[1]["type"], "outro");
    }
}
