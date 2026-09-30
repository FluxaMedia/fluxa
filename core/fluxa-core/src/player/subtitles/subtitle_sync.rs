use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::OnceLock;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SubtitleSyncRequest {
    #[serde(default)]
    subtitle_text: String,
    #[serde(default)]
    subtitle_cues: Vec<Interval>,
    #[serde(default)]
    speech_intervals: Vec<Interval>,
}

#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Interval {
    pub(crate) start: f64,
    pub(crate) end: f64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SubtitleCueRequest {
    subtitle_text: String,
    current_time: f64,
    #[serde(default = "default_cue_window")]
    window_seconds: f64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SubtitleCueListRequest {
    subtitle_text: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SubtitleSyncApplyRequest {
    captured_time: f64,
    cue_start: f64,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub(crate) struct SubtitleCue {
    pub(crate) start: f64,
    pub(crate) end: f64,
    pub(crate) text: String,
}

fn default_cue_window() -> f64 {
    30.0
}

pub(crate) fn subtitle_cues_around_time_json(request_json: &str) -> Option<String> {
    let request = serde_json::from_str::<SubtitleCueRequest>(request_json).ok()?;
    if !request.current_time.is_finite() {
        return None;
    }
    let window = request.window_seconds.clamp(5.0, 120.0);
    let cues = parse_subtitle_cues_with_text(&request.subtitle_text)
        .into_iter()
        .filter(|cue| {
            cue.end >= request.current_time - window && cue.start <= request.current_time + window
        })
        .map(|cue| json!({ "start": cue.start, "end": cue.end, "text": cue.text }))
        .collect::<Vec<_>>();
    Some(json!({ "cues": cues }).to_string())
}

pub(crate) fn subtitle_cue_list_json(request_json: &str) -> Option<String> {
    let request = serde_json::from_str::<SubtitleCueListRequest>(request_json).ok()?;
    let cues = parse_subtitle_cues_with_text(&request.subtitle_text)
        .into_iter()
        .map(|cue| json!({ "start": cue.start, "end": cue.end, "text": cue.text }))
        .collect::<Vec<_>>();
    Some(json!({ "cues": cues }).to_string())
}

pub(crate) fn subtitle_sync_capture_json(request_json: &str) -> Option<String> {
    let request = serde_json::from_str::<SubtitleCueRequest>(request_json).ok()?;
    if !request.current_time.is_finite() {
        return None;
    }
    let cues = parse_subtitle_cues_with_text(&request.subtitle_text)
        .into_iter()
        .map(|cue| json!({ "start": cue.start, "end": cue.end, "text": cue.text }))
        .collect::<Vec<_>>();
    Some(json!({ "capturedTime": request.current_time, "cues": cues }).to_string())
}

pub(crate) fn subtitle_sync_apply_json(request_json: &str) -> Option<String> {
    let request = serde_json::from_str::<SubtitleSyncApplyRequest>(request_json).ok()?;
    let delay = delay_for_cue(request.captured_time, request.cue_start)?;
    Some(
        json!({
            "delaySeconds": delay,
            "capturedTime": request.captured_time,
            "cueStart": request.cue_start,
        })
        .to_string(),
    )
}

pub(crate) fn delay_for_cue(captured_time: f64, cue_start: f64) -> Option<f64> {
    if !captured_time.is_finite() || !cue_start.is_finite() {
        return None;
    }
    let delay = (captured_time - cue_start).clamp(-10_000.0, 10_000.0);
    Some((delay * 10.0).round() / 10.0)
}

pub(crate) fn estimate_subtitle_delay_json(request_json: &str) -> Option<String> {
    let request = serde_json::from_str::<SubtitleSyncRequest>(request_json).ok()?;
    let subtitle_cues = if request.subtitle_text.trim().is_empty() {
        request.subtitle_cues
    } else {
        parse_subtitle_cues(&request.subtitle_text)
    };
    let alignment = align(subtitle_cues, request.speech_intervals)?;
    Some(
        json!({
            "delaySeconds": alignment.delay_seconds,
            "scale": alignment.scale,
            "confidence": alignment.confidence,
        })
        .to_string(),
    )
}

pub(crate) struct Alignment {
    pub(crate) delay_seconds: f64,
    pub(crate) scale: f64,
    pub(crate) confidence: f64,
    pub(crate) matched: usize,
}

pub(crate) fn speech_intervals_from_energy(frame_seconds: f64, energies: &[f64]) -> Vec<Interval> {
    if !(frame_seconds.is_finite() && frame_seconds > 0.0) || energies.len() < 10 {
        return Vec::new();
    }
    let db = energies
        .iter()
        .map(|rms| 20.0 * rms.max(1e-6).log10())
        .collect::<Vec<_>>();
    let mut sorted = db.clone();
    sorted.sort_by(f64::total_cmp);
    let percentile = |p: f64| sorted[((sorted.len() - 1) as f64 * p) as usize];
    let floor = percentile(0.2);
    let ceiling = percentile(0.95);
    if ceiling - floor < 6.0 {
        return Vec::new();
    }
    let threshold = floor + (ceiling - floor) * 0.35;

    let min_frames = (0.3 / frame_seconds).ceil() as usize;
    let max_gap = (0.4 / frame_seconds).ceil() as usize;
    let mut runs: Vec<(usize, usize)> = Vec::new();
    for (index, value) in db.iter().enumerate() {
        if *value < threshold {
            continue;
        }
        match runs.last_mut() {
            Some(last) if index - last.1 <= max_gap => last.1 = index + 1,
            _ => runs.push((index, index + 1)),
        }
    }
    runs.into_iter()
        .filter(|(start, end)| end - start >= min_frames)
        .map(|(start, end)| Interval {
            start: start as f64 * frame_seconds,
            end: end as f64 * frame_seconds,
        })
        .collect()
}

pub(crate) fn intervals_from_text(text: &str) -> Vec<Interval> {
    parse_subtitle_cues(text)
}

pub(crate) fn align_subtitle_text(
    subtitle_text: &str,
    reference: Vec<Interval>,
) -> Option<Alignment> {
    align(parse_subtitle_cues(subtitle_text), reference)
}

pub(crate) fn retime(cues: &[SubtitleCue], alignment: &Alignment) -> Vec<SubtitleCue> {
    cues.iter()
        .map(|cue| SubtitleCue {
            start: cue.start * alignment.scale + alignment.delay_seconds,
            end: cue.end * alignment.scale + alignment.delay_seconds,
            text: cue.text.clone(),
        })
        .collect()
}

fn align(subtitle_cues: Vec<Interval>, reference: Vec<Interval>) -> Option<Alignment> {
    let subtitle_cues = normalize_intervals(subtitle_cues);
    let mut reference = normalize_intervals(reference);
    reference.sort_by(|a, b| a.start.total_cmp(&b.start));
    FRAMERATE_SCALES
        .iter()
        .filter_map(|&scale| {
            let scaled = subtitle_cues
                .iter()
                .map(|cue| Interval {
                    start: cue.start * scale,
                    end: cue.end * scale,
                })
                .collect::<Vec<_>>();
            let (coarse, confidence) = estimate_delay(&scaled, &reference)?;
            align_from(&subtitle_cues, &reference, scale, coarse, confidence)
        })
        .max_by_key(|alignment| alignment.matched)
}

const FRAMERATE_SCALES: [f64; 9] = [
    1.0,
    25.0 / 23.976,
    23.976 / 25.0,
    24.0 / 23.976,
    23.976 / 24.0,
    25.0 / 24.0,
    24.0 / 25.0,
    30.0 / 29.97,
    29.97 / 30.0,
];

fn align_from(
    subtitle_cues: &[Interval],
    reference: &[Interval],
    scale: f64,
    coarse: f64,
    confidence: f64,
) -> Option<Alignment> {
    let mut fit = (scale, coarse);
    let mut pairs = Vec::new();
    for tolerance in [3.0, 2.0, 1.0, 0.6, 0.6] {
        let matched = subtitle_cues
            .iter()
            .filter_map(|cue| {
                let target = cue.start * fit.0 + fit.1;
                let index = reference.partition_point(|item| item.start < target);
                let nearest = [index.checked_sub(1), Some(index)]
                    .into_iter()
                    .flatten()
                    .filter_map(|i| reference.get(i))
                    .min_by(|a, b| {
                        (a.start - target)
                            .abs()
                            .total_cmp(&(b.start - target).abs())
                    })?;
                ((nearest.start - target).abs() <= tolerance).then_some((cue.start, nearest.start))
            })
            .collect::<Vec<_>>();
        if matched.len() < 3 {
            break;
        }
        let Some(next) = fit_line(&matched) else {
            break;
        };
        pairs = matched;
        fit = next;
    }
    if pairs.len() < 3 {
        return None;
    }
    let (scale, offset) = if (fit.0 - 1.0).abs() < 0.0005 || !(0.9..=1.1).contains(&fit.0) {
        let mut diffs = pairs.iter().map(|(x, y)| y - x).collect::<Vec<_>>();
        diffs.sort_by(f64::total_cmp);
        (1.0, *diffs.get(diffs.len() / 2)?)
    } else {
        fit
    };
    Some(Alignment {
        delay_seconds: (offset * 10.0).round() / 10.0,
        scale,
        confidence,
        matched: pairs.len(),
    })
}

fn fit_line(pairs: &[(f64, f64)]) -> Option<(f64, f64)> {
    let n = pairs.len() as f64;
    let mean_x = pairs.iter().map(|p| p.0).sum::<f64>() / n;
    let mean_y = pairs.iter().map(|p| p.1).sum::<f64>() / n;
    let variance = pairs.iter().map(|p| (p.0 - mean_x).powi(2)).sum::<f64>();
    if variance < 1e-6 {
        return None;
    }
    let covariance = pairs
        .iter()
        .map(|p| (p.0 - mean_x) * (p.1 - mean_y))
        .sum::<f64>();
    let slope = covariance / variance;
    Some((slope, mean_y - slope * mean_x))
}

fn estimate_delay(subtitle_cues: &[Interval], speech_intervals: &[Interval]) -> Option<(f64, f64)> {
    if subtitle_cues.len() < 3 || speech_intervals.len() < 3 {
        return None;
    }

    let mut scores = Vec::new();
    for step in -240..=240 {
        let delay = step as f64 * 0.25;
        scores.push((delay, overlap_score(subtitle_cues, speech_intervals, delay)));
    }
    let (best_delay, best_score) = scores
        .iter()
        .copied()
        .max_by(|left, right| left.1.total_cmp(&right.1))?;
    let second_score = scores
        .iter()
        .filter(|(delay, _)| (delay - best_delay).abs() >= 1.0)
        .map(|(_, score)| *score)
        .max_by(f64::total_cmp)
        .unwrap_or(0.0);

    let refinement_start = best_delay - 0.25;
    let refinement_end = best_delay + 0.25;
    let mut refined_delay = best_delay;
    let mut refined_score = best_score;
    for step in 0..=20 {
        let delay = refinement_start + step as f64 * (refinement_end - refinement_start) / 20.0;
        let score = overlap_score(subtitle_cues, speech_intervals, delay);
        if score > refined_score {
            refined_score = score;
            refined_delay = delay;
        }
    }

    let confidence =
        ((refined_score - second_score.max(0.0)) / refined_score.max(0.001)).clamp(0.0, 1.0);
    if refined_score < 0.18 || confidence < 0.08 {
        return None;
    }
    Some((refined_delay, confidence))
}

fn parse_subtitle_cues(text: &str) -> Vec<Interval> {
    parse_subtitle_cues_with_text(text)
        .into_iter()
        .map(|cue| Interval {
            start: cue.start,
            end: cue.end,
        })
        .collect()
}

#[expect(
    clippy::expect_used,
    reason = "static literal regex is not input-dependent"
)]
fn subtitle_tag_regex() -> &'static Regex {
    static VALUE: OnceLock<Regex> = OnceLock::new();
    VALUE.get_or_init(|| Regex::new(r"<[^>]+>").expect("valid subtitle tag regex"))
}

