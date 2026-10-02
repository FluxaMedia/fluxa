mod tick;
mod toast;
mod tracks;

pub(crate) use tick::player_tick_plan_json;
pub(crate) use toast::player_toast_plan_json;
pub(crate) use tracks::player_track_plan_json;

use crate::player::playback::desktop;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

const CARD_KINDS: [&str; 3] = ["intro", "recap", "outro"];
const MIN_CHAPTER_FRACTION: f64 = 0.004;
const SKIP_LEAD_MS: i64 = 400;

#[derive(Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
struct Chapter {
    title: String,
    start_ms: i64,
}

#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Segment {
    #[serde(rename = "type")]
    kind: String,
    start_time: i64,
    end_time: i64,
    #[serde(default)]
    provider: Option<String>,
}

#[derive(Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
struct NextEpisode {
    title: Option<String>,
    season: Option<i64>,
    episode: Option<i64>,
    thumbnail: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", default)]
struct OverlayPrefs {
    auto_skip_intro: bool,
    auto_play_next_episode: bool,
    next_episode_threshold_percent: f64,
    auto_play_countdown_secs: i64,
    use_skip_segments: bool,
}

impl Default for OverlayPrefs {
    fn default() -> Self {
        Self {
            auto_skip_intro: false,
            auto_play_next_episode: true,
            next_episode_threshold_percent: 90.0,
            auto_play_countdown_secs: 7,
            use_skip_segments: true,
        }
    }
}

#[derive(Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
struct OverlayInput {
    position_ms: i64,
    duration_ms: i64,
    chapters: Vec<Chapter>,
    segments: Vec<Value>,
    next_episode: Option<NextEpisode>,
    prefs: OverlayPrefs,
    dismissed: Vec<String>,
    next_dismissed: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ChapterSpan {
    start_ms: i64,
    start_fraction: f64,
    end_fraction: f64,
    title: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SkipCard {
    kind: String,
    start_ms: i64,
    end_ms: i64,
    seek_to_ms: i64,
    auto: bool,
    provider: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SegmentSpan {
    kind: String,
    start_fraction: f64,
    end_fraction: f64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct NextCard {
    title: Option<String>,
    season: Option<i64>,
    episode: Option<i64>,
    thumbnail: Option<String>,
    remaining_ms: i64,
    countdown_secs: i64,
    auto_play: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct OverlayPlan {
    chapters: Vec<ChapterSpan>,
    segments: Vec<SegmentSpan>,
    skip: Option<SkipCard>,
    next: Option<NextCard>,
}

pub(crate) fn player_overlay_plan_json(input: &str) -> Option<String> {
    let input: OverlayInput = serde_json::from_str(input).ok()?;
    let duration = input.duration_ms.max(0);
    let next = next_card(&input, duration);
    let skip = skip_card(&input, duration).filter(|skip| next.is_none() || skip.kind != "outro");
    let plan = OverlayPlan {
        chapters: chapter_spans(&input.chapters, duration),
        segments: segment_spans(&input, duration),
        skip,
        next,
    };
    serde_json::to_string(&plan).ok()
}

fn chapter_spans(chapters: &[Chapter], duration: i64) -> Vec<ChapterSpan> {
    if duration <= 0 {
        return Vec::new();
    }
    let mut starts: Vec<&Chapter> = chapters
        .iter()
        .filter(|chapter| chapter.start_ms >= 0 && chapter.start_ms < duration)
        .collect();
    starts.sort_by_key(|chapter| chapter.start_ms);
    starts.dedup_by_key(|chapter| chapter.start_ms);
    let fraction = |ms: i64| ms as f64 / duration as f64;
    let mut spans: Vec<ChapterSpan> = Vec::new();
    for (index, chapter) in starts.iter().enumerate() {
        let end = starts
            .get(index + 1)
            .map(|next| next.start_ms)
            .unwrap_or(duration);
        let (start_fraction, end_fraction) = (fraction(chapter.start_ms), fraction(end));
        if end_fraction - start_fraction < MIN_CHAPTER_FRACTION {
            if let Some(last) = spans.last_mut() {
                last.end_fraction = end_fraction;
            }
            continue;
        }
        spans.push(ChapterSpan {
            start_ms: chapter.start_ms,
            start_fraction,
            end_fraction,
            title: chapter.title.trim().to_string(),
        });
    }
    if let Some(first) = spans.first_mut() {
        first.start_fraction = 0.0;
    }
    spans
}

fn merged_segments(input: &OverlayInput, duration: i64) -> Vec<Segment> {
    let mut segments: Vec<Segment> = input
        .segments
        .iter()
        .filter_map(|value| serde_json::from_value(value.clone()).ok())
        .collect();
    let chapters = json!(
        input
            .chapters
            .iter()
            .map(|chapter| json!({"title": chapter.title, "startMs": chapter.start_ms}))
            .collect::<Vec<_>>()
    );
    let from_chapters: Vec<Segment> = serde_json::from_str(&desktop::chapter_skip_segments_json(
        &chapters.to_string(),
        duration,
    ))
    .unwrap_or_default();
    for segment in from_chapters {
        if !segments.iter().any(|known| known.kind == segment.kind) {
            segments.push(segment);
        }
    }
    segments.retain(|segment| {
        CARD_KINDS.contains(&segment.kind.as_str()) && segment.end_time > segment.start_time
    });
    segments
}

fn segment_spans(input: &OverlayInput, duration: i64) -> Vec<SegmentSpan> {
    if duration <= 0 || !input.prefs.use_skip_segments {
        return Vec::new();
    }
    let fraction = |ms: i64| (ms as f64 / duration as f64).clamp(0.0, 1.0);
    merged_segments(input, duration)
        .into_iter()
        .map(|segment| SegmentSpan {
            start_fraction: fraction(segment.start_time),
            end_fraction: fraction(segment.end_time),
            kind: segment.kind,
        })
        .collect()
}

fn skip_card(input: &OverlayInput, duration: i64) -> Option<SkipCard> {
    if !input.prefs.use_skip_segments {
        return None;
    }
    let position = input.position_ms;
    merged_segments(input, duration)
        .into_iter()
        .filter(|segment| !input.dismissed.contains(&segment.kind))
        .find(|segment| {
            position >= segment.start_time && position < segment.end_time - SKIP_LEAD_MS
        })
        .map(|segment| SkipCard {
            auto: segment.kind == "intro" && input.prefs.auto_skip_intro,
            seek_to_ms: segment.end_time,
            start_ms: segment.start_time,
            end_ms: segment.end_time,
            provider: segment.provider,
            kind: segment.kind,
        })
}

fn next_card(input: &OverlayInput, duration: i64) -> Option<NextCard> {
    let next = input.next_episode.as_ref()?;
    if input.next_dismissed || duration <= 0 {
        return None;
    }
    let outro_start = merged_segments(input, duration)
        .into_iter()
        .filter(|segment| segment.kind == "outro")
        .map(|segment| segment.start_time)
        .filter(|start| *start > duration / 2)
        .min();
    let threshold_ms =
        (duration as f64 * input.prefs.next_episode_threshold_percent / 100.0) as i64;
    let show_at = outro_start.map_or(threshold_ms, |start| start.min(threshold_ms));
    (input.position_ms >= show_at).then(|| NextCard {
        title: next.title.clone(),
        season: next.season,
        episode: next.episode,
        thumbnail: next.thumbnail.clone(),
        remaining_ms: (duration - input.position_ms).max(0),
        countdown_secs: input.prefs.auto_play_countdown_secs.clamp(1, 60),
        auto_play: input.prefs.auto_play_next_episode,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn plan(input: Value) -> Value {
        serde_json::from_str(&player_overlay_plan_json(&input.to_string()).unwrap()).unwrap()
    }

    #[test]
    fn chapters_cover_the_bar_and_tiny_ones_merge() {
        let result = plan(json!({
            "durationMs": 1000000,
            "chapters": [
                {"startMs": 0, "title": "Opening"},
                {"startMs": 100000, "title": "Part A"},
                {"startMs": 101000, "title": "Blip"},
                {"startMs": 600000, "title": "Part B"}
            ]
        }));
        let chapters = result["chapters"].as_array().unwrap();
        assert_eq!(chapters.len(), 3);
        assert_eq!(chapters[1]["endFraction"], 0.6);
        assert_eq!(chapters[0]["startFraction"], 0.0);
        assert_eq!(chapters[2]["endFraction"], 1.0);
    }

    #[test]
    fn skip_card_appears_inside_segment_only() {
        let segments = json!([{"type": "intro", "startTime": 30000, "endTime": 90000}]);
        let inside =
            plan(json!({"positionMs": 45000, "durationMs": 1500000, "segments": segments}));
        assert_eq!(inside["skip"]["kind"], "intro");
        assert_eq!(inside["skip"]["seekToMs"], 90000);
        let before =
            plan(json!({"positionMs": 10000, "durationMs": 1500000, "segments": segments}));
        assert!(before["skip"].is_null());
        let dismissed = plan(json!({
            "positionMs": 45000, "durationMs": 1500000,
            "segments": segments, "dismissed": ["intro"]
        }));
        assert!(dismissed["skip"].is_null());
    }

    #[test]
    fn skip_card_and_spans_carry_provider_and_fractions() {
        let segments = json!([{
            "type": "intro", "startTime": 100000, "endTime": 200000, "provider": "IntroDB"
        }]);
        let result = plan(json!({"positionMs": 150000, "durationMs": 1000000, "segments": segments}));
        assert_eq!(result["skip"]["provider"], "IntroDB");
        assert_eq!(result["segments"][0]["startFraction"], 0.1);
        assert_eq!(result["segments"][0]["endFraction"], 0.2);
    }

    #[test]
    fn auto_skip_only_applies_to_intro() {
        let segments = json!([
            {"type": "intro", "startTime": 0, "endTime": 60000},
            {"type": "recap", "startTime": 60000, "endTime": 120000}
        ]);
        let intro = plan(json!({
            "positionMs": 1000, "durationMs": 1500000,
            "segments": segments, "prefs": {"autoSkipIntro": true}
        }));
        assert_eq!(intro["skip"]["auto"], true);
        let recap = plan(json!({
            "positionMs": 70000, "durationMs": 1500000,
            "segments": segments, "prefs": {"autoSkipIntro": true}
        }));
        assert_eq!(recap["skip"]["auto"], false);
    }

    #[test]
    fn next_card_replaces_outro_skip() {
        let result = plan(json!({
            "positionMs": 1400000, "durationMs": 1500000,
            "segments": [{"type": "outro", "startTime": 1380000, "endTime": 1500000}],
            "nextEpisode": {"title": "Pilot", "season": 1, "episode": 2}
        }));
        assert_eq!(result["next"]["episode"], 2);
        assert!(result["skip"].is_null());
    }

    #[test]
    fn next_card_waits_for_threshold() {
        let early = plan(json!({
            "positionMs": 500000, "durationMs": 1500000,
            "nextEpisode": {"title": "Pilot"}
        }));
        assert!(early["next"].is_null());
        let late = plan(json!({
            "positionMs": 1400000, "durationMs": 1500000,
            "nextEpisode": {"title": "Pilot"}
        }));
        assert_eq!(late["next"]["countdownSecs"], 7);
    }
}
