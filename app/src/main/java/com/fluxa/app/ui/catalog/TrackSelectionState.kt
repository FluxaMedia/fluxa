package com.fluxa.app.ui.catalog

import com.fluxa.app.data.local.*
import com.fluxa.app.data.remote.*
import com.fluxa.app.data.repository.*
import com.fluxa.app.domain.discovery.*

import com.fluxa.app.core.rust.FluxaCoreNative
import com.fluxa.app.core.rust.models.SubtitleTrackRef
import com.fluxa.app.player.MediaTrack
import java.util.Locale

object TrackSelectionState {
    private fun resolveAudioLanguagePref(profile: UserProfile?, meta: Meta): String? {
        val isAnime = meta.genres?.any { it.lowercase().contains("anime") } == true
        if (isAnime && profile?.safeAnimePreferJapaneseAudio == true) return "ja"
        return when (val pref = profile?.preferredAudioLanguage) {
            "original" -> meta.originalLanguage?.takeIf { it.isNotBlank() }
            "device_language" -> Locale.getDefault().language.takeIf { it.isNotBlank() }
            else -> pref
        }
    }

    fun resolvePreferredAudioLanguage(profile: UserProfile?, meta: Meta): String {
        return FluxaCoreNative.playerTrackState(
            availableSubtitles = emptyList(),
            lastAudioLanguage = meta.lastAudioLanguage,
            preferredAudioLanguage = resolveAudioLanguagePref(profile, meta),
            originalLanguage = meta.originalLanguage,
            lastSubtitleLanguage = meta.lastSubtitleLanguage,
            preferredSubtitleLanguage = profile?.preferredSubtitleLanguage,
            secondarySubtitleLanguage = profile?.secondarySubtitleLanguage
        ).preferredAudioLanguage
    }

    fun matchesSubtitleLanguage(track: MediaTrack, preferredLanguage: String): Boolean {
        return FluxaCoreNative.subtitleLanguageMatches(
            label = track.label,
            language = track.language,
            preferredLanguage = preferredLanguage
        )
    }

    fun findPreferredSubtitle(
        availableSubtitles: List<MediaTrack>,
        profile: UserProfile?,
        meta: Meta
    ): MediaTrack? {
        val state = FluxaCoreNative.playerTrackState(
            availableSubtitles = availableSubtitles.map { SubtitleTrackRef(it.id, it.label, it.language) },
            lastAudioLanguage = meta.lastAudioLanguage,
            preferredAudioLanguage = resolveAudioLanguagePref(profile, meta),
            originalLanguage = meta.originalLanguage,
            lastSubtitleLanguage = meta.lastSubtitleLanguage,
            preferredSubtitleLanguage = profile?.preferredSubtitleLanguage,
            secondarySubtitleLanguage = profile?.secondarySubtitleLanguage
        )
        return state.preferredSubtitleId
            ?.let { preferredId -> availableSubtitles.firstOrNull { it.id == preferredId } }
            ?: availableSubtitles.getOrNull(state.preferredSubtitleIndex)
    }
}
