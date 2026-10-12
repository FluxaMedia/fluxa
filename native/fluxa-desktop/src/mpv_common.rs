use std::path::PathBuf;
use std::time::{Duration, Instant};

use fluxa_host::{Thumbnail, ThumbnailPlan, TrackSelection, VideoTrack};

const THUMBNAIL_WIDTH: &str = "320";
const THUMBNAIL_RETRY: Duration = Duration::from_millis(150);
const THUMBNAIL_PATIENCE: Duration = Duration::from_secs(4);

pub fn thumbnail_url(url: &str) -> String {
    if url.contains("/stream/fname") {
        format!("{url}&role=auxiliary")
    } else {
        url.to_owned()
    }
}

pub fn list_tracks(client: &fluxa_mpv::MpvClientHandle) -> Vec<VideoTrack> {
    [("audio", false), ("sub", true)]
        .into_iter()
        .flat_map(|(kind, subtitle)| {
            client
                .track_options(kind)
                .into_iter()
                .map(move |option| VideoTrack {
                    id: option.id,
                    subtitle,
                    language: option.lang,
                    title: Some(option.label),
                    selected: option.selected,
                    external: option.external,
                })
        })
        .collect()
}

pub fn set_fit(client: &fluxa_mpv::MpvClientHandle, mode: &str) -> Result<(), String> {
    let (keepaspect, panscan) = match mode {
        "fill" => ("yes", "1"),
        "stretch" => ("no", "0"),
        _ => ("yes", "0"),
    };
    client.command(&["set", "keepaspect", keepaspect])?;
    client.command(&["set", "panscan", panscan])
}

pub fn select_tracks(
    client: &fluxa_mpv::MpvClientHandle,
    selection: &TrackSelection,
) -> Result<(), String> {
    if let Some(audio) = &selection.audio {
        client.command(&["set", "aid", audio])?;
    }
    match (&selection.subtitle, selection.subtitles_off) {
        (Some(subtitle), _) => client.command(&["set", "sid", subtitle])?,
        (None, true) => client.command(&["set", "sid", "no"])?,
        (None, false) => {}
    }
    let secondary = selection.secondary_subtitle.as_deref().unwrap_or("no");
    client.command(&["set", "secondary-sid", secondary])
}

pub fn enable_thumbnails(
    client: &fluxa_mpv::MpvClientHandle,
    url: &str,
    plan: &ThumbnailPlan,
) {
    if !plan.enabled {
        let _ = client.set_option("thumbnails", "no");
        return;
    }
    let source = plan.url.clone().unwrap_or_else(|| thumbnail_url(url));
    if source != url {
        if client.set_option("thumbnail-url", &source).is_err() {
            return;
        }
        let headers = plan
            .headers
            .iter()
            .map(|(name, value)| format!("{name}: {value}").replace(',', "\\,"))
            .collect::<Vec<_>>()
            .join(",");
        let _ = client.set_option("thumbnail-http-header-fields", &headers);
    }
    let tuning = [
        ("thumbnail-width", Some(THUMBNAIL_WIDTH.to_owned())),
        ("thumbnail-interval", Some(plan.interval.max(1.0).to_string())),
        ("thumbnail-sweep", Some(if plan.sheet { "yes" } else { "no" }.to_owned())),
        ("thumbnail-delay", Some(if plan.hold { "0" } else { "3" }.to_owned())),
    ];
    for (name, value) in tuning {
        if let Some(value) = value {
            let _ = client.set_option(name, &value);
        }
    }
    if let Err(error) = client.set_option("thumbnails", "yes") {
        eprintln!("[fluxa-desktop] seek thumbnails unavailable: {error}");
    }
}

#[derive(Default)]
pub struct SeekPreview {
    wanted: Option<(f64, Instant)>,
    tried: Option<Instant>,
    ready: Option<Thumbnail>,
}

impl SeekPreview {
    pub fn request(&mut self, client: &fluxa_mpv::MpvClientHandle, time: f64) {
        self.wanted = Some((time, Instant::now()));
        self.attempt(client);
    }

    pub fn take(&mut self, client: &fluxa_mpv::MpvClientHandle) -> Option<Thumbnail> {
        let due = self.tried.is_none_or(|at| at.elapsed() >= THUMBNAIL_RETRY);
        if due && self.wanted.is_some() {
            self.attempt(client);
        }
        self.ready.take()
    }

    pub fn clear(&mut self) {
        *self = Self::default();
    }

    fn attempt(&mut self, client: &fluxa_mpv::MpvClientHandle) {
        let Some((time, since)) = self.wanted else {
            return;
        };
        if since.elapsed() >= THUMBNAIL_PATIENCE {
            self.wanted = None;
            return;
        }
        self.tried = Some(Instant::now());
        if let Some(found) = client.thumbnail(time) {
            if found.exact {
                self.wanted = None;
            }
            self.ready = Some(Thumbnail {
                time: found.time,
                size: [found.width, found.height],
                rgba: found.rgba,
            });
        }
    }
}

#[derive(Default)]
pub struct Chapters {
    list: Vec<(f64, String)>,
}

impl Chapters {
    pub fn load(&mut self, client: &fluxa_mpv::MpvClientHandle) {
        self.list = client
            .chapters_json()
            .and_then(|json| serde_json::from_str::<serde_json::Value>(&json).ok())
            .and_then(|value| {
                value.get("chapters")?.as_array().map(|chapters| {
                    chapters
                        .iter()
                        .filter_map(|chapter| {
                            Some((
                                chapter.get("startMs")?.as_u64()? as f64 / 1000.0,
                                chapter.get("title")?.as_str()?.to_owned(),
                            ))
                        })
                        .collect()
                })
            })
            .unwrap_or_default();
    }

    pub fn list(&self) -> &[(f64, String)] {
        &self.list
    }
}

pub fn buffering(position: &fluxa_mpv::PlayerPositionStatus) -> Option<f32> {
    let number = |value: Option<&String>| value.and_then(|value| value.parse::<f32>().ok());
    if position.paused_for_cache.as_deref() == Some("yes")
        && let Some(state) = number(position.cache_buffering_state.as_ref())
    {
        return Some(state / 100.0);
    }
    number(position.demuxer_cache_duration.as_ref())
        .filter(|seconds| *seconds > 0.0)
        .map(|seconds| (seconds / 5.0).min(1.0))
}

pub fn shader_dir() -> Option<PathBuf> {
    let beside = std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(|dir| dir.join("mpv-shaders/anime4k")));
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../fluxa-mpv/shaders/anime4k");
    [beside, Some(source)]
        .into_iter()
        .flatten()
        .find(|dir| dir.is_dir())
}
