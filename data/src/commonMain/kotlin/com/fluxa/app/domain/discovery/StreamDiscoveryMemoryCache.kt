package com.fluxa.app.domain.discovery

import com.fluxa.app.common.Constants
import com.fluxa.app.common.TtlMemoryCache
import com.fluxa.app.common.epochMillisNow
import com.fluxa.app.data.remote.Stream

private const val DEFAULT_MAX_STREAM_CACHE_ENTRIES = 128

class StreamDiscoveryMemoryCache(
    private val maxEntries: Int = DEFAULT_MAX_STREAM_CACHE_ENTRIES,
    private val ttlMillis: Long = Constants.Cache.STREAM_DURATION_MS,
    private val nowMillis: () -> Long = ::epochMillisNow
) {
    private val cache = TtlMemoryCache<List<Stream>>(maxEntries, ttlMillis, nowMillis)

    fun get(key: String): List<Stream>? = cache.get(key)

    fun put(key: String, streams: List<Stream>) {
        cache.put(key, streams)
    }

    fun firstWithPrefix(prefix: String): List<Stream>? = cache.firstWithPrefix(prefix)

    fun invalidatePrefix(prefix: String) {
        cache.invalidatePrefix(prefix)
    }

    fun size(): Int = cache.size()
}
