package com.fluxa.app.ui.catalog

import com.fluxa.app.common.AppStrings
import com.fluxa.app.ui.toMeta
import com.fluxa.app.data.local.*
import com.fluxa.app.data.remote.*

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.fluxa.app.core.fromState
import com.fluxa.app.core.fromStateList
import com.fluxa.app.core.rust.FluxaAndroidHeadlessEnvironment
import com.fluxa.app.core.rust.FluxaCoreNative
import com.fluxa.app.core.rust.StreamProgressUpdate
import com.fluxa.app.core.StremioId
import com.fluxa.app.data.repository.CommunityDiscussionRepository
import com.fluxa.app.data.repository.TraktIntegration
import com.fluxa.app.data.repository.library.ProviderAdapters
import com.fluxa.app.data.repository.library.ProviderCapability
import com.fluxa.app.data.repository.library.ThirdPartyProviderRepository
import com.fluxa.app.core.rust.FluxaHeadlessRuntimeFactory
import com.fluxa.app.domain.discovery.supportsStremioResource
import com.google.gson.Gson
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.async
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.*
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext

import dagger.hilt.android.lifecycle.HiltViewModel
import dagger.hilt.android.qualifiers.ApplicationContext
import javax.inject.Inject

private data class DetailInitialDecoded(
    val detail: MetaDetail?,
    val trailers: List<DetailTrailer>,
    val userAddons: List<AddonDescriptor>
)

private data class DetailLocalDecoded(
    val savedPlayback: Meta?,
    val localWatchedVideoIds: Set<String>,
    val userAddons: List<AddonDescriptor>
)

private data class DetailSecondaryDecoded(
    val watchedVideoIds: List<String>,
    val similarItems: List<Meta>,
    val trailers: List<DetailTrailer>,
    val ratings: List<MetaRating>
)

private data class DetailStreamsDecoded(
    val streams: List<Stream>,
    val availableAddons: List<String>,
    val loadingAddonNames: List<String>,
    val visibleStreams: List<Stream>
)

