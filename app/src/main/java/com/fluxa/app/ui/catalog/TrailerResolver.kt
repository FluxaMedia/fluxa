package com.fluxa.app.ui.catalog

import android.util.Log
import androidx.media3.common.MimeTypes
import androidx.media3.datasource.DataSource
import androidx.media3.datasource.DataSpec
import androidx.media3.datasource.TransferListener
import androidx.media3.datasource.okhttp.OkHttpDataSource
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import okhttp3.MediaType.Companion.toMediaType
import okhttp3.OkHttpClient
import okhttp3.Request
import okhttp3.RequestBody.Companion.toRequestBody
import okhttp3.HttpUrl.Companion.toHttpUrlOrNull
import okhttp3.Dns
import org.json.JSONObject
import java.net.Inet4Address
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
    val audioUrl: String?,
    val subtitles: List<SubtitleInfo>,
    val streamMimeType: String?
)

internal sealed interface TrailerResolveResult {
    data class Ok(val data: TrailerResult) : TrailerResolveResult
    data object GeoBlocked : TrailerResolveResult
    data object Failed : TrailerResolveResult
}

internal object TrailerResolver {

    private val memCache = ConcurrentHashMap<String, TrailerResolveResult>(16)
    private val streamMimeTypes = ConcurrentHashMap<String, String>(16)
    private val httpClient = OkHttpClient.Builder()
        .dns { hostname ->
            val addresses = Dns.SYSTEM.lookup(hostname)
            addresses.filterIsInstance<Inet4Address>().ifEmpty { addresses }
        }
        .build()

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
        if (result is TrailerResolveResult.Ok) {
            memCache[id] = result
            result.data.streamMimeType?.let { streamMimeTypes[result.data.streamUrl] = it }
        }
        result
    }

    fun streamMimeType(url: String): String? =
        streamMimeTypes[url] ?: if (url.contains(".m3u8", ignoreCase = true)) MimeTypes.APPLICATION_M3U8 else null

    fun mediaDataSourceFactory(): DataSource.Factory {
        val upstream = OkHttpDataSource.Factory(httpClient)
            .setUserAgent("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36")
        return DataSource.Factory { TrailerRangeDataSource(upstream.createDataSource()) }
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
            val hlsUrl = streaming.optString("hlsManifestUrl").takeIf { it.isNotBlank() }
                ?.let(::bestHlsVariant)
            val adaptivePair = streaming.optJSONArray("adaptiveFormats")?.let(::bestAdaptivePair)
            val streamUrl = hlsUrl
                ?: adaptivePair?.first
                ?: streaming.optJSONArray("formats")?.let { formats ->
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
            return TrailerResolveResult.Ok(
                TrailerResult(
                    streamUrl = streamUrl,
                    audioUrl = if (hlsUrl == null) adaptivePair?.second else null,
                    subtitles = subtitles,
                    streamMimeType = if (hlsUrl != null) MimeTypes.APPLICATION_M3U8 else MimeTypes.VIDEO_MP4
                )
            )
        }
    }

    private fun bestAdaptivePair(formats: org.json.JSONArray): Pair<String, String>? {
        val entries = (0 until formats.length()).mapNotNull(formats::optJSONObject)
        val video = entries
            .asSequence()
            .filter { it.optString("url").isNotBlank() }
            .filter { it.optString("mimeType").startsWith("video/mp4; codecs=\"avc1") }
            .maxWithOrNull(compareBy<JSONObject> { it.optInt("height") }.thenBy { it.optInt("bitrate") })
            ?: return null
        val audio = entries
            .asSequence()
            .filter { it.optString("url").isNotBlank() }
            .filter { it.optString("mimeType").startsWith("audio/mp4") }
            .maxByOrNull { it.optInt("bitrate") }
            ?: return null
        return video.optString("url") to audio.optString("url")
    }

    private fun bestHlsVariant(masterUrl: String): String {
        val request = Request.Builder()
            .url(masterUrl)
            .header("Accept", "application/vnd.apple.mpegurl, application/x-mpegURL, */*")
            .build()
        val manifest = runCatching {
            httpClient.newCall(request).execute().use { response ->
                if (response.isSuccessful) response.body.string() else null
            }
        }.getOrNull() ?: return masterUrl
        var attributes: String? = null
        var best: Triple<Long, Long, String>? = null
        manifest.lineSequence().map(String::trim).filter(String::isNotBlank).forEach { line ->
            if (line.startsWith("#EXT-X-STREAM-INF:")) {
                attributes = line.removePrefix("#EXT-X-STREAM-INF:")
            } else if (!line.startsWith("#")) {
                val streamAttributes = attributes ?: return@forEach
                attributes = null
                if (streamAttributes.contains("AUDIO=")) return@forEach
                val resolution = Regex("RESOLUTION=(\\d+)x(\\d+)").find(streamAttributes)
                val pixels = resolution?.let { it.groupValues[1].toLong() * it.groupValues[2].toLong() } ?: 0L
                val bandwidth = Regex("BANDWIDTH=(\\d+)").find(streamAttributes)?.groupValues?.get(1)?.toLongOrNull() ?: 0L
                val url = masterUrl.toHttpUrlOrNull()?.resolve(line)?.toString() ?: return@forEach
                if (best == null || pixels > best!!.first || (pixels == best!!.first && bandwidth > best!!.second)) {
                    best = Triple(pixels, bandwidth, url)
                }
            }
        }
        return best?.third ?: masterUrl
    }

    private fun parseResult(response: JSONObject): TrailerResolveResult = when (response.optString("status")) {
        "ok" -> TrailerResolveResult.Ok(
            TrailerResult(
                streamUrl = response.getString("streamUrl"),
                audioUrl = response.optString("audioUrl").takeIf { it.isNotBlank() },
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
                } ?: emptyList(),
                streamMimeType = response.optString("streamMimeType").takeIf { it.isNotBlank() }
            )
        )
        "geo_blocked" -> TrailerResolveResult.GeoBlocked
        else -> TrailerResolveResult.Failed
    }
}

private class TrailerRangeDataSource(
    private val delegate: DataSource
) : DataSource {
    override fun addTransferListener(transferListener: TransferListener) {
        delegate.addTransferListener(transferListener)
    }

    override fun open(dataSpec: DataSpec): Long {
        val headers = dataSpec.httpRequestHeaders
        val request = if (headers.keys.any { it.equals("Range", ignoreCase = true) }) {
            dataSpec
        } else {
            dataSpec.withRequestHeaders(headers + ("Range" to "bytes=0-"))
        }
        return delegate.open(request)
    }

    override fun read(buffer: ByteArray, offset: Int, length: Int): Int = delegate.read(buffer, offset, length)

    override fun getUri() = delegate.uri

    override fun getResponseHeaders() = delegate.responseHeaders

    override fun close() = delegate.close()
}
