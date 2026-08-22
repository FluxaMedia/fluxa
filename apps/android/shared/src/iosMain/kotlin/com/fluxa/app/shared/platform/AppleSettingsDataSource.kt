package com.fluxa.app.shared.platform

import com.fluxa.app.shared.feature.settings.PersistentSettingsDataSource
import com.fluxa.app.shared.feature.settings.SettingsChoiceOption
import com.fluxa.app.shared.feature.settings.SettingsPreferenceKeys
import com.fluxa.app.shared.feature.settings.SettingsPreferenceStore
import com.fluxa.app.shared.feature.settings.SettingsDeviceCapabilitiesUiModel
import com.fluxa.app.shared.feature.settings.SettingsUiState
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.map
import platform.Foundation.NSBundle
import platform.Foundation.NSUserDefaults
import platform.UIKit.UIDevice

class AppleSettingsDataSource : PersistentSettingsDataSource(
    preferences = AppleSettingsPreferenceStore(NSUserDefaults.standardUserDefaults),
    appVersionLabel = appleAppVersionLabel(),
) {
    override fun observeSettings(): Flow<SettingsUiState> = super.observeSettings()
        .map { state ->
            state.copy(
                playback = state.playback.copy(
                    inAppPlayerOptions = listOf(
                        SettingsChoiceOption("internal", "In-app player"),
                    ),
                    externalPlayerOptions = listOf(
                        SettingsChoiceOption("infuse", "Infuse"),
                    ),
                ),
                deviceCapabilities = appleDeviceCapabilities(),
            )
        }
        .distinctUntilChanged()
}

/**
 * Apple deliberately owns codec negotiation, multichannel mapping, AirPlay
 * and Spatial Audio. The shared Kotlin/Native API does not expose the live
 * AVAudioSession route getters, so report ownership explicitly instead of
 * inventing a per-codec passthrough list that AVPlayer does not expose
 * reliably.
 */
private fun appleDeviceCapabilities(): SettingsDeviceCapabilitiesUiModel {
    return SettingsDeviceCapabilitiesUiModel(
        deviceName = UIDevice.currentDevice.model,
        platformVersion = "Apple ${UIDevice.currentDevice.systemVersion}",
        audioRoute = "Apple system-managed route (AVAudioSession)",
        audioPcmChannels = "Apple system-managed",
        audioPcmSampleRates = "Apple system-managed",
        audioPassthroughSupported = listOf("AVPlayer native codec negotiation", "Apple Spatial Audio / AirPlay route"),
        audioPassthroughNotDetected = emptyList(),
        audioFallback = "AVPlayer native decode → Apple audio session → PCM/Spatial Audio",
        videoHardwareDecoders = listOf("AVPlayer native H.264 / HEVC decoder"),
        videoSoftwareDecoders = emptyList(),
        videoNotDetected = listOf("Per-codec profile details are managed by Apple"),
        hdrOutput = emptyList(),
    )
}

private fun appleAppVersionLabel(): String =
    (NSBundle.mainBundle.infoDictionary?.get("CFBundleShortVersionString") as? String).orEmpty()

private class AppleSettingsPreferenceStore(
    private val defaults: NSUserDefaults,
) : SettingsPreferenceStore {
    override fun getString(key: String, default: String?): String? =
        defaults.stringForKey(platformKey(key)) ?: default

    override fun getBoolean(key: String, default: Boolean): Boolean {
        if (key == SettingsPreferenceKeys.USE_SKIP_SEGMENTS) {
            val intro = booleanOrNull("fluxa.apple.settings.useIntroDb")
            val aniSkip = booleanOrNull("fluxa.apple.settings.useAniSkip")
            return when {
                intro == null && aniSkip == null -> default
                else -> intro == true || aniSkip == true
            }
        }
        return booleanOrNull(platformKey(key)) ?: default
    }

    override fun getInt(key: String, default: Int): Int {
        val mapped = platformKey(key)
        return defaults.objectForKey(mapped)?.let { defaults.integerForKey(mapped).toInt() } ?: default
    }

    override fun getLong(key: String, default: Long): Long {
        val mapped = platformKey(key)
        return defaults.objectForKey(mapped)?.let { defaults.doubleForKey(mapped).toLong() } ?: default
    }

    override fun getFloat(key: String, default: Float): Float {
        val mapped = platformKey(key)
        return defaults.objectForKey(mapped)?.let { defaults.doubleForKey(mapped).toFloat() } ?: default
    }

    override fun putString(key: String, value: String?) {
        val mapped = platformKey(key)
        if (value == null) defaults.removeObjectForKey(mapped) else defaults.setObject(value, mapped)
    }

    override fun putBoolean(key: String, value: Boolean) {
        if (key == SettingsPreferenceKeys.USE_SKIP_SEGMENTS) {
            defaults.setBool(value, "fluxa.apple.settings.useIntroDb")
            defaults.setBool(value, "fluxa.apple.settings.useAniSkip")
            return
        }
        defaults.setBool(value, platformKey(key))
    }

    override fun putInt(key: String, value: Int) =
        defaults.setInteger(value.toLong(), platformKey(key))

    override fun putLong(key: String, value: Long) =
        defaults.setDouble(value.toDouble(), platformKey(key))

    override fun putFloat(key: String, value: Float) =
        defaults.setDouble(value.toDouble(), platformKey(key))

    private fun booleanOrNull(key: String): Boolean? =
        defaults.objectForKey(key)?.let { defaults.boolForKey(key) }

    private fun platformKey(key: String): String = when (key) {
        SettingsPreferenceKeys.LANGUAGE -> "fluxa.apple.language"
        else -> "fluxa.apple.settings.$key"
    }
}
