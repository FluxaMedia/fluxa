use std::sync::{Arc, mpsc::Receiver};
use std::time::Duration;

use fluxa_ui::HeroTrailer;
use serde_json::{Map, Value, json};
use web_time::Instant;

use crate::{RendererState, Route, core_value, profile_language};

#[derive(Default)]
pub(crate) struct Trailers {
    fetched: Map<String, Value>,
    fetched_ids: Vec<Value>,
    fetching: Option<Receiver<Vec<(String, Value)>>>,
    targets: Vec<Value>,
    active: Option<Active>,
    requests: u64,
}

struct Active {
    key: String,
    urls: Vec<String>,
    ids: Vec<String>,
    next: usize,
    due: Instant,
    request: Option<String>,
    loaded: bool,
    failed: bool,
    finished: bool,
    last_position: f64,
    subtitle: Option<Receiver<Option<String>>>,
    cues: Vec<Cue>,
    cue: Option<String>,
    texture: Option<egui::TextureId>,
}

impl Trailers {
    pub(crate) fn inputs(&self) -> Arc<Value> {
        Arc::new(json!({"fetched": self.fetched, "ids": self.fetched_ids}))
    }

    fn fetched_urls(&self, id: &str) -> Vec<String> {
        self.fetched
            .get(id)
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(|trailer| {
                ["url", "link"]
                    .into_iter()
                    .find_map(|key| trailer.get(key).and_then(Value::as_str))
            })
            .take(6)
            .map(ToOwned::to_owned)
            .collect()
    }

    pub(crate) fn set_targets(&mut self, targets: Vec<Value>) {
        self.targets = targets
            .into_iter()
            .filter(|target| {
                target
                    .get("id")
                    .is_some_and(|id| !self.fetched_ids.contains(id))
            })
            .collect();
    }
}

struct Wanted {
    key: String,
    urls: Vec<String>,
    delay: f64,
}

fn wanted(state: &RendererState) -> Option<Wanted> {
    if state.player.is_some() {
        return None;
    }
    let settings = &state.settings;
    let delay = |key: &str| settings.number_value(key).unwrap_or(2.0).max(0.0);
    match state.route {
        Route::Home
            if settings.bool_value("homeHeroAutoplayTrailer") && state.home.show_hero_section =>
        {
            let context = &state.gpu.as_ref()?.egui_context;
            let hero = fluxa_ui::active_home_hero(context, &state.home)?;
            Some(Wanted {
                key: hero.item_id.clone()?,
                urls: hero.trailers.clone(),
                delay: delay("homeHeroAutoplayTrailerDelaySecs"),
            })
        }
        Route::Shorts => {
            let hero = state.shorts.items.get(state.shorts.index)?;
            let id = hero.item_id.clone()?;
            let urls = if hero.trailers.is_empty() {
                state.trailers.fetched_urls(&id)
            } else {
                hero.trailers.clone()
            };
            Some(Wanted {
                key: id,
                urls,
                delay: 0.3,
            })
        }
        Route::Detail
            if settings.bool_value("detailHeroAutoplayTrailer") && !state.detail.id.is_empty() =>
        {
            Some(Wanted {
                key: state.detail.id.clone(),
                urls: state.detail.trailers.clone(),
                delay: delay("detailHeroAutoplayTrailerDelaySecs"),
            })
        }
        _ => None,
    }
}

fn fetch(state: &mut RendererState) {
    let trailers = &mut state.trailers;
    if let Some(receiver) = trailers.fetching.as_ref() {
        let Ok(found) = receiver.try_recv() else {
            return;
        };
        trailers.fetching = None;
        for (id, list) in found {
            if list.as_array().is_some_and(|items| !items.is_empty()) {
                trailers.fetched.insert(id.clone(), list);
            }
            trailers.fetched_ids.push(Value::String(id));
        }
        state.last_snapshot_revision = None;
        return;
    }
    if trailers.targets.is_empty() {
        return;
    }
    let Some(session) = state.session.as_ref() else {
        return;
    };
    let api_key = state
        .settings
        .values
        .get("tmdbApiKey")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_owned();
    let language = profile_language(
        session
            .snapshot()
            .pointer("/profile/active")
            .unwrap_or(&Value::Null),
    );
    let targets = std::mem::take(&mut trailers.targets);
    trailers.fetching = Some(
        session
            .executor()
            .fetch_trailers(targets, api_key, language),
    );
}

fn reset(state: &mut RendererState) {
    let Some(active) = state.trailers.active.take() else {
        return;
    };
    if active.loaded
        && state.player.is_none()
        && let Some(video) = state.video.as_mut()
    {
        video.stop();
    }
    if let (Some(texture), Some(gpu)) = (active.texture, state.gpu.as_mut()) {
        gpu.egui_renderer.free_texture(&texture);
    }
}

