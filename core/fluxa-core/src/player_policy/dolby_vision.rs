use crate::core_error::{CoreError, LogAndDiscard};
use crate::dolby_vision_plan::{
    DvContainer, DvDecoderCapabilities, DvFallbackMode, DvProfile, build_dv_playback_plan,
    decoder_supports,
};
use serde::Deserialize;
use serde_json::{Value, json};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DvProxyPlanRequest {
    #[serde(default)]
    stream: Value,
    #[serde(default)]
    url: String,
    #[serde(default)]
    fallback_mode: DvFallbackMode,
    #[serde(default)]
    device_has_dv_decoder: bool,
    #[serde(default)]
    device_has_dv_display: bool,
    /// Per-profile decoder capabilities, as reported by a runtime MediaCodecList
    /// probe. When present this overrides `device_has_dv_decoder` for the
    /// profile actually detected in the stream — a device that decodes P5/P8
    /// but not P7 should still get a P7->P8.1 conversion plan, not a blanket
    /// "no DV decoder" fallback.
    #[serde(default)]
    device_capabilities: Option<DvDecoderCapabilities>,
    #[serde(default)]
    encrypted: bool,
}

/// Response fields:
///   action        — "none" | "dvcc_strip" | "rpu_convert" | "hls_rpu_convert"
///   rpuMode       — libdovi convert mode (2 = Profile 8)
///   reason        — machine-readable decision code
///   profile       — detected DV profile ("P7", "P8.1", …, "unknown")
///   compatibility — expected output format ("HDR10", "HLG", "DV8", "DV", "none")
///   safety        — "high" | "medium" | "low" | "none"
///   limitations   — list of known caveats for this action
pub(crate) fn dv_proxy_plan_json(request_json: &str) -> Option<String> {
    let req = serde_json::from_str::<DvProxyPlanRequest>(request_json)
        .map_err(|e| CoreError::BadInput {
            context: "dv_proxy_plan_json",
            detail: e.to_string(),
        })
        .log_discard()?;

    let url_lower = req.url.to_lowercase();
    let is_hls = url_lower.ends_with(".m3u8") || url_lower.contains(".m3u8?");
    let is_dash = url_lower.ends_with(".mpd") || url_lower.contains(".mpd?");
    let is_dolby_vision_content =
        req.fallback_mode != DvFallbackMode::Off && is_dolby_vision_stream(&req.stream, &req.url);

    let profile = if is_dolby_vision_content {
        detect_dv_profile(&req.stream)
    } else {
        DvProfile::Unknown
    };
    let container = detect_container(&req.url);
    let caps = req.device_capabilities.as_ref();
    let has_native_decoder = decoder_supports(caps, req.device_has_dv_decoder, profile);
    let has_p8_decoder = decoder_supports(caps, req.device_has_dv_decoder, DvProfile::P8Unknown);

    let (plan, legacy) = build_dv_playback_plan(
        profile,
        container,
        req.fallback_mode,
        is_dolby_vision_content,
        is_hls,
        is_dash,
        has_native_decoder,
        has_p8_decoder,
        req.device_has_dv_display,
        req.encrypted,
    );

    if let Err(error) = plan.validate() {
        crate::log_sink::record(
            "dv_proxy_plan_json",
            &format!("planner produced an invalid plan: {error:?}"),
        );
    }

    serde_json::to_string(&json!({
        "action": legacy.action,
        "rpuMode": plan.rpu_mode.unwrap_or(2),
        "reason": legacy.reason_code,
        "profile": plan.source_profile.label(),
        "compatibility": legacy.compatibility,
        "safety": legacy.safety,
        "limitations": legacy.limitations,
    }))
    .ok()
}

