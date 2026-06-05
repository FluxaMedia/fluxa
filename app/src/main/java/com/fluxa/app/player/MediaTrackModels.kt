package com.fluxa.app.player

import androidx.compose.runtime.Immutable
import androidx.media3.common.C
import java.util.Locale

enum class VideoFormatBadge { Hdr10, Hdr10Plus, Hlg, DolbyVision }

enum class AudioCodecBadge {
    DolbyAtmos,
    DolbyDigitalPlus,
    DolbyDigital,
    DolbyTrueHD,
    DtsX,
    DtsHd,
    Dts,
}

@Immutable
data class MediaTrack(
    val id: String,
    val label: String,
    val language: String? = null,
    val type: Int,
    val groupIndex: Int,
    val trackIndex: Int,
    val isSelected: Boolean,
    val isSupported: Boolean = true,
    val showCodecBadge: Boolean = true,
    val channelCount: Int? = null,
    val sampleMimeType: String? = null
) {
    val audioChannelLabel: String
        get() {
            if (type != C.TRACK_TYPE_AUDIO) return ""
            return when (channelCount) {
                1 -> "1.0"
                2 -> "2.0"
                3 -> "2.1"
                4 -> "4.0"
                5 -> "5.0"
                6 -> "5.1"
                7 -> "6.1"
                8 -> "7.1"
                null -> ""
                else -> "$channelCount ch"
            }
        }

    val audioCodecBadge: AudioCodecBadge?
        get() {
            if (!showCodecBadge) return null
            if (type != C.TRACK_TYPE_AUDIO) return null
            val mime = sampleMimeType?.lowercase(Locale.ROOT) ?: return null
            return when {
                mime.contains("eac3-joc") || mime.contains("eac3_joc") -> AudioCodecBadge.DolbyAtmos
                mime.contains("eac3") -> AudioCodecBadge.DolbyDigitalPlus
                // ac3 must come after eac3 checks
                mime.contains("ac3") -> AudioCodecBadge.DolbyDigital
                mime.contains("true-hd") || mime.contains("truehd") -> AudioCodecBadge.DolbyTrueHD
                // dts.uhd / dts-uhd before dts.hd / dts-hd before plain dts
                mime.contains("dts.uhd") || mime.contains("dts-uhd") || mime.contains("dts_uhd") -> AudioCodecBadge.DtsX
                mime.contains("dts.hd") || mime.contains("dts-hd") || mime.contains("dts_hd") -> AudioCodecBadge.DtsHd
                mime.contains("dts") -> AudioCodecBadge.Dts
                else -> null
            }
        }
}

@Immutable
data class ExternalSubtitleTrack(
    val url: String,
    val language: String?,
    val label: String?
)
