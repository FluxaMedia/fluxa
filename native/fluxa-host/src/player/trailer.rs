use serde_json::{Value, json};

use super::{PlayerSession, VideoBackend, VideoCommand};
use crate::trailer::{TrailerSource, trailer_sources};
use crate::{Gpu, SessionHandle};

pub(super) struct TrailerPlay {
    sources: Vec<TrailerSource>,
    next: usize,
    request: Option<String>,
    requests: u64,
    loaded: bool,
    audio: Option<String>,
    subtitle: Option<String>,
}

impl TrailerPlay {
    pub(super) fn new(urls: &[String]) -> Self {
        Self {
            sources: trailer_sources(urls),
            next: 0,
            request: None,
            requests: 0,
            loaded: false,
            audio: None,
            subtitle: None,
        }
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
            trailer.loaded = false;
            trailer.next += 1;
            return;
        }
        if player.status.has_frame {
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
                video.load(&gpu.instance, &gpu.device, url);
                trailer.audio = resolution
                    .get("audioUrl")
                    .and_then(Value::as_str)
                    .map(ToOwned::to_owned);
                trailer.subtitle =
                    crate::trailer::subtitle_url(resolution, snapshot.pointer("/profile/active"));
                trailer.loaded = true;
            }
            None => trailer.next += 1,
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
            video.load(&gpu.instance, &gpu.device, &url);
            trailer.loaded = true;
            return;
        }
        TrailerSource::YouTube(id) => id,
    };
    trailer.requests += 1;
    let request_id = format!("player-trailer-{}", trailer.requests);
    let action = json!({
        "type": "trailerResolveRequested",
        "requestId": request_id,
        "videoId": video_id,
        "maxHeight": 1080,
    });
    match session.dispatch(action) {
        Ok(()) => trailer.request = Some(request_id),
        Err(error) => player.error = Some(error),
    }
}
