package com.fluxa.app.ui.catalog

import com.fluxa.app.data.remote.Meta
import com.fluxa.app.data.remote.MetaRating
import com.fluxa.app.domain.discovery.DiscoverResultFilters
import com.fluxa.app.domain.discovery.DiscoverRequest
import org.junit.Assert.assertEquals
import org.junit.Test

class DiscoverResultFiltersTest {
    @Test
    fun filtersByReleaseYear() {
        val filtered = DiscoverResultFilters.apply(
            items = listOf(
                meta("tt1", released = "2024-01-02"),
                meta("tt2", releaseInfo = "2023")
            ),
            request = request(year = "2024")
        )

        assertEquals(listOf("tt1"), filtered.map { it.id })
    }

    @Test
    fun filtersByNormalizedRating() {
        val filtered = DiscoverResultFilters.apply(
            items = listOf(
                meta("tt1", imdbRating = "8.1"),
                meta("tt2", ratings = listOf(MetaRating("Rotten Tomatoes", "72%"))),
                meta("tt3", imdbRating = "5.2")
            ),
            request = request(rating = 7.0f)
        )

        assertEquals(listOf("tt1", "tt2"), filtered.map { it.id })
    }

    @Test
    fun filtersByRegionLanguage() {
        val filtered = DiscoverResultFilters.apply(
            items = listOf(
                meta("tt1", originalLanguage = "tr"),
                meta("tt2", originalLanguage = "en")
            ),
            request = request(region = "tr")
        )

        assertEquals(listOf("tt1"), filtered.map { it.id })
    }

    private fun request(
        year: String? = null,
        rating: Float? = null,
        region: String? = null
    ) = DiscoverRequest(
        type = "all",
        catalogKey = null,
        genre = null,
        year = year,
        rating = rating,
        provider = null,
        region = region
    )

    private fun meta(
        id: String,
        released: String? = null,
        releaseInfo: String? = null,
        imdbRating: String? = null,
        ratings: List<MetaRating>? = null,
        originalLanguage: String? = null
    ) = Meta(
        id = id,
        name = id,
        type = "movie",
        poster = null,
        released = released,
        releaseInfo = releaseInfo,
        imdbRating = imdbRating,
        ratings = ratings,
        originalLanguage = originalLanguage
    )
}
