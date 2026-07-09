package com.fluxa.app.core.rust

import android.content.Context
import android.util.Log
import com.fluxa.app.core.StremioId
import com.fluxa.app.data.local.OfflineDownloadManager
import com.fluxa.app.data.local.OfflineSubtitleOption
import com.fluxa.app.data.local.ProfileManager
import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.data.local.WatchlistManager
import com.fluxa.app.data.repository.ExternalSyncPushCoordinator
import com.fluxa.app.data.remote.IntroTimestamps
import com.fluxa.app.data.remote.AddonDescriptor
import com.fluxa.app.data.remote.CastMember
import com.fluxa.app.data.remote.DetailTrailer
import com.fluxa.app.data.remote.Meta
import com.fluxa.app.data.remote.MetaDetail
import com.fluxa.app.data.remote.MetaLink
import com.fluxa.app.data.remote.MetaRating
import com.fluxa.app.data.remote.Stream
import com.fluxa.app.data.remote.SubtitleData
import com.fluxa.app.data.remote.Video
import com.fluxa.app.data.repository.AddonRepository
import com.fluxa.app.data.repository.StremioRepository
import com.fluxa.app.data.repository.TraktRepository
import com.fluxa.app.domain.discovery.StreamDiscoveryRequest
import com.fluxa.app.domain.discovery.StreamDiscoveryUseCase
import com.fluxa.app.player.MediaPlayerController
import com.fluxa.app.player.TorrentStreamManager
import com.fluxa.app.player.TorrentStreamResult
import com.fluxa.app.data.repository.CloudStreamCatalogClient
import com.fluxa.app.data.repository.toStremioType
import com.fluxa.app.plugins.PluginManager
import com.fluxa.app.plugins.cloudstream.ExternalExtensionRunner
import com.fluxa.app.plugins.cloudstream.ScraperActor
import com.fluxa.app.plugins.cloudstream.ScraperLoadResult
import com.fluxa.app.plugins.cloudstream.ScraperSearchResult
import com.fluxa.app.plugins.cloudstream.ScraperSubtitle
import com.fluxa.app.plugins.cloudstream.ScraperTrailer
import com.fluxa.app.common.AppStrings
import com.fluxa.app.ui.catalog.CalendarWidgetProvider
import com.fluxa.app.ui.catalog.CalendarUpcomingItem
import com.fluxa.app.ui.catalog.EpisodeCalendarLoader
import com.fluxa.app.ui.catalog.EpisodeNotificationHelper
import com.fluxa.app.ui.catalog.HomeCategory
import com.fluxa.app.ui.catalog.HomeCatalogSource
import com.fluxa.app.domain.discovery.buildDiscoverCatalogOptions
import com.fluxa.app.domain.discovery.buildMetadataFeedOptions
import com.fluxa.app.domain.discovery.effectiveHomeMetadataFeedSelection
import com.fluxa.app.domain.discovery.isMetadataFeedEnabled
import com.fluxa.app.domain.discovery.orderedMetadataFeeds
import com.fluxa.app.common.ReleaseDateUtils
import com.fluxa.app.ui.catalog.DiscoverGenreOption
import com.fluxa.app.data.repository.TraktIntegration
import com.fluxa.app.ui.catalog.DirectPlaybackTarget
import com.fluxa.app.ui.catalog.TraktScrobbleWorker
import com.fluxa.app.ui.catalog.SimklScrobbleWorker
import com.fluxa.app.core.rust.models.NativeHeadlessEffect
import com.google.gson.Gson
import dagger.hilt.android.qualifiers.ApplicationContext
import javax.inject.Inject
import javax.inject.Singleton
import kotlin.coroutines.resume
import android.net.Uri
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.async
import kotlinx.coroutines.awaitAll
import kotlinx.coroutines.coroutineScope
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.SharedFlow
import kotlinx.coroutines.launch
import kotlinx.coroutines.suspendCancellableCoroutine
import kotlinx.coroutines.sync.Semaphore
import kotlinx.coroutines.sync.withPermit
import kotlinx.coroutines.withContext
import kotlinx.coroutines.withTimeoutOrNull

data class StreamProgressUpdate(
    val streams: List<Stream>,
    val completedAddonNames: List<String>,
    val loadingAddonNames: List<String>
)

