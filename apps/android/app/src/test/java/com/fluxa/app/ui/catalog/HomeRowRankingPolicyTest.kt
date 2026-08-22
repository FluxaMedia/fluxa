package com.fluxa.app.ui.catalog

import com.fluxa.app.data.local.*
import com.fluxa.app.data.remote.*
import com.fluxa.app.data.repository.*
import com.fluxa.app.domain.discovery.*

import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Test

class HomeRowRankingPolicyTest {

    @Test
    fun keepsPinnedRowsBeforeRankedCatalogRows() {
        val optimized = HomeRowRankingPolicy.optimize(
            categories = listOf(
                category("popular", "Popular", ids = (1..6).map { "p$it" }),
                category("continue_watching", "Continue Watching", ids = listOf("cw1")),
                category("trending", "Trending Now", ids = (1..6).map { "t$it" })
            ),
            lang = "en",
            preferredOrderLabels = listOf("Trending Now", "Popular"),
            preferredGenres = emptyMap(),
            preferredTypes = emptyMap()
        )

        assertEquals("continue_watching", optimized.first().id)
        assertEquals("trending", optimized[1].id)
    }

    @Test
    fun removesDuplicateItemsInsideCategory() {
        val curated = HomeRowRankingPolicy.curateItems(
            category = HomeCategory(
                id = "action",
                name = "Action",
                semanticName = "Action",
                type = "movie",
                items = listOf(
                    meta("tt1", "Action One", genres = listOf("Action")),
                    meta("tt1", "Action One Duplicate", genres = listOf("Action")),
                    meta("tt2", "Drama Two", genres = listOf("Drama"))
                )
            ),
            lang = "en"
        )

        assertEquals(listOf("tt1", "tt2"), curated.map { it.id })
    }

    @Test
    fun filtersAdultGenreFromHomeItems() {
        val curated = HomeRowRankingPolicy.curateItems(
            category = HomeCategory(
                id = "popular",
                name = "Popular",
                type = "movie",
                items = listOf(
                    meta("tt1", "Safe", genres = listOf("Drama")),
                    meta("tt2", "Adult", genres = listOf("Adult"))
                )
            ),
            lang = "en"
        )

        assertEquals(listOf("tt1"), curated.map { it.id })
    }

    @Test
    fun dropsHighlyOverlappingNonGenreRowsAfterInitialRows() {
        val baseIds = (1..6).map { "same$it" }
        val categories = (1..10).map { index ->
            category("row$index", "Editorial $index", ids = baseIds)
        }

        val optimized = HomeRowRankingPolicy.optimize(
            categories = categories,
            lang = "en",
            preferredOrderLabels = emptyList(),
            preferredGenres = emptyMap(),
            preferredTypes = emptyMap()
        )

        assertFalse(optimized.any { it.id == "row10" })
    }

    private fun category(id: String, name: String, ids: List<String>): HomeCategory {
        return HomeCategory(
            id = id,
            name = name,
            semanticName = name,
            type = "movie",
            items = ids.map { meta(it, "$name $it") }
        )
    }

    private fun meta(
        id: String,
        name: String,
        type: String = "movie",
        genres: List<String>? = null
    ): Meta {
        return Meta(
            id = id,
            name = name,
            type = type,
            poster = null,
            genres = genres
        )
    }
}
