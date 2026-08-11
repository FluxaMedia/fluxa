package com.fluxa.app.shared

import androidx.compose.runtime.Composable
import androidx.compose.ui.window.ComposeUIViewController
import com.fluxa.app.core.apple.AppleCatalogHomeSnapshot
import com.fluxa.app.core.apple.AppleSearchSnapshot
import com.fluxa.app.core.apple.AppleDiscoverRequestSnapshot
import com.fluxa.app.core.apple.AppleDiscoverSnapshot
import com.fluxa.app.shared.platform.AppleCalendarSnapshot
import com.fluxa.app.shared.platform.AppleAuthSubmitSnapshot
import com.fluxa.app.shared.platform.AppleAuthSnapshot
import com.fluxa.app.shared.platform.AppleLibrarySnapshot
import com.fluxa.app.core.apple.AppleDetailRequestSnapshot
import com.fluxa.app.core.apple.AppleDetailSeasonRequestSnapshot
import com.fluxa.app.core.apple.AppleDetailSnapshot
import com.fluxa.app.core.apple.AppleDetailStreamsRequestSnapshot
import com.fluxa.app.core.apple.ApplePlaybackRequestSnapshot
import com.fluxa.app.shared.platform.AppleAddonStoreActionSnapshot
import com.fluxa.app.shared.platform.AppleAddonStoreDataSource
import com.fluxa.app.shared.platform.AppleAddonStoreSnapshot
import com.fluxa.app.shared.platform.ApplePluginsActionSnapshot
import com.fluxa.app.shared.platform.ApplePluginsDataSource
import com.fluxa.app.shared.platform.ApplePluginsSnapshot
import com.fluxa.app.shared.platform.AppleAuthDataSource
import com.fluxa.app.shared.platform.AppleCatalogHomeDataSource
import com.fluxa.app.shared.platform.AppleCalendarDataSource
import com.fluxa.app.shared.platform.AppleDetailDataSource
import com.fluxa.app.shared.platform.AppleDiscoverDataSource
import com.fluxa.app.shared.platform.AppleFluxaPlatformServices
import com.fluxa.app.shared.platform.AppleLibraryDataSource
import com.fluxa.app.shared.platform.AppleProfileDataSource
import com.fluxa.app.shared.platform.AppleSearchDataSource
import com.fluxa.app.shared.platform.AppleSettingsDataSource
import com.fluxa.app.data.local.AppleWatchlistStore
import platform.UIKit.UIViewController

object FluxaApple {
    internal val watchlistStore = AppleWatchlistStore()
    internal val catalogHomeDataSource = AppleCatalogHomeDataSource()
    internal val detailDataSource = AppleDetailDataSource(watchlistStore)
    internal val searchDataSource = AppleSearchDataSource()
    internal val discoverDataSource = AppleDiscoverDataSource()
    internal val calendarDataSource = AppleCalendarDataSource()
    internal val libraryDataSource = AppleLibraryDataSource(watchlistStore)
    internal val profileDataSource = AppleProfileDataSource(watchlistStore::setActiveProfile)
    internal val settingsDataSource = AppleSettingsDataSource()
    internal val addonStoreDataSource = AppleAddonStoreDataSource()
    internal val pluginsDataSource = ApplePluginsDataSource()
    internal val authDataSource = AppleAuthDataSource()
    internal val platformServices = AppleFluxaPlatformServices(
        catalogHomeDataSource,
        detailDataSource,
        searchDataSource,
        discoverDataSource,
        calendarDataSource,
        libraryDataSource,
        profileDataSource,
        settingsDataSource,
        addonStoreDataSource,
        pluginsDataSource,
        authDataSource
    )
    private var onPlaybackRequested: (ApplePlaybackRequestSnapshot) -> Unit = {}

    fun rootViewController(): UIViewController = ComposeUIViewController {
        FluxaAppleApp()
    }

    fun setCatalogHomeRefreshHandler(handler: () -> Unit) {
        catalogHomeDataSource.setOnRefreshRequested(handler)
    }

    fun updateCatalogHome(snapshot: AppleCatalogHomeSnapshot) {
        catalogHomeDataSource.update(snapshot)
    }

