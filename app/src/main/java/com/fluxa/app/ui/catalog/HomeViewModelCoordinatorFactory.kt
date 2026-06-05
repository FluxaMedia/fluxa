package com.fluxa.app.ui.catalog

import android.content.Context
import com.fluxa.app.core.rust.FluxaCoreStateHandle
import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.data.local.WatchlistManager
import com.fluxa.app.data.remote.AddonDescriptor
import com.fluxa.app.data.remote.Meta
import com.fluxa.app.data.repository.AddonRepository
import com.fluxa.app.data.repository.StremioRepository
import com.fluxa.app.data.repository.TraktRepository
import com.fluxa.app.data.repository.TraktWatchedState
import com.fluxa.app.domain.discovery.DiscoverCatalogContentLoader
import com.fluxa.app.domain.discovery.StreamDiscoveryUseCase
import com.google.gson.Gson
import kotlinx.coroutines.CoroutineScope
import javax.inject.Inject

class HomeViewModelCoordinatorFactory @Inject constructor() {
    internal fun search(addonRepository: AddonRepository, searchHistoryStore: SearchHistoryStore): HomeSearchCoordinator {
        return HomeSearchCoordinator(addonRepository, searchHistoryStore)
    }

    internal fun episodeCalendar(repository: StremioRepository, watchlistManager: WatchlistManager): EpisodeCalendarLoader {
        return EpisodeCalendarLoader(repository, watchlistManager)
    }

    internal fun library(
        repository: StremioRepository,
        traktRepository: TraktRepository,
        scope: CoroutineScope,
        coreState: FluxaCoreStateHandle,
        gson: Gson
    ): HomeLibraryCoordinator {
        return HomeLibraryCoordinator(repository, traktRepository, scope, coreState, gson)
    }

    internal fun auth(
        repository: StremioRepository,
        scope: CoroutineScope,
        activeProfile: () -> UserProfile?,
        invalidateHome: () -> Unit
    ): HomeAuthCoordinator {
        return HomeAuthCoordinator(repository, scope, activeProfile, invalidateHome)
    }

    internal fun discover(
        addonRepository: AddonRepository,
        discoverCatalogContentLoader: DiscoverCatalogContentLoader,
        scope: CoroutineScope,
        activeProfile: () -> UserProfile?,
        userAddons: () -> List<AddonDescriptor>,
        setUserAddons: (List<AddonDescriptor>) -> Unit,
        normalizeCatalogItems: suspend (List<Meta>, String, String, String?) -> List<Meta>,
        coreState: FluxaCoreStateHandle
    ): HomeDiscoverController {
        return HomeDiscoverController(
            addonRepository = addonRepository,
            discoverCatalogContentLoader = discoverCatalogContentLoader,
            scope = scope,
            activeProfile = activeProfile,
            userAddons = userAddons,
            setUserAddons = setUserAddons,
            normalizeCatalogItems = normalizeCatalogItems,
            coreState = coreState
        )
    }

    internal fun calendar(
        context: Context,
        episodeCalendarLoader: EpisodeCalendarLoader,
        watchlistManager: WatchlistManager,
        scope: CoroutineScope,
        activeProfile: () -> UserProfile?,
        setActiveProfile: (UserProfile?) -> Unit,
        onSnapshotLoaded: (HomeCalendarSnapshot) -> Unit,
        coreState: FluxaCoreStateHandle
    ): HomeCalendarController {
        return HomeCalendarController(
            context = context,
            episodeCalendarLoader = episodeCalendarLoader,
            watchlistManager = watchlistManager,
            scope = scope,
            activeProfile = activeProfile,
            setActiveProfile = setActiveProfile,
            onSnapshotLoaded = onSnapshotLoaded,
            coreState = coreState
        )
    }

    internal fun playback(
        context: Context,
        repository: StremioRepository,
        watchlistManager: WatchlistManager,
        forgottenStore: ForgottenContinueWatchingStore,
        scope: CoroutineScope,
        activeProfile: () -> UserProfile?,
        localContinueWatching: () -> List<Meta>,
        externalContinueWatching: () -> List<Meta>,
        onContinueWatchingChanged: (ContinueWatchingSnapshot) -> Unit,
        refreshDynamicRows: () -> Unit
    ): HomePlaybackController {
        return HomePlaybackController(
            context = context,
            repository = repository,
            watchlistManager = watchlistManager,
            forgottenStore = forgottenStore,
            scope = scope,
            activeProfile = activeProfile,
            localContinueWatching = localContinueWatching,
            externalContinueWatching = externalContinueWatching,
            onContinueWatchingChanged = onContinueWatchingChanged,
            refreshDynamicRows = refreshDynamicRows
        )
    }

    internal fun continueWatching(
        repository: StremioRepository,
        watchlistManager: WatchlistManager,
        scope: CoroutineScope,
        activeProfile: () -> UserProfile?,
        localItems: () -> List<Meta>,
        externalItems: () -> List<Meta>,
        watchedState: () -> TraktWatchedState,
        setLocalItems: (List<Meta>) -> Unit,
        setExternalItems: (List<Meta>) -> Unit,
        setWatchlistState: (List<Meta>) -> Unit,
        setTraktUpdatedAt: (Long) -> Unit,
        refreshDynamicRows: () -> Unit,
        getConfiguredMetaDetail: suspend (String, String, String) -> com.fluxa.app.data.remote.MetaDetail?,
        getSeasonEpisodes: suspend (String, Int, String) -> List<com.fluxa.app.data.remote.Video>
    ): HomeContinueWatchingCoordinator {
        return HomeContinueWatchingCoordinator(
            repository = repository,
            watchlistManager = watchlistManager,
            scope = scope,
            activeProfile = activeProfile,
            localItems = localItems,
            externalItems = externalItems,
            watchedState = watchedState,
            setLocalItems = setLocalItems,
            setExternalItems = setExternalItems,
            setWatchlistState = setWatchlistState,
            setTraktUpdatedAt = setTraktUpdatedAt,
            refreshDynamicRows = refreshDynamicRows,
            getConfiguredMetaDetail = getConfiguredMetaDetail,
            getSeasonEpisodes = getSeasonEpisodes
        )
    }

    internal fun playbackStreams(
        repository: StremioRepository,
        streamDiscovery: StreamDiscoveryUseCase,
        activeProfile: () -> UserProfile?,
        userAddons: () -> List<AddonDescriptor>,
        setDirectLoading: (Boolean) -> Unit
    ): HomePlaybackStreamCoordinator {
        return HomePlaybackStreamCoordinator(repository, streamDiscovery, activeProfile, userAddons, setDirectLoading)
    }
}