@HiltViewModel
class DetailViewModel @Inject constructor(
    @param:ApplicationContext private val applicationContext: android.content.Context,
    private val headlessEnvironment: FluxaAndroidHeadlessEnvironment,
    private val gson: Gson,
    private val communityDiscussionRepository: CommunityDiscussionRepository,
    private val providerAdapters: ProviderAdapters,
    private val thirdPartyProviderRepository: ThirdPartyProviderRepository
) : ViewModel() {

    private val _uiState = MutableStateFlow(DetailUiState())
    val uiState: StateFlow<DetailUiState> = _uiState.asStateFlow()

    private var currentProfile: UserProfile? = null
    private var currentSeriesLookupId: String? = null
    private var currentStrictProviderData: Boolean = false
    private var currentProviderId: ThirdPartyProviderId? = null
    private var currentProviderAccountId: String? = null
    private var detailLoadGeneration = 0L
    private var detailLoadJob: Job? = null
    private var trailerJob: Job? = null
    private var streamsFetchJob: Job? = null
    private var activeStreamsRequestIds: Set<String> = emptySet()

    private val headlessRuntime = FluxaHeadlessRuntimeFactory.createUniFfi(headlessEnvironment)
    private val libraryCommandRuntime = FluxaHeadlessRuntimeFactory.createUniFfi(headlessEnvironment)
    private val downloadCoordinator by lazy {
        DetailDownloadCoordinator(
            runtime = headlessRuntime,
            gson = gson,
            currentDetail = { _uiState.value.detail },
            seasonEpisodes = { _uiState.value.seasonEpisodes },
            userAddons = { _uiState.value.userAddons },
            currentProfile = { currentProfile },
            buildRequestIds = ::buildStreamRequestIds,
            fetchSubtitles = ::getSubtitlesFromAddon
        )
    }

    init {
        viewModelScope.launch {
            headlessEnvironment.streamProgressFlow.collect { progress ->
                if (_uiState.value.isLoadingStreams && progress.requestId in activeStreamsRequestIds) {
                    val sel = _uiState.value.selectedAddon
                    _uiState.update {
                        it.copy(
                            streams = progress.streams,
                            filteredStreams = if (sel == null) progress.streams
                                             else progress.streams.filter { s -> s.addonName == sel },
                            availableAddons = progress.completedAddonNames,
                            loadingAddonNames = progress.loadingAddonNames
                        )
                    }
                }
            }
        }
    }

    fun loadDetail(
        type: String,
        id: String,
        profile: UserProfile? = null,
        sourceAddonTransportUrl: String? = null,
        sourceAddonCatalogType: String? = null,
        initialMeta: Meta? = null,
        strictProviderData: Boolean = false,
        providerId: String? = null,
        providerAccountId: String? = null
    ) {
        val generation = ++detailLoadGeneration
        detailLoadJob?.cancel()
        trailerJob?.cancel()
        DetailTrailerPreloader.discard()
        currentProfile = profile
        currentStrictProviderData = strictProviderData
        currentProviderId = ThirdPartyProviderId.from(providerId)
        currentProviderAccountId = providerAccountId?.takeIf(String::isNotBlank)
        currentSeriesLookupId = normalizeSeriesLookupId(id)
        val lang = profile?.language ?: "en"
        val initialDetail = initialMeta?.toInitialDetailFallback()
        detailLoadJob = viewModelScope.launch {
            _uiState.update {
                it.copy(
                    detail = initialDetail,
                    seasonEpisodes = emptyList(),
                    localWatchedVideoIds = emptySet(),
                    savedPlayback = null,
                    similarItems = emptyList(),
                    trailers = emptyList(),
                    trailerUrl = null,
                    traktComments = emptyList(),
                    mdblistDiscussion = emptyList(),
                    hasStreamProviders = true,
                    isLoading = true
                )
            }

            try {
                launch { refreshStreamProviderState(profile) }
                if (strictProviderData && initialDetail != null) {
                    val localState = headlessRuntime.dispatch(
                        mapOf(
                            "type" to "detailLocalStateRequested",
                            "primaryId" to id,
                            "fallbackId" to null,
                            "contentType" to type,
                            "profile" to profile,
                            "skipLibraryState" to true
                        )
                    ).state["detail"] as? Map<*, *>
                    val provider = strictProviderAdapter(profile)
                    val providerId = currentProviderId
                    val providerSnapshot = if (provider != null && providerId != null && profile != null) {
                        thirdPartyProviderRepository.load(profile, providerId, refresh = true)
                            ?.takeIf { currentProviderAccountId == null || it.accountId == currentProviderAccountId }
                    } else null
                    val identity = TraktIntegration.contentIdentityKey(initialDetail.toMeta())
                    val providerState = Triple(
                        providerSnapshot?.libraryItems.orEmpty().any {
                            TraktIntegration.contentIdentityKey(it) == identity
                        },
                        providerSnapshot?.favorites.orEmpty().any {
                            TraktIntegration.contentIdentityKey(it) == identity
                        },
                        providerSnapshot?.watchedEpisodeIdsBySeries?.get(id).orEmpty()
                    )
                    if (generation != detailLoadGeneration) return@launch
                    val episodes = if (type == "series") {
                        withContext(Dispatchers.Default) {
                            FluxaCoreNative.detailSeasonVideos(
                                gson.toJson(mapOf("season" to 1, "videos" to initialDetail.videos.orEmpty()))
                            )
                        }
                    } else emptyList()
                    val userAddons = withContext(Dispatchers.Default) {
                        gson.fromStateList<AddonDescriptor>(localState?.get("userAddons"))
                    }
                    _uiState.update {
                        it.copy(
                            detail = initialDetail,
                            seasonEpisodes = episodes,
                            watchedVideoIds = providerState.third.toList(),
                            localWatchedVideoIds = emptySet(),
                            savedPlayback = null,
                            similarItems = emptyList(),
                            trailers = emptyList(),
                            trailerUrl = null,
                            traktComments = emptyList(),
                            mdblistDiscussion = emptyList(),
                            hasStreamProviders = localState?.get("hasStreamProviders") as? Boolean ?: false,
                            userAddons = userAddons,
                            isInWatchlist = providerState.first,
                            feedback = providerState.second.takeIf {
                                provider != null && ProviderCapability.FAVORITES in provider.capabilities
                            },
                            supportsWatchlist = provider != null && ProviderCapability.LIBRARY in provider.capabilities,
                            supportsLike = provider != null && ProviderCapability.FAVORITES in provider.capabilities,
                            isLoading = false
                        )
                    }
                    return@launch
                }
                val headlessResult = headlessRuntime.dispatch(
                    mapOf(
                        "type" to "detailLoadRequested",
                        "contentType" to type,
                        "id" to id,
                        "language" to lang,
                        "sourceAddonTransportUrl" to sourceAddonTransportUrl,
                        "sourceAddonCatalogType" to sourceAddonCatalogType,
                        "profile" to profile
                    )
                )
                val detailState = headlessResult.state["detail"] as? Map<*, *>
                val initialDecoded = withContext(Dispatchers.Default) {
                    DetailInitialDecoded(
                        detail = gson.fromState(detailState?.get("meta")),
                        trailers = gson.fromStateList(detailState?.get("trailers")),
                        userAddons = gson.fromStateList(detailState?.get("userAddons"))
                    )
                }
                val result = initialDecoded.detail
                if (generation != detailLoadGeneration) return@launch
                _uiState.update {
                    it.copy(
                        detail = result ?: initialDetail,
                        trailers = initialDecoded.trailers,
                        userAddons = initialDecoded.userAddons,
                        isLoading = false
                    )
                }

                val loadPlan = FluxaCoreNative.detailLoadPlan(
                    gson.toJson(
                        mapOf(
                            "requestedType" to type,
                            "requestedId" to id,
                            "season" to 1,
                            "detail" to (result ?: initialDetail)
                        )
                    )
                )
                val effectiveType = loadPlan.effectiveType.ifBlank { type }
                currentSeriesLookupId = loadPlan.seriesLookupId.takeIf { effectiveType == "series" }
                val resolvedId = loadPlan.resolvedId.ifBlank { result?.id ?: id }
                android.util.Log.d("CS3Detail", "loadDetail: type=$type effectiveType=$effectiveType resolvedId=${resolvedId.take(30)} resultVideos=${result?.videos?.size ?: "null"}")
                val localState = headlessRuntime.dispatch(
                    mapOf(
                        "type" to "detailLocalStateRequested",
                        "primaryId" to resolvedId,
                        "fallbackId" to id,
                        "contentType" to effectiveType,
                        "profile" to profile
                    )
                ).state["detail"] as? Map<*, *>

                if (generation != detailLoadGeneration) return@launch
                val decodedLocal = withContext(Dispatchers.Default) {
                    DetailLocalDecoded(
                        savedPlayback = gson.fromState(localState?.get("savedPlayback")),
                        localWatchedVideoIds = gson.fromStateList<String>(localState?.get("localWatchedVideoIds")).toSet(),
                        userAddons = gson.fromStateList(localState?.get("userAddons"))
                    )
                }
                _uiState.update {
                    it.copy(
                        savedPlayback = decodedLocal.savedPlayback,
                        localWatchedVideoIds = decodedLocal.localWatchedVideoIds,
                        hasStreamProviders = localState?.get("hasStreamProviders") as? Boolean ?: false,
                        userAddons = decodedLocal.userAddons,
                        isInWatchlist = localState?.get("isInWatchlist") as? Boolean ?: false,
                        feedback = localState?.get("feedback") as? Boolean
                    )
                }

                launch {
                    val current = currentProfile ?: return@launch
                    val content = _uiState.value.detail ?: return@launch
                    val trakt = if (current.traktCommentsEnabled == true) {
                        communityDiscussionRepository.traktComments(content.id, content.type)
                    } else emptyList()
                    val mdblist = communityDiscussionRepository.mdblistDiscussion(content.id, content.type, current.mdblistApiKey.orEmpty())
                    if (generation == detailLoadGeneration) {
                        _uiState.update { it.copy(traktComments = trakt, mdblistDiscussion = mdblist) }
                    }
                }

                launch {
                    if (effectiveType == "series") loadSeason(resolvedId, loadPlan.season)
                    val secondary = headlessRuntime.dispatch(
                        mapOf(
                            "type" to "detailSecondaryRequested",
                            "contentType" to effectiveType,
                            "id" to resolvedId,
                            "language" to lang,
                            "profile" to profile
                        )
                    ).state["detail"] as? Map<*, *>
                    if (generation != detailLoadGeneration) return@launch
                    val decodedSecondary = withContext(Dispatchers.Default) {
                        DetailSecondaryDecoded(
                            watchedVideoIds = gson.fromStateList(secondary?.get("watchedVideoIds")),
                            similarItems = gson.fromStateList(secondary?.get("similarItems")),
                            trailers = gson.fromStateList(secondary?.get("trailers")),
                            ratings = gson.fromStateList(secondary?.get("mdblistRatings"))
                        )
                    }
                    _uiState.update {
                        val currentDetail = it.detail
                        it.copy(
                            watchedVideoIds = decodedSecondary.watchedVideoIds,
                            similarItems = decodedSecondary.similarItems,
                            trailers = if (it.trailers.isEmpty()) decodedSecondary.trailers else it.trailers,
                            detail = currentDetail?.copy(
                                ratings = (currentDetail.ratings.orEmpty() + decodedSecondary.ratings)
                                    .distinctBy { rating -> rating.source.lowercase() }
                            )
                        )
                    }
                    maybeAutoPlayDetailTrailer(generation, profile)
                }

                launch {
                    headlessRuntime.dispatch(
                        mapOf(
                            "type" to "detailPrefetchRequested",
                            "contentType" to effectiveType,
                            "id" to id,
                            "streamLookupId" to loadPlan.streamLookupId,
                            "title" to loadPlan.title,
                            "originalName" to loadPlan.originalName,
                            "year" to loadPlan.year,
                            "language" to lang,
                            "profile" to profile
                        )
                    )
                }

            } catch (e: CancellationException) {
                throw e
            } catch (e: Exception) {
                e.printStackTrace()
                if (generation == detailLoadGeneration) {
                    _uiState.update { it.copy(isLoading = false) }
                }
            }
        }
    }

    private fun maybeAutoPlayDetailTrailer(generation: Long, profile: UserProfile?) {
        if (profile?.safeTrailerOnDetailHeroEnabled != true) return
        trailerJob?.cancel()
        val delaySeconds = profile.safeTrailerOnDetailHeroDelaySeconds
        trailerJob = viewModelScope.launch {
            val trailerResolution = async {
                resolvePlayableTrailerUrl(_uiState.value.trailers, headlessRuntime::dispatch)
                    ?.also { resolved -> DetailTrailerPreloader.preload(applicationContext, resolved) }
            }
            if (delaySeconds > 0) delay(delaySeconds * 1000L)
            if (generation != detailLoadGeneration) return@launch
            val resolved = trailerResolution.await() ?: return@launch
            if (generation != detailLoadGeneration) return@launch
            _uiState.update { it.copy(trailerUrl = resolved) }
        }
    }

    private fun Meta.toInitialDetailFallback(): MetaDetail {
        return MetaDetail(
            id = id,
            type = type,
            name = name,
            genres = genres,
            poster = poster,
            background = background,
            logo = logo,
            description = description,
            releaseInfo = releaseInfo,
            released = released,
            runtime = runtime,
            videos = videos,
            trailers = trailers,
            imdbRating = imdbRating,
            ageRating = ageRating,
            ratings = ratings,
            cast = cast,
            director = null,
            links = null,
            status = null,
            seasonsCount = seasonsCount,
            platforms = null,
            awards = awards,
            originalLanguage = originalLanguage,
            originalName = originalName,
            country = null,
            productionCompanies = null,
            networks = null,
            collectionName = null,
            collectionId = null,
            collectionParts = null,
            seasonPosters = null
        )
    }

    private suspend fun refreshStreamProviderState(profile: UserProfile?) {
        val detail = headlessRuntime.dispatch(
            mapOf(
                "type" to "detailLocalStateRequested",
                "primaryId" to (currentSeriesLookupId ?: ""),
                "fallbackId" to null,
                "contentType" to "series",
                "profile" to profile
            )
        ).state["detail"] as? Map<*, *>
        _uiState.update { it.copy(hasStreamProviders = detail?.get("hasStreamProviders") as? Boolean ?: false) }
    }

    suspend fun shuffleEpisode(): String? {
        val detail = _uiState.value.detail ?: return null
        if (detail.type != "series") return null

        val allVideos = detail.videos.orEmpty().filter { (it.season ?: 0) > 0 && !detailIsUpcoming(it.released) }
        if (allVideos.isNotEmpty()) return allVideos.random().id

        if (currentStrictProviderData) return null

        val seasonsCount = detail.seasonsCount ?: return null
        if (seasonsCount <= 0) return null
        val season = (1..seasonsCount).random()
        val seasonId = currentSeriesLookupId ?: normalizeSeriesLookupId(detail.id)
        val profile = currentProfile
        val lang = currentProfile?.language ?: "en"
        val result = headlessRuntime.dispatch(
            mapOf(
                "type" to "detailSeasonRequested",
                "seriesId" to seasonId,
                "season" to season,
                "profile" to profile,
                "language" to lang
            )
        )
        val detailMap = result.state["detail"] as? Map<*, *>
        val episodes: List<Video> = withContext(Dispatchers.Default) {
            gson.fromStateList(detailMap?.get("seasonEpisodes"))
        }
        return episodes.filter { !detailIsUpcoming(it.released) }.randomOrNull()?.id
    }

    fun loadSeason(id: String, seasonNumber: Int) {
        android.util.Log.d("CS3Detail", "loadSeason called: id=${id.take(40)}, season=$seasonNumber, detailVideos=${_uiState.value.detail?.videos?.size ?: "null"}")
        val lang = currentProfile?.language ?: "en"
        viewModelScope.launch {
            _uiState.update { it.copy(seasonEpisodes = emptyList()) }

            val allVideos = _uiState.value.detail?.videos.orEmpty()
            if (allVideos.isNotEmpty()) {
                android.util.Log.d("CS3Detail", "loadSeason from detail: detail=${_uiState.value.detail?.name}, videosInDetail=${allVideos.size}")
                val episodes = withContext(Dispatchers.Default) {
                    FluxaCoreNative.detailSeasonVideos(
                        gson.toJson(mapOf("season" to seasonNumber, "videos" to allVideos))
                    )
                }
                _uiState.update { it.copy(seasonEpisodes = episodes) }
                return@launch
            }
            if (currentStrictProviderData) return@launch

            val seasonId = currentSeriesLookupId ?: normalizeSeriesLookupId(_uiState.value.detail?.id ?: id)
            val profile = currentProfile
            val result = headlessRuntime.dispatch(
                mapOf(
                    "type" to "detailSeasonRequested",
                    "seriesId" to seasonId,
                    "season" to seasonNumber,
                    "profile" to profile,
                    "language" to lang
                )
            )
            val detail = result.state["detail"] as? Map<*, *>
            val episodes = withContext(Dispatchers.Default) {
                gson.fromStateList<Video>(detail?.get("seasonEpisodes"))
            }
            _uiState.update { it.copy(seasonEpisodes = episodes) }
        }
    }


    fun markEpisodeWatched(seriesId: String, episode: Video, watched: Boolean = true) {
        val videoId = episode.id
        if (videoId.isBlank()) return
        viewModelScope.launch {
            if (currentStrictProviderData) {
                val profile = currentProfile ?: return@launch
                val provider = strictProviderAdapter(profile)
                    ?.takeIf { ProviderCapability.WATCH_HISTORY in it.capabilities }
                    ?: return@launch
                val previous = _uiState.value.watchedVideoIds
                _uiState.update { state ->
                    state.copy(watchedVideoIds = if (watched) (state.watchedVideoIds + videoId).distinct()
                    else state.watchedVideoIds - videoId)
                }
                val success = thirdPartyProviderRepository.pushWatched(
                    profile = profile,
                    providerId = currentProviderId ?: return@launch,
                    expectedAccountId = currentProviderAccountId,
                    item = _uiState.value.detail?.toMeta() ?: return@launch,
                    episodes = listOf(episode),
                    watched = watched
                )
                if (!success) _uiState.update { it.copy(watchedVideoIds = previous) }
                return@launch
            }
            _uiState.update {
                it.copy(localWatchedVideoIds = if (watched) it.localWatchedVideoIds + videoId
                                               else it.localWatchedVideoIds - videoId)
            }
            headlessRuntime.dispatch(
                mapOf(
                    "type" to "markWatchedRequested",
                    "seriesId" to seriesId,
                    "videoIds" to listOf(videoId),
                    "watched" to watched,
                    "meta" to (_uiState.value.detail?.toMeta()),
                    "episodes" to listOf(episode),
                    "profile" to currentProfile
                )
            )
        }
    }

    fun markSeasonWatched(seriesId: String, episodes: List<Video>, watched: Boolean? = null) {
        if (episodes.isEmpty()) return
        viewModelScope.launch {
            val targetIds = episodes.map { it.id }.filter { it.isNotBlank() }.toSet()
            if (targetIds.isEmpty()) return@launch
            if (currentStrictProviderData) {
                val profile = currentProfile ?: return@launch
                val provider = strictProviderAdapter(profile)
                    ?.takeIf { ProviderCapability.WATCH_HISTORY in it.capabilities }
                    ?: return@launch
                val previous = _uiState.value.watchedVideoIds
                val shouldMarkWatched = watched ?: !targetIds.all { it in previous }
                _uiState.update { state ->
                    state.copy(watchedVideoIds = if (shouldMarkWatched) (state.watchedVideoIds + targetIds).distinct()
                    else state.watchedVideoIds - targetIds)
                }
                val success = thirdPartyProviderRepository.pushWatched(
                    profile = profile,
                    providerId = currentProviderId ?: return@launch,
                    expectedAccountId = currentProviderAccountId,
                    item = _uiState.value.detail?.toMeta() ?: return@launch,
                    episodes = episodes,
                    watched = shouldMarkWatched
                )
                if (!success) _uiState.update { it.copy(watchedVideoIds = previous) }
                return@launch
            }
            val currentlyWatched = _uiState.value.localWatchedVideoIds
            val shouldMarkWatched = watched ?: !targetIds.all { it in currentlyWatched }
            _uiState.update {
                it.copy(localWatchedVideoIds = if (shouldMarkWatched) it.localWatchedVideoIds + targetIds
                                               else it.localWatchedVideoIds - targetIds)
            }
            headlessRuntime.dispatch(
                mapOf(
                    "type" to "markWatchedRequested",
                    "seriesId" to seriesId,
                    "videoIds" to targetIds.toList(),
                    "watched" to shouldMarkWatched,
                    "meta" to (_uiState.value.detail?.toMeta()),
                    "episodes" to episodes,
                    "profile" to currentProfile
                )
            )
        }
    }

    private fun strictProviderAdapter(profile: UserProfile?) =
        currentProviderId
            ?.takeIf { currentStrictProviderData && profile != null }
            ?.let { providerId ->
                val actualAccountId = profile?.providerAccountId(providerId)
                val expectedAccountId = currentProviderAccountId
                if (expectedAccountId != null && actualAccountId != expectedAccountId) null
                else providerAdapters.byId(providerId.key)
                    ?.takeIf { adapter -> profile != null && adapter.isConnected(profile) }
            }

    private fun normalizeSeriesLookupId(rawId: String): String {
        return FluxaCoreNative.detailSeriesLookupId(rawId)
    }

    fun abandonShow(profile: UserProfile) {
        if (currentStrictProviderData) return
        viewModelScope.launch {
            val currentDetail = _uiState.value.detail ?: return@launch
            headlessRuntime.dispatch(
                mapOf(
                    "type" to "clearPlaybackProgressRequested",
                    "profile" to profile,
                    "meta" to currentDetail.toMeta()
                )
            )
            loadDetail(currentDetail.type, currentDetail.id, profile)
        }
    }

    fun toggleWatchlist() {
        val currentDetail = _uiState.value.detail ?: return
        val meta = currentDetail.toMeta()
        val previous = _uiState.value.isInWatchlist
        val profile = currentProfile
        val strictProvider = strictProviderAdapter(profile)
        if (currentStrictProviderData && (strictProvider == null || ProviderCapability.LIBRARY !in strictProvider.capabilities)) {
            return
        }
        _uiState.update { it.copy(isInWatchlist = !previous) }
        viewModelScope.launch {
            try {
                if (strictProvider != null && profile != null) {
                    val success = thirdPartyProviderRepository.pushWatchlist(
                        profile = profile,
                        providerId = currentProviderId ?: return@launch,
                        expectedAccountId = currentProviderAccountId,
                        item = meta,
                        add = !previous
                    )
                    if (!success) _uiState.update { it.copy(isInWatchlist = previous) }
                } else {
                    val result = libraryCommandRuntime.dispatch(
                        mapOf(
                            "type" to "toggleWatchlistRequested",
                            "item" to meta,
                            "profile" to currentProfile
                        )
                    )
                    val library = result.state["library"] as? Map<*, *>
                    val write = library?.get("lastWrite") as? Map<*, *>
                    (write?.get("isInWatchlist") as? Boolean)?.let { confirmed ->
                        _uiState.update { it.copy(isInWatchlist = confirmed) }
                    }
                }
            } catch (e: Exception) {
                _uiState.update { it.copy(isInWatchlist = previous) }
            }
        }
    }

    fun setFeedback(isLike: Boolean) {
        val currentDetail = _uiState.value.detail ?: return
        val previous = _uiState.value.feedback
        val newValue = if (previous == isLike) null else isLike
        val profile = currentProfile
        val strictProvider = strictProviderAdapter(profile)
        if (currentStrictProviderData && (strictProvider == null || ProviderCapability.FAVORITES !in strictProvider.capabilities)) {
            return
        }
        _uiState.update { it.copy(feedback = newValue) }
        viewModelScope.launch {
            try {
                if (strictProvider != null && profile != null) {
                    val success = thirdPartyProviderRepository.pushFavorite(
                        profile = profile,
                        providerId = currentProviderId ?: return@launch,
                        expectedAccountId = currentProviderAccountId,
                        item = currentDetail.toMeta(),
                        favorite = newValue == true
                    )
                    if (!success) _uiState.update { it.copy(feedback = previous) }
                } else {
                    val result = headlessRuntime.dispatch(
                        mapOf(
                            "type" to "setFeedbackRequested",
                            "id" to currentDetail.id,
                            "value" to newValue,
                            "meta" to currentDetail.toMeta()
                        )
                    )
                    val detail = result.state["detail"] as? Map<*, *>
                    _uiState.update { it.copy(feedback = detail?.get("feedback") as? Boolean) }
                }
            } catch (e: Exception) {
                _uiState.update { it.copy(feedback = previous) }
            }
        }
    }

    fun setSelectedAddon(addon: String?) {
        viewModelScope.launch {
            val result = withContext(Dispatchers.Default) {
                headlessRuntime.dispatch(
                    mapOf(
                        "type" to "detailSelectedAddonChanged",
                        "addon" to addon
                    )
                )
            }
            val detail = result.state["detail"] as? Map<*, *>
            val selectedAddon = detail?.get("selectedAddon") as? String
            val visibleStreams = withContext(Dispatchers.Default) {
                gson.fromStateList<Stream>(detail?.get("visibleStreams"))
            }
            _uiState.update {
                it.copy(
                    selectedAddon = selectedAddon,
                    filteredStreams = visibleStreams
                )
            }
        }
    }

    fun fetchStreamsForSelection(type: String, id: String, context: android.content.Context? = null) {
        streamsFetchJob?.cancel()
        streamsFetchJob = viewModelScope.launch {
            try {
                val language = currentProfile?.language ?: "en"
                val requestIds = buildStreamRequestIds(type, id, language)
                activeStreamsRequestIds = requestIds.toSet()
                android.util.Log.d("Detail", " FETCHING STREAMS: type=$type, id=$id, requestIds=$requestIds")
                _uiState.update {
                    it.copy(
                        isLoadingStreams = true,
                        streams = emptyList(),
                        filteredStreams = emptyList(),
                        selectedAddon = null,
                        availableAddons = emptyList(),
                        loadingAddonNames = emptyList()
                    )
                }

                val isCs3Content = id.startsWith("cs3:")
                if (!_uiState.value.hasStreamProviders && !isCs3Content) {
                    showToast(context, AppStrings.t(currentProfile?.language, "auto.no_addons_found"))
                    _uiState.update { it.copy(isLoading = false) }
                    return@launch
                }

                val result = headlessRuntime.dispatch(
                    mapOf(
                        "type" to "detailStreamsRequested",
                        "contentType" to type,
                        "requestIds" to requestIds,
                        "detail" to _uiState.value.detail,
                        "seasonEpisodes" to _uiState.value.seasonEpisodes,
                        "language" to language,
                        "profile" to currentProfile
                    )
                )
                if (activeStreamsRequestIds != requestIds.toSet()) return@launch
                val detail = result.state["detail"] as? Map<*, *>
                applyDetailStreamState(detail)
                val fetchedStreams: List<Stream> = withContext(Dispatchers.Default) {
                    gson.fromStateList(detail?.get("streams"))
                }
                val resolvedRequestId = detail?.get("resolvedRequestId")?.toString() ?: requestIds.first()

                if (fetchedStreams.isNotEmpty()) {
                    android.util.Log.d("Detail", " FOUND ${fetchedStreams.size} streams for $resolvedRequestId")
                } else {
                    android.util.Log.w("Detail", " NO STREAMS found for ${requestIds.joinToString()}")
                    showToast(context, AppStrings.t(currentProfile?.language, "auto.no_sources_found"))
                }
            } catch (e: CancellationException) {
                throw e
            } catch (e: Exception) {
                android.util.Log.e("Detail", " FATAL FETCH ERROR for $id", e)
            } finally {
                _uiState.update { it.copy(isLoadingStreams = false, loadingAddonNames = emptyList()) }
            }
        }
    }

    private suspend fun applyDetailStreamState(detail: Map<*, *>?) {
        val decoded = withContext(Dispatchers.Default) {
            DetailStreamsDecoded(
                streams = gson.fromStateList(detail?.get("streams")),
                availableAddons = gson.fromStateList(detail?.get("availableAddons")),
                loadingAddonNames = gson.fromStateList(detail?.get("loadingAddonNames")),
                visibleStreams = gson.fromStateList(detail?.get("visibleStreams"))
            )
        }
        _uiState.update {
            it.copy(
                streams = decoded.streams,
                filteredStreams = decoded.visibleStreams,
                selectedAddon = detail?.get("selectedAddon") as? String,
                availableAddons = decoded.availableAddons,
                loadingAddonNames = decoded.loadingAddonNames,
                hasStreamProviders = detail?.get("hasStreamProviders") as? Boolean ?: it.hasStreamProviders
            )
        }
    }

    private fun buildStreamRequestIds(type: String, id: String, language: String): List<String> {
        if (id.startsWith("cs3:")) return listOf(id)
        val detailId = _uiState.value.detail?.id
        val canonicalBaseId = resolveCanonicalStreamBaseId(type, id, language)
        return StremioId.streamRequestIds(
            type = type,
            id = id,
            detailId = detailId,
            currentSeriesLookupId = currentSeriesLookupId,
            canonicalBaseId = canonicalBaseId
        )
    }

    private fun resolveCanonicalStreamBaseId(type: String, id: String, language: String): String? {
        StremioId.imdbId(id)?.let { return it }
        StremioId.imdbId(_uiState.value.detail?.id)?.let { return it }
        StremioId.imdbId(currentSeriesLookupId)?.let { return it }
        return null
    }

    suspend fun getSubtitlesFromAddon(baseUrl: String, type: String, id: String, extra: String = ""): List<SubtitleData> {
        val result = headlessRuntime.dispatch(
            mapOf(
                "type" to "addonResourceRequested",
                "transportUrl" to baseUrl,
                "resource" to "subtitles",
                "contentType" to type,
                "id" to id,
                "extra" to mapOf("extraArgs" to extra)
            )
        )
        val addons = result.state["addons"] as? Map<*, *>
        return withContext(Dispatchers.Default) {
            gson.fromStateList(addons?.get("lastResourceResult"))
        }
    }

    fun downloadEpisodes(episodes: List<Video?>, context: android.content.Context? = null) {
        val profile = currentProfile
        viewModelScope.launch {
            val queued = downloadCoordinator.queueEpisodes(episodes)
            val key = if (queued > 0) "downloads.queued" else "downloads.failed"
            showToast(context, AppStrings.t(profile?.safeLanguage, key), android.widget.Toast.LENGTH_SHORT)
        }
    }

    private fun showToast(context: android.content.Context?, message: String, length: Int = android.widget.Toast.LENGTH_LONG) {
        context?.let { android.widget.Toast.makeText(it, message, length).show() }
    }

    fun resetAutoSelect() {
        _uiState.update { it.copy(autoSelectedStream = null) }
    }

    override fun onCleared() {
        DetailTrailerPreloader.discard()
        headlessRuntime.close()
        libraryCommandRuntime.close()
        super.onCleared()
    }
}
