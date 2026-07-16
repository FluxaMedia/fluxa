package com.fluxa.app.player

import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.data.remote.Meta
import kotlin.test.Test
import kotlin.test.assertEquals

class PlayerTrackPreferencesTest {
    @Test
    fun resolvesDeviceAndOriginalLanguagePreferences() {
        val meta = Meta(id = "tt1", name = "Movie", type = "movie", originalLanguage = "es")
        assertEquals("es", resolveAudioLanguagePreference(profile("original"), meta, "en"))
        assertEquals("en", resolveAudioLanguagePreference(profile("device_language"), meta, "en"))
    }

    private fun profile(preference: String) = UserProfile(
        id = "profile",
        email = "user@example.com",
        authKey = "auth",
        preferredAudioLanguage = preference
    )
}