@Singleton
class FluxaAndroidHeadlessEnvironment @Inject constructor(
    @param:ApplicationContext internal val context: Context,
    internal val repository: StremioRepository,
    internal val addonRepository: AddonRepository,
    internal val traktRepository: TraktRepository,
    internal val watchlistManager: WatchlistManager,
    internal val streamDiscovery: StreamDiscoveryUseCase,
    internal val pluginManager: PluginManager,
    internal val gson: Gson,
    internal val profileManager: ProfileManager,
    internal val externalSyncPushCoordinator: ExternalSyncPushCoordinator
) : HeadlessPlatformEnvironment {

    internal val primeScope = CoroutineScope(SupervisorJob() + Dispatchers.IO)

    private val _streamProgressFlow = MutableSharedFlow<StreamProgressUpdate>(replay = 0, extraBufferCapacity = 32)
    val streamProgressFlow: SharedFlow<StreamProgressUpdate> = _streamProgressFlow

    override suspend fun execute(effect: NativeHeadlessEffect): HeadlessEffectCompletion = withContext(Dispatchers.IO) {
        runCatching {
            syncWatchlistProfile(effect)
            when (effect.type) {
                "fetchMetaDetail" -> fetchMetaDetail(effect)
                "readPlaybackProgress" -> readPlaybackProgress(effect)
                "readDetailLocalState" -> readDetailLocalState(effect)
                "fetchDetailSecondary" -> fetchDetailSecondary(effect)
                "prefetchDetailStreams" -> prefetchDetailStreams(effect)
                "fetchDetailStreams" -> fetchDetailStreams(effect)
                "fetchMetaDetailLookup" -> fetchMetaDetail(effect)
                "prepareDirectPlayback" -> prepareDirectPlayback(effect)
                "fetchIntroSegments" -> fetchIntroSegments(effect)
                "resolveIntroImdbId" -> resolveIntroImdbId(effect)
                "loadStreams" -> loadStreams(effect)
                "enqueueTraktScrobble" -> enqueueTraktScrobble(effect)
                "startTorrentStream" -> startTorrentStream(effect)
                "stopTorrent" -> stopTorrent(effect)
                "fetchAddonManifest" -> fetchAddonManifest(effect)
                "refreshInstalledAddons" -> refreshInstalledAddons(effect)
                "fetchAddonResource" -> fetchAddonResource(effect)
                "readHomeBootstrap" -> readHomeBootstrap(effect)
                "readLibraryState" -> readLibraryState(effect)
                "writeLibraryCommand" -> writeLibraryCommand(effect)
                "writeFeedback" -> writeFeedback(effect)
                "clearPlaybackProgress" -> clearPlaybackProgress(effect)
                "syncWatchedState" -> syncWatchedState(effect)
                "writePlaybackProgress" -> writePlaybackProgress(effect)
                "runSearch" -> runSearch(effect)
                "runDiscover" -> runDiscover(effect)
                "readDiscoverCatalogFilters" -> readDiscoverCatalogFilters(effect)
                "fetchCatalogPage" -> fetchCatalogPage(effect)
                "fetchSeasonEpisodes" -> fetchSeasonEpisodes(effect)
                "fetchSubtitles" -> fetchSubtitles(effect)
                "runExternalSync" -> runExternalSync(effect)
                "runAuthFlow" -> runAuthFlow(effect)
                "exchangeAuthCode" -> exchangeAuthCode(effect)
                "refreshAuthToken" -> refreshAuthToken(effect)
                "syncExternalIntegration" -> syncExternalIntegration(effect)
                "writeSettings" -> writeSettings(effect)
                "readCalendarMonth" -> readCalendarMonth(effect)
                "replaceExternalContinueWatching" -> replaceExternalContinueWatching(effect)
                "updateCalendarWidget" -> updateCalendarWidget(effect)
                "notifyReleasedEpisodes" -> notifyReleasedEpisodes(effect)
                "enqueueOfflineDownload" -> enqueueOfflineDownload(effect)
                else -> error(effect, "unsupported_effect")
            }
        }.getOrElse { throwable ->
            android.util.Log.e("HeadlessEnv", "effect ${effect.type} failed", throwable)
            error(effect, throwable.message ?: throwable::class.java.simpleName)
        }
    }

    private suspend fun fetchMetaDetail(effect: NativeHeadlessEffect): HeadlessEffectCompletion {
        val payload = effect.payload
        val id = payload.string("id")

        if (id.startsWith("cs3:")) {
            val csDetail = loadCsNativeMetaDetail(id)
            if (csDetail != null) return ok(effect, csDetail)
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
        Log.d("MetaFetch", "fetchMetaDetail result: ${if (detail != null) "SUCCESS name=${detail.name} videos=${detail.videos?.size}" else "NULL"}")
        val enriched = if (detail != null && profile?.safeTmdbApiKey?.isNotBlank() == true) {
            repository.enrichDetailWithTmdb(detail, profile.safeTmdbApiKey, profile, language)
        } else {
            detail
        }
        return ok(effect, enriched)
    }

    private suspend fun readPlaybackProgress(effect: NativeHeadlessEffect): HeadlessEffectCompletion {
        return ok(effect, watchlistManager.getPlaybackProgress(effect.payload.string("id")))
    }

    private suspend fun readDetailLocalState(effect: NativeHeadlessEffect): HeadlessEffectCompletion {
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

    private suspend fun fetchDetailSecondary(effect: NativeHeadlessEffect): HeadlessEffectCompletion {
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
        return ok(effect, mapOf("watchedVideoIds" to watchedIds.toList(), "similarItems" to similarItems, "trailers" to trailers))
    }

    private suspend fun prefetchDetailStreams(effect: NativeHeadlessEffect): HeadlessEffectCompletion {
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
                val headers = stream.getHeaders()
                primeScope.launch { MediaPlayerController.primeHttpStream(context, url, headers) }
            }
        return ok(effect, mapOf("count" to plan.count, "prewarmedUrl" to prewarmUrl))
    }

    private suspend fun fetchDetailStreams(effect: NativeHeadlessEffect): HeadlessEffectCompletion {
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
            streamDiscovery.invalidate(type, requestId, language)
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
                cs3Year = detail?.releaseInfo?.toIntOrNull()
            )
            val streams = if (index == 0) {
                streamDiscovery.discoverProgressive(request) { streams, completedAddons, loadingAddons ->
                    primeScope.launch {
                        _streamProgressFlow.emit(StreamProgressUpdate(streams, completedAddons, loadingAddons))
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

    private suspend fun prepareDirectPlayback(effect: NativeHeadlessEffect): HeadlessEffectCompletion {
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
                    cs3Year = playbackMeta.releaseInfo?.toIntOrNull()
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

    private suspend fun fetchIntroSegments(effect: NativeHeadlessEffect): HeadlessEffectCompletion {
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

    private fun resolveIntroImdbId(effect: NativeHeadlessEffect): HeadlessEffectCompletion {
        val payload = effect.payload
        val meta = gson.fromJson(gson.toJsonTree(payload["meta"]), Meta::class.java)
        val videoId = payload.stringOrNull("videoId")
        val imdbId = StremioId.imdbId(videoId)
            ?: StremioId.imdbId(meta.id)
            ?: StremioId.imdbId(FluxaCoreNative.playbackIntroLookupContentId(videoId ?: meta.id))
        return ok(effect, imdbId)
    }

    private suspend fun loadStreams(effect: NativeHeadlessEffect): HeadlessEffectCompletion {
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

    private fun enqueueTraktScrobble(effect: NativeHeadlessEffect): HeadlessEffectCompletion {
        val payload = effect.payload
        val profile = payload.profile() ?: return error(effect, "missing_profile")
        val token = payload.string("token")
        if (profile.traktAccessToken.isNullOrBlank() || token.isBlank()) return ok(effect, mapOf("queued" to false))
        TraktScrobbleWorker.enqueue(
            context = context,
            profileId = profile.id,
            mediaType = payload.string("metaType"),
            mediaId = payload.string("itemId"),
            progress = (payload.number("progress")?.toFloat() ?: 0f).coerceIn(0f, 100f),
            action = payload.string("actionName")
        )
        return ok(effect, mapOf("queued" to true))
    }

    private suspend fun startTorrentStream(effect: NativeHeadlessEffect): HeadlessEffectCompletion {
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

    private fun stopTorrent(effect: NativeHeadlessEffect): HeadlessEffectCompletion {
        TorrentStreamManager.getInstance(context).stop()
        return ok(effect, emptyMap<String, Any?>())
    }

    private suspend fun fetchAddonManifest(effect: NativeHeadlessEffect): HeadlessEffectCompletion {
        val manifest = addonRepository.getAddonManifest(
            transportUrl = effect.payload.string("transportUrl"),
            forceRefresh = effect.payload.boolean("forceRefresh")
        )
        return ok(effect, manifest)
    }

    private suspend fun refreshInstalledAddons(effect: NativeHeadlessEffect): HeadlessEffectCompletion {
        val payload = effect.payload
        val profile = payload.profile() ?: return ok(effect, mapOf("addons" to emptyList<AddonDescriptor>()))
        val addons = withTimeoutOrNull(10_000L) {
            repository.getUserAddons(
                authKey = profile.authKey,
                localAddons = profile.safeLocalAddons,
                forceRefresh = payload.boolean("forceRefresh", true)
            )
        }.orEmpty()
        return ok(effect, mapOf("addons" to addons))
    }

    private suspend fun fetchAddonResource(effect: NativeHeadlessEffect): HeadlessEffectCompletion {
        val payload = effect.payload
        val transportUrl = payload.string("transportUrl")
        val type = payload.string("contentType")
        val id = payload.string("id")
        val resource = payload.string("resource")
        Log.d("MetaFetch", "fetchAddonResource: resource=$resource type=$type id=${id.take(30)} url=$transportUrl")
        val value = when (resource) {
            "stream" -> addonRepository.getStreamsFromAddon(transportUrl, payload.string("addonName", ""), type, id)
            "catalog" -> addonRepository.getAddonCatalog(
                transportUrl = transportUrl,
                type = type,
                id = id,
                skip = payload.extraNumber("skip")?.toInt() ?: 0,
                genre = payload.extraString("genre"),
                search = payload.extraString("search")
            )
            "meta" -> addonRepository.getMetaDetailFromSpecificAddon(transportUrl, type, id)
            "subtitles" -> addonRepository.getSubtitlesFromAddon(
                transportUrl,
                type,
                id,
                payload.stringOrNull("extraArgs") ?: payload.extraString("extraArgs").orEmpty()
            )
            else -> emptyList<Any>()
        }
        return ok(effect, value)
    }

    private suspend fun readHomeBootstrap(effect: NativeHeadlessEffect): HeadlessEffectCompletion {
        val profile = effect.payload.profile()
        profile?.id?.let(watchlistManager::setActiveProfile)
        val addons = addonRepository.getUserAddons(profile?.authKey.orEmpty(), profile?.safeLocalAddons.orEmpty())
        val language = effect.payload.string("language", profile?.safeLanguage ?: "en")
        val allFeeds = buildMetadataFeedOptions(addons, language)
        val metadataFeeds = orderedMetadataFeeds(allFeeds, profile?.homeFeedOrder).let { feeds ->
            val availableKeys = feeds.map { it.key }
            val selectedKeys = effectiveHomeMetadataFeedSelection(profile?.homeFeedToggles, availableKeys)
            feeds.filter { isMetadataFeedEnabled(selectedKeys, it.key) }
        }
        val categories = metadataFeeds.mapNotNull { feed ->
            val items = runCatching {
                addonRepository.getAddonCatalog(
                    transportUrl = feed.transportUrl,
                    type = feed.type,
                    id = feed.id,
                    genre = feed.genre
                )
            }.getOrDefault(emptyList())
            if (items.isEmpty()) null else {
                HomeCategory(
                    name = feed.label,
                    semanticName = feed.label,
                    items = items,
                    id = feed.key,
                    type = feed.type,
                    catalogId = feed.id,
                    addonTransportUrl = feed.transportUrl,
                    addonGenre = feed.genre
                )
            }
        }
        return ok(
            effect,
            mapOf(
                "categories" to categories,
                "continueWatching" to watchlistManager.getContinueWatchingSnapshot(),
                "watchlist" to watchlistManager.getWatchlistSnapshot(),
                "userAddons" to addons,
                "metadataFeeds" to metadataFeeds,
                "billboard" to categories.firstOrNull()?.items?.firstOrNull()
            )
        )
    }

    private suspend fun readLibraryState(effect: NativeHeadlessEffect): HeadlessEffectCompletion {
        effect.payload.stringOrNull("profileId")?.let(watchlistManager::setActiveProfile)
        return ok(
            effect,
            mapOf(
                "watchlist" to watchlistManager.getWatchlistSnapshot(),
                "continueWatching" to watchlistManager.getContinueWatchingSnapshot(),
                "liked" to emptyList<Any>(),
                "watched" to emptyMap<String, Any>()
            )
        )
    }

    private suspend fun writeLibraryCommand(effect: NativeHeadlessEffect): HeadlessEffectCompletion {
        val command = effect.payload.objectValue("command").orEmpty()
        val profileId = effect.payload.stringOrNull("profileId")
        val profile = profileId?.let { id -> profileManager.getProfiles().firstOrNull { it.id == id } }
        val value = when (command.string("type")) {
            "toggleWatchlist" -> {
                val item = command.objectValue("item")?.let { gson.fromJson(gson.toJsonTree(it), Meta::class.java) }
                if (item != null) {
                    val wasInWatchlist = watchlistManager.isInWatchlist(item.id)
                    watchlistManager.toggleWatchlist(item)
                    val isInWatchlist = watchlistManager.isInWatchlist(item.id)
                    if (profile != null && isInWatchlist != wasInWatchlist) {
                        primeScope.launch {
                            runCatching { externalSyncPushCoordinator.pushWatchlist(profile, item, isInWatchlist) }
                        }
                    }
                    mapOf("watchlist" to watchlistManager.getWatchlistSnapshot(), "isInWatchlist" to isInWatchlist)
                } else {
                    mapOf("watchlist" to watchlistManager.getWatchlistSnapshot())
                }
            }
            "markWatched" -> {
                val watched = command.boolean("watched", true)
                val localWatched = watchlistManager.markEpisodesWatched(
                seriesId = command.string("seriesId"),
                videoIds = command.list("videoIds").mapNotNull { it?.toString() },
                watched = watched
                )
                if (profile != null) {
                    val meta = command.objectValue("meta")?.let { gson.fromJson(gson.toJsonTree(it), Meta::class.java) }
                    val episodes = command.list("episodes").mapNotNull {
                        runCatching { gson.fromJson(gson.toJsonTree(it), Video::class.java) }.getOrNull()
                    }
                    if (meta != null) {
                        primeScope.launch {
                            runCatching { externalSyncPushCoordinator.pushMarkWatched(profile, meta, episodes, watched) }
                        }
                    }
                }
                mapOf("watchlist" to watchlistManager.getWatchlistSnapshot(), "localWatchedVideoIds" to localWatched.toList())
            }
            else -> mapOf("watchlist" to watchlistManager.getWatchlistSnapshot())
        }
        return ok(effect, value)
    }

    private suspend fun writeFeedback(effect: NativeHeadlessEffect): HeadlessEffectCompletion {
        val payload = effect.payload
        val meta = gson.fromJson(gson.toJsonTree(payload["meta"]), Meta::class.java)
        val value = payload["value"] as? Boolean
        watchlistManager.setFeedback(payload.string("id"), value, meta)
        return ok(effect, mapOf("feedback" to value))
    }

    private suspend fun clearPlaybackProgress(effect: NativeHeadlessEffect): HeadlessEffectCompletion {
        val payload = effect.payload
        val profile = payload.profile()
        val meta = gson.fromJson(gson.toJsonTree(payload["meta"]), Meta::class.java)
        if (!profile?.authKey.isNullOrBlank()) {
            repository.clearPlaybackProgress(profile.authKey, meta)
        }
        repository.clearTraktPlaybackProgress(profile?.traktAccessToken, meta)
        watchlistManager.clearPlaybackProgress(meta.id)
        return ok(effect, emptyMap<String, Any?>())
    }

    private suspend fun writePlaybackProgress(effect: NativeHeadlessEffect): HeadlessEffectCompletion {
        val profile = effect.payload.profile()
        val progress = effect.payload.objectValue("progress").orEmpty()
        val meta = gson.fromJson(gson.toJsonTree(progress["meta"]), Meta::class.java)
        val timeOffset = progress.number("timeOffset")?.toLong() ?: 0L
        val duration = progress.number("duration")?.toLong() ?: 0L
        if (profile?.isGuest == false) {
            repository.savePlaybackProgress(profile.authKey, meta, timeOffset, duration)
        }
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
        val token = profile?.traktAccessToken
        if (effect.payload.boolean("scrobbleTraktPause", true) && !token.isNullOrBlank() && duration > 0L && timeOffset > 5_000L) {
            val playbackPercent = (timeOffset.toFloat() / duration.toFloat() * 100f).coerceIn(0f, 100f)
            if (playbackPercent in 0.5f..94.9f) {
                TraktScrobbleWorker.enqueue(
                    context = context,
                    profileId = profile.id,
                    mediaType = meta.type,
                    mediaId = TraktIntegration.scrobbleMediaId(meta.id, progress.stringOrNull("lastVideoId"), meta.type),
                    progress = playbackPercent,
                    action = "pause"
                )
            }
        }
        val simklToken = profile?.simklAccessToken
        if (!simklToken.isNullOrBlank() && duration > 0L && timeOffset > 5_000L) {
            SimklScrobbleWorker.enqueue(
                context = context,
                profileId = profile.id,
                mediaType = meta.type,
                mediaId = TraktIntegration.scrobbleMediaId(meta.id, progress.stringOrNull("lastVideoId"), meta.type),
                action = "pause",
                positionMs = timeOffset,
                durationMs = duration
            )
        }
        return ok(effect, emptyMap<String, Any?>())
    }

    private suspend fun syncWatchedState(effect: NativeHeadlessEffect): HeadlessEffectCompletion {
        val payload = effect.payload
        val profile = payload.profile()
        val meta = payload.objectValue("meta")?.let { gson.fromJson(gson.toJsonTree(it), Meta::class.java) }
            ?: return error(effect, "missing_meta")
        val episodes = payload.list("episodes").mapNotNull { raw ->
            runCatching { gson.fromJson(gson.toJsonTree(raw), Video::class.java) }.getOrNull()
        }
        repository.syncWatchedState(
            authKey = profile?.authKey,
            traktToken = profile?.traktAccessToken,
            meta = meta,
            episodes = episodes,
            watched = payload.boolean("watched", true)
        )
        return ok(effect, mapOf("synced" to true))
    }

    private suspend fun runSearch(effect: NativeHeadlessEffect): HeadlessEffectCompletion {
        val profile = effect.payload.profile()
        return ok(
            effect,
            mapOf(
                "results" to addonRepository.searchRows(
                    query = effect.payload.string("query"),
                    language = effect.payload.string("language", profile?.safeLanguage ?: "en"),
                    authKey = profile?.authKey.orEmpty(),
                    localAddons = profile?.safeLocalAddons.orEmpty()
                ).flatMap { it.items }
            )
        )
    }

    private suspend fun runDiscover(effect: NativeHeadlessEffect): HeadlessEffectCompletion = coroutineScope {
        val payload = effect.payload
        val profile = payload.profile()
        val contentType = payload.string("contentType")
        val filters = payload.objectValue("filters")
        val genre = filters?.stringOrNull("genre")
        val selectedCatalogKey = filters?.stringOrNull("catalogKey")
        val addons = addonRepository.getUserAddons(profile?.authKey.orEmpty(), profile?.safeLocalAddons.orEmpty())
        val catalogOptions = buildDiscoverCatalogOptions(addons, contentType)
        val catalogs = selectedCatalogKey
            ?.let { key -> catalogOptions.filter { it.key == key } }
            ?.takeIf { it.isNotEmpty() }
            ?: catalogOptions
        // Fan out catalog fetches concurrently (was a sequential flatMap). awaitAll preserves
        // catalog order in the merged result; the semaphore caps concurrent addon requests.
        val semaphore = Semaphore(8)
        val fetched = catalogs.flatMap { catalog ->
            val selectedTypes = if (catalog.type == "all") listOf("movie", "series") else listOf(catalog.type)
            selectedTypes.map { type -> catalog to type }
        }.map { (catalog, type) ->
            async {
                semaphore.withPermit {
                    runCatching {
                        val items = addonRepository.getAddonCatalog(
                            transportUrl = catalog.transportUrl,
                            type = type,
                            id = catalog.id,
                            genre = genre
                        )
                        val source = HomeCatalogSource(
                            transportUrl = catalog.transportUrl,
                            catalogId = catalog.id,
                            type = type,
                            genre = genre
                        )
                        items to source
                    }.getOrDefault(emptyList<Meta>() to HomeCatalogSource(catalog.transportUrl, catalog.id, type, genre))
                }
            }
        }.awaitAll()
        val results = fetched.flatMap { it.first }
        val resultSources = linkedMapOf<String, HomeCatalogSource>()
        fetched.forEach { (items, source) ->
            items.forEach { item ->
                resultSources["${item.type}:${item.id}"] = source
                resultSources.putIfAbsent(item.id, source)
            }
        }
        ok(effect, mapOf("results" to results, "resultSources" to resultSources))
    }

    private suspend fun readDiscoverCatalogFilters(effect: NativeHeadlessEffect): HeadlessEffectCompletion {
        val payload = effect.payload
        val profile = payload.profile()
        val addons = addonRepository.getUserAddons(profile?.authKey.orEmpty(), profile?.safeLocalAddons.orEmpty())
        val catalogOptions = buildDiscoverCatalogOptions(addons, payload.string("contentType"))
        val selectedCatalog = catalogOptions.firstOrNull { it.key == payload.stringOrNull("selectedCatalogKey") }
        val selectedGenres = selectedCatalog?.genres.orEmpty()
            .distinct()
            .sortedBy { it.lowercase(java.util.Locale.ROOT) }
            .map { DiscoverGenreOption(it, it) }
        val genres = if (selectedCatalog == null || selectedGenres.isEmpty()) {
            emptyList()
        } else if (!selectedCatalog.requiresGenre) {
            listOf(DiscoverGenreOption(null, AppStrings.t(payload.string("language", profile?.safeLanguage ?: "en"), "auto.all"))) + selectedGenres
        } else {
            selectedGenres
        }
        return ok(effect, mapOf("catalogs" to catalogOptions, "genres" to genres))
    }

    private suspend fun fetchCatalogPage(effect: NativeHeadlessEffect): HeadlessEffectCompletion {
        val payload = effect.payload
        return ok(
            effect,
            mapOf(
                "items" to addonRepository.getAddonCatalog(
                    transportUrl = payload.string("transportUrl"),
                    type = payload.string("contentType"),
                    id = payload.string("catalogId"),
                    skip = payload.number("skip")?.toInt() ?: 0,
                    genre = payload.stringOrNull("genre"),
                    search = payload.stringOrNull("search")
                )
            )
        )
    }

    private suspend fun fetchSeasonEpisodes(effect: NativeHeadlessEffect): HeadlessEffectCompletion {
        val payload = effect.payload
        val profile = payload.profile()
        val language = payload.string("language", profile?.safeLanguage ?: "en")
        val seriesId = payload.string("seriesId")
        val seasonNumber = payload.number("season")?.toInt() ?: 1
        val episodes = repository.getTvSeason(
            id = seriesId,
            seasonNumber = seasonNumber,
            language = language,
            authKey = profile?.authKey.orEmpty(),
            localAddons = profile?.safeLocalAddons.orEmpty(),
            useConfiguredAddons = true
        )
        val enriched = if (profile?.safeTmdbApiKey?.isNotBlank() == true && profile.safeTmdbEpisodeImagesEnabled) {
            val tmdbNumId = when {
                seriesId.startsWith("tmdb:", ignoreCase = true) ->
                    seriesId.removePrefix("tmdb:").substringBefore(":").takeIf { it.toIntOrNull() != null }
                seriesId.substringBefore(":").toIntOrNull() != null ->
                    seriesId.substringBefore(":")
                else -> null
            }
            if (tmdbNumId != null) {
                repository.enrichSeasonEpisodesWithTmdb(tmdbNumId, seasonNumber, episodes, profile.safeTmdbApiKey, language)
            } else {
                episodes
            }
        } else {
            episodes
        }
        return ok(effect, mapOf("episodes" to enriched))
    }

    private fun fetchSubtitles(effect: NativeHeadlessEffect): HeadlessEffectCompletion {
        val stream = effect.payload.objectValue("stream")?.let { gson.fromJson(gson.toJsonTree(it), Stream::class.java) }
        return ok(effect, mapOf("subtitles" to stream?.subtitles.orEmpty()))
    }

    private suspend fun runExternalSync(effect: NativeHeadlessEffect): HeadlessEffectCompletion {
        val profile = effect.payload.profile() ?: return ok(effect, emptyMap<String, Any?>())
        return ok(
            effect,
            mapOf(
                "snapshot" to when (effect.payload.string("provider")) {
                    "trakt" -> traktRepository.getSyncSnapshot(profile, effect.payload.string("language", profile.safeLanguage))
                    else -> repository.getExternalContinueWatching(profile, effect.payload.string("language", profile.safeLanguage))
                }
            )
        )
    }

    private suspend fun runAuthFlow(effect: NativeHeadlessEffect): HeadlessEffectCompletion {
        return when (effect.payload.string("provider")) {
            "trakt" -> when (effect.payload.string("mode")) {
                "deviceCode" -> ok(effect, repository.createTraktDeviceCode())
                else -> error(effect, "unsupported_auth_mode")
            }
            else -> error(effect, "unsupported_auth_provider")
        }
    }

    private suspend fun exchangeAuthCode(effect: NativeHeadlessEffect): HeadlessEffectCompletion {
        val payload = effect.payload
        val profile = payload.profile() ?: return error(effect, "missing_profile")
        val updated = when (payload.string("provider")) {
            "trakt" -> {
                val response = repository.exchangeTraktCode(payload.string("code"))
                profile.copy(
                    traktAccessToken = response.accessToken,
                    traktRefreshToken = response.refreshToken,
                    traktTokenExpiresAt = TraktIntegration.tokenExpiresAt(response.createdAt, response.expiresIn)
                )
            }
            "traktDevice" -> {
                val response = repository.exchangeTraktDeviceCode(payload.string("code"))
                if (!response.isSuccessful) {
                    val errorCode = response.errorBody()?.string()?.let(FluxaCoreNative::traktOAuthErrorCode)
                    return ok(
                        effect,
                        mapOf(
                            "status" to "pending",
                            "errorCode" to (errorCode ?: "http_${response.code()}"),
                            "httpCode" to response.code(),
                            "retryAfterSeconds" to response.headers()["Retry-After"]?.toLongOrNull()
                        )
                    )
                }
                val tokenResponse = response.body() ?: return error(effect, "empty_device_token")
                profile.copy(
                    traktAccessToken = tokenResponse.accessToken,
                    traktRefreshToken = tokenResponse.refreshToken,
                    traktTokenExpiresAt = TraktIntegration.tokenExpiresAt(tokenResponse.createdAt, tokenResponse.expiresIn)
                )
            }
            "mal" -> {
                val response = repository.exchangeMalCode(payload.string("code"), payload.string("codeVerifier"))
                profile.copy(
                    malAccessToken = response.accessToken,
                    malRefreshToken = response.refreshToken,
                    malTokenExpiresAt = response.expiresIn?.let { System.currentTimeMillis() + it * 1000L }
                )
            }
            "simkl" -> {
                val response = repository.exchangeSimklCode(payload.string("code"))
                profile.copy(simklAccessToken = response.accessToken)
            }
            "anilist" -> {
                val response = repository.exchangeAnilistCode(payload.string("code"))
                profile.copy(
                    anilistAccessToken = response.accessToken,
                    anilistRefreshToken = response.refreshToken,
                    anilistTokenExpiresAt = response.expiresIn?.let { System.currentTimeMillis() + it * 1000L }
                )
            }
            else -> return error(effect, "unsupported_auth_provider")
        }
        return ok(effect, mapOf("profile" to updated))
    }

    private suspend fun refreshAuthToken(effect: NativeHeadlessEffect): HeadlessEffectCompletion {
        val payload = effect.payload
        val profile = payload.profile() ?: return error(effect, "missing_profile")
        val updated = when (payload.string("provider")) {
            "trakt" -> refreshTraktTokenIfNeeded(profile)
            "mal" -> refreshMalTokenIfNeeded(profile)
            else -> return error(effect, "unsupported_auth_provider")
        }
        return ok(effect, mapOf("profile" to updated))
    }

    private suspend fun refreshTraktTokenIfNeeded(profile: UserProfile): UserProfile {
        val refreshToken = profile.traktRefreshToken?.takeIf { it.isNotBlank() } ?: return profile
        val refreshWindowMs = 24L * 60L * 60L * 1000L
        if (!profile.traktAccessToken.isNullOrBlank() && profile.safeTraktTokenExpiresAt > System.currentTimeMillis() + refreshWindowMs) {
            return profile
        }
        return runCatching {
            val response = traktRepository.refreshTraktToken(refreshToken)
            profile.copy(
                traktAccessToken = response.accessToken,
                traktRefreshToken = response.refreshToken,
                traktTokenExpiresAt = TraktIntegration.tokenExpiresAt(response.createdAt, response.expiresIn)
            )
        }.getOrElse { throwable ->
            Log.w("Trakt", "Token refresh failed", throwable)
            val status = (throwable as? retrofit2.HttpException)?.code()
            if (status == 400 || status == 401) {
                profile.copy(traktAccessToken = null, traktRefreshToken = null, traktTokenExpiresAt = null)
            } else {
                profile
            }
        }
    }

    private suspend fun refreshMalTokenIfNeeded(profile: UserProfile): UserProfile {
        val refreshToken = profile.malRefreshToken?.takeIf { it.isNotBlank() } ?: return profile
        val refreshWindowMs = 24L * 60L * 60L * 1000L
        if (!profile.malAccessToken.isNullOrBlank() && profile.safeMalTokenExpiresAt > System.currentTimeMillis() + refreshWindowMs) {
            return profile
        }
        return runCatching {
            val response = repository.refreshMalToken(refreshToken)
            profile.copy(
                malAccessToken = response.accessToken,
                malRefreshToken = response.refreshToken ?: refreshToken,
                malTokenExpiresAt = response.expiresIn?.let { System.currentTimeMillis() + it * 1000L }
            )
        }.getOrElse { throwable ->
            Log.w("Mal", "Token refresh failed", throwable)
            val status = (throwable as? retrofit2.HttpException)?.code()
            if (status == 400 || status == 401) {
                profile.copy(malAccessToken = null, malRefreshToken = null, malTokenExpiresAt = null)
            } else {
                profile
            }
        }
    }

    private suspend fun syncExternalIntegration(effect: NativeHeadlessEffect): HeadlessEffectCompletion {
        val payload = effect.payload
        val profile = payload.profile() ?: return error(effect, "missing_profile")
        val traktToken = profile.traktAccessToken
        if (traktToken.isNullOrBlank()) return error(effect, "missing_trakt_token")
        val language = payload.string("language", profile.safeLanguage)
        val snapshot = traktRepository.getTraktSyncSnapshot(profile, language)
        val watchedState = withTimeoutOrNull(8_000L) {
            traktRepository.getTraktWatchedState(traktToken)
        }
        if (watchedState != null) {
            watchlistManager.replaceExternalWatchedEpisodes("trakt", watchedState.episodeIdsBySeries)
            watchlistManager.replaceExternalWatchedContentDurations("trakt", watchedState.durationRecords)
        }
        val externalItems = repository.getExternalContinueWatching(profile, language)
        val updated = profile.copy(
            traktLastSyncAt = System.currentTimeMillis(),
            traktLastSyncedItems = snapshot.syncedItems,
            traktLastContinueWatchingCount = snapshot.continueWatchingCount,
            traktLastWatchlistCount = snapshot.watchlistCount
        )
        return ok(
            effect,
            mapOf(
                "profile" to updated,
                "snapshot" to snapshot,
                "watchedState" to watchedState,
                "externalContinueWatching" to externalItems
            )
        )
    }

    private fun writeSettings(effect: NativeHeadlessEffect): HeadlessEffectCompletion {
        return ok(
            effect,
            mapOf(
                "key" to effect.payload.string("key"),
                "value" to effect.payload["value"]
            )
        )
    }

    private suspend fun readCalendarMonth(effect: NativeHeadlessEffect): HeadlessEffectCompletion {
        val profile = effect.payload.profile()
        val year = effect.payload.number("year")?.toInt() ?: return error(effect, "missing_year")
        val month = effect.payload.number("month")?.toInt() ?: return error(effect, "missing_month")
        val plannedItems = effect.payload.list("plannedItems").mapNotNull { raw ->
            runCatching { gson.fromJson(gson.toJsonTree(raw), Meta::class.java) }.getOrNull()
        }
        val result = EpisodeCalendarLoader(repository, watchlistManager).loadMonth(profile, year, month, plannedItems)
        return ok(
            effect,
            mapOf(
                "items" to result.items,
                "localItems" to result.localItems,
                "externalItems" to result.externalItems
            )
        )
    }

    private suspend fun replaceExternalContinueWatching(effect: NativeHeadlessEffect): HeadlessEffectCompletion {
        val items = effect.payload.list("items").mapNotNull { raw ->
            runCatching { gson.fromJson(gson.toJsonTree(raw), Meta::class.java) }.getOrNull()
        }
        watchlistManager.replaceExternalContinueWatching(items)
        return ok(effect, mapOf("count" to items.size))
    }

    private fun updateCalendarWidget(effect: NativeHeadlessEffect): HeadlessEffectCompletion {
        val profile = effect.payload.profile()
        val items = calendarItems(effect)
        CalendarWidgetProvider.updateCalendar(
            context = context,
            items = items,
            language = profile?.safeLanguage ?: "en",
            accentColorArgb = profile?.safeAccentColorArgb ?: 0xFFFFFFFF.toInt()
        )
        return ok(effect, mapOf("count" to items.size))
    }

    private suspend fun notifyReleasedEpisodes(effect: NativeHeadlessEffect): HeadlessEffectCompletion {
        val profile = effect.payload.profile()
        val items = calendarItems(effect)
        EpisodeNotificationHelper.notifyReleasedEpisodes(
            context = context,
            profile = profile,
            items = items,
            todayIso = ReleaseDateUtils.todayIso()
        )
        return ok(effect, mapOf("count" to items.size))
    }

    private suspend fun enqueueOfflineDownload(effect: NativeHeadlessEffect): HeadlessEffectCompletion {
        val payload = effect.payload
        val result = OfflineDownloadManager.getInstance(context).enqueue(
            profileId = payload.stringOrNull("profileId"),
            meta = gson.fromJson(gson.toJsonTree(payload["meta"]), Meta::class.java),
            video = payload.objectValue("video")?.let { gson.fromJson(gson.toJsonTree(it), Video::class.java) },
            videoId = payload.stringOrNull("videoId"),
            stream = gson.fromJson(gson.toJsonTree(payload["stream"]), Stream::class.java),
            subtitle = payload.objectValue("subtitle")?.let { gson.fromJson(gson.toJsonTree(it), OfflineSubtitleOption::class.java) },
            profileLanguage = payload.stringOrNull("language")
        )
        return result.fold(
            onSuccess = { ok(effect, it) },
            onFailure = { error(effect, it.message ?: "offline_download_failed") }
        )
    }

    internal fun ok(effect: NativeHeadlessEffect, value: Any?): HeadlessEffectCompletion =
        HeadlessEffectCompletion(effectId = effect.id, status = "ok", value = value)

    internal fun error(effect: NativeHeadlessEffect, code: String): HeadlessEffectCompletion =
        HeadlessEffectCompletion(effectId = effect.id, status = "error", error = mapOf("code" to code))

    internal fun syncWatchlistProfile(effect: NativeHeadlessEffect) {
        val profileId = effect.payload.parseProfile(gson)?.id ?: effect.payload.stringOrNull("profileId")
        profileId?.let(watchlistManager::setActiveProfile)
    }

    internal fun Map<String, Any?>.profile(): UserProfile? = parseProfile(gson)

    internal fun calendarItems(effect: NativeHeadlessEffect): List<CalendarUpcomingItem> =
        effect.payload.list("items").mapNotNull { raw ->
            runCatching { gson.fromJson(gson.toJsonTree(raw), CalendarUpcomingItem::class.java) }.getOrNull()
        }

    internal fun isTmdbContentId(id: String): Boolean =
        id.startsWith("tmdb:", ignoreCase = true) || id.toIntOrNull() != null

    internal suspend fun loadCsNativeMetaDetail(id: String): MetaDetail? {
        val (apiName, url) = CloudStreamCatalogClient.decodeCsId(id) ?: return null
        val api = pluginManager.loadedApis.value.firstOrNull { it.name == apiName } ?: run {
            Log.w("CS3Detail", "Plugin not found: $apiName")
            return null
        }
        val runner = ExternalExtensionRunner()
        val load = withTimeoutOrNull(20_000L) { runner.loadContent(api, url) } ?: run {
            Log.w("CS3Detail", "$apiName: loadContent timed out or returned null for url=$url")
            return null
        }
        Log.d("CS3Detail", "$apiName: load type=${load.type}, episodeCount=${load.episodes?.size ?: "null"}")
        val stremioType = load.type.toStremioType()
        val videos = load.episodes?.mapIndexed { idx, ep ->
            Video(
                id = CloudStreamCatalogClient.encodeCsId(apiName, ep.data),
                name = ep.name ?: "Episode ${idx + 1}",
                season = ep.season,
                number = ep.episode,
                released = ep.date?.toCs3IsoDate(),
                thumbnail = ep.posterUrl,
                overview = ep.description,
                rating = ep.rating?.let(::cs3RatingString),
                episodeRuntime = ep.runTime
            )
        }
        Log.d("CS3Detail", "$apiName: built MetaDetail with ${videos?.size ?: "null"} videos")
        return MetaDetail(
            id = id,
            type = stremioType,
            name = load.title,
            genres = load.tags,
            poster = load.posterUrl,
            background = load.backgroundPosterUrl ?: load.posterUrl,
            logo = load.logoUrl,
            description = load.plot,
            releaseInfo = load.year?.toString(),
            released = load.year?.let { "$it-01-01" },
            runtime = load.duration?.let { "${it}m" },
            videos = videos,
            trailers = load.trailers?.mapIndexedNotNull { index, trailer -> trailer.toDetailTrailer(apiName, index) },
            imdbRating = load.rating?.let(::cs3RatingString),
            ageRating = load.contentRating,
            ratings = load.rating?.let { listOf(MetaRating("Cloudstream", cs3RatingString(it))) },
            cast = load.actors?.map { it.toCastMember() },
            links = load.toMetaLinks(),
            status = when {
                load.comingSoon -> "Coming Soon"
                else -> load.status
            },
            originalName = load.synonyms?.firstOrNull { it != load.title },
            collectionParts = load.recommendations?.mapNotNull { it.toCs3Meta(apiName) }
        )
    }

    private fun Long.toCs3IsoDate(): String {
        val millis = if (this > 10_000_000_000L) this else this * 1000L
        return java.time.Instant.ofEpochMilli(millis)
            .atZone(java.time.ZoneOffset.UTC)
            .toLocalDate()
            .toString()
    }

    private fun cs3RatingString(rating: Int): String =
        "%.1f".format(java.util.Locale.US, rating.toFloat() / 10f)

    private fun ScraperActor.toCastMember(): CastMember =
        CastMember(name = name, character = role, profilePath = image)

    private fun ScraperTrailer.toDetailTrailer(apiName: String, index: Int): DetailTrailer? {
        val targetUrl = url.takeIf { it.isNotBlank() } ?: return null
        return DetailTrailer(
            id = "cs3:$apiName:trailer:$index",
            title = "Trailer ${index + 1}",
            type = if (raw) "Trailer" else "Extractor",
            url = targetUrl,
            thumbnail = null,
            source = apiName
        )
    }

    private fun ScraperSearchResult.toCs3Meta(apiName: String): Meta? {
        val title = title.takeIf { it.isNotBlank() } ?: return null
        return Meta(
            id = CloudStreamCatalogClient.encodeCsId(apiName, url),
            name = title,
            type = type?.toStremioType() ?: "movie",
            poster = posterUrl,
            releaseInfo = year?.toString(),
            imdbRating = quality,
            background = posterUrl
        )
    }

    private fun ScraperLoadResult.toMetaLinks(): List<MetaLink>? {
        val links = mutableListOf<MetaLink>()
        uniqueUrl?.takeIf { it.isNotBlank() }?.let { links.add(MetaLink("Source", "Cloudstream", it)) }
        url.takeIf { it.isNotBlank() && it != uniqueUrl }?.let { links.add(MetaLink("Page", "Cloudstream", it)) }
        syncData.orEmpty().forEach { (key, value) ->
            if (key.isNotBlank() && value.isNotBlank()) links.add(MetaLink(key, "Cloudstream Sync", value))
        }
        synonyms.orEmpty().forEach { synonym ->
            links.add(MetaLink(synonym, "Cloudstream Synonym", synonym))
        }
        nextAiringUnixTime?.let { unix ->
            val label = listOfNotNull(
                nextAiringSeason?.let { "S$it" },
                nextAiringEpisode?.let { "E$it" }
            ).joinToString("").ifBlank { "Next airing" }
            links.add(MetaLink(label, "Cloudstream Next Airing", unix.toString()))
        }
        seasonNames.orEmpty().forEach { season ->
            val name = season.name?.takeIf { it.isNotBlank() } ?: "Season ${season.displaySeason ?: season.season}"
            links.add(MetaLink(name, "Cloudstream Season", season.season.toString()))
        }
        return links.takeIf { it.isNotEmpty() }
    }

    internal suspend fun loadCsNativeStreams(id: String, directTimeoutMs: Long = 30_000L): List<Stream> {
        val (apiName, data) = CloudStreamCatalogClient.decodeCsId(id) ?: return emptyList()
        val api = pluginManager.loadedApis.value.firstOrNull { it.name == apiName } ?: return emptyList()
        val runner = ExternalExtensionRunner()

        val directResult = try {
            withTimeoutOrNull(directTimeoutMs) { runner.loadStreams(api, data) }
        } catch (_: Throwable) { null }
        if (directResult != null && directResult.links.isNotEmpty()) {
            return directResult.links
                .sortedByDescending { csQualityScore(it.quality) }
                .map { link ->
                    Stream(
                        name = " $apiName\n${link.quality}",
                        title = link.name,
                        url = link.url,
                        subtitles = directResult.subtitles.map { it.toSubtitleData() },
                        behaviorHints = buildMap {
                            put("proxyHeaders", buildMap { put("request", link.headers) })
                            link.referer?.let { put("referer", it) }
                            put("cs3Type", link.type)
                            put("isM3u8", link.isM3u8)
                            put("isDash", link.isDash)
                        },
                        addonName = " $apiName"
                    )
                }
        }

        val streamData = try {
            withTimeoutOrNull(15_000L) { runner.loadContent(api, data)?.data }
        } catch (e: Exception) {
            null
        } ?: data
        val result = withTimeoutOrNull(30_000L) {
            runner.loadStreams(api, streamData)
        } ?: return emptyList()
        return result.links
            .sortedByDescending { csQualityScore(it.quality) }
            .map { link ->
                Stream(
                    name = " $apiName\n${link.quality}",
                    title = link.name,
                    url = link.url,
                    subtitles = result.subtitles.map { it.toSubtitleData() },
                    behaviorHints = buildMap {
                        put("proxyHeaders", buildMap { put("request", link.headers) })
                        link.referer?.let { put("referer", it) }
                        put("cs3Type", link.type)
                        put("isM3u8", link.isM3u8)
                        put("isDash", link.isDash)
                    },
                    addonName = " $apiName"
                )
            }
    }

    private fun ScraperSubtitle.toSubtitleData() = SubtitleData(
        url = url,
        lang = lang
    )

    internal fun csQualityScore(quality: String): Int {
        val q = quality.lowercase()
        return when {
            q.contains("4k") || q.contains("2160") -> 2160
            q.contains("1440") -> 1440
            q.contains("1080") -> 1080
            q.contains("720") -> 720
            q.contains("480") -> 480
            q.contains("360") -> 360
            q.contains("240") -> 240
            else -> 0
        }
    }

    internal suspend fun buildPlaybackStreamRequestIds(
        type: String,
        id: String,
        language: String,
        profile: UserProfile?,
        timeoutMs: Long,
        prefetchedDetail: MetaDetail? = null
    ): List<String> {
        val detail = prefetchedDetail ?: if (StremioId.isTmdbLikeContentId(id) || type != "series") {
            withTimeoutOrNull(timeoutMs) {
                repository.getMetaDetail(
                    type = type,
                    id = StremioId.baseContentId(id),
                    language = language,
                    authKey = profile?.authKey.orEmpty(),
                    localAddons = profile?.safeLocalAddons.orEmpty(),
                    useConfiguredAddons = true
                )
            }
        } else {
            null
        }
        return FluxaCoreNative.playbackStreamRequestIds(type, id, detail?.id)
    }

}