#[expect(
    clippy::expect_used,
    reason = "static literal regex is not input-dependent"
)]
fn timed_text_regex() -> &'static Regex {
    static VALUE: OnceLock<Regex> = OnceLock::new();
    VALUE
        .get_or_init(|| Regex::new(r#"(?s)<p\b([^>]*)>(.*?)</p>"#).expect("valid timed text regex"))
}

#[expect(
    clippy::expect_used,
    reason = "static literal regex is not input-dependent"
)]
fn timed_text_attribute_regex() -> &'static Regex {
    static VALUE: OnceLock<Regex> = OnceLock::new();
    VALUE.get_or_init(|| {
        Regex::new(r#"\b([td])=['\"](\d+)['\"]"#).expect("valid timed text attribute regex")
    })
}

fn timed_text_boundary_regex() -> &'static Regex {
    static VALUE: OnceLock<Regex> = OnceLock::new();
    VALUE.get_or_init(|| {
        Regex::new(r#"\b(begin|end)=['\"]([^'\"]+)['\"]"#).expect("valid timed text boundary regex")
    })
}

fn decode_subtitle_text(value: &str) -> String {
    subtitle_tag_regex()
        .replace_all(value, "")
        .replace("&amp;", "&")
        .replace("&nbsp;", " ")
        .replace("&#160;", " ")
        .replace("&#xA0;", " ")
        .replace("&#xa0;", " ")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&apos;", "'")
        .trim()
        .to_string()
}

