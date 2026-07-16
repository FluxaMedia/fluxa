package com.fluxa.app.data.repository

import android.util.Log
import com.google.gson.JsonElement
import com.fluxa.app.data.remote.AniSkipService
import com.fluxa.app.data.remote.IntroDbService
import com.fluxa.app.data.remote.IntroDbSubmissionRequest
import com.fluxa.app.data.remote.IntroTimestamps
import com.fluxa.app.data.remote.StremioService
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import okhttp3.Request
import org.json.JSONObject
import java.net.URLEncoder
import java.util.Locale

sealed interface IntroDbSubmitResult {
    data class Success(val status: String) : IntroDbSubmitResult
    data class Error(val reason: String) : IntroDbSubmitResult
}

internal class IntroRepository(
    private val introService: IntroDbService,
    private val aniSkipService: AniSkipService
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
                Log.w(TAG, "IntroDB submission failed for $imdbId S${season}E$episode: HTTP ${response.code()}")
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
            Log.w(TAG, "IntroDB submission failed for $imdbId S${season}E$episode", e)
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
                    response.body()?.let { results.addAll(parseIntroDbSegments(it)) }
                }
            } catch (e: Exception) {
                Log.w(TAG, "IntroDB lookup failed for $imdbId S${season}E${episode}", e)
            }
        }
        if (useAniSkip) {
            try {
                val malId = resolveMalId(imdbId, title)
                if (malId != null) {
                    val aniResp = aniSkipService.getSkipTimes(malId, episode)
                    if (aniResp.isSuccessful) {
                        aniResp.body()?.results?.forEach { res ->
                            val type = when (res.skipType) {
                                "op" -> "intro"
                                "ed" -> "outro"
                                "recap" -> "recap"
                                else -> "intro"
                            }
                            results.add(
                                IntroTimestamps(
                                    (res.interval.startTime * 1000).toLong(),
                                    (res.interval.endTime * 1000).toLong(),
                                    type
                                )
                            )
                        }
                    }
                }
            } catch (e: Exception) {
                Log.w(TAG, "AniSkip lookup failed for $imdbId / $title episode $episode", e)
            }
        }
        results.distinctBy { "${it.startTime}-${it.type}" }
    }

    private fun parseIntroDbSegments(json: JsonElement): List<IntroTimestamps> {
        if (json.isJsonArray) {
            return json.asJsonArray.flatMap { parseIntroDbSegments(it) }
        }
        if (!json.isJsonObject) return emptyList()
        val obj = json.asJsonObject
        val segments = mutableListOf<IntroTimestamps>()
        listOf("segments", "results", "data", "items").forEach { key ->
            val nested = obj.get(key)
            if (nested != null && !nested.isJsonNull) {
                segments += parseIntroDbSegments(nested)
            }
        }
        val directStart = introDbNumber(obj, "startTime", "start", "from", "start_sec", "start_time", "startTimeMs", "start_ms", "startOffset")
        val directEnd = introDbNumber(obj, "endTime", "end", "to", "end_sec", "end_time", "endTimeMs", "end_ms", "endOffset")
        val directType = (obj.get("segment_type") ?: obj.get("skip_type") ?: obj.get("category") ?: obj.get("name") ?: obj.get("type"))
            ?.asStringOrNull()
            ?.lowercase(Locale.ROOT)
            ?.takeIf { it.isNotBlank() }
            ?: "intro"
        if (directStart != null && directEnd != null) {
            segments += IntroTimestamps(normalizeSkipTime(directStart), normalizeSkipTime(directEnd), normalizeSkipType(directType))
        }
        listOf("intro", "outro", "recap").forEach { type ->
            val nested = obj.get(type)?.takeIf { it.isJsonObject }?.asJsonObject
            if (nested != null) {
                val start = introDbNumber(nested, "startTime", "start", "from", "start_sec", "start_time", "startTimeMs", "start_ms", "startOffset")
                val end = introDbNumber(nested, "endTime", "end", "to", "end_sec", "end_time", "endTimeMs", "end_ms", "endOffset")
                if (start != null && end != null) {
                    segments += IntroTimestamps(normalizeSkipTime(start), normalizeSkipTime(end), type)
                }
            }
            val start = introDbNumber(obj, "${type}Start", "${type}_start", "${type}StartTime", "${type}_start_time", "${type}StartMs", "${type}_start_ms")
            val end = introDbNumber(obj, "${type}End", "${type}_end", "${type}EndTime", "${type}_end_time", "${type}EndMs", "${type}_end_ms")
            if (start != null && end != null) {
                segments += IntroTimestamps(normalizeSkipTime(start), normalizeSkipTime(end), type)
            }
        }
        return segments
            .filter { it.endTime > it.startTime }
            .distinctBy { "${it.type}:${it.startTime}:${it.endTime}" }
    }

    private fun introDbNumber(obj: com.google.gson.JsonObject, vararg keys: String): Double? {
        return keys.firstNotNullOfOrNull { key ->
            obj.get(key)?.takeUnless { it.isJsonNull }?.let { value ->
                runCatching { value.asDouble }.getOrNull()
                    ?: value.asStringOrNull()?.let { parseIntroDbTime(it) }
            }
        }
    }

    private fun parseIntroDbTime(value: String): Double? {
        val trimmed = value.trim()
        if (trimmed.isEmpty()) return null
        trimmed.toDoubleOrNull()?.let { return it }
        val parts = trimmed.split(':').map { it.toDoubleOrNull() ?: return null }
        return when (parts.size) {
            2 -> parts[0] * 60 + parts[1]
            3 -> parts[0] * 3600 + parts[1] * 60 + parts[2]
            else -> null
        }
    }

    private fun JsonElement.asStringOrNull(): String? = runCatching { asString }.getOrNull()

    private fun normalizeSkipTime(value: Double): Long {
        return if (value < 10_000.0) (value * 1000.0).toLong() else value.toLong()
    }

    private fun normalizeSkipType(type: String): String = when (type.lowercase(Locale.ROOT)) {
        "opening", "op" -> "intro"
        "ending", "ed", "credits" -> "outro"
        "previously" -> "recap"
        else -> type
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
                JSONObject(response.body.string())
            }
            val data = json.optJSONArray("data") ?: return null
            (0 until data.length())
                .asSequence()
                .mapNotNull { idx -> data.optJSONObject(idx) }
                .mapNotNull { item -> item.optInt("mal_id").takeIf { it > 0 } }
                .firstOrNull()
        } catch (e: Exception) {
            Log.w(TAG, "MAL id resolution failed for $imdbId / $query", e)
            null
        }
    }

    private companion object {
        const val TAG = "IntroRepository"
    }
}
