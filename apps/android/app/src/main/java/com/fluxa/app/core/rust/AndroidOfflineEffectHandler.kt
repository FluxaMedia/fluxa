package com.fluxa.app.core.rust

import android.content.Context
import com.fluxa.app.core.rust.FluxaCoreNative
import com.fluxa.app.data.local.OfflineDownloadManager
import com.fluxa.app.data.local.OfflineSubtitleOption
import com.fluxa.app.data.remote.Meta
import com.fluxa.app.data.remote.Stream
import com.fluxa.app.data.remote.Video
import com.fluxa.app.data.stream.effectiveFilename
import com.fluxa.app.data.stream.effectiveVideoSize
import com.fluxa.app.data.stream.playableUrl
import com.fluxa.app.data.stream.rawDisplayTitle
import com.fluxa.app.player.TorrentStreamManager
import com.fluxa.app.player.TorrentStreamResult
import com.google.gson.Gson
import kotlin.coroutines.resume
import kotlinx.coroutines.suspendCancellableCoroutine

internal class AndroidOfflineEffectHandler(
    private val context: Context,
    private val gson: Gson
) {
    suspend fun enqueue(effect: NativeHeadlessEffect): HeadlessEffectCompletion {
        val payload = effect.payload
        val stream = gson.fromJson(gson.toJsonTree(payload["stream"]), Stream::class.java)
        val isTorrent = FluxaCoreNative.isTorrentPlaybackUrl(stream.playableUrl)
        val torrentPlaybackUrl = resolveTorrentPlaybackUrl(stream)
        if (isTorrent && torrentPlaybackUrl == null) {
            return HeadlessEffectCompletion(
                effectId = effect.id,
                status = "error",
                error = mapOf("code" to "torrent_source_unavailable")
            )
        }
        val result = OfflineDownloadManager.getInstance(context).enqueue(
            profileId = payload.stringOrNull("profileId"),
            meta = gson.fromJson(gson.toJsonTree(payload["meta"]), Meta::class.java),
            video = payload.objectValue("video")?.let { gson.fromJson(gson.toJsonTree(it), Video::class.java) },
            videoId = payload.stringOrNull("videoId"),
            stream = stream,
            subtitle = payload.objectValue("subtitle")?.let {
                gson.fromJson(gson.toJsonTree(it), OfflineSubtitleOption::class.java)
            },
            profileLanguage = payload.stringOrNull("language"),
            playbackUrlOverride = torrentPlaybackUrl
        )
        return result.fold(
            onSuccess = { HeadlessEffectCompletion(effect.id, "ok", value = it) },
            onFailure = {
                HeadlessEffectCompletion(
                    effectId = effect.id,
                    status = "error",
                    error = mapOf("code" to (it.message ?: "offline_download_failed"))
                )
            }
        )
    }

    private suspend fun resolveTorrentPlaybackUrl(stream: Stream): String? {
        val sourceUrl = stream.playableUrl ?: return null
        if (!FluxaCoreNative.isTorrentPlaybackUrl(sourceUrl)) return null
        return suspendCancellableCoroutine { continuation ->
            TorrentStreamManager.getInstance(context).startStream(
                link = sourceUrl,
                videoId = stream.rawDisplayTitle,
                playbackTitle = stream.rawDisplayTitle,
                fileIdx = stream.fileIdx,
                preferredFilename = stream.effectiveFilename,
                sources = stream.sources,
                fileSizeBytes = stream.effectiveVideoSize ?: stream.videoSize ?: 0L
            ) { result ->
                if (!continuation.isActive) return@startStream
                continuation.resume((result as? TorrentStreamResult.Success)?.url)
            }
        }
    }
}
