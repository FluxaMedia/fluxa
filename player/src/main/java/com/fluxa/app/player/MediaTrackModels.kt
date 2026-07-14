package com.fluxa.app.player

import androidx.media3.common.C

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
