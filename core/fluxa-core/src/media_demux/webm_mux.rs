//! Remuxes demuxed MKV tracks/packets into a single, complete WebM byte
//! buffer suitable for one `SourceBuffer.appendBuffer()` call.
//!
//! Bitstream copy only — no re-encoding. Intended for the codecs a browser
//! already decodes natively (VP8/VP9/AV1 video, Opus/Vorbis audio); the
//! container is the only thing that changes.

use super::ebml::write_ebml_element;
use super::mkv_demux::{DemuxResult, Packet, Track, TrackKind};

const ID_EBML: u64 = 0x1A45_DFA3;
const ID_EBML_VERSION: u64 = 0x4286;
const ID_EBML_READ_VERSION: u64 = 0x42F7;
const ID_EBML_MAX_ID_LENGTH: u64 = 0x42F2;
const ID_EBML_MAX_SIZE_LENGTH: u64 = 0x42F3;
const ID_DOC_TYPE: u64 = 0x4282;
const ID_DOC_TYPE_VERSION: u64 = 0x4287;
const ID_DOC_TYPE_READ_VERSION: u64 = 0x4285;

const ID_SEGMENT: u64 = 0x1853_8067;
const ID_INFO: u64 = 0x1549_A966;
const ID_TIMESTAMP_SCALE: u64 = 0x2A_D7B1;
const ID_MUXING_APP: u64 = 0x4D80;
const ID_WRITING_APP: u64 = 0x5741;
const ID_TRACKS: u64 = 0x1654_AE6B;
const ID_TRACK_ENTRY: u64 = 0xAE;
const ID_TRACK_NUMBER: u64 = 0xD7;
const ID_TRACK_UID: u64 = 0x73C5;
const ID_TRACK_TYPE: u64 = 0x83;
const ID_CODEC_ID: u64 = 0x86;
const ID_CODEC_PRIVATE: u64 = 0x63A2;
const ID_VIDEO: u64 = 0xE0;
const ID_PIXEL_WIDTH: u64 = 0xB0;
const ID_PIXEL_HEIGHT: u64 = 0xBA;
const ID_AUDIO: u64 = 0xE1;
const ID_SAMPLING_FREQUENCY: u64 = 0xB5;
const ID_CHANNELS: u64 = 0x9F;
const ID_CLUSTER: u64 = 0x1F43_B675;
const ID_TIMECODE: u64 = 0xE7;
const ID_SIMPLE_BLOCK: u64 = 0xA3;

const TRACK_TYPE_VIDEO: u64 = 1;
const TRACK_TYPE_AUDIO: u64 = 2;

/// Max cluster span, in output ticks — SimpleBlock relative timecodes are
/// signed 16-bit, so a cluster must never exceed +/-32767 ticks from its own
/// Timecode. Kept well under that ceiling.
const MAX_CLUSTER_TICKS: i64 = 30_000;

pub fn remux(demuxed: &DemuxResult) -> Vec<u8> {
    let video = demuxed.tracks.iter().find(|t| t.kind == TrackKind::Video);
    let audio = demuxed.tracks.iter().find(|t| t.kind == TrackKind::Audio);

    let mut out = Vec::with_capacity(demuxed.packets.iter().map(|p| p.data.len()).sum());
    out.extend_from_slice(&write_init(demuxed.timestamp_scale, video, audio));

    let mut clusters = ClusterWriter::default();
    out.extend_from_slice(&clusters.push_packets(&demuxed.packets, video, audio));
    out.extend_from_slice(&clusters.finish());
    out
}

