use serde_json::{Map, Value, json};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum HdrKind {
    Hdr10,
    Hlg,
}

#[derive(Clone, Copy, Debug)]
pub struct Mastering {
    pub primaries: Option<[(f64, f64); 3]>,
    pub white_point: Option<(f64, f64)>,
    pub luminance: Option<(f64, f64)>,
}

#[derive(Clone, Copy, Debug)]
pub struct HdrInfo {
    pub kind: HdrKind,
    pub mastering: Option<Mastering>,
    pub max_cll: Option<u32>,
    pub max_fall: Option<u32>,
    pub transfer: u32,
    pub primaries: u32,
    pub matrix: u32,
    pub full_range: bool,
}

const TRANSFER_PQ: u32 = 16;
const TRANSFER_HLG: u32 = 18;

pub fn kind_for_transfer(transfer: u32) -> Option<HdrKind> {
    match transfer {
        TRANSFER_PQ => Some(HdrKind::Hdr10),
        TRANSFER_HLG => Some(HdrKind::Hlg),
        _ => None,
    }
}

impl Mastering {
    pub fn bt2020_1000_nits() -> Mastering {
        Mastering {
            primaries: Some([(0.708, 0.292), (0.170, 0.797), (0.131, 0.046)]),
            white_point: Some((0.3127, 0.3290)),
            luminance: Some((0.0001, 1000.0)),
        }
    }
}

const CHROMATICITY: f64 = 50000.0;
const LUMINANCE: f64 = 10000.0;

pub fn hdr_payload(info: &HdrInfo, webos_major: u32) -> Option<String> {
    let mut sei = Map::new();
    if let Some(mastering) = &info.mastering {
        if let Some(primaries) = mastering.primaries {
            let order = [1, 2, 0];
            for (slot, index) in order.into_iter().enumerate() {
                sei.insert(format!("displayPrimariesX{slot}"), json!((primaries[index].0 * CHROMATICITY).round() as i64));
                sei.insert(format!("displayPrimariesY{slot}"), json!((primaries[index].1 * CHROMATICITY).round() as i64));
            }
        }
        if let Some((x, y)) = mastering.white_point {
            sei.insert("whitePointX".into(), json!((x * CHROMATICITY).round() as i64));
            sei.insert("whitePointY".into(), json!((y * CHROMATICITY).round() as i64));
        }
        if let Some((min, max)) = mastering.luminance {
            sei.insert("minDisplayMasteringLuminance".into(), json!((min * LUMINANCE).round() as i64));
            sei.insert("maxDisplayMasteringLuminance".into(), json!((max * LUMINANCE).round() as i64));
        }
    }
    if let Some(cll) = info.max_cll {
        sei.insert("maxContentLightLevel".into(), json!(cll));
    }
    if let Some(fall) = info.max_fall {
        sei.insert("maxPicAverageLightLevel".into(), json!(fall));
    }
    if sei.is_empty() {
        return None;
    }

    let legacy = webos_major < 5;
    let mut out = Map::new();
    out.insert(
        "hdrType".into(),
        json!(match info.kind {
            HdrKind::Hdr10 => "HDR10",
            HdrKind::Hlg => "HLG",
        }),
    );
    out.insert(if legacy { "mediaSei" } else { "sei" }.into(), Value::Object(sei));
    out.insert(
        if legacy { "mediaVui" } else { "vui" }.into(),
        json!({
            "transferCharacteristics": info.transfer,
            "colorPrimaries": info.primaries,
            "matrixCoeffs": info.matrix,
            "videoFullRangeFlag": info.full_range,
        }),
    );
    Some(Value::Object(out).to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn info(max_cll: Option<u32>) -> HdrInfo {
        HdrInfo {
            kind: HdrKind::Hdr10,
            mastering: None,
            max_cll,
            max_fall: None,
            transfer: 16,
            primaries: 9,
            matrix: 9,
            full_range: false,
        }
    }

    #[test]
    fn empty_sei_sends_nothing() {
        assert!(hdr_payload(&info(None), 6).is_none());
    }

    #[test]
    fn legacy_firmware_uses_media_prefixed_keys() {
        let old: Value = serde_json::from_str(&hdr_payload(&info(Some(1000)), 4).unwrap()).unwrap();
        assert!(old.get("mediaSei").is_some() && old.get("mediaVui").is_some());
        let new: Value = serde_json::from_str(&hdr_payload(&info(Some(1000)), 5).unwrap()).unwrap();
        assert!(new.get("sei").is_some() && new.get("vui").is_some());
    }

    #[test]
    fn primaries_are_reordered_from_gbr_to_rgb_slots() {
        let mut hdr = info(None);
        hdr.mastering = Some(Mastering {
            primaries: Some([(0.3, 0.6), (0.15, 0.06), (0.64, 0.33)]),
            white_point: Some((0.3127, 0.329)),
            luminance: Some((0.0001, 1000.0)),
        });
        let json: Value = serde_json::from_str(&hdr_payload(&hdr, 6).unwrap()).unwrap();
        assert_eq!(json["sei"]["displayPrimariesX0"], 7500);
        assert_eq!(json["sei"]["maxDisplayMasteringLuminance"], 10_000_000);
    }
}

#[cfg(test)]
mod classify_tests {
    use super::*;

    #[test]
    fn pq_and_hlg_are_recognised_and_sdr_is_not() {
        assert_eq!(kind_for_transfer(16), Some(HdrKind::Hdr10));
        assert_eq!(kind_for_transfer(18), Some(HdrKind::Hlg));
        assert_eq!(kind_for_transfer(1), None);
    }

    #[test]
    fn default_mastering_produces_a_payload() {
        let info = HdrInfo {
            kind: HdrKind::Hdr10,
            mastering: Some(Mastering::bt2020_1000_nits()),
            max_cll: None,
            max_fall: None,
            transfer: 16,
            primaries: 9,
            matrix: 9,
            full_range: false,
        };
        let payload = hdr_payload(&info, 6).unwrap();
        assert!(payload.contains("maxDisplayMasteringLuminance"));
    }
}
