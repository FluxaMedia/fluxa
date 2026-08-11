package com.fluxa.app.player

import android.content.Context
import android.hardware.display.DisplayManager
import android.media.MediaCodec
import android.media.MediaCodecInfo.CodecProfileLevel
import android.media.MediaCodecList
import android.media.MediaFormat
import android.os.Build
import android.view.Display
import androidx.media3.common.MimeTypes

data class DvProfileRuntimeCapability(val advertised: Boolean, val runtimeVerified: Boolean)

data class DvDeviceRuntimeCapabilities(
    val profile5: DvProfileRuntimeCapability,
    val profile7: DvProfileRuntimeCapability,
    val profile8: DvProfileRuntimeCapability
)

object AndroidDolbyVisionCapabilities {
    private const val PREFS_NAME = "fluxa_dv_capability_probe"

    fun detect(context: Context): DolbyVisionCapabilities {
        val hdrTypes = queryDisplayHdrTypes(context)
        val decoders = queryDecoderProfiles()
        return DolbyVisionCapabilities(
            displaySupportsDolbyVision = hdrTypes.contains(Display.HdrCapabilities.HDR_TYPE_DOLBY_VISION),
            displaySupportsHdr10 = hdrTypes.contains(Display.HdrCapabilities.HDR_TYPE_HDR10),
            displaySupportsHdr10Plus = hdrTypes.contains(Display.HdrCapabilities.HDR_TYPE_HDR10_PLUS),
            displaySupportsHlg = hdrTypes.contains(Display.HdrCapabilities.HDR_TYPE_HLG),
            decoderNativeP7 = decoders.nativeP7,
            decoderP5 = decoders.p5,
            decoderP8 = decoders.p8,
            decoderAnyDv = decoders.anyDv
        )
    }

    fun detectRuntimeCapabilities(context: Context): DvDeviceRuntimeCapabilities {
        val info = decoderInfoOrNull()
        val prefs = context.applicationContext.getSharedPreferences(PREFS_NAME, Context.MODE_PRIVATE)
        return DvDeviceRuntimeCapabilities(
            profile5 = capabilityFor(info, prefs, CodecProfileLevel.DolbyVisionProfileDvheStn, "p5"),
            profile7 = capabilityFor(info, prefs, CodecProfileLevel.DolbyVisionProfileDvheDtb, "p7"),
            profile8 = capabilityFor(info, prefs, CodecProfileLevel.DolbyVisionProfileDvheSt, "p8")
        )
    }

    private data class DecoderInfo(val codecName: String, val advertisedProfiles: Set<Int>)

    private fun decoderInfoOrNull(): DecoderInfo? {
        if (Build.VERSION.SDK_INT < Build.VERSION_CODES.N) return null
        return runCatching {
            for (info in MediaCodecList(MediaCodecList.REGULAR_CODECS).codecInfos) {
                if (info.isEncoder) continue
                if (!info.supportedTypes.any { it.equals(MimeTypes.VIDEO_DOLBY_VISION, ignoreCase = true) }) continue
                val capabilities = runCatching {
                    info.getCapabilitiesForType(MimeTypes.VIDEO_DOLBY_VISION)
                }.getOrNull() ?: continue
                val profiles = capabilities.profileLevels?.map { it.profile }?.toSet() ?: emptySet()
                return@runCatching DecoderInfo(info.name, profiles)
            }
            null
        }.getOrNull()
    }

    private fun capabilityFor(
        info: DecoderInfo?,
        prefs: android.content.SharedPreferences,
        profile: Int,
        label: String
    ): DvProfileRuntimeCapability {
        if (info == null) return DvProfileRuntimeCapability(advertised = false, runtimeVerified = false)
        val advertised = info.advertisedProfiles.contains(profile)
        val verified = if (advertised) true else probeRuntimeVerified(info.codecName, profile, prefs, label)
        return DvProfileRuntimeCapability(advertised, verified)
    }

    private fun probeRuntimeVerified(
        codecName: String,
        profile: Int,
        prefs: android.content.SharedPreferences,
        label: String
    ): Boolean {
        val cacheKey = "$codecName:$label:${Build.FINGERPRINT}"
        if (prefs.contains(cacheKey)) return prefs.getBoolean(cacheKey, false)

        val result = runCatching {
            val format = MediaFormat.createVideoFormat(MimeTypes.VIDEO_DOLBY_VISION, 1920, 1080).apply {
                setInteger(MediaFormat.KEY_PROFILE, profile)
            }
            val codec = MediaCodec.createByCodecName(codecName)
            try {
                codec.configure(format, null, null, 0)
                codec.start()
                codec.stop()
                true
            } finally {
                codec.release()
            }
        }.getOrDefault(false)

        prefs.edit().putBoolean(cacheKey, result).apply()
        return result
    }

    @Suppress("DEPRECATION")
    private fun queryDisplayHdrTypes(context: Context): IntArray {
        val displayManager = context.getSystemService(Context.DISPLAY_SERVICE) as? DisplayManager
        val display = displayManager?.getDisplay(Display.DEFAULT_DISPLAY) ?: return IntArray(0)
        return display.hdrCapabilities?.supportedHdrTypes ?: IntArray(0)
    }

    private data class DecoderProfiles(
        val nativeP7: Boolean,
        val p5: Boolean,
        val p8: Boolean,
        val anyDv: Boolean
    )

    private fun queryDecoderProfiles(): DecoderProfiles {
        if (Build.VERSION.SDK_INT < Build.VERSION_CODES.N) return DecoderProfiles(false, false, false, false)
        return runCatching {
            var nativeP7 = false
            var p5 = false
            var p8 = false
            var anyDv = false
            for (info in MediaCodecList(MediaCodecList.REGULAR_CODECS).codecInfos) {
                if (info.isEncoder) continue
                if (!info.supportedTypes.any { it.equals(MimeTypes.VIDEO_DOLBY_VISION, ignoreCase = true) }) continue
                anyDv = true
                val capabilities = runCatching {
                    info.getCapabilitiesForType(MimeTypes.VIDEO_DOLBY_VISION)
                }.getOrNull() ?: continue
                for (profileLevel in capabilities.profileLevels ?: continue) {
                    when (profileLevel.profile) {
                        CodecProfileLevel.DolbyVisionProfileDvheDtb -> nativeP7 = true
                        CodecProfileLevel.DolbyVisionProfileDvheStn -> p5 = true
                        CodecProfileLevel.DolbyVisionProfileDvheSt -> p8 = true
                    }
                }
            }
            DecoderProfiles(nativeP7, p5, p8, anyDv)
        }.getOrDefault(DecoderProfiles(false, false, false, false))
    }
}
