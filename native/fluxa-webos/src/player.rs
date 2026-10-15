use std::sync::mpsc::Receiver;

use serde_json::Value;

use crate::events::{Event, EventKind};
use crate::feed::FeedReply;
use crate::hdr::HdrInfo;
use crate::load::{AudioStream, VideoStream};

const MAX_FEED_AHEAD_NS: i64 = 1_600_000_000;

pub struct Packet {
    pub video: bool,
    pub pts_ns: i64,
    pub data: Vec<u8>,
}

pub struct MediaInfo {
    pub video: VideoStream,
    pub audio: Option<AudioStream>,
    pub duration_ns: i64,
    pub hdr: Option<HdrInfo>,
}

pub trait Demuxer {
    fn open(&mut self, url: &str) -> Result<MediaInfo, String>;
    fn read(&mut self) -> Option<Packet>;
    fn seek(&mut self, pts_ns: i64) -> Result<(), String>;
}

pub trait Pipeline {
    fn load(&mut self, info: &MediaInfo, start_ns: i64) -> Result<(), String>;
    fn play(&mut self);
    fn pause(&mut self);
    fn flush(&mut self, pts_ns: i64);
    fn feed(&mut self, packet: &Packet) -> FeedReply;
    fn end_of_stream(&mut self);
    fn unload(&mut self);
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Status {
    pub position: f64,
    pub duration: f64,
    pub paused: bool,
    pub volume: f64,
    pub has_frame: bool,
    pub buffering: bool,
    pub error: Option<String>,
    pub buffered: f64,
    pub audio: Option<String>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum State {
    Idle,
    Opening,
    Ready,
}

pub struct Player<D, P> {
    demuxer: D,
    pipeline: P,
    events: Receiver<Event>,
    state: State,
    info: Option<MediaInfo>,
    pending: Option<Packet>,
    position_ns: i64,
    drained: bool,
    status: Status,
}

fn seconds(value: &Value) -> f64 {
    value.as_f64().unwrap_or(0.0)
}

impl<D: Demuxer, P: Pipeline> Player<D, P> {
    pub fn new(demuxer: D, pipeline: P, events: Receiver<Event>) -> Player<D, P> {
        Player {
            demuxer,
            pipeline,
            events,
            state: State::Idle,
            info: None,
            pending: None,
            position_ns: 0,
            drained: false,
            status: Status { volume: 100.0, ..Status::default() },
        }
    }

    pub fn status(&self) -> &Status {
        &self.status
    }

    pub fn handle(&mut self, request: &Value) {
        match request["type"].as_str().unwrap_or_default() {
            "load" => self.load(request["url"].as_str().unwrap_or_default()),
            "stop" => self.stop(),
            "togglePause" => self.toggle_pause(),
            "seek" => self.seek_to(self.position_ns + (seconds(&request["seconds"]) * 1e9) as i64),
            "seekTo" => self.seek_to((seconds(&request["seconds"]) * 1e9) as i64),
            "setVolume" => self.status.volume = seconds(&request["value"]).clamp(0.0, 100.0),
            "toggleMute" => self.status.volume = if self.status.volume > 0.0 { 0.0 } else { 100.0 },
            _ => {}
        }
    }

    fn load(&mut self, url: &str) {
        self.stop();
        self.state = State::Opening;
        self.status.buffering = true;
        let info = match self.demuxer.open(url) {
            Ok(info) => info,
            Err(error) => return self.fail(error),
        };
        if let Err(error) = self.pipeline.load(&info, 0) {
            return self.fail(error);
        }
        self.status.duration = info.duration_ns as f64 / 1e9;
        self.status.audio = info.audio.as_ref().map(|audio| format!("{:?} {}ch", audio.codec, audio.channels));
        self.info = Some(info);
    }

    fn stop(&mut self) {
        if self.state != State::Idle {
            self.pipeline.unload();
        }
        self.state = State::Idle;
        self.info = None;
        self.pending = None;
        self.position_ns = 0;
        self.drained = false;
        let volume = self.status.volume;
        self.status = Status { volume, ..Status::default() };
    }

    fn fail(&mut self, error: String) {
        self.state = State::Idle;
        self.status.buffering = false;
        self.status.error = Some(error);
    }

    fn toggle_pause(&mut self) {
        if self.state != State::Ready {
            return;
        }
        if self.status.paused {
            self.pipeline.play();
        } else {
            self.pipeline.pause();
        }
    }

    fn seek_to(&mut self, target_ns: i64) {
        let Some(info) = &self.info else {
            return;
        };
        let target = target_ns.clamp(0, info.duration_ns.max(0));
        if let Err(error) = self.demuxer.seek(target) {
            return self.fail(error);
        }
        self.pending = None;
        self.drained = false;
        self.position_ns = target;
        self.status.position = target as f64 / 1e9;
        self.status.buffering = true;
        self.pipeline.flush(target);
    }

    pub fn tick(&mut self) {
        while let Ok(event) = self.events.try_recv() {
            self.apply(event);
        }
        if self.state == State::Ready {
            self.feed();
        }
    }

    fn apply(&mut self, event: Event) {
        match event.kind {
            EventKind::LoadCompleted => {
                self.state = State::Ready;
                self.pipeline.play();
            }
            EventKind::FrameReady => {
                self.position_ns = event.number;
                self.status.position = event.number as f64 / 1e9;
                self.status.has_frame = true;
                self.status.buffering = false;
            }
            EventKind::Playing => self.status.paused = false,
            EventKind::Paused => self.status.paused = true,
            EventKind::SeekDone => self.status.buffering = false,
            EventKind::Error => self.fail(event.text),
            _ => {}
        }
    }

    fn feed(&mut self) {
        loop {
            let packet = match self.pending.take() {
                Some(packet) => packet,
                None if self.drained => return,
                None => match self.demuxer.read() {
                    Some(packet) => packet,
                    None => {
                        self.drained = true;
                        self.pipeline.end_of_stream();
                        return;
                    }
                },
            };
            if packet.pts_ns - self.position_ns > MAX_FEED_AHEAD_NS {
                self.pending = Some(packet);
                return;
            }
            if self.pipeline.feed(&packet) != FeedReply::Ok {
                self.pending = Some(packet);
                return;
            }
            if packet.video {
                self.status.buffered = packet.pts_ns as f64 / 1e9;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::collections::VecDeque;
    use std::rc::Rc;
    use std::sync::mpsc::{Sender, channel};

    use serde_json::json;

    use super::*;
    use crate::load::VideoCodec;

    fn info() -> MediaInfo {
        MediaInfo {
            video: VideoStream { codec: VideoCodec::H264, width: 1920, height: 1080, fps: None, dolby_vision: None },
            audio: None,
            duration_ns: 100_000_000_000,
            hdr: None,
        }
    }

    struct Source {
        packets: VecDeque<Packet>,
        seeks: Rc<RefCell<Vec<i64>>>,
    }

    impl Demuxer for Source {
        fn open(&mut self, _url: &str) -> Result<MediaInfo, String> {
            Ok(info())
        }
        fn read(&mut self) -> Option<Packet> {
            self.packets.pop_front()
        }
        fn seek(&mut self, pts_ns: i64) -> Result<(), String> {
            self.seeks.borrow_mut().push(pts_ns);
            Ok(())
        }
    }

    #[derive(Default)]
    struct Log {
        calls: Vec<String>,
        full: bool,
    }

    struct Sink(Rc<RefCell<Log>>);

    impl Pipeline for Sink {
        fn load(&mut self, _info: &MediaInfo, _start_ns: i64) -> Result<(), String> {
            self.0.borrow_mut().calls.push("load".into());
            Ok(())
        }
        fn play(&mut self) {
            self.0.borrow_mut().calls.push("play".into());
        }
        fn pause(&mut self) {
            self.0.borrow_mut().calls.push("pause".into());
        }
        fn flush(&mut self, pts_ns: i64) {
            self.0.borrow_mut().calls.push(format!("flush {pts_ns}"));
        }
        fn feed(&mut self, packet: &Packet) -> FeedReply {
            let mut log = self.0.borrow_mut();
            if log.full {
                return FeedReply::BufferFull;
            }
            log.calls.push(format!("feed {}", packet.pts_ns));
            FeedReply::Ok
        }
        fn end_of_stream(&mut self) {
            self.0.borrow_mut().calls.push("eos".into());
        }
        fn unload(&mut self) {
            self.0.borrow_mut().calls.push("unload".into());
        }
    }

    struct Rig {
        player: Player<Source, Sink>,
        log: Rc<RefCell<Log>>,
        seeks: Rc<RefCell<Vec<i64>>>,
        events: Sender<Event>,
    }

    fn rig(pts: &[i64]) -> Rig {
        let log = Rc::new(RefCell::new(Log::default()));
        let seeks = Rc::new(RefCell::new(Vec::new()));
        let packets = pts.iter().map(|&pts_ns| Packet { video: true, pts_ns, data: vec![0] }).collect();
        let (events, rx) = channel();
        let player = Player::new(Source { packets, seeks: seeks.clone() }, Sink(log.clone()), rx);
        Rig { player, log, seeks, events }
    }

    fn event(kind: EventKind, number: i64) -> Event {
        Event { kind, number, text: String::new() }
    }

    fn started(pts: &[i64]) -> Rig {
        let mut rig = rig(pts);
        rig.player.handle(&json!({"type": "load", "url": "http://x/a.mkv"}));
        rig.events.send(event(EventKind::LoadCompleted, 0)).unwrap();
        rig.player.tick();
        rig
    }

    #[test]
    fn packets_are_fed_after_load_completes_and_stream_ends_with_eos() {
        let rig = started(&[0, 40_000_000]);
        assert_eq!(rig.log.borrow().calls, ["load", "play", "feed 0", "feed 40000000", "eos"]);
    }

    #[test]
    fn feeding_stops_when_too_far_ahead_of_playback() {
        let rig = started(&[0, 5_000_000_000]);
        assert_eq!(rig.log.borrow().calls, ["load", "play", "feed 0"]);
    }

    #[test]
    fn buffer_full_keeps_the_packet_for_the_next_tick() {
        let mut rig = rig(&[0]);
        rig.player.handle(&json!({"type": "load", "url": "u"}));
        rig.log.borrow_mut().full = true;
        rig.events.send(event(EventKind::LoadCompleted, 0)).unwrap();
        rig.player.tick();
        assert!(rig.log.borrow().calls.iter().all(|call| !call.starts_with("feed")));
        rig.log.borrow_mut().full = false;
        rig.player.tick();
        assert!(rig.log.borrow().calls.contains(&"feed 0".to_owned()));
    }

    #[test]
    fn frame_events_drive_reported_position() {
        let mut rig = started(&[]);
        rig.events.send(event(EventKind::FrameReady, 2_500_000_000)).unwrap();
        rig.player.tick();
        assert_eq!(rig.player.status().position, 2.5);
        assert!(rig.player.status().has_frame);
    }

    #[test]
    fn seek_clamps_flushes_and_drops_the_pending_packet() {
        let mut rig = started(&[0, 5_000_000_000]);
        rig.player.handle(&json!({"type": "seekTo", "seconds": 500.0}));
        assert_eq!(*rig.seeks.borrow(), vec![100_000_000_000]);
        assert!(rig.log.borrow().calls.contains(&"flush 100000000000".to_owned()));
        assert_eq!(rig.player.status().position, 100.0);
    }

    #[test]
    fn relative_seek_never_goes_negative() {
        let mut rig = started(&[]);
        rig.player.handle(&json!({"type": "seek", "seconds": -30.0}));
        assert_eq!(*rig.seeks.borrow(), vec![0]);
    }

    #[test]
    fn pipeline_error_event_surfaces_in_status() {
        let mut rig = started(&[]);
        rig.events.send(Event { kind: EventKind::Error, number: 0, text: "decoder died".into() }).unwrap();
        rig.player.tick();
        assert_eq!(rig.player.status().error.as_deref(), Some("decoder died"));
    }

    #[test]
    fn stop_unloads_and_resets_status() {
        let mut rig = started(&[]);
        rig.player.handle(&json!({"type": "stop"}));
        assert_eq!(rig.log.borrow().calls.last().unwrap(), "unload");
        assert_eq!(rig.player.status().duration, 0.0);
    }
}
