package com.fluxa.app.domain.discovery

import com.fluxa.app.core.rust.FluxaCoreNative
import com.fluxa.app.data.remote.Stream
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test

class StreamDiscoveryMemoryCacheTest {

    @Test
    fun expiresEntriesAfterTtl() {
        var now = 1_000L
        val cache = StreamDiscoveryMemoryCache(
            maxEntries = 4,
            ttlMillis = 100L,
            nowMillis = { now }
        )

        cache.put("movie|tt1|tr", listOf(Stream(name = "Source", title = null, url = "https://example.com/video.mp4")))

        assertEquals(1, cache.get("movie|tt1|tr")?.size)

        now = 1_101L

        assertNull(cache.get("movie|tt1|tr"))
        assertEquals(0, cache.size())
    }

    @Test
    fun trimsOldestEntriesWhenMaxSizeIsExceeded() {
        var now = 1_000L
        val cache = StreamDiscoveryMemoryCache(
            maxEntries = 2,
            ttlMillis = 10_000L,
            nowMillis = { now }
        )

        cache.put("movie|tt1|tr", listOf(Stream(name = "One", title = null, url = "https://example.com/1.mp4")))
        now += 1
        cache.put("movie|tt2|tr", listOf(Stream(name = "Two", title = null, url = "https://example.com/2.mp4")))
        now += 1
        cache.put("movie|tt3|tr", listOf(Stream(name = "Three", title = null, url = "https://example.com/3.mp4")))

        assertNull(cache.get("movie|tt1|tr"))
        assertEquals(1, cache.get("movie|tt2|tr")?.size)
        assertEquals(1, cache.get("movie|tt3|tr")?.size)
        assertEquals(2, cache.size())
    }

    @Test
    fun prefixLookupUsesNativeDiscoveryPrefixShape() {
        val cache = StreamDiscoveryMemoryCache(
            maxEntries = 4,
            ttlMillis = 10_000L,
            nowMillis = { 1_000L }
        )
        val prefix = FluxaCoreNative.streamDiscoveryCachePrefix("movie", "tt1", "tr")
        cache.put("$prefix|query|plugins", listOf(Stream(name = "One", title = null, url = "https://example.com/1.mp4")))

        assertEquals(1, cache.firstWithPrefix(prefix)?.size)

        cache.invalidatePrefix(prefix)

        assertNull(cache.firstWithPrefix(prefix))
    }
}
