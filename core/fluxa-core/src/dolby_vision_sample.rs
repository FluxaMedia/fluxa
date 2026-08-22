use crate::dolby_vision_plan::{DvPlaybackPlan, SampleExecutionPlan};
use ::dolby_vision::rpu::dovi_rpu::DoviRpu;
use ::dolby_vision::rpu::rpu_data_nlq::DoviELType;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use serde::{Deserialize, Serialize};

const RPU_NAL_TYPE: u8 = 62;
const EL_NAL_TYPE_63: u8 = 63;
const SEI_PREFIX_NAL_TYPE: u8 = 39;
const SEI_SUFFIX_NAL_TYPE: u8 = 40;
const HDR10PLUS_PAYLOAD_TYPE: u32 = 4;
const ITU_T_T35_COUNTRY_CODE_USA: u8 = 0xB5;
const HDR10PLUS_PROVIDER_CODE: [u8; 2] = [0x00, 0x3C];
const HDR10PLUS_PROVIDER_ORIENTED_CODE: [u8; 2] = [0x00, 0x01];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Framing {
    AnnexB,
    LengthDelimited,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EnhancementLayerKind {
    None,
    Mel,
    Fel,
    #[default]
    Unknown,
}

impl From<Option<&DoviELType>> for EnhancementLayerKind {
    fn from(value: Option<&DoviELType>) -> Self {
        match value {
            Some(DoviELType::MEL) => EnhancementLayerKind::Mel,
            Some(DoviELType::FEL) => EnhancementLayerKind::Fel,
            None => EnhancementLayerKind::None,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SampleRequest {
    #[serde(default)]
    sample_base64: String,
    framing: Framing,
    #[serde(default = "default_nal_length_size")]
    nal_length_size: u8,
    #[serde(default = "default_mode")]
    mode: u8,
    #[serde(default = "default_drop_el")]
    drop_el: bool,
    #[serde(default)]
    strip_dv_rpu: bool,
    #[serde(default)]
    strip_hdr10_plus: bool,
    #[serde(default)]
    encrypted: bool,
}

fn default_nal_length_size() -> u8 {
    4
}

fn default_mode() -> u8 {
    2
}

fn default_drop_el() -> bool {
    true
}

#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SampleTransformResult {
    ok: bool,
    changed: bool,
    output_base64: Option<String>,
    output_size: usize,
    rpu_found: u32,
    rpu_converted: u32,
    rpu_failed: u32,
    el_nals_dropped: u32,
    enhancement_layer: EnhancementLayerKind,
    hdr10_plus_messages_removed: u32,
    conversion_possible: bool,
    error: Option<String>,
}

fn strip_hdr10plus_sei(nal: &[u8]) -> (Vec<u8>, u32) {
    let mut out = Vec::with_capacity(nal.len());
    let Some(header) = nal.get(..2) else {
        return (nal.to_vec(), 0);
    };
    out.extend_from_slice(header);

    let mut i = 2;
    let mut removed = 0u32;
    let mut kept_messages = 0u32;
    while i < nal.len() {
        if nal.get(i) == Some(&0x80) && i == nal.len() - 1 {
            break;
        }

        let message_start = i;
        let mut payload_type = 0u32;
        while nal.get(i) == Some(&0xFF) {
            payload_type += 255;
            i += 1;
        }
        let Some(&last_type_byte) = nal.get(i) else {
            break;
        };
        payload_type += last_type_byte as u32;
        i += 1;

        let mut payload_size = 0usize;
        while nal.get(i) == Some(&0xFF) {
            payload_size += 255;
            i += 1;
        }
        let Some(&last_size_byte) = nal.get(i) else {
            break;
        };
        payload_size += last_size_byte as usize;
        i += 1;

        let data_start = i;
        let data_end = (data_start + payload_size).min(nal.len());
        let Some(data) = nal.get(data_start..data_end) else {
            break;
        };

        let is_hdr10plus = payload_type == HDR10PLUS_PAYLOAD_TYPE
            && data.first() == Some(&ITU_T_T35_COUNTRY_CODE_USA)
            && data.get(1..3) == Some(&HDR10PLUS_PROVIDER_CODE)
            && data.get(3..5) == Some(&HDR10PLUS_PROVIDER_ORIENTED_CODE);

        if is_hdr10plus {
            removed += 1;
        } else if let Some(message) = nal.get(message_start..data_end) {
            kept_messages += 1;
            out.extend_from_slice(message);
        }
        i = data_end;
    }

    if kept_messages == 0 {
        return (header.to_vec(), removed);
    }
    if let Some(trailing) = nal.get(i..) {
        out.extend_from_slice(trailing);
    }
    (out, removed)
}

struct Nal<'a> {
    start_code_len: usize,
    bytes: &'a [u8],
    nal_type: u8,
    layer_id: u8,
}

fn nal_header(bytes: &[u8]) -> Option<(u8, u8)> {
    let &[b0, b1, ..] = bytes else { return None };
    let nal_type = (b0 >> 1) & 0x3F;
    let layer_id = ((b0 & 0x1) << 5) | (b1 >> 3);
    Some((nal_type, layer_id))
}

fn split_annex_b(data: &[u8]) -> Option<Vec<Nal<'_>>> {
    let mut starts = Vec::new();
    let mut i = 0;
    while i + 2 < data.len() {
        if data.get(i..i + 3) == Some(&[0, 0, 1]) {
            if i > 0 && data.get(i - 1) == Some(&0) {
                starts.push((i - 1, 4usize));
            } else {
                starts.push((i, 3usize));
            }
            i += 3;
        } else {
            i += 1;
        }
    }
    if starts.is_empty() {
        return None;
    }
    let mut nals = Vec::with_capacity(starts.len());
    for (idx, &(pos, code_len)) in starts.iter().enumerate() {
        let body_start = pos + code_len;
        let body_end = starts.get(idx + 1).map(|&(p, _)| p).unwrap_or(data.len());
        let body = data.get(body_start..body_end)?;
        let (nal_type, layer_id) = nal_header(body)?;
        nals.push(Nal {
            start_code_len: code_len,
            bytes: body,
            nal_type,
            layer_id,
        });
    }
    Some(nals)
}

fn split_length_delimited(data: &[u8], nal_length_size: u8) -> Option<Vec<Nal<'_>>> {
    let size = nal_length_size as usize;
    if !(1..=4).contains(&size) {
        return None;
    }
    let mut nals = Vec::new();
    let mut i = 0;
    while i < data.len() {
        let prefix = data.get(i..i + size)?;
        let len = prefix
            .iter()
            .fold(0usize, |acc, &b| (acc << 8) | b as usize);
        let body_start = i + size;
        let body = data.get(body_start..body_start + len)?;
        let (nal_type, layer_id) = nal_header(body)?;
        nals.push(Nal {
            start_code_len: size,
            bytes: body,
            nal_type,
            layer_id,
        });
        i = body_start + len;
    }
    Some(nals)
}

