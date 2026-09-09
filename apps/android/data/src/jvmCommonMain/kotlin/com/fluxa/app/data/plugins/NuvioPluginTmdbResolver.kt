package com.fluxa.app.data.plugins

import com.fluxa.app.common.PlatformLog
import com.fluxa.app.core.rust.FluxaCoreNative
import com.fluxa.app.data.remote.TmdbService
import java.util.concurrent.ConcurrentHashMap
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock

private val imdbToTmdbCache = ConcurrentHashMap<String, String>()
private val imdbToTmdbMutex = Mutex()

/**
 * Nuvio-style best-effort TMDB resolver.
 * Numeric IDs pass through. IMDb IDs are converted when the active profile has a TMDB API key.
 * Callers should fall back to the original ID when this returns null.
 */
suspend fun resolveNuvioPluginTmdbId(
    tmdbService: TmdbService,
    contentId: String,
    mediaType: String,
    apiKey: String,
): String? {
    val normalized = FluxaCoreNative.tmdbNumericId(contentId)
    if (normalized != null) return normalized
    val imdbId = FluxaCoreNative.contentImdbId(contentId) ?: return null
    if (apiKey.isBlank()) return null

    val normalizedType = FluxaCoreNative.nuvioPluginContentType(mediaType)
    val cacheKey = "$imdbId:$normalizedType"
    imdbToTmdbCache[cacheKey]?.let { return it }

    return imdbToTmdbMutex.withLock {
        imdbToTmdbCache[cacheKey]?.let { return@withLock it }
        runCatching {
            val response = tmdbService.findById(imdbId, apiKey = apiKey)
            val id = when (normalizedType) {
                "tv" -> response.tvResults.firstOrNull()?.id
                "movie" -> response.movieResults.firstOrNull()?.id
                else -> response.movieResults.firstOrNull()?.id ?: response.tvResults.firstOrNull()?.id
            }?.takeIf { it > 0 }?.toString()
            if (id != null) imdbToTmdbCache[cacheKey] = id
            id
        }.onFailure { error ->
            PlatformLog.w("PluginTmdbResolver", "TMDB lookup failed for $imdbId ($normalizedType)", error)
        }.getOrNull()
    }
}
