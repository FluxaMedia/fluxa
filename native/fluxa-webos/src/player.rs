use std::sync::mpsc::Receiver;

use serde_json::Value;

use crate::events::{Event, EventKind};
use crate::feed::FeedReply;
use crate::hdr::HdrInfo;
use crate::load::{AudioStream, VideoStream};
use crate::subtitle::{Cue, Cues};

const MAX_FEED_AHEAD_NS: i64 = 1_600_000_000;

pub struct Packet {
    pub video: bool,
    pub pts_ns: i64,
    pub data: Vec<u8>,
    pub cue: Option<Cue>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TrackInfo {
    pub id: String,
    pub subtitle: bool,
    pub language: Option<String>,
    pub title: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Track {
    pub info: TrackInfo,
    pub selected: bool,
}

pub struct MediaInfo {
    pub video: VideoStream,
    pub audio: Option<AudioStream>,
    pub duration_ns: i64,
    pub hdr: Option<HdrInfo>,
    pub tracks: Vec<TrackInfo>,
    pub audio_id: Option<String>,
}

pub trait Demuxer {
    fn open(&mut self, url: &str) -> Result<MediaInfo, String>;
    fn read(&mut self) -> Option<Packet>;
    fn seek(&mut self, pts_ns: i64) -> Result<(), String>;
    fn select_audio(&mut self, _id: &str) -> Result<AudioStream, String> {
        Err("audio switching is not supported".into())
    }
    fn select_subtitle(&mut self, _id: Option<&str>) {}
}

pub trait Pipeline {
    fn load(&mut self, info: &MediaInfo, start_ns: i64) -> Result<(), String>;
    fn play(&mut self);
    fn pause(&mut self);
    fn flush(&mut self, pts_ns: i64);
    fn feed(&mut self, packet: &Packet) -> FeedReply;
    fn end_of_stream(&mut self);
    fn unload(&mut self);
    fn set_rate(&mut self, _rate: f64) -> bool {
        false
    }
    fn change_audio(&mut self, _info: &MediaInfo) -> bool {
        false
    }
    fn on_event(&mut self, _event: &Event) {}
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
    pub tracks: Vec<Track>,
    pub subtitle: Option<String>,
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
    cues: Cues,
    subtitle_id: Option<String>,
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
            cues: Cues::default(),
            subtitle_id: None,
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
            "setSpeed" => self.set_speed(seconds(&request["value"])),
            "setVolume" => self.status.volume = seconds(&request["value"]).clamp(0.0, 100.0),
            "toggleMute" => self.status.volume = if self.status.volume > 0.0 { 0.0 } else { 100.0 },
            "selectTracks" => self.select_tracks(request),
            _ => {}
        }
    }