fn write_length(out: &mut Vec<u8>, len: usize, size: usize) {
    for shift in (0..size).rev() {
        out.push(((len >> (shift * 8)) & 0xFF) as u8);
    }
}

pub fn process_dv_sample_json(input: &str) -> Option<String> {
    let request = match serde_json::from_str::<SampleRequest>(input) {
        Ok(request) => request,
        Err(error) => {
            return serde_json::to_string(&SampleTransformResult {
                ok: false,
                error: Some(error.to_string()),
                ..Default::default()
            })
            .ok();
        }
    };

    let sample = match BASE64.decode(request.sample_base64.as_bytes()) {
        Ok(bytes) => bytes,
        Err(error) => {
            return serde_json::to_string(&SampleTransformResult {
                ok: false,
                error: Some(error.to_string()),
                ..Default::default()
            })
            .ok();
        }
    };

    if request.encrypted {
        return serde_json::to_string(&SampleTransformResult {
            ok: true,
            output_base64: Some(request.sample_base64),
            output_size: sample.len(),
            conversion_possible: false,
            error: Some("encrypted_samples".to_string()),
            ..Default::default()
        })
        .ok();
    }

    let plan = SampleExecutionPlan {
        rpu_mode: Some(request.mode),
        drop_el: request.drop_el,
        strip_dv_rpu: request.strip_dv_rpu,
        strip_hdr10plus: request.strip_hdr10_plus,
    };
    let result = transform(&sample, request.framing, request.nal_length_size, &plan)
        .unwrap_or_else(|error| SampleTransformResult {
            ok: false,
            output_base64: Some(request.sample_base64),
            output_size: sample.len(),
            error: Some(error.to_string()),
            ..Default::default()
        });
    serde_json::to_string(&result).ok()
}

