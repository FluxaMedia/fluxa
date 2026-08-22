package com.fluxa.app.shared.feature.settings

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.fluxa.app.common.AppStrings

@Composable
internal fun SettingsDeviceCapabilitiesContent(
    model: SettingsDeviceCapabilitiesUiModel,
    lang: String?
) {
    SettingsSectionHeader(AppStrings.t(lang, "settings.device_summary"))
    SettingsGroupCard {
        SettingsInfoRow(AppStrings.t(lang, "settings.device_name"), model.deviceName)
        SettingsInfoRow(AppStrings.t(lang, "settings.platform_version"), model.platformVersion)
        SettingsInfoRow(AppStrings.t(lang, "settings.audio_route"), model.audioRoute)
    }

    SettingsSectionHeader(AppStrings.t(lang, "settings.audio_capabilities"))
    SettingsGroupCard {
        SettingsInfoRow(AppStrings.t(lang, "settings.pcm_support"), model.audioPcmChannels)
        SettingsInfoRow(AppStrings.t(lang, "settings.sample_rates"), model.audioPcmSampleRates)
        CapabilityList(AppStrings.t(lang, "settings.detected_support"), model.audioPassthroughSupported, lang)
        CapabilityList(AppStrings.t(lang, "settings.not_detected"), model.audioPassthroughNotDetected, lang)
        SettingsInfoRow(AppStrings.t(lang, "settings.audio_fallback"), model.audioFallback)
    }

    SettingsSectionHeader(AppStrings.t(lang, "settings.video_capabilities"))
    SettingsGroupCard {
        CapabilityList(AppStrings.t(lang, "settings.hardware_decoders"), model.videoHardwareDecoders, lang)
        CapabilityList(AppStrings.t(lang, "settings.software_decoders"), model.videoSoftwareDecoders, lang)
        CapabilityList(AppStrings.t(lang, "settings.hdr_output"), model.hdrOutput, lang)
        CapabilityList(AppStrings.t(lang, "settings.not_detected"), model.videoNotDetected, lang)
    }
}

@Composable
private fun CapabilityList(title: String, values: List<String>, lang: String?) {
    SettingsSectionHeader(title)
    if (values.isEmpty()) {
        Text(
            AppStrings.t(lang, "settings.none_detected"),
            color = Color.White.copy(alpha = 0.5f),
            fontSize = 12.sp,
            modifier = Modifier.padding(vertical = 8.dp)
        )
    } else {
        Column {
            values.forEach { value ->
                Text("• $value", color = Color.White.copy(alpha = 0.78f), fontSize = 13.sp, modifier = Modifier.padding(vertical = 3.dp))
            }
        }
    }
}
