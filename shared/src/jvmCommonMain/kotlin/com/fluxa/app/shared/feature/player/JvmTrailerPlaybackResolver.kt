package com.fluxa.app.shared.feature.player

import com.fluxa.app.core.rust.NativeHeadlessEngineResult
import com.fluxa.app.data.remote.DetailTrailer
import com.fluxa.app.ui.catalog.extractYoutubeVideoId
import com.fluxa.app.ui.catalog.isDirectVideoPreviewUrl
import java.util.UUID

/**
 * JVM trailer resolution shared by Android and Desktop.
 *
 * The UI/platform decides the preferred maximum height. Android mobile keeps its
 * lower bandwidth ceiling while Desktop/TV can request 1080p. Direct addon
 * trailers (Trailerio, etc.) can be preferred without changing the underlying
 * Stremio trailer model.
 */
object JvmTrailerPlaybackResolver {
    private val resolutionPattern = Regex("""(?i)(?<!\d)(2160|1440|1080|720|576|540|480|360)\s*p(?!\d)""")

    suspend fun resolvePlayable(
        trailers: List<DetailTrailer>,
        maxHeight: Int,
        preferDirect: Boolean,
        dispatchHeadless: suspend (Any) -> NativeHeadlessEngineResult,
    ): TrailerResolveResult? {
        val directUrl = selectBestDirectTrailerUrl(trailers, maxHeight)
        if (preferDirect && directUrl != null) {
            return directResult(directUrl)
        }

        val youtubeVideoIds = trailers.asSequence()
            .mapNotNull { it.url.extractYoutubeVideoId() }
            .distinct()
            .toList()
        for (videoId in youtubeVideoIds) {
            val result = resolveYoutube(videoId, maxHeight, dispatchHeadless)
            if (result is TrailerResolveResult.Ok) return result
        }

        return directUrl?.let { directResult(it) }
    }

    fun selectBestDirectTrailerUrl(
        trailers: List<DetailTrailer>,
        maxHeight: Int = Int.MAX_VALUE,
    ): String? {
        var bestDeclaredUrl: String? = null
        var bestDeclaredQuality = -1
        var firstUnknownQualityUrl: String? = null

        for (trailer in trailers) {
            if (!trailer.url.isDirectVideoPreviewUrl()) continue
            val quality = directTrailerQuality(trailer)
            if (quality <= 0) {
                if (firstUnknownQualityUrl == null) firstUnknownQualityUrl = trailer.url
                continue
            }
            if (quality > maxHeight) continue
            // Strict comparison deliberately preserves addon order for equal quality.
            if (quality > bestDeclaredQuality) {
                bestDeclaredUrl = trailer.url
                bestDeclaredQuality = quality
            }
        }

        return bestDeclaredUrl ?: firstUnknownQualityUrl
    }

    suspend fun resolveYoutube(
        videoId: String,
        maxHeight: Int,
        dispatchHeadless: suspend (Any) -> NativeHeadlessEngineResult,
    ): TrailerResolveResult {
        val requestId = UUID.randomUUID().toString()
        val result = dispatchHeadless(
            mapOf(
                "type" to "trailerResolveRequested",
                "requestId" to requestId,
                "videoId" to videoId,
                "maxHeight" to maxHeight,
            ),
        )
        val resolution = (result.state["trailer"] as? Map<*, *>)
            ?.get("resolutions") as? Map<*, *>
            ?: return TrailerResolveResult.Failed
        val entry = resolution[requestId] as? Map<*, *> ?: return TrailerResolveResult.Failed
        if (entry["status"] != "ok") return TrailerResolveResult.Failed
        val streamUrl = entry["streamUrl"] as? String ?: return TrailerResolveResult.Failed
        val subtitles = (entry["subtitles"] as? List<*>).orEmpty().mapNotNull { raw ->
            val track = raw as? Map<*, *> ?: return@mapNotNull null
            TrailerSubtitle(
                languageTag = track["languageTag"] as? String ?: "und",
                label = track["label"] as? String ?: "",
                url = track["url"] as? String ?: return@mapNotNull null,
                mimeType = track["mimeType"] as? String ?: "text/vtt",
                isAuto = track["isAuto"] as? Boolean ?: false,
            )
        }
        return TrailerResolveResult.Ok(
            TrailerResult(
                streamUrl = streamUrl,
                audioUrl = entry["audioUrl"] as? String,
                subtitles = subtitles,
                streamMimeType = null,
            ),
        )
    }

    private fun directResult(url: String): TrailerResolveResult.Ok = TrailerResolveResult.Ok(
        TrailerResult(
            streamUrl = url,
            audioUrl = null,
            subtitles = emptyList(),
            streamMimeType = when {
                url.contains(".m3u8", ignoreCase = true) -> "application/x-mpegURL"
                url.contains(".mpd", ignoreCase = true) -> "application/dash+xml"
                url.contains(".webm", ignoreCase = true) -> "video/webm"
                else -> "video/mp4"
            },
        ),
    )

    private fun directTrailerQuality(trailer: DetailTrailer): Int {
        val description = "${trailer.title} ${trailer.source} ${trailer.url}"
        if (Regex("""(?i)\b(?:4k|uhd)\b""").containsMatchIn(description)) return 2160
        return resolutionPattern.find(description)
            ?.groupValues
            ?.getOrNull(1)
            ?.toIntOrNull()
            ?: 0
    }
}
