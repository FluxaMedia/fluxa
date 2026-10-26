//! MP4 keyframe indexing from the moov atom.
//!
//! The atom is located from a small head request, or from the file tail when
//! the writer placed moov after mdat, then stts/stsc/stsz/stco/stss rebuild
//! the sample table. Only sync samples are kept, and each is fetched as a
//! single range covering the raw sample bytes.

use crate::decode::AnnexBFormat;
use crate::{AnnexBCall, Generator, Keyframe, KeyframeIndex, SourceKind};

pub enum Mp4Error {
    NotMp4,
    Truncated(&'static str),
    Unsupported(&'static str),
}

impl Mp4Error {
    pub fn message(&self) -> &'static str {
        match self {
            Mp4Error::NotMp4 => "not an MP4 file",
            Mp4Error::Truncated(m) => m,
            Mp4Error::Unsupported(m) => m,
        }
    }
}

type R<T> = std::result::Result<T, Mp4Error>;

struct BoxRef {
    kind: [u8; 4],
    content_start: usize,
    content_end: usize,
    next: usize,
}

fn walk(data: &[u8], from: usize, to: usize) -> Vec<BoxRef> {
    let mut boxes = Vec::new();
    let mut pos = from;
    let limit = to.min(data.len());
    while pos + 8 <= limit {
        let size32 = u32::from_be_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]]);
        let mut kind = [0u8; 4];
        kind.copy_from_slice(&data[pos + 4..pos + 8]);
        let (header, size) = if size32 == 1 {
            if pos + 16 > limit {
                break;
            }
            let mut be = [0u8; 8];
            be.copy_from_slice(&data[pos + 8..pos + 16]);
            (16usize, u64::from_be_bytes(be) as usize)
        } else if size32 == 0 {
            (8usize, limit - pos)
        } else {
            (8usize, size32 as usize)
        };
        if size < header || pos + size > limit {
            break;
        }
        boxes.push(BoxRef {
            kind,
            content_start: pos + header,
            content_end: pos + size,
            next: pos + size,
        });
        pos += size;
    }
    boxes
}

fn child<'a>(data: &'a [u8], from: usize, to: usize, kind: &[u8; 4]) -> Option<BoxRef> {
    walk(data, from, to).into_iter().find(|b| &b.kind == kind)
}

fn u32_at(data: &[u8], pos: usize) -> u32 {
    u32::from_be_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]])
}

fn u64_at(data: &[u8], pos: usize) -> u64 {
    let mut be = [0u8; 8];
    be.copy_from_slice(&data[pos..pos + 8]);
    u64::from_be_bytes(be)
}

pub struct KeyframeTable;

impl KeyframeTable {
    pub async fn parse(head: &[u8], g: &Generator) -> R<KeyframeIndex> {
        let top = walk(head, 0, head.len());
        let is_mp4 = head.len() >= 12
            && (&head[4..8] == b"ftyp"
                || top.iter().any(|b| &b.kind == b"mdat" || &b.kind == b"moov"));
        if !is_mp4 {
            return Err(Mp4Error::NotMp4);
        }
        let moov_bytes: Vec<u8> = match top.iter().find(|b| &b.kind == b"moov") {
            Some(b) => head[b.content_start - 8..b.content_end].to_vec(),
            None => {
                let total = g.fetch_file_size().await.map_err(|_| Mp4Error::NotMp4)?;
                if total < 16 {
                    return Err(Mp4Error::NotMp4);
                }
                let tail_len = (total as usize).min(1 << 20);
                let tail = g
                    .fetch_exact(total - tail_len as u64, tail_len as u64)
                    .await
                    .map_err(|_| Mp4Error::NotMp4)?;
                let magic = find_moov_magic(&tail)?;
                tail[magic.0..magic.1].to_vec()
            }
        };
        let table = Self::build(&moov_bytes)?;
        if table.entries.is_empty() {
            return Err(Mp4Error::Unsupported("no sync samples in track"));
        }
        Ok(KeyframeIndex {
            duration: table.duration,
            keyframes: table
                .entries
                .iter()
                .map(|e| Keyframe {
                    time: e.time,
                    offset: e.offset,
                    len: e.size.min(1 << 20) as u32,
                    annex_b: table.annex_b.clone(),
                })
                .collect(),
            kind: SourceKind::Mp4,
            payload_prefix: Vec::new(),
        })
    }

