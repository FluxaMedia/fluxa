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
        val tmdbId = FluxaCoreNative.tmdbNumericId(contentId).orEmpty()
        val imdbId = FluxaCoreNative.contentImdbId(contentId).orEmpty()
        val providerAndId = when {
            tmdbId.all(Char::isDigit) && tmdbId.isNotBlank() -> "tmdb" to tmdbId
            imdbId.isNotBlank() -> "imdb" to imdbId
            else -> return@withContext emptyList()
        }
        runCatching {
            val url = FluxaCoreNative.mdblistMediaInfoUrl(
                provider = providerAndId.first,
                mediaType = FluxaCoreNative.mdblistContentType(contentType),
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
