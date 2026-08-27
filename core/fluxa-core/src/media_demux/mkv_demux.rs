//! Matroska (MKV) demuxer: a whole-buffer `demux()` for tests/small inputs,
//! and an `IncrementalDemuxer` that consumes chunks as they arrive (e.g. from
//! a `fetch()` stream) so playback doesn't have to wait for the whole file.
//!
//! Scope: extracts track metadata and per-track packets for a single video
//! track and a single audio track — enough to remux into WebM/fMP4 for
//! MediaSource Extensions when the inner codecs are already browser-native.
//! Not a general-purpose demuxer: subtitle tracks, multiple audio tracks,
//! chapters and tags are ignored, and laced blocks are dropped rather than
//! unpacked (matches the existing lacing bail-out in
//! `fluxa-streaming-engine/src/dv_rewrite/mkv.rs`; laced audio/video is rare
//! in modern encodes).
//!
//! `IncrementalDemuxer` still buffers each *element* fully before parsing it
//! (a Cluster is typically a few seconds of media, at most a few MB — not
//! the whole file), rather than being a fully zero-copy streaming parser.
//! That's a deliberate tradeoff: it bounds memory to "one cluster", which is
//! what actually matters for not blocking playback start, without the much
//! larger complexity of a byte-at-a-time state machine.

use super::ebml::{EBML_UNKNOWN_SIZE, try_parse_ebml_header};

const ID_SEGMENT: u64 = 0x1853_8067;
const ID_INFO: u64 = 0x1549_A966;
const ID_TIMESTAMP_SCALE: u64 = 0x2A_D7B1;
const ID_TRACKS: u64 = 0x1654_AE6B;
const ID_TRACK_ENTRY: u64 = 0xAE;
const ID_TRACK_NUMBER: u64 = 0xD7;
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
const ID_BLOCK_GROUP: u64 = 0xA0;
const ID_BLOCK: u64 = 0xA1;
const ID_REFERENCE_BLOCK: u64 = 0xFB;

const TRACK_TYPE_VIDEO: u64 = 1;
const TRACK_TYPE_AUDIO: u64 = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrackKind {
    Video,
    Audio,
}

