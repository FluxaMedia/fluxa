package com.fluxa.app.ui.catalog

import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

class LibraryCollectionsJsonTest {

    @Test
    fun importKeepsFolderCoverHeroBackdropAndFocusGifSeparate() {
        val collections = importLibraryCollectionsJson(
            """
            {
              "id": "featured",
              "title": "Featured",
              "folders": [
                {
                  "id": "action",
                  "title": "Action",
                  "coverImageUrl": "https://cdn.example.com/action-cover.jpg",
                  "heroBackdropUrl": "https://cdn.example.com/action-hero.jpg",
                  "focusGifUrl": "https://cdn.example.com/action-focus.gif",
                  "catalogSources": [
                    {
                      "addonId": "cinemeta",
                      "catalogId": "top",
                      "type": "movie"
                    }
                  ]
                }
              ]
            }
            """.trimIndent()
        )

        val collection = collections.single()
        val folder = collection.folders.orEmpty().single()

        assertEquals("https://cdn.example.com/action-cover.jpg", collection.imageUrl)
        assertEquals("https://cdn.example.com/action-cover.jpg", folder.imageUrl)
        assertEquals("https://cdn.example.com/action-cover.jpg", folder.coverImageUrl)
        assertEquals("https://cdn.example.com/action-hero.jpg", folder.heroBackdropUrl)
        assertEquals("https://cdn.example.com/action-focus.gif", folder.focusGifUrl)
    }

    @Test
    fun importUsesLegacyImageAsFolderCoverOnly() {
        val collections = importLibraryCollectionsJson(
            """
            {
              "title": "Featured",
              "folders": [
                {
                  "title": "Action",
                  "image": "https://cdn.example.com/action-cover.jpg",
                  "heroBackdropUrl": "https://cdn.example.com/action-hero.jpg",
                  "catalogId": "top"
                }
              ]
            }
            """.trimIndent()
        )

        val folder = collections.single().folders.orEmpty().single()

        assertEquals("https://cdn.example.com/action-cover.jpg", folder.coverImageUrl)
        assertEquals("https://cdn.example.com/action-cover.jpg", folder.imageUrl)
        assertEquals("https://cdn.example.com/action-hero.jpg", folder.heroBackdropUrl)
    }

    @Test
    fun importAcceptsPosterAliasAndNormalizesGithubBlobArtwork() {
        val collections = importLibraryCollectionsJson(
            """
            {
              "title": "Featured",
              "folders": [
                {
                  "title": "Action",
                  "poster": "https://github.com/example/assets/blob/main/covers/action cover.jpg",
                  "background": "https://cdn.example.com/action-hero.jpg",
                  "catalogId": "top"
                }
              ]
            }
            """.trimIndent()
        )

        val folder = collections.single().folders.orEmpty().single()

        assertEquals("https://raw.githubusercontent.com/example/assets/main/covers/action%20cover.jpg", folder.coverImageUrl)
        assertEquals("https://raw.githubusercontent.com/example/assets/main/covers/action%20cover.jpg", folder.effectiveImageUrl())
        assertEquals("https://cdn.example.com/action-hero.jpg", folder.heroBackdropUrl)
    }

    @Test
    fun importDoesNotTreatPinToTopAsShowAboveContinueWatching() {
        val collections = importLibraryCollectionsJson(
            """
            {
              "title": "Featured",
              "pinToTop": true,
              "folders": [
                {
                  "title": "Action",
                  "catalogId": "top"
                }
              ]
            }
            """.trimIndent()
        )

        val collection = collections.single()

        assertFalse(collection.showOnHome == true)
        assertTrue(collection.pinToTop == true)
    }

    @Test
    fun importKeepsExplicitShowOnHomeSetting() {
        val collections = importLibraryCollectionsJson(
            """
            {
              "title": "Featured",
              "showOnHome": true,
              "folders": [
                {
                  "title": "Action",
                  "catalogId": "top"
                }
              ]
            }
            """.trimIndent()
        )

        assertTrue(collections.single().showOnHome == true)
    }
}
