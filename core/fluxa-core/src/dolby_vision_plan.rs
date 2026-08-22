use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum DvFallbackMode {
    Off,
    #[default]
    Auto,
    Dv8,
    ConvertDv81,
    Hdr10,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum DvProfile {
    P4,
    P5,
    P7,
    P8Hdr10,
    P8Hlg,
    P8Unknown,
    P10Hdr10,
    P10Other,
    Unknown,
}

impl DvProfile {
    pub(crate) fn label(self) -> &'static str {
        match self {
            DvProfile::P4 => "P4",
            DvProfile::P5 => "P5",
            DvProfile::P7 => "P7",
            DvProfile::P8Hdr10 => "P8.1",
            DvProfile::P8Hlg => "P8.4",
            DvProfile::P8Unknown => "P8",
            DvProfile::P10Hdr10 => "P10_compat1",
            DvProfile::P10Other => "P10_other",
            DvProfile::Unknown => "unknown",
        }
    }
}

pub(crate) enum DvContainer {
    Mkv,
    Mp4,
    RawHevc,
    Unknown,
}

#[derive(Debug, Clone, Copy, Default, Deserialize)]
#[serde(untagged)]
pub(crate) enum DvProfileCapability {
    #[default]
    Unset,
    Flag(bool),
    // advertised=false + runtime_verified=true is real: some decoders (e.g.
    // certain Amlogic SoCs) decode Profile 8 without ever listing it in
    // getCapabilitiesForType.
    Verified {
        #[serde(default)]
        advertised: bool,
        #[serde(default, rename = "runtimeVerified")]
        runtime_verified: bool,
    },
}

