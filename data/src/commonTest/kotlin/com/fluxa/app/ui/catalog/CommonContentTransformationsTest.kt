package com.fluxa.app.ui.catalog

import com.fluxa.app.data.local.LibraryUserCollectionFolder
import com.fluxa.app.data.remote.Meta
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertNull
import kotlin.test.assertTrue

class CommonContentTransformationsTest {
    @Test
    fun continueWatchingLabelNormalizesEpisodePrefix() {
        val meta = Meta(
            id = "tt123",
            type = "series",
            name = "Example",
            lastVideoId = "tt123:2:4",
            lastEpisodeName = "S2 E4 - The Return"
        )

        assertEquals("S2, E4: The Return", continueWatchingEpisodeLabel(meta))
    }

    @Test
    fun zeroProgressSeriesItemIsUpNext() {
        val meta = Meta(
            id = "tt123",
            type = "series",
            name = "Example",
            lastVideoId = "tt123:1:2",
            timeOffset = 0L,
            duration = 0L
        )

        assertTrue(meta.isUpNextContinueItem())
        assertFalse(meta.copy(type = "movie").isUpNextContinueItem())
    }

    @Test
    fun collectionArtworkNormalizesGithubAndProtocolRelativeUrls() {
        val github = LibraryUserCollectionFolder(
            id = "one",
            title = "One",
            coverImageUrl = "https://github.com/org/repo/blob/main/posters/a b.jpg"
        )
        val protocolRelative = github.copy(coverImageUrl = "//images.example/poster.jpg")

        assertEquals(
            "https://raw.githubusercontent.com/org/repo/main/posters/a%20b.jpg",
            github.effectiveImageUrl()
        )
        assertEquals("https://images.example/poster.jpg", protocolRelative.effectiveImageUrl())
        assertNull(github.copy(coverImageUrl = " ", imageUrl = null).effectiveImageUrl())
    }
}