/// EBML header + an *unknown-size* Segment containing Info/Tracks. Written
/// once per stream. Because the Segment size is unknown, subsequent Cluster
/// bytes from `ClusterWriter` can simply be appended after this — no
/// resizing or rewriting needed, which is what makes incremental muxing
/// possible: each `SourceBuffer.appendBuffer()` call on the JS side just
/// extends the same logical byte stream.
pub fn write_init(timestamp_scale: u64, video: Option<&Track>, audio: Option<&Track>) -> Vec<u8> {
    let mut out = Vec::new();
    write_ebml_element(&mut out, ID_EBML, &build_ebml_header());

    super::ebml::write_ebml_id(&mut out, ID_SEGMENT);
    out.extend_from_slice(&[0x01, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF]); // unknown size
    write_ebml_element(&mut out, ID_INFO, &build_info(timestamp_scale));
    write_ebml_element(&mut out, ID_TRACKS, &build_tracks(video, audio));
    out
}

/// Stateful Cluster writer: buffers packets into WebM Clusters, splitting on
/// video keyframes / a max time span (see `MAX_CLUSTER_TICKS`) — same
/// bitstream-copy rules whether called once with everything (`remux`) or
/// repeatedly with whatever packets a given demux chunk produced.
#[derive(Default)]
pub struct ClusterWriter {
    cluster_start: Option<i64>,
    cluster_body: Vec<u8>,
}

impl ClusterWriter {
    /// Feeds newly available packets in. Returns the bytes of any Clusters
    /// that were completed (and can be appended to the SourceBuffer) as a
    /// result — typically all but the last, still-open one.
    pub fn push_packets(
        &mut self,
        packets: &[Packet],
        video: Option<&Track>,
        audio: Option<&Track>,
    ) -> Vec<u8> {
        let video_number = video.map(|t| t.number);
        let mut out = Vec::new();

        for packet in packets {
            let is_video = Some(packet.track_number) == video_number;
            let is_relevant = is_video || audio.is_some_and(|a| a.number == packet.track_number);
            if !is_relevant {
                continue;
            }

            let should_start_new_cluster = match self.cluster_start {
                None => true,
                Some(start) => {
                    let span = packet.timestamp - start;
                    span >= MAX_CLUSTER_TICKS
                        || span < 0
                        || (is_video && packet.keyframe && span > 0)
                }
            };

            if should_start_new_cluster && let Some(start) = self.cluster_start {
                flush_cluster(&mut out, start, &self.cluster_body);
                self.cluster_body.clear();
                self.cluster_start = Some(packet.timestamp);
            } else if should_start_new_cluster {
                self.cluster_start = Some(packet.timestamp);
            }

            let start = self.cluster_start.unwrap_or(packet.timestamp);
            write_simple_block(&mut self.cluster_body, packet, start);
        }

        out
    }

    /// Call once at end-of-stream to flush the still-open trailing Cluster.
    pub fn finish(&mut self) -> Vec<u8> {
        let mut out = Vec::new();
        if let Some(start) = self.cluster_start
            && !self.cluster_body.is_empty()
        {
            flush_cluster(&mut out, start, &self.cluster_body);
            self.cluster_body.clear();
        }
        out
    }
}

fn build_ebml_header() -> Vec<u8> {
    let mut h = Vec::new();
    write_ebml_element(&mut h, ID_EBML_VERSION, &uint_bytes(1));
    write_ebml_element(&mut h, ID_EBML_READ_VERSION, &uint_bytes(1));
    write_ebml_element(&mut h, ID_EBML_MAX_ID_LENGTH, &uint_bytes(4));
    write_ebml_element(&mut h, ID_EBML_MAX_SIZE_LENGTH, &uint_bytes(8));
    write_ebml_element(&mut h, ID_DOC_TYPE, b"webm");
    write_ebml_element(&mut h, ID_DOC_TYPE_VERSION, &uint_bytes(2));
    write_ebml_element(&mut h, ID_DOC_TYPE_READ_VERSION, &uint_bytes(2));
    h
}

fn build_info(timestamp_scale: u64) -> Vec<u8> {
    let mut info = Vec::new();
    write_ebml_element(&mut info, ID_TIMESTAMP_SCALE, &uint_bytes(timestamp_scale));
    write_ebml_element(&mut info, ID_MUXING_APP, b"fluxa-core media_demux");
    write_ebml_element(&mut info, ID_WRITING_APP, b"fluxa-core media_demux");
    info
}

