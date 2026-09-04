use super::*;

pub(crate) fn stream_hdr10plus_strip(
    upstream: &mut reqwest::blocking::Response,
    downstream: &mut TcpStream,
) {
    run_nal_stream(upstream, downstream, NalRewriteState::new_hdr10plus_strip());
}

fn run_nal_stream(
    upstream: &mut reqwest::blocking::Response,
    downstream: &mut TcpStream,
    mut state: NalRewriteState,
) {
    let mut buf = [0u8; 65536];
    let mut out = Vec::with_capacity(65536);
    loop {
        let n = upstream.read(&mut buf).unwrap_or(0);
        if n == 0 {
            state.flush_into(&mut out);
            let _ = downstream.write_all(&out);
            break;
        }
        state.process_into(&buf[..n], &mut out);
        if downstream.write_all(&out).is_err() {
            break;
        }
    }
}

// NAL rewrite state machine
enum NalProcessMode {
    RpuConvert {
        rpu_mode: u8,
        zero_level5: bool,
        remove_hdr10plus: bool,
    },
    Hdr10PlusStrip,
}

pub(crate) struct NalRewriteState {
    pending: Vec<u8>,
    mode: NalProcessMode,
    rpu_converted: u32,
    rpu_failed: u32,
    el_dropped: u32,
}

impl NalRewriteState {
    /// rpu_convert mode — kept for tests.
    #[cfg(test)]
    pub(crate) fn new(rpu_mode: u8) -> Self {
        Self::new_rpu_convert(rpu_mode, false, false)
    }

    pub(crate) fn new_rpu_convert(rpu_mode: u8, zero_level5: bool, remove_hdr10plus: bool) -> Self {
        Self {
            pending: Vec::with_capacity(65536),
            mode: NalProcessMode::RpuConvert {
                rpu_mode,
                zero_level5,
                remove_hdr10plus,
            },
            rpu_converted: 0,
            rpu_failed: 0,
            el_dropped: 0,
        }
    }

    pub(crate) fn new_hdr10plus_strip() -> Self {
        Self {
            pending: Vec::with_capacity(65536),
            mode: NalProcessMode::Hdr10PlusStrip,
            rpu_converted: 0,
            rpu_failed: 0,
            el_dropped: 0,
        }
    }

    pub(crate) fn process(&mut self, input: &[u8]) -> Vec<u8> {
        let mut output = Vec::with_capacity(input.len());
        self.process_into(input, &mut output);
        output
    }

    pub(crate) fn process_into(&mut self, input: &[u8], output: &mut Vec<u8>) {
        self.pending.extend_from_slice(input);
        let Some((mut start, start_len)) = find_start_code(&self.pending, 0) else {
            output.clear();
            return;
        };
        let Some((mut next, _)) = find_start_code(&self.pending, start + start_len) else {
            output.clear();
            return;
        };
        output.clear();
        loop {
            let (conv, fail, dropped) = emit_nal(&self.pending[start..next], &self.mode, output);
            self.rpu_converted += conv;
            self.rpu_failed += fail;
            self.el_dropped += dropped;
            start = next;
            let Some((candidate, _)) = find_start_code(&self.pending, start + 3) else {
                break;
            };
            next = candidate;
        }
        self.pending.copy_within(start.., 0);
        self.pending.truncate(self.pending.len() - start);
    }

    pub(crate) fn rpu_stats(&self) -> (u32, u32, u32) {
        (self.rpu_converted, self.rpu_failed, self.el_dropped)
    }

    pub(crate) fn flush(self) -> Vec<u8> {
        let mut output = Vec::new();
        let mut state = self;
        state.flush_into(&mut output);
        output
    }

    pub(crate) fn flush_into(&mut self, output: &mut Vec<u8>) {
        if self.pending.is_empty() {
            output.clear();
            return;
        }
        output.clear();
        let (conv, fail, dropped) = emit_nal(&self.pending, &self.mode, output);
        self.rpu_converted += conv;
        self.rpu_failed += fail;
        self.el_dropped += dropped;
    }
}