    fn build(moov: &[u8]) -> R<Built> {
        let mvhd = child(moov, 8, moov.len(), b"mvhd").ok_or(Mp4Error::Truncated("no mvhd"))?;
        let c = &moov[mvhd.content_start..mvhd.content_end];
        let (timescale, duration) = if c[0] == 1 {
            (u32_at(c, 20), u64_at(c, 24) as f64)
        } else {
            (u32_at(c, 12), u32_at(c, 16) as f64)
        };
        let timescale = timescale.max(1) as f64;

        let traks: Vec<(usize, usize)> = walk(moov, 8, moov.len())
            .into_iter()
            .filter(|b| &b.kind == b"trak")
            .map(|b| (b.content_start - 8, b.next))
            .collect();
        let mut chosen: Option<(usize, usize, AnnexBFormat)> = None;
        for (start, end) in traks {
            let Some(stbl) = child(moov, start + 8, end, b"mdia")
                .and_then(|m| child(moov, m.content_start, m.content_end, b"minf"))
                .and_then(|m| child(moov, m.content_start, m.content_end, b"stbl"))
            else {
                continue;
            };
            let stbl_range = (stbl.content_start, stbl.content_end);
            let Some(stsd) = child(moov, stbl_range.0, stbl_range.1, b"stsd") else {
                continue;
            };
            let entry_start = stsd.content_start + 8;
            if entry_start + 4 > moov.len() {
                continue;
            }
            let format = match &moov[entry_start..entry_start + 4] {
                b"avc1" | b"avc3" => AnnexBFormat::H264,
                b"hev1" | b"hvc1" => AnnexBFormat::Hevc,
                _ => continue,
            };
            chosen = Some((start, end, format));
            break;
        }
        let Some((trak_start, trak_end, format)) = chosen else {
            return Err(Mp4Error::Unsupported("no avc1/hev1 video track"));
        };
        let stbl = child(moov, trak_start + 8, trak_end, b"mdia")
            .and_then(|m| child(moov, m.content_start, m.content_end, b"minf"))
            .and_then(|m| child(moov, m.content_start, m.content_end, b"stbl"))
            .ok_or(Mp4Error::Truncated("no stbl"))?;
        let stbl = &moov[stbl.content_start..stbl.content_end];

        let stsd = child(stbl, 8, stbl.len(), b"stsd").ok_or(Mp4Error::Truncated("no stsd"))?;
        let entry_start = stsd.content_start + 8;
        let sample_children = walk(stbl, entry_start + 78, stsd.content_end);
        let codec_box = sample_children
            .iter()
            .find(|b| &b.kind == b"avcC" || &b.kind == b"hvcC")
            .ok_or(Mp4Error::Unsupported("no avcC/hvcC"))?;
        let annex_b = AnnexBCall {
            format,
            data: stbl[codec_box.content_start..codec_box.content_end].to_vec(),
        };

        let stsz = child(stbl, 8, stbl.len(), b"stsz").ok_or(Mp4Error::Truncated("no stsz"))?;
        let d = &stbl[stsz.content_start..stsz.content_end];
        let uniform = u32_at(d, 4) as usize;
        let sizes: Vec<usize> = if uniform != 0 {
            vec![uniform; u32_at(d, 8) as usize]
        } else {
            let count = u32_at(d, 8) as usize;
            (0..count).map(|i| u32_at(d, 12 + i * 4) as usize).collect()
        };

        let stts = child(stbl, 8, stbl.len(), b"stts").ok_or(Mp4Error::Truncated("no stts"))?;
        let d = &stbl[stts.content_start..stts.content_end];
        let count = u32_at(d, 4) as usize;
        let mut deltas: Vec<u64> = Vec::new();
        for i in 0..count {
            let run = u32_at(d, 8 + i * 8) as usize;
            let delta = u32_at(d, 12 + i * 8) as u64;
            deltas.extend(std::iter::repeat_n(delta, run));
        }

        let chunk_offsets: Vec<u64> = if let Some(co64) = child(stbl, 8, stbl.len(), b"co64") {
            let d = &stbl[co64.content_start..co64.content_end];
            let count = u32_at(d, 4) as usize;
            (0..count).map(|i| u64_at(d, 8 + i * 8)).collect()
        } else {
            let stco = child(stbl, 8, stbl.len(), b"stco").ok_or(Mp4Error::Truncated("no stco"))?;
            let d = &stbl[stco.content_start..stco.content_end];
            let count = u32_at(d, 4) as usize;
            (0..count).map(|i| u32_at(d, 8 + i * 4) as u64).collect()
        };

        let stsc = child(stbl, 8, stbl.len(), b"stsc").ok_or(Mp4Error::Truncated("no stsc"))?;
        let d = &stbl[stsc.content_start..stsc.content_end];
        let count = u32_at(d, 4) as usize;
        let runs: Vec<(usize, usize)> = (0..count)
            .map(|i| {
                (
                    u32_at(d, 8 + i * 12) as usize,
                    u32_at(d, 12 + i * 12) as usize,
                )
            })
            .collect();
        let mut chunks: Vec<usize> = Vec::new();
        for (i, &(first_chunk, per_chunk)) in runs.iter().enumerate() {
            let last_chunk = runs
                .get(i + 1)
                .map(|&(next, _)| next)
                .unwrap_or(chunk_offsets.len() + 1);
            for _chunk in first_chunk..last_chunk {
                chunks.push(per_chunk);
            }
        }
        let _ = chunk_offsets.clone();

        let sync: Option<Vec<usize>> = child(stbl, 8, stbl.len(), b"stss").map(|stss| {
            let d = &stbl[stss.content_start..stss.content_end];
            let count = u32_at(d, 4) as usize;
            (0..count).map(|i| u32_at(d, 8 + i * 4) as usize).collect()
        });

        let mut entries: Vec<SampleEntry> = Vec::new();
        let mut time: u64 = 0;
        let mut sample_index = 0usize;
        for (chunk_idx, &per_chunk) in chunks.iter().enumerate() {
            let Some(&offset0) = chunk_offsets.get(chunk_idx) else {
                break;
            };
            let mut cursor = offset0;
            for _ in 0..per_chunk {
                let Some(&size) = sizes.get(sample_index) else {
                    break;
                };
                let is_sync = match &sync {
                    Some(list) => list.binary_search(&(sample_index + 1)).is_ok(),
                    None => true,
                };
                if is_sync {
                    entries.push(SampleEntry {
                        time: time as f64 / timescale,
                        offset: cursor,
                        size,
                    });
                }
                cursor += size as u64;
                time += deltas.get(sample_index).copied().unwrap_or(0);
                sample_index += 1;
            }
        }
        Ok(Built {
            duration: entries
                .last()
                .map(|e| e.time)
                .unwrap_or(duration / timescale),
            entries,
            annex_b: Some(annex_b),
        })
    }
}

struct SampleEntry {
    time: f64,
    offset: u64,
    size: usize,
}

pub struct Built {
    pub duration: f64,
    pub entries: Vec<SampleEntry>,
    pub annex_b: Option<AnnexBCall>,
}

/// Finds a plausibly-sized moov box starting anywhere in a tail slice.
fn find_moov_magic(tail: &[u8]) -> R<(usize, usize)> {
    for pos in 0..tail.len().saturating_sub(8) {
        if &tail[pos + 4..pos + 8] != b"moov" {
            continue;
        }
        let size =
            u32::from_be_bytes([tail[pos], tail[pos + 1], tail[pos + 2], tail[pos + 3]]) as usize;
        if size >= 8 && pos + size <= tail.len() {
            return Ok((pos, pos + size));
        }
    }
    Err(Mp4Error::Truncated("moov atom not found in tail"))
}
