use std::time::{Duration, Instant};

use serde_json::{Value, json};

use super::{PlayerSession, VideoBackend, VideoCommand};
use crate::trailer::{TrailerSource, range_proxy, trailer_sources};
use crate::{Gpu, SessionHandle};

const CLIENT_TRIES: usize = 4;
const FIRST_FRAME_TIMEOUT: Duration = Duration::from_secs(12);

pub(super) struct TrailerPlay {
    sources: Vec<TrailerSource>,
    next: usize,
    request: Option<String>,
    requests: u64,
    loaded: bool,
    loaded_at: Option<Instant>,
    audio: Option<String>,
    subtitle: Option<String>,
    client: Option<usize>,
    client_tries: usize,
    next_client: Option<usize>,
}

impl TrailerPlay {
    pub(super) fn new(urls: &[String]) -> Self {
        Self {
            sources: trailer_sources(urls),
            next: 0,
            request: None,
            requests: 0,
            loaded: false,
            loaded_at: None,
            audio: None,
            subtitle: None,
            client: None,
            client_tries: 0,
            next_client: None,
        }
    }
}

fn playback_failed(trailer: &mut TrailerPlay) {
    trailer.loaded = false;
    trailer.loaded_at = None;
    trailer.audio = None;
    trailer.subtitle = None;
    let youtube = matches!(
        trailer.sources.get(trailer.next),
        Some(TrailerSource::YouTube(_))
    );
    if youtube && trailer.client_tries + 1 < CLIENT_TRIES {
        trailer.client_tries += 1;
        trailer.next_client = Some(trailer.client.map_or(0, |index| index + 1));
    } else {
        trailer.client_tries = 0;
        trailer.next_client = None;
        trailer.next += 1;
    }
}

pub(super) fn tick(
    player: &mut PlayerSession,
    session: &SessionHandle,
    video: &mut Option<Box<dyn VideoBackend>>,
    gpu: &Option<Gpu>,
    snapshot: &Value,
) {
    let Some(trailer) = player.trailer.as_mut() else {
        return;
    };
    let (Some(video), Some(gpu)) = (video.as_mut(), gpu.as_ref()) else {
        return;
    };
    if trailer.loaded {
        if video.status().error.is_some() {
            video.stop();
            playback_failed(trailer);
            return;
        }
        if player.status.has_frame {
            trailer.loaded_at = None;
            trailer.client_tries = 0;
            for (command, url) in [
                ("audio-add", trailer.audio.take()),
                ("sub-add", trailer.subtitle.take()),
            ] {
                if let Some(url) = url {
                    video.command(VideoCommand::Mpv(vec![
                        command.to_owned(),
                        url,
                        "select".to_owned(),
                    ]));
                }
            }
        } else if trailer
            .loaded_at
            .is_some_and(|at| at.elapsed() > FIRST_FRAME_TIMEOUT)
        {
            video.stop();
            playback_failed(trailer);
        }
        return;
    }
    if let Some(request) = trailer.request.as_deref() {
        let Some(resolution) = snapshot
            .pointer("/trailer/resolutions")
            .and_then(|resolutions| resolutions.get(request))
        else {
            return;
        };
        trailer.request = None;
        let url = resolution
            .get("streamUrl")
            .and_then(Value::as_str)
            .filter(|_| resolution.get("status").and_then(Value::as_str) == Some("ok"));
        match url {
            Some(url) => {
                let proxied = range_proxy(url);
                video.load(
                    &gpu.instance,
                    &gpu.device,
                    proxied.as_deref().unwrap_or(url),
                );
                trailer.audio = resolution
                    .get("audioUrl")
                    .and_then(Value::as_str)
                    .map(|url| range_proxy(url).unwrap_or_else(|| url.to_owned()));
                trailer.subtitle =
                    crate::trailer::subtitle_url(resolution, snapshot.pointer("/profile/active"));
                trailer.client = resolution
                    .get("clientIndex")
                    .and_then(Value::as_u64)
                    .map(|index| index as usize);
                trailer.loaded = true;
                trailer.loaded_at = Some(Instant::now());
            }
            None => {
                trailer.client_tries = 0;
                trailer.next += 1;
            }
        }
        return;
    }
    if session.has_queued_dispatches() {
        return;
    }
    let Some(source) = trailer.sources.get(trailer.next).cloned() else {
        player.error = Some(fluxa_ui::localized(
            "detail.trailer_unavailable",
            &player.language,
        ));
        return;
    };
    let video_id = match source {
        TrailerSource::Direct(url) => {
            let proxied = range_proxy(&url);
            video.load(
                &gpu.instance,
                &gpu.device,
                proxied.as_deref().unwrap_or(url.as_str()),
            );
            trailer.loaded = true;
            trailer.loaded_at = Some(Instant::now());
            return;
        }
        TrailerSource::YouTube(id) => id,
    };
    trailer.requests += 1;
    let request_id = format!("player-trailer-{}", trailer.requests);
    let mut action = json!({
        "type": "trailerResolveRequested",
        "requestId": request_id,
        "videoId": video_id,
        "maxHeight": 1080,
    });
    if let Some(index) = trailer.next_client.take() {
        action["clientIndex"] = json!(index);
    }
    match session.dispatch(action) {
        Ok(()) => trailer.request = Some(request_id),
        Err(error) => player.error = Some(error),
    }
}
