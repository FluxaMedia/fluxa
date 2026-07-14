package com.fluxa.app.ui.catalog

import com.fluxa.app.common.ReleaseDateUtils
import com.fluxa.app.data.local.*
import com.fluxa.app.data.remote.*
import com.fluxa.app.data.repository.*
import com.fluxa.app.core.rust.FluxaAndroidHeadlessEnvironment
import com.fluxa.app.core.rust.FluxaCoreNative
import com.fluxa.app.core.rust.FluxaCoreStateHandle
import com.fluxa.app.core.rust.FluxaHeadlessRuntimeFactory
import com.fluxa.app.domain.discovery.DiscoverCatalogOption
import com.fluxa.app.domain.discovery.Cs3CatalogFeedDescriptor
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
import com.lagradost.cloudstream3.MainAPI
import kotlinx.coroutines.async
import kotlinx.coroutines.awaitAll
import kotlinx.coroutines.coroutineScope
import kotlinx.coroutines.sync.Semaphore
import kotlinx.coroutines.sync.withPermit
import dagger.hilt.android.lifecycle.HiltViewModel
import javax.inject.Inject

@HiltViewModel
class HomeViewModel @Inject constructor(
    private val repository: StremioRepository,
    private val traktRepository: TraktRepository,
    private val addonRepository: AddonRepository,
    private val watchlistManager: WatchlistManager,
    private val searchHistoryStore: SearchHistoryStore,
    private val homeCategoryCache: HomeCategoryCache,
    private val forgottenContinueWatchingStore: ForgottenContinueWatchingStore,
    private val coordinatorFactory: HomeViewModelCoordinatorFactory,
    private val headlessEnvironment: FluxaAndroidHeadlessEnvironment,
    private val pluginManager: PluginManager,
    private val cloudStreamCatalogClient: CloudStreamCatalogClient,
    private val imdbApiService: ImdbApiService,
    private val gson: Gson,
    @dagger.hilt.android.qualifiers.ApplicationContext context: android.content.Context
) : ViewModel() {

    private val appContext = context.applicationContext
    private val metaListType = object : TypeToken<List<Meta>>() {}.type
    private val streamListType = object : TypeToken<List<Stream>>() {}.type
    private val trailerListType = object : TypeToken<List<DetailTrailer>>() {}.type
    private val categoryListType = object : TypeToken<List<HomeCategory>>() {}.type
    private val calendarItemListType = object : TypeToken<List<CalendarUpcomingItem>>() {}.type
    private val videoListType = object : TypeToken<List<Video>>() {}.type
    private val subtitleListType = object : TypeToken<List<SubtitleData>>() {}.type
    private val introTimestampsListType = object : TypeToken<List<IntroTimestamps>>() {}.type
    private val discoverCatalogListType = object : TypeToken<List<DiscoverCatalogOption>>() {}.type
    private val discoverGenreListType = object : TypeToken<List<DiscoverGenreOption>>() {}.type
    private val discoverContentTypeListType = object : TypeToken<List<String>>() {}.type
    private val addonListType = object : TypeToken<List<AddonDescriptor>>() {}.type
    private val headlessRuntime = FluxaHeadlessRuntimeFactory.createUniFfi(headlessEnvironment)
    private val initialSearchHistory = searchHistoryStore.load(null)
    private val coreState: FluxaCoreStateHandle = FluxaCoreNative.createAppCoreState(
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
    private val _categories = MutableStateFlow<List<HomeCategory>>(emptyList())
    val categories: StateFlow<List<HomeCategory>> = _categories
    private val _collectionFolderCategories = MutableStateFlow<Map<String, HomeCategory>>(emptyMap())
    val collectionFolderCategories: StateFlow<Map<String, HomeCategory>> = _collectionFolderCategories.asStateFlow()

    var savedHomeScrollIndex: Int = 0
    var savedHomeScrollOffset: Int = 0
    var savedTvHomeScrollIndex: Int = 0
    var savedTvHomeScrollOffset: Int = 0
    var savedTvFocusedRowIndex: Int = -1
    val savedCategoryScrollPositions: HashMap<String, Pair<Int, Int>> = HashMap()
    private var categoriesRenderSignature: Int? = null

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

    private val _headlessDiscoverResults = MutableStateFlow<List<Meta>>(emptyList())
    private val _headlessDiscoverResultSources = MutableStateFlow<Map<String, HomeCatalogSource>>(emptyMap())
    private val _headlessDiscoverLoading = MutableStateFlow(false)
    private val _headlessDiscoverGenres = MutableStateFlow<List<DiscoverGenreOption>>(emptyList())
    private val _headlessDiscoverCatalogs = MutableStateFlow<List<DiscoverCatalogOption>>(emptyList())
    private val _headlessDiscoverContentTypes = MutableStateFlow<List<String>>(emptyList())
    private val discoverResultSourceMapType = object : TypeToken<Map<String, HomeCatalogSource>>() {}.type

    val discoverUiState: StateFlow<DiscoverUiState> = combine(
        combine(
            _headlessDiscoverResults,
            _headlessDiscoverResultSources,
            _headlessDiscoverLoading
        ) { results, resultSources, loading -> Triple(results, resultSources, loading) },
        _headlessDiscoverGenres,
        _headlessDiscoverCatalogs,
        _headlessDiscoverContentTypes
    ) { (results, resultSources, loading), genres, catalogs, contentTypes ->
        DiscoverUiState(results, loading, genres, catalogs, contentTypes, resultSources)
    }.stateIn(viewModelScope, SharingStarted.WhileSubscribed(5_000), DiscoverUiState())

    val discoverGenres: StateFlow<List<DiscoverGenreOption>> get() = _headlessDiscoverGenres.asStateFlow()

    private val _headlessCalendarItems = MutableStateFlow<List<CalendarUpcomingItem>>(emptyList())
    private val _headlessCalendarLoading = MutableStateFlow(false)

    val calendarUiState: StateFlow<CalendarUiState> = combine(
        _headlessCalendarItems,
        _headlessCalendarLoading
    ) { items, loading -> CalendarUiState(items, loading) }
        .stateIn(viewModelScope, SharingStarted.WhileSubscribed(5_000), CalendarUiState())

    private val billboardState = HomeBillboardStateHolder()
    val billboardError: StateFlow<String?> = billboardState.error
    val billboardPool: StateFlow<List<Meta>> = billboardState.pool
    val billboardIndex: StateFlow<Int> = billboardState.index
    val billboardMovie: StateFlow<Meta?> = billboardState.movie
    val billboardLogo: StateFlow<String?> = billboardState.logo
    val billboardWatchlist: StateFlow<Boolean> = billboardState.watchlist
    val billboardNextEpisode: StateFlow<String?> = billboardState.nextEpisode
    val billboardTrailerUrl: StateFlow<String?> = billboardState.trailerUrl
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
    val totalWatchedContentDuration: StateFlow<Long> = watchlistManager
        .getTotalWatchedContentDurationFlow()
        .stateIn(viewModelScope, SharingStarted.WhileSubscribed(5_000), 0L)

    private val libraryCoordinator by lazy {
        coordinatorFactory.library(repository, traktRepository, viewModelScope, coreState, gson)
    }
    val libraryUiState: StateFlow<LibraryUiState> get() = libraryCoordinator.state

    fun loadLibraryItems(activeProfile: UserProfile?) {
        libraryCoordinator.load(activeProfile)
    }

    val loadedCs3ApiNames: StateFlow<List<String>> = pluginManager.loadedApis
        .map { apis -> apis.map { it.name } }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val loadedCs3CatalogFeedOptions: StateFlow<List<MetadataFeedOption>> = pluginManager.loadedApis
        .map { apis -> buildCs3MetadataFeedOptions(apis.toCs3CatalogFeedDescriptors()) }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    private var currentWatchlist: List<Meta> = emptyList()
    private var externalContinueWatching: List<Meta> = emptyList()
    private var traktWatchedState: TraktWatchedState = TraktWatchedState()
    private var currentActiveProfile: UserProfile? = null
    private var searchJob: Job? = null
    private val _isSearchLoading = MutableStateFlow(false)
    val isSearchLoading: StateFlow<Boolean> = _isSearchLoading.asStateFlow()
    private val loadMoreInFlight = mutableSetOf<String>()
    private val playbackController by lazy {
        coordinatorFactory.playback(
            context = appContext,
            repository = repository,
            watchlistManager = watchlistManager,
            forgottenStore = forgottenContinueWatchingStore,
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
            categories = { _categories.value },
            language = { currentActiveProfile?.safeLanguage ?: "en" },
            setMovie = { billboardState.movieValue = it },
            setLogo = { billboardState.logoValue = it },
            watchlistValue = { billboardState.watchlistValue },
            setWatchlist = { billboardState.watchlistValue = it },
            setTrailerUrl = { billboardState.trailerUrlValue = it },
            setNextEpisode = { billboardState.nextEpisodeValue = it },
            setSeasonPosterUrl = { billboardState.seasonPosterUrlValue = it },
            getMetaDetail = { type, id ->
                val profile = currentActiveProfile
                addonRepository.getAddonMetaDetail(type, id, profile?.authKey ?: "", profile?.safeLocalAddons)
            },
            parseSeasonEpisode = ::formatSeasonEpisode,
            prefetchDirectPlayback = ::prefetchDirectPlayback,
            activeProfile = { currentActiveProfile },
            getTrailers = { type, id, lang -> getConfiguredMetaDetailResult(type, id, lang).trailers }
        )
    }

    private val billboardLoader by lazy {
        HomeBillboardLoader(
            addonRepository = addonRepository,
            scope = viewModelScope,
            getMetadataFeeds = { profile -> feedCoordinator.getMetadataFeeds(profile) },
            getCs3MetadataFeeds = { loadedCs3CatalogFeedOptions.value },
            fetchCs3FeedItems = { feed ->
                cloudStreamCatalogClient.fetchFeedItems(pluginManager.loadedApis.value, feed.key)
            },
            setPool = { billboardState.poolValue = it },
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
            normalizeCatalogItems = ::normalizeCatalogItems,
            setCategories = ::setCategoriesState,
            currentCategories = { _categories.value }
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
            categories = { _categories.value },
            setCategories = ::setCategoriesAndCache,
            activeProfile = { currentActiveProfile },
            buildUserCollectionHomeCategories = ::buildUserCollectionHomeCategories,
            buildContinueWatchingItems = ::buildContinueWatchingItems,
            optimizeHomeCategories = ::optimizeHomeCategories
        )
    }

    private val watchlistFlowBinder by lazy {
        HomeWatchlistFlowBinder(
            watchlistManager = watchlistManager,
            scope = viewModelScope,
            setWatchlist = ::setWatchlistState,
            setLocalContinueWatching = ::setCurrentWatchlistState,
            setExternalContinueWatching = ::setExternalContinueWatchingState,
            setLikedItems = ::setLikedItemsState,
            refreshDynamicRows = ::refreshDynamicRows,
            prefetchContinueWatchingArtwork = ::prefetchContinueWatchingArtwork
        )
    }

    init {
        watchlistFlowBinder.bind()
        observeCloudStreamPlugins()
    }

    private var cs3FetchJob: Job? = null

    private fun scheduleCs3Refresh() {
        cs3FetchJob?.cancel()
        cs3FetchJob = viewModelScope.launch {
            try {
                _hasLoadedHome.first { it }
                val profile = currentActiveProfile
                val homeFeedToggles = profile?.homeFeedToggles
                val cs3FeedsConfigured = profile?.cs3FeedsConfigured == true
                val apis = pluginManager.loadedApis.value
                    .filter { it.hasMainPage }
                val cs3FeedOptions = buildCs3MetadataFeedOptions(apis.toCs3CatalogFeedDescriptors())
                val enabledCs3FeedKeys = if (homeFeedToggles == null || !cs3FeedsConfigured) {
                    null
                } else if (homeFeedToggles.any { it.startsWith("cs3_catalog_") }) {
                    effectiveHomeMetadataFeedSelection(homeFeedToggles, cs3FeedOptions.map { it.key })?.toSet()
                } else if (homeFeedToggles.any { it.startsWith("cs3_plugin_") }) {
                    homeFeedToggles.toSet()
                } else {
                    null
                }
                android.util.Log.d("HomeViewModel", "CS3 refresh: ${apis.size} APIs: ${apis.map { it.name }}")
                if (apis.isEmpty()) {
                    val withoutCs3 = _categories.value.filter { !it.id.startsWith("cs3_") }
                    if (withoutCs3.size != _categories.value.size) setCategoriesAndCache(withoutCs3)
                    return@launch
                }
                val iconsByApiName = pluginManager.installedPlugins.value
                    .filter { !it.iconUrl.isNullOrBlank() }
                    .associate { it.name to it.iconUrl!! }
                val cs3Rows = cloudStreamCatalogClient.fetchHomeCatalogCategories(
                    apis = apis,
                    iconsByApiName = iconsByApiName,
                    enabledFeedKeys = enabledCs3FeedKeys
                )
                android.util.Log.d("HomeViewModel", "CS3 refresh: got ${cs3Rows.size} rows from ${apis.size} APIs")
                if (!isActive) return@launch
                val cs3RowsById = cs3Rows.associateBy { it.id }
                val currentCategories = _categories.value
                val existingCs3Ids = currentCategories.filter { it.id.startsWith("cs3_") }.map { it.id }.toSet()
                val merged = currentCategories.mapNotNull { cat ->
                    if (cat.id.startsWith("cs3_")) cs3RowsById[cat.id] else cat
                }
                val newCs3Rows = cs3Rows.filter { it.id !in existingCs3Ids }
                setCategoriesAndCache(merged + newCs3Rows)
            } catch (t: Throwable) {
                android.util.Log.w("HomeViewModel", "CS3 refresh failed", t)
            }
        }
    }

    private fun observeCloudStreamPlugins() {
        viewModelScope.launch {
            pluginManager.loadedApis
                .map { apis -> apis.toCs3CatalogFeedDescriptors().map { "${it.pluginName}:${it.catalogIndex}:${it.catalogName}" }.toSet() }
                .distinctUntilChanged()
                .collect {
                    scheduleCs3Refresh()
                    if (billboardState.poolValue.isEmpty()) {
                        viewModelScope.launch {
                            billboardLoader.load(currentActiveProfile)
                        }
                    }
                }
        }
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
            val result = dispatchHeadless(mapOf("type" to "toggleWatchlistRequested", "item" to meta))
            val library = result.state["library"] as? Map<*, *>
            val write = library?.get("lastWrite") as? Map<*, *>
            setWatchlistState(fromStateList(write?.get("watchlist"), metaListType))
            refreshDynamicRows()
        }
    }

    fun toggleBillboardWatchlist() {
        val movie = billboardState.movieValue ?: return
        viewModelScope.launch {
            val result = dispatchHeadless(mapOf("type" to "toggleWatchlistRequested", "item" to movie))
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
                "sourceSelectionMode" to (profile?.safeStreamSourceSelectionMode ?: STREAM_SOURCE_MODE_MANUAL),
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
                repository.getTmdbTrailers(type, id, language, profile.safeTmdbApiKey)
            }.getOrElse { emptyList() }
        } else {
            emptyList()
        }
        return HomeMetaDetailResult(detail = detail, trailers = tmdbTrailers)
    }

    suspend fun resolveExpandedPosterTrailer(meta: Meta): String? {
        val lang = currentActiveProfile?.safeLanguage ?: "en"
        val trailers = runCatching { getConfiguredMetaDetailResult(meta.type, meta.id, lang).trailers }.getOrElse { emptyList() }
        return resolvePlayableTrailerUrl(trailers)
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
                val rows = addonRepository.searchRows(
                    query = trimmedQuery,
                    language = currentActiveProfile?.safeLanguage ?: "en",
                    authKey = currentActiveProfile?.authKey.orEmpty(),
                    localAddons = currentActiveProfile?.safeLocalAddons.orEmpty()
                )
                if (!isActive) return@launch
                val cs3Rows = cloudStreamCatalogClient.searchRows(pluginManager.loadedApis.value, trimmedQuery)
                if (!isActive) return@launch
                val allRows = rows + cs3Rows
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
        val category = _categories.value.firstOrNull { it.id == categoryId }
            ?: _collectionFolderCategories.value[categoryId]
            ?: return
        if (!category.canLoadMore || !loadMoreInFlight.add(categoryId)) return
        viewModelScope.launch {
            try {
                val catalogSources = category.catalogSources.orEmpty()
                val remoteSources = category.remoteSources.orEmpty()
                if (catalogSources.isNotEmpty() || remoteSources.isNotEmpty()) {
                    val nextSkip = if (category.items.isEmpty()) 0 else category.skip + 20
                    val lang = currentActiveProfile?.safeLanguage ?: "en"
                    val semaphore = Semaphore(permits = 4)
                    val addonItems = coroutineScope {
                        catalogSources.map { source ->
                            async(Dispatchers.IO) {
                                semaphore.withPermit {
                                    normalizeCatalogItems(
                                        addonRepository.getAddonCatalog(
                                            source.transportUrl,
                                            source.type,
                                            source.catalogId,
                                            skip = nextSkip,
                                            genre = source.genre
                                        ),
                                        source.catalogId,
                                        lang,
                                        source.genre
                                    )
                                }
                            }
                        }.awaitAll().flatten()
                    }
                    val remoteItems = if (remoteSources.isEmpty()) {
                        emptyList()
                    } else {
                        val result = dispatchHeadless(
                            mapOf(
                                "type" to "catalogPageRequested",
                                "categoryId" to categoryId,
                                "transportUrl" to null,
                                "contentType" to category.type,
                                "catalogId" to category.catalogId,
                                "skip" to nextSkip,
                                "genre" to null,
                                "search" to null,
                                "remoteSource" to remoteSources,
                                "profile" to currentActiveProfile
                            )
                        )
                        val home = result.state["home"] as? Map<*, *> ?: emptyMap<Any, Any>()
                        val paging = home["paging"] as? Map<*, *> ?: emptyMap<Any, Any>()
                        fromStateList<Meta>(paging["items"], metaListType)
                    }
                    val newItems = addonItems + remoteItems
                    updateHomeCategory(categoryId) { existing ->
                        existing.copy(
                            items = if (newItems.isEmpty()) existing.items else (existing.items + newItems).distinctBy { "${it.type}:${it.id}" },
                            skip = nextSkip,
                            canLoadMore = newItems.isNotEmpty()
                        )
                    }
                    return@launch
                }
                val result = dispatchHeadless(
                    mapOf(
                        "type" to "catalogPageRequested",
                        "categoryId" to categoryId,
                        "transportUrl" to category.addonTransportUrl,
                        "contentType" to category.type,
                        "catalogId" to category.catalogId,
                        "skip" to (category.skip + category.items.size),
                        "genre" to category.addonGenre,
                        "search" to null,
                        "remoteSource" to remoteSources,
                        "profile" to currentActiveProfile
                    )
                )
                val home = result.state["home"] as? Map<*, *> ?: return@launch
                val paging = home["paging"] as? Map<*, *> ?: return@launch
                val newItems = fromStateList<Meta>(paging["items"], metaListType)
                updateHomeCategory(categoryId) { existing ->
                    existing.copy(
                        items = if (newItems.isEmpty()) existing.items else (existing.items + newItems).distinctBy { "${it.type}:${it.id}" },
                        canLoadMore = newItems.isNotEmpty()
                    )
                }
            } finally {
                loadMoreInFlight.remove(categoryId)
            }
        }
    }

    private fun updateHomeCategory(categoryId: String, update: (HomeCategory) -> HomeCategory) {
        val hiddenCategory = _collectionFolderCategories.value[categoryId]
        if (hiddenCategory != null) {
            _collectionFolderCategories.value = _collectionFolderCategories.value + (categoryId to update(hiddenCategory))
            return
        }
        setCategoriesState(
            _categories.value.map { existing ->
                if (existing.id == categoryId) {
                    update(existing)
                } else {
                    existing
                }
            }
        )
    }

    fun clearDiscoverResults() {
        _headlessDiscoverResults.value = emptyList()
    }

    fun discover(type: String, catalogKey: String?, genre: String?, year: String?, rating: Float?, provider: String?, region: String?) {
        viewModelScope.launch {
            _headlessDiscoverLoading.value = true
            try {
                val result = dispatchHeadless(
                    mapOf(
                        "type" to "discoverRequested",
                        "contentType" to type,
                        "filters" to mapOf(
                            "catalogKey" to catalogKey,
                            "genre" to genre,
                            "year" to year,
                            "rating" to rating,
                            "provider" to provider,
                            "region" to region
                        ),
                        "profile" to currentActiveProfile,
                        "language" to (currentActiveProfile?.safeLanguage ?: "en")
                    )
                )
                val discover = result.state["discover"] as? Map<*, *>
                _headlessDiscoverResults.value = fromStateList(discover?.get("results"), metaListType)
                _headlessDiscoverResultSources.value = fromStateSourceMap(discover?.get("resultSources"))
            } finally {
                _headlessDiscoverLoading.value = false
            }
        }
    }

    fun loadMoreDiscoverResults(transportUrl: String, contentType: String, catalogId: String, genre: String?) {
        if (_headlessDiscoverLoading.value) return
        viewModelScope.launch {
            _headlessDiscoverLoading.value = true
            try {
                val result = dispatchHeadless(
                    mapOf(
                        "type" to "discoverPageRequested",
                        "transportUrl" to transportUrl,
                        "contentType" to contentType,
                        "catalogId" to catalogId,
                        "skip" to _headlessDiscoverResults.value.size,
                        "genre" to genre
                    )
                )
                val discover = result.state["discover"] as? Map<*, *>
                val updatedResults = fromStateList<Meta>(discover?.get("results"), metaListType)
                val source = HomeCatalogSource(transportUrl, catalogId, contentType, genre)
                val updatedSources = _headlessDiscoverResultSources.value.toMutableMap()
                updatedResults.forEach { item ->
                    updatedSources.putIfAbsent("${item.type}:${item.id}", source)
                    updatedSources.putIfAbsent(item.id, source)
                }
                _headlessDiscoverResults.value = updatedResults
                _headlessDiscoverResultSources.value = updatedSources
            } finally {
                _headlessDiscoverLoading.value = false
            }
        }
    }

    fun loadCalendarMonth(activeProfile: UserProfile?, year: Int, month: Int, plannedItems: List<Meta> = emptyList()) {
        viewModelScope.launch {
            _headlessCalendarLoading.value = true
            try {
                val result = dispatchHeadless(
                    mapOf(
                        "type" to "calendarMonthRequested",
                        "profile" to activeProfile,
                        "year" to year,
                        "month" to month,
                        "plannedItems" to plannedItems
                    )
                )
                val calendar = result.state["calendar"] as? Map<*, *>
                _headlessCalendarItems.value = fromStateList(calendar?.get("items"), calendarItemListType)
            } finally {
                _headlessCalendarLoading.value = false
            }
        }
    }

    fun loadDiscoverGenres(type: String) {
        _headlessDiscoverGenres.value = emptyList()
    }

    fun loadDiscoverCatalogFilters(type: String, selectedCatalogKey: String?) {
        viewModelScope.launch {
            val result = dispatchHeadless(
                mapOf(
                    "type" to "discoverCatalogFiltersRequested",
                    "contentType" to type,
                    "selectedCatalogKey" to selectedCatalogKey,
                    "profile" to currentActiveProfile,
                    "language" to (currentActiveProfile?.safeLanguage ?: "en")
                )
            )
            val discover = result.state["discover"] as? Map<*, *> ?: return@launch
            _headlessDiscoverCatalogs.value = fromStateList(discover["catalogs"], discoverCatalogListType)
            _headlessDiscoverGenres.value = fromStateList(discover["genres"], discoverGenreListType)
            _headlessDiscoverContentTypes.value = fromStateList(discover["contentTypes"], discoverContentTypeListType)
        }
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
        if (_categories.value.isEmpty()) {
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
        if (force || billboardState.poolValue.isEmpty()) {
            viewModelScope.launch {
                billboardLoader.load(activeProfile)
            }
        }
    }

    fun refreshTraktTokenIfNeeded(profile: UserProfile, onProfileUpdated: (UserProfile) -> Unit) {
        refreshAuthTokenIfNeeded("trakt", profile, onProfileUpdated)
    }

    fun refreshMalTokenIfNeeded(profile: UserProfile, onProfileUpdated: (UserProfile) -> Unit) {
        refreshAuthTokenIfNeeded("mal", profile, onProfileUpdated)
    }

    private fun refreshAuthTokenIfNeeded(provider: String, profile: UserProfile, onProfileUpdated: (UserProfile) -> Unit) {
        viewModelScope.launch {
            val result = dispatchHeadless(
                mapOf(
                    "type" to "authRefreshRequested",
                    "provider" to provider,
                    "profile" to profile
                )
            )
            val auth = result.state["auth"] as? Map<*, *>
            val updated = fromStateObject(auth?.get("result")?.let { (it as? Map<*, *>)?.get("profile") } ?: auth?.get("result"), UserProfile::class.java)
            if (updated != null && updated != profile) {
                setActiveProfileState(updated)
                onProfileUpdated(updated)
            }
        }
    }

    fun refreshExternalContinueWatching() {
        currentActiveProfile?.let(::loadLibraryData)
    }

    fun loadLibraryData(activeProfile: UserProfile?) {
        viewModelScope.launch {
            val result = dispatchHeadless(
                mapOf(
                    "type" to "libraryHydrateRequested",
                    "profileId" to activeProfile?.id
                )
            )
            val library = result.state["library"] as? Map<*, *> ?: return@launch
            setWatchlistState(fromStateList(library["watchlist"], metaListType))
            setCurrentWatchlistState(fromStateList(library["continueWatching"], metaListType))
            setLikedItemsState(fromStateList(library["liked"], metaListType))
            refreshDynamicRows()
        }
    }

    fun syncTraktIntegration(
        profile: UserProfile,
        onProfileUpdated: (UserProfile) -> Unit,
        onComplete: (Boolean) -> Unit
    ) {
        viewModelScope.launch {
            val result = dispatchHeadless(
                mapOf(
                    "type" to "externalIntegrationSyncRequested",
                    "provider" to "trakt",
                    "profile" to profile,
                    "language" to profile.safeLanguage
                )
            )
            val sync = result.state["sync"] as? Map<*, *>
            val error = sync?.get("error")
            val updated = fromStateObject((sync?.get("snapshot") as? Map<*, *>)?.get("profile") ?: (result.state["profile"] as? Map<*, *>)?.get("active"), UserProfile::class.java)
            if (updated != null) {
                setActiveProfileState(updated)
                onProfileUpdated(updated)
                setExternalContinueWatchingState(fromStateList((result.state["home"] as? Map<*, *>)?.get("externalContinueWatching"), metaListType))
                refreshDynamicRows()
            }
            onComplete(error == null && updated != null)
        }
    }

    fun syncNuvioIntegration(
        profile: UserProfile,
        onProfileUpdated: (UserProfile) -> Unit,
        onComplete: (Boolean) -> Unit
    ) {
        viewModelScope.launch {
            val result = dispatchHeadless(
                mapOf(
                    "type" to "externalSyncRequested",
                    "provider" to "nuvio",
                    "profile" to profile,
                    "language" to profile.safeLanguage
                )
            )
            val sync = result.state["sync"] as? Map<*, *>
            val updated = fromStateObject(
                (sync?.get("snapshot") as? Map<*, *>)?.get("profile"),
                UserProfile::class.java
            )
            if (updated != null) {
                setActiveProfileState(updated)
                onProfileUpdated(updated)
            }
            onComplete(sync?.get("error") == null)
        }
    }

    fun syncStremioIntegration(
        profile: UserProfile,
        onProfileUpdated: (UserProfile) -> Unit,
        onComplete: (Boolean) -> Unit
    ) {
        viewModelScope.launch {
            val result = dispatchHeadless(
                mapOf(
                    "type" to "externalIntegrationSyncRequested",
                    "provider" to "stremio",
                    "profile" to profile,
                    "language" to profile.safeLanguage
                )
            )
            val sync = result.state["sync"] as? Map<*, *>
            val snapshot = sync?.get("snapshot") as? Map<*, *>
            val updated = fromStateObject(snapshot?.get("profile") ?: (result.state["profile"] as? Map<*, *>)?.get("active"), UserProfile::class.java)
            if (updated != null) {
                setActiveProfileState(updated)
                setExternalContinueWatchingState(fromStateList((result.state["home"] as? Map<*, *>)?.get("externalContinueWatching"), metaListType))
                onProfileUpdated(updated)
                refreshDynamicRows()
            }
            onComplete(sync?.get("error") == null && updated != null)
        }
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
            homeCategoryCache.save(profile, categories)
        }
    }

    private fun setCategoriesState(categories: List<HomeCategory>) {
        val hiddenCollectionFolders = categories.filter { it.isCollectionFolderCategory() }
        val visibleCategories = categories.filterNot { it.isCollectionFolderCategory() || it.type == "collection" }
        when {
            categories.isEmpty() -> _collectionFolderCategories.value = emptyMap()
            hiddenCollectionFolders.isNotEmpty() || categories.any { it.type == "collection" } -> {
                _collectionFolderCategories.value = hiddenCollectionFolders.associateBy { it.id }
            }
        }
        val signature = visibleCategories.renderSignatureForHome()
        if (categoriesRenderSignature == signature) return
        categoriesRenderSignature = signature
        _categories.value = visibleCategories
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
        if (addons.isEmpty() && _userAddons.value.isNotEmpty()) return
        _userAddons.value = addons
    }

    private fun setWatchlistState(items: List<Meta>) {
        _watchlist.value = items
    }

    private fun setLikedItemsState(items: List<Meta>) {
        _likedItems.value = items
    }

    private fun setActiveProfileState(profile: UserProfile?) {
        currentActiveProfile = profile
    }

    private fun setCurrentWatchlistState(items: List<Meta>) {
        currentWatchlist = items
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

    private suspend fun fromStateSourceMap(value: Any?): Map<String, HomeCatalogSource> {
        return withContext(Dispatchers.Default) {
            runCatching {
                gson.fromJson<Map<String, HomeCatalogSource>>(gson.toJsonTree(value), discoverResultSourceMapType)
            }.getOrNull() ?: emptyMap()
        }
    }

    fun exchangeTraktCode(code: String, onProfileUpdated: (UserProfile) -> Unit, onComplete: (Boolean) -> Unit) {
        exchangeAuthCode("trakt", code, null, onProfileUpdated, onComplete)
    }

    fun startTraktDeviceAuthorization(
        onCodeReady: (TraktDeviceCodeResponse) -> Unit,
        onProfileUpdated: (UserProfile) -> Unit,
        onComplete: (Boolean, String?) -> Unit
    ) {
        viewModelScope.launch {
            val profile = currentActiveProfile
            if (profile == null) {
                onComplete(false, "toast.trakt_connect_failed")
                return@launch
            }
            val codeResult = dispatchHeadless(
                mapOf(
                    "type" to "authFlowRequested",
                    "provider" to "trakt",
                    "mode" to "deviceCode"
                )
            )
            val codeResponse = fromStateObject((codeResult.state["auth"] as? Map<*, *>)?.get("result"), TraktDeviceCodeResponse::class.java)
            if (codeResponse == null) {
                onComplete(false, "toast.trakt_connect_failed")
                return@launch
            }
            onCodeReady(codeResponse)
            val startedAt = System.currentTimeMillis()
            val expiresAt = startedAt + codeResponse.expiresIn * 1000L
            var intervalMs = codeResponse.interval.coerceAtLeast(5) * 1000L
            var failureMessageKey: String? = "toast.trakt_connect_failed"
            while (System.currentTimeMillis() < expiresAt) {
                delay(intervalMs)
                val tokenResult = dispatchHeadless(
                    mapOf(
                        "type" to "authExchangeRequested",
                        "provider" to "traktDevice",
                        "code" to codeResponse.deviceCode,
                        "profile" to (currentActiveProfile ?: profile)
                    )
                )
                val auth = tokenResult.state["auth"] as? Map<*, *>
                val result = auth?.get("result") as? Map<*, *>
                val updated = fromStateObject(result?.get("profile"), UserProfile::class.java)
                if (updated != null) {
                    setActiveProfileState(updated)
                    onProfileUpdated(updated)
                    setCategoriesState(emptyList())
                    onComplete(true, null)
                    return@launch
                }
                val errorCode = result?.get("errorCode") as? String
                val httpCode = (result?.get("httpCode") as? Number)?.toInt()
                when {
                    errorCode == "slow_down" || httpCode == 429 -> {
                        val retryAfterMs = (result.get("retryAfterSeconds") as? Number)?.toLong()?.let { it * 1000L }
                        intervalMs = (retryAfterMs ?: (intervalMs + 5_000L)).coerceAtMost(60_000L)
                    }
                    errorCode == "expired_token" || errorCode == "invalid_grant" -> {
                        failureMessageKey = "toast.trakt_device_code_expired"
                        break
                    }
                    errorCode == "authorization_pending" || httpCode in setOf(400, 404, 409, 428) -> {
                        continue
                    }
                    else -> break
                }
            }
            if (System.currentTimeMillis() >= expiresAt) {
                failureMessageKey = "toast.trakt_device_code_expired"
            }
            onComplete(false, failureMessageKey)
        }
    }

    fun exchangeMalCode(code: String, codeVerifier: String, onProfileUpdated: (UserProfile) -> Unit, onComplete: (Boolean) -> Unit) {
        exchangeAuthCode("mal", code, codeVerifier, onProfileUpdated, onComplete)
    }

    fun exchangeSimklCode(code: String, onProfileUpdated: (UserProfile) -> Unit, onComplete: (Boolean) -> Unit) {
        exchangeAuthCode("simkl", code, null, onProfileUpdated, onComplete)
    }

    fun exchangeAnilistCode(code: String, onProfileUpdated: (UserProfile) -> Unit, onComplete: (Boolean) -> Unit) {
        exchangeAuthCode("anilist", code, null, onProfileUpdated, onComplete)
    }

    private fun exchangeAuthCode(
        provider: String,
        code: String,
        codeVerifier: String?,
        onProfileUpdated: (UserProfile) -> Unit,
        onComplete: (Boolean) -> Unit
    ) {
        viewModelScope.launch {
            val profile = currentActiveProfile
            if (profile == null) {
                onComplete(false)
                return@launch
            }
            val result = dispatchHeadless(
                mapOf(
                    "type" to "authExchangeRequested",
                    "provider" to provider,
                    "code" to code,
                    "codeVerifier" to codeVerifier,
                    "profile" to profile
                )
            )
            val updated = fromStateObject((result.state["profile"] as? Map<*, *>)?.get("active"), UserProfile::class.java)
            if (updated != null) {
                setActiveProfileState(updated)
                onProfileUpdated(updated)
                setCategoriesState(emptyList())
                onComplete(true)
            } else {
                onComplete(false)
            }
        }
    }

}