/// Emit one Annex-B NAL unit to `out` and return `(rpu_converted, rpu_failed, el_dropped)`.
fn emit_nal(nal_with_sc: &[u8], mode: &NalProcessMode, out: &mut Vec<u8>) -> (u32, u32, u32) {
    let sc = start_code_len(nal_with_sc);
    let nal = &nal_with_sc[sc..];
    if nal.len() < 2 {
        out.extend_from_slice(nal_with_sc);
        return (0, 0, 0);
    }
    let nal_type = (nal[0] >> 1) & 0x3F;
    // HEVC NAL header: nuh_layer_id lives in bits [8:3] across both header bytes.
    let layer_id = ((nal[0] & 0x01) << 5) | (nal[1] >> 3);

    match mode {
        NalProcessMode::RpuConvert {
            rpu_mode,
            zero_level5,
            remove_hdr10plus,
        } => {
            // Single-pass: strip HDR10+ SEIs and convert RPU NALs together.
            if *remove_hdr10plus && nal_is_hdr10plus_sei(nal) {
                return (0, 0, 0);
            }
            if nal_type == 62 {
                if let Some(converted) = convert_rpu_nal(nal, *rpu_mode, *zero_level5) {
                    out.extend_from_slice(&nal_with_sc[..sc]);
                    out.extend_from_slice(&converted);
                    return (1, 0, 0);
                }
                // Conversion failed: keep original
                out.extend_from_slice(nal_with_sc);
                return (0, 1, 0);
            }
            if layer_id > 0 {
                // Enhancement layer NAL — not needed for single-layer DV8.1.
                return (0, 0, 1);
            }
            out.extend_from_slice(nal_with_sc);
            (0, 0, 0)
        }
        NalProcessMode::Hdr10PlusStrip => {
            if nal_is_hdr10plus_sei(nal) {
            } else {
                out.extend_from_slice(nal_with_sc);
            }
            (0, 0, 0)
        }
    }
}

pub(crate) fn convert_rpu_nal(nal: &[u8], mode: u8, zero_level5: bool) -> Option<Vec<u8>> {
    let mut rpu = DoviRpu::parse_unspec62_nalu(nal).ok()?;
    store_l1_from_rpu(&rpu);
    rpu.convert_with_mode(mode).ok()?;
    if zero_level5 {
        let _ = rpu.crop();
    }
    rpu.write_hevc_unspec62_nalu().ok()
}

// HDR10+ SEI detector
/// Returns true if `nal` (starting with the 2-byte HEVC NAL header) is a
/// PREFIX_SEI (type 39) or SUFFIX_SEI (type 40) whose first SEI message is
/// an ITU-T T35 user_data_registered payload (type 4) carrying the HDR10+
/// signature: country_code=0xB5, terminal_provider_code=0x003C,
/// terminal_provider_oriented_code=0x0001.
pub(crate) fn nal_is_hdr10plus_sei(nal: &[u8]) -> bool {
    if nal.len() < 9 {
        return false;
    }
    // PREFIX_SEI = 39, SUFFIX_SEI = 40
    let nal_type = (nal[0] >> 1) & 0x3F;
    if nal_type != 39 && nal_type != 40 {
        return false;
    }
    // After the 2-byte HEVC NAL header, parse the variable-length SEI payload type.
    let mut i = 2;
    let mut payload_type: u32 = 0;
    while i < nal.len() && nal[i] == 0xFF {
        payload_type += 255;
        i += 1;
    }
    if i >= nal.len() {
        return false;
    }
    payload_type += nal[i] as u32;
    i += 1;
    if payload_type != 4 {
        // 4 = user_data_registered_itu_t_t35
        return false;
    }
    // Skip the variable-length payload size field.
    while i < nal.len() && nal[i] == 0xFF {
        i += 1;
    }
    i += 1; // skip final size byte
    // Check ITU-T T35 header: country=0xB5, provider=0x003C, oriented=0x0001
    i + 5 <= nal.len()
        && nal[i] == 0xB5
        && nal[i + 1] == 0x00
        && nal[i + 2] == 0x3C
        && nal[i + 3] == 0x00
        && nal[i + 4] == 0x01
}

// Annex-B utilities
#[cfg(test)]
pub(crate) fn find_start_code_positions(data: &[u8]) -> Vec<usize> {
    let mut positions = Vec::new();
    let mut i = 0;
    while let Some((position, length)) = find_start_code(data, i) {
        positions.push(position);
        i = position + length;
    }
    positions
}

fn find_start_code(data: &[u8], from: usize) -> Option<(usize, usize)> {
    let mut i = from;
    while let Some(offset) = memchr::memchr(0, &data[i..]) {
        i += offset;
        if i + 2 >= data.len() {
            break;
        }
        if data[i + 1] == 0 {
            if i + 3 < data.len() && data[i + 2] == 0 && data[i + 3] == 1 {
                return Some((i, 4));
            }
            if data[i + 2] == 1 {
                return Some((i, 3));
            }
        }
        i += 1;
    }
    None
}

pub(crate) fn start_code_len(data: &[u8]) -> usize {
    if data.len() >= 4 && data[0] == 0 && data[1] == 0 && data[2] == 0 && data[3] == 1 {
        4
    } else if data.len() >= 3 && data[0] == 0 && data[1] == 0 && data[2] == 1 {
        3
    } else {
        0
    }
}
