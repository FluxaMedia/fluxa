use dolby_vision::rpu::{ConversionMode, dovi_rpu::DoviRpu};

use crate::capabilities::Capabilities;

const NAL_RPU: u8 = 62;
const NAL_ENHANCEMENT: u8 = 63;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Mode {
    To81,
    Strip,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Plan {
    Pass,
    Rewrite(Mode),
    Unsupported(&'static str),
}

pub fn forced_from_env() -> Option<Mode> {
    match std::env::var("FLUXA_DV_MODE").ok()?.as_str() {
        "strip" => Some(Mode::Strip),
        "convert" => Some(Mode::To81),
        _ => None,
    }
}

pub fn plan(profile: u8, compat: u8, caps: &Capabilities, forced: Option<Mode>) -> Plan {
    let base_is_dolby_only = compat == 0;
    let strip = || {
        if base_is_dolby_only {
            Plan::Unsupported("Dolby Vision profile 5 cannot be shown without a Dolby Vision display")
        } else {
            Plan::Rewrite(Mode::Strip)
        }
    };
    if caps.hdr == Some(false) || forced == Some(Mode::Strip) || caps.dolby_vision != Some(true) {
        return strip();
    }
    if profile == 7 {
        return Plan::Rewrite(Mode::To81);
    }
    Plan::Pass
}

#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Stats {
    pub converted: u32,
    pub dropped: u32,
}

fn start_codes(data: &[u8]) -> Vec<(usize, usize)> {
    let mut found = Vec::new();
    let mut i = 0;
    while i + 3 <= data.len() {
        if data[i] == 0 && data[i + 1] == 0 && data[i + 2] == 1 {
            found.push((i, i + 3));
            i += 3;
        } else {
            i += 1;
        }
    }
    found
}

fn nal_type(nal: &[u8]) -> Option<u8> {
    nal.first().map(|byte| (byte >> 1) & 0x3f)
}

fn convert(nal: &[u8]) -> Result<Vec<u8>, ()> {
    let mut rpu = DoviRpu::parse_unspec62_nalu(nal).map_err(|_| ())?;
    if rpu.dovi_profile != 7 {
        return Ok(nal.to_vec());
    }
    rpu.convert_with_mode(ConversionMode::To81).map_err(|_| ())?;
    rpu.write_hevc_unspec62_nalu().map_err(|_| ())
}

pub fn rewrite(data: &[u8], mode: Mode, stats: &mut Stats) -> Vec<u8> {
    let starts = start_codes(data);
    if starts.is_empty() {
        return data.to_vec();
    }
    let mut out = Vec::with_capacity(data.len());
    for (index, &(code_start, payload)) in starts.iter().enumerate() {
        let end = starts.get(index + 1).map_or(data.len(), |next| {
            let mut end = next.0;
            while end > payload && data[end - 1] == 0 {
                end -= 1;
            }
            end
        });
        let nal = &data[payload..end];
        let prefix = &data[code_start..payload];
        match nal_type(nal) {
            Some(NAL_ENHANCEMENT) => {}
            Some(NAL_RPU) if mode == Mode::Strip => stats.dropped += 1,
            Some(NAL_RPU) => match convert(nal) {
                Ok(rewritten) => {
                    if rewritten != nal {
                        stats.converted += 1;
                    }
                    out.extend_from_slice(&[0, 0, 0, 1]);
                    out.extend_from_slice(&rewritten);
                }
                Err(()) => stats.dropped += 1,
            },
            _ => {
                if code_start > 0 && data[code_start - 1] == 0 {
                    out.push(0);
                }
                out.extend_from_slice(prefix);
                out.extend_from_slice(nal);
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use dolby_vision::rpu::generate::GenerateConfig;

    use super::*;

    fn p81_rpu() -> Vec<u8> {
        let config = GenerateConfig { length: 1, ..Default::default() };
        DoviRpu::profile81_config(&config).unwrap().write_hevc_unspec62_nalu().unwrap()
    }

    fn annexb(nals: &[&[u8]]) -> Vec<u8> {
        let mut out = Vec::new();
        for nal in nals {
            out.extend_from_slice(&[0, 0, 0, 1]);
            out.extend_from_slice(nal);
        }
        out
    }

    #[test]
    fn enhancement_layer_nals_are_dropped_and_others_kept() {
        let slice = [0x26, 0x01, 0xaa, 0xbb];
        let el = [0x7e, 0x01, 0x11, 0x22];
        let input = annexb(&[&slice, &el]);
        let mut stats = Stats::default();
        assert_eq!(rewrite(&input, Mode::To81, &mut stats), annexb(&[&slice]));
    }

    #[test]
    fn profile_81_rpu_passes_through_unchanged() {
        let rpu = p81_rpu();
        assert_eq!(nal_type(&rpu), Some(NAL_RPU));
        let input = annexb(&[&[0x26, 0x01, 0xaa], &rpu]);
        let mut stats = Stats::default();
        assert_eq!(rewrite(&input, Mode::To81, &mut stats), input);
        assert_eq!(stats, Stats::default());
    }

    #[test]
    fn unparsable_rpu_is_dropped_and_counted() {
        let input = annexb(&[&[0x26, 0x01, 0xaa], &[0x7c, 0x01, 0x01, 0x02, 0x03]]);
        let mut stats = Stats::default();
        assert_eq!(rewrite(&input, Mode::To81, &mut stats), annexb(&[&[0x26, 0x01, 0xaa]]));
        assert_eq!(stats.dropped, 1);
    }

    #[test]
    fn three_byte_start_codes_survive() {
        let input = [0, 0, 1, 0x26, 0x01, 0xaa, 0, 0, 1, 0x02, 0x01, 0xbb];
        let mut stats = Stats::default();
        assert_eq!(rewrite(&input, Mode::To81, &mut stats), input);
    }

    #[test]
    fn strip_removes_rpu_and_enhancement_layer() {
        let slice: &[u8] = &[0x26, 0x01, 0xaa];
        let rpu = p81_rpu();
        let input = annexb(&[slice, &rpu, &[0x7e, 0x01, 0x05]]);
        let mut stats = Stats::default();
        assert_eq!(rewrite(&input, Mode::Strip, &mut stats), annexb(&[slice]));
    }

    #[test]
    fn dolby_tv_keeps_profile_8_and_converts_7() {
        let caps = Capabilities { hdr: Some(true), dolby_vision: Some(true) };
        assert_eq!(plan(8, 1, &caps, None), Plan::Pass);
        assert_eq!(plan(7, 6, &caps, None), Plan::Rewrite(Mode::To81));
    }

    #[test]
    fn tv_without_dolby_strips_to_the_base_layer() {
        let caps = Capabilities { hdr: Some(true), dolby_vision: Some(false) };
        assert_eq!(plan(8, 1, &caps, None), Plan::Rewrite(Mode::Strip));
        assert_eq!(plan(7, 6, &caps, None), Plan::Rewrite(Mode::Strip));
    }

    #[test]
    fn profile_5_has_no_usable_base_layer() {
        let caps = Capabilities { hdr: Some(true), dolby_vision: Some(false) };
        assert!(matches!(plan(5, 0, &caps, None), Plan::Unsupported(_)));
    }

    #[test]
    fn unknown_dolby_support_is_treated_as_unsupported() {
        assert_eq!(plan(8, 1, &Capabilities::default(), None), Plan::Rewrite(Mode::Strip));
        assert!(matches!(plan(5, 0, &Capabilities::default(), None), Plan::Unsupported(_)));
    }
}