fn parse_timed_text_cues(text: &str) -> Vec<SubtitleCue> {
    timed_text_regex()
        .captures_iter(text)
        .filter_map(|capture| {
            let attributes = capture.get(1)?.as_str();
            let attribute = |name: &str| {
                timed_text_attribute_regex()
                    .captures_iter(attributes)
                    .find_map(|item| {
                        (item.get(1)?.as_str() == name)
                            .then(|| item.get(2)?.as_str().parse::<f64>().ok())
                            .flatten()
                    })
            };
            let boundary = |name: &str| {
                timed_text_boundary_regex()
                    .captures_iter(attributes)
                    .find_map(|item| {
                        (item.get(1)?.as_str() == name)
                            .then(|| parse_timed_text_time(item.get(2)?.as_str()))
                            .flatten()
                    })
            };
            let start = attribute("t")
                .map(|value| value / 1000.0)
                .or_else(|| boundary("begin"))?;
            let end = attribute("d")
                .map(|value| start + value / 1000.0)
                .or_else(|| boundary("end"))?;
            let cue_text = decode_subtitle_text(capture.get(2)?.as_str());
            (!cue_text.is_empty()).then_some(SubtitleCue {
                start,
                end,
                text: cue_text,
            })
        })
        .collect()
}

fn parse_timed_text_time(value: &str) -> Option<f64> {
    let trimmed = value.trim();
    if let Some(seconds) = trimmed.strip_suffix('s') {
        return seconds.parse::<f64>().ok();
    }
    parse_timestamp(trimmed)
}