#[derive(Debug, Clone)]
pub struct Track {
    pub number: u64,
    pub kind: TrackKind,
    pub codec_id: String,
    pub codec_private: Vec<u8>,
    pub width: Option<u64>,
    pub height: Option<u64>,
    pub sampling_frequency: Option<f64>,
    pub channels: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct Packet {
    pub track_number: u64,
    /// Absolute timestamp in `timestamp_scale` ticks.
    pub timestamp: i64,
    pub keyframe: bool,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct DemuxResult {
    /// Nanoseconds represented by one timestamp tick (Matroska `TimestampScale`).
    pub timestamp_scale: u64,
    pub tracks: Vec<Track>,
    pub packets: Vec<Packet>,
}

#[derive(Debug)]
pub enum DemuxError {
    NoSegment,
    Truncated,
}

pub fn demux(input: &[u8]) -> Result<DemuxResult, DemuxError> {
    let segment = find_segment(input).ok_or(DemuxError::NoSegment)?;

    let mut timestamp_scale: u64 = 1_000_000;
    let mut tracks: Vec<Track> = Vec::new();
    let mut packets: Vec<Packet> = Vec::new();

    let mut pos = 0usize;
    while pos < segment.len() {
        let Some((id, size, hlen)) = try_parse_ebml_header(&segment[pos..]) else {
            break;
        };
        let content_start = pos + hlen;
        let content_end = if size == EBML_UNKNOWN_SIZE {
            find_unknown_size_end(
                segment,
                content_start,
                &[ID_CLUSTER, ID_INFO, ID_TRACKS],
                true,
            )
            .unwrap_or(segment.len())
        } else {
            (content_start + size as usize).min(segment.len())
        };
        let Some(content) = segment.get(content_start..content_end) else {
            break;
        };

        match id {
            ID_INFO => {
                if let Some(scale) = read_uint_child(content, ID_TIMESTAMP_SCALE) {
                    timestamp_scale = scale;
                }
            }
            ID_TRACKS => {
                tracks = parse_tracks(content);
            }
            ID_CLUSTER => {
                parse_cluster(content, &tracks, &mut packets);
            }
            _ => {}
        }

        pos = content_end;
    }

    Ok(DemuxResult {
        timestamp_scale,
        tracks,
        packets,
    })
}

fn find_segment(input: &[u8]) -> Option<&[u8]> {
    let mut pos = 0usize;
    while pos < input.len() {
        let (id, size, hlen) = try_parse_ebml_header(&input[pos..])?;
        let content_start = pos + hlen;
        let content_end = if size == EBML_UNKNOWN_SIZE {
            input.len()
        } else {
            (content_start + size as usize).min(input.len())
        };
        if id == ID_SEGMENT {
            return input.get(content_start..content_end);
        }
        pos = content_end;
    }
    None
}

/// Bound an unknown-size element by scanning forward for the next byte
/// offset that parses as a header whose ID is a plausible sibling at this
/// level. This is the standard trick for streamed (unknown-size) Matroska
/// elements; it is a heuristic, not a guarantee, but real-world muxers only
/// ever follow an unknown-size Cluster/Segment with another top-level ID, so
/// in practice this finds the correct boundary.
///
/// Returns `None` when no boundary is found yet and `is_eof` is false — the
/// caller should wait for more bytes rather than guessing. At EOF, an
/// unresolved unknown-size element simply ends at the end of the buffer.
fn find_unknown_size_end(
    buf: &[u8],
    start: usize,
    sibling_ids: &[u64],
    is_eof: bool,
) -> Option<usize> {
    let mut i = start;
    while i < buf.len() {
        if let Some((id, _, _)) = try_parse_ebml_header(&buf[i..])
            && sibling_ids.contains(&id)
        {
            return Some(i);
        }
        i += 1;
    }
    is_eof.then_some(buf.len())
}

fn read_uint_child(buf: &[u8], want_id: u64) -> Option<u64> {
    let mut pos = 0usize;
    while pos < buf.len() {
        let (id, size, hlen) = try_parse_ebml_header(&buf[pos..])?;
        if size == EBML_UNKNOWN_SIZE {
            return None;
        }
        let content_start = pos + hlen;
        let content_end = (content_start + size as usize).min(buf.len());
        if id == want_id {
            return Some(be_bytes_to_u64(buf.get(content_start..content_end)?));
        }
        pos = content_end;
    }
    None
}

fn be_bytes_to_u64(bytes: &[u8]) -> u64 {
    let mut v = 0u64;
    for &b in bytes {
        v = (v << 8) | b as u64;
    }
    v
}

fn be_bytes_to_f64(bytes: &[u8]) -> f64 {
    match bytes.len() {
        4 => {
            let mut arr = [0u8; 4];
            arr.copy_from_slice(bytes);
            f32::from_be_bytes(arr) as f64
        }
        8 => {
            let mut arr = [0u8; 8];
            arr.copy_from_slice(bytes);
            f64::from_be_bytes(arr)
        }
        _ => 0.0,
    }
}

fn parse_tracks(buf: &[u8]) -> Vec<Track> {
    let mut tracks = Vec::new();
    let mut pos = 0usize;
    while pos < buf.len() {
        let Some((id, size, hlen)) = try_parse_ebml_header(&buf[pos..]) else {
            break;
        };
        if size == EBML_UNKNOWN_SIZE {
            break;
        }
        let content_start = pos + hlen;
        let content_end = (content_start + size as usize).min(buf.len());
        if id == ID_TRACK_ENTRY
            && let Some(entry) = buf.get(content_start..content_end)
            && let Some(track) = parse_track_entry(entry)
        {
            tracks.push(track);
        }
        pos = content_end;
    }
    tracks
}

fn parse_track_entry(buf: &[u8]) -> Option<Track> {
    let mut number = None;
    let mut track_type = None;
    let mut codec_id = None;
    let mut codec_private = Vec::new();
    let mut width = None;
    let mut height = None;
    let mut sampling_frequency = None;
    let mut channels = None;

    let mut pos = 0usize;
    while pos < buf.len() {
        let (id, size, hlen) = try_parse_ebml_header(&buf[pos..])?;
        if size == EBML_UNKNOWN_SIZE {
            break;
        }
        let content_start = pos + hlen;
        let content_end = (content_start + size as usize).min(buf.len());
        let content = buf.get(content_start..content_end)?;

        match id {
            ID_TRACK_NUMBER => number = Some(be_bytes_to_u64(content)),
            ID_TRACK_TYPE => track_type = Some(be_bytes_to_u64(content)),
            ID_CODEC_ID => codec_id = String::from_utf8(content.to_vec()).ok(),
            ID_CODEC_PRIVATE => codec_private = content.to_vec(),
            ID_VIDEO => {
                width = read_uint_child(content, ID_PIXEL_WIDTH);
                height = read_uint_child(content, ID_PIXEL_HEIGHT);
            }
            ID_AUDIO => {
                sampling_frequency = read_float_child(content, ID_SAMPLING_FREQUENCY);
                channels = read_uint_child(content, ID_CHANNELS);
            }
            _ => {}
        }
        pos = content_end;
    }

    let kind = match track_type? {
        TRACK_TYPE_VIDEO => TrackKind::Video,
        TRACK_TYPE_AUDIO => TrackKind::Audio,
        _ => return None,
    };

    Some(Track {
        number: number?,
        kind,
        codec_id: codec_id?,
        codec_private,
        width,
        height,
        sampling_frequency,
        channels,
    })
}

fn read_float_child(buf: &[u8], want_id: u64) -> Option<f64> {
    let mut pos = 0usize;
    while pos < buf.len() {
        let (id, size, hlen) = try_parse_ebml_header(&buf[pos..])?;
        if size == EBML_UNKNOWN_SIZE {
            return None;
        }
        let content_start = pos + hlen;
        let content_end = (content_start + size as usize).min(buf.len());
        if id == want_id {
            return Some(be_bytes_to_f64(buf.get(content_start..content_end)?));
        }
        pos = content_end;
    }
    None
}

fn parse_cluster(buf: &[u8], tracks: &[Track], packets: &mut Vec<Packet>) {
    let mut cluster_timecode: i64 = 0;
    let mut pos = 0usize;
    while pos < buf.len() {
        let Some((id, size, hlen)) = try_parse_ebml_header(&buf[pos..]) else {
            break;
        };
        if size == EBML_UNKNOWN_SIZE {
            break;
        }
        let content_start = pos + hlen;
        let content_end = (content_start + size as usize).min(buf.len());
        let Some(content) = buf.get(content_start..content_end) else {
            break;
        };

        match id {
            ID_TIMECODE => cluster_timecode = be_bytes_to_u64(content) as i64,
            ID_SIMPLE_BLOCK => {
                if let Some(packet) = parse_block_payload(content, cluster_timecode, tracks, None) {
                    packets.push(packet);
                }
            }
            ID_BLOCK_GROUP => {
                if let Some(packet) = parse_block_group(content, cluster_timecode, tracks) {
                    packets.push(packet);
                }
            }
            _ => {}
        }
        pos = content_end;
    }
}

fn parse_block_group(buf: &[u8], cluster_timecode: i64, tracks: &[Track]) -> Option<Packet> {
    let mut block: Option<&[u8]> = None;
    let mut has_reference = false;
    let mut pos = 0usize;
    while pos < buf.len() {
        let (id, size, hlen) = try_parse_ebml_header(&buf[pos..])?;
        if size == EBML_UNKNOWN_SIZE {
            break;
        }
        let content_start = pos + hlen;
        let content_end = (content_start + size as usize).min(buf.len());
        if id == ID_BLOCK {
            block = buf.get(content_start..content_end);
        } else if id == ID_REFERENCE_BLOCK {
            has_reference = true;
        }
        pos = content_end;
    }
    let block = block?;
    // Block (inside BlockGroup) has no keyframe flag of its own — Matroska
    // convention is: no ReferenceBlock child means keyframe.
    parse_block_payload(block, cluster_timecode, tracks, Some(!has_reference))
}

/// Parses a (Simple)Block payload: track-number vint, 2-byte relative
/// timecode, 1-byte flags, then frame data. Laced blocks (flags bits 0x06 !=
/// 0) are dropped — see module docs.
///
/// `block_group_keyframe` is `None` for a SimpleBlock (keyframe comes from
/// its own flags byte) or `Some(from_reference_block_absence)` for a Block
/// inside a BlockGroup (which carries no such flag).
fn parse_block_payload(
    block: &[u8],
    cluster_timecode: i64,
    tracks: &[Track],
    block_group_keyframe: Option<bool>,
) -> Option<Packet> {
    let (track_number, vint_len) = crate::media_demux::ebml::parse_ebml_vint(block)?;
    let rest = block.get(vint_len..)?;
    if rest.len() < 3 {
        return None;
    }
    let rel_timecode = i16::from_be_bytes([rest[0], rest[1]]) as i64;
    let flags = rest[2];
    let lacing = (flags >> 1) & 0x03;
    if lacing != 0 {
        return None;
    }
    let frame = rest.get(3..)?;

    let is_video = tracks
        .iter()
        .any(|t| t.number == track_number && t.kind == TrackKind::Video);
    let keyframe = if is_video {
        block_group_keyframe.unwrap_or((flags & 0x80) != 0)
    } else {
        true
    };

    Some(Packet {
        track_number,
        timestamp: cluster_timecode + rel_timecode,
        keyframe,
        data: frame.to_vec(),
    })
}

/// Result of one `IncrementalDemuxer::push`/`flush` call: whatever became
/// newly available from the bytes fed in so far.
#[derive(Debug, Default)]
pub struct IncrementalStep {
    /// `true` on the call where `tracks` first became available (Matroska
    /// puts Tracks before the first Cluster in every muxer this targets).
    pub tracks_ready: bool,
    pub packets: Vec<Packet>,
}

pub struct IncrementalDemuxer {
    buf: Vec<u8>,
    in_segment: bool,
    pub timestamp_scale: u64,
    pub tracks: Vec<Track>,
}

impl Default for IncrementalDemuxer {
    fn default() -> Self {
        Self {
            buf: Vec::new(),
            in_segment: false,
            timestamp_scale: 1_000_000,
            tracks: Vec::new(),
        }
    }
}

impl IncrementalDemuxer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, chunk: &[u8]) -> IncrementalStep {
        self.buf.extend_from_slice(chunk);
        self.drive(false)
    }

