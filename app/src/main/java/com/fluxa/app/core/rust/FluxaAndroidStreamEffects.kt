@file:androidx.annotation.OptIn(androidx.media3.common.util.UnstableApi::class)

package com.fluxa.app.core.rust

import com.fluxa.app.data.local.*
import com.fluxa.app.data.remote.Meta
import com.fluxa.app.data.remote.Stream
import com.fluxa.app.data.remote.Video
import com.fluxa.app.domain.discovery.StreamDiscoveryRequest
import com.fluxa.app.player.TorrentStreamManager
import com.fluxa.app.player.TorrentStreamResult
import kotlin.coroutines.resume
import kotlinx.coroutines.suspendCancellableCoroutine

internal suspend fun FluxaAndroidHeadlessEnvironment.loadStreams(effect: NativeHeadlessEffect): HeadlessEffectCompletion {
    val payload = effect.payload
    if (payload.boolean("useInitialStreams")) {
        return ok(effect, payload.list("initialStreams"))
    }
    val id = payload.string("id")
    if (id.startsWith("cs3:")) {
        return ok(effect, loadCsNativeStreams(id))
    }
    val profile = payload.profile()
    val addons = addonRepository.getUserAddons(profile?.authKey.orEmpty(), profile?.safeLocalAddons.orEmpty())
    val streams = streamDiscovery.discover(
        StreamDiscoveryRequest(
            addons = addons,
            type = payload.string("contentType"),
            id = id,
            language = profile?.safeLanguage ?: "en",
            preferFastStart = true,
            cs3PluginApis = pluginManager.loadedApis.value,
            cs3SearchQuery = payload.stringOrNull("title"),
            cs3OriginalName = payload.stringOrNull("originalName"),
            cs3Year = payload.number("year")?.toInt()
        )
    )
    // Pre-warm only the top-ranked torrent stream. Pre-warming the whole
    // list splits rqbit's peer slots across every magnet and slows the
    // one the user actually picks. Other streams are added on demand by
    // startStream → stream_fname.
    val torrentManager = TorrentStreamManager.getInstance(context)
    val contentId = id
    streams
        .firstOrNull { it.infoHash != null || FluxaCoreNative.isTorrentPlaybackUrl(it.playableUrl) }
        ?.let { topStream ->
            topStream.playableUrl?.takeIf(String::isNotBlank)?.let { url ->
                torrentManager.preWarm(url, contentId, topStream.fileIdx)
            }
        }
    return ok(effect, streams)
}

internal fun FluxaAndroidHeadlessEnvironment.enqueueTraktScrobble(effect: NativeHeadlessEffect): HeadlessEffectCompletion {
    val payload = effect.payload
    val profile = payload.profile() ?: return error(effect, "missing_profile")
    val token = payload.string("token")
    if (token.isBlank()) return ok(effect, mapOf("queued" to false))
    val queued = playbackSyncCoordinator.scheduleTraktScrobble(
        profile = profile,
        mediaType = payload.string("metaType"),
        mediaId = payload.string("itemId"),
        progress = (payload.number("progress")?.toFloat() ?: 0f).coerceIn(0f, 100f),
        action = payload.string("actionName")
    )
    return ok(effect, mapOf("queued" to queued))
}

internal suspend fun FluxaAndroidHeadlessEnvironment.startTorrentStream(effect: NativeHeadlessEffect): HeadlessEffectCompletion {
    val payload = effect.payload
    val url = payload.string("url")
    val stream = payload.objectValue("stream")?.let { gson.fromJson(gson.toJsonTree(it), Stream::class.java) }
    val activeProfile = profileManager.getLastActiveProfileId()
        ?.let { id -> profileManager.getProfiles().firstOrNull { it.id == id } }
    val result = suspendCancellableCoroutine<TorrentStreamResult> { continuation ->
        TorrentStreamManager.getInstance(context).startStream(
            link = url,
            videoId = payload.string("currentVideoId", payload.string("title", "Fluxa")),
            playbackTitle = payload.string("title", "Fluxa"),
            fileIdx = payload.number("fileIdx")?.toInt() ?: stream?.fileIdx,
            preferredFilename = payload.stringOrNull("preferredFilename") ?: stream?.effectiveFilename,
            sources = payload.list("sources").mapNotNull { it?.toString() }.ifEmpty { stream?.sources.orEmpty() },
            fileSizeBytes = stream?.effectiveVideoSize ?: stream?.videoSize ?: 0L,
            durationMs = payload.number("durationMs")?.toLong() ?: 0L,
            wifiOnly = activeProfile?.safeTorrentWifiOnly == true
        ) { torrentResult ->
            if (continuation.isActive) continuation.resume(torrentResult)
        }
    }
    return when (result) {
        is TorrentStreamResult.Success -> ok(effect, mapOf("url" to result.url))
        is TorrentStreamResult.Error -> error(effect, result.message)
    }
}

