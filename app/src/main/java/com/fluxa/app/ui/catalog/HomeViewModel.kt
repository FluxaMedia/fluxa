package com.fluxa.app.ui.catalog

import com.fluxa.app.common.ReleaseDateUtils
import com.fluxa.app.data.local.*
import com.fluxa.app.data.remote.*
import com.fluxa.app.data.repository.*
import com.fluxa.app.core.rust.FluxaAndroidHeadlessEnvironment
import com.fluxa.app.core.rust.FluxaCoreNative
import com.fluxa.app.core.rust.FluxaCoreUniFfi
import com.fluxa.app.core.rust.FluxaUniFfiCoreStateHandle
import com.fluxa.app.core.rust.FluxaHeadlessRuntimeFactory
import com.fluxa.app.domain.discovery.DiscoverCatalogOption
import com.fluxa.app.domain.discovery.MetadataFeedOption
import com.fluxa.app.domain.discovery.buildCs3MetadataFeedOptions
import com.fluxa.app.domain.discovery.effectiveHomeMetadataFeedSelection
import com.fluxa.app.domain.discovery.isMetadataFeedEnabled
import com.google.gson.Gson
import com.google.gson.reflect.TypeToken
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.delay
import kotlinx.coroutines.isActive
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import java.util.Locale

import com.fluxa.app.plugins.PluginManager
import com.fluxa.app.data.repository.CloudStreamCatalogClient
import kotlinx.coroutines.async
import kotlinx.coroutines.awaitAll
import dagger.hilt.android.lifecycle.HiltViewModel
import javax.inject.Inject