impl DvProfileCapability {
    fn supported(self) -> bool {
        match self {
            DvProfileCapability::Unset => false,
            DvProfileCapability::Flag(supported) => supported,
            DvProfileCapability::Verified {
                advertised,
                runtime_verified,
            } => advertised || runtime_verified,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DvDecoderCapabilities {
    #[serde(default)]
    pub(crate) profile4: DvProfileCapability,
    #[serde(default)]
    pub(crate) profile5: DvProfileCapability,
    #[serde(default)]
    pub(crate) profile7: DvProfileCapability,
    #[serde(default)]
    pub(crate) profile8: DvProfileCapability,
    #[serde(default)]
    pub(crate) profile10: DvProfileCapability,
}

pub(crate) fn decoder_supports(
    caps: Option<&DvDecoderCapabilities>,
    legacy_flag: bool,
    profile: DvProfile,
) -> bool {
    let Some(caps) = caps else {
        return legacy_flag;
    };
    match profile {
        DvProfile::P4 => caps.profile4.supported(),
        DvProfile::P5 => caps.profile5.supported(),
        DvProfile::P7 => caps.profile7.supported(),
        DvProfile::P8Hdr10 | DvProfile::P8Hlg | DvProfile::P8Unknown => caps.profile8.supported(),
        DvProfile::P10Hdr10 | DvProfile::P10Other => caps.profile10.supported(),
        DvProfile::Unknown => legacy_flag,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DvPlaybackAction {
    Native,
    ConvertToDv81,
    StripToHdr10,
    StripToHlg,
    #[allow(dead_code)]
    SoftwareToneMap,
    Unsupported,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DvPlanReason {
    UserDisabled,
    NotDolbyVisionContent,
    ManifestHandledByHost,
    NativeProfileSupported,
    NativeP7UnavailableButP8Verified,
    Hdr10CompatibleBaseLayer,
    HlgCompatibleBaseLayer,
    NoDolbyVisionDisplay,
    NoHdrBaseLayer,
    RawHevcRequiredForConversion,
    UnknownProfile,
    EncryptedSample,
    UserForcedHdr10,
    UserForcedDv81,
    NoSafeFallback,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub(crate) struct DvOutputSignaling {
    pub(crate) codec_profile: Option<u8>,
    pub(crate) codec_string_override: Option<String>,
    pub(crate) use_dolby_vision_mime: bool,
    pub(crate) media_codec_profile: Option<DvProfile>,
}

fn native_signaling(profile: DvProfile) -> DvOutputSignaling {
    DvOutputSignaling {
        codec_profile: None,
        codec_string_override: None,
        use_dolby_vision_mime: true,
        media_codec_profile: Some(profile),
    }
}

fn convert_to_dv81_signaling() -> DvOutputSignaling {
    DvOutputSignaling {
        codec_profile: Some(8),
        codec_string_override: None,
        use_dolby_vision_mime: true,
        media_codec_profile: Some(DvProfile::P8Hdr10),
    }
}

fn plain_hevc_signaling() -> DvOutputSignaling {
    DvOutputSignaling {
        codec_profile: None,
        codec_string_override: None,
        use_dolby_vision_mime: false,
        media_codec_profile: None,
    }
}

#[derive(Debug, Clone)]
pub(crate) struct DvPlaybackPlan {
    pub(crate) action: DvPlaybackAction,
    pub(crate) source_profile: DvProfile,
    pub(crate) output_profile: Option<DvProfile>,
    pub(crate) rpu_mode: Option<u8>,
    pub(crate) drop_el: bool,
    pub(crate) strip_dv_rpu: bool,
    pub(crate) strip_hdr10plus: bool,
    #[allow(dead_code)]
    pub(crate) zero_level5: bool,
    #[allow(dead_code)]
    pub(crate) output_signaling: DvOutputSignaling,
    pub(crate) reason: DvPlanReason,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DvPlanError {
    MissingOutputProfile,
    MissingRpuMode,
    MustDropEl,
    MustNotStripRpu,
}

impl DvPlaybackPlan {
    pub(crate) fn validate(&self) -> Result<(), DvPlanError> {
        if self.action == DvPlaybackAction::ConvertToDv81 {
            if self.output_profile != Some(DvProfile::P8Hdr10) {
                return Err(DvPlanError::MissingOutputProfile);
            }
            if self.rpu_mode.is_none() {
                return Err(DvPlanError::MissingRpuMode);
            }
            if !self.drop_el {
                return Err(DvPlanError::MustDropEl);
            }
            if self.strip_dv_rpu {
                return Err(DvPlanError::MustNotStripRpu);
            }
        }
        Ok(())
    }
}

// Deliberately excludes source_profile/output_signaling/reason so the
// transformer can't make its own profile-based decisions.
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct SampleExecutionPlan {
    pub(crate) rpu_mode: Option<u8>,
    pub(crate) drop_el: bool,
    pub(crate) strip_dv_rpu: bool,
    pub(crate) strip_hdr10plus: bool,
}

impl From<&DvPlaybackPlan> for SampleExecutionPlan {
    fn from(plan: &DvPlaybackPlan) -> Self {
        SampleExecutionPlan {
            rpu_mode: plan.rpu_mode,
            drop_el: plan.drop_el,
            strip_dv_rpu: plan.strip_dv_rpu,
            strip_hdr10plus: plan.strip_hdr10plus,
        }
    }
}

pub(crate) struct DvLegacyPlanDetails {
    pub(crate) action: &'static str,
    pub(crate) reason_code: &'static str,
    pub(crate) compatibility: &'static str,
    pub(crate) safety: &'static str,
    pub(crate) limitations: Vec<&'static str>,
}

fn legacy(
    action: &'static str,
    reason_code: &'static str,
    compatibility: &'static str,
    safety: &'static str,
    limitations: &[&'static str],
) -> DvLegacyPlanDetails {
    DvLegacyPlanDetails {
        action,
        reason_code,
        compatibility,
        safety,
        limitations: limitations.to_vec(),
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "planning inputs are all independent facts about the stream/device, not naturally groupable without an intermediate context struct this crate doesn't otherwise need"
)]
pub(crate) fn build_dv_playback_plan(
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
mod tests {
    use super::*;

    #[test]
    fn convert_to_dv81_plan_validates() {
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
        assert_eq!(plan.action, DvPlaybackAction::ConvertToDv81);
        assert!(plan.validate().is_ok());
    }

    #[test]
    fn convert_action_with_el_not_dropped_fails_validation() {
        let mut plan = DvPlaybackPlan {
            action: DvPlaybackAction::ConvertToDv81,
            source_profile: DvProfile::P7,
            output_profile: Some(DvProfile::P8Hdr10),
            rpu_mode: Some(2),
            drop_el: false,
            strip_dv_rpu: false,
            strip_hdr10plus: false,
            zero_level5: false,
            output_signaling: convert_to_dv81_signaling(),
            reason: DvPlanReason::UserForcedDv81,
        };
        assert_eq!(plan.validate(), Err(DvPlanError::MustDropEl));
        plan.drop_el = true;
        assert!(plan.validate().is_ok());
    }

    #[test]
    fn convert_action_stripping_rpu_fails_validation() {
        let plan = DvPlaybackPlan {
            action: DvPlaybackAction::ConvertToDv81,
            source_profile: DvProfile::P7,
            output_profile: Some(DvProfile::P8Hdr10),
            rpu_mode: Some(2),
            drop_el: true,
            strip_dv_rpu: true,
            strip_hdr10plus: false,
            zero_level5: false,
            output_signaling: convert_to_dv81_signaling(),
            reason: DvPlanReason::UserForcedDv81,
        };
        assert_eq!(plan.validate(), Err(DvPlanError::MustNotStripRpu));
    }

    #[test]
    fn convert_action_missing_rpu_mode_fails_validation() {
        let plan = DvPlaybackPlan {
            action: DvPlaybackAction::ConvertToDv81,
            source_profile: DvProfile::P7,
            output_profile: Some(DvProfile::P8Hdr10),
            rpu_mode: None,
            drop_el: true,
            strip_dv_rpu: false,
            strip_hdr10plus: false,
            zero_level5: false,
            output_signaling: convert_to_dv81_signaling(),
            reason: DvPlanReason::UserForcedDv81,
        };
        assert_eq!(plan.validate(), Err(DvPlanError::MissingRpuMode));
    }

    #[test]
    fn native_action_plan_always_validates() {
        let (plan, _) = build_dv_playback_plan(
            DvProfile::P7,
            DvContainer::Mkv,
            DvFallbackMode::Auto,
            true,
            false,
            false,
            true,
            true,
            true,
            false,
        );
        assert_eq!(plan.action, DvPlaybackAction::Native);
        assert!(plan.validate().is_ok());
    }

    #[test]
    fn strip_to_hdr10_signaling_disables_dolby_vision_mime() {
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
        assert_eq!(plan.action, DvPlaybackAction::StripToHdr10);
        assert!(!plan.output_signaling.use_dolby_vision_mime);
        assert!(plan.strip_dv_rpu);
    }

    #[test]
    fn convert_to_dv81_signaling_targets_profile_eight() {
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
        assert_eq!(plan.output_signaling.codec_profile, Some(8));
        assert_eq!(
            plan.output_signaling.media_codec_profile,
            Some(DvProfile::P8Hdr10)
        );
        assert!(plan.output_signaling.use_dolby_vision_mime);
    }

    #[test]
    fn p7_convert_reason_explains_p8_verified_fallback() {
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
        assert_eq!(plan.reason, DvPlanReason::NativeP7UnavailableButP8Verified);
        assert!(!plan.zero_level5);
    }

    #[test]
    fn user_forced_hdr10_mode_overrides_profile_derived_reason() {
        let (plan, _) = build_dv_playback_plan(
            DvProfile::P8Hdr10,
            DvContainer::Mp4,
            DvFallbackMode::Hdr10,
            true,
            false,
            false,
            false,
            false,
            false,
            false,
        );
        assert_eq!(plan.action, DvPlaybackAction::StripToHdr10);
        assert_eq!(plan.reason, DvPlanReason::UserForcedHdr10);
    }

    #[test]
    fn encrypted_sample_is_unsupported_regardless_of_profile() {
        let (plan, legacy) = build_dv_playback_plan(
            DvProfile::P7,
            DvContainer::RawHevc,
            DvFallbackMode::ConvertDv81,
            true,
            false,
            false,
            false,
            true,
            false,
            true,
        );
        assert_eq!(plan.action, DvPlaybackAction::Unsupported);
        assert_eq!(plan.reason, DvPlanReason::EncryptedSample);
        assert_eq!(legacy.reason_code, "encrypted_samples");
    }
}
