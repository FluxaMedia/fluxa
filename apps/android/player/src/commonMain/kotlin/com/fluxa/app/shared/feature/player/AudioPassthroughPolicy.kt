package com.fluxa.app.shared.feature.player

/**
 * Backend-specific encoded candidates for the Android MPV audio output.
 *
 * Android's route resolver remains authoritative for actual device support;
 * this policy only prevents MPV from advertising formats its `audio-spdif`
 * bridge cannot represent, such as Dolby MAT, MPEG-H and DTS-UHD.
 */
object AudioPassthroughPolicy {
    fun isMpvCandidate(sampleMimeType: String?): Boolean {
        val mime = sampleMimeType
            ?.lowercase()
            ?.replace('.', '-')
            ?.replace('_', '-')
            ?: return false
        return when {
            mime.contains("truehd") || mime.contains("true-hd") -> true
            mime.contains("eac3") || mime.contains("ec-3") -> true
            mime.contains("ac3") || mime.contains("ac-3") -> true
            mime.contains("ac4") -> true
            mime.contains("dts-uhd") -> false
            mime.contains("dts-hd") || mime.contains("dtshd") -> true
            mime.contains("dts") -> true
            else -> false
        }
    }
}
