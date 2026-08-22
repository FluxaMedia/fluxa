package com.fluxa.app.shared.feature.player

/**
 * Chooses the best compatible track without changing an explicit user choice.
 * A route-confirmed passthrough track outranks a merely higher-ranked codec,
 * because preserving the encoded object-audio path is better than decoding it
 * to PCM and losing its metadata.
 */
object AudioTrackQualityPolicy {
    fun choose(
        tracks: List<MediaTrack>,
        preferredLanguage: String,
        passthroughTrackIds: Set<String> = emptySet(),
        supportedSampleRates: Set<Int> = emptySet(),
        maxPcmChannels: Int? = null,
    ): MediaTrack? {
        val candidates = tracks.filter { it.isSupported }
        if (candidates.isEmpty()) return null

        val preferred = preferredLanguage.trim().lowercase()
        val matching = candidates.filter { track ->
            val language = track.language?.lowercase().orEmpty()
            preferred.isNotBlank() && (
                language == preferred ||
                    language.substringBefore('-') == preferred.substringBefore('-')
                )
        }
        return (matching.ifEmpty { candidates }).maxWithOrNull(
            compareBy<MediaTrack> { if (it.id in passthroughTrackIds) 1 else 0 }
                .thenBy {
                    val rate = it.sampleRate
                    if (rate == null || supportedSampleRates.isEmpty() || rate in supportedSampleRates) 1 else 0
                }
                // A multichannel lossless source can be decoded and downmixed
                // to a stereo sink without a lossy re-encode. Do not let the
                // sink's channel count make us choose a lower-quality stereo
                // alternate track before comparing source quality.
                .thenBy { codecRank(it.sampleMimeType) }
                .thenBy {
                    val channels = it.channelCount ?: 2
                    if (maxPcmChannels == null || channels <= maxPcmChannels) 1 else 0
                }
                .thenBy { it.channelCount ?: 0 }
                .thenBy { it.bitrate ?: 0L }
                // A larger sample-rate number is not inherently higher
                // fidelity. It is already used above only as a route
                // compatibility gate; never prefer needless 96/192 kHz
                // resampling over a native film/TV rate as a quality tie-break.
                // Preserve the active track when all route/codec qualities
                // are equal. This prevents a route refresh from needlessly
                // switching between equivalent tracks and interrupting
                // playback.
                .thenBy { if (it.isSelected) 1 else 0 }
                // Track list ordering can change after a route/decoder
                // rebuild. Keep equal candidates deterministic so selection
                // does not oscillate between equivalent tracks.
                .thenBy { it.id }
        )
    }

    private fun codecRank(mime: String?): Int {
        val value = mime?.lowercase().orEmpty().replace("-", "").replace(".", "")
        return when {
            value.contains("truehd") || value.contains("dtshd") || value.contains("dtsx") ||
                value.contains("dtsuhd") || value.contains("dolbymat") ||
                value.contains("mpegh") || value.contains("dra") -> 600
            value.contains("flac") || value.contains("alac") || value.contains("pcm") ||
                value.contains("raw") || value.contains("wav") || value.contains("aiff") ||
                value.contains("ape") || value.contains("monkeysaudio") || value.contains("wavpack") ||
                value.contains("tta") || value.contains("tak") || value.contains("shn") ||
                value.contains("mlp") -> 500
            value.contains("eac3") || value.contains("ac3") || value.contains("ac4") || value.contains("dts") -> 400
            value.contains("opus") || value.contains("vorbis") -> 300
            value.contains("aac") || value.contains("mp4a") -> 200
            value.contains("mpeg") || value.contains("mp3") -> 100
            else -> 0
        }
    }
}
