//! Matroska (MKV/WebM) keyframe indexing through the Cues element.
//!
//! The head request covers the EBML header, Segment meta, SeekHead, Info and
//! Tracks; SeekHead points at the Cues element which lists cluster offsets.
//! A decoded preview payload is synthesized as header bytes followed by the
//! Segment id with an unknown-size marker and the target cluster's bytes, a
//! stream ffmpeg accepts directly.

use crate::{Generator, Keyframe, KeyframeIndex, SourceKind};

pub enum MkvError {
    NotMatroska,
    Truncated(&'static str),
}

impl MkvError {
    pub fn message(&self) -> &'static str {
        match self {
            MkvError::NotMatroska => "not a Matroska file",
            MkvError::Truncated(m) => m,
        }
    }
}

type R<T> = std::result::Result<T, MkvError>;

fn vint(data: &[u8], pos: usize, keep_marker: bool) -> Option<(u64, usize)> {
    let first = *data.get(pos)?;
    let mut len = 1usize;
    let mut mask = 0x80u8;
    while len <= 8 && first & mask == 0 {
        mask >>= 1;
        len += 1;
    }
    if len > 8 || pos + len > data.len() {
        return None;
    }
    let mut value = if keep_marker {
        first as u64
    } else {
        (first & (mask - 1)) as u64
    };
    for byte in &data[pos + 1..pos + len] {
        value = (value << 8) | *byte as u64;
    }
    Some((value, len))
}

struct Elem {
    id: u64,
    content_start: usize,
    content_end: usize,
    next: usize,
}

fn children(data: &[u8], from: usize, to: usize) -> Vec<Elem> {
    let mut out = Vec::new();
    let mut pos = from;
    let limit = to.min(data.len());
    while pos < limit {
        let Some((id, id_len)) = vint(data, pos, true) else {
            break;
        };
        let Some((size, size_len)) = vint(data, pos + id_len, false) else {
            break;
        };
        let content_start = pos + id_len + size_len;
        let size = if size == 0x01FF_FFFF_FFFF_FFFF {
            limit - content_start
        } else {
            size as usize
        };
        if content_start + size > limit {
            break;
        }
        out.push(Elem {
            id,
            content_start,
            content_end: content_start + size,
            next: content_start + size,
        });
        pos = content_start + size;
    }
    out
}

fn find<'a>(elems: &'a [Elem], id: u64) -> Option<&'a Elem> {
    elems.iter().find(|e| e.id == id)
}

fn uint(data: &[u8], e: &Elem) -> u64 {
    let mut value = 0u64;
    for &b in &data[e.content_start..e.content_end] {
        value = (value << 8) | b as u64;
    }
    value
}

fn float(data: &[u8], e: &Elem) -> f64 {
    let d = &data[e.content_start..e.content_end];
    match d.len() {
        4 => f32::from_be_bytes([d[0], d[1], d[2], d[3]]) as f64,
        8 => f64::from_be_bytes([d[0], d[1], d[2], d[3], d[4], d[5], d[6], d[7]]),
        _ => 0.0,
    }
}

const ID_SEGMENT: u64 = 0x18538067;
const ID_SEEK_HEAD: u64 = 0x114D9B74;
const ID_INFO: u64 = 0x1549A966;
const ID_CUES: u64 = 0x1C53BB6B;
const ID_SEEK: u64 = 0x4DBB;
const ID_SEEK_ID: u64 = 0x53AB;
const ID_SEEK_POSITION: u64 = 0x53AC;
const ID_TIMECODE_SCALE: u64 = 0x2AD7B1;
const ID_DURATION: u64 = 0x4489;
const ID_CUE_POINT: u64 = 0xBB;
const ID_CUE_TIME: u64 = 0xB3;
const ID_CUE_TRACK_POSITIONS: u64 = 0xB7;
const ID_CUE_CLUSTER_POSITION: u64 = 0xF1;

const SEGMENT_ID_BYTES: [u8; 4] = [0x18, 0x53, 0x80, 0x67];
const UNKNOWN_SIZE: [u8; 8] = [0x01, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF];

pub struct KeyframeTable;

struct CuePoint {
    time: u64,
    cluster_pos: u64,
}

