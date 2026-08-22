package com.fluxa.app.domain.discovery

import kotlin.test.Test
import kotlin.test.assertEquals

class StremioResourceUrlBuilderTest {
    @Test
    fun normalizesManifestAndEncodesResourceSegments() {
        assertEquals(
            "https://example.com/manifest.json",
            StremioResourceUrlBuilder.normalizeManifestUrl("stremio://example.com")
        )
        assertEquals(
            "https://example.com/catalog/movie/popular/search=Dune%20Part%20Two.json",
            StremioResourceUrlBuilder.catalogUrl(
                "https://example.com/manifest.json",
                "movie",
                "popular",
                "search",
                "Dune Part Two"
            )
        )
    }
}
