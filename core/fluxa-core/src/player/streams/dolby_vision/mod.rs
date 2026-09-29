use serde::Deserialize;
use types::*;

mod types;
pub use types::*;

#[expect(
    clippy::too_many_arguments,
    reason = "planning inputs are all independent facts about the stream/device, not naturally groupable without an intermediate context struct this crate doesn't otherwise need"
)]
pub fn build_dv_playback_plan(
    source_profile: DvProfile,
    container: DvContainer,
    fallback_mode: DvFallbackMode,
    is_dolby_vision_content: bool,
    is_hls: bool,
    is_dash: bool,
    has_native_decoder: bool,
    has_p8_decoder: bool,
    has_dv_display: bool,
    encrypted: bool,
) -> (DvPlaybackPlan, DvLegacyPlanDetails) {
    let unsupported = |reason: DvPlanReason, legacy_details: DvLegacyPlanDetails| {
        (
            DvPlaybackPlan {
                action: DvPlaybackAction::Unsupported,
                source_profile,
                output_profile: None,
                rpu_mode: None,
                drop_el: false,
                strip_dv_rpu: false,
                strip_hdr10plus: false,
                zero_level5: false,
                output_signaling: plain_hevc_signaling(),
                reason,
            },
            legacy_details,
        )
    };

    if fallback_mode == DvFallbackMode::Off {
        return (
            DvPlaybackPlan {
                action: DvPlaybackAction::Native,
                source_profile: DvProfile::Unknown,
                output_profile: None,
                rpu_mode: None,
                drop_el: false,
                strip_dv_rpu: false,
                strip_hdr10plus: false,
                zero_level5: false,
                output_signaling: native_signaling(DvProfile::Unknown),
                reason: DvPlanReason::UserDisabled,
            },
            legacy("none", "user_disabled", "none", "high", &[]),
        );
    }

    if !is_dolby_vision_content {
        return (
            DvPlaybackPlan {
                action: DvPlaybackAction::Native,
                source_profile: DvProfile::Unknown,
                output_profile: None,
                rpu_mode: None,
                drop_el: false,
                strip_dv_rpu: false,
                strip_hdr10plus: false,
                zero_level5: false,
                output_signaling: native_signaling(DvProfile::Unknown),
                reason: DvPlanReason::NotDolbyVisionContent,
            },
            legacy("none", "not_dv", "none", "high", &[]),
        );
    }

    if encrypted {
        return (
            DvPlaybackPlan {
                action: DvPlaybackAction::Unsupported,
                source_profile,
                output_profile: None,
                rpu_mode: None,
                drop_el: false,
                strip_dv_rpu: false,
                strip_hdr10plus: false,
                zero_level5: false,
                output_signaling: plain_hevc_signaling(),
                reason: DvPlanReason::EncryptedSample,
            },
            legacy(
                "none",
                "encrypted_samples",
                "none",
                "none",
                &["sample_rewrite_requires_plaintext_bitstream"],
            ),
        );
    }

    let native_passthrough =
        has_native_decoder && (has_dv_display || fallback_mode != DvFallbackMode::ConvertDv81);
    if native_passthrough {
        return (
            DvPlaybackPlan {
                action: DvPlaybackAction::Native,
                source_profile,
                output_profile: Some(source_profile),
                rpu_mode: None,
                drop_el: false,
                strip_dv_rpu: false,
                strip_hdr10plus: false,
                zero_level5: false,
                output_signaling: native_signaling(source_profile),
                reason: DvPlanReason::NativeProfileSupported,
            },
            legacy("none", "hw_dv_decoder", "DV", "high", &[]),
        );
    }

    if is_hls || is_dash {
        if is_hls
            && matches!(source_profile, DvProfile::P7)
            && fallback_mode == DvFallbackMode::ConvertDv81
            && has_p8_decoder
        {
            return (
                DvPlaybackPlan {
                    action: DvPlaybackAction::ConvertToDv81,
                    source_profile,
                    output_profile: Some(DvProfile::P8Hdr10),
                    rpu_mode: Some(2),
                    drop_el: true,
                    strip_dv_rpu: false,
                    strip_hdr10plus: false,
                    zero_level5: false,
                    output_signaling: convert_to_dv81_signaling(),
                    reason: DvPlanReason::NativeP7UnavailableButP8Verified,
                },
                legacy(
                    "hls_rpu_convert",
                    "p7_hls_segment_rpu_convert",
                    "DV8",
                    "medium",
                    &[],
                ),
            );
        }
        return (
            DvPlaybackPlan {
                action: DvPlaybackAction::Native,
                source_profile,
                output_profile: None,
                rpu_mode: None,
                drop_el: false,
                strip_dv_rpu: false,
                strip_hdr10plus: false,
                zero_level5: false,
                output_signaling: native_signaling(source_profile),
                reason: DvPlanReason::ManifestHandledByHost,
            },
            legacy("none", "manifest_handled", "none", "high", &[]),
        );
    }

    match source_profile {
        DvProfile::P4 | DvProfile::P5 => {
            return unsupported(
                DvPlanReason::NoHdrBaseLayer,
                legacy(
                    "none",
                    "no_hdr_base_layer",
                    "none",
                    "none",
                    &["p4_p5_no_hdr_fallback_possible"],
                ),
            );
        }
        DvProfile::P10Other => {
            return unsupported(
                DvPlanReason::NoHdrBaseLayer,
                legacy(
                    "none",
                    "p10_compat_id_no_hdr_base",
                    "none",
                    "none",
                    &["only_p10_compat_id_1_has_hdr10_base"],
                ),
            );
        }
        DvProfile::Unknown => {
            return unsupported(
                DvPlanReason::UnknownProfile,
                legacy(
                    "none",
                    "unknown_profile_no_safe_fallback",
                    "none",
                    "none",
                    &["set_dvProfile_field_or_codec_string_for_safe_rewrite"],
                ),
            );
        }
        _ => {}
    }

    let (mut plan, legacy) = match source_profile {
        DvProfile::P7 => match (fallback_mode, &container) {
            (DvFallbackMode::ConvertDv81, _) if has_p8_decoder => (
                DvPlaybackPlan {
                    action: DvPlaybackAction::ConvertToDv81,
                    source_profile,
                    output_profile: Some(DvProfile::P8Hdr10),
                    rpu_mode: Some(2),
                    drop_el: true,
                    strip_dv_rpu: false,
                    strip_hdr10plus: false,
                    zero_level5: false,
                    output_signaling: convert_to_dv81_signaling(),
                    reason: DvPlanReason::NativeP7UnavailableButP8Verified,
                },
                legacy(
                    "rpu_convert",
                    "p7_rpu_convert_to_dv81",
                    "DV8",
                    "medium",
                    &[],
                ),
            ),
            (DvFallbackMode::Dv8, DvContainer::RawHevc) => (
                DvPlaybackPlan {
                    action: DvPlaybackAction::ConvertToDv81,
                    source_profile,
                    output_profile: Some(DvProfile::P8Hdr10),
                    rpu_mode: Some(2),
                    drop_el: true,
                    strip_dv_rpu: false,
                    strip_hdr10plus: false,
                    zero_level5: false,
                    output_signaling: convert_to_dv81_signaling(),
                    reason: DvPlanReason::UserForcedDv81,
                },
                legacy(
                    "rpu_convert",
                    "p7_rpu_convert_to_dv8_annexb",
                    "DV8",
                    "medium",
                    &["annexb_only"],
                ),
            ),
            (DvFallbackMode::Auto, DvContainer::RawHevc) if has_dv_display => (
                DvPlaybackPlan {
                    action: DvPlaybackAction::ConvertToDv81,
                    source_profile,
                    output_profile: Some(DvProfile::P8Hdr10),
                    rpu_mode: Some(2),
                    drop_el: true,
                    strip_dv_rpu: false,
                    strip_hdr10plus: false,
                    zero_level5: false,
                    output_signaling: convert_to_dv81_signaling(),
                    reason: DvPlanReason::NativeP7UnavailableButP8Verified,
                },
                legacy(
                    "rpu_convert",
                    "p7_rpu_convert_auto_dv_display_annexb",
                    "DV8",
                    "medium",
                    &["annexb_only"],
                ),
            ),
            (DvFallbackMode::Dv8, _) => (
                DvPlaybackPlan {
                    action: DvPlaybackAction::StripToHdr10,
                    source_profile,
                    output_profile: None,
                    rpu_mode: None,
                    drop_el: false,
                    strip_dv_rpu: true,
                    strip_hdr10plus: false,
                    zero_level5: false,
                    output_signaling: plain_hevc_signaling(),
                    reason: DvPlanReason::RawHevcRequiredForConversion,
                },
                legacy(
                    "dvcc_strip",
                    "rpu_convert_rejected_not_annexb",
                    "HDR10",
                    "medium",
                    &[
                        "rpu_convert_requires_annexb_hevc",
                        "container_is_not_raw_hevc_fallback_to_dvcc_strip",
                        "header_only_patch",
                        "does_not_transcode",
                        "does_not_remove_rpu_nals",
                    ],
                ),
            ),
            _ => (
                DvPlaybackPlan {
                    action: DvPlaybackAction::StripToHdr10,
                    source_profile,
                    output_profile: None,
                    rpu_mode: None,
                    drop_el: false,
                    strip_dv_rpu: true,
                    strip_hdr10plus: false,
                    zero_level5: false,
                    output_signaling: plain_hevc_signaling(),
                    reason: DvPlanReason::NoDolbyVisionDisplay,
                },
                legacy(
                    "dvcc_strip",
                    "p7_dvcc_strip_hdr10_base",
                    "HDR10",
                    "medium",
                    &[
                        "does_not_convert_bitstream",
                        "rpu_nals_remain_in_stream_ignored",
                        "header_only_patch",
                        "does_not_transcode",
                        "does_not_remove_rpu_nals",
                    ],
                ),
            ),
        },
        DvProfile::P8Hdr10 => (
            DvPlaybackPlan {
                action: DvPlaybackAction::StripToHdr10,
                source_profile,
                output_profile: None,
                rpu_mode: None,
                drop_el: false,
                strip_dv_rpu: true,
                strip_hdr10plus: false,
                zero_level5: false,
                output_signaling: plain_hevc_signaling(),
                reason: DvPlanReason::Hdr10CompatibleBaseLayer,
            },
            legacy(
                "dvcc_strip",
                "p8_1_hdr10_compat_base",
                "HDR10",
                "low",
                &[
                    "single_layer_hdr10_base_fully_compatible",
                    "header_only_patch",
                    "does_not_transcode",
                    "does_not_remove_rpu_nals",
                ],
            ),
        ),
        DvProfile::P8Hlg => (
            DvPlaybackPlan {
                action: DvPlaybackAction::StripToHlg,
                source_profile,
                output_profile: None,
                rpu_mode: None,
                drop_el: false,
                strip_dv_rpu: true,
                strip_hdr10plus: false,
                zero_level5: false,
                output_signaling: plain_hevc_signaling(),
                reason: DvPlanReason::HlgCompatibleBaseLayer,
            },
            legacy(
                "dvcc_strip",
                "p8_4_hlg_compat_base",
                "HLG",
                "medium",
                &[
                    "hlg_base_not_hdr10_color_rendering_may_differ",
                    "header_only_patch",
                    "does_not_transcode",
                    "does_not_remove_rpu_nals",
                ],
            ),
        ),
        DvProfile::P8Unknown => (
            DvPlaybackPlan {
                action: DvPlaybackAction::StripToHdr10,
                source_profile,
                output_profile: None,
                rpu_mode: None,
                drop_el: false,
                strip_dv_rpu: true,
                strip_hdr10plus: false,
                zero_level5: false,
                output_signaling: plain_hevc_signaling(),
                reason: DvPlanReason::NoSafeFallback,
            },
            legacy(
                "dvcc_strip",
                "p8_compat_id_unknown_hdr10_assumed",
                "HDR10_assumed",
                "medium",
                &[
                    "compat_id_unknown_hdr10_base_assumed",
                    "header_only_patch",
                    "does_not_transcode",
                    "does_not_remove_rpu_nals",
                ],
            ),
        ),
        DvProfile::P10Hdr10 => (
            DvPlaybackPlan {
                action: DvPlaybackAction::StripToHdr10,
                source_profile,
                output_profile: None,
                rpu_mode: None,
                drop_el: false,
                strip_dv_rpu: true,
                strip_hdr10plus: false,
                zero_level5: false,
                output_signaling: plain_hevc_signaling(),
                reason: DvPlanReason::Hdr10CompatibleBaseLayer,
            },
            legacy(
                "dvcc_strip",
                "p10_compat_id_1_hdr10_base",
                "HDR10",
                "medium",
                &[
                    "does_not_convert_bitstream",
                    "header_only_patch",
                    "does_not_transcode",
                    "does_not_remove_rpu_nals",
                ],
            ),
        ),
        _ => (
            DvPlaybackPlan {
                action: DvPlaybackAction::StripToHdr10,
                source_profile,
                output_profile: None,
                rpu_mode: None,
                drop_el: false,
                strip_dv_rpu: true,
                strip_hdr10plus: false,
                zero_level5: false,
                output_signaling: plain_hevc_signaling(),
                reason: DvPlanReason::NoSafeFallback,
            },
            legacy(
                "dvcc_strip",
                "unknown_profile_dvcc_strip_fallback",
                "HDR10_assumed",
                "medium",
                &[
                    "header_only_patch",
                    "does_not_transcode",
                    "does_not_remove_rpu_nals",
                ],
            ),
        ),
    };

    if fallback_mode == DvFallbackMode::Hdr10 && plan.action == DvPlaybackAction::StripToHdr10 {
        plan.reason = DvPlanReason::UserForcedHdr10;
    }

    (plan, legacy)
}

#[cfg(test)]
mod tests;
