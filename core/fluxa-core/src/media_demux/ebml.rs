//! Low-level EBML primitives shared by every Matroska/WebM reader and writer
//! in this workspace (the MKV demuxer here, and the Dolby Vision RPU rewriter
//! and chapter extractor in `fluxa-streaming-engine`). Kept dependency-free so
//! it compiles for `wasm` as well as native targets.

pub const EBML_UNKNOWN_SIZE: u64 = u64::MAX;

pub fn ebml_id_width(first_byte: u8) -> usize {
    match first_byte {
        0x80..=0xFF => 1,
        0x40..=0x7F => 2,
        0x20..=0x3F => 3,
        0x10..=0x1F => 4,
        _ => 0,
    }
}

/// Byte-width of an EBML variable-length integer (vint) whose first byte is
/// `first_byte`.
pub fn ebml_vint_width(first_byte: u8) -> usize {
    if first_byte & 0x80 != 0 {
        return 1;
    }
    if first_byte & 0x40 != 0 {
        return 2;
    }
    if first_byte & 0x20 != 0 {
        return 3;
    }
    if first_byte & 0x10 != 0 {
        return 4;
    }
    if first_byte & 0x08 != 0 {
        return 5;
    }
    if first_byte & 0x04 != 0 {
        return 6;
    }
    if first_byte & 0x02 != 0 {
        return 7;
    }
    if first_byte & 0x01 != 0 {
        return 8;
    }
    0
}

/// Parse an EBML element ID from `buf`. Returns `Some((id, bytes_consumed))`.
/// EBML IDs are stored as raw big-endian bytes (marker bits are part of the ID).
pub fn parse_ebml_id(buf: &[u8]) -> Option<(u64, usize)> {
    if buf.is_empty() {
        return None;
    }
    let width = ebml_id_width(buf[0]);
    if width == 0 || buf.len() < width {
        return None;
    }
    let mut id = 0u64;
    for &b in &buf[..width] {
        id = (id << 8) | b as u64;
    }
    Some((id, width))
}

/// Parse an EBML variable-length integer from `buf`.
/// Returns `Some((value, bytes_consumed))`.
/// Returns `EBML_UNKNOWN_SIZE` for the all-ones vint (unknown-size marker).
pub fn parse_ebml_vint(buf: &[u8]) -> Option<(u64, usize)> {
    if buf.is_empty() {
        return None;
    }
    let width = ebml_vint_width(buf[0]);
    if width == 0 || buf.len() < width {
        return None;
    }

    let unknown_size = match width {
        1 => buf[0] == 0xFF,
        2 => buf[0] == 0x7F && buf[1] == 0xFF,
        3 => buf[0] == 0x3F && buf[1] == 0xFF && buf[2] == 0xFF,
        4 => buf[0] == 0x1F && buf[1] == 0xFF && buf[2] == 0xFF && buf[3] == 0xFF,
        5 => buf[0] == 0x0F && buf[1..5].iter().all(|&b| b == 0xFF),
        6 => buf[0] == 0x07 && buf[1..6].iter().all(|&b| b == 0xFF),
        7 => buf[0] == 0x03 && buf[1..7].iter().all(|&b| b == 0xFF),
        8 => buf[0] == 0x01 && buf[1..8].iter().all(|&b| b == 0xFF),
        _ => false,
    };
    if unknown_size {
        return Some((EBML_UNKNOWN_SIZE, width));
    }

    let marker_mask = 0x80u8 >> (width - 1);
    let mut value = (buf[0] & !marker_mask) as u64;
    for &b in &buf[1..width] {
        value = (value << 8) | b as u64;
    }
    Some((value, width))
}

/// Parse a complete EBML element header (ID + data-size vint) from `buf`.
/// Returns `Some((id, data_size, header_len))` where `header_len` = id bytes +
/// vint bytes. `data_size` may be `EBML_UNKNOWN_SIZE`.
pub fn try_parse_ebml_header(buf: &[u8]) -> Option<(u64, u64, usize)> {
    let (id, id_len) = parse_ebml_id(buf)?;
    let (data_size, vint_len) = parse_ebml_vint(buf.get(id_len..)?)?;
    Some((id, data_size, id_len + vint_len))
}

