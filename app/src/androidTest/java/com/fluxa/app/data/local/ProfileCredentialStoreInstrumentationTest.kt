package com.fluxa.app.data.local

import androidx.test.core.app.ApplicationProvider
import androidx.test.ext.junit.runners.AndroidJUnit4
import com.google.gson.Gson
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
class ProfileCredentialStoreInstrumentationTest {
    @Test
    fun redactsNestedAndTopLevelCredentials() {
        val store = ProfileCredentialStore(ApplicationProvider.getApplicationContext(), Gson())
        val profile = UserProfile(
            id = "profile",
            email = "profile@example.com",
            authKey = "stremio-token",
            traktAccessToken = "trakt-token",
            traktRefreshToken = "trakt-refresh",
            simklAccessToken = "simkl-token",
            externalAccounts = ExternalAccounts(
                traktAccessToken = "nested-trakt-token",
                traktRefreshToken = "nested-trakt-refresh",
                simklAccessToken = "nested-simkl-token"
            )
        )

        val redacted = store.redact(profile)

        assertEquals("", redacted.authKey)
        assertNull(redacted.traktAccessToken)
        assertNull(redacted.traktRefreshToken)
        assertNull(redacted.simklAccessToken)
        assertNull(redacted.externalAccounts?.traktAccessToken)
        assertNull(redacted.externalAccounts?.traktRefreshToken)
        assertNull(redacted.externalAccounts?.simklAccessToken)
    }
}
