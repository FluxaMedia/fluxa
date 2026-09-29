use super::id::{parse_episode_locator, scan_compact_episode_codes};

// pub rather than pub(crate): re-exported under fuzz_targets for the `fuzz/`
// crate (see lib.rs). Not part of the supported public API otherwise.
pub fn contains_compact_episode(text: &str, season: i32, episode: i32) -> bool {
    scan_compact_episode_codes(text)
        .into_iter()
        .any(|(s, e, next_is_digit)| s == season && e == episode && !next_is_digit)
}

// pub rather than pub(crate): re-exported under fuzz_targets for the `fuzz/`
// crate (see lib.rs). Not part of the supported public API otherwise.
pub fn contains_spaced_episode(text: &str, season: i32, episode: i32) -> bool {
    let lower = text.to_ascii_lowercase();
    let mut offset = 0;
    // `offset` can land exactly at lower.len() after a non-matching episode at the
    // very end of the string — .get() (rather than direct indexing) keeps that a
    // clean "nothing left to search" instead of a panic.
    while let Some(season_index) = lower.get(offset..).and_then(|rest| rest.find("season")) {
        let mut cursor = offset + season_index + "season".len();
        while lower
            .as_bytes()
            .get(cursor)
            .is_some_and(u8::is_ascii_whitespace)
        {
            cursor += 1;
        }
        let season_start = cursor;
        while lower.as_bytes().get(cursor).is_some_and(u8::is_ascii_digit) {
            cursor += 1;
        }
        if season_start == cursor || lower[season_start..cursor].parse::<i32>().ok() != Some(season)
        {
            offset = cursor.saturating_add(1);
            continue;
        }
        let Some(episode_word_index) = lower[cursor..].find("episode") else {
            return false;
        };
        cursor += episode_word_index + "episode".len();
        while lower
            .as_bytes()
            .get(cursor)
            .is_some_and(u8::is_ascii_whitespace)
        {
            cursor += 1;
        }
        let episode_start = cursor;
        while lower.as_bytes().get(cursor).is_some_and(u8::is_ascii_digit) {
            cursor += 1;
        }
        let next_is_digit = lower.as_bytes().get(cursor).is_some_and(u8::is_ascii_digit);
        if episode_start != cursor
            && lower[episode_start..cursor].parse::<i32>().ok() == Some(episode)
            && !next_is_digit
        {
            return true;
        }
        offset = cursor.saturating_add(1);
    }
    false
}

pub(crate) fn text_matches_episode(text: &str, season: i32, episode: i32) -> bool {
    contains_compact_episode(text, season, episode)
        || contains_spaced_episode(text, season, episode)
}

#[expect(
    clippy::indexing_slicing,
    reason = "loop and cursor guards establish all byte and ASCII slice bounds"
)]
fn dash_episode_matches(text: &str, episode: i32) -> Option<bool> {
    let lower = text.to_ascii_lowercase();
    let bytes = lower.as_bytes();
    let mut found = false;
    for index in 1..bytes.len() {
        if bytes[index] != b'-' || !bytes[index - 1].is_ascii_whitespace() {
            continue;
        }
        let mut cursor = index + 1;
        while cursor < bytes.len() && bytes[cursor].is_ascii_whitespace() {
            cursor += 1;
        }
        let start = cursor;
        while cursor < bytes.len() && bytes[cursor].is_ascii_digit() {
            cursor += 1;
        }
        let length = cursor.saturating_sub(start);
        if length == 0 || length > 3 {
            continue;
        }
        if cursor < bytes.len()
            && bytes[cursor].is_ascii_alphabetic()
            && !(bytes[cursor] == b'v'
                && cursor + 1 < bytes.len()
                && bytes[cursor + 1].is_ascii_digit())
        {
            continue;
        }
        let Ok(candidate) = lower[start..cursor].parse::<i32>() else {
            continue;
        };
        found = true;
        if candidate == episode {
            return Some(true);
        }
    }
    found.then_some(false)
}

pub(crate) fn stream_matches_episode(video_id: &str, fields: &[String]) -> bool {
    let Some((_, season, episode)) = parse_episode_locator(video_id) else {
        return true;
    };
    let text = fields
        .iter()
        .filter(|value| !value.trim().is_empty())
        .cloned()
        .collect::<Vec<_>>()
        .join(" ");
    if text.trim().is_empty() {
        return true;
    }
    if text_matches_episode(&text, season, episode) {
        return true;
    }
    if let Some(matches) = dash_episode_matches(&text, episode) {
        return matches;
    }
    !contains_any_compact_episode(&text)
}

#[expect(
    clippy::indexing_slicing,
    reason = "cursor bounds are checked before every compact-code byte access"
)]
pub(crate) fn contains_any_compact_episode(text: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    let bytes = lower.as_bytes();
    for index in 0..bytes.len() {
        if bytes[index] != b's' {
            continue;
        }
        let mut cursor = index + 1;
        let season_start = cursor;
        while cursor < bytes.len() && bytes[cursor].is_ascii_digit() {
            cursor += 1;
        }
        if season_start == cursor || season_start + 2 < cursor {
            continue;
        }
        if cursor >= bytes.len() || bytes[cursor] != b'e' {
            continue;
        }
        let episode_start = cursor + 1;
        cursor = episode_start;
        while cursor < bytes.len() && bytes[cursor].is_ascii_digit() {
            cursor += 1;
        }
        if episode_start != cursor && cursor - episode_start <= 3 {
            return true;
        }
    }
    false
}
