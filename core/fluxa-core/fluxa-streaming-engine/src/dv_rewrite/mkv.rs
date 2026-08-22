use super::*;
use std::borrow::Cow;

// Low-level EBML primitives (vint/ID parsing, element encoding) live in
// fluxa_core::media_demux::ebml now, shared with the in-browser MKV demuxer.
// `write_id` keeps its old local name here via an alias to avoid touching
// every call site below.
pub(crate) use fluxa_core::media_demux::ebml::{
    ebml_id_width, ebml_vint_width, parse_ebml_id, parse_ebml_vint, try_parse_ebml_header, write_ebml_element,
    write_ebml_id as write_id, write_ebml_vint,
};
#[cfg(test)]
pub(crate) use fluxa_core::media_demux::ebml::{encode_ebml_element, encode_ebml_vint};

// BlockGroup processor
/// Process a complete buffered BlockGroup payload.
/// Extracts RPU from BlockAdditions, converts it, injects it into Block frame
/// data, and removes BlockAdditions from the output.
///
/// Returns `(processed_data, rpu_injected_count)`.
pub(crate) fn process_block_group_data(
    data: &[u8],
    rpu_mode: u8,
    zero_level5: bool,
) -> (Cow<'_, [u8]>, u32) {
    // Parse all child elements of this BlockGroup.
    let mut block_offset: Option<usize> = None;
    let mut block_end: Option<usize> = None;
    let mut block_additions: Option<&[u8]> = None;
    let mut pos = 0;

    while pos < data.len() {
        let Some((id, data_size, hlen)) = try_parse_ebml_header(&data[pos..]) else {
            break;
        };
        if data_size == EBML_UNKNOWN_SIZE {
            // Can't safely walk past unknown-size children; return original.
            return (Cow::Borrowed(data), 0);
        }
        let child_start = pos + hlen;
        let child_end = child_start + data_size as usize;
        if child_end > data.len() {
            break;
        }

        if id == EBML_BLOCK {
            block_offset = Some(pos);
            block_end = Some(child_end);
        } else if id == EBML_BLOCK_ADDITIONS {
            block_additions = Some(&data[child_start..child_end]);
        }
        pos = child_end;
    }

    // Without a Block element, return original.
    let (block_start, block_data_end) = match (block_offset, block_end) {
        (Some(s), Some(e)) => (s, e),
        _ => return (Cow::Borrowed(data), 0),
    };

    // Parse Block header to find its ID/vint extent so we can replace the element.
    let (_, block_data_size, block_hlen) = match try_parse_ebml_header(&data[block_start..]) {
        Some(h) => h,
        None => return (Cow::Borrowed(data), 0),
    };
    let block_payload_start = block_start + block_hlen;
    let block_payload = &data[block_payload_start..block_data_end];

    // Try to extract an RPU from BlockAdditions.
    let rpu_raw = block_additions.and_then(extract_dv_rpu_from_block_additions);

    // If no RPU or conversion fails, build output without BlockAdditions but
    // with original Block intact.
    let rpu_injected;
    let new_block_payload: Cow<'_, [u8]> = match rpu_raw {
        Some(rpu_nal) => match convert_rpu_nal(&rpu_nal, rpu_mode, zero_level5) {
            Some(converted_rpu) => {
                rpu_injected = 1u32;
                inject_rpu_into_mkv_block(block_payload, &converted_rpu)
            }
            None => {
                rpu_injected = 0;
                Cow::Borrowed(block_payload)
            }
        },
        None => {
            // No RPU found — return data unchanged (BlockAdditions kept if present).
            return (Cow::Borrowed(data), 0);
        }
    };

    // Reconstruct BlockGroup: elements before Block + new Block + elements after
    // Block, skipping BlockAdditions.
    let mut out = Vec::with_capacity(data.len());

    // Elements before Block.
    out.extend_from_slice(&data[..block_start]);

    // New Block element with (possibly updated) payload.
    write_id(&mut out, EBML_BLOCK);
    write_ebml_vint(&mut out, new_block_payload.len() as u64);
    out.extend_from_slice(new_block_payload.as_ref());

    // Elements after Block, skipping BlockAdditions.
    let mut pos = block_data_end;
    while pos < data.len() {
        let Some((id, ds, hlen)) = try_parse_ebml_header(&data[pos..]) else {
            break;
        };
        if ds == EBML_UNKNOWN_SIZE {
            break;
        }
        let elem_end = pos + hlen + ds as usize;
        if elem_end > data.len() {
            break;
        }
        if id != EBML_BLOCK_ADDITIONS {
            out.extend_from_slice(&data[pos..elem_end]);
        }
        pos = elem_end;
    }

    // Update the Block element size in case `new_block_payload` changed length.
    // (We already wrote the correct vint above.)
    let _ = block_data_size; // used for reference only

    (Cow::Owned(out), rpu_injected)
}

