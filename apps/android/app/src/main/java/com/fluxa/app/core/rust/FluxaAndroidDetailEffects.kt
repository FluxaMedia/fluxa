@file:androidx.annotation.OptIn(androidx.media3.common.util.UnstableApi::class)

package com.fluxa.app.core.rust

import android.util.Log
import com.fluxa.app.core.StremioId
import com.fluxa.app.data.local.*
import com.fluxa.app.data.remote.IntroTimestamps
import com.fluxa.app.data.remote.Meta
import com.fluxa.app.data.remote.MetaDetail
import com.fluxa.app.data.remote.Stream
import com.fluxa.app.data.remote.Video
import com.fluxa.app.domain.discovery.StreamDiscoveryRequest
import com.fluxa.app.player.MediaPlayerController
import com.fluxa.app.player.TorrentStreamManager
import com.fluxa.app.common.ReleaseDateUtils
import com.fluxa.app.ui.catalog.DirectPlaybackTarget
import android.net.Uri
import kotlinx.coroutines.launch
import kotlinx.coroutines.withTimeoutOrNull

internal suspend fun FluxaAndroidHeadlessEnvironment.fetchMetaDetail(effect: NativeHeadlessEffect): HeadlessEffectCompletion {
    val payload = effect.payload
    val id = payload.string("id")

    val isLookup = effect.type == "fetchMetaDetailLookup"

    if (id.startsWith("cs3:")) {
        val csDetail = loadCsNativeMetaDetail(id)
        if (csDetail != null) return ok(effect, if (isLookup) csDetail else mapOf("meta" to csDetail))
    }

    val profile = payload.profile()
    val language = payload.string("language", profile?.safeLanguage ?: "en")
    val contentType = payload.string("contentType")
    val preferredUrl = payload.stringOrNull("sourceAddonTransportUrl")
    val preferredCatalogType = payload.stringOrNull("sourceAddonCatalogType")
    Log.d("MetaFetch", "fetchMetaDetail: type=$contentType id=${id.take(40)} preferredUrl=$preferredUrl preferredCatalogType=$preferredCatalogType localAddons=${profile?.safeLocalAddons?.size}")
    val detail = repository.getMetaDetail(
        type = contentType,
        id = id,
        language = language,
        authKey = profile?.authKey.orEmpty(),
        localAddons = profile?.safeLocalAddons.orEmpty(),
        useConfiguredAddons = true,
        preferredAddonTransportUrl = preferredUrl,
        preferredAddonCatalogType = preferredCatalogType
    )
    val detailSummary = when {
        detail == null -> "NULL"
        detail.type.equals("series", ignoreCase = true) || detail.type.equals("tv", ignoreCase = true) ->
            "SUCCESS name=${detail.name} episodes=${detail.videos?.size ?: 0}"
        else -> "SUCCESS name=${detail.name}"
    }
    Log.d("MetaFetch", "fetchMetaDetail result: $detailSummary")
    val enriched = if (detail != null && profile?.safeTmdbApiKey?.isNotBlank() == true) {
        repository.enrichDetailWithTmdb(detail, profile.safeTmdbApiKey, profile, language)
    } else {
        detail
    }
    return ok(effect, if (isLookup) enriched else mapOf("meta" to enriched))
}

internal suspend fun FluxaAndroidHeadlessEnvironment.readPlaybackProgress(effect: NativeHeadlessEffect): HeadlessEffectCompletion {
    return ok(effect, watchlistManager.getPlaybackProgress(effect.payload.string("id")))
}

