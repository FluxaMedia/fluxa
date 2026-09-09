use serde_json::{json, Value};
use std::collections::HashSet;
use std::sync::OnceLock;

use regex::Regex;

pub(crate) fn cloudstream_quality_score(value: &str) -> i64 {
    let value = value.to_ascii_lowercase();
    if value.contains("4k") || value.contains("2160") {
        2160
    } else if value.contains("1440") {
        1440
    } else if value.contains("1080") {
        1080
    } else if value.contains("720") {
        720
    } else if value.contains("480") {
        480
    } else if value.contains("360") {
        360
    } else if value.contains("240") {
        240
    } else {
        0
    }
}

pub(crate) fn cloudstream_quality_label(value: &str) -> Option<String> {
    static DIMENSIONS: OnceLock<Regex> = OnceLock::new();
    static PROGRESSIVE: OnceLock<Regex> = OnceLock::new();
    let dimensions = DIMENSIONS.get_or_init(|| Regex::new(r"\b\d{3,5}\s*x\s*\d{3,5}\b").unwrap());
    if let Some(value) = dimensions.find(value) {
        return Some(value.as_str().replace(char::is_whitespace, ""));
    }
    let progressive = PROGRESSIVE.get_or_init(|| Regex::new(r"(?i)\b\d{3,4}p\b").unwrap());
    progressive
        .find(value)
        .map(|value| value.as_str().to_ascii_lowercase())
}

pub(crate) fn cloudstream_content_type(value: &str) -> &'static str {
    match value.to_ascii_lowercase().as_str() {
        "tvseries" | "anime" | "ova" | "cartoon" | "asiandrama" => "series",
        _ => "movie",
    }
}

pub(crate) fn cloudstream_stream_order_json(args_json: &str) -> Option<String> {
    let args: Value = serde_json::from_str(args_json).ok()?;
    let links = args.get("links")?.as_array()?;
    let sort_by_quality = args
        .get("sortByQuality")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let mut seen = HashSet::new();
    let mut ordered = links
        .iter()
        .enumerate()
        .filter_map(|(index, link)| {
            let url = link.get("url").and_then(Value::as_str).unwrap_or("");
            let key = url.trim().to_ascii_lowercase();
            if !key.is_empty() && !seen.insert(key) {
                return None;
            }
            Some((
                index,
                cloudstream_quality_score(
                    link.get("quality").and_then(Value::as_str).unwrap_or(""),
                ),
            ))
        })
        .collect::<Vec<_>>();
    if sort_by_quality {
        ordered.sort_by(|left, right| right.1.cmp(&left.1));
    }
    serde_json::to_string(
        &ordered
            .into_iter()
            .map(|(index, _)| index)
            .collect::<Vec<_>>(),
    )
    .ok()
}

pub(crate) fn cloudstream_match_score_json(args_json: &str) -> Option<String> {
    let args: Value = serde_json::from_str(args_json).ok()?;
    let mode = args
        .get("mode")
        .and_then(Value::as_str)
        .unwrap_or("scraper");
    let target_title = args.get("targetTitle").and_then(Value::as_str)?;
    let candidate_title = args.get("candidateTitle").and_then(Value::as_str)?;
    let original_title = args.get("originalTitle").and_then(Value::as_str);
    let target_year = args.get("targetYear").and_then(Value::as_i64);
    let candidate_year = args.get("candidateYear").and_then(Value::as_i64);
    let candidate_type = args
        .get("candidateType")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_ascii_lowercase();
    let is_movie = args
        .get("isMovie")
        .and_then(Value::as_bool)
        .unwrap_or(false);

    let title_similarity = cloudstream_similarity(candidate_title, target_title);
    let similarity = original_title
        .map(|title| title_similarity.max(cloudstream_similarity(candidate_title, title)))
        .unwrap_or(title_similarity);
    let type_bonus = match (mode, is_movie) {
        ("search", true)
            if ["movie", "animemovie", "documentary"]
                .iter()
                .any(|kind| candidate_type.contains(kind)) =>
        {
            0.1
        }
        ("search", false)
            if ["tvseries", "anime", "ova", "cartoon"]
                .iter()
                .any(|kind| candidate_type.contains(kind)) =>
        {
            0.1
        }
        (_, true)
            if ["movie", "animemovie"]
                .iter()
                .any(|kind| candidate_type.contains(kind)) =>
        {
            0.15
        }
        (_, false)
            if ["tvseries", "anime", "asiandrama"]
                .iter()
                .any(|kind| candidate_type.contains(kind)) =>
        {
            0.15
        }
        ("search", _) if !candidate_type.is_empty() => -0.1,
        _ => 0.0,
    };
    let year_bonus = (target_year.is_some() && target_year == candidate_year)
        .then_some(if mode == "search" { 0.1 } else { 0.25 })
        .unwrap_or(0.0);
    let score = similarity + type_bonus + year_bonus;
    let threshold = if mode == "search" { 0.35 } else { 0.4 };
    (score >= threshold)
        .then(|| serde_json::to_string(&json!(score)).ok())
        .flatten()
}

