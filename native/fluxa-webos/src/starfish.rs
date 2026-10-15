use std::sync::mpsc::Sender;

use crate::diagnostics::trace;
use crate::acb::Acb;
use crate::events::{Event, EventKind};
use crate::feed::{FeedReply, feed_payload};
use crate::ffi::Starfish;
use crate::hdr::hdr_payload;
use crate::load::{LoadOptions, load_payload};
use crate::player::{MediaInfo, Packet, Pipeline};

pub struct StarfishPipeline {
    inner: Starfish,
    options: LoadOptions,
    resume_at: Option<i64>,
    acb: Option<Acb>,
}

impl StarfishPipeline {
    pub fn open(options: LoadOptions, events: Sender<Event>) -> Result<StarfishPipeline, crate::ffi::Error> {
        let inner = Starfish::open(&options.app_id, options.webos_major, events)?;
        let acb = if options.window_id.is_none() {
            let app = std::env::var("APPID").unwrap_or_else(|_| options.app_id.clone());
            Acb::open(&app).map_err(|error| trace(format!("acb unavailable: {error:?}"))).ok()
        } else {
            None
        };
        Ok(StarfishPipeline { inner, options, resume_at: None, acb })
    }
}

impl Pipeline for StarfishPipeline {
    fn load(&mut self, info: &MediaInfo, start_ns: i64) -> Result<(), String> {
        let mut options = self.options.clone();
        options.start_pts_ns = start_ns;
        let payload = load_payload(&info.video, info.audio.as_ref(), &options);
        self.inner.notify_foreground();
        self.inner.load(&payload).map_err(|error| format!("{error:?}"))?;
        if let Some(hdr) = info.hdr.as_ref().and_then(|hdr| hdr_payload(hdr, options.webos_major)) {
            if !self.inner.set_hdr_info(&hdr) {
                trace("setHdrInfo rejected");
            }
        }
        Ok(())
    }

    fn play(&mut self) {
        self.inner.play();
    }

    fn pause(&mut self) {
        self.inner.pause();
    }

    fn flush(&mut self, pts_ns: i64) {
        if !self.inner.flush() {
            trace("flush rejected");
        }
        self.resume_at = Some(pts_ns);
    }

    fn feed(&mut self, packet: &Packet) -> FeedReply {
        if packet.video {
            if self.resume_at.take().is_some() {
                let payload = format!("{{\"position\":{}}}", packet.pts_ns);
                if !self.inner.set_time_to_decode(&payload) {
                    trace("setTimeToDecode rejected");
                }
            }
        }
        let payload = feed_payload(packet.data.as_ptr() as usize, packet.data.len(), packet.pts_ns, packet.video);
        self.inner.feed(&payload)
    }

    fn end_of_stream(&mut self) {
        if !self.inner.push_eos() {
            trace("pushEOS rejected");
        }
    }

    fn unload(&mut self) {
        self.inner.unload();
    }

    fn on_event(&mut self, event: &Event) {
        let Some(acb) = &self.acb else {
            return;
        };
        match event.kind {
            EventKind::VideoInfo => acb.video_data(&event.text),
            EventKind::LoadCompleted => {
                if let Some(media) = self.inner.media_id() {
                    acb.bind(&media);
                }
            }
            EventKind::Playing => acb.playing(1920, 1080),
            EventKind::Paused => acb.paused(),
            EventKind::UnloadCompleted => acb.unloaded(),
            _ => {}
        }
    }
}
