package com.fluxa.app.domain.discovery

import com.fluxa.app.data.remote.Stream
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNull

class StreamDiscoveryMemoryCacheTest {
    @Test
    fun expiresEntriesAfterTtl() {
        var now = 1_000L
        val cache = StreamDiscoveryMemoryCache(4, 100L) { now }
        cache.put("movie|tt1|tr", listOf(Stream(name = "Source", title = null, url = "https://example.com/video.mp4")))

        assertEquals(1, cache.get("movie|tt1|tr")?.size)
        now = 1_101L
        assertNull(cache.get("movie|tt1|tr"))
        assertEquals(0, cache.size())
    }

    @Test
    fun trimsOldestEntriesWhenMaxSizeIsExceeded() {
        var now = 1_000L
        val cache = StreamDiscoveryMemoryCache(2, 10_000L) { now }
        cache.put("one", listOf(Stream(name = "One", title = null, url = "https://example.com/1.mp4")))
        now += 1
        cache.put("two", listOf(Stream(name = "Two", title = null, url = "https://example.com/2.mp4")))
        now += 1
        cache.put("three", listOf(Stream(name = "Three", title = null, url = "https://example.com/3.mp4")))

        assertNull(cache.get("one"))
        assertEquals(1, cache.get("two")?.size)
        assertEquals(1, cache.get("three")?.size)
        assertEquals(2, cache.size())
    }

    @Test
    fun prefixLookupAndInvalidationUseStableKeys() {
        val cache = StreamDiscoveryMemoryCache(4, 10_000L) { 1_000L }
        val prefix = "movie|tt1|tr"
        cache.put("$prefix|query|plugins", listOf(Stream(name = "One", title = null, url = "https://example.com/1.mp4")))

        assertEquals(1, cache.firstWithPrefix(prefix)?.size)
        cache.invalidatePrefix(prefix)
        assertNull(cache.firstWithPrefix(prefix))
    }
}
