use super::helpers::{TMDB_ID_PREFIX, imdb_regex};

// pub rather than pub(crate): re-exported under fuzz_targets for the `fuzz/`
// crate (see lib.rs). Not part of the supported public API otherwise.
#[expect(
    clippy::indexing_slicing,
    reason = "length checks guard all split-part indexing"
)]
pub fn parse_episode_locator(raw: &str) -> Option<(String, i32, i32)> {
    let parts = raw
        .split(':')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>();
    if parts.len() >= 3 {
        let season = parts[parts.len() - 2].parse::<i32>().ok()?;
        let episode = parts[parts.len() - 1].parse::<i32>().ok()?;
        let base_id = parts[..parts.len() - 2].join(":");
        if !base_id.is_empty() {
            return Some((base_id, season, episode));
        }
    }

    if let Some((season, episode, _)) = scan_compact_episode_codes(raw).into_iter().next() {
        return Some((String::new(), season, episode));
    }

    let parts = raw
        .split([':', '/', '-', '_'])
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>();
    if parts.len() >= 3 {
        let season = parts[parts.len() - 2].parse::<i32>().ok()?;
        let episode = parts[parts.len() - 1].parse::<i32>().ok()?;
        return Some((parts[..parts.len() - 2].join(":"), season, episode));
    }
    None
}

// Scans `text` for every SxxExx-style code (case-insensitive), yielding each
// match's season, episode, and whether a digit immediately follows the parsed
// episode number — callers that match against a specific target use that to
// reject a longer digit run than what they're looking for (e.g. "S01E100"
// shouldn't count as a match for episode 10).
#[expect(
    clippy::indexing_slicing,
    reason = "cursor bounds are checked before each byte or ASCII slice access"
)]
pub(crate) fn scan_compact_episode_codes(text: &str) -> Vec<(i32, i32, bool)> {
    let lower = text.to_ascii_lowercase();
    let bytes = lower.as_bytes();
    let mut matches = Vec::new();
    for index in 0..bytes.len() {
        if bytes[index] != b's' {
            continue;
        }
        let mut cursor = index + 1;
        let season_start = cursor;
        while cursor < bytes.len() && bytes[cursor].is_ascii_digit() {
            cursor += 1;
        }
        if season_start == cursor || cursor >= bytes.len() || bytes[cursor] != b'e' {
            continue;
        }
        let episode_start = cursor + 1;
        cursor = episode_start;
        while cursor < bytes.len() && bytes[cursor].is_ascii_digit() {
            cursor += 1;
        }
        if episode_start == cursor {
            continue;
        }
        let season = lower[season_start..episode_start - 1].parse::<i32>().ok();
        let episode = lower[episode_start..cursor].parse::<i32>().ok();
        let next_is_digit = cursor < bytes.len() && bytes[cursor].is_ascii_digit();
        if let (Some(season), Some(episode)) = (season, episode) {
            matches.push((season, episode, next_is_digit));
        }
    }
    matches
}

pub(crate) fn imdb_id(raw: &str) -> Option<String> {
    imdb_regex()
        .find(raw)
        .map(|matched| matched.as_str().to_string())
}

pub(crate) fn base_content_id(id: &str) -> String {
    parse_episode_locator(id)
        .map(|(base_id, _, _)| {
            if base_id.is_empty() {
                id.to_string()
            } else {
                base_id
            }
        })
        .unwrap_or_else(|| id.to_string())
}

pub(crate) fn normalize_series_lookup_id(raw_id: &str) -> String {
    imdb_id(raw_id).unwrap_or_else(|| base_content_id(raw_id))
}

pub(crate) fn is_tmdb_like_content_id(id: &str) -> bool {
    let base = base_content_id(id);
    base.to_ascii_lowercase().starts_with(TMDB_ID_PREFIX) || base.parse::<i32>().is_ok()
}

pub(crate) fn tmdb_numeric_id(id: &str) -> Option<String> {
    let base = base_content_id(id);
    let numeric = base.strip_prefix(TMDB_ID_PREFIX).unwrap_or(&base);
    numeric.parse::<i32>().ok().map(|_| numeric.to_string())
}

pub(crate) fn episode_id(base_id: &str, season: i32, episode: i32) -> String {
    format!("{base_id}:{season}:{episode}")
}

#[expect(
    clippy::indexing_slicing,
    reason = "part lengths are checked before video-id component indexing"
)]
pub(crate) fn parse_video_id_json(id: &str) -> String {
    let parts: Vec<&str> = id.split(':').collect();
    let mut map = serde_json::Map::new();
    if parts.first().map(|p| p.starts_with("tt")).unwrap_or(false) {
        map.insert("imdb".into(), parts[0].into());
        if parts.len() >= 3 {
            if let (Ok(s), Ok(e)) = (parts[1].parse::<i64>(), parts[2].parse::<i64>()) {
                map.insert("season".into(), s.into());
                map.insert("episode".into(), e.into());
                map.insert("isEpisode".into(), true.into());
            } else {
                map.insert("isEpisode".into(), false.into());
            }
        } else {
            map.insert("isEpisode".into(), false.into());
        }
    } else if parts.first().map(|p| *p == "tmdb").unwrap_or(false) && parts.len() >= 2 {
        map.insert("tmdb".into(), parts[1].into());
        if parts.len() >= 4 {
            if let (Ok(s), Ok(e)) = (parts[2].parse::<i64>(), parts[3].parse::<i64>()) {
                map.insert("season".into(), s.into());
                map.insert("episode".into(), e.into());
                map.insert("isEpisode".into(), true.into());
            } else {
                map.insert("isEpisode".into(), false.into());
            }
        } else {
            map.insert("isEpisode".into(), false.into());
        }
    } else {
        map.insert("isEpisode".into(), false.into());
    }
    serde_json::to_string(&serde_json::Value::Object(map))
        .unwrap_or_else(|_| r#"{"isEpisode":false}"#.to_string())
}

pub(crate) fn build_trakt_ids_json(video_id: &str) -> Option<String> {
    let parsed_json = parse_video_id_json(video_id);
    let parsed: serde_json::Value = serde_json::from_str(&parsed_json).ok()?;
    if let Some(imdb) = parsed.get("imdb").and_then(serde_json::Value::as_str) {
        return serde_json::to_string(&serde_json::json!({"imdb": imdb})).ok();
    }
    if let Some(tmdb) = parsed.get("tmdb").and_then(serde_json::Value::as_str)
        && let Ok(n) = tmdb.parse::<i64>()
    {
        return serde_json::to_string(&serde_json::json!({"tmdb": n})).ok();
    }
    None
}
