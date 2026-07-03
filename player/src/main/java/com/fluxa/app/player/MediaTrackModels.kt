package com.fluxa.app.player

import androidx.compose.runtime.Immutable
import androidx.media3.common.C

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
}

@Immutable
data class ExternalSubtitleTrack(
    val url: String,
    val language: String?,
    val label: String?
)

data class NativeAssTrack(
    val id: String,
    val label: String?,
    val language: String?,
    val assData: ByteArray,
    val fonts: List<NativeAssFont> = emptyList()
) {
    override fun equals(other: Any?): Boolean {
        if (this === other) return true
        if (other !is NativeAssTrack) return false
        return id == other.id && label == other.label && language == other.language &&
            assData.contentEquals(other.assData) && fonts == other.fonts
    }
    override fun hashCode(): Int {
        var result = id.hashCode()
        result = 31 * result + (label?.hashCode() ?: 0)
        result = 31 * result + (language?.hashCode() ?: 0)
        result = 31 * result + assData.contentHashCode()
        result = 31 * result + fonts.hashCode()
        return result
    }
}

data class NativeAssFont(
    val name: String,
    val data: ByteArray
) {
    override fun equals(other: Any?): Boolean {
        if (this === other) return true
        if (other !is NativeAssFont) return false
        return name == other.name && data.contentEquals(other.data)
    }
    override fun hashCode(): Int {
        var result = name.hashCode()
        result = 31 * result + data.contentHashCode()
        return result
    }
}
