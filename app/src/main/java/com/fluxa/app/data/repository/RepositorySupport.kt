package com.fluxa.app.data.repository

import com.fluxa.app.common.TtlMemoryCache
import com.google.gson.Gson
import com.google.gson.reflect.TypeToken

import java.util.Locale

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

internal object TmdbLanguageResolver {
    fun languageTag(language: String?): String {
        val normalized = normalizedLanguage(language)
        val parts = normalized.split('_').filter { it.isNotBlank() }
        val languageCode = languageCode(language)
        val explicitCountry = parts.getOrNull(1)?.takeIf { it.length == 2 }?.uppercase(Locale.ROOT)
        return if (explicitCountry != null) "$languageCode-$explicitCountry" else languageCode
    }

    fun languageCode(language: String?): String {
        val parts = normalizedLanguage(language).split('_').filter { it.isNotBlank() }
        return when (parts.firstOrNull()) {
            "english" -> "en"
            null, "" -> "en"
            else -> parts.first().take(2)
        }
    }

    private fun normalizedLanguage(language: String?): String {
        return language
            ?.trim()
            ?.lowercase(Locale.ROOT)
            ?.removeSuffix(".json")
            ?.replace('-', '_')
            .orEmpty()
    }
}