    fun setSearchHandler(handler: (String) -> Unit) {
        searchDataSource.setOnSearchRequested(handler)
    }

    fun updateSearch(snapshot: AppleSearchSnapshot) {
        searchDataSource.update(snapshot)
    }

    fun setDiscoverHandler(handler: (AppleDiscoverRequestSnapshot) -> Unit) {
        discoverDataSource.setOnDiscoverRequested(handler)
    }

    fun updateDiscover(snapshot: AppleDiscoverSnapshot) {
        discoverDataSource.update(snapshot)
    }

    fun setCalendarMonthHandler(handler: (Int, Int) -> Unit) {
        calendarDataSource.setOnMonthRequested(handler)
    }

    fun updateCalendar(snapshot: AppleCalendarSnapshot) {
        calendarDataSource.update(snapshot)
    }

    fun setAuthSubmitHandler(handler: (AppleAuthSubmitSnapshot) -> Unit) {
        authDataSource.setOnSubmitRequested(handler)
    }

    fun updateAuth(snapshot: AppleAuthSnapshot) {
        authDataSource.update(snapshot)
    }

    fun setLibraryRefreshHandler(handler: () -> Unit) {
        libraryDataSource.setOnRefreshRequested(handler)
    }

    fun updateLibrary(snapshot: AppleLibrarySnapshot) {
        libraryDataSource.update(snapshot)
    }

    fun setAddonStoreActionHandler(handler: (AppleAddonStoreActionSnapshot) -> Unit) {
        addonStoreDataSource.setOnActionRequested(handler)
    }

    fun updateAddonStore(snapshot: AppleAddonStoreSnapshot) {
        addonStoreDataSource.update(snapshot)
    }

    fun setPluginsActionHandler(handler: (ApplePluginsActionSnapshot) -> Unit) {
        pluginsDataSource.setOnActionRequested(handler)
    }

    fun updatePlugins(snapshot: ApplePluginsSnapshot) {
        pluginsDataSource.update(snapshot)
    }

    fun setDetailHandlers(
        load: (AppleDetailRequestSnapshot) -> Unit,
        watchlist: (AppleDetailRequestSnapshot) -> Unit,
        season: (AppleDetailSeasonRequestSnapshot) -> Unit = {},
        streams: (AppleDetailStreamsRequestSnapshot) -> Unit = {},
        addonFilter: (String?) -> Unit = {},
        downloadEpisode: (String) -> Unit = {},
        downloadSeason: (AppleDetailSeasonRequestSnapshot) -> Unit = {}
    ) {
        detailDataSource.setHandlers(load, watchlist, season, streams, addonFilter, downloadEpisode, downloadSeason)
    }

    fun updateDetail(snapshot: AppleDetailSnapshot) {
        detailDataSource.update(snapshot)
    }

    fun setPlaybackHandler(handler: (ApplePlaybackRequestSnapshot) -> Unit) {
        onPlaybackRequested = handler
    }

    internal fun requestPlayback(snapshot: ApplePlaybackRequestSnapshot) {
        onPlaybackRequested(snapshot)
    }

    internal fun requestFirstAvailablePlayback(resumePositionMs: Long) {
        detailDataSource.firstPlaybackRequest(resumePositionMs)?.let(onPlaybackRequested)
    }






}

@Composable
private fun FluxaAppleApp() {
    FluxaAppHost(
        platformServices = FluxaApple.platformServices,
        callbacks = com.fluxa.app.shared.FluxaAppHostCallbacks(
            navigation = com.fluxa.app.shared.FluxaAppNavigationCallbacks(
                onDetailNavigationEvent = { event ->
                    if (event is com.fluxa.app.shared.feature.detail.DetailNavigationEvent.PlayStream) {
                        FluxaApple.requestPlayback(
                            FluxaApple.detailDataSource.playbackRequest(
                                stream = event.stream,
                                resumePositionMs = event.resumeProgress,
                                videoId = event.episodeId,
                            )
                        )
                    } else if (event is com.fluxa.app.shared.feature.detail.DetailNavigationEvent.SelectSources) {
                        FluxaApple.requestFirstAvailablePlayback(event.resumeProgress)
                    }
                },
            ),
        ),
    )
}