#[allow(dead_code)]
pub(crate) fn process_dv_sample(
    sample: &[u8],
    framing: Framing,
    nal_length_size: u8,
    plan: &DvPlaybackPlan,
) -> SampleTransformResult {
    let execution_plan = SampleExecutionPlan::from(plan);
    transform(sample, framing, nal_length_size, &execution_plan).unwrap_or_else(|error| {
        SampleTransformResult {
            ok: false,
            output_base64: Some(BASE64.encode(sample)),
            output_size: sample.len(),
            error: Some(error.to_string()),
            ..Default::default()
        }
    })
}

enum SampleParseError {
    FailedToParseNalUnits,
}

impl std::fmt::Display for SampleParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SampleParseError::FailedToParseNalUnits => write!(f, "failed_to_parse_nal_units"),
        }
    }
}

#[cfg(feature = "fuzzing")]
pub fn fuzz_process_sample(data: &[u8]) {
    let Some((&flags, sample)) = data.split_first() else {
        return;
    };
    let framing = if flags & 1 == 0 {
        Framing::AnnexB
    } else {
        Framing::LengthDelimited
    };
    let nal_length_size = 1 + ((flags >> 1) & 0x3);
    let plan = SampleExecutionPlan {
        rpu_mode: Some(2),
        drop_el: flags & 0x10 != 0,
        strip_dv_rpu: flags & 0x20 != 0,
        strip_hdr10plus: flags & 0x40 != 0,
    };
    let _ = transform(sample, framing, nal_length_size, &plan);
}

