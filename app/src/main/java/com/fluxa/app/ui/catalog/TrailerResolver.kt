package com.fluxa.app.ui.catalog

import android.util.Log
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import okhttp3.MediaType.Companion.toMediaType
import okhttp3.OkHttpClient
import okhttp3.Request
import okhttp3.RequestBody.Companion.toRequestBody
import org.json.JSONArray
import org.json.JSONObject
import java.io.File
import java.util.Locale
import java.util.concurrent.ConcurrentHashMap
import java.util.concurrent.TimeUnit

private const val CACHE_TTL_MS = 6 * 60 * 60 * 1000L

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

    private val httpClient = OkHttpClient.Builder()
        .connectTimeout(15, TimeUnit.SECONDS)
        .readTimeout(20, TimeUnit.SECONDS)
        .build()

    private val memCache = ConcurrentHashMap<String, TrailerResolveResult>(16)
    private var cacheFile: File? = null

    fun init(cacheDir: File) {
        cacheFile = File(cacheDir, "trailer_url_cache.json")
        loadDiskCache()
    }

    private fun loadDiskCache() {
        val file = cacheFile ?: return
        if (!file.exists()) return
        runCatching {
            val json = JSONObject(file.readText())
            val now = System.currentTimeMillis()
            val keys = json.keys()
            while (keys.hasNext()) {
                val id = keys.next()
                val entry = json.getJSONObject(id)
                val cachedAt = entry.getLong("cachedAt")
                if (now - cachedAt > CACHE_TTL_MS) continue
                val url = entry.getString("url")
                memCache[id] = TrailerResolveResult.Ok(TrailerResult(url, emptyList()))
            }
        }.onFailure { Log.w("TrailerResolver", "disk cache load: ${it.message}") }
    }

    private fun saveToDiskCache(videoId: String, url: String) {
        val file = cacheFile ?: return
        runCatching {
            val json = if (file.exists()) JSONObject(file.readText()) else JSONObject()
            json.put(videoId, JSONObject().apply {
                put("url", url)
                put("cachedAt", System.currentTimeMillis())
            })
            val now = System.currentTimeMillis()
            val pruned = JSONObject()
            val keys = json.keys()
            while (keys.hasNext()) {
                val k = keys.next()
                val entry = json.getJSONObject(k)
                if (now - entry.getLong("cachedAt") <= CACHE_TTL_MS) pruned.put(k, entry)
            }
            file.writeText(pruned.toString())
        }.onFailure { Log.w("TrailerResolver", "disk cache save: ${it.message}") }
    }

    private fun videoId(youtubeUrl: String): String? =
        Regex("(?:v=|youtu\\.be/|embed/)([A-Za-z0-9_-]{11})").find(youtubeUrl)?.groupValues?.get(1)

    suspend fun resolve(youtubeUrl: String): TrailerResolveResult = withContext(Dispatchers.IO) {
        val id = videoId(youtubeUrl) ?: return@withContext TrailerResolveResult.Failed
        memCache[id]?.let { return@withContext it }

        val result = doExtract(id, androidClient())
        if (result is TrailerResolveResult.Ok) {
            memCache[id] = result
            saveToDiskCache(id, result.data.streamUrl)
            return@withContext result
        }

        if (result is TrailerResolveResult.GeoBlocked) {
            Log.d("TrailerResolver", "geo-blocked, retrying via embedded client")
            val embedResult = doExtract(id, embeddedClient())
            if (embedResult is TrailerResolveResult.Ok) {
                memCache[id] = embedResult
                saveToDiskCache(id, embedResult.data.streamUrl)
            }
            return@withContext embedResult
        }

        result
    }

    private fun androidClient() = JSONObject().apply {
        put("clientName", "ANDROID_TESTSUITE")
        put("clientVersion", "1.9")
        put("androidSdkVersion", 30)
        put("hl", "en")
        put("gl", "US")
    }

    private fun embeddedClient() = JSONObject().apply {
        put("clientName", "TVHTML5_SIMPLY_EMBEDDED_PLAYER")
        put("clientVersion", "2.0")
        put("hl", "en")
        put("gl", "US")
        put("embedUrl", "https://www.youtube.com/")
    }

    private fun doExtract(videoId: String, client: JSONObject): TrailerResolveResult {
        return try {
            val body = JSONObject().apply {
                put("videoId", videoId)
                put("context", JSONObject().put("client", client))
            }.toString()

            val request = Request.Builder()
                .url("https://www.youtube.com/youtubei/v1/player?prettyPrint=false")
                .post(body.toRequestBody("application/json; charset=utf-8".toMediaType()))
                .header("X-YouTube-Client-Name", client.optString("clientName"))
                .header("X-YouTube-Client-Version", client.optString("clientVersion"))
                .build()

            val response = httpClient.newCall(request).execute().use { resp ->
                if (!resp.isSuccessful) return TrailerResolveResult.Failed
                JSONObject(resp.body!!.string())
            }

            val status = response.optJSONObject("playabilityStatus")?.optString("status")
            if (status == "UNPLAYABLE" || status == "LOGIN_REQUIRED") {
                return TrailerResolveResult.GeoBlocked
            }

            val streamingData = response.optJSONObject("streamingData")
                ?: return TrailerResolveResult.Failed

            val hlsUrl = streamingData.optString("hlsManifestUrl").takeIf { it.startsWith("http") }
            if (hlsUrl != null) {
                return TrailerResolveResult.Ok(TrailerResult(hlsUrl, extractSubtitles(response)))
            }

            val dashUrl = streamingData.optString("dashManifestUrl").takeIf { it.startsWith("http") }
            if (dashUrl != null) {
                return TrailerResolveResult.Ok(TrailerResult(dashUrl, extractSubtitles(response)))
            }

            val streamUrl = bestMp4(streamingData.optJSONArray("formats"))
                ?: bestMp4(streamingData.optJSONArray("adaptiveFormats"))
                ?: return TrailerResolveResult.Failed

            TrailerResolveResult.Ok(TrailerResult(streamUrl, extractSubtitles(response)))
        } catch (e: Exception) {
            Log.e("TrailerResolver", "doExtract($videoId) → ${e.javaClass.simpleName}: ${e.message}")
            TrailerResolveResult.Failed
        }
    }

    private fun bestMp4(formats: JSONArray?): String? {
        formats ?: return null
        var bestUrl: String? = null
        var bestHeight = -1
        for (i in 0 until formats.length()) {
            val fmt = formats.getJSONObject(i)
            if (!fmt.optString("mimeType").startsWith("video/mp4")) continue
            val url = fmt.optString("url").takeIf { it.startsWith("http") } ?: continue
            val height = fmt.optString("qualityLabel").removeSuffix("p").toIntOrNull() ?: 0
            if (height > bestHeight) {
                bestHeight = height
                bestUrl = url
            }
        }
        return bestUrl
    }

    private fun extractSubtitles(response: JSONObject): List<SubtitleInfo> {
        return runCatching {
            val tracks = response
                .optJSONObject("captions")
                ?.optJSONObject("playerCaptionsTracklistRenderer")
                ?.optJSONArray("captionTracks")
                ?: return emptyList()
            (0 until tracks.length()).mapNotNull { i ->
                val track = tracks.getJSONObject(i)
                val baseUrl = track.optString("baseUrl").takeIf { it.startsWith("http") } ?: return@mapNotNull null
                val lang = track.optString("languageCode").ifBlank { return@mapNotNull null }
                val isAuto = track.optString("kind") == "asr"
                val name = runCatching {
                    Locale.forLanguageTag(lang).getDisplayLanguage(Locale.ENGLISH)
                        .replaceFirstChar { it.uppercase() }
                }.getOrElse { lang }
                SubtitleInfo(
                    languageTag = lang,
                    label = if (isAuto) "$name (auto)" else name,
                    url = "$baseUrl&fmt=vtt",
                    mimeType = "text/vtt",
                    isAuto = isAuto
                )
            }.distinctBy { it.languageTag }
        }.onFailure { Log.w("TrailerResolver", "subtitles: ${it.message}") }
            .getOrElse { emptyList() }
    }
}
