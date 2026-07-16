package com.fluxa.app.data.repository

import com.fluxa.app.common.TtlMemoryCache
import com.google.gson.Gson
import com.google.gson.reflect.TypeToken

import javax.inject.Inject
import javax.inject.Singleton

@Singleton
class RepositoryMemoryCache @Inject constructor(
    @PublishedApi internal val gson: Gson
) {
    private val durationMs: Long = 10 * 60 * 1000L
    private val maxEntries: Int = 512
    @PublishedApi
    internal val cache = TtlMemoryCache<String>(maxEntries, durationMs)

    inline fun <reified T> get(key: String): T? {
        val json = cache.get(key) ?: return null
        return gson.fromJson<T>(json, object : TypeToken<T>() {}.type)
    }

    fun put(key: String, value: Any) {
        cache.put(key, gson.toJson(value))
    }
}