/// Walk BlockAdditions → BlockMore → BlockAddID == 1 → BlockAdditional.
fn extract_dv_rpu_from_block_additions(ba: &[u8]) -> Option<&[u8]> {
    let mut pos = 0;
    while pos < ba.len() {
        let (id, ds, hlen) = try_parse_ebml_header(&ba[pos..])?;
        if ds == EBML_UNKNOWN_SIZE {
            return None;
        }
        let child_end = pos + hlen + ds as usize;
        if child_end > ba.len() {
            return None;
        }
        if id == EBML_BLOCK_MORE
            && let Some(rpu) =
                extract_rpu_from_block_more(&ba[pos + hlen..pos + hlen + ds as usize])
        {
            return Some(rpu);
        }
        pos = child_end;
    }
    None
}

fn extract_rpu_from_block_more(bm: &[u8]) -> Option<&[u8]> {
    let mut pos = 0;
    let mut add_id: Option<u64> = None;
    let mut additional: Option<&[u8]> = None;
    while pos < bm.len() {
        let (id, ds, hlen) = try_parse_ebml_header(&bm[pos..])?;
        if ds == EBML_UNKNOWN_SIZE {
            return None;
        }
        let child_start = pos + hlen;
        let child_end = child_start + ds as usize;
        if child_end > bm.len() {
            return None;
        }
        if id == EBML_BLOCK_ADD_ID {
            // Parse the integer value.
            let val_bytes = &bm[child_start..child_end];
            let mut v = 0u64;
            for &b in val_bytes {
                v = (v << 8) | b as u64;
            }
            add_id = Some(v);
        } else if id == EBML_BLOCK_ADDITIONAL {
            additional = Some(&bm[child_start..child_end]);
        }
        pos = child_end;
    }
    if add_id == Some(DV_BLOCK_ADD_ID) {
        additional
    } else {
        None
    }
}

/// Inject a converted RPU NAL into the Block's frame data.
///
/// Block layout:  track VINT | 2-byte timecode | 1-byte flags | frame data ...
///
/// If lacing flags (bits 5-4 of flags byte) are non-zero, the block is laced
/// and we cannot safely append — return the block unchanged.
pub(crate) fn inject_rpu_into_mkv_block<'a>(block: &'a [u8], rpu: &[u8]) -> Cow<'a, [u8]> {
    if block.is_empty() {
        return Cow::Borrowed(block);
    }

    // Parse track number vint.
    let Some((_, track_vint_len)) = parse_ebml_vint(block) else {
        return Cow::Borrowed(block);
    };
    // Timecode: 2 bytes, Flags: 1 byte.
    let flags_offset = track_vint_len + 2;
    if flags_offset >= block.len() {
        return Cow::Borrowed(block);
    }
    let flags = block[flags_offset];
    // Lacing bits: 5-4.
    let lacing = (flags >> 1) & 0x03;
    if lacing != 0 {
        // Laced block — cannot safely inject RPU.
        return Cow::Borrowed(block);
    }

    let frame_start = flags_offset + 1;
    if frame_start > block.len() {
        return Cow::Borrowed(block);
    }
    let frame = &block[frame_start..];

    // Detect framing: Annex-B vs length-delimited.
    let is_annexb = (frame.len() >= 3 && frame[0] == 0 && frame[1] == 0 && frame[2] == 1)
        || (frame.len() >= 4 && frame[0] == 0 && frame[1] == 0 && frame[2] == 0 && frame[3] == 1);

    let mut out = Vec::with_capacity(block.len() + 4 + rpu.len());
    out.extend_from_slice(&block[..frame_start]);
    out.extend_from_slice(frame);
    if is_annexb {
        out.extend_from_slice(&[0x00, 0x00, 0x00, 0x01]);
        out.extend_from_slice(rpu);
    } else {
        // Length-delimited: 4-byte BE size + payload.
        let len = rpu.len() as u32;
        out.extend_from_slice(&len.to_be_bytes());
        out.extend_from_slice(rpu);
    }
    Cow::Owned(out)
}

