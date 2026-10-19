//! NAL unit stream rewriting for MP4 samples (length-prefixed to Annex B).

use crate::decode::AnnexBFormat;

fn nal_length(data: &[u8], pos: usize, length_size: usize) -> Option<(usize, usize)> {
    if pos + length_size > data.len() {
        return None;
    }
    let mut len = 0usize;
    for byte in &data[pos..pos + length_size] {
        len = (len << 8) | *byte as usize;
    }
    Some((len, pos + length_size))
}

/// Rewrites length-prefixed NAL units to Annex B start codes and prepends
/// the codec description NALs (SPS/PPS or VPS/SPS/PPS).
pub fn to_annex_b(sample: &[u8], extradata: &[u8], format: AnnexBFormat) -> Vec<u8> {
    let length_size = match extradata.first() {
        Some(b) if format == AnnexBFormat::H264 => ((*b & 0x03) + 1) as usize,
        Some(_) if format == AnnexBFormat::Hevc => (((extradata[0] >> 5) & 0x03) + 1) as usize,
        _ => 4,
    };
    let mut out = Vec::with_capacity(sample.len() + extradata.len() + 16);
    for unit in extradata_units(extradata, format) {
        out.extend_from_slice(&[0, 0, 0, 1]);
        out.extend_from_slice(unit);
    }
    let mut pos = 0usize;
    while let Some((len, next)) = nal_length(sample, pos, length_size) {
        let end = next + len;
        if end > sample.len() {
            break;
        }
        out.extend_from_slice(&[0, 0, 0, 1]);
        out.extend_from_slice(&sample[next..end]);
        pos = end;
    }
    out
}

fn extradata_units<'a>(extradata: &'a [u8], format: AnnexBFormat) -> Vec<&'a [u8]> {
    let mut units = Vec::new();
    match format {
        AnnexBFormat::H264 => {
            if extradata.len() < 7 {
                return units;
            }
            let mut pos = 5usize;
            let sps_count = (extradata[5] & 0x1f) as usize;
            pos += 1;
            for _ in 0..sps_count {
                if pos + 2 > extradata.len() {
                    break;
                }
                let len = u16::from_be_bytes([extradata[pos], extradata[pos + 1]]) as usize;
                pos += 2;
                if len == 0 || pos + len > extradata.len() {
                    break;
                }
                units.push(&extradata[pos..pos + len]);
                pos += len;
            }
            if pos < extradata.len() {
                let pps_count = extradata[pos] as usize;
                pos += 1;
                for _ in 0..pps_count {
                    if pos + 2 > extradata.len() {
                        break;
                    }
                    let len = u16::from_be_bytes([extradata[pos], extradata[pos + 1]]) as usize;
                    pos += 2;
                    if len == 0 || pos + len > extradata.len() {
                        break;
                    }
                    units.push(&extradata[pos..pos + len]);
                    pos += len;
                }
            }
        }
        AnnexBFormat::Hevc => {
            if extradata.len() < 23 {
                return units;
            }
            let num_arrays = extradata[22] as usize;
            let mut pos = 23usize;
            for _ in 0..num_arrays {
                if pos + 3 > extradata.len() {
                    break;
                }
                let count = extradata[pos + 2] as usize;
                pos += 3;
                for _ in 0..count {
                    if pos + 2 > extradata.len() {
                        break;
                    }
                    let len = u16::from_be_bytes([extradata[pos], extradata[pos + 1]]) as usize;
                    pos += 2;
                    if len == 0 || pos + len > extradata.len() {
                        break;
                    }
                    units.push(&extradata[pos..pos + len]);
                    pos += len;
                }
            }
        }
    }
    units
}
