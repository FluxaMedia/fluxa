use std::sync::mpsc::Receiver;

use fluxa_effects::{SessionHandle, core_value};
use serde_json::{Value, json};

use super::*;

const RESOLVE_TIMEOUT: Duration = Duration::from_secs(20);
const NO_PROGRESS_TIMEOUT: Duration = Duration::from_secs(15);
const HOLD_LIMIT: Duration = Duration::from_secs(120);

pub(super) struct SeekPreviews {
    plan: ThumbnailPlan,
    pregenerate: bool,
    resolving: Option<(Receiver<Option<Value>>, Instant)>,
    manifest: Option<Receiver<Option<Vec<u8>>>>,
    server: Option<(String, String)>,
    loaded_at: Option<Instant>,
    holding: bool,
    progress: Option<ThumbnailProgress>,
}

impl SeekPreviews {
    pub(super) fn begin(
        settings: &SettingsModel,
        snapshot: &Value,
        meta: &Value,
        session: Option<&SessionHandle>,
        media_server: bool,
    ) -> Self {
        let planned = core_value("trickplayPlan", json!({"settings": settings.values}))
            .unwrap_or(Value::Null);
        let flag = |key: &str| planned.get(key).and_then(Value::as_bool).unwrap_or(false);
        let source = planned.get("source").unwrap_or(&Value::Null);
        let external = source.get("kind").and_then(Value::as_str) != Some("stream");
        let enabled = flag("enabled") && !media_server;
        let video_id = snapshot
            .pointer("/player/currentVideoId")
            .or_else(|| meta.get("id"))
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_owned();
        let server = (source.get("kind").and_then(Value::as_str) == Some("server"))
            .then(|| source.get("id").and_then(Value::as_str))
            .flatten()
            .map(|base| (base.to_owned(), video_id.clone()));
        let manifest = session.zip(server.as_ref()).filter(|_| enabled).and_then(|(session, (base, id))| {
            let url = core_value("trickplayServerManifest", json!({"base": base, "videoId": id}))?;
            Some(session.fetch_bytes(url.as_str()?.to_owned()))
        });
        let enabled = enabled && server.is_none();
        let external = external && server.is_none();
        let resolving = session.filter(|_| enabled && external).map(|session| {
            let payload = json!({
                "contentType": meta.get("type").and_then(Value::as_str).unwrap_or("movie"),
                "videoId": video_id,
                "source": source,
                "regex": planned.get("regex"),
            });
            (session.resolve_trickplay_source(payload), Instant::now())
        });
        Self {
            plan: ThumbnailPlan {
                enabled,
                url: None,
                headers: Vec::new(),
                interval: planned.get("interval").and_then(Value::as_f64).unwrap_or(10.0),
                sheet: flag("sheet"),
                hold: false,
            },
            pregenerate: flag("pregenerate"),
            resolving,
            manifest,
            server,
            loaded_at: None,
            holding: false,
            progress: None,
        }
    }

    pub(super) fn server_source(&mut self) -> Option<fluxa_core::player::trickplay::Source> {
        let bytes = self.manifest.as_ref()?.try_recv();
        let bytes = match bytes {
            Ok(bytes) => bytes,
            Err(std::sync::mpsc::TryRecvError::Empty) => return None,
            Err(_) => None,
        };
        self.manifest = None;
        let (base, id) = self.server.take()?;
        let manifest: Value = serde_json::from_slice(&bytes?).ok()?;
        let source = core_value(
            "trickplayServerSource",
            json!({"base": base, "videoId": id, "manifest": manifest}),
        )?;
        serde_json::from_value(source).ok()
    }

    pub(super) fn ready_to_load(&mut self) -> bool {
        if !self.pregenerate {
            return true;
        }
        self.poll_source();
        self.resolving.is_none()
    }

    pub(super) fn initial_plan(&mut self) -> ThumbnailPlan {
        let mut plan = self.plan.clone();
        if self.resolving.is_some() {
            plan.enabled = false;
        }
        plan.hold = self.pregenerate && plan.enabled;
        self.holding = plan.hold;
        plan
    }

    pub(super) fn loaded(&mut self) {
        self.loaded_at = Some(Instant::now());
    }

    pub(super) fn holding(&self) -> Option<Option<ThumbnailProgress>> {
        self.holding.then_some(self.progress)
    }

    pub(super) fn preparing(&self) -> bool {
        self.pregenerate && self.resolving.is_some()
    }

    pub(super) fn tick(&mut self, video: &mut dyn VideoBackend) {
        if self.poll_source() {
            let mut plan = self.plan.clone();
            plan.hold = self.holding;
            video.thumbnails(&plan);
        }
        let Some(loaded_at) = self.loaded_at.filter(|_| self.holding) else {
            return;
        };
        self.progress = video.thumbnail_progress().filter(|progress| progress.count > 0);
        let finished = self
            .progress
            .is_some_and(|progress| progress.done >= progress.count);
        let stalled = self.progress.is_none() && loaded_at.elapsed() >= NO_PROGRESS_TIMEOUT;
        if finished || stalled || loaded_at.elapsed() >= HOLD_LIMIT {
            self.holding = false;
            video.release_start();
        }
    }

    fn poll_source(&mut self) -> bool {
        let Some((receiver, since)) = self.resolving.as_ref() else {
            return false;
        };
        let picked = match receiver.try_recv() {
            Ok(picked) => picked,
            Err(std::sync::mpsc::TryRecvError::Empty) if since.elapsed() < RESOLVE_TIMEOUT => {
                return false;
            }
            Err(_) => None,
        };
        self.resolving = None;
        if let Some(picked) = picked {
            self.plan.url = picked.get("url").and_then(Value::as_str).map(str::to_owned);
            self.plan.headers = picked
                .get("headers")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .filter_map(|pair| {
                    Some((pair.get(0)?.as_str()?.to_owned(), pair.get(1)?.as_str()?.to_owned()))
                })
                .collect();
        }
        true
    }
}
