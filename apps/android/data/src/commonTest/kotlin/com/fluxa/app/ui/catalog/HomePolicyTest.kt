package com.fluxa.app.ui.catalog

import kotlin.test.Test
import kotlin.test.assertFalse
import kotlin.test.assertTrue

class HomePolicyTest {
    @Test
    fun rejectsCalendarPlaceholderArtwork() {
        assertFalse(isUsableCalendarArtwork("https://example.com/default-poster.png"))
        assertFalse(isUsableCalendarArtwork("null"))
        assertTrue(isUsableCalendarArtwork("https://example.com/poster.jpg"))
    }
}
