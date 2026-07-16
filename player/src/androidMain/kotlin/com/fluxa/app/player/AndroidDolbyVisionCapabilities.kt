package com.fluxa.app.player

import android.content.Context
import android.hardware.display.DisplayManager
import android.media.MediaCodecInfo.CodecProfileLevel
import android.media.MediaCodecList
import android.os.Build
import android.view.Display
import androidx.media3.common.MimeTypes

object AndroidDolbyVisionCapabilities {
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
