const START: [u8; 4] = [0, 0, 0, 1];

pub struct Converter {
    length_size: usize,
    parameter_sets: Vec<u8>,
}

fn push_nal(out: &mut Vec<u8>, nal: &[u8]) {
    out.extend_from_slice(&START);
    out.extend_from_slice(nal);
}

fn be16(data: &[u8], at: usize) -> Option<usize> {
    Some(u16::from_be_bytes([*data.get(at)?, *data.get(at + 1)?]) as usize)
}

impl Converter {
    pub fn avcc(extradata: &[u8]) -> Option<Converter> {
        if extradata.len() < 7 || extradata[0] != 1 {
            return None;
        }
        let mut sets = Vec::new();
        let mut at = 5;
        let sps_count = (extradata[at] & 0x1f) as usize;
        at += 1;
        for _ in 0..sps_count {
            let len = be16(extradata, at)?;
            push_nal(&mut sets, extradata.get(at + 2..at + 2 + len)?);
            at += 2 + len;
        }
        let pps_count = *extradata.get(at)? as usize;
        at += 1;
        for _ in 0..pps_count {
            let len = be16(extradata, at)?;
            push_nal(&mut sets, extradata.get(at + 2..at + 2 + len)?);
            at += 2 + len;
        }
        Some(Converter { length_size: (extradata[4] & 3) as usize + 1, parameter_sets: sets })
    }

    pub fn hvcc(extradata: &[u8]) -> Option<Converter> {
        if extradata.len() < 23 || extradata[0] != 1 {
            return None;
        }
        let mut sets = Vec::new();
        let mut at = 23;
        for _ in 0..extradata[22] {
            let count = be16(extradata, at + 1)?;
            at += 3;
            for _ in 0..count {
                let len = be16(extradata, at)?;
                push_nal(&mut sets, extradata.get(at + 2..at + 2 + len)?);
                at += 2 + len;
            }
        }
        Some(Converter { length_size: (extradata[21] & 3) as usize + 1, parameter_sets: sets })
    }

    pub fn convert(&self, packet: &[u8], keyframe: bool) -> Vec<u8> {
        let mut out = Vec::with_capacity(packet.len() + self.parameter_sets.len() + 16);
        if keyframe {
            out.extend_from_slice(&self.parameter_sets);
        }
        let mut at = 0;
        while at + self.length_size <= packet.len() {
            let len = packet[at..at + self.length_size].iter().fold(0usize, |acc, byte| acc << 8 | *byte as usize);
            at += self.length_size;
            let Some(nal) = packet.get(at..at + len) else {
                break;
            };
            push_nal(&mut out, nal);
            at += len;
        }
        out
    }
}

pub fn split(data: &[u8]) -> Vec<(usize, usize, usize)> {
    let mut starts = Vec::new();
    let mut i = 0;
    while i + 3 <= data.len() {
        if data[i] == 0 && data[i + 1] == 0 && data[i + 2] == 1 {
            starts.push((i, i + 3));
            i += 3;
        } else {
            i += 1;
        }
    }
    starts
        .iter()
        .enumerate()
        .map(|(index, &(code_start, payload))| {
            let end = starts.get(index + 1).map_or(data.len(), |next| {
                let mut end = next.0;
                while end > payload && data[end - 1] == 0 {
                    end -= 1;
                }
                end
            });
            (code_start, payload, end)
        })
        .collect()
}

pub fn is_annexb(packet: &[u8]) -> bool {
    packet.starts_with(&[0, 0, 1]) || packet.starts_with(&START)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn avcc() -> Vec<u8> {
        vec![1, 100, 0, 40, 0xff, 0xe1, 0, 2, 0x67, 0xaa, 1, 0, 2, 0x68, 0xbb]
    }

    #[test]
    fn avcc_parameter_sets_precede_keyframes_only() {
        let converter = Converter::avcc(&avcc()).unwrap();
        let packet = [0, 0, 0, 2, 0x65, 0x11];
        let key = converter.convert(&packet, true);
        assert_eq!(key, [0, 0, 0, 1, 0x67, 0xaa, 0, 0, 0, 1, 0x68, 0xbb, 0, 0, 0, 1, 0x65, 0x11]);
        assert_eq!(converter.convert(&packet, false), [0, 0, 0, 1, 0x65, 0x11]);
    }

    #[test]
    fn two_byte_length_prefix_is_honoured() {
        let mut extra = avcc();
        extra[4] = 0xfd;
        let converter = Converter::avcc(&extra).unwrap();
        assert_eq!(converter.convert(&[0, 1, 0x41, 0, 2, 0x01, 0x02], false), [0, 0, 0, 1, 0x41, 0, 0, 0, 1, 1, 2]);
    }

    #[test]
    fn truncated_nal_is_dropped_not_panicked() {
        let converter = Converter::avcc(&avcc()).unwrap();
        assert_eq!(converter.convert(&[0, 0, 0, 9, 1, 2], false), Vec::<u8>::new());
    }

    #[test]
    fn hvcc_arrays_become_start_code_prefixed_nals() {
        let mut extra = vec![0u8; 23];
        extra[0] = 1;
        extra[21] = 0x03;
        extra[22] = 1;
        extra.extend_from_slice(&[32, 0, 1, 0, 2, 0x40, 0x01]);
        let converter = Converter::hvcc(&extra).unwrap();
        assert_eq!(converter.convert(&[0, 0, 0, 1, 0x26], true), [0, 0, 0, 1, 0x40, 0x01, 0, 0, 0, 1, 0x26]);
    }

    #[test]
    fn annexb_streams_are_detected() {
        assert!(is_annexb(&[0, 0, 0, 1, 0x65]));
        assert!(!is_annexb(&[0, 0, 0, 2, 0x65, 1]));
    }

    #[test]
    fn bad_extradata_is_rejected() {
        assert!(Converter::avcc(&[2, 0, 0]).is_none());
        assert!(Converter::hvcc(&[1; 10]).is_none());
    }
}
