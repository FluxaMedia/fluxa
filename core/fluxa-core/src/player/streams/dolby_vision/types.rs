use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DvFallbackMode {
    Off,
    #[default]
    Auto,
    Dv8,
    ConvertDv81,
    Hdr10,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DvProfile {
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
    pub fn label(self) -> &'static str {
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

pub enum DvContainer {
    Mkv,
    Mp4,
    RawHevc,
    Unknown,
}

#[derive(Debug, Clone, Copy, Default, Deserialize)]
#[serde(untagged)]
pub enum DvProfileCapability {
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
pub struct DvDecoderCapabilities {
    #[serde(default)]
    pub profile4: DvProfileCapability,
    #[serde(default)]
    pub profile5: DvProfileCapability,
    #[serde(default)]
    pub profile7: DvProfileCapability,
    #[serde(default)]
    pub profile8: DvProfileCapability,
    #[serde(default)]
    pub profile10: DvProfileCapability,
}

pub fn decoder_supports(
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
pub enum DvPlaybackAction {
    Native,
    ConvertToDv81,
    StripToHdr10,
    StripToHlg,
    #[allow(dead_code)]
    SoftwareToneMap,
    Unsupported,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DvPlanReason {
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
pub struct DvOutputSignaling {
    pub codec_profile: Option<u8>,
    pub codec_string_override: Option<String>,
    pub use_dolby_vision_mime: bool,
    pub media_codec_profile: Option<DvProfile>,
}

pub(super) fn native_signaling(profile: DvProfile) -> DvOutputSignaling {
    DvOutputSignaling {
        codec_profile: None,
        codec_string_override: None,
        use_dolby_vision_mime: true,
        media_codec_profile: Some(profile),
    }
}

pub(super) fn convert_to_dv81_signaling() -> DvOutputSignaling {
    DvOutputSignaling {
        codec_profile: Some(8),
        codec_string_override: None,
        use_dolby_vision_mime: true,
        media_codec_profile: Some(DvProfile::P8Hdr10),
    }
}

pub(super) fn plain_hevc_signaling() -> DvOutputSignaling {
    DvOutputSignaling {
        codec_profile: None,
        codec_string_override: None,
        use_dolby_vision_mime: false,
        media_codec_profile: None,
    }
}

#[derive(Debug, Clone)]
pub struct DvPlaybackPlan {
    pub action: DvPlaybackAction,
    pub source_profile: DvProfile,
    pub output_profile: Option<DvProfile>,
    pub rpu_mode: Option<u8>,
    pub drop_el: bool,
    pub strip_dv_rpu: bool,
    pub strip_hdr10plus: bool,
    #[allow(dead_code)]
    pub zero_level5: bool,
    #[allow(dead_code)]
    pub output_signaling: DvOutputSignaling,
    pub reason: DvPlanReason,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DvPlanError {
    MissingOutputProfile,
    MissingRpuMode,
    MustDropEl,
    MustNotStripRpu,
}

impl DvPlaybackPlan {
    pub fn validate(&self) -> Result<(), DvPlanError> {
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
pub struct SampleExecutionPlan {
    pub rpu_mode: Option<u8>,
    pub drop_el: bool,
    pub strip_dv_rpu: bool,
    pub strip_hdr10plus: bool,
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

pub struct DvLegacyPlanDetails {
    pub action: &'static str,
    pub reason_code: &'static str,
    pub compatibility: &'static str,
    pub safety: &'static str,
    pub limitations: Vec<&'static str>,
}

pub(super) fn legacy(
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