/// Derive the DV profile from stream metadata, codec strings, and text hints.
fn detect_dv_profile(stream: &Value) -> DvProfile {
    // 1. Explicit integer fields set by the addon.
    let profile_num = stream
        .get("dvProfile")
        .or_else(|| stream.get("dv_profile"))
        .and_then(Value::as_i64);
    let compat_id = stream
        .get("dvCompatId")
        .or_else(|| stream.get("dvCompatibility"))
        .and_then(Value::as_i64);
    if let Some(p) = profile_num {
        return profile_from_nums(p, compat_id);
    }

    // 2. ISO-BMFF / HLS codec string: "dvhe.07.06", "dvh1.08.01", …
    let codecs = stream.get("codecs").and_then(Value::as_str).unwrap_or("");
    if let Some(p) = parse_dv_codec_string(codecs) {
        return p;
    }

    // 3. Codec token embedded in freetext fields (e.g., "dvhe.07.06 BDRemux").
    let name = stream.get("name").and_then(Value::as_str).unwrap_or("");
    let desc = stream
        .get("description")
        .and_then(Value::as_str)
        .unwrap_or("");
    let filename = stream
        .get("effectiveFilename")
        .or_else(|| stream.get("filename"))
        .and_then(Value::as_str)
        .unwrap_or("");
    let text = format!("{} {} {}", name, desc, filename);
    if let Some(p) = parse_dv_codec_string(&text) {
        return p;
    }

    // 4. Short profile tokens: "P8.1", "P7", "P8", …
    parse_dv_profile_text(&text).unwrap_or(DvProfile::Unknown)
}

fn profile_from_nums(profile: i64, compat_id: Option<i64>) -> DvProfile {
    match profile {
        4 => DvProfile::P4,
        5 => DvProfile::P5,
        7 => DvProfile::P7,
        8 => match compat_id {
            Some(1) => DvProfile::P8Hdr10,
            Some(4) => DvProfile::P8Hlg,
            _ => DvProfile::P8Unknown,
        },
        10 => match compat_id {
            Some(1) => DvProfile::P10Hdr10,
            _ => DvProfile::P10Other,
        },
        _ => DvProfile::Unknown,
    }
}

// The second field is level, not dv_bl_signal_compatibility_id — never derive compat_id from it.
fn parse_dv_codec_string(text: &str) -> Option<DvProfile> {
    let lower = text.to_lowercase();
    for prefix in &["dvhe.", "dvh1.", "dva1.", "dvav."] {
        if let Some(pos) = lower.find(prefix) {
            let after = &text[pos + prefix.len()..];
            let mut parts = after.splitn(3, '.');
            let profile: i64 = leading_digits(parts.next()?)?.parse().ok()?;
            return Some(profile_from_nums(profile, None));
        }
    }
    None
}

fn leading_digits(s: &str) -> Option<&str> {
    let end = s.find(|c: char| !c.is_ascii_digit()).unwrap_or(s.len());
    if end == 0 { None } else { Some(&s[..end]) }
}

/// Recognise short profile tokens ("P8.1", "P7", "P8") in freetext.
fn parse_dv_profile_text(text: &str) -> Option<DvProfile> {
    // Ordered so longer patterns match before their shorter prefixes.
    let patterns: &[(&str, DvProfile)] = &[
        ("P8.1", DvProfile::P8Hdr10),
        ("P8.4", DvProfile::P8Hlg),
        ("P7", DvProfile::P7),
        ("P8", DvProfile::P8Unknown),
        ("P10", DvProfile::P10Other),
        ("P5", DvProfile::P5),
        ("P4", DvProfile::P4),
    ];
    for (pat, profile) in patterns {
        if contains_word(text, pat) {
            return Some(*profile);
        }
    }
    None
}