pub(crate) fn subtitle_url(resolution: &Value, profile: Option<&Value>) -> Option<String> {
    let profile = profile.unwrap_or(&Value::Null);
    if profile.get("autoEnableSubtitles").and_then(Value::as_bool) == Some(false) {
        return None;
    }
    let tracks = resolution
        .get("subtitles")?
        .as_array()
        .filter(|tracks| !tracks.is_empty())?;
    let language = profile_language(profile);
    let track = core_value(
        "trailerSubtitleSelectionPlan",
        json!({
            "tracks": tracks,
            "preferred": profile.get("preferredSubtitleLanguage"),
            "secondary": profile.get("secondarySubtitleLanguage"),
            "systemLanguage": language,
        }),
    )
    .filter(|track| !track.is_null())?;
    core_value(
        "normalizeTrailerSubtitleUrl",
        json!({"url": track.get("url")?}),
    )?
    .as_str()
    .map(ToOwned::to_owned)
}

fn youtube_ids(urls: &[String]) -> Vec<String> {
    if urls.is_empty() {
        return Vec::new();
    }
    core_value("trailerYoutubeVideoIds", json!({"urls": urls}))
        .and_then(|ids| {
            ids.as_array().map(|ids| {
                ids.iter()
                    .filter_map(Value::as_str)
                    .map(ToOwned::to_owned)
                    .collect()
            })
        })
        .unwrap_or_default()
}

fn queue_shorts(state: &mut RendererState) {
    if state.route != Route::Shorts {
        return;
    }
    let trailers = &mut state.trailers;
    for hero in state.shorts.items.iter().skip(state.shorts.index).take(3) {
        let Some(id) = hero.item_id.as_deref() else {
            continue;
        };
        let id = Value::String(id.to_owned());
        let queued = trailers
            .targets
            .iter()
            .any(|target| target.get("id") == Some(&id));
        if hero.trailers.is_empty() && !queued && !trailers.fetched_ids.contains(&id) {
            trailers.targets.push(hero.raw.clone());
        }
    }
}

pub(crate) fn tick(state: &mut RendererState) {
    queue_shorts(state);
    fetch(state);
    let wanted = wanted(state);
    let current = state
        .trailers
        .active
        .as_ref()
        .map(|active| active.key.as_str());
    if current != wanted.as_ref().map(|wanted| wanted.key.as_str()) {
        reset(state);
        if let Some(wanted) = wanted {
            state.trailers.active = Some(Active {
                ids: youtube_ids(&wanted.urls),
                urls: wanted.urls,
                key: wanted.key,
                next: 0,
                due: Instant::now() + Duration::from_secs_f64(wanted.delay),
                request: None,
                loaded: false,
                failed: false,
                finished: false,
                last_position: 0.0,
                subtitle: None,
                cues: Vec::new(),
                cue: None,
                texture: None,
            });
        }
    } else if let (Some(active), Some(wanted)) = (state.trailers.active.as_mut(), wanted)
        && !active.loaded
        && active.request.is_none()
        && active.urls != wanted.urls
    {
        active.ids = youtube_ids(&wanted.urls);
        active.urls = wanted.urls;
        active.next = 0;
        active.failed = false;
    }
    resolve(state);
    play(state);
    publish(state);
}

fn resolve(state: &mut RendererState) {
    let RendererState {
        trailers,
        session,
        video,
        gpu,
        ..
    } = state;
    let (Some(active), Some(session)) = (trailers.active.as_mut(), session.as_ref()) else {
        return;
    };
    if active.loaded || active.failed || Instant::now() < active.due {
        return;
    }
    if let Some(request) = active.request.as_deref() {
        let snapshot = session.snapshot();
        let Some(resolution) = snapshot
            .get("trailer")
            .and_then(|trailer| trailer.get("resolutions"))
            .and_then(|resolutions| resolutions.get(request))
        else {
            return;
        };
        active.request = None;
        let url = resolution
            .get("streamUrl")
            .and_then(Value::as_str)
            .filter(|_| resolution.get("status").and_then(Value::as_str) == Some("ok"));
        match (url, video.as_mut(), gpu.as_ref()) {
            (Some(url), Some(video), Some(gpu)) => {
                let proxied = fluxa_effects::range_proxy(url);
                video.load_preview(&gpu.instance, &gpu.device, proxied.as_deref().unwrap_or(url));
                active.subtitle = subtitle_url(resolution, snapshot.pointer("/profile/active"))
                    .map(|url| fluxa_effects::fetch_text(&url));
                active.loaded = true;
            }
            (Some(_), _, _) => active.failed = true,
            (None, _, _) => active.next += 1,
        }
        return;
    }
    if session.has_queued_dispatches() {
        return;
    }
    let Some(video_id) = active.ids.get(active.next) else {
        active.failed = true;
        return;
    };
    trailers.requests += 1;
    let request_id = format!("native-trailer-{}", trailers.requests);
    let action = json!({
        "type": "trailerResolveRequested",
        "requestId": request_id,
        "videoId": video_id,
        "maxHeight": 1080,
    });
    match session.dispatch(action) {
        Ok(()) => active.request = Some(request_id),
        Err(_) => active.failed = true,
    }
}

