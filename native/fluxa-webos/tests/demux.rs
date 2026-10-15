#![cfg(feature = "ffmpeg")]

use std::path::PathBuf;
use std::process::Command;

use fluxa_webos::demux::FfmpegDemuxer;
use fluxa_webos::player::Demuxer;
use fluxa_webos::{AudioCodec, VideoCodec};

fn sample(name: &str, args: &[&str]) -> Option<PathBuf> {
    let dir = std::env::var_os("FLUXA_TEST_TMP").map(PathBuf::from).unwrap_or_else(std::env::temp_dir);
    std::fs::create_dir_all(&dir).ok()?;
    let path = dir.join(name);
    let status = Command::new("ffmpeg")
        .args(["-y", "-loglevel", "error", "-f", "lavfi", "-i", "testsrc=size=320x240:rate=25:duration=2"])
        .args(["-f", "lavfi", "-i", "sine=frequency=440:duration=2"])
        .args(args)
        .arg(&path)
        .status()
        .ok()?;
    status.success().then_some(path)
}

#[test]
fn mp4_h264_aac_yields_annexb_video_and_info() {
    let Some(path) = sample("fluxa-webos-a.mp4", &["-c:v", "libx264", "-c:a", "aac", "-shortest"]) else {
        return;
    };
    let mut demuxer = FfmpegDemuxer::default();
    let info = demuxer.open(path.to_str().unwrap()).unwrap();
    assert_eq!(info.video.codec, VideoCodec::H264);
    assert_eq!((info.video.width, info.video.height), (320, 240));
    assert!(matches!(info.audio.unwrap().codec, AudioCodec::Aac { raw: true, .. }));
    assert!((1_900_000_000..=2_100_000_000).contains(&info.duration_ns));

    let first_video = std::iter::from_fn(|| demuxer.read()).find(|packet| packet.video).unwrap();
    assert_eq!(&first_video.data[..4], &[0, 0, 0, 1]);
    let _ = std::fs::remove_file(path);
}

#[test]
fn seek_lands_near_the_target() {
    let Some(path) = sample("fluxa-webos-b.mkv", &["-c:v", "libx264", "-g", "5", "-c:a", "libopus", "-shortest"]) else {
        return;
    };
    let mut demuxer = FfmpegDemuxer::default();
    demuxer.open(path.to_str().unwrap()).unwrap();
    demuxer.seek(1_000_000_000).unwrap();
    let packet = demuxer.read().unwrap();
    assert!(packet.pts_ns <= 1_100_000_000 && packet.pts_ns >= 700_000_000, "pts {}", packet.pts_ns);
    let _ = std::fs::remove_file(path);
}
