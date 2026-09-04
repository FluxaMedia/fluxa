package com.fluxa.app.ui.catalog

import com.fluxa.app.common.ReleaseDateUtils
import com.fluxa.app.data.local.*
import com.fluxa.app.data.remote.*
import com.fluxa.app.data.repository.*
import com.fluxa.app.data.repository.library.ThirdPartyProviderRepository
import com.fluxa.app.core.rust.FluxaAndroidHeadlessEnvironment
import com.fluxa.app.core.rust.NativeHeadlessEffect
import com.fluxa.app.core.rust.FluxaCoreNative
import com.fluxa.app.core.rust.FluxaCoreUniFfi
import com.fluxa.app.core.rust.FluxaUniFfiCoreStateHandle
import com.fluxa.app.core.rust.FluxaHeadlessRuntimeFactory
import com.fluxa.app.domain.discovery.DiscoverCatalogOption
import com.fluxa.app.domain.discovery.MetadataFeedOption
import com.fluxa.app.domain.playback.PlaybackSyncCoordinator
import com.fluxa.app.domain.discovery.buildCs3MetadataFeedOptions
import com.google.gson.Gson
import com.google.gson.reflect.TypeToken
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext

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
    private val providerAdapters: com.fluxa.app.data.repository.library.ProviderAdapters,
    private val thirdPartyProviderRepository: ThirdPartyProviderRepository,
    private val headlessEnvironment: FluxaAndroidHeadlessEnvironment,
    private val nuvioSyncCoordinator: NuvioSyncCoordinator,
    private val nuvioAccountImportCoordinator: NuvioAccountImportCoordinator,
    private val platformContentGateway: HomePlatformContentGateway,
    private val imdbApiService: ImdbApiService,
    private val playbackSyncCoordinator: PlaybackSyncCoordinator,
    internal val gson: Gson,
    @dagger.hilt.android.qualifiers.ApplicationContext context: android.content.Context
) : ViewModel() {

    private val appContext = context.applicationContext
    private val metaListType = object : TypeToken<List<Meta>>() {}.type
    private val categoryListType = object : TypeToken<List<HomeCategory>>() {}.type
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

    private data class CoreAction(val type: String, val value: Any?)

    private data class CoreStateSnapshot(val home: CoreHomeSnapshot = CoreHomeSnapshot())

    private data class CoreHomeSnapshot(
        val categories: List<HomeCategory> = emptyList(),
        val isLoading: Boolean = false,
        val currentFilter: String = "all",
        val isDirectLoading: Boolean = false,
        val traktContinueWatchingLastUpdatedAt: Long = 0L,
        val userAddons: List<AddonDescriptor> = emptyList(),
        val watchlist: List<Meta> = emptyList(),
        val likedItems: List<Meta> = emptyList(),
        val activeProfile: UserProfile? = null,
        val currentWatchlist: List<Meta> = emptyList(),
        val externalContinueWatching: List<Meta> = emptyList(),
        val traktWatchedState: TraktWatchedState = TraktWatchedState()
    )

    private fun dispatchHomeState(type: String, value: Any?): CoreHomeSnapshot? =
        runCatching {
            gson.fromJson(
                coreState.dispatch(CoreAction(type, value)),
                CoreStateSnapshot::class.java
            )?.home
        }.getOrNull()

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
    val billboardTrailerSubtitleCues: StateFlow<List<com.fluxa.app.shared.feature.player.TrailerCue>> = billboardState.trailerSubtitleCues
    val billboardSeasonPosterUrl: StateFlow<String?> = billboardState.seasonPosterUrl

    private val _isDirectLoading = MutableStateFlow(false)
    val isDirectLoading: StateFlow<Boolean> = _isDirectLoading

    private val _traktContinueWatchingLastUpdatedAt = MutableStateFlow(0L)
    val traktContinueWatchingLastUpdatedAt: StateFlow<Long> = _traktContinueWatchingLastUpdatedAt.asStateFlow()

    private val _userAddons = MutableStateFlow<List<AddonDescriptor>>(emptyList())
    val userAddons: StateFlow<List<AddonDescriptor>> = _userAddons

    private val parentsGuideCoordinator = HomeParentsGuideCoordinator(viewModelScope, imdbApiService)
    val parentsGuide: StateFlow<List<ParentsGuideCategory>> = parentsGuideCoordinator.state

    fun loadParentsGuide(metaId: String) = parentsGuideCoordinator.load(metaId)

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
        coordinatorFactory.library(viewModelScope, coreState, gson)
    }
    val libraryUiState: StateFlow<LibraryUiState> get() = libraryCoordinator.state

    fun loadLibraryItems(activeProfile: UserProfile?, force: Boolean = false) {
        libraryCoordinator.load(activeProfile, force)
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
    private val _connectErrors = MutableStateFlow<Map<String, String>>(emptyMap())
    val connectErrors: StateFlow<Map<String, String>> = _connectErrors.asStateFlow()

    fun clearConnectError(provider: String) {
        setConnectError(provider, null)
    }

    internal fun exchangeSimklPkceCode(
        code: String,
        codeVerifier: String,
        profile: UserProfile,
        onProfileUpdated: (UserProfile) -> Unit,
        onComplete: (Boolean) -> Unit,
    ) {
        viewModelScope.launch {
            val response = runCatching { repository.exchangeSimklCode(code, codeVerifier) }.getOrNull()
            if (response == null || response.accessToken.isBlank()) {
                onComplete(false)
                return@launch
            }
            val updated = profile.copy(
                simklAccessToken = response.accessToken,
                simklUsername = repository.getSimklUsername(response.accessToken),
                simklLastSyncAt = System.currentTimeMillis(),
            )
            if (currentActiveProfile?.id == profile.id) {
                setActiveProfileState(updated)
            }
            onProfileUpdated(updated)
            onComplete(true)
        }
    }

    private fun setConnectError(provider: String, error: String?) {
        _connectErrors.value = if (error.isNullOrBlank()) {
            _connectErrors.value - provider
        } else {
            _connectErrors.value + (provider to error)
        }
    }
    private var externalContinueWatching: List<Meta> = emptyList()
    private var traktWatchedState: TraktWatchedState = TraktWatchedState()
    private var currentActiveProfile: UserProfile? = null
    private val searchCoordinator by lazy {
        HomeSearchCoordinator(
            scope = viewModelScope,
            platformContentGateway = platformContentGateway,
            searchHistoryStore = searchHistoryStore,
            state = searchFocusState,
            activeProfile = { currentActiveProfile },
        )
    }
    val isSearchLoading: StateFlow<Boolean> get() = searchCoordinator.isLoading
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
            watchlistManager = watchlistManager,
            forgottenStore = forgottenContinueWatchingStore,
            playbackSyncCoordinator = playbackSyncCoordinator,
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

    private val headlessPlaybackCoordinator by lazy {
        HomeHeadlessPlaybackCoordinator(
            scope = viewModelScope,
            gson = gson,
            dispatch = ::dispatchHeadless,
            repository = repository,
            watchlistManager = watchlistManager,
            platformContentGateway = platformContentGateway,
            activeProfile = { currentActiveProfile },
            setDirectLoading = ::setDirectLoadingState,
            setWatchlist = ::setWatchlistState,
            loadLibraryData = ::loadLibraryData,
            refreshDynamicRows = ::refreshDynamicRows,
            billboardMovie = { billboardState.movieValue },
            setBillboardWatchlist = { billboardState.watchlistValue = it },
        )
    }

    private val externalPlaybackCoordinator by lazy {
        HomeExternalPlaybackCoordinator(
            scope = viewModelScope,
            context = appContext,
            activeProfile = { currentActiveProfile },
            playbackSyncCoordinator = playbackSyncCoordinator,
            saveProgress = { meta, position, duration, videoId, streamIndex, episodeName, streamUrl, streamTitle ->
                headlessPlaybackCoordinator.savePlaybackProgress(
                    meta = meta,
                    timeOffset = position,
                    duration = duration,
                    videoId = videoId,
                    streamIndex = streamIndex,
                    episodeName = episodeName,
                    lastStreamUrl = streamUrl,
                    lastStreamTitle = streamTitle,
                    scrobbleTraktPause = false
                )
            },
            markWatched = { meta, videoId, episodeName, duration ->
                headlessPlaybackCoordinator.markWatchedFromPlayback(
                    meta = meta,
                    videoId = videoId,
                    episodeName = episodeName,
                    watchedDuration = duration
                )
            }
        )
    }

    private val continueWatchingCoordinator by lazy {
        coordinatorFactory.continueWatching(
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

    private val bootstrapCoordinator by lazy {
        HomeBootstrapCoordinator(
            scope = viewModelScope,
            gson = gson,
            categoryListType = categoryListType,
            addonListType = addonListType,
            metaListType = metaListType,
            categoryState = categoryState,
            homeCategoryCache = homeCategoryCache,
            homeBillboardCache = homeBillboardCache,
            billboardState = billboardState,
            billboardRuntime = billboardRuntime,
            billboardLoader = billboardLoader,
            dispatchHeadless = ::dispatchHeadless,
            fetchExternalContinueWatching = continueWatchingCoordinator::fetchExternal,
            resetScrollState = ::resetHomeScrollState,
            setActiveProfile = ::setActiveProfileState,
            setCategories = ::setCategoriesState,
            setCategoriesAndCache = ::setCategoriesAndCache,
            setUserAddons = ::setUserAddonsState,
            setCurrentWatchlist = ::setCurrentWatchlistState,
            setWatchlist = ::setWatchlistState,
            setExternalContinueWatching = ::setExternalContinueWatchingState,
            refreshDynamicRows = ::refreshDynamicRows,
            scheduleCs3Refresh = ::scheduleCs3Refresh,
            setLoading = ::setLoadingState,
            setLoaded = { _hasLoadedHome.value = it },
            initialProfileId = currentActiveProfile?.id,
        )
    }

    private suspend fun dispatchHeadless(action: Any) = withContext(Dispatchers.Default) {
        headlessRuntime.dispatch(action)
    }

    private val feedCoordinator by lazy {
        val activityManager = appContext.getSystemService(android.app.ActivityManager::class.java)
        val isTelevision = appContext.packageManager.hasSystemFeature(android.content.pm.PackageManager.FEATURE_LEANBACK)
        val feedConcurrency = when {
            isTelevision && activityManager?.isLowRamDevice == true -> 3
            isTelevision -> 5
            else -> 8
        }
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
            currentCategories = categoryState::currentCategories,
            feedConcurrency = feedConcurrency
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

    internal val authCoordinator by lazy {
        HomeAuthCoordinator(
            scope = viewModelScope,
            gson = gson,
            dispatch = ::dispatchHeadless,
            activeProfile = { currentActiveProfile },
            updateActiveProfile = ::setActiveProfileState,
            invalidateHome = { setCategoriesState(emptyList()) }
        )
    }

    private val accountConnectionCoordinator by lazy {
        HomeAccountConnectionCoordinator(
            scope = viewModelScope,
            repository = repository,
            nuvioAccountImportCoordinator = nuvioAccountImportCoordinator,
            nuvioSyncCoordinator = nuvioSyncCoordinator,
            setProviderSyncing = ::setProviderSyncing,
            setConnectError = ::setConnectError,
            syncStremio = ::syncStremioIntegration,
            syncNuvio = ::syncNuvioIntegration,
        )
    }

    private val watchlistFlowBinder by lazy {
        HomeWatchlistFlowBinder(
            watchlistStore = watchlistStore,
            scope = viewModelScope,
            setWatchlist = ::setWatchlistState,
            setLocalContinueWatching = ::setCurrentWatchlistState,
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

    fun toggleWatchlist(meta: Meta) = headlessPlaybackCoordinator.toggleWatchlist(meta)

    fun addToWatchlist(meta: Meta) = headlessPlaybackCoordinator.addToWatchlist(meta)

    fun addProviderItemToLibrary(
        meta: Meta,
        providerId: ThirdPartyProviderId,
        providerAccountId: String?
    ) {
        val profile = currentActiveProfile ?: return
        viewModelScope.launch {
            thirdPartyProviderRepository.pushWatchlist(
                profile = profile,
                providerId = providerId,
                expectedAccountId = providerAccountId,
                item = meta,
                add = true
            )
            refreshProviderState(profile, providerId)
        }
    }

    fun markProviderItemWatched(
        meta: Meta,
        providerId: ThirdPartyProviderId,
        providerAccountId: String?
    ) {
        val profile = currentActiveProfile ?: return
        viewModelScope.launch {
            thirdPartyProviderRepository.pushWatched(
                profile = profile,
                providerId = providerId,
                expectedAccountId = providerAccountId,
                item = meta,
                watched = true
            )
            refreshProviderState(profile, providerId)
        }
    }

    fun dropProviderContinueWatching(
        meta: Meta,
        providerId: ThirdPartyProviderId,
        providerAccountId: String?
    ) {
        val profile = currentActiveProfile ?: return
        viewModelScope.launch {
            thirdPartyProviderRepository.removeContinueWatching(
                profile = profile,
                providerId = providerId,
                expectedAccountId = providerAccountId,
                item = meta
            )
            refreshProviderState(profile, providerId, refreshRemoteContinueWatching = false)
        }
    }

    fun clearProviderData(
        profile: UserProfile,
        providerId: ThirdPartyProviderId,
        onCleared: () -> Unit
    ) {
        viewModelScope.launch {
            thirdPartyProviderRepository.clear(profile, providerId)
            onCleared()
        }
    }

    private fun refreshProviderState(
        profile: UserProfile,
        providerId: ThirdPartyProviderId,
        refreshRemoteContinueWatching: Boolean = true
    ) {
        loadLibraryItems(profile, force = true)
        if (ThirdPartyProviderId.from(profile.safeContinueWatchingSource) == providerId) {
            if (refreshRemoteContinueWatching) {
                refreshExternalContinueWatching(profile)
            } else {
                viewModelScope.launch {
                    val cached = thirdPartyProviderRepository.cached(profile, providerId)
                        ?.continueWatching
                        .orEmpty()
                    setExternalContinueWatchingState(cached)
                    refreshDynamicRows()
                }
            }
        }
    }

    fun toggleBillboardWatchlist() = headlessPlaybackCoordinator.toggleBillboardWatchlist()

    fun setFeedback(movie: Meta, isLike: Boolean) =
        headlessPlaybackCoordinator.setFeedback(movie, isLike)

    fun startExternalPlaybackTracking(
        meta: Meta,
        videoId: String?,
        initialPositionMs: Long,
        initialDurationMs: Long = 0L,
        streamIndex: Int,
        episodeName: String?,
        streamUrl: String?,
        streamTitle: String?,
        targetPackage: String?,
    ) = externalPlaybackCoordinator.start(
        meta = meta,
        videoId = videoId,
        initialPositionMs = initialPositionMs,
        initialDurationMs = initialDurationMs,
        streamIndex = streamIndex,
        episodeName = episodeName,
        streamUrl = streamUrl,
        streamTitle = streamTitle,
        targetPackage = targetPackage,
    )

    fun finishExternalPlaybackTracking(
        returnedPositionMs: Long? = null,
        returnedDurationMs: Long? = null,
    ) = externalPlaybackCoordinator.finish(returnedPositionMs, returnedDurationMs)

    fun externalPlaybackMediaSessionAccessAvailable(): Boolean =
        externalPlaybackCoordinator.hasMediaSessionAccess()

    fun savePlaybackProgress(
        meta: Meta,
        timeOffset: Long,
        duration: Long,
        videoId: String? = null,
        streamIndex: Int? = null,
        episodeName: String? = null,
        lastStreamUrl: String? = null,
        lastStreamTitle: String? = null,
        lastBingeGroup: String? = null,
        lastAudioLanguage: String? = null,
        lastSubtitleLanguage: String? = null,
        scrobbleTraktPause: Boolean = true,
    ) = headlessPlaybackCoordinator.savePlaybackProgress(
        meta = meta,
        timeOffset = timeOffset,
        duration = duration,
        videoId = videoId,
        streamIndex = streamIndex,
        episodeName = episodeName,
        lastStreamUrl = lastStreamUrl,
        lastStreamTitle = lastStreamTitle,
        lastBingeGroup = lastBingeGroup,
        lastAudioLanguage = lastAudioLanguage,
        lastSubtitleLanguage = lastSubtitleLanguage,
        scrobbleTraktPause = scrobbleTraktPause,
    )

    fun scrobblePlayback(
        token: String,
        metaType: String,
        itemId: String,
        progress: Float,
        action: String,
    ) = headlessPlaybackCoordinator.scrobblePlayback(
        token = token,
        metaType = metaType,
        itemId = itemId,
        progress = progress,
        action = action,
    )

    internal fun enqueueDurableTraktScrobble(
        profile: UserProfile,
        mediaType: String,
        mediaId: String,
        progress: Float,
        action: String,
    ): Boolean = playbackSyncCoordinator.scheduleTraktScrobble(
        profile = profile,
        mediaType = mediaType,
        mediaId = mediaId,
        progress = progress,
        action = action,
    )

    internal fun enqueueDurableSimklScrobble(
        profile: UserProfile,
        meta: Meta,
        videoId: String?,
        action: String,
        positionMs: Long,
        durationMs: Long,
    ): Boolean = playbackSyncCoordinator.scheduleSimklScrobble(
        profile = profile,
        meta = meta,
        videoId = videoId,
        positionMs = positionMs,
        durationMs = durationMs,
        action = action,
    )

    fun onNextEpisodeCardShown(
        meta: Meta,
        nextVideoId: String,
        activeProfile: UserProfile?,
    ) = headlessPlaybackCoordinator.onNextEpisodeCardShown(meta, nextVideoId, activeProfile)

    fun markWatchedFromPlayback(
        meta: Meta,
        videoId: String? = null,
        episodeName: String? = null,
        nextEpisode: Video? = null,
        watchedDuration: Long = 0L,
    ) = headlessPlaybackCoordinator.markWatchedFromPlayback(
        meta = meta,
        videoId = videoId,
        episodeName = episodeName,
        nextEpisode = nextEpisode,
        watchedDuration = watchedDuration,
    )

    fun forgetPlaybackProgress(meta: Meta) =
        headlessPlaybackCoordinator.forgetPlaybackProgress(meta)

    suspend fun getStreams(type: String, id: String): List<Stream> =
        headlessPlaybackCoordinator.getStreams(type, id)

    internal suspend fun loadPlayerStreams(
        meta: Meta,
        currentVideoId: String?,
        initialVideoId: String?,
        initialStreams: List<Stream>,
        initialStreamIndex: Int,
        savedUrl: String?,
        savedTitle: String?,
        activeProfile: UserProfile?,
        preferredBingeGroup: String?,
    ): PlayerRuntimeCoreState = headlessPlaybackCoordinator.loadPlayerStreams(
        meta = meta,
        currentVideoId = currentVideoId,
        initialVideoId = initialVideoId,
        initialStreams = initialStreams,
        initialStreamIndex = initialStreamIndex,
        savedUrl = savedUrl,
        savedTitle = savedTitle,
        profileOverride = activeProfile,
        preferredBingeGroup = preferredBingeGroup,
    )

    internal suspend fun resolvePlayerPlayback(
        url: String,
        stream: Stream?,
        currentVideoId: String?,
        title: String,
    ): PlayerRuntimeCoreState = headlessPlaybackCoordinator.resolvePlayerPlayback(
        url = url,
        stream = stream,
        currentVideoId = currentVideoId,
        title = title,
    )

    suspend fun prepareDirectPlayback(meta: Meta): DirectPlaybackTarget? =
        headlessPlaybackCoordinator.prepareDirectPlayback(meta)

    suspend fun getSeasonEpisodes(
        id: String,
        seasonNumber: Int,
        language: String,
    ): List<Video> = headlessPlaybackCoordinator.getSeasonEpisodes(id, seasonNumber, language)

    suspend fun getSubtitlesFromAddon(
        baseUrl: String,
        type: String,
        id: String,
        extra: String = "",
    ): List<SubtitleData> = headlessPlaybackCoordinator.getSubtitlesFromAddon(
        baseUrl = baseUrl,
        type = type,
        id = id,
        extra = extra,
    )

    suspend fun getIntroSegments(
        imdbId: String,
        season: Int,
        episode: Int,
        title: String?,
        useIntroDb: Boolean,
        useAniSkip: Boolean,
    ): List<IntroTimestamps> = headlessPlaybackCoordinator.getIntroSegments(
        imdbId = imdbId,
        season = season,
        episode = episode,
        title = title,
        useIntroDb = useIntroDb,
        useAniSkip = useAniSkip,
    )

    suspend fun submitIntroSegment(
        apiKey: String,
        segmentType: String,
        imdbId: String,
        season: Int,
        episode: Int,
        startSec: Double,
        endSec: Double,
    ): IntroDbSubmitResult = headlessPlaybackCoordinator.submitIntroSegment(
        apiKey = apiKey,
        segmentType = segmentType,
        imdbId = imdbId,
        season = season,
        episode = episode,
        startSec = startSec,
        endSec = endSec,
    )

    suspend fun resolvePlaybackIntroImdbId(
        meta: Meta,
        videoId: String?,
        language: String,
    ): String? = headlessPlaybackCoordinator.resolvePlaybackIntroImdbId(meta, videoId, language)

    suspend fun loadTerminalRecommendations(meta: Meta, hasNextEpisode: Boolean, language: String): List<Meta> {
        if (hasNextEpisode) return emptyList()
        val completion = headlessEnvironment.execute(
            NativeHeadlessEffect(
                id = "terminal-recommendations-${meta.id}",
                type = "fetchDetailSecondary",
                payload = mapOf(
                    "contentType" to meta.type,
                    "id" to meta.id,
                    "language" to language,
                    "profile" to currentActiveProfile,
                ),
            ),
        )
        if (completion.status != "ok") return emptyList()
        val value = completion.value as? Map<*, *> ?: return emptyList()
        val candidates = runCatching {
            gson.fromJson<List<Meta>>(gson.toJson(value["similarItems"]), metaListType)
        }.getOrDefault(emptyList())
        val watchedIds = runCatching {
            gson.fromJson<List<String>>(gson.toJson(value["watchedVideoIds"]), object : TypeToken<List<String>>() {}.type)
        }.getOrDefault(emptyList())
        if (candidates.isEmpty()) return emptyList()
        val planJson = FluxaCoreUniFfi.coreInvokeValue(
            "terminalRecommendationPlan",
            gson.toJson(
                mapOf(
                    "current" to meta,
                    "candidates" to candidates,
                    "hasNextEpisode" to false,
                    "watchedIds" to watchedIds,
                ),
            ),
        )
        val plan = runCatching { gson.fromJson(planJson, Map::class.java) }.getOrNull() ?: return emptyList()
        if (plan["showRecommendations"] != true) return emptyList()
        return runCatching {
            gson.fromJson<List<Meta>>(gson.toJson(plan["items"]), metaListType)
        }.getOrDefault(emptyList())
    }

    private suspend fun getConfiguredMetaDetail(
        type: String,
        id: String,
        language: String,
    ): MetaDetail? = headlessPlaybackCoordinator.getConfiguredMetaDetail(type, id, language)

    private suspend fun getConfiguredMetaDetailResult(
        type: String,
        id: String,
        language: String,
    ): HomeMetaDetailResult =
        headlessPlaybackCoordinator.getConfiguredMetaDetailResult(type, id, language)

    suspend fun resolveExpandedPosterTrailer(meta: Meta): String? =
        headlessPlaybackCoordinator.resolveExpandedPosterTrailer(meta)

    fun search(query: String) = searchCoordinator.search(query)

    fun addToSearchHistory(meta: Meta) = searchCoordinator.addToHistory(meta)

    fun recordSearchSelection(id: String, type: String) =
        searchCoordinator.recordSelection(id, type)

    fun clearSearchHistory() = searchCoordinator.clearHistory()

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

    fun pauseHomeBackgroundWork() {
        billboardRuntime.pauseBackgroundWork()
        feedCoordinator.pauseRemainingCatalogs()
    }

    fun discover(type: String, catalogKey: String?, genre: String?, year: String?, rating: Float?, provider: String?, region: String?) {
        pauseHomeBackgroundWork()
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

    fun loadInitialData(activeProfile: UserProfile?, force: Boolean = false) =
        bootstrapCoordinator.load(activeProfile, force)

    private fun resetHomeScrollState() {
        savedHomeScrollIndex = 0
        savedHomeScrollOffset = 0
        savedTvHomeScrollIndex = 0
        savedTvHomeScrollOffset = 0
        savedTvFocusedRowIndex = -1
        savedCategoryScrollPositions.clear()
    }

    fun refreshTraktTokenIfNeeded(profile: UserProfile, onProfileUpdated: (UserProfile) -> Unit) {
        authCoordinator.refreshTokenIfNeeded("trakt", profile, onProfileUpdated)
    }

    fun refreshExternalContinueWatching(activeProfile: UserProfile? = currentActiveProfile) {
        val profile = activeProfile ?: return
        viewModelScope.launch {
            val items = continueWatchingCoordinator.fetchExternal(profile)
            setExternalContinueWatchingState(items)
            refreshDynamicRows()
        }
    }

    fun loadLibraryData(activeProfile: UserProfile?) {
        syncCoordinator.loadLibrary(activeProfile)
    }

    fun syncThirdPartyProvider(
        profile: UserProfile,
        providerId: ThirdPartyProviderId,
        onProfileUpdated: (UserProfile) -> Unit,
        onComplete: (Boolean) -> Unit
    ) {
        val providerKey = providerId.key
        val expectedAccountId = profile.providerAccountId(providerId)
        if (expectedAccountId == null || !profile.isProviderConnected(providerId)) {
            setConnectError(providerKey, "${providerId.displayName} is not connected")
            onComplete(false)
            return
        }

        setProviderSyncing(providerKey, true)
        clearConnectError(providerKey)
        viewModelScope.launch {
            var success = false
            try {
                val snapshot = thirdPartyProviderRepository.load(profile, providerId, refresh = true)
                val current = currentActiveProfile?.takeIf { it.id == profile.id } ?: profile
                val ownerStillMatches = current.isProviderConnected(providerId) &&
                    current.providerAccountId(providerId) == expectedAccountId
                if (
                    snapshot == null ||
                    snapshot.fromCache ||
                    snapshot.accountId != expectedAccountId ||
                    !ownerStillMatches
                ) {
                    setConnectError(providerKey, "${providerId.displayName} sync did not return fresh account data")
                    return@launch
                }

                val updated = current.withProviderLastSyncAt(providerId, snapshot.syncedAt)
                setActiveProfileState(updated)
                onProfileUpdated(updated)
                loadLibraryItems(updated, force = true)
                if (ThirdPartyProviderId.from(updated.safeContinueWatchingSource) == providerId) {
                    refreshExternalContinueWatching(updated)
                }
                success = true
            } catch (error: Exception) {
                setConnectError(providerKey, error.message ?: "${providerId.displayName} sync failed")
            } finally {
                setProviderSyncing(providerKey, false)
                onComplete(success)
            }
        }
    }

    fun syncTraktIntegration(
        profile: UserProfile,
        onProfileUpdated: (UserProfile) -> Unit,
        onComplete: (Boolean) -> Unit
    ) {
        syncCoordinator.syncTrakt(profile, onProfileUpdated, onComplete) { updated ->
            loadLibraryItems(updated, force = true)
        }
    }

    fun syncNuvioIntegration(
        profile: UserProfile,
        onProfileUpdated: (UserProfile) -> Unit,
        onComplete: (Boolean) -> Unit
    ) {
        syncCoordinator.syncNuvio(profile, onProfileUpdated, onComplete) { updated ->
            loadLibraryItems(updated, force = true)
        }
    }

    suspend fun isNuvioHealthy(): Boolean = accountConnectionCoordinator.isNuvioHealthy()


    fun syncStremioIntegration(
        profile: UserProfile,
        onProfileUpdated: (UserProfile) -> Unit,
        onComplete: (Boolean) -> Unit
    ) {
        syncCoordinator.syncStremio(profile, onProfileUpdated, onComplete) { updated ->
            loadLibraryItems(updated, force = true)
        }
    }

    fun connectNuvioWithCredentials(
        email: String,
        password: String,
        profile: UserProfile,
        onProfileUpdated: (UserProfile) -> Unit,
        onComplete: (Boolean) -> Unit,
    ) = accountConnectionCoordinator.connectNuvio(
        email,
        password,
        profile,
        onProfileUpdated,
        onComplete,
    )

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
        categoryState.setCategories(dispatchHomeState("setHomeCategories", categories)?.categories ?: categories)
    }

    private fun setLoadingState(isLoading: Boolean) {
        _isLoading.value = dispatchHomeState("setHomeLoading", isLoading)?.isLoading ?: isLoading
    }

    private fun setCurrentFilterState(filter: String) {
        val normalized = filter.takeIf { it.isNotEmpty() } ?: "all"
        _currentFilter.value = dispatchHomeState("setHomeCurrentFilter", normalized)?.currentFilter ?: normalized
    }

    private fun setDirectLoadingState(isLoading: Boolean) {
        _isDirectLoading.value = dispatchHomeState("setHomeDirectLoading", isLoading)?.isDirectLoading ?: isLoading
    }

    private fun setTraktUpdatedAtState(updatedAt: Long) {
        _traktContinueWatchingLastUpdatedAt.value = dispatchHomeState(
            "setTraktContinueWatchingLastUpdatedAt",
            updatedAt
        )?.traktContinueWatchingLastUpdatedAt ?: updatedAt
    }

    private fun setUserAddonsState(addons: List<AddonDescriptor>) {
        _userAddons.value = dispatchHomeState("setUserAddons", addons)?.userAddons ?: addons
    }

    private fun setWatchlistState(items: List<Meta>) {
        _watchlist.value = dispatchHomeState("setWatchlist", items)?.watchlist ?: items
    }

    private fun setLikedItemsState(items: List<Meta>) {
        _likedItems.value = dispatchHomeState("setLikedItems", items)?.likedItems ?: items
    }

    private fun setActiveProfileState(profile: UserProfile?) {
        if (profile == null && currentActiveProfile != null) return
        val home = dispatchHomeState("setActiveProfile", profile)
        currentActiveProfile = home?.activeProfile ?: profile
        watchlistManager.setActiveProfile(profile?.id.orEmpty())
    }

    private fun setCurrentWatchlistState(items: List<Meta>) {
        val projected = dispatchHomeState("setCurrentWatchlist", items)?.currentWatchlist ?: items
        currentWatchlist = projected
        _currentContinueWatchingCount.value = projected.size
    }

    private fun setExternalContinueWatchingState(items: List<Meta>) {
        externalContinueWatching = dispatchHomeState("setExternalContinueWatching", items)?.externalContinueWatching ?: items
    }

    private fun setTraktWatchedState(state: TraktWatchedState) {
        traktWatchedState = dispatchHomeState("setTraktWatchedState", state)?.traktWatchedState ?: state
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

}