private fun List<MainAPI>.toCs3CatalogFeedDescriptors(): List<Cs3CatalogFeedDescriptor> {
    return filter { it.hasMainPage }.flatMap { api ->
        api.mainPage.mapIndexed { index, page ->
            Cs3CatalogFeedDescriptor(
                pluginName = api.name,
                catalogName = page.name.takeIf { it.isNotBlank() } ?: api.name,
                catalogIndex = index
            )
        }
    }
}

private fun List<HomeCategory>.renderSignatureForHome(): Int {
    var result = size
    for (category in this) {
        result = 31 * result + category.id.hashCode()
        result = 31 * result + category.name.hashCode()
        result = 31 * result + category.type.hashCode()
        result = 31 * result + (category.addonIconUrl?.hashCode() ?: 0)
        result = 31 * result + category.items.size
        result = 31 * result + category.skip
        result = 31 * result + category.canLoadMore.hashCode()
        if (category.type == "collection_folder") {
            continue
        }
        val visibleCount = minOf(category.items.size, 24)
        for (index in 0 until visibleCount) {
            val item = category.items[index]
            result = 31 * result + item.id.hashCode()
            result = 31 * result + item.type.hashCode()
            result = 31 * result + item.name.hashCode()
            result = 31 * result + (item.poster?.hashCode() ?: 0)
            result = 31 * result + (item.background?.hashCode() ?: 0)
            result = 31 * result + (item.logo?.hashCode() ?: 0)
            result = 31 * result + (item.releaseInfo?.hashCode() ?: 0)
            result = 31 * result + (item.reason?.hashCode() ?: 0)
            result = 31 * result + (item.timeOffset?.hashCode() ?: 0)
            result = 31 * result + (item.duration?.hashCode() ?: 0)
            result = 31 * result + (item.lastVideoId?.hashCode() ?: 0)
            result = 31 * result + (item.lastEpisodeName?.hashCode() ?: 0)
            result = 31 * result + (item.continueWatchingPoster?.hashCode() ?: 0)
            result = 31 * result + (item.continueWatchingBackground?.hashCode() ?: 0)
        }
    }
    return result
}

private fun HomeCategory.isCollectionFolderCategory(): Boolean {
    return type == "collection_folder"
}

private fun Video.continueWatchingTitleForHome(): String {
    val seasonEpisode = if (season != null && number != null) "S$season, E$number" else null
    val title = name?.trim()?.takeIf { it.isNotBlank() }
    return when {
        seasonEpisode != null && title != null -> "$seasonEpisode: $title"
        seasonEpisode != null -> seasonEpisode
        else -> title.orEmpty()
    }
}