@HiltViewModel
class HomeViewModel @Inject constructor(
    private val repository: StremioRepository,
    private val traktRepository: TraktRepository,
    private val addonRepository: AddonRepository,
    private val watchlistManager: WatchlistManager,
    private val watchlistStore: WatchlistStore,
    private val searchHistoryStore: SearchHistoryStore,
    private val homeCategoryCache: HomeCategoryCache,
    private val homeBillboardCache: HomeBillboardCache,
    private val forgottenContinueWatchingStore: ForgottenContinueWatchingStore,
    private val coordinatorFactory: HomeViewModelCoordinatorFactory,
    private val externalSyncPushCoordinator: ExternalSyncPushCoordinator,
    private val headlessEnvironment: FluxaAndroidHeadlessEnvironment,
    private val nuvioSyncCoordinator: NuvioSyncCoordinator,
    private val platformContentGateway: HomePlatformContentGateway,
    private val imdbApiService: ImdbApiService,
    private val gson: Gson,
    @dagger.hilt.android.qualifiers.ApplicationContext context: android.content.Context
) : ViewModel() {

    private val appContext = context.applicationContext
    private val metaListType = object : TypeToken<List<Meta>>() {}.type
    private val streamListType = object : TypeToken<List<Stream>>() {}.type
    private val trailerListType = object : TypeToken<List<DetailTrailer>>() {}.type
    private val categoryListType = object : TypeToken<List<HomeCategory>>() {}.type
    private val videoListType = object : TypeToken<List<Video>>() {}.type
    private val subtitleListType = object : TypeToken<List<SubtitleData>>() {}.type
    private val introTimestampsListType = object : TypeToken<List<IntroTimestamps>>() {}.type
    private val addonListType = object : TypeToken<List<AddonDescriptor>>() {}.type
    private val headlessRuntime = FluxaHeadlessRuntimeFactory.createUniFfi(headlessEnvironment)
    private val initialSearchHistory = searchHistoryStore.load(null)
    private val coreState: FluxaUniFfiCoreStateHandle = FluxaCoreUniFfi.createAppCoreState(
        mapOf(
            "home" to mapOf(
                "categories" to emptyList<HomeCategory>(),
                "isLoading" to false,
                "currentFilter" to "all",
                "isDirectLoading" to false,
                "traktContinueWatchingLastUpdatedAt" to 0L,
                "userAddons" to emptyList<AddonDescriptor>(),
                "watchlist" to emptyList<Meta>(),
                "likedItems" to emptyList<Meta>(),
                "activeProfile" to null,
                "currentWatchlist" to emptyList<Meta>(),
                "externalContinueWatching" to emptyList<Meta>(),
                "traktWatchedState" to TraktWatchedState()
            ),
            "homeSearch" to mapOf(
                "searchHistory" to initialSearchHistory
            ),
            "billboard" to emptyMap<String, Any?>(),
            "discover" to emptyMap<String, Any?>(),
            "calendar" to emptyMap<String, Any?>(),
            "library" to mapOf("uiState" to LibraryUiState())
        )
    )
    private val categoryState = HomeCategoryStateStore()
    val categories: StateFlow<List<HomeCategory>> = categoryState.categories
    val collectionFolderCategories: StateFlow<Map<String, HomeCategory>> = categoryState.folderCategories

    var savedHomeScrollIndex: Int = 0
    var savedHomeScrollOffset: Int = 0
    var savedTvHomeScrollIndex: Int = 0
    var savedTvHomeScrollOffset: Int = 0
    var savedTvFocusedRowIndex: Int = -1
    val savedCategoryScrollPositions: HashMap<String, Pair<Int, Int>> = HashMap()

    private val _isLoading = MutableStateFlow(false)
    val isLoading: StateFlow<Boolean> = _isLoading

    private val _hasLoadedHome = MutableStateFlow(false)
    val hasLoadedHome: StateFlow<Boolean> = _hasLoadedHome.asStateFlow()

    private val _currentFilter = MutableStateFlow("all")
    val currentFilter: StateFlow<String> = _currentFilter

    private val searchFocusState = HomeSearchFocusStateHolder(
        initialHistory = initialSearchHistory,
        coreState = coreState,
        ownsCoreState = false,
        gson = gson
    )
    val searchResults: StateFlow<List<Meta>> = searchFocusState.searchResults
    val searchRows: StateFlow<List<SearchResultRow>> = searchFocusState.searchRows

    private val browseCoordinator by lazy {
        HomeHeadlessBrowseCoordinator(
            scope = viewModelScope,
            gson = gson,
            dispatch = ::dispatchHeadless,
            activeProfile = { currentActiveProfile },
            userAddons = { _userAddons.value }
        )
    }
    val discoverUiState: StateFlow<DiscoverUiState> get() = browseCoordinator.discoverUiState
    val discoverGenres: StateFlow<List<DiscoverGenreOption>> get() = browseCoordinator.discoverGenres
    val calendarUiState: StateFlow<CalendarUiState> get() = browseCoordinator.calendarUiState

    private val syncCoordinator by lazy {
        HomeHeadlessSyncCoordinator(
            scope = viewModelScope,
            gson = gson,
            dispatch = ::dispatchHeadless,
            activeProfile = { currentActiveProfile },
            setActiveProfile = { setActiveProfileState(it) },
            setWatchlist = ::setWatchlistState,
            setContinueWatching = ::setCurrentWatchlistState,
            setExternalContinueWatching = ::setExternalContinueWatchingState,
            setLiked = ::setLikedItemsState,
            refreshDynamicRows = ::refreshDynamicRows
        )
    }

    private val billboardState = HomeBillboardStateHolder()
    val billboardError: StateFlow<String?> = billboardState.error
    val billboardPool: StateFlow<List<Meta>> = billboardState.pool
    val billboardIndex: StateFlow<Int> = billboardState.index
    val billboardMovie: StateFlow<Meta?> = billboardState.movie
    val billboardLogo: StateFlow<String?> = billboardState.logo
    val billboardWatchlist: StateFlow<Boolean> = billboardState.watchlist
    val billboardNextEpisode: StateFlow<String?> = billboardState.nextEpisode
    val billboardTrailerUrl: StateFlow<String?> = billboardState.trailerUrl
    val billboardTrailerSubtitleCues: StateFlow<List<com.fluxa.app.player.TrailerCue>> = billboardState.trailerSubtitleCues
    val billboardSeasonPosterUrl: StateFlow<String?> = billboardState.seasonPosterUrl

    private val _isDirectLoading = MutableStateFlow(false)
    val isDirectLoading: StateFlow<Boolean> = _isDirectLoading

    private val _traktContinueWatchingLastUpdatedAt = MutableStateFlow(0L)
    val traktContinueWatchingLastUpdatedAt: StateFlow<Long> = _traktContinueWatchingLastUpdatedAt.asStateFlow()

    private val _userAddons = MutableStateFlow<List<AddonDescriptor>>(emptyList())
    val userAddons: StateFlow<List<AddonDescriptor>> = _userAddons

    private val parentsGuideCache = java.util.concurrent.ConcurrentHashMap<String, List<ParentsGuideCategory>>()
    private val _parentsGuide = MutableStateFlow<List<ParentsGuideCategory>>(emptyList())
    val parentsGuide: StateFlow<List<ParentsGuideCategory>> = _parentsGuide.asStateFlow()

    fun loadParentsGuide(metaId: String) {
        val imdbId = com.fluxa.app.core.StremioId.imdbId(metaId)
        if (imdbId == null) {
            _parentsGuide.value = emptyList()
            return
        }
        parentsGuideCache[imdbId]?.let {
            _parentsGuide.value = it
            return
        }
        viewModelScope.launch {
            try {
                val guide = imdbApiService.getParentsGuide(imdbId).parentsGuide
                parentsGuideCache[imdbId] = guide
                _parentsGuide.value = guide
            } catch (e: Exception) {
                e.printStackTrace()
            }
        }
    }

    val searchHistory: StateFlow<List<Meta>> = searchFocusState.searchHistory
    val focusedMovie: StateFlow<Meta?> = searchFocusState.focusedMovie
    val focusedMovieTrailerUrl: StateFlow<String?> = searchFocusState.focusedMovieTrailerUrl
    val previewUrl: StateFlow<String?> = searchFocusState.previewUrl

    private val _watchlist = MutableStateFlow<List<Meta>>(emptyList())
    val watchlist: StateFlow<List<Meta>> = _watchlist.asStateFlow()

    private val _likedItems = MutableStateFlow<List<Meta>>(emptyList())
    val likedItems: StateFlow<List<Meta>> = _likedItems.asStateFlow()
    val totalWatchedContentDuration: StateFlow<Long> = watchlistStore
        .observeTotalWatchedDuration()
        .stateIn(viewModelScope, SharingStarted.WhileSubscribed(5_000), 0L)

    private val libraryCoordinator by lazy {
        coordinatorFactory.library(repository, traktRepository, watchlistManager, externalSyncPushCoordinator, viewModelScope, coreState, gson)
    }
    val libraryUiState: StateFlow<LibraryUiState> get() = libraryCoordinator.state

    fun loadLibraryItems(activeProfile: UserProfile?) {
        libraryCoordinator.load(activeProfile)
    }

    suspend fun loadFolderSections(
        folder: com.fluxa.app.data.local.LibraryUserCollectionFolder
    ): List<Pair<String, List<Meta>>> {
        return feedCoordinator.fetchFolderSections(folder, currentActiveProfile?.safeLanguage ?: "en")
    }

    val loadedCs3ApiNames: StateFlow<List<String>> = platformContentGateway.loadedApis
        .map { apis -> apis.map { it.name } }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val loadedCs3CatalogFeedOptions: StateFlow<List<MetadataFeedOption>> = platformContentGateway.loadedApis
        .map { apis -> buildCs3MetadataFeedOptions(apis.toCs3CatalogFeedDescriptors()) }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    private var currentWatchlist: List<Meta> = emptyList()
    private val _currentContinueWatchingCount = MutableStateFlow(0)
    val currentContinueWatchingCount: StateFlow<Int> = _currentContinueWatchingCount.asStateFlow()
    private val _syncingProviders = MutableStateFlow<Set<String>>(emptySet())
    val syncingProviders: StateFlow<Set<String>> = _syncingProviders.asStateFlow()

    fun setProviderSyncing(provider: String, syncing: Boolean) {
        _syncingProviders.value = if (syncing) {
            _syncingProviders.value + provider
        } else {
            _syncingProviders.value - provider
        }
    }
    private var externalContinueWatching: List<Meta> = emptyList()
    private var traktWatchedState: TraktWatchedState = TraktWatchedState()
    private var currentActiveProfile: UserProfile? = null
    private var hasFetchedFreshBillboard = false
    private var searchJob: Job? = null
    private val _isSearchLoading = MutableStateFlow(false)
    val isSearchLoading: StateFlow<Boolean> = _isSearchLoading.asStateFlow()
    private val pagingCoordinator by lazy {
        HomeCatalogPagingCoordinator(
            scope = viewModelScope,
            platformContentGateway = platformContentGateway,
            activeProfile = { currentActiveProfile },
            categories = categoryState::currentCategories,
            setCategories = ::setCategoriesState,
            folderCategories = categoryState::currentFolderCategories,
            setFolderCategories = categoryState::replaceFolderCategories,
            normalizeItems = ::normalizeCatalogItems,
            dispatch = ::dispatchHeadless,
            decodeItems = { fromStateList(it, metaListType) }
        )
    }
    private val playbackController by lazy {
        coordinatorFactory.playback(
            context = appContext,
            repository = repository,
            watchlistManager = watchlistManager,
            forgottenStore = forgottenContinueWatchingStore,
            gson = gson,
            scope = viewModelScope,
            activeProfile = { currentActiveProfile },
            localContinueWatching = { currentWatchlist },
            externalContinueWatching = { externalContinueWatching },
            onContinueWatchingChanged = { snapshot ->
                setCurrentWatchlistState(snapshot.localItems)
                setExternalContinueWatchingState(snapshot.externalItems)
                setWatchlistState(snapshot.localItems)
            },
            refreshDynamicRows = ::refreshDynamicRows
        )
    }

    private val continueWatchingCoordinator by lazy {
        coordinatorFactory.continueWatching(
            repository = repository,
            watchlistManager = watchlistManager,
            scope = viewModelScope,
            activeProfile = { currentActiveProfile },
            localItems = { currentWatchlist },
            externalItems = { externalContinueWatching },
            watchedState = { traktWatchedState },
            setLocalItems = ::setCurrentWatchlistState,
            setExternalItems = ::setExternalContinueWatchingState,
            setWatchlistState = ::setWatchlistState,
            setTraktUpdatedAt = ::setTraktUpdatedAtState,
            refreshDynamicRows = ::refreshDynamicRows,
            getConfiguredMetaDetail = ::getConfiguredMetaDetail,
            getSeasonEpisodes = ::getSeasonEpisodes
        )
    }

    private val billboardRuntime by lazy {
        HomeBillboardRuntime(
            scope = viewModelScope,
            watchlistManager = watchlistManager,
            pool = { billboardState.poolValue },
            setPool = { billboardState.poolValue = it },
            index = { billboardState.indexValue },
            setIndex = { billboardState.indexValue = it },
            categories = categoryState::currentCategories,
            language = { currentActiveProfile?.safeLanguage ?: "en" },
            setMovie = { billboardState.movieValue = it },
            setLogo = { billboardState.logoValue = it },
            watchlistValue = { billboardState.watchlistValue },
            setWatchlist = { billboardState.watchlistValue = it },
            setTrailerUrl = { billboardState.trailerUrlValue = it },
            setTrailerSubtitleCues = { billboardState.trailerSubtitleCuesValue = it },
            setNextEpisode = { billboardState.nextEpisodeValue = it },
            setSeasonPosterUrl = { billboardState.seasonPosterUrlValue = it },
            getMetaDetail = { type, id ->
                val profile = currentActiveProfile
                platformContentGateway.addonMetaDetail(type, id, profile?.authKey ?: "", profile?.safeLocalAddons.orEmpty())
            },
            parseSeasonEpisode = ::formatSeasonEpisode,
            prefetchDirectPlayback = ::prefetchDirectPlayback,
            activeProfile = { currentActiveProfile },
            getTrailers = { type, id, lang -> getConfiguredMetaDetailResult(type, id, lang).trailers },
            dispatchHeadless = headlessRuntime::dispatch
        )
    }

    private val billboardLoader by lazy {
        HomeBillboardLoader(
            addonRepository = addonRepository,
            scope = viewModelScope,
            getMetadataFeeds = { profile -> feedCoordinator.getMetadataFeeds(profile) },
            getCs3MetadataFeeds = { loadedCs3CatalogFeedOptions.value },
            fetchCs3FeedItems = { feed ->
                platformContentGateway.cloudFeedItems(feed.key)
            },
            setPool = { pool ->
                billboardState.poolValue = pool
                homeBillboardCache.save(currentActiveProfile, pool)
            },
            updateContent = billboardRuntime::updateContent,
            normalizePool = { items -> items.distinctBy(HomeBillboardRanking::contentIdentityKey) },
            startRotation = billboardRuntime::startRotation
        )
    }

    private suspend fun dispatchHeadless(action: Any) = withContext(Dispatchers.Default) {
        headlessRuntime.dispatch(action)
    }

    private val feedCoordinator by lazy {
        HomeCatalogFeedCoordinator(
            repository = repository,
            addonRepository = addonRepository,
            scope = viewModelScope,
            userAddons = { _userAddons.value },
            setUserAddons = ::setUserAddonsState,
            continueWatchingItems = ::buildContinueWatchingItems,
            isUpcoming = continueWatchingCoordinator::isUpcoming,
            normalizeCatalogItems = ::normalizeCatalogItems,
            setCategories = ::setCategoriesState,
            currentCategories = categoryState::currentCategories
        )
    }

    private val focusCoordinator by lazy {
        HomeFocusCoordinator(
            scope = viewModelScope,
            focusedMovie = { searchFocusState.focusedMovieValue },
            setFocusedMovie = { searchFocusState.focusedMovieValue = it },
            setFocusedTrailer = { searchFocusState.focusedMovieTrailerUrlValue = it },
            setPreview = { searchFocusState.previewUrlValue = it },
            activeProfile = { currentActiveProfile },
            getConfiguredMetaDetail = ::getConfiguredMetaDetailResult
        )
    }

    private val dynamicRowsCoordinator by lazy {
        HomeDynamicRowsCoordinator(
            scope = viewModelScope,
            categories = categoryState::currentCategories,
            setCategories = ::setCategoriesAndCache,
            activeProfile = { currentActiveProfile },
            buildUserCollectionHomeCategories = ::buildUserCollectionHomeCategories,
            buildContinueWatchingItems = ::buildContinueWatchingItems,
            isUpcoming = continueWatchingCoordinator::isUpcoming,
            optimizeHomeCategories = ::optimizeHomeCategories
        )
    }

    private val authCoordinator by lazy {
        HomeAuthCoordinator(
            scope = viewModelScope,
            gson = gson,
            dispatch = ::dispatchHeadless,
            activeProfile = { currentActiveProfile },
            updateActiveProfile = ::setActiveProfileState,
            invalidateHome = { setCategoriesState(emptyList()) }
        )
    }

    private val watchlistFlowBinder by lazy {
        HomeWatchlistFlowBinder(
            watchlistStore = watchlistStore,
            scope = viewModelScope,
            setWatchlist = ::setWatchlistState,
            setLocalContinueWatching = ::setCurrentWatchlistState,
            setExternalContinueWatching = ::setExternalContinueWatchingState,
            setLikedItems = ::setLikedItemsState,
            refreshDynamicRows = ::refreshDynamicRows,
            prefetchContinueWatchingArtwork = ::prefetchContinueWatchingArtwork
        )
    }

    private val cloudStreamCoordinator by lazy {
        HomeCloudStreamCoordinator(
            scope = viewModelScope,
            gateway = platformContentGateway,
            hasLoadedHome = _hasLoadedHome,
            activeProfile = { currentActiveProfile },
            categories = categoryState::currentCategories,
            setCategories = ::setCategoriesAndCache,
            billboardIsEmpty = { billboardState.poolValue.isEmpty() },
            refreshBillboard = billboardLoader::load
        )
    }

    init {
        watchlistFlowBinder.bind()
        cloudStreamCoordinator.bind()
    }

    private fun scheduleCs3Refresh() {
        cloudStreamCoordinator.refresh()
    }

    override fun onCleared() {
        searchFocusState.close()
        coreState.close()
        headlessRuntime.close()
        super.onCleared()
    }

    fun setFilter(filter: String) {
        if (_currentFilter.value == filter) return
        setCurrentFilterState(filter)
    }

    fun onMovieFocused(movie: Meta) {
        focusCoordinator.onMovieFocused(movie)
    }

    fun nextBillboard() {
        billboardRuntime.next()
    }

    fun prevBillboard() {
        billboardRuntime.previous()
    }

    fun jumpToBillboard(index: Int) {
        billboardRuntime.jumpTo(index)
    }

    fun syncBillboardIndex(index: Int) {
        billboardRuntime.syncIndex(index)
    }

    fun pauseBillboardRotation() {
        billboardRuntime.pauseRotation()
    }

    fun toggleWatchlist(meta: Meta) {
        viewModelScope.launch {
            val result = dispatchHeadless(mapOf("type" to "toggleWatchlistRequested", "item" to meta, "profile" to currentActiveProfile))
            val library = result.state["library"] as? Map<*, *>
            val write = library?.get("lastWrite") as? Map<*, *>
            setWatchlistState(fromStateList(write?.get("watchlist"), metaListType))
            refreshDynamicRows()
        }
    }

    fun addToWatchlist(meta: Meta) {
        viewModelScope.launch {
            if (!watchlistManager.isInWatchlist(meta.id)) toggleWatchlist(meta)
        }
    }

    fun toggleBillboardWatchlist() {
        val movie = billboardState.movieValue ?: return
        viewModelScope.launch {
            val result = dispatchHeadless(mapOf("type" to "toggleWatchlistRequested", "item" to movie, "profile" to currentActiveProfile))
            val library = result.state["library"] as? Map<*, *>
            val write = library?.get("lastWrite") as? Map<*, *>
            setWatchlistState(fromStateList(write?.get("watchlist"), metaListType))
            billboardState.watchlistValue = (write?.get("isInWatchlist") as? Boolean) ?: billboardState.watchlistValue
            refreshDynamicRows()
        }
    }

    fun setFeedback(movie: Meta, isLike: Boolean) {
        viewModelScope.launch {
            dispatchHeadless(
                mapOf(
                    "type" to "setFeedbackRequested",
                    "id" to movie.id,
                    "value" to isLike,
                    "meta" to movie
                )
            )
        }
    }

    fun savePlaybackProgress(meta: Meta, timeOffset: Long, duration: Long, videoId: String? = null, streamIndex: Int? = null, episodeName: String? = null, lastStreamUrl: String? = null, lastStreamTitle: String? = null, lastBingeGroup: String? = null, lastAudioLanguage: String? = null, lastSubtitleLanguage: String? = null, scrobbleTraktPause: Boolean = true) {
        viewModelScope.launch {
            dispatchHeadless(
                mapOf(
                    "type" to "savePlaybackProgressRequested",
                    "profile" to currentActiveProfile,
                    "meta" to meta,
                    "timeOffset" to timeOffset,
                    "duration" to duration,
                    "lastVideoId" to videoId,
                    "lastStreamIndex" to streamIndex,
                    "lastEpisodeName" to episodeName,
                    "lastStreamUrl" to lastStreamUrl,
                    "lastStreamTitle" to lastStreamTitle,
                    "lastBingeGroup" to lastBingeGroup,
                    "lastAudioLanguage" to lastAudioLanguage,
                    "lastSubtitleLanguage" to lastSubtitleLanguage,
                    "scrobbleTraktPause" to scrobbleTraktPause
                )
            )
            loadLibraryData(currentActiveProfile)
        }
    }

    fun scrobblePlayback(
        token: String,
        metaType: String,
        itemId: String,
        progress: Float,
        action: String
    ) {
        viewModelScope.launch {
            dispatchHeadless(
                mapOf(
                    "type" to "scrobbleRequested",
                    "token" to token,
                    "metaType" to metaType,
                    "itemId" to itemId,
                    "progress" to progress,
                    "actionName" to action,
                    "profile" to currentActiveProfile
                )
            )
        }
    }

    fun onNextEpisodeCardShown(meta: Meta, nextVideoId: String, activeProfile: UserProfile?) {
        viewModelScope.launch {
            dispatchHeadless(
                mapOf(
                    "type" to "playerNextEpisodeCardShown",
                    "contentType" to meta.type,
                    "seriesId" to meta.id,
                    "nextVideoId" to nextVideoId,
                    "title" to meta.name,
                    "originalName" to meta.originalName,
                    "year" to meta.releaseInfo?.toIntOrNull(),
                    "language" to ((activeProfile ?: currentActiveProfile)?.safeLanguage ?: "en"),
                    "profile" to (activeProfile ?: currentActiveProfile)
                )
            )
        }
    }

    fun markWatchedFromPlayback(meta: Meta, videoId: String? = null, episodeName: String? = null, nextEpisode: Video? = null, watchedDuration: Long = 0L) {
        viewModelScope.launch {
            currentActiveProfile?.id?.let(watchlistManager::setActiveProfile)
            watchlistManager.recordWatchedContentDuration(meta.id, videoId, watchedDuration)
            val episodes = if (meta.type == "series" && !videoId.isNullOrBlank()) {
                val parsed = com.fluxa.app.core.StremioId.parseEpisodeLocator(videoId)
                listOf(
                    Video(
                        id = videoId,
                        name = episodeName,
                        season = parsed?.first,
                        number = parsed?.second,
                        released = null,
                        thumbnail = meta.background
                    )
                )
            } else {
                emptyList()
            }
            dispatchHeadless(
                mapOf(
                    "type" to "markWatchedRequested",
                    "seriesId" to meta.id,
                    "videoIds" to listOfNotNull(videoId),
                    "watched" to true,
                    "meta" to meta,
                    "episodes" to episodes,
                    "profile" to currentActiveProfile
                )
            )
            if (meta.type == "series" && nextEpisode != null) {
                savePlaybackProgress(
                    meta = meta.copy(
                        lastVideoId = nextEpisode.id,
                        continueWatchingPoster = nextEpisode.thumbnail ?: meta.continueWatchingPoster,
                        continueWatchingBackground = nextEpisode.thumbnail ?: meta.continueWatchingBackground
                    ),
                    timeOffset = 0L,
                    duration = 0L,
                    videoId = nextEpisode.id,
                    episodeName = nextEpisode.continueWatchingTitleForHome()
                )
            }
            loadLibraryData(currentActiveProfile)
        }
    }

    fun forgetPlaybackProgress(meta: Meta) {
        viewModelScope.launch {
            dispatchHeadless(
                mapOf(
                    "type" to "clearPlaybackProgressRequested",
                    "profile" to currentActiveProfile,
                    "meta" to meta
                )
            )
            loadLibraryData(currentActiveProfile)
            refreshDynamicRows()
        }
    }

    suspend fun getStreams(type: String, id: String): List<Stream> {
        val result = dispatchHeadless(
            mapOf(
                "type" to "playerLoadStreamsRequested",
                "contentType" to type,
                "id" to id,
                "currentVideoId" to id,
                "initialVideoId" to id,
                "initialStreams" to emptyList<Stream>(),
                "initialStreamIndex" to 0
            )
        )
        val player = result.state["player"] as? Map<*, *>
        return fromStateList(player?.get("currentStreams"), streamListType)
    }

    internal suspend fun loadPlayerStreams(
        meta: Meta,
        currentVideoId: String?,
        initialVideoId: String?,
        initialStreams: List<Stream>,
        initialStreamIndex: Int,
        savedUrl: String?,
        savedTitle: String?,
        activeProfile: UserProfile?,
        preferredBingeGroup: String?
    ): PlayerRuntimeCoreState {
        val profile = activeProfile ?: currentActiveProfile
        val result = dispatchHeadless(
            mapOf(
                "type" to "playerLoadStreamsRequested",
                "contentType" to meta.type,
                "id" to (currentVideoId ?: meta.id),
                "currentVideoId" to currentVideoId,
                "initialVideoId" to initialVideoId,
                "initialStreams" to initialStreams,
                "initialStreamIndex" to initialStreamIndex,
                "savedUrl" to savedUrl,
                "savedTitle" to savedTitle,
                "sourceSelectionMode" to (profile?.safeStreamSourceSelectionMode ?: com.fluxa.app.player.STREAM_SOURCE_MODE_MANUAL),
                "regexPattern" to profile?.safeStreamSourceRegexPattern,
                "preferredBingeGroup" to preferredBingeGroup,
                "title" to meta.name,
                "originalName" to meta.originalName,
                "year" to meta.releaseInfo?.toIntOrNull(),
                "language" to (profile?.safeLanguage ?: "en"),
                "profile" to profile
            )
        )
        return fromStateObject(result.state["player"], PlayerRuntimeCoreState::class.java)
            ?: PlayerRuntimeCoreState(playerError = "generic")
    }

    internal suspend fun resolvePlayerPlayback(
        url: String,
        stream: Stream?,
        currentVideoId: String?,
        title: String
    ): PlayerRuntimeCoreState {
        val result = dispatchHeadless(
            mapOf(
                "type" to "playerResolvePlaybackRequested",
                "url" to url,
                "stream" to stream,
                "currentVideoId" to currentVideoId,
                "title" to title
            )
        )
        return fromStateObject(result.state["player"], PlayerRuntimeCoreState::class.java)
            ?: PlayerRuntimeCoreState(playerError = "generic")
    }

    suspend fun prepareDirectPlayback(meta: Meta): DirectPlaybackTarget? {
        setDirectLoadingState(true)
        try {
            val result = dispatchHeadless(
                mapOf(
                    "type" to "directPlaybackRequested",
                    "meta" to meta,
                    "profile" to currentActiveProfile,
                    "language" to (currentActiveProfile?.safeLanguage ?: "en")
                )
            )
            val player = result.state["player"] as? Map<*, *>
            return fromStateObject(player?.get("directPlaybackTarget"), DirectPlaybackTarget::class.java)
        } finally {
            setDirectLoadingState(false)
        }
    }

    suspend fun getSeasonEpisodes(id: String, seasonNumber: Int, language: String): List<Video> {
        val result = dispatchHeadless(
            mapOf(
                "type" to "detailSeasonRequested",
                "seriesId" to id,
                "season" to seasonNumber,
                "profile" to currentActiveProfile,
                "language" to language
            )
        )
        val detail = result.state["detail"] as? Map<*, *>
        return fromStateList(detail?.get("seasonEpisodes"), videoListType)
    }

    suspend fun getSubtitlesFromAddon(baseUrl: String, type: String, id: String, extra: String = ""): List<SubtitleData> {
        val result = dispatchHeadless(
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
        return fromStateList(addons?.get("lastResourceResult"), subtitleListType)
    }

    suspend fun getIntroSegments(
        imdbId: String,
        season: Int,
        episode: Int,
        title: String?,
        useIntroDb: Boolean,
        useAniSkip: Boolean
    ): List<IntroTimestamps> {
        val result = dispatchHeadless(
            mapOf(
                "type" to "introSegmentsRequested",
                "imdbId" to imdbId,
                "season" to season,
                "episode" to episode,
                "title" to title,
                "useIntroDb" to useIntroDb,
                "useAniSkip" to useAniSkip
            )
        )
        val player = result.state["player"] as? Map<*, *>
        return fromStateList(player?.get("introSegments"), introTimestampsListType)
    }

    suspend fun submitIntroSegment(
        apiKey: String,
        segmentType: String,
        imdbId: String,
        season: Int,
        episode: Int,
        startSec: Double,
        endSec: Double
    ): IntroDbSubmitResult {
        return repository.submitIntroSegment(apiKey, segmentType, imdbId, season, episode, startSec, endSec)
    }

    suspend fun resolvePlaybackIntroImdbId(meta: Meta, videoId: String?, language: String): String? {
        val result = dispatchHeadless(
            mapOf(
                "type" to "introImdbIdRequested",
                "meta" to meta,
                "videoId" to videoId,
                "language" to language
            )
        )
        val player = result.state["player"] as? Map<*, *>
        return player?.get("introImdbId") as? String
    }

    private suspend fun getConfiguredMetaDetail(type: String, id: String, language: String): MetaDetail? {
        return getConfiguredMetaDetailResult(type, id, language).detail
    }

    private suspend fun getConfiguredMetaDetailResult(type: String, id: String, language: String): HomeMetaDetailResult {
        val result = dispatchHeadless(
            mapOf(
                "type" to "metaDetailRequested",
                "contentType" to type,
                "id" to id,
                "language" to language,
                "profile" to currentActiveProfile
            )
        )
        val lookup = result.state["lookup"] as? Map<*, *>
        val detail = fromStateObject(lookup?.get("metaDetail"), MetaDetail::class.java)
        val addonTrailers = fromStateList<DetailTrailer>(lookup?.get("trailers"), trailerListType)
        if (addonTrailers.isNotEmpty()) {
            return HomeMetaDetailResult(detail = detail, trailers = addonTrailers)
        }
        val profile = currentActiveProfile
        val tmdbTrailers = if (profile?.safeTmdbApiKey?.isNotBlank() == true && profile.safeTmdbTrailersEnabled) {
            runCatching {
                platformContentGateway.trailers(type, id, language, profile.safeTmdbApiKey)
            }.getOrElse { emptyList() }
        } else {
            emptyList()
        }
        return HomeMetaDetailResult(detail = detail, trailers = tmdbTrailers)
    }

    suspend fun resolveExpandedPosterTrailer(meta: Meta): String? {
        val lang = currentActiveProfile?.safeLanguage ?: "en"
        val trailers = runCatching { getConfiguredMetaDetailResult(meta.type, meta.id, lang).trailers }.getOrElse { emptyList() }
        return resolvePlayableTrailerUrl(trailers, headlessRuntime::dispatch)
    }

    fun search(query: String) {
        searchJob?.cancel()
        if (query.isEmpty()) {
            searchFocusState.searchResultsValue = emptyList()
            searchFocusState.searchRowsValue = emptyList()
            _isSearchLoading.value = false
            return
        }
        searchJob = viewModelScope.launch {
            try {
                _isSearchLoading.value = true
                delay(400)
                if (!isActive) return@launch
                val trimmedQuery = query.trim()
                val allRows = platformContentGateway.searchRows(
                    query = trimmedQuery,
                    language = currentActiveProfile?.safeLanguage ?: "en",
                    authKey = currentActiveProfile?.authKey.orEmpty(),
                    localAddons = currentActiveProfile?.safeLocalAddons.orEmpty()
                )
                if (!isActive) return@launch
                searchFocusState.searchRowsValue = allRows
                searchFocusState.searchResultsValue = allRows.flatMap { it.items }.distinctBy { it.id }.take(80)
            } catch (e: Exception) {
                e.printStackTrace()
            } finally {
                _isSearchLoading.value = false
            }
        }
    }

    fun addToSearchHistory(meta: Meta) {
        val current = searchFocusState.searchHistoryValue.toMutableList()
        val existingIndex = current.indexOfFirst { it.id == meta.id || it.name.equals(meta.name, ignoreCase = true) }
        if (existingIndex != -1) current.removeAt(existingIndex)
        val updated = (listOf(meta.copy(description = null, cast = null, ratings = null, awards = null)) + current).take(10)
        searchHistoryStore.save(updated, currentActiveProfile)
        searchFocusState.searchHistoryValue = updated
    }

    fun recordSearchSelection(id: String, type: String) {
        val selected = searchFocusState.searchResultsValue.firstOrNull { meta ->
            meta.id == id && meta.type == type
        } ?: searchFocusState.searchResultsValue.firstOrNull { it.id == id }
        selected?.let(::addToSearchHistory)
    }

    fun clearSearchHistory() {
        searchHistoryStore.save(emptyList(), currentActiveProfile)
        searchFocusState.searchHistoryValue = emptyList()
    }

    fun loadMore(categoryId: String) {
        pagingCoordinator.loadMore(categoryId)
    }

    fun clearDiscoverResults() {
        browseCoordinator.clearResults()
    }

    fun discoverCatalogOptions(type: String): List<DiscoverCatalogOption> =
        browseCoordinator.catalogOptions(type)

    fun discoverContentTypes(): List<String> = browseCoordinator.availableContentTypes()

    fun setDiscoverLoading(isLoading: Boolean) {
        browseCoordinator.setLoading(isLoading)
    }

    fun discover(type: String, catalogKey: String?, genre: String?, year: String?, rating: Float?, provider: String?, region: String?) {
        browseCoordinator.discover(type, catalogKey, genre, year, rating, provider, region)
    }

    fun loadMoreDiscoverResults(transportUrl: String, contentType: String, catalogId: String, genre: String?) {
        browseCoordinator.loadMore(transportUrl, contentType, catalogId, genre)
    }

    fun loadCalendarMonth(activeProfile: UserProfile?, year: Int, month: Int, plannedItems: List<Meta> = emptyList()) {
        browseCoordinator.loadCalendar(activeProfile, year, month, plannedItems)
    }

    fun loadDiscoverGenres(type: String) {
        browseCoordinator.clearGenres()
    }

    fun loadDiscoverCatalogFilters(
        type: String,
        selectedCatalogKey: String?,
        onLoaded: ((List<DiscoverCatalogOption>) -> Unit)? = null
    ) {
        browseCoordinator.loadFilters(type, selectedCatalogKey, onLoaded)
    }

    fun setUserAddons(addons: List<AddonDescriptor>) {
        setUserAddonsState(addons)
    }

    fun refreshInstalledAddons(forceRefresh: Boolean = true) {
        viewModelScope.launch {
            val result = dispatchHeadless(
                mapOf(
                    "type" to "addonsRefreshRequested",
                    "profile" to currentActiveProfile,
                    "forceRefresh" to forceRefresh
                )
            )
            val home = result.state["home"] as? Map<*, *>
            setUserAddonsState(fromStateList(home?.get("userAddons"), addonListType))
        }
    }

    fun applyUpdatedProfile(profile: UserProfile, refreshHomeSideEffects: Boolean = true) {
        setActiveProfileState(profile)
        viewModelScope.launch {
            dispatchHeadless(mapOf("type" to "profileActivated", "profile" to profile))
            if (refreshHomeSideEffects) {
                refreshDynamicRows()
                scheduleCs3Refresh()
            }
        }
    }

    fun loadInitialData(activeProfile: UserProfile?, force: Boolean = false) {
        val profileChanged = activeProfile?.id != currentActiveProfile?.id
        if (profileChanged) {
            savedHomeScrollIndex = 0
            savedHomeScrollOffset = 0
            savedTvHomeScrollIndex = 0
            savedTvHomeScrollOffset = 0
            savedTvFocusedRowIndex = -1
            savedCategoryScrollPositions.clear()
        }
        if (categoryState.currentCategories().isEmpty()) {
            val cached = homeCategoryCache.load(activeProfile)
            if (cached.isNotEmpty()) {
                setCategoriesState(cached)
            }
        }
        viewModelScope.launch {
            setLoadingState(true)
            try {
                val result = dispatchHeadless(
                    mapOf(
                        "type" to "homeLoadRequested",
                        "profile" to activeProfile,
                        "language" to (activeProfile?.safeLanguage ?: "en"),
                        "force" to force
                    )
                )
                val home = result.state["home"] as? Map<*, *> ?: return@launch
                setActiveProfileState(activeProfile)
                val rawCategories = fromStateList<HomeCategory>(home["categories"], categoryListType)
                val filteredCategories = if (activeProfile?.homeFeedToggles != null) {
                    val allIds = rawCategories.map { it.id }
                    val selectedKeys = effectiveHomeMetadataFeedSelection(activeProfile.homeFeedToggles, allIds)
                    rawCategories.filter { isMetadataFeedEnabled(selectedKeys, it.id) }
                } else rawCategories
                setUserAddonsState(fromStateList(home["userAddons"], addonListType))
                setCategoriesAndCache(filteredCategories)
                setCurrentWatchlistState(fromStateList(home["continueWatching"], metaListType))
                setWatchlistState(fromStateList(home["watchlist"], metaListType))
                refreshDynamicRows()
            } finally {
                _hasLoadedHome.value = true
                setLoadingState(false)
            }
            scheduleCs3Refresh()
        }
        if (billboardState.poolValue.isEmpty()) {
            val cachedPool = homeBillboardCache.load(activeProfile)
            if (cachedPool.isNotEmpty()) {
                billboardState.poolValue = cachedPool
                viewModelScope.launch {
                    billboardRuntime.updateContent(cachedPool[0])
                    billboardRuntime.startRotation()
                }
            }
        }
        if (force || !hasFetchedFreshBillboard) {
            viewModelScope.launch {
                billboardLoader.load(activeProfile)
                hasFetchedFreshBillboard = true
            }
        }
    }

    fun refreshTraktTokenIfNeeded(profile: UserProfile, onProfileUpdated: (UserProfile) -> Unit) {
        authCoordinator.refreshTokenIfNeeded("trakt", profile, onProfileUpdated)
    }

    fun refreshMalTokenIfNeeded(profile: UserProfile, onProfileUpdated: (UserProfile) -> Unit) {
        authCoordinator.refreshTokenIfNeeded("mal", profile, onProfileUpdated)
    }

    fun refreshExternalContinueWatching() {
        currentActiveProfile?.let(::loadLibraryData)
    }

    fun loadLibraryData(activeProfile: UserProfile?) {
        syncCoordinator.loadLibrary(activeProfile)
    }

    fun syncTraktIntegration(
        profile: UserProfile,
        onProfileUpdated: (UserProfile) -> Unit,
        onComplete: (Boolean) -> Unit
    ) {
        syncCoordinator.syncTrakt(profile, onProfileUpdated, onComplete)
    }

    fun syncNuvioIntegration(
        profile: UserProfile,
        onProfileUpdated: (UserProfile) -> Unit,
        onComplete: (Boolean) -> Unit
    ) {
        syncCoordinator.syncNuvio(profile, onProfileUpdated, onComplete, ::loadLibraryData)
    }

    suspend fun isNuvioHealthy(): Boolean = nuvioSyncCoordinator.isHealthy()

    fun pushNuvioAddons(profile: UserProfile) {
        viewModelScope.launch {
            runCatching { nuvioSyncCoordinator.pushAddons(profile) }
        }
    }

    fun syncStremioIntegration(
        profile: UserProfile,
        onProfileUpdated: (UserProfile) -> Unit,
        onComplete: (Boolean) -> Unit
    ) {
        syncCoordinator.syncStremio(profile, onProfileUpdated, onComplete)
    }

    private fun buildUserCollectionHomeCategories(profile: UserProfile?, showAboveContinueWatching: Boolean? = null): List<HomeCategory> {
        return feedCoordinator.buildUserCollectionHomeCategories(profile, showAboveContinueWatching)
    }

    private fun refreshDynamicRows() {
        dynamicRowsCoordinator.refresh()
    }

    private fun buildContinueWatchingItems(lang: String): List<Meta> {
        return continueWatchingCoordinator.buildItems(lang, playbackController)
    }

    private fun prefetchContinueWatchingArtwork(items: List<Meta>) {
        continueWatchingCoordinator.prefetchArtwork(items)
    }

    private suspend fun normalizeCatalogItems(
        items: List<Meta>,
        catalogId: String,
        lang: String,
        genre: String? = null
    ): List<Meta> {
        return HomeCatalogItemNormalizer.normalize(items, catalogId, lang, genre)
    }

    private fun optimizeHomeCategories(categories: List<HomeCategory>, lang: String): List<HomeCategory> {
        return feedCoordinator.optimizeHomeCategories(categories, lang)
    }

    private var cacheSaveJob: Job? = null
    private fun setCategoriesAndCache(categories: List<HomeCategory>) {
        setCategoriesState(categories)
        val profile = currentActiveProfile
        cacheSaveJob?.cancel()
        cacheSaveJob = viewModelScope.launch(Dispatchers.Default) {
            delay(600)
            homeCategoryCache.save(profile, categories.filterNot { it.id == "continue_watching" })
        }
    }

    private fun setCategoriesState(categories: List<HomeCategory>) {
        categoryState.setCategories(categories)
    }

    private fun setLoadingState(isLoading: Boolean) {
        _isLoading.value = isLoading
    }

    private fun setCurrentFilterState(filter: String) {
        _currentFilter.value = filter.takeIf { it.isNotEmpty() } ?: "all"
    }

    private fun setDirectLoadingState(isLoading: Boolean) {
        _isDirectLoading.value = isLoading
    }

    private fun setTraktUpdatedAtState(updatedAt: Long) {
        _traktContinueWatchingLastUpdatedAt.value = updatedAt
    }

    private fun setUserAddonsState(addons: List<AddonDescriptor>) {
        _userAddons.value = addons
    }

    private fun setWatchlistState(items: List<Meta>) {
        _watchlist.value = items
    }

    private fun setLikedItemsState(items: List<Meta>) {
        _likedItems.value = items
    }

    private fun setActiveProfileState(profile: UserProfile?) {
        if (profile == null && currentActiveProfile != null) return
        currentActiveProfile = profile
        watchlistManager.setActiveProfile(profile?.id ?: "guest")
    }

    private fun setCurrentWatchlistState(items: List<Meta>) {
        currentWatchlist = items
        _currentContinueWatchingCount.value = items.size
    }

    private fun setExternalContinueWatchingState(items: List<Meta>) {
        externalContinueWatching = items
    }

    private fun setTraktWatchedState(state: TraktWatchedState) {
        traktWatchedState = state
    }

    private fun prefetchDirectPlayback(meta: Meta, detail: MetaDetail?) {
        val profile = currentActiveProfile
        val language = profile?.safeLanguage ?: "en"
        viewModelScope.launch(Dispatchers.Default) {
            val plan = FluxaCoreNative.directPlaybackPlan(meta, detail, ReleaseDateUtils.todayIso())
            dispatchHeadless(
                mapOf(
                    "type" to "detailPrefetchRequested",
                    "contentType" to meta.type,
                    "id" to meta.id,
                    "streamLookupId" to plan.lookupId.ifBlank { detail?.id ?: meta.id },
                    "title" to meta.name,
                    "originalName" to meta.originalName,
                    "year" to meta.releaseInfo?.toIntOrNull(),
                    "language" to language,
                    "profile" to profile
                )
            )
        }
    }

    private suspend fun <T> fromStateList(value: Any?, type: java.lang.reflect.Type): List<T> {
        if (value == null) return emptyList()
        return withContext(Dispatchers.Default) {
            runCatching {
                gson.fromJson<List<T>>(gson.toJsonTree(value), type)
            }.getOrDefault(emptyList())
        }
    }

    private suspend fun <T> fromStateObject(value: Any?, clazz: Class<T>): T? {
        if (value == null) return null
        return withContext(Dispatchers.Default) {
            runCatching {
                gson.fromJson(gson.toJsonTree(value), clazz)
            }.onFailure {
                android.util.Log.e("HomeViewModel", "fromStateObject failed for ${clazz.simpleName}: $value", it)
            }.getOrNull()
        }
    }

    fun exchangeTraktCode(code: String, onProfileUpdated: (UserProfile) -> Unit, onComplete: (Boolean) -> Unit) =
        authCoordinator.exchangeCode("trakt", code, null, onProfileUpdated, onComplete)

    fun startTraktDeviceAuthorization(onCodeReady: (TraktDeviceCodeResponse) -> Unit, onProfileUpdated: (UserProfile) -> Unit, onComplete: (Boolean, String?) -> Unit) =
        authCoordinator.startTraktDeviceAuthorization(onCodeReady, onProfileUpdated, onComplete)

    fun exchangeMalCode(code: String, codeVerifier: String, onProfileUpdated: (UserProfile) -> Unit, onComplete: (Boolean) -> Unit) =
        authCoordinator.exchangeCode("mal", code, codeVerifier, onProfileUpdated, onComplete)

    fun exchangeSimklCode(code: String, onProfileUpdated: (UserProfile) -> Unit, onComplete: (Boolean) -> Unit) =
        authCoordinator.exchangeCode("simkl", code, null, onProfileUpdated, onComplete)

    fun exchangeAnilistCode(code: String, onProfileUpdated: (UserProfile) -> Unit, onComplete: (Boolean) -> Unit) =
        authCoordinator.exchangeCode("anilist", code, null, onProfileUpdated, onComplete)

}
