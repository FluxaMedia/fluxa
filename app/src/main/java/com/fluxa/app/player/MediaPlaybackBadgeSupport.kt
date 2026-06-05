package com.fluxa.app.player

import android.content.Context
import android.hardware.display.DisplayManager
import android.media.AudioFormat
import android.media.AudioManager
import android.view.Display
import androidx.media3.common.MimeTypes
import java.util.Locale

internal object MediaPlaybackBadgeSupport {
    fun supportsEncodedAudioBadge(context: Context, sampleMimeType: String?): Boolean {
        val lower = sampleMimeType?.lowercase(Locale.ROOT) ?: return false
        val encoding = when {
            lower.contains("eac3-joc") || lower.contains("eac3_joc") -> AudioFormat.ENCODING_E_AC3_JOC
            lower.contains("eac3") || lower.contains("ec-3") -> AudioFormat.ENCODING_E_AC3
            lower.contains("ac3") -> AudioFormat.ENCODING_AC3
            lower.contains("true-hd") || lower.contains("truehd") -> AudioFormat.ENCODING_DOLBY_TRUEHD
            lower.contains("dts.hd") || lower.contains("dts-hd") || lower.contains("dts_hd") -> AudioFormat.ENCODING_DTS_HD
            lower.contains("dts") -> AudioFormat.ENCODING_DTS
            else -> return false
        }
        val audioManager = context.getSystemService(Context.AUDIO_SERVICE) as? AudioManager ?: return false
        return audioManager.getDevices(AudioManager.GET_DEVICES_OUTPUTS).any { device ->
            device.encodings.any { it == encoding }
        }
    }

    fun supportsHdrBadge(context: Context, badge: VideoFormatBadge): Boolean {
        val displayManager = context.getSystemService(Context.DISPLAY_SERVICE) as? DisplayManager
        val display = displayManager?.getDisplay(Display.DEFAULT_DISPLAY) ?: return false
        @Suppress("DEPRECATION")
        val hdrTypes = display.hdrCapabilities.supportedHdrTypes
        return when (badge) {
            VideoFormatBadge.DolbyVision -> hdrTypes.any { it == Display.HdrCapabilities.HDR_TYPE_DOLBY_VISION }
            VideoFormatBadge.Hdr10 -> hdrTypes.any { it == Display.HdrCapabilities.HDR_TYPE_HDR10 }
            VideoFormatBadge.Hdr10Plus -> hdrTypes.any { it == Display.HdrCapabilities.HDR_TYPE_HDR10_PLUS }
            VideoFormatBadge.Hlg -> hdrTypes.any { it == Display.HdrCapabilities.HDR_TYPE_HLG }
        }
    }

    fun sampleMimeTypeForMpvCodec(codec: String?): String? {
        val lower = codec?.lowercase(Locale.ROOT) ?: return null
        return when {
            lower.contains("eac3") || lower.contains("e-ac-3") || lower.contains("ec-3") -> MimeTypes.AUDIO_E_AC3
            lower.contains("ac3") || lower.contains("a52") -> MimeTypes.AUDIO_AC3
            lower.contains("truehd") || lower.contains("true-hd") -> MimeTypes.AUDIO_TRUEHD
            lower.contains("dts") -> "audio/dts"
            else -> codec
        }
    }
}
