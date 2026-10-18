use std::sync::mpsc::Sender;

use crate::diagnostics::trace;
use crate::acb::Acb;
use crate::events::{Event, EventKind};
use crate::feed::{FeedReply, feed_payload};
use crate::ffi::Starfish;
use crate::hdr::{hdr_payload, merge_into_video_data};
use crate::load::{AudioCodec, LoadOptions, audio_change, load_payload};
use crate::player::{MediaInfo, Packet, Pipeline};

pub struct StarfishPipeline {
    inner: Starfish,
    events: Sender<Event>,
    used: bool,
    atmos: bool,
    options: LoadOptions,
    seek_pending: bool,
    acb_hdr: Option<String>,
    acb: Option<Acb>,
}

impl StarfishPipeline {
    pub fn open(options: LoadOptions, events: Sender<Event>) -> Result<StarfishPipeline, crate::ffi::Error> {
        let inner = Starfish::open(options.webos_major, events.clone())?;
        let acb = if options.window_id.is_none() {
            let app = std::env::var("APPID").unwrap_or_else(|_| options.app_id.clone());
            Acb::open(&app).map_err(|error| trace(format!("acb unavailable: {error:?}"))).ok()
        } else {
            None
        };
        Ok(StarfishPipeline { inner, events, used: false, atmos: false, options, seek_pending: false, acb_hdr: None, acb })
    }
}

impl StarfishPipeline {
    fn seek_to(&mut self, pts_ns: i64) {
        let position = format!("{{\"position\":{}}}", pts_ns.max(0));
        if self.inner.set_time_to_decode(&position) && self.inner.send_segment_event() {
            return;
        }
        trace("segment seek unavailable, using Seek");
        if !self.inner.seek_ms((pts_ns.max(0) / 1_000_000) as u64) {
            trace("Seek refused");
        }
    }
}

impl Pipeline for StarfishPipeline {
    fn load(&mut self, info: &MediaInfo, start_ns: i64) -> Result<(), String> {
        if let Some(reason) = crate::jail::refusal() {
            return Err(reason);
        }
        if self.used {
            self.inner = Starfish::open(self.options.webos_major, self.events.clone()).map_err(|error| format!("{error:?}"))?;
        }
        self.used = true;
        self.atmos = matches!(info.audio.as_ref().map(|audio| audio.codec), Some(AudioCodec::Eac3 { atmos: true }));
        let mut options = self.options.clone();
        options.start_pts_ns = start_ns;
        if options.max_resolution.is_none() {
            options.max_resolution = crate::ffi::max_video_resolution(info.video.codec.name(), options.webos_major);
        }
        let payload = load_payload(&info.video, info.audio.as_ref(), &options);
        self.inner.notify_foreground();
        self.inner.load(&payload).map_err(|error| format!("{error:?}"))?;
        self.acb_hdr = None;
        if let Some(hdr) = info.hdr.as_ref().and_then(|hdr| hdr_payload(hdr, options.webos_major)) {
            if self.acb.is_some() {
                self.acb_hdr = Some(hdr);
            } else if !self.inner.set_hdr_info(&hdr) {
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

    fn flush(&mut self, _pts_ns: i64) {
        if !self.inner.flush() {
            trace("flush rejected");
        }
        self.seek_pending = true;
    }

    fn feed(&mut self, packet: &Packet) -> FeedReply {
        if self.seek_pending {
            if !packet.video {
                return FeedReply::Ok;
            }
            self.seek_pending = false;
            self.seek_to(packet.pts_ns);
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

    fn awaits_unload(&self) -> bool {
        true
    }

    fn set_rate(&mut self, rate: f64) -> bool {
        let payload = format!("{{\"audioOutput\":{},\"playRate\":{rate}}}", rate.abs() <= 2.0);
        self.inner.set_play_rate(&payload).unwrap_or(false)
    }

    fn change_audio(&mut self, info: &MediaInfo) -> bool {
        let Some(audio) = info.audio.as_ref().filter(|_| self.options.webos_major >= 6) else {
            return false;
        };
        let (codec, options) = audio_change(audio);
        let changed = self.inner.change_audio_codec(codec, &options).unwrap_or(false);
        if !changed {
            trace("changeAudioCodec unavailable or rejected");
        }
        changed
    }

    fn on_event(&mut self, event: &Event) {
        let Some(acb) = &self.acb else {
            return;
        };
        match event.kind {
            EventKind::VideoInfo => match &self.acb_hdr {
                Some(hdr) => acb.video_data(&merge_into_video_data(&event.text, hdr)),
                None => acb.video_data(&event.text),
            },
            EventKind::LoadCompleted => {
                if let Some(media) = self.inner.media_id() {
                    acb.bind(&media);
                    if self.atmos {
                        acb.audio_data(&atmos_descriptor(&media));
                    }
                }
            }
            EventKind::Playing => acb.playing(1920, 1080),
            EventKind::Paused => acb.paused(),
            EventKind::UnloadCompleted => acb.unloaded(),
            _ => {}
        }
    }
}

fn atmos_descriptor(media_id: &str) -> String {
    serde_json::json!({ "audio": { "immersive": "ATMOS" }, "context": media_id }).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn atmos_descriptor_carries_the_media_id_as_context() {
        let json: serde_json::Value = serde_json::from_str(&atmos_descriptor("abc")).unwrap();
        assert_eq!(json["audio"]["immersive"], "ATMOS");
        assert_eq!(json["context"], "abc");
    }
}
