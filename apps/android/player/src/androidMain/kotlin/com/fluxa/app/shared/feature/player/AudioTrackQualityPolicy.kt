package com.fluxa.app.shared.feature.player

import com.fluxa.app.core.rust.FluxaCoreNative
import com.google.gson.Gson

private val audioTrackGson = Gson()

object AudioTrackQualityPolicy {
    fun choose(
        tracks: List<MediaTrack>,
        preferredLanguage: String,
        passthroughTrackIds: Set<String> = emptySet(),
        supportedSampleRates: Set<Int> = emptySet(),
        maxPcmChannels: Int? = null,
    ): MediaTrack? {
        val selectedId = FluxaCoreNative.selectAudioTrack(
            tracksJson = audioTrackGson.toJson(tracks),
            preferredLanguage = preferredLanguage,
            passthroughTrackIds = passthroughTrackIds,
            supportedSampleRates = supportedSampleRates,
            maxPcmChannels = maxPcmChannels,
        ) ?: return null
        return tracks.firstOrNull { it.id == selectedId }
    }
}
