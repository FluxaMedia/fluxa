package com.fluxa.app.ui.catalog

import com.fluxa.app.data.local.*
import com.fluxa.app.data.remote.*
import com.fluxa.app.data.repository.*
import com.fluxa.app.domain.discovery.*

import androidx.media3.common.C
import com.fluxa.app.shared.feature.player.MediaTrack
import org.junit.Assert.assertEquals
import org.junit.Assert.assertSame
import org.junit.Assert.assertTrue
import org.junit.Test

class TrackSelectionStateTest {

    @Test
    fun titleAudioMemoryOverridesProfilePreference() {
        val language = TrackSelectionState.resolvePreferredAudioLanguage(
            profile = profile(preferredAudioLanguage = "en"),
            meta = meta(lastAudioLanguage = "tr")
        )

        assertEquals("tr", language)
    }

    @Test
    fun englishAudioPreferenceUsesJapaneseOriginalForAnimeLikeContent() {
        val language = TrackSelectionState.resolvePreferredAudioLanguage(
            profile = profile(preferredAudioLanguage = "en"),
            meta = meta(originalLanguage = "ja")
        )

        assertEquals("ja", language)
    }

    @Test
    fun matchesTurkishSubtitleByLabelWhenLanguageMissing() {
        val track = subtitle(label = "Turkce altyazi", language = null)

        assertTrue(TrackSelectionState.matchesSubtitleLanguage(track, "tr-TR"))
    }

    @Test
    fun subtitleMemoryTakesPrecedenceOverProfilePreference() {
        val english = subtitle(id = "en", label = "English", language = "en")
        val turkish = subtitle(id = "tr", label = "Turkish", language = "tr")

        val selected = TrackSelectionState.findPreferredSubtitle(
            availableSubtitles = listOf(english, turkish),
            profile = profile(preferredSubtitleLanguage = "en"),
            meta = meta(lastSubtitleLanguage = "tr")
        )

        assertSame(turkish, selected)
    }

    @Test
    fun secondarySubtitleUsedWhenPrimaryMissing() {
        val turkish = subtitle(id = "tr", label = "Turkish", language = "tr")

        val selected = TrackSelectionState.findPreferredSubtitle(
            availableSubtitles = listOf(turkish),
            profile = profile(preferredSubtitleLanguage = "en", secondarySubtitleLanguage = "tr"),
            meta = meta()
        )

        assertSame(turkish, selected)
    }

    @Test
    fun noSubtitleSelectedWhenPreferredLanguagesAreMissing() {
        val turkish = subtitle(id = "tr", label = "Turkish", language = "tr")

        val selected = TrackSelectionState.findPreferredSubtitle(
            availableSubtitles = listOf(turkish),
            profile = profile(preferredSubtitleLanguage = "en"),
            meta = meta()
        )

        assertEquals(null, selected)
    }

    private fun profile(
        preferredAudioLanguage: String? = null,
        preferredSubtitleLanguage: String? = null,
        secondarySubtitleLanguage: String? = null
    ): UserProfile {
        return UserProfile(
            id = "profile",
            email = "profile@example.com",
            authKey = "auth",
            preferredAudioLanguage = preferredAudioLanguage,
            preferredSubtitleLanguage = preferredSubtitleLanguage,
            secondarySubtitleLanguage = secondarySubtitleLanguage
        )
    }

    private fun meta(
        lastAudioLanguage: String? = null,
        lastSubtitleLanguage: String? = null,
        originalLanguage: String? = null
    ): Meta {
        return Meta(
            id = "tt1",
            name = "Title",
            type = "movie",
            poster = null,
            lastAudioLanguage = lastAudioLanguage,
            lastSubtitleLanguage = lastSubtitleLanguage,
            originalLanguage = originalLanguage
        )
    }

    private fun subtitle(
        id: String = "sub",
        label: String,
        language: String?
    ): MediaTrack {
        return MediaTrack(
            id = id,
            label = label,
            language = language,
            type = C.TRACK_TYPE_TEXT,
            groupIndex = 0,
            trackIndex = 0,
            isSelected = false
        )
    }
}
