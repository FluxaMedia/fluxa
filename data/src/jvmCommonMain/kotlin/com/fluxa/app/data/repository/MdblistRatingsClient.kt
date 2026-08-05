package com.fluxa.app.data.repository

import com.fluxa.app.core.rust.FluxaCoreNative
import com.fluxa.app.data.remote.MetaRating
import com.fluxa.app.data.remote.StremioService
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import okhttp3.Request
import javax.inject.Inject
import javax.inject.Singleton

@Singleton
class MdblistRatingsClient @Inject constructor() {
    suspend fun fetch(contentType: String, contentId: String, apiKey: String): List<MetaRating> = withContext(Dispatchers.IO) {
        if (apiKey.isBlank()) return@withContext emptyList()
        val tmdbId = contentId.removePrefix("tmdb:").substringBefore(':')
        val imdbId = contentId.substringBefore(':')
        val providerAndId = when {
            tmdbId.all(Char::isDigit) && tmdbId.isNotBlank() -> "tmdb" to tmdbId
            imdbId.matches(Regex("(?i)tt\\d+")) -> "imdb" to imdbId
            else -> return@withContext emptyList()
        }
        runCatching {
            val url = FluxaCoreNative.mdblistMediaInfoUrl(
                provider = providerAndId.first,
                mediaType = if (contentType == "series") "show" else "movie",
                mediaId = providerAndId.second
            )
            val request = Request.Builder().url("$url&apikey=$apiKey").get().build()
            StremioService.sharedClient.newCall(request).execute().use { response ->
                if (!response.isSuccessful) return@use emptyList()
                val body = response.body.string()
                FluxaCoreNative.mdblistMediaRatingsFromResponse(body)
                    .map { (source, value) -> MetaRating(source, value) }
            }
        }.getOrDefault(emptyList())
    }
}
