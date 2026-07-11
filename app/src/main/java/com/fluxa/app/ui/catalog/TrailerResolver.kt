package com.fluxa.app.ui.catalog

import android.util.Log
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import okhttp3.MediaType.Companion.toMediaType
import okhttp3.OkHttpClient
import okhttp3.Request
import okhttp3.RequestBody.Companion.toRequestBody
import org.json.JSONObject
import java.util.concurrent.ConcurrentHashMap

internal data class SubtitleInfo(
    val languageTag: String,
    val label: String,
    val url: String,
    val mimeType: String,
    val isAuto: Boolean
)

internal data class TrailerResult(
    val streamUrl: String,
    val subtitles: List<SubtitleInfo>
)

internal sealed interface TrailerResolveResult {
    data class Ok(val data: TrailerResult) : TrailerResolveResult
    data object GeoBlocked : TrailerResolveResult
    data object Failed : TrailerResolveResult
}

internal object TrailerResolver {

    private val memCache = ConcurrentHashMap<String, TrailerResolveResult>(16)
    private val httpClient = OkHttpClient()

    fun init(cacheDir: java.io.File) = Unit

    private fun videoId(youtubeUrl: String): String? =
        Regex("(?:v=|youtu\\.be/|embed/)([A-Za-z0-9_-]{11})").find(youtubeUrl)?.groupValues?.get(1)

    suspend fun resolve(youtubeUrl: String): TrailerResolveResult = withContext(Dispatchers.IO) {
        val id = videoId(youtubeUrl) ?: return@withContext TrailerResolveResult.Failed
        memCache[id]?.let { return@withContext it }

        val result = try {
            resolveInnertubeTrailer(id)
        } catch (e: Exception) {
            Log.e("TrailerResolver", "resolve($id) → ${e.javaClass.simpleName}: ${e.message}")
            TrailerResolveResult.Failed
        }
        if (result is TrailerResolveResult.Ok) memCache[id] = result
        result
    }

    private fun resolveInnertubeTrailer(videoId: String): TrailerResolveResult {
        val body = JSONObject()
            .put("videoId", videoId)
            .put("context", JSONObject().put("client", JSONObject()
                .put("clientName", "ANDROID")
                .put("clientVersion", "20.10.38")
                .put("androidSdkVersion", 35)
            ))
            .toString()
            .toRequestBody("application/json".toMediaType())
        val request = Request.Builder()
            .url("https://www.youtube.com/youtubei/v1/player?prettyPrint=false")
            .header("User-Agent", "com.google.android.youtube/20.10.38 (Linux; U; Android 15) gzip")
            .header("X-YouTube-Client-Name", "3")
            .header("X-YouTube-Client-Version", "20.10.38")
            .post(body)
            .build()
        httpClient.newCall(request).execute().use { response ->
            if (!response.isSuccessful) return TrailerResolveResult.Failed
            val payload = JSONObject(response.body.string())
            val playability = payload.optJSONObject("playabilityStatus")
            if (playability?.optString("status") == "LOGIN_REQUIRED" || playability?.optString("reason")?.contains("country", true) == true) {
                return TrailerResolveResult.GeoBlocked
            }
            if (playability?.optString("status") != "OK") return TrailerResolveResult.Failed
            val streaming = payload.optJSONObject("streamingData") ?: return TrailerResolveResult.Failed
            val streamUrl = streaming.optString("hlsManifestUrl").takeIf { it.isNotBlank() }
                ?: streaming.optJSONArray("formats")?.let { formats ->
                    (0 until formats.length())
                        .asSequence()
                        .map { formats.optJSONObject(it)?.optString("url").orEmpty() }
                        .firstOrNull { it.isNotBlank() }
                }
                ?: streaming.optJSONArray("adaptiveFormats")?.let { formats ->
                    (0 until formats.length())
                        .asSequence()
                        .map { formats.optJSONObject(it)?.optString("url").orEmpty() }
                        .firstOrNull { it.isNotBlank() }
                }
                ?: return TrailerResolveResult.Failed
            val subtitles = payload.optJSONObject("captions")
                ?.optJSONObject("playerCaptionsTracklistRenderer")
                ?.optJSONArray("captionTracks")
                ?.let { tracks ->
                    (0 until tracks.length()).mapNotNull { index ->
                        tracks.optJSONObject(index)?.let { track ->
                            val url = track.optString("baseUrl")
                            if (url.isBlank()) null else SubtitleInfo(
                                languageTag = track.optString("languageCode", "und"),
                                label = track.optJSONObject("name")?.optString("simpleText")?.ifBlank { track.optString("languageCode") } ?: "",
                                url = url,
                                mimeType = "text/vtt",
                                isAuto = track.optString("kind") == "asr"
                            )
                        }
                    }
                }.orEmpty()
            return TrailerResolveResult.Ok(TrailerResult(streamUrl, subtitles))
        }
    }

    private fun parseResult(response: JSONObject): TrailerResolveResult = when (response.optString("status")) {
        "ok" -> TrailerResolveResult.Ok(
            TrailerResult(
                streamUrl = response.getString("streamUrl"),
                subtitles = response.optJSONArray("subtitles")?.let { tracks ->
                    (0 until tracks.length()).map { i ->
                        val track = tracks.getJSONObject(i)
                        SubtitleInfo(
                            languageTag = track.getString("languageTag"),
                            label = track.getString("label"),
                            url = track.getString("url"),
                            mimeType = track.getString("mimeType"),
                            isAuto = track.getBoolean("isAuto")
                        )
                    }
                } ?: emptyList()
            )
        )
        "geo_blocked" -> TrailerResolveResult.GeoBlocked
        else -> TrailerResolveResult.Failed
    }
}
