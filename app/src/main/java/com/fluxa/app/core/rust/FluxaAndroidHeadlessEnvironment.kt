package com.fluxa.app.core.rust

import android.content.Context
import android.util.Log
import com.fluxa.app.core.StremioId
import com.fluxa.app.BuildConfig
import com.fluxa.app.data.local.*
import com.fluxa.app.data.local.LibraryRemoteSource
import com.fluxa.app.data.local.OfflineDownloadManager
import com.fluxa.app.data.local.OfflineSubtitleOption
import com.fluxa.app.data.local.ProfileManager
import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.data.local.WatchlistManager
import com.fluxa.app.data.repository.ExternalSyncPushCoordinator
import com.fluxa.app.data.repository.NuvioAccountImportCoordinator
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
import com.fluxa.app.data.remote.TmdbMeta
import com.fluxa.app.data.remote.TmdbService
import com.fluxa.app.data.remote.TraktApi
import com.fluxa.app.data.repository.AddonRepository
import com.fluxa.app.data.repository.StremioRepository
import com.fluxa.app.data.repository.TraktRepository
import com.fluxa.app.domain.discovery.StreamDiscoveryRequest
import com.fluxa.app.domain.discovery.StreamDiscoveryUseCase
import com.fluxa.app.player.MediaPlayerController
import com.fluxa.app.player.TorrentStreamManager
import com.fluxa.app.player.TorrentStreamResult
import com.fluxa.app.data.repository.CloudStreamCatalogClient
import com.fluxa.app.data.repository.HttpEffectExecutor
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
import com.fluxa.app.domain.discovery.buildDiscoverContentTypes
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
import com.fluxa.app.ui.catalog.NuvioPlaybackProgressPushWorker
import com.fluxa.app.ui.catalog.StremioPlaybackProgressPushWorker
import com.google.gson.Gson
import com.google.gson.JsonArray
import com.google.gson.JsonElement
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
import okhttp3.MediaType.Companion.toMediaType
import okhttp3.OkHttpClient
import okhttp3.RequestBody.Companion.toRequestBody
import javax.inject.Named

