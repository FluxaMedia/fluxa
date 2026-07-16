package com.fluxa.app.data.remote

import kotlinx.serialization.json.Json
import kotlin.test.Test
import kotlin.test.assertEquals

class SharedAccountModelsTest {
    private val json = Json { ignoreUnknownKeys = true }

    @Test
    fun decodesTraktTokenWireFields() {
        val token = json.decodeFromString<TraktTokenResponse>(
            """{"access_token":"access","token_type":"bearer","expires_in":3600,"refresh_token":"refresh","scope":"public","created_at":42}"""
        )
        assertEquals("access", token.accessToken)
        assertEquals("refresh", token.refreshToken)
        assertEquals(3600, token.expiresIn)
    }

    @Test
    fun decodesNestedSimklWireFields() {
        val payload = json.decodeFromString<SimklAllItemsResponse>(
            """{"shows":[{"title":"Show","last_watched_at":"2026-01-02","seasons":[{"number":1,"episodes":[{"number":2,"watched_at":"2026-01-01"}]}]}]}"""
        )
        val show = payload.shows.single()
        assertEquals("2026-01-02", show.lastWatchedAt)
        assertEquals("2026-01-01", show.seasons?.single()?.episodes?.single()?.watchedAt)
    }

    @Test
    fun encodesNuvioRefreshWireField() {
        assertEquals("""{"refresh_token":"refresh"}""", json.encodeToString(NuvioRefreshRequest("refresh")))
    }
}