    fn set_speed(&mut self, rate: f64) {
        if self.state == State::Ready && rate != 0.0 {
            self.pipeline.set_rate(rate);
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
        self.subtitle_id = None;
        self.cues.clear();
        self.info = Some(info);
        self.refresh_tracks();
    }

    fn refresh_tracks(&mut self) {
        let Some(info) = &self.info else {
            return;
        };
        self.status.tracks = info
            .tracks
            .iter()
            .map(|track| Track {
                selected: if track.subtitle {
                    self.subtitle_id.as_ref() == Some(&track.id)
                } else {
                    info.audio_id.as_ref() == Some(&track.id)
                },
                info: track.clone(),
            })
            .collect();
    }

    fn select_tracks(&mut self, request: &Value) {
        if self.state != State::Ready {
            return;
        }
        if let Some(id) = request["audio"].as_str() {
            self.switch_audio(id);
        }
        let wanted = match request["subtitle"].as_str() {
            Some(id) => Some(id.to_owned()),
            None if request["subtitlesOff"].as_bool() == Some(true) => None,
            None => return self.refresh_tracks(),
        };
        if wanted == self.subtitle_id {
            return;
        }
        self.demuxer.select_subtitle(wanted.as_deref());
        self.subtitle_id = wanted;
        self.cues.clear();
        self.status.subtitle = None;
        self.refresh_tracks();
        if self.subtitle_id.is_some() {
            self.seek_to(self.position_ns);
        }
    }

    fn switch_audio(&mut self, id: &str) {
        let Some(mut info) = self.info.take() else {
            return;
        };
        if info.audio_id.as_deref() == Some(id) {
            self.info = Some(info);
            return;
        }
        let stream = match self.demuxer.select_audio(id) {
            Ok(stream) => stream,
            Err(error) => {
                self.info = Some(info);
                return self.fail(error);
            }
        };
        info.audio = Some(stream);
        info.audio_id = Some(id.to_owned());
        let position = self.position_ns;
        if self.pipeline.change_audio(&info) {
            self.info = Some(info);
            self.refresh_tracks();
            return self.seek_to(position);
        }
        self.pipeline.unload();
        self.pending = None;
        self.drained = false;
        self.cues.clear();
        self.state = State::Opening;
        self.status.buffering = true;
        let reloaded = self.demuxer.seek(position).and_then(|_| self.pipeline.load(&info, position));
        self.info = Some(info);
        self.refresh_tracks();
        if let Err(error) = reloaded {
            self.fail(error);
        }
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
        self.cues.clear();
        self.subtitle_id = None;
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
        self.status.subtitle = self.cues.at(self.position_ns);
    }

    fn apply(&mut self, event: Event) {
        self.pipeline.on_event(&event);
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
            if let Some(cue) = packet.cue {
                self.cues.push(cue);
                continue;
            }
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
            tracks: Vec::new(),
            audio_id: None,
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
        fn select_audio(&mut self, _id: &str) -> Result<AudioStream, String> {
            Ok(AudioStream { codec: crate::load::AudioCodec::Ac3, channels: 6, sample_rate: 48000, extradata: Vec::new() })
        }
    }

    #[derive(Default)]
    struct Log {
        calls: Vec<String>,
        full: bool,
        hot_audio_switch: bool,
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
        fn set_rate(&mut self, rate: f64) -> bool {
            self.0.borrow_mut().calls.push(format!("rate {rate}"));
            true
        }
        fn change_audio(&mut self, _info: &MediaInfo) -> bool {
            let mut log = self.0.borrow_mut();
            if log.hot_audio_switch {
                log.calls.push("change_audio".into());
            }
            log.hot_audio_switch
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
        let packets = pts.iter().map(|&pts_ns| Packet { video: true, pts_ns, data: vec![0], cue: None }).collect();
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

    #[test]
    fn switching_audio_reloads_the_pipeline_at_the_current_position() {
        let mut rig = started(&[]);
        rig.events.send(event(EventKind::FrameReady, 7_000_000_000)).unwrap();
        rig.player.tick();
        rig.player.handle(&json!({"type": "selectTracks", "audio": "a2"}));
        let calls = rig.log.borrow().calls.clone();
        assert_eq!(&calls[calls.len() - 2..], ["unload", "load"]);
        assert_eq!(*rig.seeks.borrow(), vec![7_000_000_000]);
    }

    #[test]
    fn hot_audio_switch_flushes_instead_of_reloading() {
        let mut rig = started(&[]);
        rig.log.borrow_mut().hot_audio_switch = true;
        rig.events.send(event(EventKind::FrameReady, 7_000_000_000)).unwrap();
        rig.player.tick();
        rig.player.handle(&json!({"type": "selectTracks", "audio": "a2"}));
        let calls = rig.log.borrow().calls.clone();
        assert_eq!(&calls[calls.len() - 2..], ["change_audio", "flush 7000000000"]);
        assert!(!calls.contains(&"unload".to_owned()));
    }

    #[test]
    fn speed_requests_reach_the_pipeline_once_loaded() {
        let mut rig = rig(&[]);
        rig.player.handle(&json!({"type": "setSpeed", "value": 4.0}));
        assert!(rig.log.borrow().calls.is_empty());
        rig.player.handle(&json!({"type": "load", "url": "u"}));
        rig.events.send(event(EventKind::LoadCompleted, 0)).unwrap();
        rig.player.tick();
        rig.player.handle(&json!({"type": "setSpeed", "value": 4.0}));
        assert_eq!(rig.log.borrow().calls.last().unwrap(), "rate 4");
    }

    #[test]
    fn cue_text_is_reported_only_while_it_is_on_screen() {
        let mut rig = started(&[]);
        rig.player.cues.push(Cue { start_ns: 1_000_000_000, end_ns: 2_000_000_000, text: "hi".into() });
        rig.events.send(event(EventKind::FrameReady, 1_500_000_000)).unwrap();
        rig.player.tick();
        assert_eq!(rig.player.status().subtitle.as_deref(), Some("hi"));
        rig.events.send(event(EventKind::FrameReady, 2_500_000_000)).unwrap();
        rig.player.tick();
        assert_eq!(rig.player.status().subtitle, None);
    }
}
