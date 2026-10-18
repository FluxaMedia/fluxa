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

#[test]
fn second_audio_and_srt_subtitle_are_listed_and_selectable() {
    let dir = std::env::var_os("FLUXA_TEST_TMP").map(PathBuf::from).unwrap_or_else(std::env::temp_dir);
    std::fs::create_dir_all(&dir).unwrap();
    let srt = dir.join("fluxa-webos-c.srt");
    std::fs::write(&srt, "1\n00:00:00,500 --> 00:00:01,500\nHello <i>there</i>\n\n").unwrap();
    let args = [
        "-f", "lavfi", "-i", "sine=frequency=880:duration=2", "-i", srt.to_str().unwrap(),
        "-map", "0:v", "-map", "1:a", "-map", "2:a", "-map", "3", "-c:v", "libx264", "-c:a", "libopus",
        "-c:s", "srt", "-metadata:s:a:1", "language=tur", "-metadata:s:s:0", "language=eng", "-shortest",
    ];
    let Some(path) = sample("fluxa-webos-c.mkv", &args) else {
        return;
    };
    let mut demuxer = FfmpegDemuxer::default();
    let info = demuxer.open(path.to_str().unwrap()).unwrap();
    let audio: Vec<_> = info.tracks.iter().filter(|track| !track.subtitle).collect();
    let subtitles: Vec<_> = info.tracks.iter().filter(|track| track.subtitle).collect();
    assert_eq!(audio.len(), 2);
    assert_eq!(audio[1].language.as_deref(), Some("tur"));
    assert_eq!(subtitles.len(), 1);
    assert_eq!(info.audio_id.as_deref(), Some(audio[0].id.as_str()));

    demuxer.select_audio(&audio[1].id).unwrap();
    demuxer.select_subtitle(Some(&subtitles[0].id));
    let cue = std::iter::from_fn(|| demuxer.read()).find_map(|packet| packet.cue).unwrap();
    assert_eq!(cue.text, "Hello there");
    assert_eq!(cue.start_ns, 500_000_000);
    let _ = std::fs::remove_file(path);
    let _ = std::fs::remove_file(srt);
}

#[test]
fn pq_tagged_stream_is_reported_as_hdr10() {
    let args = [
        "-vf", "setparams=color_primaries=bt2020:color_trc=smpte2084:colorspace=bt2020nc", "-c:v", "libx264",
        "-an", "-shortest",
    ];
    let Some(path) = sample("fluxa-webos-d.mkv", &args) else {
        return;
    };
    let mut demuxer = FfmpegDemuxer::default();
    let info = demuxer.open(path.to_str().unwrap()).unwrap();
    let hdr = info.hdr.expect("hdr info");
    assert_eq!(hdr.kind, fluxa_webos::HdrKind::Hdr10);
    assert_eq!((hdr.transfer, hdr.primaries, hdr.matrix), (16, 9, 9));
    assert!(info.video.dolby_vision.is_none());
    let _ = std::fs::remove_file(path);
}

#[test]
fn sdr_stream_has_no_hdr_info() {
    let Some(path) = sample("fluxa-webos-e.mp4", &["-c:v", "libx264", "-an", "-shortest"]) else {
        return;
    };
    let mut demuxer = FfmpegDemuxer::default();
    assert!(demuxer.open(path.to_str().unwrap()).unwrap().hdr.is_none());
    let _ = std::fs::remove_file(path);
}

#[test]
fn flac_audio_is_transcoded_to_ac3_with_monotonic_timestamps() {
    let Some(path) = sample("fluxa-webos-f.mkv", &["-c:v", "libx264", "-c:a", "flac", "-shortest"]) else {
        return;
    };
    let mut demuxer = FfmpegDemuxer::default();
    let info = demuxer.open(path.to_str().unwrap()).unwrap();
    let audio = info.audio.expect("transcoded audio");
    assert_eq!(audio.codec, AudioCodec::Ac3);
    assert_eq!(audio.sample_rate, 44_100);

    let mut last = -1;
    let mut count = 0;
    while let Some(packet) = demuxer.read() {
        if packet.video {
            continue;
        }
        assert_eq!(&packet.data[..2], &[0x0b, 0x77]);
        assert!(packet.pts_ns > last);
        last = packet.pts_ns;
        count += 1;
    }
    assert!(count > 40, "only {count} ac3 frames");
    let _ = std::fs::remove_file(path);
}

#[test]
fn hdr_is_dropped_when_the_tv_has_no_hdr() {
    let args = [
        "-vf", "setparams=color_primaries=bt2020:color_trc=smpte2084:colorspace=bt2020nc", "-c:v", "libx264",
        "-an", "-shortest",
    ];
    let Some(path) = sample("fluxa-webos-g.mkv", &args) else {
        return;
    };
    let caps = fluxa_webos::capabilities::Capabilities { hdr: Some(false), dolby_vision: Some(false), ..Default::default() };
    let mut demuxer = FfmpegDemuxer::with_capabilities(caps);
    assert!(demuxer.open(path.to_str().unwrap()).unwrap().hdr.is_none());
    let _ = std::fs::remove_file(path);
}
