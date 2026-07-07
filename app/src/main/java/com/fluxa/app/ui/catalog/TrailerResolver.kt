package com.fluxa.app.ui.catalog

import android.util.Log
import com.fluxa.app.core.rust.FluxaStreamingNative
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import org.json.JSONObject
import java.io.File
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
    private var cacheDir: File? = null

    fun init(cacheDir: File) {
        this.cacheDir = File(cacheDir, "youtube-trailer-cache").apply { mkdirs() }
    }

    private fun videoId(youtubeUrl: String): String? =
        Regex("(?:v=|youtu\\.be/|embed/)([A-Za-z0-9_-]{11})").find(youtubeUrl)?.groupValues?.get(1)

    suspend fun resolve(youtubeUrl: String): TrailerResolveResult = withContext(Dispatchers.IO) {
        val id = videoId(youtubeUrl) ?: return@withContext TrailerResolveResult.Failed
        memCache[id]?.let { return@withContext it }

        val cacheDir = cacheDir ?: return@withContext TrailerResolveResult.Failed
        val result = try {
            val json = FluxaStreamingNative.resolveYoutubeTrailerJson(id, cacheDir.absolutePath)
                ?: return@withContext TrailerResolveResult.Failed
            parseResult(JSONObject(json))
        } catch (e: Exception) {
            Log.e("TrailerResolver", "resolve($id) → ${e.javaClass.simpleName}: ${e.message}")
            TrailerResolveResult.Failed
        }
        if (result is TrailerResolveResult.Ok) memCache[id] = result
        result
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