fn build_tracks(video: Option<&Track>, audio: Option<&Track>) -> Vec<u8> {
    let mut tracks = Vec::new();
    if let Some(t) = video {
        write_ebml_element(
            &mut tracks,
            ID_TRACK_ENTRY,
            &build_track_entry(t, TRACK_TYPE_VIDEO),
        );
    }
    if let Some(t) = audio {
        write_ebml_element(
            &mut tracks,
            ID_TRACK_ENTRY,
            &build_track_entry(t, TRACK_TYPE_AUDIO),
        );
    }
    tracks
}

fn build_track_entry(track: &Track, track_type: u64) -> Vec<u8> {
    let mut entry = Vec::new();
    write_ebml_element(&mut entry, ID_TRACK_NUMBER, &uint_bytes(track.number));
    write_ebml_element(&mut entry, ID_TRACK_UID, &uint_bytes(track.number));
    write_ebml_element(&mut entry, ID_TRACK_TYPE, &uint_bytes(track_type));
    write_ebml_element(&mut entry, ID_CODEC_ID, track.codec_id.as_bytes());
    if !track.codec_private.is_empty() {
        write_ebml_element(&mut entry, ID_CODEC_PRIVATE, &track.codec_private);
    }
    match track_type {
        TRACK_TYPE_VIDEO => {
            let mut video = Vec::new();
            if let Some(w) = track.width {
                write_ebml_element(&mut video, ID_PIXEL_WIDTH, &uint_bytes(w));
            }
            if let Some(h) = track.height {
                write_ebml_element(&mut video, ID_PIXEL_HEIGHT, &uint_bytes(h));
            }
            write_ebml_element(&mut entry, ID_VIDEO, &video);
        }
        TRACK_TYPE_AUDIO => {
            let mut audio = Vec::new();
            let sampling_frequency = track.sampling_frequency.unwrap_or(48_000.0);
            write_ebml_element(
                &mut audio,
                ID_SAMPLING_FREQUENCY,
                &f64_bytes(sampling_frequency),
            );
            write_ebml_element(
                &mut audio,
                ID_CHANNELS,
                &uint_bytes(track.channels.unwrap_or(2)),
            );
            write_ebml_element(&mut entry, ID_AUDIO, &audio);
        }
        _ => {}
    }
    entry
}

fn flush_cluster(out: &mut Vec<u8>, start_timecode: i64, body: &[u8]) {
    let mut cluster = Vec::with_capacity(body.len() + 16);
    write_ebml_element(
        &mut cluster,
        ID_TIMECODE,
        &uint_bytes(start_timecode.max(0) as u64),
    );
    cluster.extend_from_slice(body);
    write_ebml_element(out, ID_CLUSTER, &cluster);
}

fn write_simple_block(out: &mut Vec<u8>, packet: &Packet, cluster_start: i64) {
    let relative =
        (packet.timestamp - cluster_start).clamp(i16::MIN as i64, i16::MAX as i64) as i16;
    let mut payload = Vec::with_capacity(4 + packet.data.len());
    write_track_number_vint(&mut payload, packet.track_number);
    payload.extend_from_slice(&relative.to_be_bytes());
    payload.push(if packet.keyframe { 0x80 } else { 0x00 });
    payload.extend_from_slice(&packet.data);
    write_ebml_element(out, ID_SIMPLE_BLOCK, &payload);
}

fn write_track_number_vint(out: &mut Vec<u8>, track_number: u64) {
    super::ebml::write_ebml_vint(out, track_number);
}

fn uint_bytes(mut value: u64) -> Vec<u8> {
    if value == 0 {
        return vec![0];
    }
    let mut bytes = Vec::new();
    while value > 0 {
        bytes.push((value & 0xFF) as u8);
        value >>= 8;
    }
    bytes.reverse();
    bytes
}

fn f64_bytes(value: f64) -> Vec<u8> {
    value.to_be_bytes().to_vec()
}