internal suspend fun FluxaAndroidHeadlessEnvironment.readDetailLocalState(effect: NativeHeadlessEffect): HeadlessEffectCompletion {
    val payload = effect.payload
    val profile = payload.profile()
    profile?.id?.let(watchlistManager::setActiveProfile)
    val primaryId = payload.string("primaryId")
    val fallbackId = payload.stringOrNull("fallbackId")
    val userAddons = repository.getUserAddons(profile?.authKey.orEmpty(), profile?.safeLocalAddons.orEmpty())
    val providerPlan = FluxaCoreNative.headlessProviderAvailability(
        addons = userAddons,
        pluginNames = pluginManager.loadedApis.value.map { it.name }
    )
    val savedPlayback = watchlistManager.getPlaybackProgress(primaryId)
        ?: fallbackId?.let { watchlistManager.getPlaybackProgress(it) }
    val localWatched = if (payload.string("contentType") == "series") {
        watchlistManager.getLocalWatchedVideoIds(primaryId)
            .ifEmpty { fallbackId?.let { watchlistManager.getLocalWatchedVideoIds(it) }.orEmpty() }
    } else {
        emptySet()
    }
    return ok(
        effect,
        mapOf(
            "savedPlayback" to savedPlayback,
            "localWatchedVideoIds" to localWatched.toList(),
            "isInWatchlist" to (watchlistManager.isInWatchlist(primaryId) || fallbackId?.let { watchlistManager.isInWatchlist(it) } == true),
            "feedback" to (watchlistManager.getFeedback(primaryId) ?: fallbackId?.let { watchlistManager.getFeedback(it) }),
            "hasStreamProviders" to providerPlan.hasStreamProviders,
            "userAddons" to userAddons
        )
    )
}

internal suspend fun FluxaAndroidHeadlessEnvironment.fetchDetailSecondary(effect: NativeHeadlessEffect): HeadlessEffectCompletion {
    val payload = effect.payload
    val profile = payload.profile()
    val type = payload.string("contentType")
    val id = payload.string("id")
    val language = payload.string("language", profile?.safeLanguage ?: "en")
    val watchedIds = if (type == "series" && !profile?.authKey.isNullOrBlank()) {
        repository.getWatchedVideoIds(profile.authKey, id)
    } else {
        emptySet()
    }
    val apiKey = profile?.safeTmdbApiKey.orEmpty()
    val similarItems = if (isTmdbContentId(id) && apiKey.isNotBlank() && (profile?.safeTmdbSimilarResultsEnabled == true || profile?.safeTmdbRecommendationsEnabled == true)) {
        repository.getSimilar(type, id, language)
    } else {
        emptyList()
    }
    val trailers = if (apiKey.isNotBlank() && profile?.safeTmdbTrailersEnabled == true) {
        repository.getTmdbTrailers(type, id, language, apiKey)
    } else {
        emptyList()
    }
    val mdblistRatings = profile?.mdblistApiKey
        ?.takeIf(String::isNotBlank)
        ?.let { mdblistRatingsClient.fetch(type, id, it) }
        .orEmpty()
    return ok(
        effect,
        mapOf(
            "watchedVideoIds" to watchedIds.toList(),
            "similarItems" to similarItems,
            "trailers" to trailers,
            "mdblistRatings" to mdblistRatings
        )
    )
}

internal suspend fun FluxaAndroidHeadlessEnvironment.prefetchDetailStreams(effect: NativeHeadlessEffect): HeadlessEffectCompletion {
    val payload = effect.payload
    val profile = payload.profile()
    val addons = repository.getUserAddons(profile?.authKey.orEmpty(), profile?.safeLocalAddons.orEmpty())
    val preFetched = streamDiscovery.prefetch(
        StreamDiscoveryRequest(
            addons = addons,
            type = payload.string("contentType"),
            id = payload.string("streamLookupId"),
            language = payload.string("language", profile?.safeLanguage ?: "en"),
            preferFastStart = true,
            cs3PluginApis = pluginManager.loadedApis.value,
            cs3SearchQuery = payload.stringOrNull("title"),
            cs3OriginalName = payload.stringOrNull("originalName"),
            cs3Year = payload.number("year")?.toInt()
        )
    )
    val plan = FluxaCoreNative.headlessPrefetchDetailStreams(preFetched)
    val prewarmUrl = plan.prewarmUrl
    if (plan.shouldPrewarmTorrent && prewarmUrl != null) {
        TorrentStreamManager.getInstance(context).preWarm(prewarmUrl, payload.string("id"))
    }
    // For HTTP streams, prime the first 2 MB into ExoPlayer's disk cache so the
    // next-episode transition starts from cache instead of a cold network open.
    preFetched
        .firstOrNull { stream ->
            val url = stream.playableUrl ?: return@firstOrNull false
            val scheme = Uri.parse(url).scheme?.lowercase() ?: return@firstOrNull false
            stream.infoHash == null && (scheme == "http" || scheme == "https")
        }
        ?.let { stream ->
            val url = stream.playableUrl!!
            val headers = stream.resolveHeaders()
            primeScope.launch { MediaPlayerController.primeHttpStream(context, url, headers) }
        }
    return ok(effect, mapOf("count" to plan.count, "prewarmedUrl" to prewarmUrl))
}