fn transform(
    sample: &[u8],
    framing: Framing,
    nal_length_size: u8,
    plan: &SampleExecutionPlan,
) -> Result<SampleTransformResult, SampleParseError> {
    let nals = match framing {
        Framing::AnnexB => split_annex_b(sample),
        Framing::LengthDelimited => split_length_delimited(sample, nal_length_size),
    };
    let Some(nals) = nals else {
        return Err(SampleParseError::FailedToParseNalUnits);
    };

    let mut rpu_found = 0u32;
    let mut rpu_failed = 0u32;
    let mut enhancement_layer = EnhancementLayerKind::Unknown;
    let mut converted_rpu: Vec<Option<Vec<u8>>> = Vec::with_capacity(nals.len());
    let mut any_conversion_failed = false;

    for nal in &nals {
        if nal.nal_type != RPU_NAL_TYPE || nal.layer_id != 0 {
            converted_rpu.push(None);
            continue;
        }
        rpu_found += 1;
        if plan.strip_dv_rpu {
            converted_rpu.push(None);
            continue;
        }
        let Some(rpu_mode) = plan.rpu_mode else {
            converted_rpu.push(None);
            continue;
        };
        match DoviRpu::parse_unspec62_nalu(nal.bytes) {
            Ok(mut rpu) => {
                enhancement_layer = EnhancementLayerKind::from(rpu.el_type.as_ref());
                let wrong_output_profile =
                    |rpu: &DoviRpu| matches!(rpu_mode, 2 | 3) && rpu.dovi_profile != 8;
                let converted = match rpu.convert_with_mode(rpu_mode) {
                    Ok(()) if wrong_output_profile(&rpu) => None,
                    Ok(()) => rpu.write_hevc_unspec62_nalu().ok(),
                    Err(_) => None,
                };
                match converted {
                    Some(bytes) => converted_rpu.push(Some(bytes)),
                    None => {
                        rpu_failed += 1;
                        any_conversion_failed = true;
                        converted_rpu.push(None);
                    }
                }
            }
            Err(_) => {
                rpu_failed += 1;
                any_conversion_failed = true;
                converted_rpu.push(None);
            }
        }
    }

    if any_conversion_failed {
        return Ok(SampleTransformResult {
            ok: true,
            changed: false,
            output_base64: Some(BASE64.encode(sample)),
            output_size: sample.len(),
            rpu_found,
            rpu_converted: 0,
            rpu_failed,
            el_nals_dropped: 0,
            enhancement_layer,
            hdr10_plus_messages_removed: 0,
            conversion_possible: true,
            error: Some("rpu_conversion_failed_original_sample_kept".to_string()),
        });
    }

    let mut output = Vec::with_capacity(sample.len());
    let mut el_nals_dropped = 0u32;
    let mut rpu_removed = 0u32;
    let mut rpu_converted = 0u32;
    let is_annex_b = matches!(framing, Framing::AnnexB);
    let mut hdr10_plus_messages_removed = 0u32;

    for (nal, replacement) in nals.iter().zip(converted_rpu.iter()) {
        let is_el = plan.drop_el && (nal.layer_id != 0 || nal.nal_type == EL_NAL_TYPE_63);
        if is_el {
            el_nals_dropped += 1;
            continue;
        }
        if plan.strip_dv_rpu && nal.nal_type == RPU_NAL_TYPE && nal.layer_id == 0 {
            rpu_removed += 1;
            continue;
        }
        let bytes = if let Some(replacement) = replacement {
            rpu_converted += 1;
            replacement.as_slice()
        } else {
            nal.bytes
        };

        let is_sei = nal.layer_id == 0
            && (nal.nal_type == SEI_PREFIX_NAL_TYPE || nal.nal_type == SEI_SUFFIX_NAL_TYPE);
        let stripped_sei = if plan.strip_hdr10plus && is_sei {
            let (stripped, removed) = strip_hdr10plus_sei(bytes);
            hdr10_plus_messages_removed += removed;
            Some(stripped)
        } else {
            None
        };
        let bytes = stripped_sei.as_deref().unwrap_or(bytes);
        if bytes.len() <= 2 {
            continue;
        }

        if is_annex_b {
            output.extend_from_slice(&[0, 0, 0, 1]);
            output.extend_from_slice(bytes);
        } else {
            write_length(&mut output, bytes.len(), nal.start_code_len);
            output.extend_from_slice(bytes);
        }
    }

    let changed = rpu_converted > 0
        || rpu_removed > 0
        || el_nals_dropped > 0
        || hdr10_plus_messages_removed > 0;
    let output_base64 = BASE64.encode(&output);
    let output_size = output.len();

    Ok(SampleTransformResult {
        ok: true,
        changed,
        output_base64: Some(output_base64),
        output_size,
        rpu_found,
        rpu_converted,
        rpu_failed: 0,
        el_nals_dropped,
        enhancement_layer,
        hdr10_plus_messages_removed,
        conversion_possible: true,
        error: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn el_nal(layer_id: u8) -> Vec<u8> {
        let b0 = 0x02u8 | ((layer_id >> 5) & 0x1);
        let b1 = (layer_id & 0x1F) << 3;
        vec![b0, b1, 0xAA, 0xBB]
    }

    fn annex_b_sample(nals: &[Vec<u8>]) -> Vec<u8> {
        let mut out = Vec::new();
        for nal in nals {
            out.extend_from_slice(&[0, 0, 0, 1]);
            out.extend_from_slice(nal);
        }
        out
    }

    fn sei_nal(messages: &[(u32, Vec<u8>)]) -> Vec<u8> {
        let mut nal = vec![0x4E, 0x01];
        for (payload_type, data) in messages {
            nal.push(*payload_type as u8);
            nal.push(data.len() as u8);
            nal.extend_from_slice(data);
        }
        nal.push(0x80);
        nal
    }

    fn hdr10plus_message() -> Vec<u8> {
        vec![0xB5, 0x00, 0x3C, 0x00, 0x01, 0, 0, 0, 0, 0]
    }

    #[test]
    fn hdr10plus_message_stripped_other_sei_messages_preserved() {
        let vcl = vec![0x26, 0x01, 0x00, 0x11];
        let sei = sei_nal(&[(4, hdr10plus_message()), (137, vec![1, 2, 3])]);
        let sample = annex_b_sample(&[vcl, sei]);
        let request = format!(
            r#"{{"sampleBase64":"{}","framing":"annex_b","stripHdr10Plus":true}}"#,
            BASE64.encode(&sample)
        );
        let result: serde_json::Value =
            serde_json::from_str(&process_dv_sample_json(&request).unwrap()).unwrap();
        assert_eq!(result["hdr10PlusMessagesRemoved"], 1);
        assert_eq!(result["changed"], true);
        let out = BASE64
            .decode(result["outputBase64"].as_str().unwrap())
            .unwrap();
        let expected_sei = sei_nal(&[(137, vec![1, 2, 3])]);
        assert!(
            out.windows(expected_sei.len())
                .any(|window| window == expected_sei.as_slice()),
            "surviving SEI message should still be present"
        );
        assert!(
            !out.windows(hdr10plus_message().len())
                .any(|window| window == hdr10plus_message().as_slice()),
            "HDR10+ message bytes must not appear in output"
        );
    }

    #[test]
    fn sei_nal_with_only_hdr10plus_message_is_dropped_entirely() {
        let vcl = vec![0x26, 0x01, 0x00, 0x11];
        let sei = sei_nal(&[(4, hdr10plus_message())]);
        let sample = annex_b_sample(&[vcl.clone(), sei]);
        let request = format!(
            r#"{{"sampleBase64":"{}","framing":"annex_b","stripHdr10Plus":true}}"#,
            BASE64.encode(&sample)
        );
        let result: serde_json::Value =
            serde_json::from_str(&process_dv_sample_json(&request).unwrap()).unwrap();
        let out = BASE64
            .decode(result["outputBase64"].as_str().unwrap())
            .unwrap();
        assert_eq!(out, annex_b_sample(&[vcl]));
    }

    #[test]
    fn strip_hdr10_plus_defaults_to_off() {
        let vcl = vec![0x26, 0x01, 0x00, 0x11];
        let sei = sei_nal(&[(4, hdr10plus_message())]);
        let sample = annex_b_sample(&[vcl, sei.clone()]);
        let request = format!(
            r#"{{"sampleBase64":"{}","framing":"annex_b"}}"#,
            BASE64.encode(&sample)
        );
        let result: serde_json::Value =
            serde_json::from_str(&process_dv_sample_json(&request).unwrap()).unwrap();
        assert_eq!(result["changed"], false);
        let out = BASE64
            .decode(result["outputBase64"].as_str().unwrap())
            .unwrap();
        assert!(
            out.windows(sei.len())
                .any(|window| window == sei.as_slice())
        );
    }

    #[test]
    fn el_nal_with_nonzero_layer_id_is_dropped() {
        let vcl = vec![0x26, 0x01, 0x00, 0x11];
        let el = el_nal(1);
        let sample = annex_b_sample(&[vcl.clone(), el]);
        let request = format!(
            r#"{{"sampleBase64":"{}","framing":"annex_b"}}"#,
            BASE64.encode(&sample)
        );
        let result: serde_json::Value =
            serde_json::from_str(&process_dv_sample_json(&request).unwrap()).unwrap();
        assert_eq!(result["elNalsDropped"], 1);
        assert_eq!(result["changed"], true);
        let out = BASE64
            .decode(result["outputBase64"].as_str().unwrap())
            .unwrap();
        assert_eq!(out, annex_b_sample(&[vcl]));
    }

    #[test]
    fn nal_type_63_is_dropped_even_at_layer_zero() {
        let vcl = vec![0x26, 0x01, 0x00, 0x11];
        let el = vec![0x7E, 0x01, 0xAA];
        let sample = annex_b_sample(&[vcl.clone(), el]);
        let request = format!(
            r#"{{"sampleBase64":"{}","framing":"annex_b"}}"#,
            BASE64.encode(&sample)
        );
        let result: serde_json::Value =
            serde_json::from_str(&process_dv_sample_json(&request).unwrap()).unwrap();
        assert_eq!(result["elNalsDropped"], 1);
        let out = BASE64
            .decode(result["outputBase64"].as_str().unwrap())
            .unwrap();
        assert_eq!(out, annex_b_sample(&[vcl]));
    }

    #[test]
    fn no_el_or_rpu_present_gives_unchanged_output() {
        let vcl = vec![0x26, 0x01, 0x00, 0x11];
        let sample = annex_b_sample(&[vcl.clone()]);
        let request = format!(
            r#"{{"sampleBase64":"{}","framing":"annex_b"}}"#,
            BASE64.encode(&sample)
        );
        let result: serde_json::Value =
            serde_json::from_str(&process_dv_sample_json(&request).unwrap()).unwrap();
        assert_eq!(result["changed"], false);
        assert_eq!(result["elNalsDropped"], 0);
        assert_eq!(result["rpuFound"], 0);
    }

    #[test]
    fn encrypted_sample_is_never_rewritten() {
        let rpu = {
            let mut header = vec![0x7C, 0x01];
            header.extend_from_slice(&[0xAA; 8]);
            header
        };
        let sample = annex_b_sample(&[rpu]);
        let request = format!(
            r#"{{"sampleBase64":"{}","framing":"annex_b","encrypted":true}}"#,
            BASE64.encode(&sample)
        );
        let result: serde_json::Value =
            serde_json::from_str(&process_dv_sample_json(&request).unwrap()).unwrap();
        assert_eq!(result["conversionPossible"], false);
        assert_eq!(result["changed"], false);
        let out = BASE64
            .decode(result["outputBase64"].as_str().unwrap())
            .unwrap();
        assert_eq!(out, sample);
    }

    #[test]
    fn process_dv_sample_executes_a_convert_to_dv81_plan() {
        use crate::dolby_vision_plan::{
            DvContainer, DvFallbackMode, DvProfile, build_dv_playback_plan,
        };

        let (plan, _) = build_dv_playback_plan(
            DvProfile::P7,
            DvContainer::RawHevc,
            DvFallbackMode::ConvertDv81,
            true,
            false,
            false,
            false,
            true,
            false,
            false,
        );
        assert!(plan.validate().is_ok());

        let bl = vec![0x26, 0x01, 0x00, 0x11];
        let el = el_nal(1);
        let sample = annex_b_sample(&[bl.clone(), el]);

        let result = process_dv_sample(&sample, Framing::AnnexB, 4, &plan);
        assert_eq!(result.el_nals_dropped, 1);
        let out = BASE64.decode(result.output_base64.unwrap()).unwrap();
        assert_eq!(out, annex_b_sample(&[bl]));
    }

    #[test]
    fn process_dv_sample_executes_a_strip_to_hdr10_plan_by_dropping_the_rpu() {
        use crate::dolby_vision_plan::{
            DvContainer, DvFallbackMode, DvProfile, build_dv_playback_plan,
        };

        let (plan, _) = build_dv_playback_plan(
            DvProfile::P8Hdr10,
            DvContainer::Mp4,
            DvFallbackMode::Auto,
            true,
            false,
            false,
            false,
            false,
            false,
            false,
        );
        assert!(plan.strip_dv_rpu);

        let bl = vec![0x26, 0x01, 0x00, 0x11];
        let rpu = vec![0x7C, 0x01, 0xAA, 0xBB];
        let sample = annex_b_sample(&[bl.clone(), rpu]);

        let result = process_dv_sample(&sample, Framing::AnnexB, 4, &plan);
        assert_eq!(result.rpu_found, 1);
        assert_eq!(result.rpu_converted, 0);
        let out = BASE64.decode(result.output_base64.unwrap()).unwrap();
        assert_eq!(out, annex_b_sample(&[bl]));
    }

    #[test]
    fn malformed_sample_falls_back_to_original() {
        let request = format!(
            r#"{{"sampleBase64":"{}","framing":"annex_b"}}"#,
            BASE64.encode(b"not a valid nal stream at all")
        );
        let result: serde_json::Value =
            serde_json::from_str(&process_dv_sample_json(&request).unwrap()).unwrap();
        assert_eq!(result["ok"], false);
        assert!(result["outputBase64"].is_string());
    }

    #[test]
    fn length_delimited_framing_round_trips_size_prefix() {
        let vcl = vec![0x26, 0x01, 0x00, 0x11];
        let mut sample = Vec::new();
        write_length(&mut sample, vcl.len(), 4);
        sample.extend_from_slice(&vcl);
        let request = format!(
            r#"{{"sampleBase64":"{}","framing":"length_delimited","nalLengthSize":4}}"#,
            BASE64.encode(&sample)
        );
        let result: serde_json::Value =
            serde_json::from_str(&process_dv_sample_json(&request).unwrap()).unwrap();
        assert_eq!(result["changed"], false);
        let out = BASE64
            .decode(result["outputBase64"].as_str().unwrap())
            .unwrap();
        assert_eq!(out, sample);
    }

    #[test]
    fn length_delimited_framing_round_trips_every_prefix_width() {
        for size in 1u8..=4 {
            let vcl = vec![0x26, 0x01, 0x00, 0x11];
            let mut sample = Vec::new();
            write_length(&mut sample, vcl.len(), size as usize);
            sample.extend_from_slice(&vcl);
            let request = format!(
                r#"{{"sampleBase64":"{}","framing":"length_delimited","nalLengthSize":{}}}"#,
                BASE64.encode(&sample),
                size
            );
            let result: serde_json::Value =
                serde_json::from_str(&process_dv_sample_json(&request).unwrap()).unwrap();
            assert_eq!(result["changed"], false, "size={size}");
            let out = BASE64
                .decode(result["outputBase64"].as_str().unwrap())
                .unwrap();
            assert_eq!(out, sample, "size={size}");
        }
    }

    #[test]
    fn length_delimited_el_drop_round_trips_every_prefix_width() {
        for size in 1u8..=4 {
            let bl = vec![0x26, 0x01, 0x00, 0x11];
            let el = el_nal(1);
            let mut sample = Vec::new();
            write_length(&mut sample, bl.len(), size as usize);
            sample.extend_from_slice(&bl);
            write_length(&mut sample, el.len(), size as usize);
            sample.extend_from_slice(&el);

            let mut expected = Vec::new();
            write_length(&mut expected, bl.len(), size as usize);
            expected.extend_from_slice(&bl);

            let request = format!(
                r#"{{"sampleBase64":"{}","framing":"length_delimited","nalLengthSize":{}}}"#,
                BASE64.encode(&sample),
                size
            );
            let result: serde_json::Value =
                serde_json::from_str(&process_dv_sample_json(&request).unwrap()).unwrap();
            assert_eq!(result["elNalsDropped"], 1, "size={size}");
            let out = BASE64
                .decode(result["outputBase64"].as_str().unwrap())
                .unwrap();
            assert_eq!(out, expected, "size={size}");
        }
    }

    fn annex_b_sample_with_start_codes(nals: &[(usize, Vec<u8>)]) -> Vec<u8> {
        let mut out = Vec::new();
        for (start_code_len, nal) in nals {
            match start_code_len {
                3 => out.extend_from_slice(&[0, 0, 1]),
                _ => out.extend_from_slice(&[0, 0, 0, 1]),
            }
            out.extend_from_slice(nal);
        }
        out
    }

    #[test]
    fn annex_b_three_byte_start_codes_are_parsed() {
        let vcl = vec![0x26, 0x01, 0x00, 0x11];
        let sample = annex_b_sample_with_start_codes(&[(3, vcl.clone())]);
        let expected = annex_b_sample_with_start_codes(&[(4, vcl)]);
        let request = format!(
            r#"{{"sampleBase64":"{}","framing":"annex_b"}}"#,
            BASE64.encode(&sample)
        );
        let result: serde_json::Value =
            serde_json::from_str(&process_dv_sample_json(&request).unwrap()).unwrap();
        let out = BASE64
            .decode(result["outputBase64"].as_str().unwrap())
            .unwrap();
        assert_eq!(out, expected);
    }

    #[test]
    fn annex_b_mixed_three_and_four_byte_start_codes_drop_el_correctly() {
        let bl = vec![0x26, 0x01, 0x00, 0x11];
        let el = el_nal(1);
        let trailer = vec![0x24, 0x01, 0x99];
        let sample =
            annex_b_sample_with_start_codes(&[(4, bl.clone()), (3, el), (3, trailer.clone())]);
        let expected = annex_b_sample_with_start_codes(&[(4, bl), (4, trailer)]);

        let request = format!(
            r#"{{"sampleBase64":"{}","framing":"annex_b"}}"#,
            BASE64.encode(&sample)
        );
        let result: serde_json::Value =
            serde_json::from_str(&process_dv_sample_json(&request).unwrap()).unwrap();
        assert_eq!(result["elNalsDropped"], 1);
        let out = BASE64
            .decode(result["outputBase64"].as_str().unwrap())
            .unwrap();
        assert_eq!(out, expected);
    }
}
