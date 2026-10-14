use super::menu::{self, Menu};
use super::*;
use std::sync::mpsc::{Receiver, TryRecvError};

const ANALYSIS_SECONDS: f64 = 240.0;
const CUE_BEFORE: f64 = 30.0;
const CUE_AFTER: f64 = 10.0;
const CUE_LIMIT: usize = 40;
const REFERENCE_LIMIT: usize = 6;

#[derive(Default)]
pub(super) struct SyncState {
    job: Option<Job>,
    pub(super) message: Option<(&'static str, Option<String>)>,
    pub(super) captured: f64,
    pub(super) cues: Vec<(f64, String)>,
}

struct Job {
    kind: Kind,
    subtitle: Receiver<Option<String>>,
    subtitle_text: Option<String>,
    reference: Option<Receiver<Option<String>>>,
    reference_text: Option<String>,
    audio: Option<Receiver<Option<SpeechEnergy>>>,
    start: f64,
}

#[derive(Clone, Copy, PartialEq)]
pub(super) enum Kind {
    Manual,
    Reference(usize),
    Audio,
}

enum Step {
    Wait,
    Done,
}

fn active_url(player: &PlayerSession) -> Option<String> {
    let track = player
        .tracks
        .iter()
        .find(|track| track.subtitle && track.selected && track.external)?;
    let listed = &player.overlay.addon_subtitles;
    listed
        .iter()
        .find(|entry| entry["label"].as_str() == track.title.as_deref())
        .or_else(|| (listed.len() == 1).then(|| &listed[0]))?["url"]
        .as_str()
        .map(str::to_owned)
}

pub(super) fn references(player: &PlayerSession) -> Vec<(usize, String)> {
    let active = active_url(player);
    player
        .overlay
        .addon_subtitles
        .iter()
        .enumerate()
        .filter(|(_, entry)| {
            entry["url"].as_str().is_some() && entry["url"].as_str() != active.as_deref()
        })
        .take(REFERENCE_LIMIT)
        .map(|(index, entry)| {
            let label = entry["label"].as_str().unwrap_or("Subtitle");
            let label = match entry["lang"].as_str() {
                Some(lang) => format!("{label} ({lang})"),
                None => label.to_owned(),
            };
            (index, label)
        })
        .collect()
}

fn say(player: &mut PlayerSession, key: &'static str, arg: Option<String>) {
    player.sync.message = Some((key, arg));
}

pub(super) fn begin(state: &mut RendererState, kind: Kind) {
    if let (Some(video), Some(player)) = (state.video.as_mut(), state.player.as_mut()) {
        player.tracks = video.tracks();
    }
    let Some(player) = state.player.as_mut() else {
        return;
    };
    let Some(url) = active_url(player) else {
        say(player, "player.sync_unavailable", None);
        return;
    };
    let reference = match kind {
        Kind::Reference(index) => player
            .overlay
            .addon_subtitles
            .get(index)
            .and_then(|entry| entry["url"].as_str())
            .map(crate::trailer::fetch_text),
        _ => None,
    };
    if matches!(kind, Kind::Reference(_)) && reference.is_none() {
        say(player, "player.sync_unavailable", None);
        return;
    }
    let duration = player.status.duration;
    let start = player
        .status
        .position
        .min((duration - ANALYSIS_SECONDS - 5.0).max(0.0))
        .max(0.0);
    player.sync.captured = player.status.position;
    player.sync.job = Some(Job {
        kind,
        subtitle: crate::trailer::fetch_text(&url),
        subtitle_text: None,
        reference,
        reference_text: None,
        audio: None,
        start,
    });
    say(player, "player.sync_loading", None);
}

pub(super) fn reset(state: &mut RendererState) {
    if let Some(player) = state.player.as_mut() {
        player.subtitle_delay = 0.0;
        player.sync.job = None;
        say(player, "player.sync_reset_done", None);
    }
    menu::mpv(state, "sub-delay", "0");
    menu::mpv(state, "sub-speed", "1");
}

pub(super) fn pick_cue(state: &mut RendererState, index: usize) {
    let Some(player) = state.player.as_ref() else {
        return;
    };
    let Some((start, _)) = player.sync.cues.get(index) else {
        return;
    };
    let plan = core_value(
        "subtitleSyncApply",
        json!({"capturedTime": player.sync.captured, "cueStart": start}),
    );
    match plan.and_then(|plan| plan["delaySeconds"].as_f64()) {
        Some(delay) => apply(state, delay, 1.0),
        None => {
            if let Some(player) = state.player.as_mut() {
                say(player, "player.sync_failed", None);
                player.panel = Some(Panel::Menu(Menu::Sync));
            }
        }
    }
}

fn apply(state: &mut RendererState, delay: f64, scale: f64) {
    if let Some(player) = state.player.as_mut() {
        player.subtitle_delay = delay;
        say(player, "player.sync_applied", Some(format!("{delay:+.1}s")));
        player.panel = Some(Panel::Menu(Menu::Sync));
    }
    menu::mpv(state, "sub-delay", &delay.to_string());
    let scale = if (scale - 1.0).abs() < 0.0005 {
        1.0
    } else {
        scale
    };
    menu::mpv(state, "sub-speed", &scale.to_string());
}

fn fail(state: &mut RendererState, key: &'static str) {
    if let Some(player) = state.player.as_mut() {
        say(player, key, None);
    }
}

fn poll<T>(receiver: &Receiver<Option<T>>) -> Result<Option<T>, ()> {
    match receiver.try_recv() {
        Ok(Some(value)) => Ok(Some(value)),
        Ok(None) | Err(TryRecvError::Disconnected) => Err(()),
        Err(TryRecvError::Empty) => Ok(None),
    }
}

pub(super) fn tick(state: &mut RendererState) {
    let Some(mut job) = state
        .player
        .as_mut()
        .and_then(|player| player.sync.job.take())
    else {
        return;
    };
    if let Step::Wait = advance(state, &mut job)
        && let Some(player) = state.player.as_mut()
    {
        player.sync.job = Some(job);
    }
}

fn advance(state: &mut RendererState, job: &mut Job) -> Step {
    if job.subtitle_text.is_none() {
        match poll(&job.subtitle) {
            Ok(text) => job.subtitle_text = text,
            Err(()) => {
                fail(state, "player.sync_failed");
                return Step::Done;
            }
        }
    }
    if let (Some(reference), None) = (job.reference.as_ref(), job.reference_text.as_ref()) {
        match poll(reference) {
            Ok(text) => job.reference_text = text,
            Err(()) => {
                fail(state, "player.sync_failed");
                return Step::Done;
            }
        }
    }
    let Some(text) = job.subtitle_text.clone() else {
        return Step::Wait;
    };
    match job.kind {
        Kind::Manual => manual(state, &text),
        Kind::Reference(_) => match job.reference_text.as_deref() {
            Some(reference) => estimate(
                state,
                json!({"subtitleText": text, "referenceText": reference}),
            ),
            None => Step::Wait,
        },
        Kind::Audio => audio(state, job, text),
    }
}

fn manual(state: &mut RendererState, text: &str) -> Step {
    let Some(player) = state.player.as_mut() else {
        return Step::Done;
    };
    let captured = player.sync.captured;
    let plan = core_value(
        "subtitleSyncCapture",
        json!({"subtitleText": text, "currentTime": captured}),
    );
    let cues: Vec<(f64, String)> = plan
        .as_ref()
        .and_then(|plan| plan["cues"].as_array())
        .into_iter()
        .flatten()
        .filter_map(|cue| Some((cue["start"].as_f64()?, cue["text"].as_str()?.to_owned())))
        .filter(|(start, _)| (captured - CUE_BEFORE..=captured + CUE_AFTER).contains(start))
        .take(CUE_LIMIT)
        .collect();
    if cues.is_empty() {
        say(player, "player.sync_failed", None);
        return Step::Done;
    }
    player.sync.cues = cues;
    player.panel = Some(Panel::Menu(Menu::SyncCues));
    Step::Done
}

fn audio(state: &mut RendererState, job: &mut Job, text: String) -> Step {
    if let Some(receiver) = job.audio.as_ref() {
        return match poll(receiver) {
            Ok(None) => Step::Wait,
            Ok(Some(energy)) => estimate(
                state,
                json!({
                    "subtitleText": text,
                    "frameSeconds": energy.frame_seconds,
                    "energies": energy.energies,
                    "energyStart": energy.start,
                }),
            ),
            Err(()) => {
                fail(state, "player.sync_failed");
                Step::Done
            }
        };
    }
    let url = state
        .player
        .as_ref()
        .and_then(|player| player.loaded_url.clone());
    let receiver = url.and_then(|url| {
        state
            .video
            .as_mut()
            .and_then(|video| video.analyze_audio(&url, job.start, ANALYSIS_SECONDS))
    });
    match receiver {
        Some(receiver) => {
            job.audio = Some(receiver);
            if let Some(player) = state.player.as_mut() {
                say(player, "player.sync_analyzing", None);
            }
            Step::Wait
        }
        None => {
            fail(state, "player.sync_unavailable_audio");
            Step::Done
        }
    }
}

fn estimate(state: &mut RendererState, request: Value) -> Step {
    let found = core_value("subtitleSyncEstimate", request)
        .and_then(|plan| Some((plan["delaySeconds"].as_f64()?, plan["scale"].as_f64()?)));
    match found {
        Some((delay, scale)) => apply(state, delay, scale),
        None => fail(state, "player.sync_failed"),
    }
    Step::Done
}