fn play(state: &mut RendererState) {
    let RendererState {
        trailers,
        video,
        gpu,
        ..
    } = state;
    let (Some(active), Some(video), Some(gpu)) =
        (trailers.active.as_mut(), video.as_mut(), gpu.as_mut())
    else {
        return;
    };
    if !active.loaded || active.failed {
        return;
    }
    if video.status().error.is_some() {
        video.stop();
        active.loaded = false;
        active.failed = true;
        if let Some(texture) = active.texture.take() {
            gpu.egui_renderer.free_texture(&texture);
        }
        return;
    }
    let status = video.status();
    if !active.finished
        && status.duration > 0.0
        && (status.position + 0.6 >= status.duration || status.position + 1.0 < active.last_position)
    {
        active.finished = true;
    }
    active.last_position = status.position;
    if let Some(receiver) = active.subtitle.as_ref()
        && let Ok(text) = receiver.try_recv()
    {
        active.cues = text.map(|text| parse_cues(&text)).unwrap_or_default();
        active.subtitle = None;
    }
    active.cue = active
        .cues
        .iter()
        .find(|cue| cue.start <= status.position && status.position < cue.end)
        .map(|cue| cue.text.clone());
    if let Some(view) = video.render(&gpu.device) {
        if let Some(old) = active.texture.take() {
            gpu.egui_renderer.free_texture(&old);
        }
        active.texture = Some(gpu.egui_renderer.register_native_texture(
            &gpu.device,
            &view,
            wgpu::FilterMode::Linear,
        ));
    }
}

fn publish(state: &mut RendererState) {
    let active = state
        .trailers
        .active
        .as_ref()
        .filter(|active| !active.failed);
    state.home.trailer = active
        .filter(|_| state.route == Route::Home)
        .map(|active| HeroTrailer {
            item_id: active.key.clone(),
            texture: active.texture,
            finished: active.finished,
            subtitle: active.cue.clone(),
        });
    state.detail.trailer = active
        .filter(|active| state.route == Route::Detail && active.key == state.detail.id)
        .and_then(|active| active.texture);
    state.detail.trailer_subtitle = active
        .filter(|active| state.route == Route::Detail && active.key == state.detail.id)
    state.shorts.trailer = active
        .filter(|_| state.route == Route::Shorts)
        .map(|active| HeroTrailer {
            item_id: active.key.clone(),
            texture: active.texture,
            finished: active.finished,
            subtitle: active.cue.clone(),
        });
        .and_then(|active| active.cue.clone());
}

pub(crate) fn next_redraw(state: &RendererState, now: Instant) -> Option<Instant> {
    let active = state.trailers.active.as_ref();
    let fetching = state
        .trailers
        .fetching
        .is_some()
        .then(|| now + Duration::from_millis(100));
    let playing = active.filter(|active| !active.failed).map(|active| {
        if active.loaded {
            now
        } else if active.request.is_some() {
            now + Duration::from_millis(50)
        } else {
            active.due.max(now)
        }
    });
    [fetching, playing].into_iter().flatten().min()
}

struct Cue {
    start: f64,
    end: f64,
    text: String,
}

fn timestamp(value: &str) -> Option<f64> {
    let mut seconds = 0.0;
    for part in value.trim().split(':') {
        seconds = seconds * 60.0 + part.parse::<f64>().ok()?;
    }
    Some(seconds)
}

fn parse_cues(vtt: &str) -> Vec<Cue> {
    let mut cues = Vec::new();
    let mut lines = vtt.lines();
    while let Some(line) = lines.next() {
        let Some((start, rest)) = line.split_once("-->") else {
            continue;
        };
        let end = rest.split_whitespace().next().unwrap_or("");
        let (Some(start), Some(end)) = (timestamp(start), timestamp(end)) else {
            continue;
        };
        let mut text = String::new();
        let mut in_tag = false;
        for line in lines.by_ref().take_while(|line| !line.trim().is_empty()) {
            if !text.is_empty() {
                text.push('\n');
            }
            for character in line.chars() {
                match character {
                    '<' => in_tag = true,
                    '>' => in_tag = false,
                    _ if !in_tag => text.push(character),
                    _ => {}
                }
            }
        }
        let text = text
            .replace("&nbsp;", " ")
            .replace("&lt;", "<")
            .replace("&gt;", ">")
            .replace("&quot;", "\"")
            .replace("&#39;", "'")
            .replace("&amp;", "&");
        if !text.trim().is_empty() {
            cues.push(Cue { start, end, text });
        }
    }
    cues
}
