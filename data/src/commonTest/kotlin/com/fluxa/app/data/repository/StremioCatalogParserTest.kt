package com.fluxa.app.data.repository

import kotlin.test.Test
import kotlin.test.assertEquals

class StremioCatalogParserTest {
    @Test
    fun parsesPortableCatalogMetadata() {
        val items = StremioCatalogParser.parse(
            """{"metas":[{"id":"tt1","name":"Example","type":"movie","poster":"https://image"}]}"""
        )

        assertEquals("tt1", items?.single()?.id)
        assertEquals("https://image", items?.single()?.poster)
    }
}
