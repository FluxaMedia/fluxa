//! Platform-independent local-media naming and metadata matching policy.
//!
//! Hosts enumerate local/SMB/WebDAV files and perform network requests. This module owns the
//! decisions that must be identical on every host: parsing a media filename, scoring metadata
//! candidates, and mapping a parsed episode to a metadata video.

use regex::Regex;
use serde_json::{Value, json};
use std::sync::OnceLock;

fn video_extensions() -> &'static [&'static str] {
    &["mkv", "mp4", "m4v", "avi", "mov", "webm", "ts", "m2ts", "wmv", "flv"]
}

fn season_episode() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"(?i)(?:^|[ ._\-])S(\d{1,3})[ ._\-]*E(\d{1,4})(?:[^0-9]|$)").unwrap())
}

fn season_episode_alt() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"(?i)(?:^|[ ._\-])(\d{1,2})x(\d{1,4})(?:[^0-9]|$)").unwrap())
}

fn episode_only() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"(?i)(?:^|[ ._\-])(?:EP?|Episode)[ ._\-]*(\d{1,4})(?:[^0-9]|$)").unwrap())
}

fn anime_absolute() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"(?i)(?:^|[ ._\-])-?[ ._]*(\d{1,4})(?:v\d+)?(?=[ ._\-]*(?:\[|\(|$))").unwrap())
}

fn year_pattern() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"(?:^|[^0-9])((?:19|20)\d{2})(?:[^0-9]|$)").unwrap())
}

fn explicit_id() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"(?i)(?:\{|\[|\()?(imdb|tmdb)[-_: ](tt\d+|\d+)(?:\}|\]|\))?").unwrap())
}

fn release_noise() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"(?i)\b(?:2160p|1080p|720p|480p|uhd|bluray|blu-ray|bdrip|brrip|web[- .]?dl|webrip|hdtv|remux|x26[45]|h26[45]|hevc|av1|hdr10\+?|hdr|dv|dolby[ .]?vision|atmos|truehd|dts(?:-hd)?|aac|ddp?\d?(?:\.\d)?|proper|repack|extended|multi|dual|nf|amzn|dsnp|hmax)\b").unwrap())
}

fn bracket_group() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^\[[^]]+][ ._\-]*").unwrap())
}

fn is_video_file(name: &str) -> bool {
    let extension = name.rsplit_once('.').map(|(_, ext)| ext).unwrap_or("").to_ascii_lowercase();
    video_extensions().contains(&extension.as_str())
}

fn string_array(value: Option<&Value>) -> Vec<&str> {
    value
        .and_then(Value::as_array)
        .map(|items| items.iter().filter_map(Value::as_str).collect())
        .unwrap_or_default()
}

fn clean_title(raw: &str, year: Option<i64>, episode_start: Option<usize>) -> String {
    let mut value = raw.to_string();
    if let Some(start) = episode_start.filter(|start| *start > 0 && *start < value.len()) {
        value.truncate(start);
    }
    value = bracket_group().replace(&value, "").into_owned();
    value = explicit_id().replace_all(&value, " ").into_owned();
    if let Some(year) = year {
        value = value.replace(&year.to_string(), " ");
    }
    value = release_noise().replace_all(&value, " ").into_owned();
    value = Regex::new(r"(?i)[ ._\-]+S\d{1,3}[ ._\-]*E\d{1,4}.*$").unwrap().replace(&value, "").into_owned();
    value = Regex::new(r"(?i)[ ._\-]+\d{1,2}x\d{1,4}.*$").unwrap().replace(&value, "").into_owned();
    value = Regex::new(r"[._]+").unwrap().replace_all(&value, " ").into_owned();
    value = Regex::new(r"\s+-\s+\d{1,4}.*$").unwrap().replace(&value, "").into_owned();
    value = Regex::new(r"\s+").unwrap().replace_all(&value, " ").into_owned();
    value.trim_matches([' ', '-', '_', '.']).to_string()
}

