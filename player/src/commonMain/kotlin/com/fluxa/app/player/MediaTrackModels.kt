package com.fluxa.app.player

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
    val sampleMimeType: String? = null,
    val containerTrackId: String? = null
) {
    val audioChannelLabel: String
        get() {
            if (type != AUDIO_TRACK_TYPE) return ""
            return when (channelCount) {
                1 -> "Mono"
                2 -> "Stereo"
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

    val subtitleFormatLabel: String
        get() = when {
            sampleMimeType == null -> ""
            sampleMimeType.contains("subrip") || sampleMimeType.contains("srt") -> "SRT"
            sampleMimeType.contains("ssa") || sampleMimeType == "ass" || sampleMimeType.contains("x-ass") -> "ASS"
            sampleMimeType.contains("vtt") -> "VTT"
            sampleMimeType.contains("ttml") || sampleMimeType.contains("mov_text") -> "TTML"
            sampleMimeType.contains("pgs") -> "PGS"
            sampleMimeType.contains("dvbsubs") || sampleMimeType.contains("dvb_subtitle") -> "DVB"
            sampleMimeType.contains("dvd_subtitle") || sampleMimeType.contains("vobsub") -> "VOBSUB"
            else -> ""
        }

    private companion object {
        const val AUDIO_TRACK_TYPE = 1
    }
}
