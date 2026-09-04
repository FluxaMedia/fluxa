package com.fluxa.app.player

import android.content.Context
import androidx.media3.common.C
import androidx.media3.common.Tracks
import androidx.media3.common.util.UnstableApi
import androidx.media3.common.TrackSelectionOverride
import androidx.media3.exoplayer.ExoPlayer
import com.fluxa.app.shared.feature.player.AudioTrackQualityPolicy
import com.fluxa.app.shared.feature.player.MediaTrack
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow

@UnstableApi
internal class MediaPlayerTrackController(
    private val context: Context,
    private val exoPlayer: ExoPlayer,
    private val externalAudio: () -> List<ExternalAudioTrack>
) {
    var preferredAudioLanguageCode: String = ""
    var manualAudioSelection = false
    var automaticAudioFallbackSelection = false
    var applyingAudioPreference = false

    private val _availableAudios = MutableStateFlow<List<MediaTrack>>(emptyList())
    val availableAudios: StateFlow<List<MediaTrack>> = _availableAudios

    private val _availableSubtitles = MutableStateFlow<List<MediaTrack>>(emptyList())
    val availableSubtitles: StateFlow<List<MediaTrack>> = _availableSubtitles

    private val _currentAudio = MutableStateFlow<MediaTrack?>(null)
    val currentAudio: StateFlow<MediaTrack?> = _currentAudio

    private val _currentSubtitle = MutableStateFlow<MediaTrack?>(null)
    val currentSubtitle: StateFlow<MediaTrack?> = _currentSubtitle

    fun updateTracks(tracks: Tracks) {
        val audios = mutableListOf<MediaTrack>()
        val subtitles = mutableListOf<MediaTrack>()
        tracks.groups.forEachIndexed { groupIndex, group ->
            if (group.type == C.TRACK_TYPE_AUDIO) {
                val external = externalAudioFor(group.mediaTrackGroup.id)
                for (i in 0 until group.length) {
                    val format = group.getTrackFormat(i)
                    audios.add(
                        MediaTrack(
                            id = "audio_$groupIndex-$i",
                            label = external?.label ?: format.label ?: format.language ?: "Ses ${audios.size + 1}",
                            language = external?.language ?: format.language,
                            sourceName = external?.sourceName,
                            type = C.TRACK_TYPE_AUDIO,
                            groupIndex = groupIndex,
                            trackIndex = i,
                            isSelected = group.isTrackSelected(i),
                            isSupported = group.isTrackSupported(i),
                            channelCount = format.channelCount,
                            sampleMimeType = format.sampleMimeType,
                            bitrate = format.bitrate.takeIf { it > 0 }?.toLong(),
                            sampleRate = format.sampleRate.takeIf { it > 0 },
                        )
                    )
                }
            } else if (group.type == C.TRACK_TYPE_TEXT) {
                for (i in 0 until group.length) {
                    val format = group.getTrackFormat(i)
                    LibassDebugLog.d(
                        "track discovered group=$groupIndex track=$i selected=${group.isTrackSelected(i)} supported=${group.isTrackSupported(i)} ${LibassDebugLog.formatSummary(format)}"
                    )
                    subtitles.add(
                        MediaTrack(
                            id = "sub_$groupIndex-$i",
                            label = format.label ?: format.language ?: "Subtitle ${subtitles.size + 1}",
                            language = format.language,
                            type = C.TRACK_TYPE_TEXT,
                            groupIndex = groupIndex,
                            trackIndex = i,
                            isSelected = group.isTrackSelected(i),
                            isSupported = group.isTrackSupported(i),
                            sampleMimeType = format.sampleMimeType,
                            containerTrackId = format.id
                        )
                    )
                }
            }
        }
        _availableAudios.value = audios
        _availableSubtitles.value = subtitles
        _currentAudio.value = audios.find { it.isSelected }
        val selectedSubtitle = subtitles.find { it.isSelected }
        _currentSubtitle.value = selectedSubtitle
        LibassDebugLog.d(
            "tracks updated subtitles=${subtitles.size} selected=${selectedSubtitle?.let { "${it.id} mime=${it.sampleMimeType} label=${it.label} lang=${it.language}" } ?: "<none>"}"
        )
        val relay = MediaPlayerControllerFactory.getLibassRelay(exoPlayer)
        if (relay != null) {
            val selectedFormat = selectedSubtitle?.let {
                tracks.groups.getOrNull(it.groupIndex)?.getTrackFormat(it.trackIndex)
            }
            relay.setSelectedTrackId(selectedFormat?.id?.toIntOrNull())
        }
    }

    fun selectTrack(track: MediaTrack) {
        LibassDebugLog.d("select track id=${track.id} type=${track.type} group=${track.groupIndex} track=${track.trackIndex} mime=${track.sampleMimeType} label=${track.label} lang=${track.language}")
        exoPlayer.trackSelectionParameters = exoPlayer.trackSelectionParameters
            .buildUpon()
            .setOverrideForType(
                TrackSelectionOverride(exoPlayer.currentTracks.groups[track.groupIndex].mediaTrackGroup, track.trackIndex)
            )
            .build()
        if (track.type == C.TRACK_TYPE_AUDIO) {
            manualAudioSelection = true
            automaticAudioFallbackSelection = false
            _currentAudio.value = track
        } else {
            _currentSubtitle.value = track
        }
    }

    fun disableSubtitles() {
        LibassDebugLog.d("disable subtitles")
        exoPlayer.trackSelectionParameters = exoPlayer.trackSelectionParameters
            .buildUpon()
            .clearOverridesOfType(C.TRACK_TYPE_TEXT)
            .build()
        _currentSubtitle.value = null
    }

    fun applyPreferredAudioLanguage(languageCode: String) {
        if (languageCode.isBlank() || languageCode == "none") return
        manualAudioSelection = false
        automaticAudioFallbackSelection = false
        preferredAudioLanguageCode = languageCode
        exoPlayer.trackSelectionParameters = exoPlayer.trackSelectionParameters
            .buildUpon()
            .setPreferredAudioLanguage(languageCode)
            .build()
        selectBestAudioTrack()
    }

    fun selectBestAudioTrack() {
        if (manualAudioSelection || applyingAudioPreference) return
        val audioTracks = _availableAudios.value.filter { it.isSupported }
        if (audioTracks.isEmpty()) return
        val capabilities = AudioCapabilityResolver.resolve(
            context,
            AudioCapabilityResolver.mediaAudioAttributes()
        )
        val best = AudioTrackQualityPolicy.choose(
            tracks = audioTracks,
            preferredLanguage = preferredAudioLanguageCode,
            passthroughTrackIds = passthroughTrackIds(audioTracks, capabilities),
            supportedSampleRates = capabilities.pcmSampleRates,
            maxPcmChannels = capabilities.maxPcmChannels,
        ) ?: return
        if (best.isSelected) return
        val group = exoPlayer.currentTracks.groups.getOrNull(best.groupIndex) ?: return
        applyingAudioPreference = true
        try {
            exoPlayer.trackSelectionParameters = exoPlayer.trackSelectionParameters
                .buildUpon()
                .setOverrideForType(TrackSelectionOverride(group.mediaTrackGroup, best.trackIndex))
                .build()
        } finally {
            applyingAudioPreference = false
        }
    }

    fun passthroughTrackIds(
        tracks: List<MediaTrack>,
        capabilities: AudioOutputCapabilities,
    ): Set<String> {
        if (MediaPlayerControllerFactory.audioProcessingMode(exoPlayer) != "reference") return emptySet()
        val attributes = AudioCapabilityResolver.mediaAudioAttributes()
        return tracks.mapNotNull { track ->
            val format = exoPlayer.currentTracks.groups
                .getOrNull(track.groupIndex)
                ?.takeIf { it.type == C.TRACK_TYPE_AUDIO }
                ?.getTrackFormat(track.trackIndex)
                ?: return@mapNotNull null
            track.id.takeIf { capabilities.supportsPassthrough(format, attributes) }
        }.toSet()
    }

    private fun externalAudioFor(trackGroupId: String?): ExternalAudioTrack? {
        val childIndex = trackGroupId?.substringBefore(':', "")?.toIntOrNull() ?: return null
        return externalAudio().getOrNull(childIndex - 1)
    }
}