#[expect(
    clippy::indexing_slicing,
    reason = "line cursor is checked against the collected line count before each access"
)]
pub(crate) fn parse_subtitle_cues_with_text(text: &str) -> Vec<SubtitleCue> {
    let trimmed = text.trim_start();
    if trimmed.starts_with("<?xml") || trimmed.starts_with("<timedtext") {
        return parse_timed_text_cues(text);
    }
    let mut cues = Vec::new();
    let lines = text.lines().collect::<Vec<_>>();
    let mut index = 0;
    while index < lines.len() {
        let line = lines[index].trim();
        if let Some(dialogue) = line.strip_prefix("Dialogue:") {
            let fields = dialogue.splitn(10, ',').collect::<Vec<_>>();
            if let (Some(start), Some(end), Some(text)) =
                (fields.get(1), fields.get(2), fields.get(9))
                && let (Some(start), Some(end)) =
                    (parse_timestamp(start.trim()), parse_timestamp(end.trim()))
                && end > start
            {
                cues.push(SubtitleCue {
                    start,
                    end,
                    text: text
                        .replace("\\N", " ")
                        .replace("{\\", "{")
                        .trim()
                        .to_string(),
                });
            }
            index += 1;
            continue;
        }
        if let Some((start, rest)) = line.split_once("-->") {
            let end = rest.split_whitespace().next().and_then(parse_timestamp);
            let start = parse_timestamp(start.trim());
            index += 1;
            let mut cue_text = Vec::new();
            while index < lines.len() && !lines[index].trim().is_empty() {
                cue_text.push(lines[index].trim());
                index += 1;
            }
            if let (Some(start), Some(end)) = (start, end)
                && end > start
            {
                cues.push(SubtitleCue {
                    start,
                    end,
                    text: decode_subtitle_text(&cue_text.join("\n")),
                });
            }
            continue;
        }
        index += 1;
    }
    cues
}