impl KeyframeTable {
    pub async fn parse(head: &[u8], g: &Generator) -> R<KeyframeIndex> {
        let top = children(head, 0, head.len());
        let is_mkv = head.len() >= 4 && head[0] == 0x1A && head[1] == 0x45 && head[2] == 0xDF && head[3] == 0xA3;
        if !is_mkv {
            return Err(MkvError::NotMatroska);
        }
        let segment = find(&top, ID_SEGMENT).ok_or(MkvError::Truncated("no segment"))?;
        let seg_data_start = segment.content_start;

        let seek_head = find(&children(head, seg_data_start, head.len()), ID_SEEK_HEAD)
            .ok_or(MkvError::Truncated("no seek head"))?;
        let mut info_pos: Option<u64> = None;
        let mut cues_pos: Option<u64> = None;
        for seek in &children(head, seek_head.content_start, seek_head.content_end) {
            if seek.id != ID_SEEK {
                continue;
            }
            let parts = children(head, seek.content_start, seek.content_end);
            let (Some(id_elem), Some(pos_elem)) = (find(&parts, ID_SEEK_ID), find(&parts, ID_SEEK_POSITION)) else {
                continue;
            };
            let id_bytes = &head[id_elem.content_start..id_elem.content_end];
            let id = id_bytes
                .iter()
                .fold(0u64, |acc, b| (acc << 8) | *b as u64);
            let pos = uint(head, pos_elem);
            if id == ID_INFO {
                info_pos = Some(seg_data_start as u64 + pos);
            } else if id == ID_CUES {
                cues_pos = Some(seg_data_start as u64 + pos);
            }
        }
        let Some(cues_abs) = cues_pos else {
            return Err(MkvError::Truncated("no cues"));
        };

        let info_data = fetch_element(head, g, info_pos.unwrap_or(seg_data_start as u64), ID_INFO).await?;
        let mut timecode_scale = 1_000_000u64;
        let mut duration = 0.0f64;
        for e in &children(&info_data, header_len(&info_data), info_data.len()) {
            if e.id == ID_TIMECODE_SCALE {
                timecode_scale = uint(&info_data, e).max(1);
            } else if e.id == ID_DURATION {
                duration = float(&info_data, e);
            }
        }

        let cues = fetch_element(head, g, cues_abs, ID_CUES).await?;
        let mut points: Vec<CuePoint> = Vec::new();
        for point in &children(&cues, header_len(&cues), cues.len()) {
            if point.id != ID_CUE_POINT {
                continue;
            }
            let parts = children(&cues, point.content_start, point.content_end);
            let (Some(time_e), Some(pos_e)) = (
                find(&parts, ID_CUE_TIME),
                find(&parts, ID_CUE_TRACK_POSITIONS),
            ) else {
                continue;
            };
            let Some(cluster_pos_e) = find(
                &children(&cues, pos_e.content_start, pos_e.content_end),
                ID_CUE_CLUSTER_POSITION,
            ) else {
                continue;
            };
            points.push(CuePoint {
                time: uint(&cues, time_e),
                cluster_pos: uint(&cues, cluster_pos_e),
            });
        }
        if points.is_empty() {
            return Err(MkvError::Truncated("empty cues"));
        }

        let mut header_prefix = head[..seg_data_start].to_vec();
        header_prefix.extend_from_slice(&SEGMENT_ID_BYTES);
        header_prefix.extend_from_slice(&UNKNOWN_SIZE);

        let mut keyframes: Vec<Keyframe> = Vec::new();
        for (i, point) in points.iter().enumerate() {
            let offset = seg_data_start as u64 + point.cluster_pos;
            let len = if i + 1 < points.len() {
                (seg_data_start as u64 + points[i + 1].cluster_pos - offset).min(1 << 20) as u32
            } else {
                256 * 1024
            };
            keyframes.push(Keyframe {
                time: point.time as f64 * timecode_scale as f64 / 1_000_000_000.0,
                offset,
                len,
                annex_b: None,
            });
        }
        if duration == 0.0 {
            duration = keyframes.last().map(|k| k.time).unwrap_or(0.0);
        }
        Ok(KeyframeIndex {
            duration: duration * timecode_scale as f64 / 1_000_000_000.0,
            keyframes,
            kind: SourceKind::Matroska,
            payload_prefix: header_prefix,
        })
    }
}

fn header_len(data: &[u8]) -> usize {
    let Some((_, id_len)) = vint(data, 0, true) else {
        return 0;
    };
    let Some((_, size_len)) = vint(data, id_len, false) else {
        return 0;
    };
    id_len + size_len
}

/// Fetches an element that may live beyond the head slice, reading its size
/// prefix first and then the exact content range.
async fn fetch_element(head: &[u8], g: &Generator, offset: u64, expect_id: u64) -> R<Vec<u8>> {
    if (offset as usize) < head.len() {
        let elems = children(head, offset as usize, head.len());
        if let Some(e) = elems.first() {
            if e.id == expect_id {
                return Ok(head[(offset as usize)..e.next].to_vec());
            }
        }
    }
    let prefix = g
        .fetch_range(offset, 16)
        .await
        .map_err(|_| MkvError::Truncated("element size unreachable"))?;
    let (_, id_len) = vint(&prefix, 0, true).ok_or(MkvError::Truncated("bad id"))?;
    let (size, size_len) = vint(&prefix, id_len, false).ok_or(MkvError::Truncated("bad size"))?;
    let total = (id_len + size_len) as u64 + size;
    Ok(g.fetch_exact(offset, total).await.map_err(|_| MkvError::Truncated("element fetch failed"))?)
}
