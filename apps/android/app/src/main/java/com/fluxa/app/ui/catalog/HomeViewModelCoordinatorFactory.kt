package com.fluxa.app.ui.catalog

import com.fluxa.app.core.rust.FluxaUniFfiCoreStateHandle
import com.fluxa.app.data.local.UserProfile
import com.fluxa.app.data.local.WatchlistManager
import com.fluxa.app.data.remote.AddonDescriptor
import com.fluxa.app.data.remote.Meta
import com.fluxa.app.data.repository.StremioRepository
import com.fluxa.app.data.repository.TraktWatchedState
import com.fluxa.app.data.repository.library.ProviderContinueWatchingRepository
import com.fluxa.app.data.repository.library.ThirdPartyProviderRepository
import com.fluxa.app.domain.discovery.StreamDiscoveryUseCase
import com.google.gson.Gson
import com.fluxa.app.domain.playback.PlaybackProgressScheduler
import com.fluxa.app.domain.playback.PlaybackSyncCoordinator
import kotlinx.coroutines.CoroutineScope
import javax.inject.Inject

class HomeViewModelCoordinatorFactory @Inject constructor(
    private val playbackProgressScheduler: PlaybackProgressScheduler,
    private val providerContinueWatchingRepository: ProviderContinueWatchingRepository,
    private val thirdPartyProviderRepository: ThirdPartyProviderRepository,
) {
    internal fun library(
        scope: CoroutineScope,
        coreState: FluxaUniFfiCoreStateHandle,
        gson: Gson
    ): HomeLibraryCoordinator {
        return HomeLibraryCoordinator(
            providerRepository = thirdPartyProviderRepository,
            scope = scope,
            coreState = coreState,
            gson = gson
        )
    }

    internal fun playback(
        watchlistManager: WatchlistManager,
        forgottenStore: ForgottenContinueWatchingStore,
        playbackSyncCoordinator: PlaybackSyncCoordinator,
        scope: CoroutineScope,
        activeProfile: () -> UserProfile?,
        localContinueWatching: () -> List<Meta>,
        externalContinueWatching: () -> List<Meta>,
        onContinueWatchingChanged: (ContinueWatchingSnapshot) -> Unit,
        refreshDynamicRows: () -> Unit
    ): HomePlaybackController {
        return HomePlaybackController(
            watchlistManager = watchlistManager,
            forgottenStore = forgottenStore,
            playbackSyncCoordinator = playbackSyncCoordinator,
            scope = scope,
            activeProfile = activeProfile,
            localContinueWatching = localContinueWatching,
            externalContinueWatching = externalContinueWatching,
            onContinueWatchingChanged = onContinueWatchingChanged,
            refreshDynamicRows = refreshDynamicRows
        )
    }

    internal fun continueWatching(
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
            providerContinueWatchingRepository = providerContinueWatchingRepository,
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