fn parse_filename(args: &Value) -> Option<Value> {
    let file_name = args.get("fileName")?.as_str()?;
    let kind = args.get("kind").and_then(Value::as_str).unwrap_or("movies");
    if !is_video_file(file_name) {
        return None;
    }
    let parent_hints = string_array(args.get("parentHints"));
    let stem = file_name.rsplit_once('.').map(|(stem, _)| stem).unwrap_or(file_name);
    let explicit = explicit_id().captures(stem).map(|captures| (
        captures.get(1).map(|value| value.as_str().to_ascii_lowercase()),
        captures.get(2).map(|value| value.as_str().to_string()),
    )).or_else(|| parent_hints.iter().find_map(|hint| explicit_id().captures(hint).map(|captures| (
        captures.get(1).map(|value| value.as_str().to_ascii_lowercase()),
        captures.get(2).map(|value| value.as_str().to_string()),
    ))));
    let season_match = season_episode().find(stem).or_else(|| season_episode_alt().find(stem));
    let season_captures = season_episode().captures(stem).or_else(|| season_episode_alt().captures(stem));
    let season = season_captures.as_ref().and_then(|c| c.get(1)?.as_str().parse::<i64>().ok());
    let episode = season_captures.as_ref().and_then(|c| c.get(2)?.as_str().parse::<i64>().ok());
    let episode_only_value = if season_match.is_none() {
        episode_only().captures(stem).and_then(|c| c.get(1)?.as_str().parse::<i64>().ok())
    } else { None };
    let absolute = if kind.eq_ignore_ascii_case("anime") && season_match.is_none() {
        episode_only_value.or_else(|| anime_absolute().captures(stem).and_then(|c| c.get(1)?.as_str().parse::<i64>().ok()))
    } else { None };
    let year = year_pattern().captures(stem).and_then(|c| c.get(1)?.as_str().parse::<i64>().ok())
        .or_else(|| parent_hints.iter().find_map(|hint| year_pattern().captures(hint).and_then(|c| c.get(1)?.as_str().parse::<i64>().ok())));
    let generic_folder = Regex::new(r"(?i)^(?:movies?|films?|tv(?:[ ._-]*shows?)?|series|shows?|anime|media|season[ ._-]*\d+|s\d+)$").unwrap();
    let parent_title = if kind.eq_ignore_ascii_case("movies") { None } else {
        parent_hints.iter().copied().find(|hint| !hint.trim().is_empty() && !generic_folder.is_match(hint.trim()))
    };
    let title_source = parent_title.unwrap_or(stem);
    let title = clean_title(title_source, year, season_match.as_ref().and_then(|m| if parent_title.is_none() { Some(m.start()) } else { None }));
    if title.is_empty() { return None; }
    Some(json!({
        "title": title,
        "year": year,
        "season": season,
        "episode": episode.or_else(|| if kind.eq_ignore_ascii_case("tvShows") { episode_only_value } else { None }),
        "absoluteEpisode": absolute,
        "explicitMetadataId": explicit.as_ref().and_then(|(_, id)| id.clone()),
        "explicitMetadataProvider": explicit.and_then(|(provider, _)| provider),
    }))
}

fn normalized_title(value: &str) -> String {
    let mut output = String::new();
    for ch in value.to_ascii_lowercase().chars() {
        if ch.is_ascii_lowercase() || ch.is_ascii_digit() { output.push(ch); } else if !output.ends_with(' ') { output.push(' '); }
    }
    output.trim().split_whitespace().collect::<Vec<_>>().join(" ")
}

fn title_similarity(left: &str, right: &str) -> f64 {
    let left = normalized_title(left);
    let right = normalized_title(right);
    if left == right { return 1.0; }
    if left.is_empty() || right.is_empty() { return 0.0; }
    let left_tokens: std::collections::HashSet<&str> = left.split(' ').filter(|v| !v.is_empty()).collect();
    let right_tokens: std::collections::HashSet<&str> = right.split(' ').filter(|v| !v.is_empty()).collect();
    let union = left_tokens.union(&right_tokens).count() as f64;
    let token_score = if union == 0.0 { 0.0 } else { left_tokens.intersection(&right_tokens).count() as f64 / union };
    let prefix = left.chars().zip(right.chars()).take_while(|(a, b)| a == b).count() as f64;
    let prefix_score = prefix / left.chars().count().max(right.chars().count()).max(1) as f64;
    (token_score * 0.82 + prefix_score * 0.18).clamp(0.0, 1.0)
}