internal suspend fun FluxaAndroidHeadlessEnvironment.fetchDetailStreams(effect: NativeHeadlessEffect): HeadlessEffectCompletion {
    val payload = effect.payload
    val requestIds = payload.list("requestIds").mapNotNull { it?.toString()?.takeIf(String::isNotBlank) }

    val cs3Id = requestIds.firstOrNull { it.startsWith("cs3:") }
    if (cs3Id != null) {
        val streams = loadCsNativeStreams(cs3Id)
        return ok(effect, mapOf(
            "streams" to streams,
            "availableAddons" to emptyList<String>(),
            "resolvedRequestId" to cs3Id,
            "hasStreamProviders" to streams.isNotEmpty()
        ))
    }

    val profile = payload.profile()
    val type = payload.string("contentType")
    val language = payload.string("language", profile?.safeLanguage ?: "en")
    val addons = repository.getUserAddons(profile?.authKey.orEmpty(), profile?.safeLocalAddons.orEmpty())
    val loadedApis = pluginManager.loadedApis.value
    val providerPlan = FluxaCoreNative.headlessProviderAvailability(addons, loadedApis.map { it.name })
    val detail = payload.objectValue("detail")?.let { gson.fromJson(gson.toJsonTree(it), MetaDetail::class.java) }
    val seasonEpisodes = payload.list("seasonEpisodes").mapNotNull { raw ->
        runCatching { gson.fromJson(gson.toJsonTree(raw), Video::class.java) }.getOrNull()
    }
    val attempts = mutableListOf<Pair<String, List<Stream>>>()
    for ((index, requestId) in requestIds.withIndex()) {
        val episodeContext = FluxaCoreNative.streamDiscoveryEpisodeContext(type, requestId, detail, seasonEpisodes)
        val request = StreamDiscoveryRequest(
            addons = addons,
            type = type,
            id = requestId,
            language = language,
            preferFastStart = true,
            expectedEpisodeTitles = episodeContext.expectedEpisodeTitles,
            seasonEpisodeTitles = episodeContext.seasonEpisodeTitles.mapKeysNotNullToInt(),
            seasonEpisodeIds = episodeContext.seasonEpisodeIds.mapKeysNotNullToInt(),
            cs3PluginApis = loadedApis,
            cs3SearchQuery = detail?.name.orEmpty(),
            cs3OriginalName = detail?.originalName,
            cs3Year = detail?.releaseInfo?.toIntOrNull(),
            pluginTmdbId = detail?.tmdbId,
            pluginSeason = detail?.videos.orEmpty().firstOrNull { it.id == requestId }?.season
                ?: requestId.split(':').let { parts -> parts.getOrNull(parts.lastIndex - 1)?.toIntOrNull() },
            pluginEpisode = detail?.videos.orEmpty().firstOrNull { it.id == requestId }?.number
                ?: requestId.substringAfterLast(':', "").toIntOrNull(),
        )
        val streams = if (index == 0) {
            streamDiscovery.discoverProgressive(request) { streams, completedAddons, loadingAddons ->
                val streamsSnapshot = streams.toList()
                val completedSnapshot = completedAddons.toList()
                val loadingSnapshot = loadingAddons.toList()
                primeScope.launch {
                    _streamProgressFlow.emit(StreamProgressUpdate(requestId, streamsSnapshot, completedSnapshot, loadingSnapshot))
                }
            }
        } else {
            streamDiscovery.discover(request)
        }
        attempts += requestId to streams
        if (streams.isNotEmpty()) {
            break
        }
    }
    val plan = FluxaCoreNative.headlessDetailStreamResult(attempts, providerPlan.hasStreamProviders)
    return ok(
        effect,
        mapOf(
            "streams" to plan.streams,
            "availableAddons" to plan.availableAddons,
            "resolvedRequestId" to plan.resolvedRequestId,
            "hasStreamProviders" to plan.hasStreamProviders
        )
    )
}

