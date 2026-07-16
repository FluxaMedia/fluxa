package com.fluxa.app.data.apple

import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertTrue

class AppleStremioBridgeTest {
    @Test
    fun parsesManifestCatalogCapabilities() {
        val manifest = AppleStremioBridge.parseManifest(
            """{"id":"addon","resources":[{"name":"catalog"}],"catalogs":[{"type":"movie","id":"popular","extra":[{"name":"search"},{"name":"genre","isRequired":true,"options":["Action"]}]}]}"""
        )

        assertEquals("addon", manifest?.id)
        assertTrue(manifest?.supportsCatalog == true)
        val catalog = manifest?.catalogs?.single()
        assertTrue(catalog?.supportsSearch == true)
        assertFalse(catalog?.supportsInitialLoad == true)
        assertFalse(catalog?.hasRequiredExtraExceptGenre == true)
    }
}
