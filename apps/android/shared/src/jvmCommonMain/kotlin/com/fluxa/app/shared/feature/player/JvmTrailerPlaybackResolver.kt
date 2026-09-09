package com.fluxa.app.shared.feature.player

import com.fluxa.app.core.rust.NativeHeadlessEngineResult
import com.fluxa.app.core.rust.FluxaCoreNative
import com.fluxa.app.data.remote.DetailTrailer
import java.util.UUID

object JvmTrailerPlaybackResolver {
    suspend fun resolvePlayable(
        trailers: List<DetailTrailer>,
        maxHeight: Int,
        preferDirect: Boolean,
        dispatchHeadless: suspend (Any) -> NativeHeadlessEngineResult,
    ): TrailerResolveResult? {
        val direct = FluxaCoreNative.trailerDirectSelection(trailers, maxHeight)
        if (preferDirect && direct != null) {
            return directResult(direct)
        }

        val youtubeVideoIds = FluxaCoreNative.trailerYoutubeVideoIds(trailers.map(DetailTrailer::url))
        for (videoId in youtubeVideoIds) {
            val result = resolveYoutube(videoId, maxHeight, dispatchHeadless)
            if (result is TrailerResolveResult.Ok) return result
        }

        return direct?.let(::directResult)
    }

    fun selectBestDirectTrailerUrl(
        trailers: List<DetailTrailer>,
        maxHeight: Int = Int.MAX_VALUE,
    ): String? = FluxaCoreNative.trailerDirectSelection(trailers, maxHeight)?.first

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

    private fun directResult(selection: Pair<String, String?>): TrailerResolveResult.Ok = TrailerResolveResult.Ok(
        TrailerResult(
            streamUrl = selection.first,
            audioUrl = null,
            subtitles = emptyList(),
            streamMimeType = selection.second,
        ),
    )
}