// MKV RPU rewriter streaming state machine
enum MkvState {
    /// Accumulating bytes to parse the next EBML element header.
    Header,
    /// Forwarding a non-BlockGroup element's content verbatim.
    Forward { remaining: u64 },
    /// Accumulating a complete BlockGroup payload before processing.
    BlockGroup { buf: Vec<u8>, remaining: u64 },
}

pub(crate) struct MkvRpuRewriter {
    pending: Vec<u8>,
    state: MkvState,
    /// `Some(n)` = bytes remaining inside current (sized) Cluster; `None` = not
    /// tracking (unknown-size cluster or not in cluster).
    cluster_remaining: Option<u64>,
    rpu_mode: u8,
    zero_level5: bool,
}

impl MkvRpuRewriter {
    pub(crate) fn new(rpu_mode: u8, zero_level5: bool) -> Self {
        Self {
            pending: Vec::with_capacity(12),
            state: MkvState::Header,
            cluster_remaining: None,
            rpu_mode,
            zero_level5,
        }
    }

    fn in_cluster(&self) -> bool {
        self.cluster_remaining.is_some()
    }

    pub(crate) fn process(&mut self, input: &[u8]) -> Vec<u8> {
        let mut out = Vec::new();
        let mut pos = 0;

        while pos < input.len() {
            let state = std::mem::replace(&mut self.state, MkvState::Header);
            match state {
                MkvState::Header => {
                    // Accumulate bytes until we can parse a header.
                    let take = (input.len() - pos).min(12usize.saturating_sub(self.pending.len()));
                    self.pending.extend_from_slice(&input[pos..pos + take]);
                    pos += take;

                    // Determine minimum bytes needed: id_width + vint_width.
                    let min_needed = if self.pending.is_empty() {
                        1
                    } else {
                        let iw = ebml_id_width(self.pending[0]);
                        if iw == 0 || self.pending.len() < iw {
                            iw.max(1)
                        } else {
                            let vw = ebml_vint_width(self.pending[iw]);
                            iw + vw
                        }
                    };

                    if self.pending.len() < min_needed {
                        // Need more bytes.
                        self.state = MkvState::Header;
                        break;
                    }

                    match try_parse_ebml_header(&self.pending) {
                        None => {
                            // Cannot parse — emit as-is and break.
                            out.extend_from_slice(&self.pending);
                            self.pending.clear();
                            self.state = MkvState::Header;
                            break;
                        }
                        Some((id, data_size, hlen)) => {
                            let header_bytes = self.pending[..hlen].to_vec();
                            self.pending.drain(..hlen);
                            // Any bytes left in pending beyond hlen should be re-fed.
                            // Carry them back into pos logic by prepending to the stream.
                            // We do this by not advancing pos when pending still has bytes —
                            // but since we drained hlen we need to handle leftover pending.
                            // Actually: after drain, self.pending contains bytes AFTER the header
                            // that we haven't consumed yet. We need to process them.
                            // Simplest: move remaining pending into a temp, clear pending, let
                            // the loop iteration handle them via a re-entrant call.
                            let leftover = std::mem::take(&mut self.pending);

                            if id == EBML_CLUSTER {
                                // Emit cluster header, possibly replacing size vint with UNKNOWN.
                                if data_size != EBML_UNKNOWN_SIZE {
                                    // Replace size with unknown-size (8-byte all-ones vint).
                                    let id_len = ebml_id_width(header_bytes[0]);
                                    out.extend_from_slice(&header_bytes[..id_len]);
                                    // Emit 8-byte unknown-size vint.
                                    out.extend_from_slice(&[
                                        0x01, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
                                    ]);
                                    self.cluster_remaining = Some(data_size);
                                } else {
                                    out.extend_from_slice(&header_bytes);
                                    self.cluster_remaining = None;
                                }
                                self.state = MkvState::Header;
                            } else if id == EBML_BLOCK_GROUP
                                && self.in_cluster()
                                && data_size != EBML_UNKNOWN_SIZE
                            {
                                // Buffer BlockGroup — do NOT emit header yet.
                                // Decrement cluster_remaining by full element size.
                                if let Some(cr) = self.cluster_remaining.as_mut() {
                                    let full_size = hlen as u64 + data_size;
                                    *cr = cr.saturating_sub(full_size);
                                    if *cr == 0 {
                                        self.cluster_remaining = None;
                                    }
                                }
                                if data_size == 0 {
                                    // Empty BlockGroup: re-encode and emit immediately.
                                    write_ebml_element(&mut out, EBML_BLOCK_GROUP, &[]);
                                    self.state = MkvState::Header;
                                } else {
                                    self.state = MkvState::BlockGroup {
                                        buf: Vec::new(),
                                        remaining: data_size,
                                    };
                                }
                            } else {
                                // All other elements: emit header.
                                out.extend_from_slice(&header_bytes);
                                // Decrement cluster_remaining for elements inside a cluster.
                                if self.in_cluster()
                                    && let Some(cr) = self.cluster_remaining.as_mut()
                                {
                                    let full_size = hlen as u64
                                        + if data_size == EBML_UNKNOWN_SIZE {
                                            0
                                        } else {
                                            data_size
                                        };
                                    *cr = cr.saturating_sub(full_size);
                                    if *cr == 0 {
                                        self.cluster_remaining = None;
                                    }
                                }
                                if data_size == 0 || data_size == EBML_UNKNOWN_SIZE {
                                    if data_size == EBML_UNKNOWN_SIZE {
                                        self.state = MkvState::Forward {
                                            remaining: u64::MAX,
                                        };
                                    } else {
                                        self.state = MkvState::Header;
                                    }
                                } else {
                                    self.state = MkvState::Forward {
                                        remaining: data_size,
                                    };
                                }
                            }

                            // Re-process leftover bytes from pending.
                            if !leftover.is_empty() {
                                let extra = self.process(&leftover);
                                out.extend_from_slice(&extra);
                            }
                        }
                    }
                }

                MkvState::Forward { mut remaining } => {
                    let available = (input.len() - pos) as u64;
                    let take = if remaining == u64::MAX {
                        available
                    } else {
                        available.min(remaining)
                    };
                    out.extend_from_slice(&input[pos..pos + take as usize]);
                    pos += take as usize;
                    if remaining != u64::MAX {
                        remaining -= take;
                        self.state = if remaining == 0 {
                            MkvState::Header
                        } else {
                            MkvState::Forward { remaining }
                        };
                    } else {
                        self.state = MkvState::Forward {
                            remaining: u64::MAX,
                        };
                    }
                }

                MkvState::BlockGroup {
                    mut buf,
                    mut remaining,
                } => {
                    let available = (input.len() - pos) as u64;
                    let take = available.min(remaining) as usize;
                    buf.extend_from_slice(&input[pos..pos + take]);
                    pos += take;
                    remaining -= take as u64;

                    if remaining == 0 {
                        let (processed, _rpu_count) =
                            process_block_group_data(&buf, self.rpu_mode, self.zero_level5);
                        write_ebml_element(&mut out, EBML_BLOCK_GROUP, &processed);
                        self.state = MkvState::Header;
                    } else {
                        self.state = MkvState::BlockGroup { buf, remaining };
                    }
                }
            }
        }

        out
    }