    /// Call once at end-of-stream to flush any trailing unknown-size element
    /// (a Cluster whose size we could only determine by reaching EOF).
    pub fn flush(&mut self) -> IncrementalStep {
        self.drive(true)
    }

    fn drive(&mut self, is_eof: bool) -> IncrementalStep {
        let mut step = IncrementalStep::default();
        loop {
            let Some((id, size, hlen)) = try_parse_ebml_header(&self.buf) else {
                break;
            };

            if !self.in_segment {
                if id == ID_SEGMENT {
                    self.buf.drain(..hlen);
                    self.in_segment = true;
                    continue;
                }
                // Non-Segment top-level element (the EBML header, or Cues/
                // SeekHead siblings if Segment isn't first) — skip once its
                // full extent is known.
                let Some(end) = (if size == EBML_UNKNOWN_SIZE {
                    is_eof.then_some(self.buf.len())
                } else {
                    Some(hlen + size as usize)
                }) else {
                    break;
                };
                if self.buf.len() < end {
                    if is_eof {
                        self.buf.clear();
                    }
                    break;
                }
                self.buf.drain(..end);
                continue;
            }

            let content_start = hlen;
            let Some(end) = (if size == EBML_UNKNOWN_SIZE {
                find_unknown_size_end(
                    &self.buf,
                    content_start,
                    &[ID_CLUSTER, ID_INFO, ID_TRACKS],
                    is_eof,
                )
            } else {
                Some(content_start + size as usize)
            }) else {
                break;
            };
            if self.buf.len() < end {
                if !is_eof {
                    break;
                }
            }
            let end = end.min(self.buf.len());
            if end < content_start {
                break;
            }
            let content = self.buf[content_start..end].to_vec();

            match id {
                ID_INFO => {
                    if let Some(scale) = read_uint_child(&content, ID_TIMESTAMP_SCALE) {
                        self.timestamp_scale = scale;
                    }
                }
                ID_TRACKS => {
                    self.tracks = parse_tracks(&content);
                    step.tracks_ready = true;
                }
                ID_CLUSTER => {
                    parse_cluster(&content, &self.tracks, &mut step.packets);
                }
                _ => {}
            }
            self.buf.drain(..end);
        }
        step
    }
}