internal fun FluxaAndroidHeadlessEnvironment.stopTorrent(effect: NativeHeadlessEffect): HeadlessEffectCompletion {
    TorrentStreamManager.getInstance(context).stop()
    return ok(effect, emptyMap<String, Any?>())
}

internal suspend fun FluxaAndroidHeadlessEnvironment.clearPlaybackProgress(effect: NativeHeadlessEffect): HeadlessEffectCompletion {
    val payload = effect.payload
    val profile = payload.profile()
    val meta = gson.fromJson(gson.toJsonTree(payload["meta"]), Meta::class.java)
    watchlistManager.clearPlaybackProgress(meta.id)
    if (profile != null) {
        playbackSyncCoordinator.clearProgress(profile, meta)
    }
    return ok(effect, emptyMap<String, Any?>())
}

internal suspend fun FluxaAndroidHeadlessEnvironment.writePlaybackProgress(effect: NativeHeadlessEffect): HeadlessEffectCompletion {
    val profile = effect.payload.profile()
    val progress = effect.payload.objectValue("progress").orEmpty()
    val meta = gson.fromJson(gson.toJsonTree(progress["meta"]), Meta::class.java)
    val timeOffset = progress.number("timeOffset")?.toLong() ?: 0L
    val duration = progress.number("duration")?.toLong() ?: 0L
    watchlistManager.savePlaybackProgress(
        meta = meta,
        timeOffset = timeOffset,
        duration = duration,
        lastVideoId = progress.stringOrNull("lastVideoId"),
        lastStreamIndex = progress.number("lastStreamIndex")?.toInt(),
        lastEpisodeName = progress.stringOrNull("lastEpisodeName"),
        lastStreamUrl = progress.stringOrNull("lastStreamUrl"),
        lastStreamTitle = progress.stringOrNull("lastStreamTitle"),
        lastBingeGroup = progress.stringOrNull("lastBingeGroup"),
        lastAudioLanguage = progress.stringOrNull("lastAudioLanguage"),
        lastSubtitleLanguage = progress.stringOrNull("lastSubtitleLanguage")
    )
    if (profile != null) {
        playbackSyncCoordinator.scheduleProgress(
            profile = profile,
            meta = meta,
            videoId = progress.stringOrNull("lastVideoId"),
            positionMs = timeOffset,
            durationMs = duration,
            allowPauseScrobble = effect.payload.boolean("scrobbleTraktPause", true)
        )
    }
    return ok(effect, emptyMap<String, Any?>())
}

internal suspend fun FluxaAndroidHeadlessEnvironment.syncWatchedState(effect: NativeHeadlessEffect): HeadlessEffectCompletion {
    val payload = effect.payload
    val profile = payload.profile()
    val meta = payload.objectValue("meta")?.let { gson.fromJson(gson.toJsonTree(it), Meta::class.java) }
        ?: return error(effect, "missing_meta")
    val episodes = payload.list("episodes").mapNotNull { raw ->
        runCatching { gson.fromJson(gson.toJsonTree(raw), Video::class.java) }.getOrNull()
    }
    if (profile != null) {
        playbackSyncCoordinator.pushWatched(
            profile = profile,
            meta = meta,
            episodes = episodes,
            watched = payload.boolean("watched", true)
        )
    }
    return ok(effect, mapOf("synced" to true))
}