fn parse_timestamp(value: &str) -> Option<f64> {
    let normalized = value.replace(',', ".");
    let segments = normalized.split(':').collect::<Vec<_>>();
    let seconds = segments.last()?.parse::<f64>().ok()?;
    let minutes = segments
        .iter()
        .rev()
        .nth(1)
        .map(|value| value.parse::<f64>())
        .transpose()
        .ok()?
        .unwrap_or_default();
    let hours = segments
        .iter()
        .rev()
        .nth(2)
        .map(|value| value.parse::<f64>())
        .transpose()
        .ok()?
        .unwrap_or_default();
    Some(hours * 3600.0 + minutes * 60.0 + seconds)
}

fn normalize_intervals(intervals: Vec<Interval>) -> Vec<Interval> {
    intervals
        .into_iter()
        .filter(|item| item.start.is_finite() && item.end.is_finite() && item.end > item.start)
        .collect()
}

fn overlap_score(subtitles: &[Interval], speech: &[Interval], delay: f64) -> f64 {
    let subtitle_duration = subtitles
        .iter()
        .map(|cue| cue.end - cue.start)
        .sum::<f64>()
        .max(0.001);
    let overlap = subtitles
        .iter()
        .map(|cue| {
            let start = cue.start + delay;
            let end = cue.end + delay;
            speech
                .iter()
                .map(|activity| (end.min(activity.end) - start.max(activity.start)).max(0.0))
                .sum::<f64>()
        })
        .sum::<f64>();
    let boundary_alignment = subtitles
        .iter()
        .map(|cue| {
            let time = cue.start + delay;
            let nearest = speech
                .iter()
                .flat_map(|activity| [activity.start, activity.end])
                .map(|boundary| (time - boundary).abs())
                .fold(f64::INFINITY, f64::min);
            (-nearest / 0.5).exp()
        })
        .sum::<f64>()
        / subtitles.len() as f64;
    overlap / subtitle_duration * 0.7 + boundary_alignment * 0.3
}

#[cfg(test)]
mod tests {
    use super::{
        estimate_subtitle_delay_json, parse_subtitle_cues_with_text, speech_intervals_from_energy,
        subtitle_cue_list_json, subtitle_sync_apply_json, subtitle_sync_capture_json,
    };
    use serde_json::Value;

    #[test]
    fn estimates_delay_from_subtitle_and_speech_timelines() {
        let result = estimate_subtitle_delay_json(r#"{
          "subtitleText":"00:00:10,000 --> 00:00:10,800\na\n\n00:00:20,000 --> 00:00:20,800\nb\n\n00:00:30,000 --> 00:00:30,800\nc\n",
          "speechIntervals":[{"start":12.0,"end":12.8},{"start":22.0,"end":22.8},{"start":32.0,"end":32.8}]
        }"#).expect("sync estimate");
        let value: Value = serde_json::from_str(&result).expect("valid result");
        assert!((value["delaySeconds"].as_f64().expect("delay") - 2.0).abs() <= 0.1);
    }