fn cloudstream_similarity(left: &str, right: &str) -> f64 {
    let left = left.trim().to_ascii_lowercase();
    let right = right.trim().to_ascii_lowercase();
    if left == right {
        return 1.0;
    }
    if left.contains(&right) || right.contains(&left) {
        return 0.85;
    }
    let distance = levenshtein(&left, &right);
    let max_len = left.chars().count().max(right.chars().count());
    let levenshtein_score = 1.0 - (distance as f64 / max_len as f64);
    let left_words: HashSet<&str> = left
        .split([' ', '\t', '\n', '\r', '-', '_', ':'])
        .filter(|word| word.chars().count() > 1)
        .collect();
    let right_words: HashSet<&str> = right
        .split([' ', '\t', '\n', '\r', '-', '_', ':'])
        .filter(|word| word.chars().count() > 1)
        .collect();
    let common_words = left_words.intersection(&right_words).count();
    let total_words = left_words.len().max(right_words.len()).max(1);
    let word_score = common_words as f64 / total_words as f64;
    levenshtein_score.max(word_score * 0.9)
}

fn levenshtein(left: &str, right: &str) -> usize {
    let right: Vec<char> = right.chars().collect();
    let mut previous: Vec<usize> = (0..=right.len()).collect();
    for (row, left_char) in left.chars().enumerate() {
        let mut current = vec![row + 1; right.len() + 1];
        for (column, right_char) in right.iter().enumerate() {
            current[column + 1] = if left_char == *right_char {
                previous[column]
            } else {
                1 + previous[column]
                    .min(previous[column + 1])
                    .min(current[column])
            };
        }
        previous = current;
    }
    previous[right.len()]
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, Value};

    #[test]
    fn quality_score_prefers_the_highest_known_resolution() {
        assert_eq!(cloudstream_quality_score("WEB-DL 4K"), 2160);
        assert_eq!(cloudstream_quality_score("1080p"), 1080);
        assert_eq!(cloudstream_quality_score("unknown"), 0);
    }

    #[test]
    fn quality_label_extracts_dimensions_and_progressive_resolution() {
        assert_eq!(
            cloudstream_quality_label("WEB 1920 x 1080"),
            Some("1920x1080".into())
        );
        assert_eq!(
            cloudstream_quality_label("release 1080P"),
            Some("1080p".into())
        );
        assert_eq!(cloudstream_quality_label("unknown"), None);
    }

    #[test]
    fn content_type_mapping_keeps_cloudstream_series_as_series() {
        assert_eq!(cloudstream_content_type("TvSeries"), "series");
        assert_eq!(cloudstream_content_type("Anime"), "series");
        assert_eq!(cloudstream_content_type("AnimeMovie"), "movie");
        assert_eq!(cloudstream_content_type("Documentary"), "movie");
    }

    #[test]
    fn stream_order_deduplicates_urls_and_keeps_quality_order_optional() {
        let ordered: Value = serde_json::from_str(
            &cloudstream_stream_order_json(
                &json!({
                    "sortByQuality": true,
                    "links": [
                        {"url":"https://a/video", "quality":"720p"},
                        {"url":"HTTPS://A/VIDEO", "quality":"1080p"},
                        {"url":"https://b/video", "quality":"4K"}
                    ]
                })
                .to_string(),
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(ordered, json!([2, 0]));
    }

    #[test]
    fn match_score_applies_title_year_and_type_policy() {
        let score: Value = serde_json::from_str(
            &cloudstream_match_score_json(
                &json!({
                    "mode": "scraper",
                    "targetTitle": "Example",
                    "originalTitle": "Beispiel",
                    "candidateTitle": "Example",
                    "targetYear": 2025,
                    "candidateYear": 2025,
                    "candidateType": "TvSeries",
                    "isMovie": false
                })
                .to_string(),
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(score, json!(1.4));
    }
}
