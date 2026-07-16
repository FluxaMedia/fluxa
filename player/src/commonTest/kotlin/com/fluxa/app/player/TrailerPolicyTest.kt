package com.fluxa.app.player

import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNull

class TrailerPolicyTest {
    @Test
    fun extractsSupportedYoutubeVideoIds() {
        assertEquals("abcdefghijk", TrailerPolicy.youtubeVideoId("https://youtu.be/abcdefghijk"))
        assertEquals("abcdefghijk", TrailerPolicy.youtubeVideoId("https://youtube.com/watch?v=abcdefghijk"))
        assertNull(TrailerPolicy.youtubeVideoId("https://example.com/video"))
    }
}
