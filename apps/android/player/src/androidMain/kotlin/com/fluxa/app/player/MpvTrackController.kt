package com.fluxa.app.player

import android.content.Context
import androidx.media3.common.C
import com.fluxa.app.shared.feature.player.AudioPassthroughPolicy
import com.fluxa.app.shared.feature.player.AudioTrackQualityPolicy
import com.fluxa.app.shared.feature.player.MediaTrack
import dev.jdtech.mpv.MPVLib
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import java.util.Locale

internal class MpvTrackController(
    private val context: Context,
    private val mpv: MPVLib,
    private val isInitialized: () -> Boolean,
    private val audioProcessingMode: String,
) {
    private var lastTrackListKey = ""
    private var preferredAudioLanguageCode = ""
    private var manualAudioSelection = false
    private var applyingAudioPreference = false

    private val _availableAudios = MutableStateFlow<List<MediaTrack>>(emptyList())
    val availableAudios: StateFlow<List<MediaTrack>> = _availableAudios

    private val _availableSubtitles = MutableStateFlow<List<MediaTrack>>(emptyList())
    val availableSubtitles: StateFlow<List<MediaTrack>> = _availableSubtitles

    private val _currentAudio = MutableStateFlow<MediaTrack?>(null)
    val currentAudio: StateFlow<MediaTrack?> = _currentAudio

    private val _currentSubtitle = MutableStateFlow<MediaTrack?>(null)
    val currentSubtitle: StateFlow<MediaTrack?> = _currentSubtitle

    fun resetForNewMedia() {
        manualAudioSelection = false
        lastTrackListKey = ""
    }

    fun updateTracksFromProperties() {
        val count = mpv.getPropertyInt("track-list/count") ?: 0
        val aid = mpv.getPropertyString("aid").orEmpty()
        val sid = mpv.getPropertyString("sid").orEmpty()
        val key = "$count:$aid:$sid"
        if (key == lastTrackListKey) return
        lastTrackListKey = key
        val audios = mutableListOf<MediaTrack>()
        val subtitles = mutableListOf<MediaTrack>()
        for (index in 0 until count) {
            val type = mpv.getPropertyString("track-list/$index/type").orEmpty()
            val mpvId = mpv.getPropertyInt("track-list/$index/id") ?: continue
            val selected = mpv.getPropertyBoolean("track-list/$index/selected") ?: false
            val language = mpv.getPropertyString("track-list/$index/lang")?.takeIf { it.isNotBlank() }
            val title = mpv.getPropertyString("track-list/$index/title")?.takeIf { it.isNotBlank() }
            val codec = mpv.getPropertyString("track-list/$index/codec")?.takeIf { it.isNotBlank() }
            val channelCount = mpv.getPropertyInt("track-list/$index/demux-channel-count")
                ?: mpv.getPropertyInt("audio-params/channel-count")
            val bitrate = mpv.getPropertyInt("track-list/$index/demux-bitrate")?.takeIf { it > 0 }?.toLong()
            val sampleRate = mpv.getPropertyInt("track-list/$index/demux-samplerate")?.takeIf { it > 0 }
            when (type) {
                "audio" -> audios.add(
                    MediaTrack(
                        id = "mpv_audio_$mpvId",
                        label = title ?: language ?: "Audio ${audios.size + 1}",
                        language = language,
                        type = C.TRACK_TYPE_AUDIO,
                        groupIndex = mpvId,
                        trackIndex = index,
                        isSelected = selected,
                        channelCount = channelCount,
                        sampleMimeType = codec,
                        bitrate = bitrate,
                        sampleRate = sampleRate,
                    )
                )
                "sub" -> subtitles.add(
                    MediaTrack(
                        id = "mpv_sub_$mpvId",
                        label = title ?: language ?: "Subtitle ${subtitles.size + 1}",
                        language = language,
                        type = C.TRACK_TYPE_TEXT,
                        groupIndex = mpvId,
                        trackIndex = index,
                        isSelected = selected,
                        sampleMimeType = codec
                    )
                )
            }
        }
        _availableAudios.value = audios
        _availableSubtitles.value = subtitles
        _currentAudio.value = audios.firstOrNull { it.isSelected }
        _currentSubtitle.value = subtitles.firstOrNull { it.isSelected }
        if (!manualAudioSelection && !applyingAudioPreference) selectBestAudioTrack()
    }

    fun selectAudio(track: MediaTrack) {
        if (track.type != C.TRACK_TYPE_AUDIO || !isInitialized()) return
        manualAudioSelection = true
        mpv.setPropertyString("aid", track.mpvTrackId())
        updateTracksFromProperties()
    }

    fun enableSubtitle(track: MediaTrack) {
        if (track.type != C.TRACK_TYPE_TEXT || !isInitialized()) return
        mpv.setPropertyString("sid", track.mpvTrackId())
        updateTracksFromProperties()
    }

    fun disableSubtitles() {
        if (!isInitialized()) return
        mpv.setPropertyString("sid", "no")
        _currentSubtitle.value = null
        updateTracksFromProperties()
    }

    fun applyPreferredAudioLanguage(languageCode: String) {
        if (languageCode.isBlank() || languageCode == "none" || !isInitialized()) return
        manualAudioSelection = false
        preferredAudioLanguageCode = languageCode
        mpv.setOptionString("alang", languageCode)
        selectBestAudioTrack()
    }

    private fun selectBestAudioTrack() {
        if (!isInitialized() || manualAudioSelection || applyingAudioPreference) return
        val tracks = _availableAudios.value
        val capabilities = AudioCapabilityResolver.resolve(
            context,
            AudioCapabilityResolver.mediaAudioAttributes()
        )
        val passthroughTrackIds = if (audioProcessingMode == "reference") {
            tracks.filter {
                AudioPassthroughPolicy.isMpvCandidate(it.sampleMimeType) &&
                    capabilities.supportsPassthroughMime(
                        sampleMimeType = it.sampleMimeType,
                        channelCount = it.channelCount ?: 2,
                        sampleRate = it.sampleRate ?: 48_000,
                    )
            }.mapTo(mutableSetOf()) { it.id }
        } else {
            emptySet()
        }
        val best = AudioTrackQualityPolicy.choose(
            tracks = tracks,
            preferredLanguage = preferredAudioLanguageCode,
            passthroughTrackIds = passthroughTrackIds,
            supportedSampleRates = capabilities.pcmSampleRates,
            maxPcmChannels = capabilities.maxPcmChannels,
        ) ?: return
        if (best.isSelected) return
        applyingAudioPreference = true
        try {
            mpv.setPropertyString("aid", best.mpvTrackId())
        } finally {
            applyingAudioPreference = false
        }
    }

    private fun MediaTrack.mpvTrackId(): String {
        return id.substringAfterLast('_').takeIf { it.toIntOrNull() != null } ?: groupIndex.toString()
    }
}
