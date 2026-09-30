use crate::headless_engine::HeadlessEngine;
use crate::player::subtitles::subtitle_sync::{
    Interval, SubtitleCue, align_subtitle_text, delay_for_cue, intervals_from_text,
    parse_subtitle_cues_with_text, retime, speech_intervals_from_energy,
};
use crate::runtime::EffectEnvelope;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", default)]
pub(crate) struct SubtitleSyncState {
    pub(crate) delay_seconds: f64,
    pub(crate) scale: f64,
    pub(crate) captured_time: Option<f64>,
    pub(crate) cues: Vec<SubtitleCue>,
    pub(crate) confidence: Option<f64>,
    pub(crate) retimed_cues: Vec<SubtitleCue>,
}

impl Default for SubtitleSyncState {
    fn default() -> Self {
        Self {
            delay_seconds: 0.0,
            scale: 1.0,
            captured_time: None,
            cues: Vec::new(),
            confidence: None,
            retimed_cues: Vec::new(),
        }
    }
}

pub(crate) fn dispatch_capture(
    engine: &mut HeadlessEngine,
    subtitle_text: String,
    current_time: f64,
) -> Vec<EffectEnvelope> {
    if current_time.is_finite() {
        let sync = &mut engine.state.player.subtitle_sync;
        sync.captured_time = Some(current_time);
        sync.cues = parse_subtitle_cues_with_text(&subtitle_text);
    }
    vec![]
}

pub(crate) fn dispatch_cue_selected(
    engine: &mut HeadlessEngine,
    cue_start: f64,
) -> Vec<EffectEnvelope> {
    let sync = &mut engine.state.player.subtitle_sync;
    if let Some(delay) = sync
        .captured_time
        .and_then(|captured| delay_for_cue(captured, cue_start))
    {
        sync.delay_seconds = delay;
        sync.scale = 1.0;
        sync.confidence = None;
        sync.retimed_cues.clear();
        sync.captured_time = None;
        sync.cues.clear();
    }
    vec![]
}

pub(crate) fn dispatch_estimate(
    engine: &mut HeadlessEngine,
    subtitle_text: String,
    reference_text: Option<String>,
    speech_intervals: Vec<Interval>,
) -> Vec<EffectEnvelope> {
    let reference = match reference_text {
        Some(text) => intervals_from_text(&text),
        None => speech_intervals,
    };
    if let Some(alignment) = align_subtitle_text(&subtitle_text, reference) {
        let sync = &mut engine.state.player.subtitle_sync;
        sync.delay_seconds = alignment.delay_seconds;
        sync.scale = alignment.scale;
        sync.confidence = Some(alignment.confidence);
        sync.retimed_cues = if alignment.scale == 1.0 {
            Vec::new()
        } else {
            retime(&parse_subtitle_cues_with_text(&subtitle_text), &alignment)
        };
    }
    vec![]
}

pub(crate) fn dispatch_audio_estimate(
    engine: &mut HeadlessEngine,
    subtitle_text: String,
    frame_seconds: f64,
    energies: Vec<f64>,
) -> Vec<EffectEnvelope> {
    let speech_intervals = speech_intervals_from_energy(frame_seconds, &energies);
    dispatch_estimate(engine, subtitle_text, None, speech_intervals)
}
