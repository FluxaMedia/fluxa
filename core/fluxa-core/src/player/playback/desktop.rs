use serde::Deserialize;
use serde_json::Value;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Chapter {
    #[serde(default)]
    title: String,
    #[serde(alias = "startMs")]
    start_time: i64,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum ChapterInput {
    List(Vec<Chapter>),
    Envelope { chapters: Vec<Chapter> },
}

pub(crate) fn should_play_next_episode(has_next_episode: bool, auto_play: bool) -> bool {
    has_next_episode && auto_play
}

pub(crate) fn chapter_skip_segments_json(chapters_json: &str, duration_ms: i64) -> String {
    let chapters = match serde_json::from_str::<ChapterInput>(chapters_json) {
        Ok(ChapterInput::List(chapters)) => chapters,
        Ok(ChapterInput::Envelope { chapters }) => chapters,
        Err(_) => return "[]".to_string(),
    };
    let segments = chapters
        .iter()
        .enumerate()
        .filter_map(|(index, chapter)| {
            let segment_type = classify_chapter(&chapter.title, index, chapters.len())?;
            let end_time = chapters
                .get(index + 1)
                .map(|next| next.start_time)
                .unwrap_or(duration_ms);
            (end_time > chapter.start_time).then(|| {
                serde_json::json!({
                    "type": segment_type,
                    "startTime": chapter.start_time,
                    "endTime": end_time,
                    "provider": "Chapters",
                })
            })
        })
        .collect::<Vec<Value>>();
    serde_json::to_string(&segments).unwrap_or_else(|_| "[]".to_string())
}

fn classify_chapter(title: &str, index: usize, chapter_count: usize) -> Option<&'static str> {
    let normalized = normalize_title(title);
    let compact = normalized.replace(' ', "");
    if matches_any(
        &normalized,
        &[
            "intro",
            "introduction",
            "opening",
            "opening credits",
            "opening theme",
            "opening song",
            "main title",
            "title sequence",
            "theme song",
        ],
    ) || numbered_variant(&normalized, &["intro", "introduction", "opening", "op"])
        || compact_numbered_variant(&compact, "op")
        || is_first_chapter(&normalized)
        || matches_any(&compact, &["オープニング"])
    {
        return Some("intro");
    }
    if matches_any(
        &normalized,
        &[
            "outro",
            "ending",
            "ending credits",
            "ending theme",
            "end credits",
            "closing credits",
            "closing theme",
            "credits",
            "staff roll",
        ],
    ) || numbered_variant(&normalized, &["ending", "ed"])
        || compact_numbered_variant(&compact, "ed")
        || matches_any(&compact, &["エンディング"])
    {
        return Some("outro");
    }
    if matches_any(
        &normalized,
        &[
            "recap",
            "episode recap",
            "previous",
            "previously",
            "previously on",
            "previous episode",
            "last time",
            "last episode",
            "story so far",
            "the story so far",
            "summary",
            "synopsis",
        ],
    ) || matches_any(&compact, &["あらすじ", "前回までのあらすじ"])
    {
        return Some("recap");
    }
    if matches_any(
        &normalized,
        &[
            "preview",
            "next",
            "next time",
            "next episode",
            "next episode preview",
            "episode preview",
            "trailer",
            "teaser",
        ],
    ) || matches_any(&compact, &["予告", "次回予告"])
    {
        return Some("preview");
    }
    if is_last_chapter(&normalized, index, chapter_count) {
        return Some("outro");
    }
    None
}

fn normalize_title(title: &str) -> String {
    title
        .to_lowercase()
        .chars()
        .map(|character| match character {
            '(' | ')' | '[' | ']' | '{' | '}' | '.' | '_' | '-' => ' ',
            character => character,
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn matches_any(value: &str, candidates: &[&str]) -> bool {
    candidates.iter().any(|candidate| *candidate == value)
}

fn numbered_variant(value: &str, bases: &[&str]) -> bool {
    bases.iter().any(|base| {
        value
            .strip_prefix(&format!("{base} "))
            .is_some_and(|suffix| {
                !suffix.is_empty() && suffix.chars().all(|character| character.is_ascii_digit())
            })
    })
}

fn compact_numbered_variant(value: &str, base: &str) -> bool {
    value.strip_prefix(base).is_some_and(|suffix| {
        !suffix.is_empty() && suffix.chars().all(|character| character.is_ascii_digit())
    })
}

fn is_first_chapter(value: &str) -> bool {
    value == "chapter 1"
        || value.starts_with("chapter 1 ")
        || value == "chapter one"
        || value.starts_with("chapter one ")
        || value == "first chapter"
        || value.starts_with("first chapter ")
}

fn is_last_chapter(value: &str, index: usize, chapter_count: usize) -> bool {
    if value == "last chapter" || value == "final chapter" {
        return true;
    }
    if index + 1 != chapter_count {
        return false;
    }
    let Some(suffix) = value.strip_prefix("chapter ") else {
        return false;
    };
    suffix
        .split_whitespace()
        .next()
        .is_some_and(|number| number.chars().all(|character| character.is_ascii_digit()))
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
            r#"{"chapters":[{"title":"Opening Theme","startMs":0},{"title":"Episode","startMs":90000},{"title":"Chapter 7","startMs":120000}]}"#,
            140000,
        );
        let value: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(value[0]["type"], "intro");
        assert_eq!(value[0]["endTime"], 90000);
        assert_eq!(value[1]["type"], "outro");
        assert_eq!(value[1]["provider"], "Chapters");
    }

    #[test]
    fn does_not_treat_an_intermediate_numbered_chapter_as_outro() {
        let result = chapter_skip_segments_json(
            r#"{"chapters":[{"title":"Chapter 1","startMs":0},{"title":"Chapter 2","startMs":60000},{"title":"Chapter 3","startMs":120000},{"title":"Chapter 4","startMs":180000},{"title":"Chapter 5","startMs":240000}]}"#,
            300000,
        );
        let value: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(value.as_array().map(Vec::len), Some(2));
        assert_eq!(value[0]["type"], "intro");
        assert_eq!(value[0]["endTime"], 60000);
        assert_eq!(value[1]["type"], "outro");
        assert_eq!(value[1]["startTime"], 240000);
    }

    #[test]
    fn recognizes_common_anime_chapter_variants_without_risky_story_labels() {
        assert_eq!(classify_chapter("OP 2", 0, 3), Some("intro"));
        assert_eq!(classify_chapter("First Chapter", 0, 3), Some("intro"));
        assert_eq!(classify_chapter("ED1", 1, 3), Some("outro"));
        assert_eq!(classify_chapter("次回予告", 2, 3), Some("preview"));
        assert_eq!(classify_chapter("Prologue", 0, 3), None);
        assert_eq!(classify_chapter("Cold Open", 0, 3), None);
        assert_eq!(classify_chapter("Flashback", 0, 3), None);
    }
}
