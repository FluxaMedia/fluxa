package com.fluxa.app.data.repository

import com.fluxa.app.common.PlatformLog
import com.fluxa.app.core.rust.FluxaCoreNative
import com.google.gson.Gson
import com.google.gson.JsonParser
import com.fluxa.app.data.remote.AniSkipService
import com.fluxa.app.data.remote.IntroDbService
import com.fluxa.app.data.remote.IntroDbSubmissionRequest
import com.fluxa.app.data.remote.IntroTimestamps
import com.fluxa.app.data.remote.StremioService
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import okhttp3.Request
import java.net.URLEncoder

sealed interface IntroDbSubmitResult {
    data class Success(val status: String) : IntroDbSubmitResult
    data class Error(val reason: String) : IntroDbSubmitResult
}

class IntroRepository(
    private val introService: IntroDbService,
    private val aniSkipService: AniSkipService,
    private val gson: Gson = Gson()
) {
    private val httpClient get() = StremioService.sharedClient

    suspend fun submitSegment(
        apiKey: String,
        segmentType: String,
        imdbId: String,
        season: Int,
        episode: Int,
        startSec: Double,
        endSec: Double
    ): IntroDbSubmitResult = withContext(Dispatchers.IO) {
        try {
            val response = introService.submitSegment(
                apiKey,
                IntroDbSubmissionRequest(segmentType, imdbId, season, episode, startSec, endSec)
            )
            if (response.isSuccessful) {
                IntroDbSubmitResult.Success(response.body()?.status ?: "pending")
            } else {
                PlatformLog.w(TAG, "IntroDB submission failed for $imdbId S${season}E$episode: HTTP ${response.code()}")
                IntroDbSubmitResult.Error(
                    when (response.code()) {
                        401 -> "invalid_key"
                        429 -> "rate_limited"
                        400 -> "invalid_segment"
                        else -> "network_error"
                    }
                )
            }
        } catch (e: Exception) {
            PlatformLog.w(TAG, "IntroDB submission failed for $imdbId S${season}E$episode", e)
            IntroDbSubmitResult.Error("network_error")
        }
    }

    suspend fun getIntro(
        imdbId: String,
        season: Int,
        episode: Int,
        title: String? = null,
        useIntroDb: Boolean = true,
        useAniSkip: Boolean = true
    ): List<IntroTimestamps> = withContext(Dispatchers.IO) {
        val results = mutableListOf<IntroTimestamps>()
        if (useIntroDb) {
            try {
                val response = introService.getIntro(imdbId, season, episode)
                if (response.isSuccessful) {
                    response.body()?.let { body ->
                        results.addAll(FluxaCoreNative.parseIntroDbSegments(gson.toJsonTree(body)))
                    }
                }
            } catch (e: Exception) {
                PlatformLog.w(TAG, "IntroDB lookup failed for $imdbId S${season}E${episode}", e)
            }
        }
        if (useAniSkip) {
            try {
                val malId = resolveMalId(imdbId, title)
                if (malId != null) {
                    val aniResp = aniSkipService.getSkipTimes(malId, episode)
                    if (aniResp.isSuccessful) {
                        aniResp.body()?.let { body ->
                            results.addAll(FluxaCoreNative.parseAniskipResults(gson.toJsonTree(body)))
                        }
                    }
                }
            } catch (e: Exception) {
                PlatformLog.w(TAG, "AniSkip lookup failed for $imdbId / $title episode $episode", e)
            }
        }
        FluxaCoreNative.mergeIntroSegments(listOf(results))
    }

    private suspend fun resolveMalId(imdbId: String, title: String?): Int? {
        val query = title
            ?.replace(Regex("""\s+\(\d{4}\)$"""), "")
            ?.trim()
            ?.takeIf { it.length >= 2 }
            ?: return null
        return try {
            val encoded = URLEncoder.encode(query, "UTF-8")
            val request = Request.Builder()
                .url("https://api.jikan.moe/v4/anime?q=$encoded&limit=5")
                .build()
            val json = httpClient.newCall(request).execute().use { response ->
                if (!response.isSuccessful) return null
                JsonParser.parseString(response.body.string()).asJsonObject
            }
            val data = json.getAsJsonArray("data") ?: return null
            data.asSequence()
                .filter { it.isJsonObject }
                .mapNotNull { it.asJsonObject.get("mal_id")?.takeIf { id -> id.isJsonPrimitive }?.asInt }
                .firstOrNull { it > 0 }
        } catch (e: Exception) {
            PlatformLog.w(TAG, "MAL id resolution failed for $imdbId / $query", e)
            null
        }
    }

    private companion object {
        const val TAG = "IntroRepository"
    }
}