    pub(crate) fn flush(self) -> Vec<u8> {
        let mut out = Vec::new();
        // Emit any pending header bytes unchanged.
        if !self.pending.is_empty() {
            out.extend_from_slice(&self.pending);
        }
        match self.state {
            MkvState::Forward { .. } => {}
            MkvState::BlockGroup { buf, .. } => {
                // Incomplete BlockGroup at EOF — emit as-is.
                write_ebml_element(&mut out, EBML_BLOCK_GROUP, &buf);
            }
            MkvState::Header => {}
        }
        out
    }
}

pub(crate) fn stream_rpu_convert_mkv(
    probe: &[u8],
    upstream: &mut reqwest::blocking::Response,
    downstream: &mut std::net::TcpStream,
    rpu_mode: u8,
    zero_level5: bool,
) {
    let mut rewriter = MkvRpuRewriter::new(rpu_mode, zero_level5);
    let init = rewriter.process(probe);
    if !init.is_empty() && downstream.write_all(&init).is_err() {
        return;
    }
    let mut buf = [0u8; 65536];
    loop {
        let n = upstream.read(&mut buf).unwrap_or(0);
        if n == 0 {
            let tail = rewriter.flush();
            let _ = downstream.write_all(&tail);
            break;
        }
        let out = rewriter.process(&buf[..n]);
        if downstream.write_all(&out).is_err() {
            break;
        }
    }
}
