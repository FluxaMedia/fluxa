use crate::annexb::split;

const NAL_PREFIX_SEI: u8 = 39;
const SEI_USER_DATA_T35: u8 = 4;
const HDR10_PLUS_HEADER: [u8; 6] = [0xb5, 0x00, 0x3c, 0x00, 0x01, 0x04];
const TRAILING_SLACK: usize = 4;

fn is_hdr10_plus(nal: &[u8]) -> bool {
    if nal.len() < 4 || (nal[0] >> 1) & 0x3f != NAL_PREFIX_SEI || nal[2] != SEI_USER_DATA_T35 {
        return false;
    }
    let size = nal[3] as usize;
    size >= HDR10_PLUS_HEADER.len()
        && nal.get(4..4 + HDR10_PLUS_HEADER.len()) == Some(&HDR10_PLUS_HEADER[..])
        && nal.len() <= 4 + size + TRAILING_SLACK
}

pub fn strip(data: &[u8]) -> Option<Vec<u8>> {
    let nals = split(data);
    if !nals.iter().any(|&(_, payload, end)| is_hdr10_plus(&data[payload..end])) {
        return None;
    }
    let mut out = Vec::with_capacity(data.len());
    for (code_start, payload, end) in nals {
        if is_hdr10_plus(&data[payload..end]) {
            continue;
        }
        if code_start > 0 && data[code_start - 1] == 0 {
            out.push(0);
        }
        out.extend_from_slice(&data[code_start..end]);
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sei() -> Vec<u8> {
        let mut nal = vec![0x4e, 0x01, SEI_USER_DATA_T35, 10];
        nal.extend_from_slice(&HDR10_PLUS_HEADER);
        nal.extend_from_slice(&[1, 2, 3, 4, 0x80]);
        nal
    }

    fn annexb(nals: &[&[u8]]) -> Vec<u8> {
        nals.iter().flat_map(|nal| [&[0u8, 0, 0, 1][..], nal].concat()).collect()
    }

    #[test]
    fn dynamic_metadata_sei_is_removed_and_slices_kept() {
        let slice: &[u8] = &[0x26, 0x01, 0xaa];
        let input = annexb(&[&sei(), slice]);
        assert_eq!(strip(&input), Some(annexb(&[slice])));
    }

    #[test]
    fn packets_without_it_are_left_alone() {
        let slice: &[u8] = &[0x26, 0x01, 0xaa];
        assert_eq!(strip(&annexb(&[slice])), None);
    }

    #[test]
    fn other_sei_messages_survive() {
        let mastering: &[u8] = &[0x4e, 0x01, 137, 24, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x80];
        assert_eq!(strip(&annexb(&[mastering])), None);
    }
}
