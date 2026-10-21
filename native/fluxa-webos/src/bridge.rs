use std::sync::{Arc, Mutex};

use fluxa_host::{VideoBridge, VideoStatus, VideoTrack};

use crate::player::{Demuxer, Pipeline, Player};

pub fn pump<D: Demuxer, P: Pipeline>(player: &mut Player<D, P>, bridge: &Arc<Mutex<VideoBridge>>) {
    let requests = bridge.lock().map(|mut bridge| bridge.take_requests()).unwrap_or_default();
    for request in &requests {
        player.handle(request);
    }
    player.tick();
    let status = player.status();
    let report = VideoStatus {
        position: status.position,
        duration: status.duration,
        paused: status.paused,
        muted: status.volume <= 0.0,
        volume: status.volume,
        has_frame: status.has_frame,
        error: status.error.clone(),
        chapters: Vec::new(),
        buffering: status.buffering.then_some(0.0),
        buffered: status.buffered,
        stalled: status.buffering && status.has_frame,
    };
    let tracks = status
        .tracks
        .iter()
        .map(|track| VideoTrack {
            id: track.info.id.clone(),
            subtitle: track.info.subtitle,
            language: track.info.language.clone(),
            title: track.info.title.clone(),
            selected: track.selected,
            external: false,
            forced: false,
        })
        .collect();
    if let Ok(mut bridge) = bridge.lock() {
        bridge.set_tracks(tracks);
        bridge.set_subtitle(status.subtitle.clone());
        bridge.set_status(report);
    }
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc::channel;

    use fluxa_host::{BridgeVideo, VideoBackend};

    use super::*;
    use crate::feed::FeedReply;
    use crate::player::{MediaInfo, Packet};

    struct Empty;

    impl Demuxer for Empty {
        fn open(&mut self, _url: &str) -> Result<MediaInfo, String> {
            Err("no such file".into())
        }
        fn read(&mut self) -> Option<Packet> {
            None
        }
        fn seek(&mut self, _pts_ns: i64) -> Result<(), String> {
            Ok(())
        }
    }

    impl Pipeline for Empty {
        fn load(&mut self, _info: &MediaInfo, _start_ns: i64) -> Result<(), String> {
            Ok(())
        }
        fn play(&mut self) {}
        fn pause(&mut self) {}
        fn flush(&mut self, _pts_ns: i64) {}
        fn feed(&mut self, _packet: &Packet) -> FeedReply {
            FeedReply::Ok
        }
        fn end_of_stream(&mut self) {}
        fn unload(&mut self) {}
    }

    #[test]
    fn load_failure_reaches_the_host_status() {
        let bridge = Arc::new(Mutex::new(VideoBridge::default()));
        let (_tx, rx) = channel();
        let mut player = Player::new(Empty, Empty, rx);
        let mut backend = BridgeVideo(bridge.clone());
        let request = serde_json::json!({"type": "load", "url": "x"});
        player.handle(&request);
        pump(&mut player, &bridge);
        assert_eq!(backend.status().error.as_deref(), Some("no such file"));
        assert_eq!(backend.status().volume, 100.0);
    }
}