data class StreamProgressUpdate(
    val requestId: String,
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
    internal val externalSyncPushCoordinator: ExternalSyncPushCoordinator,
    internal val nuvioAccountImportCoordinator: NuvioAccountImportCoordinator,
    internal val httpEffectExecutor: HttpEffectExecutor,
    @param:Named("PluginScraperClient") internal val pluginScraperHttpClient: OkHttpClient
) : HeadlessPlatformEnvironment {

    internal val primeScope = CoroutineScope(SupervisorJob() + Dispatchers.IO)

    private val _streamProgressFlow = MutableSharedFlow<StreamProgressUpdate>(replay = 0, extraBufferCapacity = 32)
    val streamProgressFlow: SharedFlow<StreamProgressUpdate> = _streamProgressFlow

    private val authEffectHandler = AndroidAuthEffectHandler(
        repository = repository,
        traktRepository = traktRepository,
        watchlistManager = watchlistManager,
        nuvioAccountImportCoordinator = nuvioAccountImportCoordinator,
        gson = gson
    )

    private val calendarEffectHandler = AndroidCalendarEffectHandler(
        context = context,
        repository = repository,
        watchlistManager = watchlistManager,
        gson = gson
    )

    private val offlineEffectHandler = AndroidOfflineEffectHandler(context, gson)
    private val cloudStreamRuntime = AndroidCloudStreamRuntime(pluginManager)
    private val trailerHttpClient = OkHttpClient.Builder().build()

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
                "fetchPluginManifest" -> fetchPluginManifest(effect)
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
                "fetchDiscoverPage" -> fetchCatalogPage(effect)
                "fetchSeasonEpisodes" -> fetchSeasonEpisodes(effect)
                "fetchSubtitles" -> fetchSubtitles(effect)
                "runExternalSync",
                "runAuthFlow",
                "exchangeAuthCode",
                "refreshAuthToken",
                "syncExternalIntegration" -> authEffectHandler.execute(effect)
                "writeSettings" -> writeSettings(effect)
                "readCalendarMonth",
                "replaceExternalContinueWatching",
                "updateCalendarWidget",
                "notifyReleasedEpisodes" -> calendarEffectHandler.execute(effect)
                "enqueueOfflineDownload" -> offlineEffectHandler.enqueue(effect)
                "fetchYoutubeTrailerWatchConfig",
                "fetchYoutubeTrailerPlayer" -> executeTrailerHttpEffect(effect)
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
                val headers = stream.resolveHeaders()
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
                        _streamProgressFlow.emit(StreamProgressUpdate(requestId, streams, completedAddons, loadingAddons))
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

    private fun fetchPluginManifest(effect: NativeHeadlessEffect): HeadlessEffectCompletion {
        val manifestUrl = effect.payload.string("manifestUrl")
        val result = httpEffectExecutor.execute(pluginScraperHttpClient, manifestUrl)
        val body = result.body
        val statusCode = result.statusCode
        if (result.error != null || statusCode == null || statusCode !in 200..299 || body == null) {
            return error(effect, result.error ?: "http_${statusCode ?: 0}")
        }
        val manifest = FluxaCoreUniFfi.coreInvokeValue("pluginManifestParse", body)
        return ok(effect, mapOf("manifestUrl" to manifestUrl, "manifest" to manifest))
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

    private suspend fun readHomeBootstrap(effect: NativeHeadlessEffect): HeadlessEffectCompletion = coroutineScope {
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
        val semaphore = Semaphore(8)
        val categories = metadataFeeds.map { feed ->
            async {
                val items = semaphore.withPermit {
                    runCatching {
                        addonRepository.getAddonCatalog(
                            transportUrl = feed.transportUrl,
                            type = feed.type,
                            id = feed.id,
                            genre = feed.genre
                        )
                    }.getOrDefault(emptyList())
                }
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
        }.awaitAll().filterNotNull()
        ok(
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
        profileId?.let(watchlistManager::setActiveProfile)
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
                val seriesId = command.string("seriesId")
                val previouslyWatchedVideoIds = watchlistManager.getLocalWatchedVideoIds(seriesId)
                val localWatched = watchlistManager.markEpisodesWatched(
                seriesId = seriesId,
                videoIds = command.list("videoIds").mapNotNull { it?.toString() },
                watched = watched
                )
                if (profile != null) {
                    val meta = command.objectValue("meta")?.let { gson.fromJson(gson.toJsonTree(it), Meta::class.java) }
                    val episodes = command.list("episodes").mapNotNull {
                        runCatching { gson.fromJson(gson.toJsonTree(it), Video::class.java) }.getOrNull()
                    }
                    val episodesToPush = if (watched && meta?.type != "movie") {
                        episodes.filter { it.id != null && it.id !in previouslyWatchedVideoIds }
                    } else {
                        episodes
                    }
                    val shouldPush = meta?.type == "movie" || episodesToPush.isNotEmpty()
                    if (meta != null && shouldPush) {
                        primeScope.launch {
                            runCatching { externalSyncPushCoordinator.pushMarkWatched(profile, meta, episodesToPush, watched) }
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
            StremioPlaybackProgressPushWorker.enqueue(context, gson, profile.id, meta, timeOffset, duration)
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
        if (profile != null && !profile.nuvioAccessToken.isNullOrBlank()) {
            NuvioPlaybackProgressPushWorker.enqueue(
                context = context,
                profileId = profile.id,
                contentId = meta.id,
                contentType = meta.type,
                videoId = progress.stringOrNull("lastVideoId"),
                position = timeOffset,
                duration = duration
            )
        }
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
            ?: catalogOptions.take(1)
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
        val contentTypes = buildDiscoverContentTypes(addons)
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
        return ok(effect, mapOf("catalogs" to catalogOptions, "genres" to genres, "contentTypes" to contentTypes))
    }

    private suspend fun fetchCatalogPage(effect: NativeHeadlessEffect): HeadlessEffectCompletion {
        val payload = effect.payload
        val remoteSources = payload.remoteSources()
        if (remoteSources.isNotEmpty()) {
            return ok(
                effect,
                mapOf(
                    "items" to fetchRemoteCollectionSources(
                        sources = remoteSources,
                        skip = payload.number("skip")?.toInt() ?: 0,
                        profile = payload.profile()
                    )
                )
            )
        }
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

    private suspend fun fetchRemoteCollectionSources(
        sources: List<LibraryRemoteSource>,
        skip: Int,
        profile: UserProfile?
    ): List<Meta> = coroutineScope {
        sources.map { source ->
            async {
                when (source.provider.trim().lowercase()) {
                    "trakt" -> fetchTraktCollectionSource(source, skip)
                    "tmdb" -> fetchTmdbCollectionSource(source, skip, profile)
                    else -> emptyList()
                }
            }
        }.awaitAll().flatten().distinctBy { "${it.type}:${it.id}" }
    }

    private suspend fun fetchTraktCollectionSource(source: LibraryRemoteSource, skip: Int): List<Meta> {
        if (!TraktIntegration.hasClient(BuildConfig.TRAKT_CLIENT_ID)) return emptyList()
        val listId = source.traktListId ?: return emptyList()
        val isSeries = source.mediaType.equals("series", ignoreCase = true) || source.mediaType.equals("show", ignoreCase = true) || source.mediaType.equals("tv", ignoreCase = true)
        return TraktApi.create().getListItems(
            listId = listId,
            type = if (isSeries) "show" else "movie",
            apiKey = BuildConfig.TRAKT_CLIENT_ID,
            page = (skip / 50) + 1,
            sortBy = source.sortBy,
            sortHow = source.sortHow
        ).mapNotNull { item ->
            val summary = (if (isSeries) item.show else item.movie) ?: return@mapNotNull null
            val id = summary.ids.imdb ?: summary.ids.tmdb?.let { "tmdb:$it" } ?: return@mapNotNull null
            Meta(
                id = id,
                name = summary.title ?: return@mapNotNull null,
                type = if (isSeries) "series" else "movie",
                poster = null,
                releaseInfo = summary.year?.toString(),
                runtime = summary.runtime?.toString()
            )
        }
    }

    private suspend fun fetchTmdbCollectionSource(source: LibraryRemoteSource, skip: Int, profile: UserProfile?): List<Meta> {
        val apiKey = profile?.safeTmdbApiKey.orEmpty()
        val sourceId = source.tmdbId ?: return emptyList()
        if (apiKey.isBlank()) return emptyList()
        val mediaType = if (source.mediaType.equals("series", true) || source.mediaType.equals("tv", true) || source.mediaType.equals("show", true)) "tv" else "movie"
        val sourceType = source.tmdbSourceType.orEmpty().uppercase()
        val path = when (sourceType) {
            "LIST" -> "list/$sourceId"
            "COLLECTION" -> "collection/$sourceId"
            "PERSON", "DIRECTOR" -> "person/$sourceId/combined_credits"
            "COMPANY" -> "discover/$mediaType"
            "NETWORK" -> "discover/tv"
            else -> "discover/$mediaType"
        }
        val url = Uri.parse("https://api.themoviedb.org/3/$path").buildUpon()
            .appendQueryParameter("api_key", apiKey)
            .appendQueryParameter("language", profile?.safeLanguage ?: "en")
            .apply {
                if (sourceType !in setOf("COLLECTION", "PERSON", "DIRECTOR")) {
                    appendQueryParameter("page", (skip / 20 + 1).toString())
                }
                when (sourceType) {
                    "COMPANY" -> appendQueryParameter("with_companies", sourceId.toString())
                    "NETWORK" -> appendQueryParameter("with_networks", sourceId.toString())
                }
                if (sourceType !in setOf("LIST", "COLLECTION", "PERSON", "DIRECTOR")) {
                    appendQueryParameter("sort_by", source.sortBy ?: "popularity.desc")
                }
                val filters = source.filters.orEmpty()
                mapOf(
                    "year" to if (mediaType == "tv") "first_air_date_year" else "year",
                    "withGenres" to "with_genres",
                    "watchRegion" to "watch_region",
                    "voteCountGte" to "vote_count.gte",
                    "withKeywords" to "with_keywords",
                    "withNetworks" to "with_networks",
                    "withCompanies" to "with_companies",
                    "releaseDateGte" to if (mediaType == "tv") "first_air_date.gte" else "primary_release_date.gte",
                    "releaseDateLte" to if (mediaType == "tv") "first_air_date.lte" else "primary_release_date.lte",
                    "voteAverageGte" to "vote_average.gte",
                    "voteAverageLte" to "vote_average.lte",
                    "withOriginCountry" to "with_origin_country",
                    "withWatchProviders" to "with_watch_providers",
                    "withOriginalLanguage" to "with_original_language"
                ).forEach { (input, output) -> filters[input]?.let { appendQueryParameter(output, it.toString()) } }
            }
            .build()
            .toString()
        val root = TmdbService.create().getCollectionSource(url)
        val items = root.asJsonObjectOrNull()?.let { objectNode ->
            when {
                sourceType == "DIRECTOR" -> objectNode.getAsJsonArrayOrNull("crew")?.filter { it.asJsonObjectOrNull()?.get("job")?.asString == "Director" && it.asJsonObjectOrNull()?.get("media_type")?.asString == mediaType }
                sourceType == "PERSON" -> objectNode.getAsJsonArrayOrNull("cast")?.filter { it.asJsonObjectOrNull()?.get("media_type")?.asString == mediaType }
                else -> listOf("results", "parts", "items", "cast", "crew").firstNotNullOfOrNull { key -> objectNode.getAsJsonArrayOrNull(key) }
            }
        } ?: emptyList<JsonElement>()
        return items.mapNotNull { item ->
            runCatching { gson.fromJson(item, TmdbMeta::class.java) }.getOrNull()?.toCollectionMeta(mediaType)
        }
    }

    private fun TmdbMeta.toCollectionMeta(defaultMediaType: String): Meta? {
        val type = if (media_type == "tv" || defaultMediaType == "tv") "series" else "movie"
        val title = if (type == "series") name else title
        return title?.let {
            Meta(
                id = "tmdb:$id",
                name = it,
                type = type,
                poster = posterPath?.let { path -> "https://image.tmdb.org/t/p/w500$path" },
                background = backdropPath?.let { path -> "https://image.tmdb.org/t/p/w1280$path" },
                description = overview,
                releaseInfo = (if (type == "series") first_air_date else release_date)?.take(4),
                originalName = original_name
            )
        }
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

    private fun writeSettings(effect: NativeHeadlessEffect): HeadlessEffectCompletion {
        return ok(
            effect,
            mapOf(
                "key" to effect.payload.string("key"),
                "value" to effect.payload["value"]
            )
        )
    }

    private fun executeTrailerHttpEffect(effect: NativeHeadlessEffect): HeadlessEffectCompletion {
        val payload = effect.payload
        val headers = payload.objectValue("headers")?.mapValues { it.value.toString() }.orEmpty()
        val method = payload.string("method", "GET")
        val body = payload["body"]?.let {
            gson.toJson(it).toRequestBody("application/json".toMediaType())
        }
        val result = httpEffectExecutor.execute(trailerHttpClient, payload.string("url"), method, headers, body)
        val statusCode = result.statusCode
        if (result.error != null || statusCode == null || statusCode !in 200..299) {
            return error(effect, "http_${statusCode ?: 0}")
        }
        return ok(effect, mapOf("body" to result.body))
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

    private fun Map<String, Any?>.remoteSources(): List<LibraryRemoteSource> {
        val raw = this["remoteSource"] ?: return emptyList()
        val values = raw as? List<*> ?: listOf(raw)
        return values.mapNotNull { value ->
            runCatching { gson.fromJson(gson.toJsonTree(value), LibraryRemoteSource::class.java) }.getOrNull()
        }
    }

    private fun JsonElement.asJsonObjectOrNull() = takeIf { it.isJsonObject }?.asJsonObject

    private fun com.google.gson.JsonObject.getAsJsonArrayOrNull(key: String): JsonArray? =
        get(key)?.takeIf { it.isJsonArray }?.asJsonArray

    internal fun isTmdbContentId(id: String): Boolean =
        id.startsWith("tmdb:", ignoreCase = true) || id.toIntOrNull() != null

    internal suspend fun loadCsNativeMetaDetail(id: String): MetaDetail? = cloudStreamRuntime.loadMetaDetail(id)

    internal suspend fun loadCsNativeStreams(id: String, directTimeoutMs: Long = 30_000L): List<Stream> =
        cloudStreamRuntime.loadStreams(id, directTimeoutMs)

    internal fun csQualityScore(quality: String): Int = cloudStreamRuntime.qualityScore(quality)

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
