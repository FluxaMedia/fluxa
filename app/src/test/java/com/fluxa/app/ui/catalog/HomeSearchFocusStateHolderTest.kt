package com.fluxa.app.ui.catalog

import com.fluxa.app.data.remote.Meta
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test

class HomeSearchFocusStateHolderTest {

    @Test
    fun exposesMutableSearchAndFocusStateAsReadOnlyFlows() {
        val initialHistory = listOf(Meta(id = "tt1", name = "Initial", type = "movie", poster = null))
        val holder = HomeSearchFocusStateHolder(initialHistory)
        val focused = Meta(id = "tt2", name = "Focused", type = "movie", poster = null)

        holder.searchResultsValue = listOf(focused)
        holder.searchHistoryValue = initialHistory + focused
        holder.focusedMovieValue = focused
        holder.previewUrlValue = "https://example.com/preview.mp4"

        assertEquals(listOf(focused), holder.searchResults.value)
        assertEquals(2, holder.searchHistory.value.size)
        assertEquals(focused, holder.focusedMovie.value)
        assertEquals("https://example.com/preview.mp4", holder.previewUrl.value)
        assertNull(holder.focusedMovieTrailerUrl.value)
    }
}
