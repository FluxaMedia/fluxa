package com.fluxa.app.ui.catalog

import com.fluxa.app.data.remote.Meta
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertTrue

class HomePolicyTest {
    @Test
    fun filtersCatalogItemsWithoutDroppingContinueWatchingFallback() {
        val movie = Meta(id = "movie", type = "movie", name = "Movie")
        val series = Meta(id = "series", type = "series", name = "Series")
        val categories = listOf(
            HomeCategory("Continue", listOf(movie), CONTINUE_WATCHING_CATEGORY_ID, "mixed"),
            HomeCategory("Catalog", listOf(movie, series), "catalog", "mixed")
        )

        val result = orderHomeCategories(categories, "series")

        assertEquals(listOf(movie), result[0].items)
        assertEquals(listOf(series), result[1].items)
    }

    @Test
    fun rejectsCalendarPlaceholderArtwork() {
        assertFalse(isUsableCalendarArtwork("https://example.com/default-poster.png"))
        assertFalse(isUsableCalendarArtwork("null"))
        assertTrue(isUsableCalendarArtwork("https://example.com/poster.jpg"))
    }
}
