use super::*;
use std::sync::mpsc::{Receiver, TryRecvError};

const WINDOW_SECONDS: f64 = 20.0;
const POSITIONS: [f64; 5] = [45.0, 200.0, 420.0, 700.0, 980.0];
const RATE_SPREAD: f64 = 0.07;
const MARGIN_SECONDS: f64 = 12.0;
const SETTING_KEY: &str = "externalAudioSource";

#[derive(Default)]
pub(super) struct ExternalAudio {
    job: Option<Job>,
    pub(super) message: Option<(&'static str, Option<String>)>,
    pub(super) active: bool,
    pub(super) label: Option<String>,
}

struct Job {
    stream: Receiver<Option<Value>>,
    url: Option<String>,
    queue: Vec<f64>,
    windows: Vec<Value>,
    current: Option<Current>,
    label: String,
}

struct Current {
    reference: Receiver<Option<SpeechEnergy>>,
    reference_energy: Option<SpeechEnergy>,
    external: Option<Receiver<Option<SpeechEnergy>>>,
    external_energy: Option<SpeechEnergy>,
    start: f64,
}

fn say(player: &mut PlayerSession, key: &'static str, arg: Option<String>) {
    player.external_audio.message = Some((key, arg));
}

pub(super) fn source_label(state: &RendererState) -> Option<String> {
    let (options, current, label) = fluxa_ui::account_source_state(&state.settings, SETTING_KEY);
    (!current.is_empty() && options.iter().any(|(value, _)| *value == current)).then_some(label)
}

pub(super) fn begin(state: &mut RendererState) {
    let source = state
        .settings
        .str_value(SETTING_KEY)
        .filter(|source| !source.is_empty())
        .map(str::to_owned);
    let label = source_label(state);
    let (Some(source), Some(label)) = (source, label) else {
        return;
    };
    let Some(player) = state.player.as_mut() else {
        return;
    };
    if player.external_audio.job.is_some() || player.loaded_url.is_none() {
        return;
    }
    let Some(content_type) = player
        .meta
        .get("type")
        .and_then(Value::as_str)
        .map(str::to_owned)
    else {
        return;
    };
    let id = player
        .meta
        .get("lastVideoId")
        .and_then(Value::as_str)
        .filter(|id| !id.is_empty())
        .or_else(|| player.meta.get("id").and_then(Value::as_str))
        .map(str::to_owned);
    let (Some(id), Some(session)) = (id, state.session.as_ref()) else {
        return;
    };
    let stream = session
        .executor()
        .fetch_hero_stream(id, content_type, source);
    let Some(player) = state.player.as_mut() else {
        return;
    };
    player.external_audio.job = Some(Job {
        stream,
        url: None,
        queue: Vec::new(),
        windows: Vec::new(),
        current: None,
        label,
    });
    say(player, "player.external_audio_finding", None);
}

fn failed(state: &mut RendererState, key: &'static str) {
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
    if state
        .player
        .as_ref()
        .is_some_and(|player| matches!(player.panel, Some(Panel::Menu(menu::Menu::Tracks))))
    {
        let label = source_label(state);
        if let Some(player) = state.player.as_mut() {
            player.external_audio.label = label;
        }
    }
    let Some(mut job) = state
        .player
        .as_mut()
        .and_then(|player| player.external_audio.job.take())
    else {
        return;
    };
    if advance(state, &mut job)
        && let Some(player) = state.player.as_mut()
    {
        player.external_audio.job = Some(job);
    }
}

fn advance(state: &mut RendererState, job: &mut Job) -> bool {
    if job.url.is_none() {
        let stream = match poll(&job.stream) {
            Ok(Some(stream)) => stream,
            Ok(None) => return true,
            Err(()) => {
                failed(state, "player.external_audio_not_found");
                return false;
            }
        };
        let url = state
            .player
            .as_ref()
            .and_then(|player| resolve::playback_url(&stream, &player.meta));
        let Some(url) = url else {
            failed(state, "player.external_audio_not_found");
            return false;
        };
        let duration = state.player.as_ref().map_or(0.0, |p| p.status.duration);
        job.queue = POSITIONS
            .into_iter()
            .filter(|start| start + WINDOW_SECONDS + 5.0 < duration)
            .rev()
            .collect();
        if job.queue.len() < 2 {
            job.queue = vec![(duration * 0.1).max(5.0)];
        }
        job.url = Some(url);
        if let Some(player) = state.player.as_mut() {
            say(player, "player.external_audio_analyzing", None);
        }
    }
    if job.current.is_none() {
        let Some(start) = job.queue.pop() else {
            return finish(state, job);
        };
        let Some(url) = state.player.as_ref().and_then(|p| p.loaded_url.clone()) else {
            return false;
        };
        let reference = state
            .video
            .as_mut()
            .and_then(|video| video.analyze_audio(&url, start, WINDOW_SECONDS));
        let Some(reference) = reference else {
            failed(state, "player.external_audio_unavailable");
            return false;
        };
        job.current = Some(Current {
            reference,
            reference_energy: None,
            external: None,
            external_energy: None,
            start,
        });
        return true;
    }
    let Some(current) = job.current.as_mut() else {
        return true;
    };
    if current.reference_energy.is_none() {
        match poll(&current.reference) {
            Ok(energy) => current.reference_energy = energy,
            Err(()) => {
                failed(state, "player.external_audio_failed");
                return false;
            }
        }
    }
    if current.reference_energy.is_some() && current.external.is_none() {
        let low = (current.start * (1.0 - RATE_SPREAD) - MARGIN_SECONDS).max(0.0);
        let high = (current.start + WINDOW_SECONDS) * (1.0 + RATE_SPREAD) + MARGIN_SECONDS;
        let url = job.url.clone().unwrap_or_default();
        let external = state
            .video
            .as_mut()
            .and_then(|video| video.analyze_audio(&url, low, high - low));
        match external {
            Some(receiver) => current.external = Some(receiver),
            None => {
                failed(state, "player.external_audio_unavailable");
                return false;
            }
        }
    }
    if let Some(receiver) = current.external.as_ref()
        && current.external_energy.is_none()
    {
        match poll(receiver) {
            Ok(energy) => current.external_energy = energy,
            Err(()) => {
                failed(state, "player.external_audio_failed");
                return false;
            }
        }
    }
    if let (Some(reference), Some(external)) = (
        current.reference_energy.as_ref(),
        current.external_energy.as_ref(),
    ) {
        job.windows.push(json!({
            "reference": energy_json(reference),
            "external": energy_json(external),
        }));
        job.current = None;
    }
    true
}

fn energy_json(energy: &SpeechEnergy) -> Value {
    json!({
        "start": energy.start,
        "frameSeconds": energy.frame_seconds,
        "energies": energy.energies,
    })
}

fn finish(state: &mut RendererState, job: &Job) -> bool {
    let found = core_value("audioSyncEstimate", json!({"windows": job.windows}));
    let Some((rate, offset)) = found
        .as_ref()
        .and_then(|sync| Some((sync["rate"].as_f64()?, sync["offset"].as_f64()?)))
    else {
        failed(state, "player.external_audio_no_match");
        return false;
    };
    let Some(url) = job.url.clone() else {
        return false;
    };
    host_log(format!(
        "external audio sync: rate {rate:.5} offset {offset:.2} from {} windows",
        job.windows.len()
    ));
    command(
        state,
        VideoCommand::Mpv(vec![
            "audio-add".to_owned(),
            url,
            "select".to_owned(),
            job.label.clone(),
            String::new(),
            format!("{rate:.9}"),
            format!("{offset:.3}"),
        ]),
    );
    if let Some(player) = state.player.as_mut() {
        player.external_audio.active = true;
        say(
            player,
            "player.external_audio_applied",
            Some(job.label.clone()),
        );
    }
    false
}