/// True when `word` appears in `text` surrounded by non-alphanumeric (or absent) bytes.
#[expect(
    clippy::indexing_slicing,
    reason = "ASCII byte checks are guarded by explicit length bounds"
)]
fn contains_word(text: &str, word: &str) -> bool {
    let tb = text.as_bytes();
    let wb = word.as_bytes();
    let wlen = wb.len();
    if tb.len() < wlen {
        return false;
    }
    for i in 0..=(tb.len() - wlen) {
        if &tb[i..i + wlen] == wb {
            let before_ok = i == 0 || !tb[i - 1].is_ascii_alphanumeric();
            let after_ok = i + wlen >= tb.len() || !tb[i + wlen].is_ascii_alphanumeric();
            if before_ok && after_ok {
                return true;
            }
        }
    }
    false
}

/// Returns true when the stream or URL is identifiable as Dolby Vision content.
fn is_dolby_vision_stream(stream: &Value, url: &str) -> bool {
    if stream.get("dv").and_then(Value::as_bool).unwrap_or(false)
        || stream
            .get("dolbyVision")
            .and_then(Value::as_bool)
            .unwrap_or(false)
        || stream.get("dvProfile").and_then(Value::as_i64).is_some()
    {
        return true;
    }

    let name = stream.get("name").and_then(Value::as_str).unwrap_or("");
    let desc = stream
        .get("description")
        .and_then(Value::as_str)
        .unwrap_or("");
    let filename = stream
        .get("effectiveFilename")
        .or_else(|| stream.get("filename"))
        .and_then(Value::as_str)
        .unwrap_or("");

    let lower = format!("{} {} {} {}", name, desc, filename, url).to_lowercase();

    if lower.contains("dvhe")
        || lower.contains("dvh1")
        || lower.contains("dva1")
        || lower.contains("dvav")
        || lower.contains("dolby vision")
        || lower.contains("dolby-vision")
        || lower.contains("dovi")
    {
        return true;
    }

    // "DV" as a standalone token (case-sensitive — avoids "DVD", "HDVD", etc.).
    is_standalone_dv_token(&format!("{} {} {} {}", name, desc, filename, url))
}

#[expect(
    clippy::indexing_slicing,
    reason = "ASCII token scan checks every index against the byte length"
)]
fn is_standalone_dv_token(text: &str) -> bool {
    let bytes = text.as_bytes();
    let len = bytes.len();
    let mut i = 0;
    while i + 1 < len {
        if bytes[i] == b'D' && bytes[i + 1] == b'V' {
            let before_ok = i == 0 || !bytes[i - 1].is_ascii_alphabetic();
            let after_ok = i + 2 >= len || !bytes[i + 2].is_ascii_alphanumeric();
            if before_ok && after_ok {
                return true;
            }
        }
        i += 1;
    }
    false
}

fn detect_container(url: &str) -> DvContainer {
    let path = url.split('?').next().unwrap_or(url).to_lowercase();
    let ext = path.rsplit('.').next().unwrap_or("");
    match ext {
        "mkv" | "mk3d" | "mka" | "mks" | "webm" => DvContainer::Mkv,
        "mp4" | "m4v" | "m4a" | "mov" => DvContainer::Mp4,
        "hevc" | "h265" | "265" => DvContainer::RawHevc,
        _ => DvContainer::Unknown,
    }
}

#[expect(
    clippy::indexing_slicing,
    reason = "episode format is length-checked before accessing its fixed fields"
)]
pub(crate) fn episode_path_matches_id(path: &str, video_id: &str) -> bool {
    let parts: Vec<&str> = video_id.split(':').collect();
    if parts.len() < 3 {
        return false;
    }
    let season = parts[1].parse::<i32>().unwrap_or(0);
    let episode = parts[2].parse::<i32>().unwrap_or(0);
    if season == 0 || episode == 0 {
        return false;
    }
    let path_lower = path.to_lowercase();
    let pattern_s_e = format!("s{:02}e{:02}", season, episode);
    let pattern_sx_ex = format!("{}x{:02}", season, episode);
    let pattern_ep = format!("e{:02}", episode);
    path_lower.contains(&pattern_s_e)
        || path_lower.contains(&pattern_sx_ex)
        || path_lower.contains(&pattern_ep)
}
