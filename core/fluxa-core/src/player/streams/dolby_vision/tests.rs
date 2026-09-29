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
