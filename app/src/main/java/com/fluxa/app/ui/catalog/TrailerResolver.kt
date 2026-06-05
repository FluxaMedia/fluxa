package com.fluxa.app.ui.catalog

import android.util.Log
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import okhttp3.MediaType.Companion.toMediaTypeOrNull
import okhttp3.OkHttpClient
import okhttp3.RequestBody.Companion.toRequestBody
import org.schabi.newpipe.extractor.MediaFormat
import org.schabi.newpipe.extractor.NewPipe
import org.schabi.newpipe.extractor.downloader.Downloader
import org.schabi.newpipe.extractor.downloader.Request
import org.schabi.newpipe.extractor.downloader.Response
import org.schabi.newpipe.extractor.exceptions.GeographicRestrictionException
import org.schabi.newpipe.extractor.stream.DeliveryMethod
import java.util.Locale
import java.util.concurrent.ConcurrentHashMap
import java.util.concurrent.TimeUnit

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

    private val downloader = object : Downloader() {
        override fun execute(request: Request): Response {
            val builder = okhttp3.Request.Builder().url(request.url())
            request.headers().forEach { (key, values) ->
                values.forEach { builder.header(key, it) }
            }
            val body = request.dataToSend()
            if (body != null) {
                builder.post(body.toRequestBody("application/json; charset=utf-8".toMediaTypeOrNull()))
            }
            val res = httpClient.newCall(builder.build()).execute()
            return Response(res.code, res.message, res.headers.toMultimap(), res.body.string(), request.url())
        }
    }

    @Volatile private var initialized = false

    // Cache resolved results — avoids repeated network calls for the same trailer
    private val cache = ConcurrentHashMap<String, TrailerResolveResult>(16)

    private fun ensureInit() {
        if (!initialized) synchronized(this) {
            if (!initialized) {
                NewPipe.init(downloader)
                initialized = true
            }
        }
    }

    private fun videoId(youtubeUrl: String): String? =
        Regex("(?:v=|youtu\\.be/|embed/)([A-Za-z0-9_-]{11})").find(youtubeUrl)?.groupValues?.get(1)

    suspend fun resolve(youtubeUrl: String): TrailerResolveResult = withContext(Dispatchers.IO) {
        val cacheKey = videoId(youtubeUrl) ?: youtubeUrl
        cache[cacheKey]?.let { return@withContext it }

        val result = doExtract(youtubeUrl)
        if (result !is TrailerResolveResult.GeoBlocked) {
            if (result is TrailerResolveResult.Ok) cache[cacheKey] = result
            return@withContext result
        }

        // Embed URL uses TVHTML5_SIMPLY_EMBEDDED_PLAYER client — bypasses geo-block for most videos
        val embedUrl = videoId(youtubeUrl)?.let { "https://www.youtube.com/embed/$it" }
            ?: return@withContext TrailerResolveResult.GeoBlocked
        Log.d("TrailerResolver", "geo-blocked, retrying via embed: $embedUrl")
        val embedResult = doExtract(embedUrl)
        if (embedResult is TrailerResolveResult.Ok) cache[cacheKey] = embedResult
        embedResult
    }

    private fun doExtract(url: String): TrailerResolveResult {
        return try {
            ensureInit()
            val extractor = NewPipe.getServiceByUrl(url).getStreamExtractor(url)
            extractor.fetchPage()

            // Prefer HLS manifest (adaptive, supports 1080p+) — ExoPlayer selects best quality
            val hlsUrl = runCatching { extractor.hlsUrl }.getOrNull()?.takeIf { it.startsWith("http") }
            if (hlsUrl != null) {
                val subtitles = extractSubtitles(extractor)
                return TrailerResolveResult.Ok(TrailerResult(hlsUrl, subtitles))
            }

            // Prefer DASH manifest (1080p/4K) if HLS unavailable
            val dashUrl = runCatching { extractor.dashMpdUrl }.getOrNull()?.takeIf { it.startsWith("http") }
            if (dashUrl != null) {
                val subtitles = extractSubtitles(extractor)
                return TrailerResolveResult.Ok(TrailerResult(dashUrl, subtitles))
            }

            // Fall back: highest-resolution progressive mp4 (max 720p on YouTube)
            val progressive = runCatching { extractor.videoStreams }.getOrElse { emptyList() }
                .filter { it.deliveryMethod == DeliveryMethod.PROGRESSIVE_HTTP }
            val streamUrl = progressive
                .filter { it.format == MediaFormat.MPEG_4 }
                .ifEmpty { progressive }
                .maxByOrNull { it.getResolution().removeSuffix("p").toIntOrNull() ?: 0 }
                ?.content?.takeIf { it.startsWith("http") }
                ?: run {
                    val any = runCatching { extractor.videoStreams }.getOrElse { emptyList() }
                    any.filter { it.format == MediaFormat.MPEG_4 }.ifEmpty { any }
                        .maxByOrNull { it.getResolution().removeSuffix("p").toIntOrNull() ?: 0 }
                        ?.content?.takeIf { it.startsWith("http") }
                }

            if (streamUrl == null) {
                Log.w("TrailerResolver", "No playable stream found for $url")
                return TrailerResolveResult.Failed
            }

            TrailerResolveResult.Ok(TrailerResult(streamUrl, extractSubtitles(extractor)))
        } catch (e: GeographicRestrictionException) {
            TrailerResolveResult.GeoBlocked
        } catch (e: Exception) {
            Log.e("TrailerResolver", "doExtract($url) → ${e.javaClass.simpleName}: ${e.message}")
            TrailerResolveResult.Failed
        }
    }

    private fun extractSubtitles(extractor: org.schabi.newpipe.extractor.stream.StreamExtractor): List<SubtitleInfo> {
        return runCatching {
            extractor.subtitlesDefault
                .filter { it.content.startsWith("http") }
                .mapNotNull { sub ->
                    val mime = when (sub.format) {
                        MediaFormat.VTT -> "text/vtt"
                        MediaFormat.TTML -> "application/ttml+xml"
                        else -> return@mapNotNull null
                    }
                    val lang = sub.languageTag.orEmpty().ifBlank { return@mapNotNull null }
                    val displayName = runCatching {
                        Locale.forLanguageTag(lang).getDisplayLanguage(Locale.ENGLISH)
                            .replaceFirstChar { it.uppercase() }
                    }.getOrElse { lang }
                    SubtitleInfo(
                        languageTag = lang,
                        label = if (sub.isAutoGenerated) "$displayName (auto)" else displayName,
                        url = sub.content,
                        mimeType = mime,
                        isAuto = sub.isAutoGenerated
                    )
                }
                .distinctBy { it.languageTag }
        }.onFailure { Log.w("TrailerResolver", "subtitles: ${it.message}") }
            .getOrElse { emptyList() }
    }
}