fn score_candidate(args: &Value) -> Option<Value> {
    let parsed = args.get("parsed")?;
    let meta = args.get("meta")?;
    let kind = args.get("kind").and_then(Value::as_str).unwrap_or("movies");
    let parsed_title = parsed.get("title")?.as_str()?;
    let meta_name = meta.get("name").and_then(Value::as_str).unwrap_or("");
    let mut score = title_similarity(parsed_title, meta_name) * 0.82;
    let parsed_year = parsed.get("year").and_then(Value::as_i64);
    let meta_year = meta.get("releaseInfo").and_then(Value::as_str).and_then(|v| v.get(..4)).and_then(|v| v.parse::<i64>().ok())
        .or_else(|| meta.get("released").and_then(Value::as_str).and_then(|v| v.get(..4)).and_then(|v| v.parse::<i64>().ok()));
    if let (Some(a), Some(b)) = (parsed_year, meta_year) { score += match (a - b).abs() { 0 => 0.18, 1 => 0.06, _ => -0.12 }; } else { score += 0.08; }
    let meta_type = meta.get("type").and_then(Value::as_str).unwrap_or("").to_ascii_lowercase();
    let type_matches = if kind.eq_ignore_ascii_case("movies") { meta_type == "movie" } else { matches!(meta_type.as_str(), "series" | "tv" | "anime") };
    if !type_matches { score -= 0.35; }
    Some(json!(score.clamp(0.0, 1.0)))
}

fn resolve_video(args: &Value) -> Option<Value> {
    let parsed = args.get("parsed")?;
    let videos = args.get("videos")?.as_array()?;
    let season = parsed.get("season").and_then(Value::as_i64);
    let episode = parsed.get("episode").and_then(Value::as_i64);
    if let (Some(season), Some(episode)) = (season, episode) {
        return videos.iter().find(|video| video.get("season").and_then(Value::as_i64) == Some(season) && video.get("number").and_then(Value::as_i64) == Some(episode)).cloned()
            .or_else(|| videos.iter().find(|video| video.get("number").and_then(Value::as_i64) == Some(episode)).cloned());
    }
    let absolute = parsed.get("absoluteEpisode").and_then(Value::as_i64)?;
    videos.iter().find(|video| video.get("number").and_then(Value::as_i64) == Some(absolute) && video.get("season").and_then(Value::as_i64).unwrap_or(1) <= 1).cloned()
        .or_else(|| videos.iter().filter(|video| video.get("number").and_then(Value::as_i64).unwrap_or(0) > 0).nth(absolute.saturating_sub(1) as usize).cloned())
}

pub(crate) fn route(method: &str, args: &Value) -> Option<Value> {
    match method {
        "localMediaIsVideoFile" => Some(json!(args.get("name").and_then(Value::as_str).is_some_and(is_video_file))),
        "localMediaNormalizedTitle" => Some(json!(normalized_title(args.get("value")?.as_str()?))),
        "localMediaParseFilename" => Some(parse_filename(args).unwrap_or(Value::Null)),
        "localMediaTitleSimilarity" => Some(json!(title_similarity(args.get("left")?.as_str()?, args.get("right")?.as_str()?))),
        "localMediaScoreCandidate" => score_candidate(args),
        "localMediaResolveVideo" => Some(resolve_video(args).unwrap_or(Value::Null)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::route;
    use serde_json::json;

    #[test]
    fn parses_series_filename_and_explicit_id() {
        let value = route("localMediaParseFilename", &json!({
            "fileName": "[Grp] The.Show.2024.S01E02.1080p.mkv",
            "parentHints": [], "kind": "tvShows"
        })).unwrap();
        assert_eq!(value["title"], "The Show");
        assert_eq!(value["year"], 2024);
        assert_eq!(value["season"], 1);
        assert_eq!(value["episode"], 2);
    }

    #[test]
    fn scores_exact_metadata_highly() {
        let score = route("localMediaScoreCandidate", &json!({
            "parsed": {"title":"The Matrix", "year":1999},
            "meta": {"name":"The Matrix", "type":"movie", "releaseInfo":"1999"},
            "kind":"movies"
        })).unwrap().as_f64().unwrap();
        assert!(score >= 0.99);
    }
}
