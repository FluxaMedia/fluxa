package com.fluxa.app.ui.catalog

import com.fluxa.app.data.local.LibraryUserCollectionFolder
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNull

class LibraryCollectionsJsonTest {
    @Test
    fun collectionArtworkUsesCorePresentationPolicy() {
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