    #[test]
    fn energy_envelope_becomes_speech_intervals() {
        let mut energies = vec![0.002; 1000];
        for range in [100..160, 300..330, 302..310, 600..700, 650..652] {
            for i in range {
                energies[i] = 0.1;
            }
        }
        for i in 330..334 {
            energies[i] = 0.002;
        }
        energies[500] = 0.1;
        let intervals = speech_intervals_from_energy(0.05, &energies);
        let starts = intervals.iter().map(|i| i.start).collect::<Vec<_>>();
        assert_eq!(starts, vec![5.0, 15.0, 30.0]);
        assert!((intervals[1].end - 16.5).abs() < 0.01);
    }

    #[test]
    fn flat_envelope_has_no_speech() {
        assert!(speech_intervals_from_energy(0.05, &[0.01; 500]).is_empty());
    }

    #[test]
    fn parses_ass_dialogue_cues() {
        let result = estimate_subtitle_delay_json(r#"{
          "subtitleText":"[Events]\nDialogue: 0,0:00:10.00,0:00:10.80,Default,,0,0,0,,a\nDialogue: 0,0:00:20.00,0:00:20.80,Default,,0,0,0,,b\nDialogue: 0,0:00:30.00,0:00:30.80,Default,,0,0,0,,c",
          "speechIntervals":[{"start":12.0,"end":12.8},{"start":22.0,"end":22.8},{"start":32.0,"end":32.8}]
        }"#).expect("sync estimate");
        let value: Value = serde_json::from_str(&result).expect("valid result");
        assert!((value["delaySeconds"].as_f64().expect("delay") - 2.0).abs() <= 0.1);
    }

    #[test]
    fn shared_parser_handles_short_vtt_and_timed_text() {
        let vtt = parse_subtitle_cues_with_text(
            "WEBVTT\n\n01.000 --> 02.500\n<b>Hello</b> &amp; world&nbsp;",
        );
        assert_eq!(vtt[0].start, 1.0);
        assert_eq!(vtt[0].text, "Hello & world");
        let timed =
            parse_subtitle_cues_with_text("<timedtext><p d='500' t='1000'>Hi</p></timedtext>");
        assert_eq!(timed[0].end, 1.5);
        let ttml = parse_subtitle_cues_with_text(
            "<?xml version='1.0'?><tt><body><p begin='00:00:02.000' end='2.75s'>Hi</p></body></tt>",
        );
        assert_eq!(ttml[0].start, 2.0);
        assert_eq!(ttml[0].end, 2.75);
    }

    #[test]
    fn exposes_all_cues_and_applies_clicked_cue_delay() {
        let captured = subtitle_sync_capture_json(r#"{
          "subtitleText":"1\n00:00:10,000 --> 00:00:11,000\nMerhaba\n\n2\n00:00:20,000 --> 00:00:21,000\nNasılsın?",
          "currentTime":11.6
        }"#).expect("capture");
        let captured: Value = serde_json::from_str(&captured).expect("valid capture");
        assert_eq!(captured["cues"].as_array().map(Vec::len), Some(2));
        assert_eq!(captured["cues"][0]["text"], "Merhaba");

        let applied = subtitle_sync_apply_json(&format!(
            r#"{{"capturedTime":{},"cueStart":{}}}"#,
            captured["capturedTime"], captured["cues"][0]["start"]
        ))
        .expect("apply");
        let applied: Value = serde_json::from_str(&applied).expect("valid apply");
        assert_eq!(applied["delaySeconds"], 1.6);
    }

    #[test]
    fn cue_list_is_available_without_a_playback_time() {
        let result =
            subtitle_cue_list_json(r#"{"subtitleText":"00:00:01,000 --> 00:00:02,000\nHello"}"#)
                .expect("cue list");
        let value: Value = serde_json::from_str(&result).expect("valid cue list");
        assert_eq!(value["cues"][0]["start"], 1.0);
    }
}
