package com.fluxa.app.ui.catalog

import com.fluxa.app.data.local.*
import com.fluxa.app.data.remote.*
import com.fluxa.app.data.repository.*
import com.fluxa.app.domain.discovery.*
import com.fluxa.app.core.StremioId

import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Test

class StremioIdTest {

    @Test
    fun keepsTmdbPrefixWhenRemovingEpisodeSuffix() {
        assertEquals("tmdb:12345", StremioId.baseContentId("tmdb:12345:1:2"))
        assertEquals("tmdb:12345", StremioId.normalizeSeriesLookupId("tmdb:12345:1:2"))
    }

    @Test
    fun keepsRequestedTmdbEpisodeBeforeCanonicalFallback() {
        val ids = StremioId.streamRequestIds(
            type = "series",
            id = "tmdb:12345:1:2",
            detailId = "tmdb:12345",
            currentSeriesLookupId = "tmdb:12345",
            canonicalBaseId = "tt9999999"
        )

        assertEquals("tmdb:12345:1:2", ids.first())
        assertEquals("tt9999999:1:2", ids.last())
        assertFalse(ids.contains("tmdb:1:2"))
    }

    @Test
    fun keepsCustomAddonEpisodeIdBeforeCanonicalFallback() {
        val ids = StremioId.streamRequestIds(
            type = "series",
            id = "kitsu:777:1:2",
            detailId = "tt9999999",
            currentSeriesLookupId = "kitsu:777",
            canonicalBaseId = "tt9999999"
        )

        assertEquals("kitsu:777:1:2", ids.first())
        assertEquals("tt9999999:1:2", ids[1])
    }

    @Test
    fun keepsCanonicalMovieIdBeforeTmdbFallback() {
        val ids = StremioId.streamRequestIds(
            type = "movie",
            id = "tmdb:12345",
            detailId = null,
            currentSeriesLookupId = null,
            canonicalBaseId = "tt9999999"
        )

        assertEquals(listOf("tt9999999", "tmdb:12345"), ids)
    }
}
