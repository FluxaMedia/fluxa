use ::dolby_vision::rpu::dovi_rpu::DoviRpu;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct RpuRequest {
    #[serde(default)]
    rpu_hex: String,
    #[serde(default)]
    rpu_base64: String,
    #[serde(default = "default_mode")]
    mode: u8,
}

#[derive(Serialize)]
struct RpuInfo {
    ok: bool,
    profile: Option<u8>,
    el_type: Option<String>,
    error: Option<String>,
}

#[derive(Serialize)]
struct RpuConvertResult {
    ok: bool,
    profile_before: Option<u8>,
    profile_after: Option<u8>,
    el_type_before: Option<String>,
    el_type_after: Option<String>,
    rpu_hex: Option<String>,
    rpu_base64: Option<String>,
    error: Option<String>,
}

fn default_mode() -> u8 {
    2
}

pub fn dolby_vision_rpu_info_json(input: &str) -> Option<String> {
    let request = serde_json::from_str::<RpuRequest>(input).ok()?;
    let bytes = request.rpu_bytes().map_err(|error| error.to_string());
    let result = match bytes {
        Ok(bytes) => match DoviRpu::parse_unspec62_nalu(&bytes) {
            Ok(rpu) => RpuInfo {
                ok: true,
                profile: Some(rpu.dovi_profile),
                el_type: rpu.el_type.as_ref().map(|value| format!("{value:?}")),
                error: None,
            },
            Err(error) => RpuInfo {
                ok: false,
                profile: None,
                el_type: None,
                error: Some(error.to_string()),
            },
        },
        Err(error) => RpuInfo {
            ok: false,
            profile: None,
            el_type: None,
            error: Some(error),
        },
    };
    serde_json::to_string(&result).ok()
}

pub fn dolby_vision_convert_rpu_json(input: &str) -> Option<String> {
    let request = serde_json::from_str::<RpuRequest>(input).ok()?;
    let bytes = request.rpu_bytes().map_err(|error| error.to_string());
    let result = match bytes {
        Ok(bytes) => match DoviRpu::parse_unspec62_nalu(&bytes) {
            Ok(mut rpu) => {
                let profile_before = rpu.dovi_profile;
                let el_type_before = rpu.el_type.as_ref().map(|value| format!("{value:?}"));
                match rpu.convert_with_mode(request.mode) {
                    Ok(()) => match rpu.write_hevc_unspec62_nalu() {
                        Ok(out) => RpuConvertResult {
                            ok: true,
                            profile_before: Some(profile_before),
                            profile_after: Some(rpu.dovi_profile),
                            el_type_before,
                            el_type_after: rpu.el_type.as_ref().map(|value| format!("{value:?}")),
                            rpu_hex: Some(hex_encode(&out)),
                            rpu_base64: Some(BASE64.encode(&out)),
                            error: None,
                        },
                        Err(error) => {
                            convert_error(profile_before, el_type_before, error.to_string())
                        }
                    },
                    Err(error) => convert_error(profile_before, el_type_before, error.to_string()),
                }
            }
            Err(error) => convert_error_empty(error.to_string()),
        },
        Err(error) => convert_error_empty(error),
    };
    serde_json::to_string(&result).ok()
}

fn convert_error(
    profile_before: u8,
    el_type_before: Option<String>,
    error: String,
) -> RpuConvertResult {
    RpuConvertResult {
        ok: false,
        profile_before: Some(profile_before),
        profile_after: None,
        el_type_before,
        el_type_after: None,
        rpu_hex: None,
        rpu_base64: None,
        error: Some(error),
    }
}

fn convert_error_empty(error: String) -> RpuConvertResult {
    RpuConvertResult {
        ok: false,
        profile_before: None,
        profile_after: None,
        el_type_before: None,
        el_type_after: None,
        rpu_hex: None,
        rpu_base64: None,
        error: Some(error),
    }
}

impl RpuRequest {
    fn rpu_bytes(&self) -> Result<Vec<u8>, String> {
        if !self.rpu_base64.is_blank() {
            return BASE64
                .decode(self.rpu_base64.as_bytes())
                .map_err(|error| error.to_string());
        }
        hex_decode(&self.rpu_hex)
    }
}

trait Blank {
    fn is_blank(&self) -> bool;
}

impl Blank for String {
    fn is_blank(&self) -> bool {
        self.trim().is_empty()
    }
}

fn hex_decode(value: &str) -> Result<Vec<u8>, String> {
    // Collect into chars (not a String) and decode each digit individually —
    // slicing a String by byte range assumes 1 char == 1 byte, which a
    // multi-byte UTF-8 character in malformed input would violate and panic on.
    let clean = value
        .chars()
        .filter(|ch| !ch.is_whitespace() && *ch != ':')
        .collect::<Vec<char>>();
    if clean.len() % 2 != 0 {
        return Err("hex length must be even".to_string());
    }
    clean
        .chunks(2)
        .map(|pair| match pair {
            [first, second] => match (first.to_digit(16), second.to_digit(16)) {
                (Some(hi), Some(lo)) => Ok((hi * 16 + lo) as u8),
                _ => Err(format!("invalid hex digit in pair: {first}{second}")),
            },
            _ => Err("internal hex pair invariant violated".to_string()),
        })
        .collect()
}

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_decode_strips_whitespace_and_colons() {
        assert_eq!(hex_decode("AB:CD EF").unwrap(), vec![0xAB, 0xCD, 0xEF]);
    }

    #[test]
    fn hex_decode_rejects_malformed_input_instead_of_panicking() {
        // A multi-byte UTF-8 character mixed into otherwise-hex-looking input used
        // to panic (byte-range slicing assumed 1 char == 1 byte); it must now
        // surface as an error.
        assert!(hex_decode("aébb").is_err());
        assert!(
            hex_decode("abc").is_err(),
            "odd length must error, not panic"
        );
    }
}
