package com.fluxa.app.ui.catalog

import com.fluxa.app.data.local.*
import com.fluxa.app.data.remote.*
import com.fluxa.app.data.repository.*
import com.fluxa.app.domain.discovery.*

import org.junit.Test
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertNull

class TraktIntegrationTest {
    @Test
    fun contentIdFromPrefixesTmdbIds() {
        assertEquals("tt1234567", TraktIntegration.contentIdFrom(TraktIds(imdb = "tt1234567", tmdb = 42)))
        assertEquals("tmdb:42", TraktIntegration.contentIdFrom(TraktIds(tmdb = 42)))
        assertEquals("tvdb:12345", TraktIntegration.contentIdFrom(TraktIds(tvdb = 12345)))
        assertEquals("trakt:show-slug", TraktIntegration.contentIdFrom(TraktIds(slug = "show-slug")))
    }

    @Test
    fun tokenExpiresAtKeepsFiveMinuteRefreshBuffer() {
        assertEquals(1_700_003_300_000L, TraktIntegration.tokenExpiresAt(1_700_000_000L, 3_600L))
        assertEquals(1_700_000_000_000L, TraktIntegration.tokenExpiresAt(1_700_000_000L, 120L))
    }

    @Test
    fun idsFromContentIdSupportsStremioEpisodeIds() {
        assertEquals(TraktIds(imdb = "tt1234567"), TraktIntegration.idsFromContentId("tt1234567:1:2"))
        assertEquals(TraktIds(tmdb = 42), TraktIntegration.idsFromContentId("tmdb:42:1:2"))
        assertEquals(TraktIds(tvdb = 12345), TraktIntegration.idsFromContentId("tvdb:12345:1:2"))
        assertEquals(TraktIds(tmdb = 42), TraktIntegration.idsFromContentId("42:1:2"))
    }

    @Test
    fun historyRequestDoesNotSendSeriesAsMovieWhenEpisodeIsMissing() {
        val request = TraktIntegration.buildHistoryRequest(
            meta = Meta(id = "tt1234567", name = "Show", type = "series", poster = null),
            episodes = emptyList()
        )

        assertNull(request)
    }

    @Test
    fun historyRequestBuildsShowSeasonsFromEpisodeIds() {
        val request = TraktIntegration.buildHistoryRequest(
            meta = Meta(id = "tt1234567", name = "Show", type = "series", poster = null),
            episodes = listOf(
                Video(id = "tt1234567:1:2", name = null, season = null, number = null, released = null, thumbnail = null)
            )
        )

        assertNotNull(request)
        val nonNullRequest = request!!
        assertEquals(1, nonNullRequest.shows?.single()?.seasons?.single()?.number)
        assertEquals(2, nonNullRequest.shows?.single()?.seasons?.single()?.episodes?.single()?.number)
        assertNull(nonNullRequest.movies)
    }
}