/// Encode a value as a minimum-width EBML variable-length integer.
pub fn write_ebml_vint(out: &mut Vec<u8>, value: u64) {
    if value < 0x7F {
        out.push(0x80 | value as u8)
    } else if value < 0x3FFF {
        out.extend_from_slice(&[0x40 | (value >> 8) as u8, (value & 0xFF) as u8])
    } else if value < 0x1F_FFFF {
        out.extend_from_slice(&[
            0x20 | (value >> 16) as u8,
            (value >> 8) as u8,
            (value & 0xFF) as u8,
        ])
    } else if value < 0x0FFF_FFFF {
        out.extend_from_slice(&[
            0x10 | (value >> 24) as u8,
            (value >> 16) as u8,
            (value >> 8) as u8,
            (value & 0xFF) as u8,
        ])
    } else if value < 0x07_FFFF_FFFF {
        out.extend_from_slice(&[
            0x08 | (value >> 32) as u8,
            (value >> 24) as u8,
            (value >> 16) as u8,
            (value >> 8) as u8,
            (value & 0xFF) as u8,
        ])
    } else if value < 0x03FF_FFFF_FFFF {
        out.extend_from_slice(&[
            0x04 | (value >> 40) as u8,
            (value >> 32) as u8,
            (value >> 24) as u8,
            (value >> 16) as u8,
            (value >> 8) as u8,
            (value & 0xFF) as u8,
        ])
    } else if value < 0x01_FFFF_FFFF_FFFF {
        out.extend_from_slice(&[
            0x02 | (value >> 48) as u8,
            (value >> 40) as u8,
            (value >> 32) as u8,
            (value >> 24) as u8,
            (value >> 16) as u8,
            (value >> 8) as u8,
            (value & 0xFF) as u8,
        ])
    } else {
        out.extend_from_slice(&[
            0x01,
            (value >> 48) as u8,
            (value >> 40) as u8,
            (value >> 32) as u8,
            (value >> 24) as u8,
            (value >> 16) as u8,
            (value >> 8) as u8,
            (value & 0xFF) as u8,
        ])
    }
}

/// Encode an EBML element ID as minimum big-endian bytes.
pub fn write_ebml_id(out: &mut Vec<u8>, id: u64) {
    if id <= 0xFF {
        out.push(id as u8)
    } else if id <= 0xFFFF {
        out.extend_from_slice(&[(id >> 8) as u8, (id & 0xFF) as u8])
    } else if id <= 0xFF_FFFF {
        out.extend_from_slice(&[(id >> 16) as u8, (id >> 8) as u8, (id & 0xFF) as u8])
    } else {
        out.extend_from_slice(&[
            (id >> 24) as u8,
            (id >> 16) as u8,
            (id >> 8) as u8,
            (id & 0xFF) as u8,
        ])
    }
}

pub fn write_ebml_element(out: &mut Vec<u8>, id: u64, data: &[u8]) {
    write_ebml_id(out, id);
    write_ebml_vint(out, data.len() as u64);
    out.extend_from_slice(data);
}

// Not #[cfg(test)]: fluxa-streaming-engine's own tests (a downstream crate)
// need these too, and cfg(test) items aren't visible across a crate boundary.
pub fn encode_ebml_vint(value: u64) -> Vec<u8> {
    let mut out = Vec::with_capacity(8);
    write_ebml_vint(&mut out, value);
    out
}

pub fn encode_ebml_element(id: u64, data: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(12 + data.len());
    write_ebml_element(&mut out, id, data);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vint_roundtrip() {
        for value in [0u64, 1, 127, 128, 16383, 16384, 5_000_000] {
            let encoded = encode_ebml_vint(value);
            let (decoded, len) = parse_ebml_vint(&encoded).unwrap();
            assert_eq!(decoded, value);
            assert_eq!(len, encoded.len());
        }
    }

    #[test]
    fn header_roundtrip() {
        let elem = encode_ebml_element(0x1654_69A5, &[1, 2, 3, 4]);
        let (id, size, hlen) = try_parse_ebml_header(&elem).unwrap();
        assert_eq!(id, 0x1654_69A5);
        assert_eq!(size, 4);
        assert_eq!(&elem[hlen..], &[1, 2, 3, 4]);
    }
}