internal suspend fun FluxaAndroidHeadlessEnvironment.prepareDirectPlayback(effect: NativeHeadlessEffect): HeadlessEffectCompletion {
    val payload = effect.payload
    val profile = payload.profile()
    val meta = gson.fromJson(gson.toJsonTree(payload["meta"]), Meta::class.java)
    val requestedVideoId = payload.stringOrNull("videoId")

    val cs3PlaybackId = requestedVideoId?.takeIf { it.startsWith("cs3:") }
        ?: meta.id.takeIf { it.startsWith("cs3:") }
    if (cs3PlaybackId != null) {
        val streams = loadCsNativeStreams(cs3PlaybackId)
        return if (streams.isNotEmpty()) ok(effect, DirectPlaybackTarget(meta, requestedVideoId, streams)) else ok(effect, null)
    }

    val language = payload.string("language", profile?.safeLanguage ?: "en")
    val policy = FluxaCoreNative.headlessDirectPlaybackPolicy()
    val detail = withTimeoutOrNull(policy.metaDetailTimeoutMs) {
        repository.getMetaDetail(
            type = meta.type,
            id = meta.id,
            language = language,
            authKey = profile?.authKey.orEmpty(),
            localAddons = profile?.safeLocalAddons.orEmpty(),
            useConfiguredAddons = true
        )
    }
    val plan = FluxaCoreNative.directPlaybackPlan(meta, detail, ReleaseDateUtils.todayIso())
    val playbackMeta = plan.meta ?: meta
    val targetVideoId = plan.targetVideoId
    val lookupId = plan.lookupId.ifBlank { targetVideoId ?: detail?.id ?: meta.id }
    val addons = repository.getUserAddons(profile?.authKey.orEmpty(), profile?.safeLocalAddons.orEmpty())
    val prefetchedDetail = if (StremioId.baseContentId(lookupId) == StremioId.baseContentId(meta.id)) detail else null
    val requestIds = buildPlaybackStreamRequestIds(meta.type, lookupId, language, profile, policy.streamDetailTimeoutMs, prefetchedDetail)
    val attempts = mutableListOf<Pair<String, List<Stream>>>()
    for (requestId in requestIds) {
        val streams = streamDiscovery.discover(
            StreamDiscoveryRequest(
                addons = addons,
                type = meta.type,
                id = requestId,
                language = language,
                preferFastStart = true,
                cs3PluginApis = pluginManager.loadedApis.value,
                cs3SearchQuery = playbackMeta.name,
                cs3OriginalName = playbackMeta.originalName,
                cs3Year = playbackMeta.releaseInfo?.toIntOrNull(),
                pluginTmdbId = prefetchedDetail?.tmdbId,
                pluginSeason = prefetchedDetail?.videos.orEmpty().firstOrNull { it.id == requestId }?.season
                    ?: requestId.split(':').let { parts -> parts.getOrNull(parts.lastIndex - 1)?.toIntOrNull() },
                pluginEpisode = prefetchedDetail?.videos.orEmpty().firstOrNull { it.id == requestId }?.number
                    ?: requestId.substringAfterLast(':', "").toIntOrNull(),
            )
        )
        attempts += requestId to streams
        if (streams.isNotEmpty()) {
            val streamPlan = FluxaCoreNative.headlessDetailStreamResult(attempts, hasStreamProviders = true)
            return ok(effect, DirectPlaybackTarget(playbackMeta, targetVideoId, streamPlan.streams))
        }
    }
    return ok(effect, null)
}

internal suspend fun FluxaAndroidHeadlessEnvironment.fetchIntroSegments(effect: NativeHeadlessEffect): HeadlessEffectCompletion {
    val payload = effect.payload
    val items: List<IntroTimestamps> = repository.getIntro(
        imdbId = payload.string("imdbId"),
        season = payload.number("season")?.toInt() ?: 0,
        episode = payload.number("episode")?.toInt() ?: 0,
        title = payload.stringOrNull("title"),
        useIntroDb = payload.boolean("useIntroDb", true),
        useAniSkip = payload.boolean("useAniSkip", true)
    )
    return ok(effect, items)
}

internal fun FluxaAndroidHeadlessEnvironment.resolveIntroImdbId(effect: NativeHeadlessEffect): HeadlessEffectCompletion {
    val payload = effect.payload
    val meta = gson.fromJson(gson.toJsonTree(payload["meta"]), Meta::class.java)
    val videoId = payload.stringOrNull("videoId")
    val imdbId = StremioId.imdbId(videoId)
        ?: StremioId.imdbId(meta.id)
        ?: StremioId.imdbId(FluxaCoreNative.playbackIntroLookupContentId(videoId ?: meta.id))
    return ok(effect, imdbId)
}
